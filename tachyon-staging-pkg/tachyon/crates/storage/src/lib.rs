//! Tachyon Storage — pluggable storage backends for documents.

// Tachyon Storage — Pluggable document storage backends
//
// Implementations:
// - `SqliteStore`: Local embedded storage via sqlx (offline-first)
// - `MemoryStore`: In-memory HashMap-based storage (testing)
// - `SyncQueue`: Persistent operation journal for offline→online reconciliation

pub mod memory;
pub mod sqlite;
pub mod sync_queue;

pub use memory::MemoryStore;
pub use sqlite::SqliteStore;
pub use sync_queue::{
    FlushResult, SyncEntryStatus, SyncOperation, SyncQueue, SyncQueueEntry, SyncQueueSummary,
};
