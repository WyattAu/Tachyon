//! Tachyon Editor — native Rust text editor with syntax highlighting and collaboration.

pub mod buffer;
pub mod cursor;
pub mod transaction;
pub mod highlight;
pub mod search;
pub mod editor;

pub use editor::{Editor, WikilinkState};
pub use buffer::TextBuffer;
pub use cursor::{Cursor, Selection, SelectionKind};
pub use transaction::{Transaction, EditKind};
pub use highlight::{Highlighter, HighlightSpan, HighlightToken};
pub use search::SearchResult;
