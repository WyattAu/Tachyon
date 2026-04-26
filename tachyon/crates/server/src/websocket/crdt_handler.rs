// CRDT Sync Handler — y-websocket Binary Relay with Server-side Yrs State
//
// Implements a binary relay for y-websocket (Yjs) collaboration with
// server-side document state management using Yrs.
//
// The server:
// 1. Extracts the room name from the URL path (/ws/crdt/{room})
// 2. Joins the client to that room
// 3. Applies updates to a server-side Yrs document
// 4. Sends the current document state to newly connected clients
// 5. Forwards all binary messages to other clients in the same room
// 6. Removes the client on disconnect
//
// The y-websocket protocol uses binary messages where:
// - Byte 0 = message type (0=sync, 1=awareness, 2=awareness query, 3=broadcast)
// - Remaining bytes = lib0-encoded payload
//
// We relay awareness messages unchanged but track document state via Yrs.

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Path, State,
    },
    response::Response,
};
use futures_util::{SinkExt, StreamExt};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use tracing::{debug, info, warn};

use crate::crdt::CrdtDocumentManager;
use tachyon_core::types::crdt::{CollaborationInfo, CrdtDocumentState, SelectionRange};

// ============================================================================
// Connection Manager
// ============================================================================

/// A connected client in a relay room.
#[derive(Debug)]
#[allow(dead_code)]
struct ConnectedClient {
    client_id: String,
    room: String,
    send: tokio::sync::mpsc::UnboundedSender<Message>,
}

/// Broadcast event for the relay.
#[derive(Debug, Clone)]
#[allow(dead_code)]
enum RelayEvent {
    /// Binary message from a client to relay to others.
    Binary { room: String, sender: String, data: Vec<u8> },
    /// Selection update from a client to relay to others.
    Selection { room: String, sender: String, data: Vec<u8>, selection: SelectionRange },
    /// A client joined a room.
    Joined { room: String, client_id: String },
    /// A client left a room.
    Left { room: String, client_id: String },
}

/// CRDT connection manager — public API compatible with the old handler.
///
/// Tracks room membership and provides introspection methods.
/// The actual message relay is handled by per-connection tasks.
/// Includes a CrdtDocumentManager for server-side document state.
#[derive(Clone)]
pub struct CrdtConnectionManager {
    clients: Arc<RwLock<HashMap<String, ConnectedClient>>>,
    room_clients: Arc<RwLock<HashMap<String, Vec<String>>>>,
    /// Per-document CRDT state for the public API (get_collaboration_info).
    /// The relay itself does not use this for message routing.
    documents: Arc<RwLock<HashMap<String, CrdtDocumentState>>>,
    broadcast_tx: broadcast::Sender<RelayEvent>,
    /// Server-side Yrs document manager for applying and storing CRDT state.
    crdt_manager: Arc<CrdtDocumentManager>,
}

impl CrdtConnectionManager {
    pub fn new() -> Self {
        let (broadcast_tx, _) = broadcast::channel(256);
        Self {
            clients: Arc::new(RwLock::new(HashMap::new())),
            room_clients: Arc::new(RwLock::new(HashMap::new())),
            documents: Arc::new(RwLock::new(HashMap::new())),
            broadcast_tx,
            crdt_manager: Arc::new(CrdtDocumentManager::new()),
        }
    }

    /// Get a reference to the server-side CRDT document manager.
    pub fn crdt_manager(&self) -> &Arc<CrdtDocumentManager> {
        &self.crdt_manager
    }

    /// Number of currently connected clients.
    pub async fn client_count(&self) -> usize {
        self.clients.read().await.len()
    }

    /// Get collaboration info for a document room.
    pub async fn get_collaboration_info(&self, document_id: &str) -> Option<CollaborationInfo> {
        let room_clients = self.room_clients.read().await;
        let clients = room_clients.get(document_id)?;
        Some(CollaborationInfo {
            document_id: document_id.to_string(),
            active_users: Vec::new(), // Relay mode doesn't track individual users
            connection_count: clients.len(),
        })
    }

    /// Initialize a document room with persisted update log.
    /// Kept for backward compatibility — the relay doesn't use this
    /// for message routing, but the data is available for inspection.
    pub async fn init_document_from_persisted(
        &self,
        document_id: &str,
        update_log: Vec<u8>,
    ) {
        let mut docs = self.documents.write().await;
        docs.insert(
            document_id.to_string(),
            CrdtDocumentState::from_persisted(document_id, update_log),
        );
    }

    /// Get the update log for a document room.
    pub async fn get_document_update_log(&self, document_id: &str) -> Option<Vec<u8>> {
        let docs = self.documents.read().await;
        docs.get(document_id).map(|d| d.get_update_log().to_vec())
    }

