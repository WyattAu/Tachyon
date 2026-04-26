// WebSocket handler
// Handles WebSocket connection upgrades and message routing

use axum::{
    extract::{
        State,
        ws::{Message, WebSocket, WebSocketUpgrade},
    },
    response::{IntoResponse, Response},
};
use futures_util::{SinkExt, StreamExt};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use super::operational_transform::{DocumentState, Operation};
use super::types::{MessageType, WebSocketMessage, EditOperation, DocumentEdit};

#[derive(Debug)]
pub enum WebSocketUpgradeError {
    InvalidUpgrade,
    ConnectionError(String),
    InternalError(String),
}

impl std::fmt::Display for WebSocketUpgradeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidUpgrade => write!(f, "Invalid WebSocket upgrade request"),
            Self::ConnectionError(msg) => write!(f, "Connection error: {}", msg),
            Self::InternalError(msg) => write!(f, "Internal error: {}", msg),
        }
    }
}

impl std::error::Error for WebSocketUpgradeError {}

#[derive(Debug, Clone)]
struct ConnectedClient {
    #[allow(dead_code)]
    client_id: String,
    user_id: Option<String>,
    user_name: Option<String>,
    rooms: Vec<String>,
}

#[derive(Clone)]
pub struct ConnectionManager {
    clients: Arc<RwLock<HashMap<String, ConnectedClient>>>,
    rooms: Arc<RwLock<HashMap<String, Vec<String>>>>,
    broadcast_tx: broadcast::Sender<WebSocketMessage>,
    document_states: Arc<RwLock<HashMap<String, DocumentState>>>,
    document_presence: Arc<RwLock<HashMap<String, Vec<PresenceInfo>>>>,
}

#[derive(Debug, Clone)]
pub struct PresenceInfo {
    pub client_id: String,
    pub user_id: String,
    pub user_name: String,
    pub cursor_position: usize,
}

