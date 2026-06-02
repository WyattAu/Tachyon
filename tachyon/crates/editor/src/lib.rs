//! Tachyon Editor — native Rust text editor with syntax highlighting and collaboration.

pub mod buffer;
pub mod cursor;
pub mod editor;
pub mod highlight;
pub mod search;
pub mod sync_queue;
pub mod transaction;

pub use buffer::TextBuffer;
pub use cursor::{Cursor, Selection, SelectionKind};
pub use editor::{Editor, WikilinkState};
pub use highlight::{
    HighlightProvider, HighlightSpan, HighlightToken, RegexHighlighter, css_class,
};
pub use search::SearchResult;
pub use sync_queue::{OfflineSyncQueue, QueuedUpdate, SyncQueueSummary, SyncStatus};
pub use transaction::{EditKind, Transaction};

/// Backward-compatible alias — `RegexHighlighter` is the canonical name.
#[deprecated(
    since = "0.1.0",
    note = "Use `RegexHighlighter` or `dyn HighlightProvider` instead."
)]
pub use RegexHighlighter as Highlighter;

#[cfg(feature = "native-tree-sitter")]
pub use highlight::composite::CompositeHighlighter;
#[cfg(feature = "native-tree-sitter")]
pub use highlight::tree_sitter::{TreeSitterHighlighter, TsLanguage};