    /// Subscribe to relay events (used by per-connection tasks).
    fn subscribe(&self) -> broadcast::Receiver<RelayEvent> {
        self.broadcast_tx.subscribe()
    }

    /// Register a client in a room.
    async fn join_room(&self, client_id: &str, room: &str) {
        let mut rooms = self.room_clients.write().await;
        rooms.entry(room.to_string()).or_default().push(client_id.to_string());
    }

    /// Remove a client from its room.
    async fn leave_room(&self, client_id: &str) -> Option<String> {
        let client = self.clients.write().await.remove(client_id)?;
        let mut rooms = self.room_clients.write().await;
        if let Some(clients) = rooms.get_mut(&client.room) {
            clients.retain(|id| id != client_id);
        }
        Some(client.room)
    }
}

impl Default for CrdtConnectionManager {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// WebSocket Handler
// ============================================================================

/// Handle WebSocket upgrade for CRDT collaboration.
///
/// The room name is extracted from the URL path: `/ws/crdt/{room}`.
/// y-websocket appends the room name to the base URL, so the full
/// WebSocket URL from the client is `ws://host/ws/crdt/{documentId}`.
pub async fn handle_crdt_websocket_upgrade(
    ws: WebSocketUpgrade,
    State(manager): State<CrdtConnectionManager>,
    Path(room): Path<String>,
) -> Response {
    ws.on_upgrade(move |socket| handle_crdt_socket(socket, manager, room))
}

/// Handle an individual CRDT WebSocket connection.
///
/// This is a simple relay: forward all binary messages to other clients
/// in the same room, and receive messages from other clients.
async fn handle_crdt_socket(socket: WebSocket, manager: CrdtConnectionManager, room: String) {
    let client_id = uuid::Uuid::new_v4().to_string();
    info!(client_id = %client_id, room = %room, "CRDT client connecting");

    // Split the WebSocket into sender and receiver.
    let (mut ws_sender, mut ws_receiver) = socket.split();

    // Store the client.
    {
        // We don't need the send channel for the relay architecture.
        // The send_task reads from broadcast and writes to ws_sender directly.
        let (_tx, _rx) = tokio::sync::mpsc::unbounded_channel::<Message>();
        let client = ConnectedClient {
            client_id: client_id.clone(),
            room: room.clone(),
            send: _tx,
        };
        manager.clients.write().await.insert(client_id.clone(), client);
    }
    manager.join_room(&client_id, &room).await;
    let _ = manager.broadcast_tx.send(RelayEvent::Joined {
        room: room.clone(),
        client_id: client_id.clone(),
    });

    // Subscribe to relay events.
    let mut relay_rx = manager.subscribe();

    // Send the current document state to the newly connected client.
    // This is the Yrs-encoded state vector + updates, which the client
    // uses to sync its local document.
    if let Ok(state) = manager.crdt_manager().get_state(&room) {
        if !state.is_empty() {
            let sync_msg = encode_sync_step1(&state);
            if let Err(e) = ws_sender.send(Message::Binary(sync_msg.into())).await {
                warn!(client_id = %client_id, error = %e, "Failed to send initial document state");
            }
        }
    }

    // Forward relay events to this client's WebSocket.
    let room_for_send = room.clone();
    let client_id_for_send = client_id.clone();
    let send_task = async move {
        loop {
            match relay_rx.recv().await {
                Ok(RelayEvent::Binary { room: event_room, sender, data }) => {
                    if event_room == room_for_send && sender != client_id_for_send {
                        if let Err(e) = ws_sender.send(Message::Binary(data.into())).await {
                            warn!(client_id = %client_id_for_send, error = %e, "Failed to send binary message");
                            break;
                        }
                    }
                }
                Ok(RelayEvent::Selection { room: event_room, sender, data, selection: _ }) => {
                    if event_room == room_for_send && sender != client_id_for_send {
                        if let Err(e) = ws_sender.send(Message::Binary(data.into())).await {
                            warn!(client_id = %client_id_for_send, error = %e, "Failed to send selection message");
                            break;
                        }
                    }
                }
                Ok(RelayEvent::Joined { room: event_room, .. }) => {
                    debug!(
                        client_id = %client_id_for_send,
                        event_room = %event_room,
                        same_room = event_room == room_for_send,
                        "Client joined"
                    );
                }
                Ok(RelayEvent::Left { room: event_room, .. }) => {
                    debug!(
                        client_id = %client_id_for_send,
                        event_room = %event_room,
                        "Client left room"
                    );
                }
                Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                    warn!(client_id = %client_id_for_send, lagged = n, "Broadcast receiver lagged");
                    break;
                }
                Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                    debug!(client_id = %client_id_for_send, "Broadcast channel closed");
                    break;
                }
            }
        }
    };

    // Read from the WebSocket and relay to other clients.
    let room_for_recv = room.clone();
    let client_id_for_recv = client_id.clone();
    let broadcast_tx = manager.broadcast_tx.clone();
    let crdt_manager = manager.crdt_manager().clone();
    let recv_task = async move {
        while let Some(msg) = ws_receiver.next().await {
            match msg {
                Ok(Message::Binary(data)) => {
                    let data_vec: Vec<u8> = data.to_vec();

                    if data_vec.is_empty() {
                        continue;
                    }

                    let msg_type = data_vec[0];
                    match msg_type {
                        0 => {
                            // Sync: apply update to server-side Yrs document, then broadcast.
                            if data_vec.len() > 1 {
                                if let Err(e) = crdt_manager.apply_update(&room_for_recv, &data_vec[1..]) {
                                    warn!(
                                        client_id = %client_id_for_recv,
                                        room = %room_for_recv,
                                        error = %e,
                                        "Failed to apply CRDT update"
                                    );
                                }
                            }
                            let _ = broadcast_tx.send(RelayEvent::Binary {
                                room: room_for_recv.clone(),
                                sender: client_id_for_recv.clone(),
                                data: data_vec,
                            });
                        }
                        1 => {
                            // Awareness: forward unchanged.
                            let _ = broadcast_tx.send(RelayEvent::Binary {
                                room: room_for_recv.clone(),
                                sender: client_id_for_recv.clone(),
                                data: data_vec,
                            });
                        }
                        2 => {
                            // Selection update: parse, validate, and broadcast.
                            if let Some(selection) = parse_selection_update(&data_vec) {
                                debug!(
                                    client_id = %client_id_for_recv,
                                    room = %room_for_recv,
                                    start = selection.start,
                                    end = selection.end,
                                    "Selection update"
                                );
                                let _ = broadcast_tx.send(RelayEvent::Selection {
                                    room: room_for_recv.clone(),
                                    sender: client_id_for_recv.clone(),
                                    data: data_vec,
                                    selection,
                                });
                            } else {
                                warn!(
                                    client_id = %client_id_for_recv,
                                    "Invalid selection update payload"
                                );
                            }
                        }
                        _ => {
                            // Unknown type: forward unchanged.
                            let _ = broadcast_tx.send(RelayEvent::Binary {
                                room: room_for_recv.clone(),
                                sender: client_id_for_recv.clone(),
                                data: data_vec,
                            });
                        }
                    }
                }
                Ok(Message::Close(_)) => {
                    debug!(client_id = %client_id_for_recv, "Client sent close frame");
                    break;
                }
                Ok(Message::Ping(_data)) => {
                    // y-websocket handles ping/pong via the WebSocket layer.
                    debug!(client_id = %client_id_for_recv, "Received ping");
                }
                Ok(Message::Pong(_)) => {
                    // Ignore pongs.
                }
                Ok(Message::Text(text)) => {
                    // y-websocket may occasionally send text messages.
                    // Forward them as binary (the protocol is binary).
                    let data_vec: Vec<u8> = text.as_bytes().to_vec();
                    if let Err(e) = broadcast_tx.send(RelayEvent::Binary {
                        room: room_for_recv.clone(),
                        sender: client_id_for_recv.clone(),
                        data: data_vec,
                    }) {
                        warn!(client_id = %client_id_for_recv, error = %e, "Failed to broadcast text message");
                    }
                }
                Err(e) => {
                    warn!(client_id = %client_id_for_recv, error = %e, "WebSocket error");
                    break;
                }
            }
        }
    };

    // Run both tasks concurrently. When either finishes, clean up.
    tokio::select! {
        _ = send_task => {
            debug!(client_id = %client_id, "Send task ended");
        }
        _ = recv_task => {
            debug!(client_id = %client_id, "Receive task ended");
        }
    }

    // Clean up: remove client and notify.
    if let Some(removed_room) = manager.leave_room(&client_id).await {
        info!(client_id = %client_id, room = %removed_room, "CRDT client disconnected");
        let _ = manager.broadcast_tx.send(RelayEvent::Left {
            room: removed_room,
            client_id: client_id.clone(),
        });
    }
}

