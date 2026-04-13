// CRDT Sync Handler
//
// Yjs-compatible WebSocket handler for real-time collaboration.
// Routes binary Yjs updates between clients and persists merged state.
//
// The server does NOT run Yjs itself. It acts as a dumb relay:
// 1. Accumulates binary updates from all clients
// 2. Stores the merged update log per document
// 3. Sends the full update log to new clients joining a document
// 4. Broadcasts new updates to all other clients in the room
//
// Yjs CRDT guarantees converge — the server just needs to deliver
// all updates to all clients. No OT transform needed.

use axum::{
    extract::{ws::{Message, WebSocket, WebSocketUpgrade}, State},
    response::Response,
};
use futures_util::{SinkExt, StreamExt};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use tachyon_core::types::crdt::{
    CrdtDocumentState, CrdtUser, CollaborationInfo, SelectionRange,
};

// ============================================================================
// CRDT Connection Manager
// ============================================================================

/// A connected client in a CRDT collaboration session.
#[derive(Debug, Clone)]
struct CrdtClient {
    client_id: String,
    user_id: String,
    user_name: String,
    document_id: String,
}

/// CRDT-aware connection manager.
///
/// Manages per-document Yjs update logs and routes binary updates
/// between collaborating clients.
#[derive(Clone)]
pub struct CrdtConnectionManager {
    /// All connected clients, keyed by client_id.
    clients: Arc<RwLock<HashMap<String, CrdtClient>>>,
    /// Per-document CRDT state (Yjs update logs).
    documents: Arc<RwLock<HashMap<String, CrdtDocumentState>>>,
    /// Per-document client list for targeted broadcast.
    document_clients: Arc<RwLock<HashMap<String, Vec<String>>>>,
    /// Per-document presence info.
    presence: Arc<RwLock<HashMap<String, HashMap<String, CrdtUser>>>>,
    /// Broadcast channel for pushing updates to clients.
    broadcast_tx: broadcast::Sender<CrdtBroadcastEvent>,
}

/// Internal broadcast event for routing updates to specific clients.
#[derive(Debug, Clone)]
enum CrdtBroadcastEvent {
    /// A Yjs update from another client (binary).
    Update {
        document_id: String,
        sender_client_id: String,
        update: Vec<u8>,
    },
    /// Presence update from another client.
    Presence {
        document_id: String,
        user: CrdtUser,
    },
    /// Awareness event (join/leave).
    Awareness {
        document_id: String,
        user_id: String,
        user_name: String,
        awareness_type: String,
    },
    /// A client disconnected.
    ClientLeft {
        #[allow(dead_code)]
        client_id: String,
        document_id: String,
        user_id: String,
    },
}

// ============================================================================
// Message wire format
// ============================================================================

/// The wire format for CRDT WebSocket messages.
///
/// We use a simple JSON envelope with a `type` discriminator:
/// - `{ "type": "sync", ... }` — Yjs sync protocol
/// - `{ "type": "presence", ... }` — cursor/selection updates
/// - `{ "type": "awareness", ... }` — join/leave signals
/// - `{ "type": "join", "documentId": "...", "userId": "...", "userName": "..." }` — join a document room
///
/// Binary messages are raw Yjs updates (for efficiency, but we use JSON for now
/// since the update payload is base64-encoded anyway).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
enum WireMessage {
    /// Join a document collaboration room.
    Join {
        #[serde(rename = "documentId")]
        document_id: String,
        #[serde(rename = "userId")]
        user_id: String,
        #[serde(rename = "userName")]
        user_name: String,
    },
    /// Yjs sync step 1: client sends state vector or update.
    #[serde(rename = "sync")]
    Sync {
        step: u8,
        /// Base64-encoded Yjs update or state vector.
        #[serde(with = "wire_base64")]
        update: Vec<u8>,
    },
    /// Presence update (cursor, selection).
    Presence {
        #[serde(rename = "documentId")]
        document_id: String,
        cursor: Option<u32>,
        selection: Option<SelectionRange>,
        color: Option<String>,
    },
    /// Awareness signal.
    Awareness {
        #[serde(rename = "documentId")]
        document_id: String,
        #[serde(rename = "awarenessType")]
        awareness_type: String,
    },
}

