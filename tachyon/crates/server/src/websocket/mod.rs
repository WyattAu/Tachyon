// WebSocket module
// Handles real-time document editing and multi-user synchronization

pub mod crdt_handler;
pub mod handler;
pub mod operational_transform;
pub mod redis_relay;
pub mod types;

pub use crdt_handler::{handle_crdt_websocket_upgrade, CrdtConnectionManager};
pub use handler::{
    handle_websocket_upgrade, websocket_upgrade_error, ConnectionManager, WebSocketUpgradeError,
};
pub use operational_transform::{compose, transform, DocumentState, Operation};
pub use types::*;
