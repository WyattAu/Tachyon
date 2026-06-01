// WebSocket message types
// Defines all message types for real-time collaboration

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MessageType {
    Edit,
    Activity,
    Presence,
    Join,
    Leave,
    Notification,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSocketMessage {
    #[serde(rename = "type")]
    pub message_type: MessageType,
    pub document_id: Option<String>,
    pub user_id: Option<String>,
    pub data: Option<Value>,
    pub timestamp: DateTime<Utc>,
    /// Monotonically increasing sequence number for message ordering.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seq: Option<u64>,
    /// Target room for room-filtered delivery. None means global broadcast.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub room_id: Option<String>,
}

impl WebSocketMessage {
    pub fn new(message_type: MessageType) -> Self {
        Self {
            message_type,
            document_id: None,
            user_id: None,
            data: None,
            timestamp: Utc::now(),
            seq: None,
            room_id: None,
        }
    }

    pub fn with_seq(mut self, seq: u64) -> Self {
        self.seq = Some(seq);
        self
    }

    pub fn with_room(mut self, room_id: String) -> Self {
        self.room_id = Some(room_id);
        self
    }

    pub fn with_document(mut self, document_id: String) -> Self {
        self.document_id = Some(document_id);
        self
    }

    pub fn with_user(mut self, user_id: String) -> Self {
        self.user_id = Some(user_id);
        self
    }

    pub fn with_data(mut self, data: Value) -> Self {
        self.data = Some(data);
        self
    }

    pub fn join(document_id: String, user_id: String, user_name: String) -> Self {
        Self::new(MessageType::Join)
            .with_document(document_id)
            .with_user(user_id.clone())
            .with_data(serde_json::json!({ "user_name": user_name }))
    }

    pub fn leave(document_id: String, user_id: String) -> Self {
        Self::new(MessageType::Leave)
            .with_document(document_id)
            .with_user(user_id)
    }

    pub fn edit(document_id: String, user_id: String, edit_data: DocumentEdit) -> Self {
        Self::new(MessageType::Edit)
            .with_document(document_id)
            .with_user(user_id)
            .with_data(serde_json::to_value(edit_data).unwrap_or(Value::Null))
    }

    pub fn activity(document_id: String, user_id: String, activity: ActivityUpdate) -> Self {
        Self::new(MessageType::Activity)
            .with_document(document_id)
            .with_user(user_id)
            .with_data(serde_json::to_value(activity).unwrap_or(Value::Null))
    }

    pub fn presence(document_id: String, users: Vec<PresenceUser>) -> Self {
        Self::new(MessageType::Presence)
            .with_document(document_id)
            .with_data(serde_json::to_value(users).unwrap_or(Value::Null))
    }

    pub fn notification(room_id: String, data: Value) -> Self {
        Self::new(MessageType::Notification)
            .with_room(room_id)
            .with_data(data)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentEdit {
    pub operation_id: String,
    pub operation: EditOperation,
    pub version: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "lowercase")]
pub enum EditOperation {
    Insert {
        position: usize,
        text: String,
    },
    Delete {
        position: usize,
        length: usize,
    },
    Replace {
        position: usize,
        length: usize,
        text: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityUpdate {
    pub activity_type: String,
    pub description: String,
    pub metadata: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresenceUpdate {
    pub document_id: String,
    pub users: Vec<PresenceUser>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresenceUser {
    pub user_id: String,
    pub user_name: String,
    pub cursor_position: usize,
    pub selection: Option<SelectionRange>,
    pub color: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectionRange {
    pub start: usize,
    pub end: usize,
}

/// Presence information for a document.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresenceInfo {
    pub user_id: String,
    pub username: String,
    pub status: PresenceStatus,
    pub cursor_pos: Option<(usize, usize)>,
    pub last_seen: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PresenceStatus {
    Active,
    Idle,
    Away,
}

#[derive(Debug, Clone, PartialEq)]
pub struct HeartbeatConfig {
    pub interval_secs: u64,
    pub timeout_secs: u64,
    pub enabled: bool,
}

impl Default for HeartbeatConfig {
    fn default() -> Self {
        Self {
            interval_secs: 30,
            timeout_secs: 10,
            enabled: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum HeartbeatMessage {
    Ping { timestamp: u64 },
    Pong { timestamp: u64 },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_serialization() {
        let msg = WebSocketMessage::join(
            "doc-1".to_string(),
            "user-1".to_string(),
            "Alice".to_string(),
        );
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("join"));
        assert!(json.contains("doc-1"));
    }

    #[test]
    fn test_edit_operation() {
        let edit = DocumentEdit {
            operation_id: "op-1".to_string(),
            operation: EditOperation::Insert {
                position: 10,
                text: "hello".to_string(),
            },
            version: 1,
        };
        let msg = WebSocketMessage::edit("doc-1".to_string(), "user-1".to_string(), edit);
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("edit"));
    }

    #[test]
    fn test_heartbeat_config_defaults() {
        let config = HeartbeatConfig::default();
        assert_eq!(config.interval_secs, 30);
        assert_eq!(config.timeout_secs, 10);
        assert!(config.enabled);
    }

    #[test]
    fn test_ping_message_serialization() {
        let ping = HeartbeatMessage::Ping {
            timestamp: 1234567890,
        };
        let json = serde_json::to_string(&ping).unwrap();
        assert!(json.contains("\"type\":\"ping\""));
        assert!(json.contains("\"timestamp\":1234567890"));

        let parsed: HeartbeatMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(
            parsed,
            HeartbeatMessage::Ping {
                timestamp: 1234567890
            }
        );
    }

    #[test]
    fn test_pong_message_serialization() {
        let pong = HeartbeatMessage::Pong {
            timestamp: 1234567890,
        };
        let json = serde_json::to_string(&pong).unwrap();
        assert!(json.contains("\"type\":\"pong\""));
        assert!(json.contains("\"timestamp\":1234567890"));

        let parsed: HeartbeatMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(
            parsed,
            HeartbeatMessage::Pong {
                timestamp: 1234567890
            }
        );
    }

    #[test]
    fn test_heartbeat_config_disabled() {
        let config = HeartbeatConfig {
            enabled: false,
            ..Default::default()
        };
        assert!(!config.enabled);
        assert_eq!(config.interval_secs, 30);
        assert_eq!(config.timeout_secs, 10);
    }
}