// ============================================================================
// Helpers
// ============================================================================

/// Encode a Yrs sync step 1 message.
///
/// The y-websocket protocol uses:
/// - Byte 0: message type (0 = sync)
/// - Byte 1: sync step (1 = state request/response)
/// - Remaining: lib0-encoded payload
fn encode_sync_step1(update: &[u8]) -> Vec<u8> {
    let mut msg = Vec::with_capacity(2 + update.len());
    msg.push(0); // message type: sync
    msg.push(1); // sync step 1
    msg.extend_from_slice(update);
    msg
}

/// Parse a selection update from a binary message.
///
/// Expected format after the message type byte (0x02):
/// - Bytes 1-4: start position (u32, little-endian)
/// - Bytes 5-8: end position (u32, little-endian)
fn parse_selection_update(data: &[u8]) -> Option<SelectionRange> {
    if data.len() < 9 {
        return None;
    }
    let start = u32::from_le_bytes(data[1..5].try_into().ok()?);
    let end = u32::from_le_bytes(data[5..9].try_into().ok()?);
    Some(SelectionRange { start, end })
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_manager_new() {
        let manager = CrdtConnectionManager::new();
        assert_eq!(manager.client_count().await, 0);
    }

    #[tokio::test]
    async fn test_join_and_leave() {
        let manager = CrdtConnectionManager::new();
        manager.join_room("client1", "room1").await;
        manager.join_room("client2", "room1").await;
        manager.join_room("client3", "room2").await;

        let info = manager.get_collaboration_info("room1").await.unwrap();
        assert_eq!(info.connection_count, 2);

        let info = manager.get_collaboration_info("room2").await.unwrap();
        assert_eq!(info.connection_count, 1);

        let info = manager.get_collaboration_info("nonexistent").await;
        assert!(info.is_none());
    }

    #[tokio::test]
    async fn test_init_document_from_persisted() {
        let manager = CrdtConnectionManager::new();
        let update_log = vec![1, 2, 3, 4, 5];
        manager.init_document_from_persisted("doc1", update_log.clone()).await;

        let log = manager.get_document_update_log("doc1").await;
        assert_eq!(log, Some(update_log));
    }

    #[tokio::test]
    async fn test_client_count() {
        let manager = CrdtConnectionManager::new();
        assert_eq!(manager.client_count().await, 0);

        // Simulate adding clients
        let (tx1, _) = tokio::sync::mpsc::unbounded_channel();
        let (tx2, _) = tokio::sync::mpsc::unbounded_channel();

        manager.clients.write().await.insert("c1".to_string(), ConnectedClient {
            client_id: "c1".to_string(),
            room: "room1".to_string(),
            send: tx1,
        });
        manager.clients.write().await.insert("c2".to_string(), ConnectedClient {
            client_id: "c2".to_string(),
            room: "room1".to_string(),
            send: tx2,
        });

        assert_eq!(manager.client_count().await, 2);
    }

    #[test]
    fn test_parse_selection_update_valid() {
        let mut data = vec![0x02];
        data.extend_from_slice(&10u32.to_le_bytes());
        data.extend_from_slice(&20u32.to_le_bytes());
        let selection = parse_selection_update(&data).unwrap();
        assert_eq!(selection.start, 10);
        assert_eq!(selection.end, 20);
    }

    #[test]
    fn test_parse_selection_update_too_short() {
        let data = vec![0x02, 0x01, 0x00];
        assert!(parse_selection_update(&data).is_none());
    }

    #[test]
    fn test_parse_selection_update_empty() {
        let data: Vec<u8> = vec![];
        assert!(parse_selection_update(&data).is_none());
    }

    #[test]
    fn test_parse_selection_update_minimal() {
        let mut data = vec![0x02];
        data.extend_from_slice(&0u32.to_le_bytes());
        data.extend_from_slice(&0u32.to_le_bytes());
        let selection = parse_selection_update(&data).unwrap();
        assert_eq!(selection.start, 0);
        assert_eq!(selection.end, 0);
    }

    #[test]
    fn test_parse_selection_update_large_offsets() {
        let mut data = vec![0x02];
        data.extend_from_slice(&u32::MAX.to_le_bytes());
        data.extend_from_slice(&u32::MAX.to_le_bytes());
        let selection = parse_selection_update(&data).unwrap();
        assert_eq!(selection.start, u32::MAX);
        assert_eq!(selection.end, u32::MAX);
    }

    #[test]
    fn test_parse_selection_update_wrong_type() {
        let mut data = vec![0x00];
        data.extend_from_slice(&10u32.to_le_bytes());
        data.extend_from_slice(&20u32.to_le_bytes());
        let selection = parse_selection_update(&data);
        assert_eq!(selection.unwrap().start, 10);
    }

    #[test]
    fn test_encode_sync_step1() {
        let update = vec![0xAA, 0xBB, 0xCC];
        let msg = encode_sync_step1(&update);
        assert_eq!(msg[0], 0);
        assert_eq!(msg[1], 1);
        assert_eq!(&msg[2..], &[0xAA, 0xBB, 0xCC]);
    }
}
