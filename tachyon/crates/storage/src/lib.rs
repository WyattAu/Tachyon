// Tachyon Storage — Pluggable document storage backends
//
// Implementations:
// - `SqliteStore`: Local embedded storage via rusqlite (offline-first)
// - `MemoryStore`: In-memory HashMap-based storage (testing)

pub mod sqlite;
pub mod memory;

pub use sqlite::SqliteStore;
pub use memory::MemoryStore;
