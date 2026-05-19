// CRDT Collaboration Types
//
// Defines the Yjs-compatible sync protocol for real-time collaboration.
// The server acts as a routing/persistence layer for Yjs binary updates
// without needing to understand the document structure.
//
// Protocol overview:
// 1. Client connects and sends a `sync` message with its Yjs state vector
// 2. Server responds with its own state vector (or empty if new doc)
// 3. Client computes the diff and sends binary Yjs updates
// 4. Server applies updates to its in-memory Yjs document and broadcasts to other clients
// 5. Server periodically persists the merged Yjs document state to the database

use serde::{Deserialize, Serialize};

// ============================================================================
// Yjs Sync Protocol Message Types
// ============================================================================

/// Yjs sync message types (matches the y-protocol spec).
///
/// The sync protocol is:
/// 1. Client sends SyncStep1 (with state vector)
/// 2. Server responds with SyncStep2 (with diff + server state vector)
/// 3. Client sends SyncUpdate (with binary Yjs update)
/// 4. Server broadcasts update to other clients in the room
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum CrdtMessage {
    /// Step 1 of the Yjs sync protocol.
    /// Client sends its state vector; server responds with missing updates.
    #[serde(rename = "sync")]
    Sync(CrtSyncMessage),

    /// Presence update (cursor position, selection, user info).
    #[serde(rename = "presence")]
    Presence(CrtPresenceMessage),

    /// Awareness signal (user typing, viewing, etc.).
    #[serde(rename = "awareness")]
    Awareness(CrdtAwarenessMessage),
}

/// Yjs sync message content.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrtSyncMessage {
    /// The sync step: 1 (request), 2 (reply), or update.
    pub step: u8,
    /// Binary Yjs encoded update (from `Y.encodeStateAsUpdate()` or `Y.diffUpdate()`).
    #[serde(with = "base64_bytes")]
    pub update: Vec<u8>,
}

/// Presence information for a collaborator.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrtPresenceMessage {
    /// The document this presence is for.
    pub document_id: String,
    /// The user's cursor and selection info.
    pub user: CrdtUser,
}

/// A user's collaborative presence state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrdtUser {
    pub user_id: String,
    pub user_name: String,
    /// Absolute cursor position in the document (character offset).
    pub cursor: Option<u32>,
    /// Selection range (start, end) in character offsets.
    pub selection: Option<SelectionRange>,
    /// Color for the user's cursor/selection highlight (hex, e.g. "#ff0000").
    pub color: Option<String>,
}

/// A range in the document (character offsets).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectionRange {
    pub start: u32,
    pub end: u32,
}

/// Awareness signal — lightweight "user is active" message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrdtAwarenessMessage {
    pub document_id: String,
    pub user_id: String,
    pub user_name: String,
    /// The type of awareness: "enter", "leave", "active".
    pub awareness_type: String,
}

// ============================================================================
// CRDT Document State (server-side)
// ============================================================================

/// Server-side CRDT state for a single document.
///
/// Stores the raw Yjs binary update log and the current merged state.
/// The server doesn't parse Yjs data — it just aggregates and routes binary updates.
#[derive(Debug, Clone)]
pub struct CrdtDocumentState {
    /// Document ID.
    pub document_id: String,
    /// Accumulated Yjs update log (concatenated binary updates).
    /// This is what new clients receive as the initial document state.
    pub update_log: Vec<u8>,
    /// Server's Yjs state vector (for computing diffs).
    /// Stored as raw binary from `Y.encodeStateVector()`.
    pub state_vector: Vec<u8>,
    /// Number of connected clients currently editing this document.
    pub active_clients: usize,
    /// Last update timestamp (RFC3339).
    pub last_updated: String,
}

impl CrdtDocumentState {
    /// Create a new empty CRDT document state.
    pub fn new(document_id: impl Into<String>) -> Self {
        Self {
            document_id: document_id.into(),
            update_log: Vec::new(),
            state_vector: Vec::new(),
            active_clients: 0,
            last_updated: chrono::Utc::now().to_rfc3339(),
        }
    }

