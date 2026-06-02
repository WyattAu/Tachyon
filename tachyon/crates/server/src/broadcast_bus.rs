//! Shared broadcast bus for WebSocket event distribution.
//!
//! Unifies OT (JSON-text) and CRDT (binary relay) broadcast channels into a
//! single [`SharedBroadcastBus`] that both handlers subscribe to. External
//! consumers (collaboration routes, notification dispatcher) publish through
//! the bus instead of directly calling handler methods.

use std::collections::HashMap;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tokio::sync::{broadcast, RwLock};
use tracing::trace;

/// Broadcast event variants for both OT and CRDT handlers.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BroadcastEvent {
    /// JSON-text WebSocket message (Edit, Join, Leave, Presence, Activity, Notification).
    OtMessage {
        room_id: String,
        sender_client_id: Option<String>,
        payload: String,
    },
    /// Raw binary relay message (y-websocket sync/awareness/selection).
    CrdtBinary {
        room_id: String,
        sender_client_id: String,
        data: Vec<u8>,
    },
}

/// Room-scoped membership + fan-out broadcast.
///
/// Created once in `AppState`, shared via `Arc`. Each handler keeps its own
/// domain state (OT DocumentState, CRDT Yrs docs, presence maps) but delegates
/// room membership and broadcast to this bus.
#[derive(Clone)]
pub struct SharedBroadcastBus {
    rooms: Arc<RwLock<HashMap<String, Vec<String>>>>,
    tx: broadcast::Sender<BroadcastEvent>,
}

impl SharedBroadcastBus {
    /// Create a new bus with the given channel capacity.
    pub fn new(capacity: usize) -> Self {
        let (tx, _) = broadcast::channel(capacity);
        Self {
            rooms: Arc::new(RwLock::new(HashMap::new())),
            tx,
        }
    }

    /// Join a client to a room. No-op if already a member.
    pub async fn join_room(&self, client_id: &str, room_id: &str) {
        let mut rooms = self.rooms.write().await;
        let members = rooms.entry(room_id.to_string()).or_default();
        if !members.iter().any(|id| id == client_id) {
            members.push(client_id.to_string());
            trace!(client = %client_id, room = %room_id, "client joined room");
        }
    }

    /// Leave a client from a room. Returns true if the client was actually removed.
    pub async fn leave_room(&self, client_id: &str, room_id: &str) -> bool {
        let mut rooms = self.rooms.write().await;
        let (removed, should_remove) = if let Some(members) = rooms.get_mut(room_id) {
            let before = members.len();
            members.retain(|id| id != client_id);
            (members.len() < before, members.is_empty())
        } else {
            (false, false)
        };
        if should_remove {
            rooms.remove(room_id);
        }
        if removed {
            trace!(client = %client_id, room = %room_id, "client left room");
        }
        removed
    }

    /// Remove a client from all rooms. Returns list of rooms they were in.
    pub async fn remove_client_from_all_rooms(&self, client_id: &str) -> Vec<String> {
        let mut rooms = self.rooms.write().await;
        let mut removed = Vec::new();
        let mut empty_rooms = Vec::new();
        for (room_id, members) in rooms.iter_mut() {
            let before = members.len();
            members.retain(|id| id != client_id);
            if members.len() < before {
                removed.push(room_id.clone());
            }
            if members.is_empty() {
                empty_rooms.push(room_id.clone());
            }
        }
        for r in empty_rooms {
            rooms.remove(&r);
        }
        if !removed.is_empty() {
            trace!(client = %client_id, rooms = ?removed, "client removed from all rooms");
        }
        removed
    }

    /// Check whether a client is in a given room.
    pub async fn is_client_in_room(&self, client_id: &str, room_id: &str) -> bool {
        self.rooms
            .read()
            .await
            .get(room_id)
            .map(|members| members.iter().any(|id| id == client_id))
            .unwrap_or(false)
    }

    /// Get all members of a room.
    pub async fn room_members(&self, room_id: &str) -> Vec<String> {
        self.rooms
            .read()
            .await
            .get(room_id)
            .cloned()
            .unwrap_or_default()
    }

    /// Count of active rooms.
    pub async fn room_count(&self) -> usize {
        self.rooms.read().await.len()
    }

    /// Subscribe to all broadcast events.
    pub fn subscribe(&self) -> broadcast::Receiver<BroadcastEvent> {
        self.tx.subscribe()
    }

    /// Publish an OT (JSON-text) event to a room.
    ///
    /// Callers must pre-serialize the payload. Only publishes if the room has
    /// at least one member.
    pub async fn publish_ot(&self, room_id: &str, sender: Option<&str>, payload: String) {
        let has_members = self
            .rooms
            .read()
            .await
            .get(room_id)
            .map(|m| !m.is_empty())
            .unwrap_or(false);
        if has_members {
            let _ = self.tx.send(BroadcastEvent::OtMessage {
                room_id: room_id.to_string(),
                sender_client_id: sender.map(|s| s.to_string()),
                payload,
            });
        }
    }

