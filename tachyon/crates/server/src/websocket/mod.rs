// WebSocket module
// Handles real-time document editing and multi-user synchronization

pub mod crdt_handler;
pub mod redis_relay;
pub mod types;

pub use crdt_handler::{CrdtConnectionManager, handle_crdt_websocket_upgrade};
pub use types::*;
