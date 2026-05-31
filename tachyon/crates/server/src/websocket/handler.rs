use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::{IntoResponse, Response},
};
use futures_util::{SinkExt, StreamExt};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, mpsc, RwLock};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use super::operational_transform::{DocumentState, Operation};
use super::types::{
    DocumentEdit, EditOperation, HeartbeatConfig, HeartbeatMessage, MessageType, WebSocketMessage,
};

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
    user_id: Option<String>,
    user_name: Option<String>,
    rooms: Vec<String>,
    last_seen: std::time::Instant,
}

#[derive(Clone)]
pub struct ConnectionManager {
    clients: Arc<RwLock<HashMap<String, ConnectedClient>>>,
    rooms: Arc<RwLock<HashMap<String, Vec<String>>>>,
    broadcast_tx: broadcast::Sender<WebSocketMessage>,
    document_states: Arc<RwLock<HashMap<String, DocumentState>>>,
    document_presence: Arc<RwLock<HashMap<String, Vec<PresenceInfo>>>>,
    seq_counter: Arc<std::sync::atomic::AtomicU64>,
    max_connections: usize,
    heartbeat_config: HeartbeatConfig,
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
            seq_counter: Arc::new(std::sync::atomic::AtomicU64::new(0)),
            max_connections: 1000,
            heartbeat_config: HeartbeatConfig::default(),
        }
    }

    pub fn with_config(max_connections: usize, heartbeat_interval_secs: u64) -> Self {
        let (broadcast_tx, _) = broadcast::channel(1024);
        let heartbeat_config = HeartbeatConfig {
            interval_secs: heartbeat_interval_secs,
            ..HeartbeatConfig::default()
        };
        Self {
            clients: Arc::new(RwLock::new(HashMap::new())),
            rooms: Arc::new(RwLock::new(HashMap::new())),
            broadcast_tx,
            document_states: Arc::new(RwLock::new(HashMap::new())),
            document_presence: Arc::new(RwLock::new(HashMap::new())),
            seq_counter: Arc::new(std::sync::atomic::AtomicU64::new(0)),
            max_connections,
            heartbeat_config,
        }
    }

    fn next_seq(&self) -> u64 {
        self.seq_counter
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    }

    pub async fn active_connection_count(&self) -> usize {
        self.clients.read().await.len()
    }

    pub fn max_connections(&self) -> usize {
        self.max_connections
    }

    pub async fn client_count(&self) -> usize {
        self.clients.read().await.len()
    }

    pub async fn room_count(&self, room_id: &str) -> usize {
        self.rooms
            .read()
            .await
            .get(room_id)
            .map(|r| r.len())
            .unwrap_or(0)
    }

    pub async fn add_client(&self, client_id: String) {
        let client = ConnectedClient {
            user_id: None,
            user_name: None,
            rooms: Vec::new(),
            last_seen: std::time::Instant::now(),
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
            let client_rooms = client.rooms.clone();
            {
                let mut rooms = self.rooms.write().await;
                for room_id in &client_rooms {
                    if let Some(members) = rooms.get_mut(room_id) {
                        members.retain(|id| id != client_id);
                        if members.is_empty() {
                            rooms.remove(room_id);
                        }
                    }
                }
            }

            for room_id in &client_rooms {
                if let Some(doc_id) = room_id.strip_prefix("doc:") {
                    let should_remove = {
                        let mut presence = self.document_presence.write().await;
                        if let Some(list) = presence.get_mut(doc_id) {
                            list.retain(|p| p.client_id != client_id);
                            list.is_empty()
                        } else {
                            false
                        }
                    };
                    if should_remove {
                        self.document_presence.write().await.remove(doc_id);
                    }

                    if let Some(ref uid) = client.user_id {
                        let leave_msg = WebSocketMessage::leave(doc_id.to_string(), uid.clone());
                        self.broadcast_to_room(&format!("doc:{}", doc_id), leave_msg)
                            .await;
                    }
                }
            }

            info!("Client disconnected");
        }
    }

    pub async fn handle_heartbeat_timeout(&self, client_id: &str) {
        warn!(client_id = %client_id, "Client disconnected due to heartbeat timeout");
        self.remove_client(client_id).await;
    }

    pub async fn join_room(&self, client_id: &str, room_id: &str) {
        {
            let mut rooms = self.rooms.write().await;
            rooms
                .entry(room_id.to_string())
                .or_default()
                .push(client_id.to_string());
        }
        if let Some(client) = self.clients.write().await.get_mut(client_id) {
            if !client.rooms.contains(&room_id.to_string()) {
                client.rooms.push(room_id.to_string());
            }
        }
        debug!(client_id = %client_id, room_id = %room_id, "Client joined room");
    }

    pub async fn join_document(
        &self,
        client_id: &str,
        document_id: &str,
        user_id: &str,
        user_name: &str,
    ) {
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

    pub async fn is_client_in_room(&self, client_id: &str, room_id: &str) -> bool {
        let clients = self.clients.read().await;
        clients
            .get(client_id)
            .map(|c| c.rooms.iter().any(|r| r == room_id))
            .unwrap_or(false)
    }

    pub async fn broadcast(&self, message: WebSocketMessage) {
        let seq = self.next_seq();
        let msg = message.with_seq(seq);
        let _ = self.broadcast_tx.send(msg);
    }

    pub async fn broadcast_to_room(&self, room_id: &str, message: WebSocketMessage) {
        let rooms = self.rooms.read().await;
        if let Some(members) = rooms.get(room_id) {
            let seq = self.next_seq();
            let msg = message.with_seq(seq).with_room(room_id.to_string());
            let _ = self.broadcast_tx.send(msg);
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
                        clients
                            .get(id)
                            .and_then(|c| Some((c.user_id.clone()?, c.user_name.clone()?)))
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

    pub async fn apply_edit(
        &self,
        document_id: &str,
        operation: Operation,
        client_version: u64,
    ) -> Option<u64> {
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

    pub async fn touch_client(&self, client_id: &str) {
        if let Some(client) = self.clients.write().await.get_mut(client_id) {
            client.last_seen = std::time::Instant::now();
        }
    }

    pub async fn cleanup_stale_clients(&self, timeout_secs: u64) {
        let timeout = std::time::Duration::from_secs(timeout_secs);
        let stale_clients: Vec<(String, Vec<String>)> = {
            let clients = self.clients.read().await;
            clients
                .iter()
                .filter(|(_, c)| c.last_seen.elapsed() > timeout)
                .map(|(id, c)| (id.clone(), c.rooms.clone()))
                .collect()
        };

        if stale_clients.is_empty() {
            return;
        }

        let mut clients = self.clients.write().await;
        let mut rooms = self.rooms.write().await;
        let mut presence = self.document_presence.write().await;
        let mut doc_states = self.document_states.write().await;

        for (client_id, client_rooms) in &stale_clients {
            clients.remove(client_id);

            for room_id in client_rooms {
                if let Some(members) = rooms.get_mut(room_id) {
                    members.retain(|id| id != client_id);
                    if members.is_empty() {
                        rooms.remove(room_id);
                    }
                }

                if let Some(doc_id) = room_id.strip_prefix("doc:") {
                    if let Some(list) = presence.get_mut(doc_id) {
                        list.retain(|p| p.client_id != *client_id);
                        if list.is_empty() {
                            presence.remove(doc_id);
                        }
                    }
                }
            }

            info!(client_id = %client_id, "Cleaned up stale client");
        }

        let stale_doc_ids: Vec<String> = stale_clients
            .iter()
            .flat_map(|(_, rooms)| rooms.iter())
            .filter_map(|r| r.strip_prefix("doc:").map(|s| s.to_string()))
            .collect();

        for doc_id in stale_doc_ids {
            let has_members = rooms
                .get(&format!("doc:{}", doc_id))
                .map(|m| !m.is_empty())
                .unwrap_or(false);
            if !has_members {
                doc_states.remove(&doc_id);
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
    let current = manager.active_connection_count().await;
    if current >= manager.max_connections() {
        return (
            axum::http::StatusCode::SERVICE_UNAVAILABLE,
            axum::Json(serde_json::json!({
                "error": "Maximum WebSocket connections reached",
            })),
        )
            .into_response();
    }
    ws.on_upgrade(move |socket| handle_socket(socket, manager))
}

async fn handle_socket(socket: WebSocket, manager: ConnectionManager) {
    let client_id = Uuid::new_v4().to_string();
    manager.add_client(client_id.clone()).await;

    let (mut sender, mut receiver) = socket.split();
    let mut broadcast_rx = manager.subscribe();
    let config = manager.heartbeat_config.clone();

    let (outgoing_tx, mut outgoing_rx) = mpsc::channel::<String>(32);
    let (pong_tx, pong_rx) = mpsc::channel::<u64>(8);
    let heartbeat_timed_out = Arc::new(std::sync::atomic::AtomicBool::new(false));

    info!(client_id = %client_id, "WebSocket connection established");

    let client_id_recv = client_id.clone();
    let manager_clone = manager.clone();
    let recv_task = async move {
        while let Some(result) = receiver.next().await {
            match result {
                Ok(Message::Text(text)) => {
                    if let Ok(HeartbeatMessage::Pong { timestamp }) =
                        serde_json::from_str::<HeartbeatMessage>(&text)
                    {
                        let _ = pong_tx.send(timestamp).await;
                        continue;
                    }
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

    let manager_for_send = manager.clone();
    let client_id_for_send = client_id.clone();
    let send_task = async move {
        loop {
            tokio::select! {
                msg = broadcast_rx.recv() => {
                    match msg {
                        Ok(msg) => {
                            let should_deliver = match &msg.room_id {
                                None => true,
                                Some(room) => {
                                    manager_for_send.is_client_in_room(&client_id_for_send, room).await
                                }
                            };

                            if should_deliver {
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
                        }
                        Err(broadcast::error::RecvError::Lagged(n)) => {
                            warn!(skipped = n, "Broadcast receiver lagged, skipping messages");
                        }
                        Err(broadcast::error::RecvError::Closed) => {
                            break;
                        }
                    }
                }
                msg = outgoing_rx.recv() => {
                    match msg {
                        Some(json) => {
                            if sender.send(Message::Text(json.into())).await.is_err() {
                                break;
                            }
                        }
                        None => break,
                    }
                }
            }
        }
    };

    let heartbeat_timed_out_clone = heartbeat_timed_out.clone();
    let heartbeat_task = async move {
        if !config.enabled {
            std::future::pending::<()>().await;
        }

        let mut interval =
            tokio::time::interval(std::time::Duration::from_secs(config.interval_secs));
        interval.tick().await;
        let mut pong_rx = pong_rx;

        loop {
            interval.tick().await;
            let ts = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            let ping = HeartbeatMessage::Ping { timestamp: ts };
            let json = serde_json::to_string(&ping)
                .unwrap_or_else(|_| r#"{"type":"ping","timestamp":0}"#.to_string());
            if outgoing_tx.send(json).await.is_err() {
                break;
            }
            match tokio::time::timeout(
                std::time::Duration::from_secs(config.timeout_secs),
                pong_rx.recv(),
            )
            .await
            {
                Ok(Some(_)) => {}
                Ok(None) | Err(_) => {
                    heartbeat_timed_out_clone.store(true, std::sync::atomic::Ordering::Relaxed);
                    break;
                }
            }
        }
    };

    tokio::select! {
        _ = recv_task => {}
        _ = send_task => {}
        _ = heartbeat_task => {}
    }

    if heartbeat_timed_out.load(std::sync::atomic::Ordering::Relaxed) {
        manager.handle_heartbeat_timeout(&client_id).await;
    } else {
        manager.remove_client(&client_id).await;
    }
    info!(client_id = %client_id, "WebSocket connection closed");
}

async fn handle_client_message(
    manager: &ConnectionManager,
    client_id: &str,
    msg: WebSocketMessage,
) {
    manager.touch_client(client_id).await;
    match msg.message_type {
        MessageType::Join => {
            if let (Some(doc_id), Some(user_id)) = (&msg.document_id, &msg.user_id) {
                let user_name = msg
                    .data
                    .as_ref()
                    .and_then(|d| d.get("user_name"))
                    .and_then(|n| n.as_str())
                    .unwrap_or("Unknown")
                    .to_string();

                manager
                    .set_user_info(client_id, user_id.clone(), user_name.clone())
                    .await;
                manager
                    .join_document(client_id, doc_id, user_id, &user_name)
                    .await;

                let join_msg = WebSocketMessage::join(doc_id.clone(), user_id.clone(), user_name);
                manager
                    .broadcast_to_room(&format!("doc:{}", doc_id), join_msg)
                    .await;

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
                manager
                    .broadcast_to_room(&format!("doc:{}", doc_id), presence_msg)
                    .await;
            }
        }
        MessageType::Leave => {
            if let (Some(doc_id), Some(user_id)) = (&msg.document_id, &msg.user_id) {
                manager.leave_document(client_id, doc_id).await;

                let leave_msg = WebSocketMessage::leave(doc_id.clone(), user_id.clone());
                manager
                    .broadcast_to_room(&format!("doc:{}", doc_id), leave_msg)
                    .await;
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
                            EditOperation::Replace {
                                position,
                                length,
                                text,
                            } => {
                                vec![
                                    Operation::delete(position, length),
                                    Operation::insert(position, text),
                                ]
                            }
                        };

                        if let Some(first_op) = operations.first() {
                            let new_version = if operations.len() == 1 {
                                manager
                                    .apply_edit(doc_id, first_op.clone(), edit.version)
                                    .await
                            } else {
                                let v1 = manager
                                    .apply_edit(doc_id, first_op.clone(), edit.version)
                                    .await;
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
                                        obj.insert(
                                            "version".to_string(),
                                            serde_json::json!(new_version),
                                        );
                                    }
                                }
                                manager
                                    .broadcast_to_room(&format!("doc:{}", doc_id), response_msg)
                                    .await;
                            }
                        }
                    }
                }
            }
        }
        MessageType::Activity | MessageType::Presence => {
            if let Some(doc_id) = &msg.document_id {
                manager
                    .broadcast_to_room(&format!("doc:{}", doc_id), msg)
                    .await;
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

pub async fn run_heartbeat_loop(
    config: HeartbeatConfig,
    outgoing: mpsc::Sender<String>,
    mut pong_rx: mpsc::Receiver<u64>,
) -> bool {
    let mut interval = tokio::time::interval(std::time::Duration::from_secs(config.interval_secs));
    interval.tick().await;

    loop {
        interval.tick().await;
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let ping = HeartbeatMessage::Ping { timestamp: ts };
        let json = serde_json::to_string(&ping)
            .unwrap_or_else(|_| r#"{"type":"ping","timestamp":0}"#.to_string());
        if outgoing.send(json).await.is_err() {
            return false;
        }
        match tokio::time::timeout(
            std::time::Duration::from_secs(config.timeout_secs),
            pong_rx.recv(),
        )
        .await
        {
            Ok(Some(_)) => {}
            Ok(None) | Err(_) => {
                return true;
            }
        }
    }
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

        let msg = WebSocketMessage::join(
            "doc-1".to_string(),
            "user-1".to_string(),
            "Alice".to_string(),
        );
        manager.broadcast(msg).await;

        let received = rx.try_recv();
        assert!(received.is_ok());
    }

    #[tokio::test]
    async fn test_connection_cleanup_on_timeout() {
        let manager = ConnectionManager::new();
        let client_id = "timeout-client".to_string();

        manager.add_client(client_id.clone()).await;
        manager.join_room(&client_id, "doc:test-doc").await;
        manager
            .set_user_info(&client_id, "user-1".to_string(), "Test User".to_string())
            .await;

        assert_eq!(manager.client_count().await, 1);
        assert_eq!(manager.room_count("doc:test-doc").await, 1);

        manager.handle_heartbeat_timeout(&client_id).await;

        assert_eq!(manager.client_count().await, 0);
        assert_eq!(manager.room_count("doc:test-doc").await, 0);
    }

    #[tokio::test]
    async fn test_pong_resets_timeout() {
        let (outgoing_tx, mut outgoing_rx) = mpsc::channel::<String>(32);
        let (pong_tx, pong_rx) = mpsc::channel::<u64>(8);

        let config = HeartbeatConfig {
            interval_secs: 1,
            timeout_secs: 2,
            enabled: true,
        };

        let pong_tx_clone = pong_tx.clone();
        let responder = tokio::spawn(async move {
            while let Some(_ping_json) = outgoing_rx.recv().await {
                let _ = pong_tx_clone.send(12345).await;
            }
        });

        let heartbeat_handle = tokio::spawn(run_heartbeat_loop(config, outgoing_tx, pong_rx));

        tokio::time::sleep(std::time::Duration::from_secs(5)).await;

        let finished = heartbeat_handle.is_finished();
        assert!(
            !finished,
            "Heartbeat should not time out when pong is received"
        );

        heartbeat_handle.abort();
        responder.abort();
        drop(pong_tx);
    }

    #[tokio::test]
    async fn test_heartbeat_loop_timeout() {
        let (outgoing_tx, mut outgoing_rx) = mpsc::channel::<String>(32);
        let (_pong_tx, pong_rx) = mpsc::channel::<u64>(8);

        let config = HeartbeatConfig {
            interval_secs: 1,
            timeout_secs: 1,
            enabled: true,
        };

        tokio::spawn(async move { while outgoing_rx.recv().await.is_some() {} });

        let result = tokio::time::timeout(
            std::time::Duration::from_secs(5),
            run_heartbeat_loop(config, outgoing_tx, pong_rx),
        )
        .await;

        assert!(result.is_ok());
        assert!(
            result.unwrap(),
            "Heartbeat loop should return true on timeout"
        );
    }
}