mod wire_base64 {
    use base64::{Engine as _, engine::general_purpose::STANDARD};
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S: Serializer>(data: &Vec<u8>, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&STANDARD.encode(data))
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Vec<u8>, D::Error> {
        let encoded: String = String::deserialize(deserializer)?;
        STANDARD.decode(&encoded).map_err(serde::de::Error::custom)
    }
}

// ============================================================================
// CRDT Connection Manager Implementation
// ============================================================================

impl CrdtConnectionManager {
    /// Create a new CRDT connection manager.
    pub fn new() -> Self {
        let (broadcast_tx, _) = broadcast::channel(4096);
        Self {
            clients: Arc::new(RwLock::new(HashMap::new())),
            documents: Arc::new(RwLock::new(HashMap::new())),
            document_clients: Arc::new(RwLock::new(HashMap::new())),
            presence: Arc::new(RwLock::new(HashMap::new())),
            broadcast_tx,
        }
    }

    /// Subscribe to broadcast events.
    fn subscribe(&self) -> broadcast::Receiver<CrdtBroadcastEvent> {
        self.broadcast_tx.subscribe()
    }

    /// Get the number of connected clients.
    pub async fn client_count(&self) -> usize {
        self.clients.read().await.len()
    }

    /// Get collaboration info for a document.
    pub async fn get_collaboration_info(&self, document_id: &str) -> Option<CollaborationInfo> {
        let presence = self.presence.read().await;
        let doc_clients = self.document_clients.read().await;
        let users = presence
            .get(document_id)?
            .values()
            .cloned()
            .collect();
        let count = doc_clients.get(document_id)?.len();
        Some(CollaborationInfo {
            document_id: document_id.to_string(),
            active_users: users,
            connection_count: count,
        })
    }

    /// Get or create CRDT document state.
    #[allow(dead_code)]
    async fn get_or_create_document(&self, document_id: &str) -> CrdtDocumentState {
        let mut docs = self.documents.write().await;
        docs.entry(document_id.to_string())
            .or_insert_with(|| CrdtDocumentState::new(document_id))
            .clone()
    }

    /// Initialize document state from persisted Yjs updates.
    pub async fn init_document_from_persisted(&self, document_id: &str, update_log: Vec<u8>) {
        let mut docs = self.documents.write().await;
        docs.entry(document_id.to_string())
            .or_insert_with(|| CrdtDocumentState::from_persisted(document_id, update_log));
    }

    /// Get the persisted Yjs update log for a document.
    pub async fn get_document_update_log(&self, document_id: &str) -> Option<Vec<u8>> {
        let docs = self.documents.read().await;
        docs.get(document_id).map(|d| d.get_update_log().to_vec())
    }

    /// Add a client to a document room.
    async fn join_document(&self, client_id: &str, document_id: &str, user_id: &str, user_name: &str) {
        // Register client
        {
            let mut clients = self.clients.write().await;
            clients.insert(client_id.to_string(), CrdtClient {
                client_id: client_id.to_string(),
                user_id: user_id.to_string(),
                user_name: user_name.to_string(),
                document_id: document_id.to_string(),
            });
        }

        // Add to document room
        {
            let mut doc_clients = self.document_clients.write().await;
            doc_clients
                .entry(document_id.to_string())
                .or_default()
                .push(client_id.to_string());
        }

        // Ensure document state exists and increment active client count
        {
            let mut docs = self.documents.write().await;
            let doc = docs.entry(document_id.to_string())
                .or_insert_with(|| CrdtDocumentState::new(document_id));
            doc.active_clients += 1;
        }

        // Add presence
        {
            let mut presence = self.presence.write().await;
            presence
                .entry(document_id.to_string())
                .or_default()
                .insert(client_id.to_string(), CrdtUser {
                    user_id: user_id.to_string(),
                    user_name: user_name.to_string(),
                    cursor: None,
                    selection: None,
                    color: None,
                });
        }

        info!(
            client_id = %client_id,
            document_id = %document_id,
            user = %user_name,
            "Client joined CRDT document room"
        );
    }

