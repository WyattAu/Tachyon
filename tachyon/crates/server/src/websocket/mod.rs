// WebSocket module
// Handles real-time document editing and multi-user synchronization

pub mod handler;
pub mod operational_transform;
pub mod types;

pub use handler::{ConnectionManager, WebSocketUpgradeError, handle_websocket_upgrade, websocket_upgrade_error};
pub use operational_transform::{DocumentState, Operation, transform};
pub use types::*;
