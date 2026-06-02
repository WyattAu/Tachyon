// WebSocket module
// Handles real-time document editing and multi-user synchronization

pub mod crdt_handler;
pub mod handler;
pub mod operational_transform;
pub mod redis_relay;
pub mod types;

pub use crdt_handler::{CrdtConnectionManager, handle_crdt_websocket_upgrade};
pub use handler::{
    ConnectionManager, WebSocketUpgradeError, handle_websocket_upgrade, websocket_upgrade_error,
};
pub use operational_transform::{DocumentState, Operation, compose, transform};
pub use types::*;