    /// Create from persisted Yjs state.
    pub fn from_persisted(document_id: impl Into<String>, update_log: Vec<u8>) -> Self {
        Self {
            document_id: document_id.into(),
            update_log,
            state_vector: Vec::new(), // Will be computed by Yjs when a client connects
            active_clients: 0,
            last_updated: chrono::Utc::now().to_rfc3339(),
        }
    }

    /// Append a Yjs update to the log.
    /// Returns the new total size of the update log.
    pub fn append_update(&mut self, update: &[u8]) -> usize {
        self.update_log.extend_from_slice(update);
        self.last_updated = chrono::Utc::now().to_rfc3339();
        self.update_log.len()
    }

    /// Get the current update log for sending to new clients.
    pub fn get_update_log(&self) -> &[u8] {
        &self.update_log
    }

    /// Check if the document has any content.
    pub fn is_empty(&self) -> bool {
        self.update_log.is_empty()
    }

    /// Get the size of the persisted update log in bytes.
    pub fn update_log_size(&self) -> usize {
        self.update_log.len()
    }
}

// ============================================================================
// Collaboration Session Info
// ============================================================================

/// A CRDT operation for offline queuing.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum CrdtOperation {
    Insert { position: u32, text: String },
    Delete { position: u32, length: u32 },
}

/// Queued offline operation awaiting sync.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OfflineOperation {
    pub document_id: String,
    pub client_id: String,
    pub operation: CrdtOperation,
    pub timestamp: u64,
    pub sequence: u64,
}

/// Operation queue for offline edits.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OfflineQueue {
    operations: Vec<OfflineOperation>,
    max_size: usize,
}

impl Default for OfflineQueue {
    fn default() -> Self {
        Self::new(1000)
    }
}

impl OfflineQueue {
    pub fn new(max_size: usize) -> Self {
        Self {
            operations: Vec::new(),
            max_size,
        }
    }

    pub fn push(&mut self, op: OfflineOperation) {
        if self.operations.len() >= self.max_size {
            self.operations.remove(0);
        }
        self.operations.push(op);
    }

    pub fn drain(&mut self) -> Vec<OfflineOperation> {
        std::mem::take(&mut self.operations)
    }

    pub fn is_empty(&self) -> bool {
        self.operations.is_empty()
    }

    pub fn len(&self) -> usize {
        self.operations.len()
    }
}

/// Summary of active collaboration on a document.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollaborationInfo {
    pub document_id: String,
    /// Users currently editing the document.
    pub active_users: Vec<CrdtUser>,
    /// Total number of active connections.
    pub connection_count: usize,
}

// ============================================================================
// Base64 serde helper for Vec<u8>
// ============================================================================

