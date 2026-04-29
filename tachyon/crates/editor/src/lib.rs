//! Tachyon Editor — native Rust text editor with syntax highlighting and collaboration.

pub mod buffer;
pub mod cursor;
pub mod editor;
pub mod highlight;
pub mod search;
pub mod transaction;

pub use buffer::TextBuffer;
pub use cursor::{Cursor, Selection, SelectionKind};
pub use editor::{Editor, WikilinkState};
pub use highlight::{HighlightSpan, HighlightToken, Highlighter};
pub use search::SearchResult;
pub use transaction::{EditKind, Transaction};
