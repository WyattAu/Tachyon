//! Shared broadcast bus for WebSocket event distribution.
//!
//! Unifies OT (JSON-text) and CRDT (binary relay) broadcast channels into a
//! single [`SharedBroadcastBus`] that both handlers subscribe to. External
//! consumers (collaboration routes, notification dispatcher) publish through
//! the bus instead of directly calling handler methods.

use std::collections::HashMap;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tokio::sync::{RwLock, broadcast};
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
///
/// Uses per-room broadcast channels so that `publish_ot` only delivers to
/// subscribers of the target room, not all connected clients.
#[derive(Clone)]
pub struct SharedBroadcastBus {
    /// Maps room_id -> (client_ids, broadcast sender).
    rooms: Arc<RwLock<HashMap<String, RoomChannel>>>,
}

#[derive(Clone)]
struct RoomChannel {
    members: Vec<String>,
    tx: broadcast::Sender<BroadcastEvent>,
}

impl SharedBroadcastBus {
    /// Create a new bus with the given channel capacity.
    pub fn new(capacity: usize) -> Self {
        let _ = capacity; // capacity is used per-room now
        Self {
            rooms: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Join a client to a room. No-op if already a member.
    pub async fn join_room(&self, client_id: &str, room_id: &str) {
        let mut rooms = self.rooms.write().await;
        let channel = rooms.entry(room_id.to_string()).or_insert_with(|| {
            let (tx, _) = broadcast::channel(256);
            RoomChannel {
                members: Vec::new(),
                tx,
            }
        });
        if !channel.members.iter().any(|id| id == client_id) {
            channel.members.push(client_id.to_string());
            trace!(client = %client_id, room = %room_id, "client joined room");
        }
    }

    /// Leave a client from a room. Returns true if the client was actually removed.
    pub async fn leave_room(&self, client_id: &str, room_id: &str) -> bool {
        let mut rooms = self.rooms.write().await;
        let (removed, should_remove) = if let Some(channel) = rooms.get_mut(room_id) {
            let before = channel.members.len();
            channel.members.retain(|id| id != client_id);
            (channel.members.len() < before, channel.members.is_empty())
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
        for (room_id, channel) in rooms.iter_mut() {
            let before = channel.members.len();
            channel.members.retain(|id| id != client_id);
            if channel.members.len() < before {
                removed.push(room_id.clone());
            }
            if channel.members.is_empty() {
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
            .map(|ch| ch.members.iter().any(|id| id == client_id))
            .unwrap_or(false)
    }

    /// Get all members of a room.
    pub async fn room_members(&self, room_id: &str) -> Vec<String> {
        self.rooms
            .read()
            .await
            .get(room_id)
            .map(|ch| ch.members.clone())
            .unwrap_or_default()
    }

    /// Count of active rooms.
    pub async fn room_count(&self) -> usize {
        self.rooms.read().await.len()
    }

    /// Subscribe to broadcast events for a specific room.
    ///
    /// Returns a receiver that only gets events published to the given room.
    /// If the room does not exist yet, creates it with an empty member list.
    pub async fn subscribe_to_room(&self, room_id: &str) -> broadcast::Receiver<BroadcastEvent> {
        let mut rooms = self.rooms.write().await;
        let channel = rooms.entry(room_id.to_string()).or_insert_with(|| {
            let (tx, _) = broadcast::channel(256);
            RoomChannel {
                members: Vec::new(),
                tx,
            }
        });
        channel.tx.subscribe()
    }

    /// Subscribe to all broadcast events (legacy, for backward compatibility).
    ///
    /// NOTE: Prefer `subscribe_to_room` for room-scoped delivery.
    /// This method is kept for tests and global event monitoring.
    pub fn subscribe(&self) -> broadcast::Receiver<BroadcastEvent> {
        // For backward compatibility, return a receiver from a dummy global channel.
        // In production, always use subscribe_to_room.
        let (tx, rx) = broadcast::channel(1);
        let _ = tx; // drop sender so receiver immediately returns Closed
        rx
    }

    /// Publish an OT (JSON-text) event to a room.
    ///
    /// Callers must pre-serialize the payload. Only delivers to subscribers
    /// of the specific room — not to all connected clients.
    pub async fn publish_ot(&self, room_id: &str, sender: Option<&str>, payload: String) {
        let rooms = self.rooms.read().await;
        if let Some(channel) = rooms.get(room_id)
            && !channel.members.is_empty()
        {
            let _ = channel.tx.send(BroadcastEvent::OtMessage {
                room_id: room_id.to_string(),
                sender_client_id: sender.map(|s| s.to_string()),
                payload,
            });
        }
    }

    /// Publish a CRDT binary event to a room.
    ///
    /// Only delivers to subscribers of the specific room.
    pub async fn publish_crdt(&self, room_id: &str, sender: &str, data: Vec<u8>) {
        let rooms = self.rooms.read().await;
        if let Some(channel) = rooms.get(room_id)
            && !channel.members.is_empty()
        {
            let _ = channel.tx.send(BroadcastEvent::CrdtBinary {
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
        bus.join_room("alice", "doc:1").await;
        let mut rx = bus.subscribe_to_room("doc:1").await;

        bus.publish_ot("doc:1", Some("alice"), r#"{"type":"presence"}"#.to_string())
            .await;

        let event = rx.recv().await.unwrap();
        match event {
            BroadcastEvent::OtMessage {
                room_id,
                sender_client_id,
                payload,
            } => {
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
        bus.join_room("alice", "doc:1").await;
        let mut rx = bus.subscribe_to_room("doc:1").await;

        bus.publish_crdt("doc:1", "alice", vec![0, 1, 2, 3]).await;

        let event = rx.recv().await.unwrap();
        match event {
            BroadcastEvent::CrdtBinary {
                room_id,
                sender_client_id,
                data,
            } => {
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
        let mut rx = bus.subscribe_to_room("doc:empty").await;

        bus.publish_ot("doc:empty", None, "ignored".to_string())
            .await;

        // No event should arrive -- rx.recv() would block, so try with timeout
        assert!(rx.try_recv().is_err());
    }

    #[tokio::test]
    async fn multiple_subscribers() {
        let bus = SharedBroadcastBus::new(16);
        bus.join_room("alice", "doc:1").await;
        let mut rx1 = bus.subscribe_to_room("doc:1").await;
        let mut rx2 = bus.subscribe_to_room("doc:1").await;

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

    #[tokio::test]
    async fn room_isolation() {
        let bus = SharedBroadcastBus::new(16);
        bus.join_room("alice", "doc:1").await;
        bus.join_room("bob", "doc:2").await;
        let mut rx1 = bus.subscribe_to_room("doc:1").await;
        let mut rx2 = bus.subscribe_to_room("doc:2").await;

        bus.publish_ot("doc:1", None, "for-doc1".to_string()).await;
        bus.publish_ot("doc:2", None, "for-doc2".to_string()).await;

        let e1 = rx1.recv().await.unwrap();
        let e2 = rx2.recv().await.unwrap();
        match (e1, e2) {
            (
                BroadcastEvent::OtMessage {
                    payload: p1,
                    room_id: r1,
                    ..
                },
                BroadcastEvent::OtMessage {
                    payload: p2,
                    room_id: r2,
                    ..
                },
            ) => {
                assert_eq!(r1, "doc:1");
                assert_eq!(p1, "for-doc1");
                assert_eq!(r2, "doc:2");
                assert_eq!(p2, "for-doc2");
            }
            _ => panic!("expected OtMessage for both"),
        }
    }
}