    /// Remove a client from its document room.
    async fn leave_document(&self, client_id: &str) -> Option<CrdtClient> {
        let client = self.clients.write().await.remove(client_id)?;

        // Remove from document room
        {
            let mut doc_clients = self.document_clients.write().await;
            if let Some(members) = doc_clients.get_mut(&client.document_id) {
                members.retain(|id| id != client_id);
            }
        }

        // Decrement active client count
        {
            let mut docs = self.documents.write().await;
            if let Some(doc) = docs.get_mut(&client.document_id) {
                doc.active_clients = doc.active_clients.saturating_sub(1);
            }
        }

        // Remove presence
        {
            let mut presence = self.presence.write().await;
            if let Some(doc_presence) = presence.get_mut(&client.document_id) {
                doc_presence.remove(client_id);
            }
        }

        info!(
            client_id = %client_id,
            document_id = %client.document_id,
            "Client left CRDT document room"
        );

        Some(client)
    }

    /// Apply a Yjs update to the document state and broadcast to other clients.
    async fn apply_update(&self, document_id: &str, sender_id: &str, update: Vec<u8>) {
        // Store the update
        {
            let mut docs = self.documents.write().await;
            if let Some(doc) = docs.get_mut(document_id) {
                doc.append_update(&update);
            }
        }

        // Broadcast to all clients in the document room (except sender)
        let _ = self.broadcast_tx.send(CrdtBroadcastEvent::Update {
            document_id: document_id.to_string(),
            sender_client_id: sender_id.to_string(),
            update,
        });
    }