mod base64_bytes {
    use base64::{engine::general_purpose::STANDARD, Engine as _};
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S: Serializer>(data: &Vec<u8>, serializer: S) -> Result<S::Ok, S::Error> {
        let encoded = STANDARD.encode(data);
        serializer.serialize_str(&encoded)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Vec<u8>, D::Error> {
        let encoded: String = String::deserialize(deserializer)?;
        STANDARD.decode(&encoded).map_err(serde::de::Error::custom)
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crdt_document_state_new() {
        let state = CrdtDocumentState::new("doc-123");
        assert_eq!(state.document_id, "doc-123");
        assert!(state.is_empty());
        assert_eq!(state.active_clients, 0);
    }

    #[test]
    fn test_crdt_document_state_append() {
        let mut state = CrdtDocumentState::new("doc-123");
        assert_eq!(state.append_update(&[1, 2, 3]), 3);
        assert_eq!(state.append_update(&[4, 5]), 5);
        assert_eq!(state.get_update_log(), &[1, 2, 3, 4, 5]);
        assert!(!state.is_empty());
    }

    #[test]
    fn test_crdt_document_state_from_persisted() {
        let updates = vec![0x01, 0x02, 0x03, 0x04];
        let state = CrdtDocumentState::from_persisted("doc-456", updates);
        assert_eq!(state.document_id, "doc-456");
        assert_eq!(state.update_log_size(), 4);
    }

    #[test]
    fn test_sync_message_serialization() {
        let msg = CrdtMessage::Sync(CrtSyncMessage {
            step: 1,
            update: vec![0x01, 0x02, 0x03],
        });
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("\"type\":\"sync\""));
        assert!(json.contains("\"step\":1"));

        let decoded: CrdtMessage = serde_json::from_str(&json).unwrap();
        match decoded {
            CrdtMessage::Sync(sync) => {
                assert_eq!(sync.step, 1);
                assert_eq!(sync.update, vec![0x01, 0x02, 0x03]);
            }
            _ => panic!("Expected Sync message"),
        }
    }

    #[test]
    fn test_presence_message_serialization() {
        let msg = CrdtMessage::Presence(CrtPresenceMessage {
            document_id: "doc-1".to_string(),
            user: CrdtUser {
                user_id: "user-1".to_string(),
                user_name: "Alice".to_string(),
                cursor: Some(42),
                selection: Some(SelectionRange { start: 40, end: 55 }),
                color: Some("#ff0000".to_string()),
            },
        });
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("\"type\":\"presence\""));
        assert!(json.contains("Alice"));
        assert!(json.contains("42"));

        let decoded: CrdtMessage = serde_json::from_str(&json).unwrap();
        match decoded {
            CrdtMessage::Presence(p) => {
                assert_eq!(p.document_id, "doc-1");
                assert_eq!(p.user.cursor, Some(42));
                assert_eq!(p.user.color, Some("#ff0000".to_string()));
            }
            _ => panic!("Expected Presence message"),
        }
    }

    #[test]
    fn test_collaboration_info() {
        let info = CollaborationInfo {
            document_id: "doc-1".to_string(),
            active_users: vec![CrdtUser {
                user_id: "u1".to_string(),
                user_name: "Alice".to_string(),
                cursor: Some(10),
                selection: None,
                color: None,
            }],
            connection_count: 1,
        };
        let json = serde_json::to_string(&info).unwrap();
        assert!(json.contains("Alice"));
    }

    #[test]
    fn test_offline_queue_push_and_drain() {
        let mut queue = OfflineQueue::default();
        assert!(queue.is_empty());
        assert_eq!(queue.len(), 0);

        queue.push(OfflineOperation {
            document_id: "doc-1".to_string(),
            client_id: "client-1".to_string(),
            operation: CrdtOperation::Insert {
                position: 0,
                text: "hello".to_string(),
            },
            timestamp: 1000,
            sequence: 1,
        });
        assert_eq!(queue.len(), 1);
        assert!(!queue.is_empty());

        let ops = queue.drain();
        assert_eq!(ops.len(), 1);
        assert!(queue.is_empty());
    }

    #[test]
    fn test_offline_queue_eviction() {
        let mut queue = OfflineQueue::new(3);
        for i in 0..4 {
            queue.push(OfflineOperation {
                document_id: "doc-1".to_string(),
                client_id: "client-1".to_string(),
                operation: CrdtOperation::Insert {
                    position: i,
                    text: format!("op{}", i),
                },
                timestamp: 1000 + i as u64,
                sequence: i as u64,
            });
        }
        assert_eq!(queue.len(), 3);
        let ops = queue.drain();
        assert_eq!(ops[0].sequence, 1);
        assert_eq!(ops[2].sequence, 3);
    }

    #[test]
    fn test_offline_queue_order_preserved() {
        let mut queue = OfflineQueue::default();
        for i in 0..5 {
            queue.push(OfflineOperation {
                document_id: "doc-1".to_string(),
                client_id: "client-1".to_string(),
                operation: CrdtOperation::Delete {
                    position: i,
                    length: 1,
                },
                timestamp: 2000 + i as u64,
                sequence: i as u64,
            });
        }
        let ops = queue.drain();
        for (idx, op) in ops.iter().enumerate() {
            assert_eq!(op.sequence, idx as u64);
        }
    }
}