    /// Publish a CRDT binary event to a room.
    ///
    /// Only publishes if the room has at least one member.
    pub async fn publish_crdt(&self, room_id: &str, sender: &str, data: Vec<u8>) {
        let has_members = self
            .rooms
            .read()
            .await
            .get(room_id)
            .map(|m| !m.is_empty())
            .unwrap_or(false);
        if has_members {
            let _ = self.tx.send(BroadcastEvent::CrdtBinary {
                room_id: room_id.to_string(),
                sender_client_id: sender.to_string(),
                data,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn new_bus_is_empty() {
        let bus = SharedBroadcastBus::new(16);
        assert_eq!(bus.room_count().await, 0);
    }

    #[tokio::test]
    async fn join_and_leave_room() {
        let bus = SharedBroadcastBus::new(16);
        bus.join_room("alice", "doc:1").await;
        assert!(bus.is_client_in_room("alice", "doc:1").await);
        assert_eq!(bus.room_members("doc:1").await, vec!["alice".to_string()]);

        let removed = bus.leave_room("alice", "doc:1").await;
        assert!(removed);
        assert!(!bus.is_client_in_room("alice", "doc:1").await);
        assert_eq!(bus.room_count().await, 0);
    }

    #[tokio::test]
    async fn leave_nonexistent_client_returns_false() {
        let bus = SharedBroadcastBus::new(16);
        let removed = bus.leave_room("ghost", "doc:1").await;
        assert!(!removed);
    }

    #[tokio::test]
    async fn join_room_idempotent() {
        let bus = SharedBroadcastBus::new(16);
        bus.join_room("alice", "doc:1").await;
        bus.join_room("alice", "doc:1").await;
        assert_eq!(bus.room_members("doc:1").await.len(), 1);
    }

    #[tokio::test]
    async fn remove_from_all_rooms() {
        let bus = SharedBroadcastBus::new(16);
        bus.join_room("alice", "doc:1").await;
        bus.join_room("alice", "doc:2").await;
        bus.join_room("alice", "user:alice").await;

        let removed = bus.remove_client_from_all_rooms("alice").await;
        assert_eq!(removed.len(), 3);
        assert_eq!(bus.room_count().await, 0);
    }

    #[tokio::test]
    async fn publish_ot_receives_event() {
        let bus = SharedBroadcastBus::new(16);
        let mut rx = bus.subscribe();
        bus.join_room("alice", "doc:1").await;

        bus.publish_ot("doc:1", Some("alice"), r#"{"type":"presence"}"#.to_string())
            .await;

        let event = rx.recv().await.unwrap();
        match event {
            BroadcastEvent::OtMessage { room_id, sender_client_id, payload } => {
                assert_eq!(room_id, "doc:1");
                assert_eq!(sender_client_id, Some("alice".to_string()));
                assert_eq!(payload, r#"{"type":"presence"}"#);
            }
            BroadcastEvent::CrdtBinary { .. } => panic!("expected OtMessage"),
        }
    }

    #[tokio::test]
    async fn publish_crdt_receives_event() {
        let bus = SharedBroadcastBus::new(16);
        let mut rx = bus.subscribe();
        bus.join_room("alice", "doc:1").await;

        bus.publish_crdt("doc:1", "alice", vec![0, 1, 2, 3]).await;

        let event = rx.recv().await.unwrap();
        match event {
            BroadcastEvent::CrdtBinary { room_id, sender_client_id, data } => {
                assert_eq!(room_id, "doc:1");
                assert_eq!(sender_client_id, "alice");
                assert_eq!(data, vec![0, 1, 2, 3]);
            }
            BroadcastEvent::OtMessage { .. } => panic!("expected CrdtBinary"),
        }
    }

    #[tokio::test]
    async fn publish_to_empty_room_dropped() {
        let bus = SharedBroadcastBus::new(16);
        let mut rx = bus.subscribe();

        bus.publish_ot("doc:empty", None, "ignored".to_string()).await;

        // No event should arrive -- rx.recv() would block, so try with timeout
        assert!(rx.try_recv().is_err());
    }

    #[tokio::test]
    async fn multiple_subscribers() {
        let bus = SharedBroadcastBus::new(16);
        let mut rx1 = bus.subscribe();
        let mut rx2 = bus.subscribe();
        bus.join_room("alice", "doc:1").await;

        bus.publish_ot("doc:1", None, "hello".to_string()).await;

        let e1 = rx1.recv().await.unwrap();
        let e2 = rx2.recv().await.unwrap();
        match (e1, e2) {
            (
                BroadcastEvent::OtMessage { payload: p1, .. },
                BroadcastEvent::OtMessage { payload: p2, .. },
            ) => {
                assert_eq!(p1, "hello");
                assert_eq!(p2, "hello");
            }
            _ => panic!("expected OtMessage"),
        }
    }
}