    /// Update a user's presence (cursor, selection).
    async fn update_presence(&self, document_id: &str, client_id: &str, user: CrdtUser) {
        let mut presence = self.presence.write().await;
        if let Some(doc_presence) = presence.get_mut(document_id) {
            doc_presence.insert(client_id.to_string(), user.clone());
        }

        // Broadcast presence to other clients
        let _ = self.broadcast_tx.send(CrdtBroadcastEvent::Presence {
            document_id: document_id.to_string(),
            user,
        });
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

/// Handle CRDT WebSocket upgrade.
pub async fn handle_crdt_websocket_upgrade(
    ws: WebSocketUpgrade,
    State(manager): State<CrdtConnectionManager>,
) -> Response {
    ws.on_upgrade(move |socket| handle_crdt_socket(socket, manager))
}

/// Handle a single CRDT WebSocket connection.
async fn handle_crdt_socket(socket: WebSocket, manager: CrdtConnectionManager) {
    let client_id = Uuid::new_v4().to_string();
    let (mut sender, mut receiver) = socket.split();
    let mut rx = manager.subscribe();

    info!(client_id = %client_id, "CRDT WebSocket connection established");

    let client_id_recv = client_id.clone();
    let manager_recv = manager.clone();

    // Receive task: handle incoming messages from the client
    let recv_task = async move {
        while let Some(result) = receiver.next().await {
            match result {
                Ok(Message::Text(text)) => {
                    match serde_json::from_str::<WireMessage>(&text) {
                        Ok(msg) => {
                            handle_crdt_message(&manager_recv, &client_id_recv, msg).await;
                        }
                        Err(e) => {
                            warn!(client_id = %client_id_recv, error = %e, "Failed to parse CRDT message");
                        }
                    }
                }
                Ok(Message::Binary(data)) => {
                    // Binary messages: treat as raw Yjs update
                    // The first 4 bytes are the document ID length (for routing),
                    // followed by the document ID, then the Yjs update.
                    if data.len() > 4 {
                        let id_len = u32::from_be_bytes([data[0], data[1], data[2], data[3]]) as usize;
                        if data.len() > 4 + id_len {
                            let doc_id = String::from_utf8_lossy(&data[4..4 + id_len]);
                            let update = data[4 + id_len..].to_vec();
                            manager_recv.apply_update(&doc_id, &client_id_recv, update).await;
                            continue;
                        }
                    }
                    warn!(client_id = %client_id_recv, "Invalid binary message format");
                }
                Ok(Message::Close(_)) => {
                    info!(client_id = %client_id_recv, "Client sent close frame");
                    break;
                }
                Ok(Message::Ping(_)) => {
                    debug!(client_id = %client_id_recv, "Received ping");
                }
                Ok(Message::Pong(_)) => {
                    debug!(client_id = %client_id_recv, "Received pong");
                }
                Err(e) => {
                    error!(client_id = %client_id_recv, error = %e, "WebSocket error");
                    break;
                }
            }
        }
    };

    // Send task: forward broadcast events to the client
    let client_id_send = client_id.clone();
    let manager_for_send = manager.clone();
    let send_task = async move {
        let manager_clients = manager_for_send.clients.clone();

        loop {
            tokio::select! {
                event = rx.recv() => {
                    match event {
                        Ok(event) => {
                            // Filter: only send events relevant to this client's document
                            let should_send = match &event {
                                CrdtBroadcastEvent::Update { document_id, sender_client_id, .. } => {
                                    // Only send if we're in the same document and we're not the sender
                                    let clients = manager_clients.read().await;
                                    clients.get(&client_id_send)
                                        .map(|c| c.document_id == *document_id && c.client_id != *sender_client_id)
                                        .unwrap_or(false)
                                }
                                CrdtBroadcastEvent::Presence { document_id, .. } => {
                                    let clients = manager_clients.read().await;
                                    clients.get(&client_id_send)
                                        .map(|c| c.document_id == *document_id)
                                        .unwrap_or(false)
                                }
                                CrdtBroadcastEvent::Awareness { document_id, .. } => {
                                    let clients = manager_clients.read().await;
                                    clients.get(&client_id_send)
                                        .map(|c| c.document_id == *document_id)
                                        .unwrap_or(false)
                                }
                                CrdtBroadcastEvent::ClientLeft { document_id, .. } => {
                                    let clients = manager_clients.read().await;
                                    clients.get(&client_id_send)
                                        .map(|c| c.document_id == *document_id)
                                        .unwrap_or(false)
                                }
                            };

                            if !should_send {
                                continue;
                            }

                            let msg = match event {
                                CrdtBroadcastEvent::Update { update, .. } => {
                                    // Send as sync step 2 (server has new updates)
                                    serde_json::json!({
                                        "type": "sync",
                                        "step": 2,
                                        "update": base64_encode(&update)
                                    })
                                }
                                CrdtBroadcastEvent::Presence { user, .. } => {
                                    serde_json::json!({
                                        "type": "presence",
                                        "userId": user.user_id,
                                        "userName": user.user_name,
                                        "cursor": user.cursor,
                                        "selection": user.selection,
                                        "color": user.color
                                    })
                                }
                                CrdtBroadcastEvent::Awareness { user_id, user_name, awareness_type, .. } => {
                                    serde_json::json!({
                                        "type": "awareness",
                                        "userId": user_id,
                                        "userName": user_name,
                                        "awarenessType": awareness_type
                                    })
                                }
                                CrdtBroadcastEvent::ClientLeft { user_id, .. } => {
                                    serde_json::json!({
                                        "type": "awareness",
                                        "userId": user_id,
                                        "awarenessType": "leave"
                                    })
                                }
                            };

                            if let Ok(text) = serde_json::to_string(&msg) {
                                if sender.send(Message::Text(text.into())).await.is_err() {
                                    break;
                                }
                            }
                        }
                        Err(broadcast::error::RecvError::Lagged(n)) => {
                            warn!(skipped = n, "Broadcast receiver lagged, skipping events");
                            continue;
                        }
                        Err(broadcast::error::RecvError::Closed) => {
                            break;
                        }
                    }
                }
                _ = tokio::time::sleep(std::time::Duration::from_secs(30)) => {
                    // Heartbeat ping
                    if sender.send(Message::Ping(Vec::new().into())).await.is_err() {
                        break;
                    }
                }
            }
        }
    };

    // Wait for either task to finish, then clean up
    tokio::select! {
        _ = recv_task => {},
        _ = send_task => {},
    }

    // Clean up: notify other clients this one left
    if let Some(client) = manager.leave_document(&client_id).await {
        let _ = manager.broadcast_tx.send(CrdtBroadcastEvent::ClientLeft {
            client_id: client_id.clone(),
            document_id: client.document_id,
            user_id: client.user_id,
        });
    }

    info!(client_id = %client_id, "CRDT WebSocket connection closed");
}

// ============================================================================
// Message Handler
// ============================================================================

async fn handle_crdt_message(manager: &CrdtConnectionManager, client_id: &str, msg: WireMessage) {
    match msg {
        WireMessage::Join { document_id, user_id, user_name } => {
            // Join the document room
            manager.join_document(client_id, &document_id, &user_id, &user_name).await;

            // Send the current document state (full update log) to the joining client
            let docs = manager.documents.read().await;
            let update_log = docs
                .get(&document_id)
                .map(|d| d.get_update_log().to_vec())
                .unwrap_or_default();

            // Send as sync step 2 with the full update log
            let _response = serde_json::json!({
                "type": "sync",
                "step": 2,
                "update": base64_encode(&update_log)
            });

            // We can't send directly here since we don't have the sender.
            // Instead, we broadcast to the room — the client will pick it up
            // via the send task. But since the client just joined, it IS in the room.
            // The send task filters by document_id match, so this works.
            // However, the sender filtering excludes the sender_client_id...
            // We need a special "init" path. For now, use awareness to signal init.

            // Send init message via broadcast (send task won't filter it because
            // it's not an Update/Presence/Awareness from another client)
            let _ = manager.broadcast_tx.send(CrdtBroadcastEvent::Update {
                document_id: document_id.clone(),
                sender_client_id: "__init__".to_string(), // Special ID that won't match any real client
                update: update_log,
            });

            // Broadcast awareness to other clients
            let _ = manager.broadcast_tx.send(CrdtBroadcastEvent::Awareness {
                document_id,
                user_id: user_id.clone(),
                user_name,
                awareness_type: "enter".to_string(),
            });
        }

        WireMessage::Sync { step, update } => {
            // Get the client's document ID
            let clients = manager.clients.read().await;
            let document_id = match clients.get(client_id) {
                Some(c) => c.document_id.clone(),
                None => {
                    warn!(client_id = %client_id, "Sync from unjoined client, ignoring");
                    return;
                }
            };
            drop(clients);

            if step == 1 {
                // Client is sending its state vector or initial update.
                // In the simple relay model, we just store any updates sent.
                if !update.is_empty() {
                    manager.apply_update(&document_id, client_id, update).await;
                }
            } else if step == 2 {
                // Client is sending an update (diff or full).
                // Store and broadcast.
                if !update.is_empty() {
                    manager.apply_update(&document_id, client_id, update).await;
                }
            }
        }

        WireMessage::Presence { document_id, cursor, selection, color } => {
            // Get user info
            let user_info = {
                let clients = manager.clients.read().await;
                match clients.get(client_id) {
                    Some(c) => (c.user_id.clone(), c.user_name.clone()),
                    None => return,
                }
            };

            // Check that the client is in the right document
            {
                let clients = manager.clients.read().await;
                if clients.get(client_id).map(|c| c.document_id == document_id).unwrap_or(false) {
                    manager.update_presence(&document_id, client_id, CrdtUser {
                        user_id: user_info.0,
                        user_name: user_info.1,
                        cursor,
                        selection,
                        color,
                    }).await;
                }
            }
        }

        WireMessage::Awareness { document_id, awareness_type } => {
            let clients = manager.clients.read().await;
            let user_info = match clients.get(client_id) {
                Some(c) => (c.user_id.clone(), c.user_name.clone()),
                None => return,
            };

            // Broadcast awareness to other clients
            let _ = manager.broadcast_tx.send(CrdtBroadcastEvent::Awareness {
                document_id,
                user_id: user_info.0,
                user_name: user_info.1,
                awareness_type,
            });
        }
    }
}

// ============================================================================
// Utility
// ============================================================================

fn base64_encode(data: &[u8]) -> String {
    use base64::{Engine as _, engine::general_purpose::STANDARD};
    STANDARD.encode(data)
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_crdt_manager_new() {
        let manager = CrdtConnectionManager::new();
        assert_eq!(manager.client_count().await, 0);
    }

    #[tokio::test]
    async fn test_join_and_leave() {
        let manager = CrdtConnectionManager::new();
        manager.join_document("client-1", "doc-1", "user-1", "Alice").await;
        assert_eq!(manager.client_count().await, 1);

        let info = manager.get_collaboration_info("doc-1").await.unwrap();
        assert_eq!(info.connection_count, 1);
        assert_eq!(info.active_users.len(), 1);
        assert_eq!(info.active_users[0].user_name, "Alice");

        manager.leave_document("client-1").await;
        assert_eq!(manager.client_count().await, 0);
    }

    #[tokio::test]
    async fn test_apply_update() {
        let manager = CrdtConnectionManager::new();
        manager.join_document("c1", "doc-1", "u1", "Alice").await;

        let update = vec![0x01, 0x02, 0x03];
        manager.apply_update("doc-1", "c1", update.clone()).await;

        let log = manager.get_document_update_log("doc-1").await.unwrap();
        assert_eq!(log, update);
    }

    #[tokio::test]
    async fn test_multiple_clients() {
        let manager = CrdtConnectionManager::new();
        manager.join_document("c1", "doc-1", "u1", "Alice").await;
        manager.join_document("c2", "doc-1", "u2", "Bob").await;

        let info = manager.get_collaboration_info("doc-1").await.unwrap();
        assert_eq!(info.connection_count, 2);
        assert_eq!(info.active_users.len(), 2);

        // Apply updates from both clients
        manager.apply_update("doc-1", "c1", vec![0x01]).await;
        manager.apply_update("doc-1", "c2", vec![0x02]).await;

        let log = manager.get_document_update_log("doc-1").await.unwrap();
        assert_eq!(log, vec![0x01, 0x02]);
    }

    #[tokio::test]
    async fn test_init_from_persisted() {
        let manager = CrdtConnectionManager::new();
        let persisted = vec![0xAA, 0xBB, 0xCC];
        manager.init_document_from_persisted("doc-1", persisted.clone()).await;

        let log = manager.get_document_update_log("doc-1").await.unwrap();
        assert_eq!(log, persisted);

        // New updates append to the persisted log
        manager.join_document("c1", "doc-1", "u1", "Alice").await;
        manager.apply_update("doc-1", "c1", vec![0xDD]).await;

        let log = manager.get_document_update_log("doc-1").await.unwrap();
        assert_eq!(log, vec![0xAA, 0xBB, 0xCC, 0xDD]);
    }

    #[test]
    fn test_wire_message_join() {
        let msg = WireMessage::Join {
            document_id: "doc-1".to_string(),
            user_id: "u1".to_string(),
            user_name: "Alice".to_string(),
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("\"type\":\"join\""));
        assert!(json.contains("\"documentId\":\"doc-1\""));

        let decoded: WireMessage = serde_json::from_str(&json).unwrap();
        match decoded {
            WireMessage::Join { document_id, user_id, user_name } => {
                assert_eq!(document_id, "doc-1");
                assert_eq!(user_id, "u1");
                assert_eq!(user_name, "Alice");
            }
            _ => panic!("Expected Join"),
        }
    }

    #[test]
    fn test_wire_message_sync() {
        let msg = WireMessage::Sync {
            step: 1,
            update: vec![0x01, 0x02, 0x03],
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("\"type\":\"sync\""));
        assert!(json.contains("\"step\":1"));

        let decoded: WireMessage = serde_json::from_str(&json).unwrap();
        match decoded {
            WireMessage::Sync { step, update } => {
                assert_eq!(step, 1);
                assert_eq!(update, vec![0x01, 0x02, 0x03]);
            }
            _ => panic!("Expected Sync"),
        }
    }
}
