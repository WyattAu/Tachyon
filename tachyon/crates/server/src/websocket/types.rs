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
        }
    }

    pub fn with_seq(mut self, seq: u64) -> Self {
        self.seq = Some(seq);
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
}