impl ConnectionManager {
    pub fn new() -> Self {
        let (broadcast_tx, _) = broadcast::channel(1024);
        Self {
            clients: Arc::new(RwLock::new(HashMap::new())),
            rooms: Arc::new(RwLock::new(HashMap::new())),
            broadcast_tx,
            document_states: Arc::new(RwLock::new(HashMap::new())),
            document_presence: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn client_count(&self) -> usize {
        self.clients.read().await.len()
    }

    pub async fn room_count(&self, room_id: &str) -> usize {
        self.rooms.read().await.get(room_id).map(|r| r.len()).unwrap_or(0)
    }

    pub async fn add_client(&self, client_id: String) {
        let client = ConnectedClient {
            client_id: client_id.clone(),
            user_id: None,
            user_name: None,
            rooms: Vec::new(),
        };
        self.clients.write().await.insert(client_id, client);
        info!("Client connected");
    }

    pub async fn set_user_info(&self, client_id: &str, user_id: String, user_name: String) {
        if let Some(client) = self.clients.write().await.get_mut(client_id) {
            client.user_id = Some(user_id);
            client.user_name = Some(user_name);
        }
    }

    pub async fn remove_client(&self, client_id: &str) {
        let client = {
            let mut clients = self.clients.write().await;
            clients.remove(client_id)
        };
        if let Some(client) = client {
            let mut rooms = self.rooms.write().await;
            for room_id in client.rooms {
                if let Some(members) = rooms.get_mut(&room_id) {
                    members.retain(|id| id != client_id);
                    if members.is_empty() {
                        rooms.remove(&room_id);
                    }
                }
            }
            info!("Client disconnected");
        }
    }

    pub async fn join_room(&self, client_id: &str, room_id: &str) {
        {
            let mut rooms = self.rooms.write().await;
            rooms.entry(room_id.to_string()).or_default().push(client_id.to_string());
        }
        if let Some(client) = self.clients.write().await.get_mut(client_id) {
            if !client.rooms.contains(&room_id.to_string()) {
                client.rooms.push(room_id.to_string());
            }
        }
        debug!(client_id = %client_id, room_id = %room_id, "Client joined room");
    }

    pub async fn join_document(&self, client_id: &str, document_id: &str, user_id: &str, user_name: &str) {
        let room_id = format!("doc:{}", document_id);
        self.join_room(client_id, &room_id).await;
        
        let presence = PresenceInfo {
            client_id: client_id.to_string(),
            user_id: user_id.to_string(),
            user_name: user_name.to_string(),
            cursor_position: 0,
        };
        
        self.document_presence
            .write()
            .await
            .entry(document_id.to_string())
            .or_default()
            .push(presence);
        
        debug!(client_id = %client_id, document_id = %document_id, "Client joined document");
    }

    pub async fn leave_document(&self, client_id: &str, document_id: &str) {
        let room_id = format!("doc:{}", document_id);
        self.leave_room(client_id, &room_id).await;
        
        let should_remove = {
            let mut presence = self.document_presence.write().await;
            if let Some(list) = presence.get_mut(document_id) {
                list.retain(|p| p.client_id != client_id);
                list.is_empty()
            } else {
                false
            }
        };
        
        if should_remove {
            self.document_presence.write().await.remove(document_id);
        }
        
        debug!(client_id = %client_id, document_id = %document_id, "Client left document");
    }

    pub async fn leave_room(&self, client_id: &str, room_id: &str) {
        {
            let mut rooms = self.rooms.write().await;
            if let Some(members) = rooms.get_mut(room_id) {
                members.retain(|id| id != client_id);
                if members.is_empty() {
                    rooms.remove(room_id);
                }
            }
        }
        if let Some(client) = self.clients.write().await.get_mut(client_id) {
            client.rooms.retain(|r| r != room_id);
        }
        debug!(client_id = %client_id, room_id = %room_id, "Client left room");
    }

    pub fn subscribe(&self) -> broadcast::Receiver<WebSocketMessage> {
        self.broadcast_tx.subscribe()
    }

    pub async fn broadcast(&self, message: WebSocketMessage) {
        let _ = self.broadcast_tx.send(message);
    }

    pub async fn broadcast_to_room(&self, room_id: &str, message: WebSocketMessage) {
        let rooms = self.rooms.read().await;
        if let Some(members) = rooms.get(room_id) {
            let _ = self.broadcast_tx.send(message);
            debug!(room_id = %room_id, member_count = members.len(), "Broadcasting to room");
        }
    }

    pub async fn get_room_users(&self, room_id: &str) -> Vec<(String, String)> {
        let rooms = self.rooms.read().await;
        let clients = self.clients.read().await;
        
        rooms
            .get(room_id)
            .map(|members| {
                members
                    .iter()
                    .filter_map(|id| {
                        clients.get(id).and_then(|c| {
                            Some((c.user_id.clone()?, c.user_name.clone()?))
                        })
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    pub async fn get_document_presence(&self, document_id: &str) -> Vec<PresenceInfo> {
        self.document_presence
            .read()
            .await
            .get(document_id)
            .cloned()
            .unwrap_or_default()
    }

    pub async fn get_document_state(&self, document_id: &str) -> Option<DocumentState> {
        self.document_states.read().await.get(document_id).cloned()
    }

    pub async fn init_document_state(&self, document_id: &str, content: String) {
        let mut states = self.document_states.write().await;
        if !states.contains_key(document_id) {
            states.insert(document_id.to_string(), DocumentState::new(content));
        }
    }

    pub async fn apply_edit(&self, document_id: &str, operation: Operation, client_version: u64) -> Option<u64> {
        let mut states = self.document_states.write().await;
        if let Some(state) = states.get_mut(document_id) {
            state.apply(operation, client_version).ok()
        } else {
            None
        }
    }

    pub async fn update_cursor(&self, client_id: &str, document_id: &str, cursor_position: usize) {
        if let Some(presence_list) = self.document_presence.write().await.get_mut(document_id) {
            if let Some(presence) = presence_list.iter_mut().find(|p| p.client_id == client_id) {
                presence.cursor_position = cursor_position;
            }
        }
    }
}

impl Default for ConnectionManager {
    fn default() -> Self {
        Self::new()
    }
}

pub async fn handle_websocket_upgrade(
    ws: WebSocketUpgrade,
    State(manager): State<ConnectionManager>,
) -> Response {
    ws.on_upgrade(move |socket| handle_socket(socket, manager))
}

async fn handle_socket(socket: WebSocket, manager: ConnectionManager) {
    let client_id = Uuid::new_v4().to_string();
    manager.add_client(client_id.clone()).await;
    
    let (mut sender, mut receiver) = socket.split();
    let mut rx = manager.subscribe();

    info!(client_id = %client_id, "WebSocket connection established");

    let client_id_recv = client_id.clone();
    let manager_clone = manager.clone();
    let recv_task = async move {
        while let Some(result) = receiver.next().await {
            match result {
                Ok(Message::Text(text)) => {
                    if let Ok(msg) = serde_json::from_str::<WebSocketMessage>(&text) {
                        handle_client_message(&manager_clone, &client_id_recv, msg).await;
                    } else {
                        warn!(text = %text, "Failed to parse WebSocket message");
                    }
                }
                Ok(Message::Binary(data)) => {
                    if let Ok(text) = String::from_utf8(data.to_vec()) {
                        if let Ok(msg) = serde_json::from_str::<WebSocketMessage>(&text) {
                            handle_client_message(&manager_clone, &client_id_recv, msg).await;
                        }
                    }
                }
                Ok(Message::Close(_)) => {
                    info!(client_id = %client_id_recv, "Client sent close frame");
                    break;
                }
                Ok(Message::Ping(_)) => {
                    debug!(client_id = %client_id_recv, "Received protocol ping");
                    // tokio-tungstenite auto-responds with Pong at the protocol level
                }
                Ok(Message::Pong(_)) => {
                    debug!(client_id = %client_id_recv, "Received pong — connection alive");
                }
                Err(e) => {
                    error!(client_id = %client_id_recv, error = %e, "WebSocket error");
                    break;
                }
            }
        }
    };

    // Send task also handles periodic heartbeat pings to detect dead connections.
    // tokio-tungstenite handles protocol-level Ping/Pong automatically,
    // but we send application-level pings via the broadcast channel as a backup.
    let client_id_send = client_id.clone();
    let send_task = async move {
        let mut heartbeat = tokio::time::interval(tokio::time::Duration::from_secs(30));
        heartbeat.tick().await; // skip the immediate first tick

        loop {
            tokio::select! {
                msg = rx.recv() => {
                    match msg {
                        Ok(msg) => {
                            let json = match serde_json::to_string(&msg) {
                                Ok(j) => j,
                                Err(e) => {
                                    error!(error = %e, "Failed to serialize message");
                                    continue;
                                }
                            };
                            
                            if sender.send(Message::Text(json.into())).await.is_err() {
                                break;
                            }
                        }
                        Err(broadcast::error::RecvError::Lagged(n)) => {
                            warn!(skipped = n, "Broadcast receiver lagged, skipping messages");
                        }
                        Err(broadcast::error::RecvError::Closed) => {
                            break;
                        }
                    }
                }
                _ = heartbeat.tick() => {
                    // Send a protocol-level Ping frame to detect dead connections.
                    // tokio-tungstenite will auto-close the connection if Pong isn't received.
                    if sender.send(Message::Ping(vec![].into())).await.is_err() {
                        info!(client_id = %client_id_send, "Failed to send heartbeat ping — connection dead");
                        break;
                    }
                }
            }
        }
    };

    tokio::select! {
        _ = recv_task => {}
        _ = send_task => {}
    }

    manager.remove_client(&client_id).await;
    info!(client_id = %client_id, "WebSocket connection closed");
}

async fn handle_client_message(manager: &ConnectionManager, client_id: &str, msg: WebSocketMessage) {
    match msg.message_type {
        MessageType::Join => {
            if let (Some(doc_id), Some(user_id)) = (&msg.document_id, &msg.user_id) {
                let user_name = msg.data
                    .as_ref()
                    .and_then(|d| d.get("user_name"))
                    .and_then(|n| n.as_str())
                    .unwrap_or("Unknown")
                    .to_string();
                
                manager.set_user_info(client_id, user_id.clone(), user_name.clone()).await;
                manager.join_document(client_id, doc_id, user_id, &user_name).await;
                
                let join_msg = WebSocketMessage::join(doc_id.clone(), user_id.clone(), user_name);
                manager.broadcast_to_room(&format!("doc:{}", doc_id), join_msg).await;
                
                let presence = manager.get_document_presence(doc_id).await;
                let presence_users: Vec<super::types::PresenceUser> = presence
                    .into_iter()
                    .filter(|p| p.client_id != client_id)
                    .map(|p| super::types::PresenceUser {
                        user_id: p.user_id,
                        user_name: p.user_name,
                        cursor_position: p.cursor_position,
                        selection: None,
                        color: None,
                    })
                    .collect();
                
                let presence_msg = WebSocketMessage::presence(doc_id.clone(), presence_users);
                manager.broadcast_to_room(&format!("doc:{}", doc_id), presence_msg).await;
            }
        }
        MessageType::Leave => {
            if let (Some(doc_id), Some(user_id)) = (&msg.document_id, &msg.user_id) {
                manager.leave_document(client_id, doc_id).await;
                
                let leave_msg = WebSocketMessage::leave(doc_id.clone(), user_id.clone());
                manager.broadcast_to_room(&format!("doc:{}", doc_id), leave_msg).await;
            }
        }
        MessageType::Edit => {
            if let Some(doc_id) = &msg.document_id {
                if let Some(data) = &msg.data {
                    if let Ok(edit) = serde_json::from_value::<DocumentEdit>(data.clone()) {
                        let operations = match edit.operation {
                            EditOperation::Insert { position, text } => {
                                vec![Operation::insert(position, text)]
                            }
                            EditOperation::Delete { position, length } => {
                                vec![Operation::delete(position, length)]
                            }
                            EditOperation::Replace { position, length, text } => {
                                vec![Operation::delete(position, length), Operation::insert(position, text)]
                            }
                        };
                        
                        if let Some(first_op) = operations.first() {
                            let new_version = if operations.len() == 1 {
                                manager.apply_edit(doc_id, first_op.clone(), edit.version).await
                            } else {
                                let v1 = manager.apply_edit(doc_id, first_op.clone(), edit.version).await;
                                let mut result = v1;
                                for op in operations.iter().skip(1) {
                                    let next_v = v1.unwrap_or(edit.version);
                                    result = manager.apply_edit(doc_id, op.clone(), next_v).await;
                                }
                                result
                            };
                            
                            if let Some(new_version) = new_version {
                                let mut response_msg = msg.clone();
                                if let Some(ref mut data) = response_msg.data {
                                    if let Some(obj) = data.as_object_mut() {
                                        obj.insert("version".to_string(), serde_json::json!(new_version));
                                    }
                                }
                                manager.broadcast_to_room(&format!("doc:{}", doc_id), response_msg).await;
                            }
                        }
                    }
                }
            }
        }
        MessageType::Activity | MessageType::Presence => {
            if let Some(doc_id) = &msg.document_id {
                manager.broadcast_to_room(&format!("doc:{}", doc_id), msg).await;
            }
        }
    }
}

pub fn websocket_upgrade_error(err: WebSocketUpgradeError) -> Response {
    let body = serde_json::json!({
        "error": err.to_string(),
    });
    (axum::http::StatusCode::BAD_REQUEST, axum::Json(body)).into_response()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_connection_manager() {
        let manager = ConnectionManager::new();
        let client_id = "test-client".to_string();
        
        manager.add_client(client_id.clone()).await;
        assert_eq!(manager.client_count().await, 1);
        
        manager.join_room(&client_id, "doc:test-doc").await;
        assert_eq!(manager.room_count("doc:test-doc").await, 1);
        
        manager.remove_client(&client_id).await;
        assert_eq!(manager.client_count().await, 0);
    }

    #[tokio::test]
    async fn test_broadcast() {
        let manager = ConnectionManager::new();
        let mut rx = manager.subscribe();
        
        let msg = WebSocketMessage::join("doc-1".to_string(), "user-1".to_string(), "Alice".to_string());
        manager.broadcast(msg).await;
        
        let received = rx.try_recv();
        assert!(received.is_ok());
    }
}
