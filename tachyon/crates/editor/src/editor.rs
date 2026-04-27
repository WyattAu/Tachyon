use std::time::{SystemTime, UNIX_EPOCH};

use crate::buffer::TextBuffer;
use crate::cursor::{Cursor, Selection, SelectionKind};
use crate::highlight::{HighlightSpan, Highlighter};
use crate::search::{Search, SearchResult};
use crate::transaction::{EditKind, Transaction, UndoStack};
use yrs::updates::decoder::Decode;
use yrs::updates::encoder::Encode;
use yrs::{Doc, GetString, ReadTxn, Text, TextRef, Transact, Update};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct WikilinkState {
    pub active: bool,
    pub query: String,
    pub start_line: usize,
    pub start_col: usize,
}

fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

pub struct Editor {
    buffer: TextBuffer,
    cursor: Cursor,
    selection: Selection,
    undo_stack: UndoStack,
    highlighter: Highlighter,
    search: Search,
    is_dirty: bool,
    #[allow(dead_code)]
    word_wrap: bool,
    tab_size: usize,
    #[allow(dead_code)]
    auto_indent: bool,
    #[allow(dead_code)]
    bracket_matching: bool,
    #[allow(dead_code)]
    line_numbers: bool,
    current_search_results: Vec<SearchResult>,
    current_search_index: usize,
    in_code_block: bool,
    scroll_offset_lines: usize,
    scroll_offset_cols: usize,
    /// CRDT document — source of truth for collaboration sync.
    /// All local edits are mirrored here; remote updates rebuild the rope.
    crdt_doc: Doc,
    crdt_text: TextRef,
    /// Tracks the last encoded state vector for incremental updates.
    last_encoded_state: Vec<u8>,
}

impl Editor {
    pub fn new() -> Self {
        let crdt_doc = Doc::new();
        let crdt_text = crdt_doc.get_or_insert_text("content");
        let last_encoded_state = crdt_doc.transact().state_vector().encode_v1();
        Self {
            buffer: TextBuffer::new(),
            cursor: Cursor::zero(),
            selection: Selection::caret(Cursor::zero()),
            undo_stack: UndoStack::new(1000),
            highlighter: Highlighter::new(),
            search: Search::new(),
            is_dirty: false,
            word_wrap: false,
            tab_size: 2,
            auto_indent: true,
            bracket_matching: true,
            line_numbers: true,
            current_search_results: Vec::new(),
            current_search_index: 0,
            in_code_block: false,
            scroll_offset_lines: 0,
            scroll_offset_cols: 0,
            crdt_doc,
            crdt_text,
            last_encoded_state,
        }
    }

    pub fn with_content(content: &str) -> Self {
        let mut editor = Self::new();
        editor.set_content(content);
        editor
    }

    pub fn buffer(&self) -> &TextBuffer {
        &self.buffer
    }

    pub fn buffer_mut(&mut self) -> &mut TextBuffer {
        &mut self.buffer
    }

    pub fn cursor(&self) -> &Cursor {
        &self.cursor
    }

    pub fn selection(&self) -> &Selection {
        &self.selection
    }

    pub fn is_dirty(&self) -> bool {
        self.is_dirty
    }

    pub fn set_content(&mut self, content: &str) {
        self.buffer = TextBuffer::from_str(content);
        self.cursor = Cursor::zero();
        self.selection = Selection::caret(Cursor::zero());
        self.undo_stack.clear();
        self.is_dirty = false;
        self.current_search_results.clear();
        self.in_code_block = false;
        self.sync_rope_to_yrs();
    }

    pub fn content(&self) -> String {
        self.buffer.to_string()
    }

    // ─── CRDT Sync Methods ───────────────────────────────────────────────

    /// Mirror the current rope content into the CRDT document.
    /// Called after local edits so the CRDT layer stays in sync.
    fn sync_rope_to_yrs(&mut self) {
        let content = self.buffer.to_string();
        let len = self.crdt_text.len(&self.crdt_doc.transact());
        let content_len = content.chars().count();
        let mut txn = self.crdt_doc.transact_mut();
        if len > 0 {
            self.crdt_text.remove_range(&mut txn, 0, len);
        }
        if content_len > 0 {
            self.crdt_text.insert(&mut txn, 0, &content);
        }
    }

    /// Rebuild the rope from the CRDT document content.
    /// Called after applying remote updates.
    fn sync_rope_from_yrs(&mut self) {
        let txn = self.crdt_doc.transact();
        let content = self.crdt_text.get_string(&txn);
        drop(txn);
        self.buffer = TextBuffer::from_str(&content);
    }

    /// Encode the CRDT document state as an incremental update.
    /// Returns the binary diff since the last call to `encode_update`.
    /// Returns empty Vec if no changes.
    pub fn encode_update(&mut self) -> Vec<u8> {
        let txn = self.crdt_doc.transact();
        let current_sv = txn.state_vector();
        let update = txn.encode_diff_v1(
            &yrs::StateVector::decode_v1(&self.last_encoded_state).unwrap_or_default(),
        );
        self.last_encoded_state = current_sv.encode_v1();
        update
    }

    /// Apply a remote CRDT update (binary) and rebuild the rope.
    /// Returns true if the rope content changed.
    pub fn apply_remote_update(&mut self, update: &[u8]) -> bool {
        let old_content = self.buffer.to_string();
        let decoded = match Update::decode_v1(update) {
            Ok(u) => u,
            Err(_) => return false,
        };
        let result = self.crdt_doc.transact_mut().apply_update(decoded);
        match result {
            Ok(()) => {
                self.sync_rope_from_yrs();
                let new_content = self.buffer.to_string();
                // Clamp cursor to new buffer bounds
                let max_line = self.buffer.len_lines().saturating_sub(1);
                if self.cursor.line > max_line {
                    self.cursor.line = max_line;
                    self.cursor.col = self.buffer.line_len(max_line);
                } else {
                    self.cursor.col = self.cursor.col.min(self.buffer.line_len(self.cursor.line));
                }
                self.selection = Selection::caret(self.cursor);
                old_content != new_content
            }
            Err(_) => false,
        }
    }

    /// Get the full CRDT document state as a binary update.
    /// Used for initial sync when joining a collaboration session.
    pub fn encode_full_state(&self) -> Vec<u8> {
        self.crdt_doc
            .transact()
            .encode_state_as_update_v1(&yrs::StateVector::default())
    }

    /// Get the current CRDT state vector (for requesting incremental updates).
    pub fn state_vector(&self) -> Vec<u8> {
        self.crdt_doc.transact().state_vector().encode_v1()
    }

    fn push_transaction(&mut self, kind: EditKind, start: Cursor, end: Cursor) {
        let tx = Transaction {
            kind,
            start,
            end,
            timestamp: current_timestamp(),
        };
        self.undo_stack.push(tx);
        self.is_dirty = true;
        self.sync_rope_to_yrs();
    }

    pub fn insert_text(&mut self, text: &str) {
        if !self.selection.is_empty() {
            self.delete_selection();
        }

        let start = self.cursor;
        self.buffer.insert(self.cursor.line, self.cursor.col, text);

        let lines_added = text.matches('\n').count();
        let last_line_len = text.rsplit('\n').next().map(|l| l.len()).unwrap_or(0);

        if lines_added > 0 {
            let new_col = last_line_len;
            let new_line = self.cursor.line + lines_added;
            self.cursor = Cursor::new(new_line, new_col);
        } else {
            self.cursor.col += text.chars().count();
        }

        self.push_transaction(
            EditKind::Insert {
                text: text.to_string(),
            },
            start,
            self.cursor,
        );

        self.selection = Selection::caret(self.cursor);
    }

    pub fn delete_selection(&mut self) -> Option<String> {
        if self.selection.is_empty() {
            return None;
        }

        let (start, end) = self.selection.normalize();
        let deleted = self
            .buffer
            .delete_range(start.line, start.col, end.line, end.col);
        self.cursor = start;
        self.selection = Selection::caret(self.cursor);

        if !deleted.is_empty() {
            self.push_transaction(
                EditKind::Delete {
                    text: deleted.clone(),
                },
                start,
                self.cursor,
            );
        }

        Some(deleted)
    }

    pub fn delete_backwards(&mut self) {
        if !self.selection.is_empty() {
            self.delete_selection();
            return;
        }

        if self.cursor.col > 0 {
            let start = Cursor::new(self.cursor.line, self.cursor.col - 1);
            let deleted =
                self.buffer
                    .delete_range(start.line, start.col, self.cursor.line, self.cursor.col);
            self.cursor.col -= 1;
            self.selection = Selection::caret(self.cursor);
            if !deleted.is_empty() {
                self.push_transaction(EditKind::Delete { text: deleted }, self.cursor, self.cursor);
            }
        } else if self.cursor.line > 0 {
            let prev_line_len = self.buffer.line_len(self.cursor.line - 1);
            let start = Cursor::new(self.cursor.line - 1, prev_line_len);
            let deleted =
                self.buffer
                    .delete_range(start.line, start.col, self.cursor.line, self.cursor.col);
            self.cursor.line -= 1;
            self.cursor.col = prev_line_len;
            self.selection = Selection::caret(self.cursor);
            if !deleted.is_empty() {
                self.push_transaction(EditKind::Delete { text: deleted }, self.cursor, self.cursor);
            }
        }
    }

    pub fn delete_forwards(&mut self) {
        if !self.selection.is_empty() {
            self.delete_selection();
            return;
        }

        let line_len = self.buffer.line_len(self.cursor.line);
        if self.cursor.col < line_len {
            let start = self.cursor;
            let deleted = self.buffer.delete_range(
                self.cursor.line,
                self.cursor.col,
                self.cursor.line,
                self.cursor.col + 1,
            );
            self.selection = Selection::caret(self.cursor);
            if !deleted.is_empty() {
                self.push_transaction(EditKind::Delete { text: deleted }, start, self.cursor);
            }
        } else if self.cursor.line < self.buffer.len_lines() - 1 {
            let start = self.cursor;
            let deleted = self.buffer.delete_range(
                self.cursor.line,
                self.cursor.col,
                self.cursor.line + 1,
                0,
            );
            self.selection = Selection::caret(self.cursor);
            if !deleted.is_empty() {
                self.push_transaction(EditKind::Delete { text: deleted }, start, self.cursor);
            }
        }
    }

    pub fn delete_word_backwards(&mut self) {
        let start = self.cursor;
        self.cursor.move_word_left(&self.buffer);
        if self.cursor != start {
            let deleted =
                self.buffer
                    .delete_range(self.cursor.line, self.cursor.col, start.line, start.col);
            self.selection = Selection::caret(self.cursor);
            if !deleted.is_empty() {
                self.push_transaction(EditKind::Delete { text: deleted }, start, self.cursor);
            }
        }
    }

    pub fn delete_word_forwards(&mut self) {
        let start = self.cursor;
        self.cursor.move_word_right(&self.buffer);
        if self.cursor != start {
            let deleted =
                self.buffer
                    .delete_range(start.line, start.col, self.cursor.line, self.cursor.col);
            let end = Cursor::new(start.line, start.col);
            self.cursor = end;
            self.selection = Selection::caret(self.cursor);
            if !deleted.is_empty() {
                self.push_transaction(EditKind::Delete { text: deleted }, start, end);
            }
        }
    }

    pub fn delete_line(&mut self) {
        let line = self.cursor.line;
        let line_text = self.buffer.line_content_with_newline(line);
        let start = Cursor::new(line, 0);

        if line < self.buffer.len_lines() - 1 {
            self.buffer.delete_range(line, 0, line + 1, 0);
        } else if line > 0 {
            let prev_len = self.buffer.line_len(line - 1);
            self.buffer
                .delete_range(line - 1, prev_len, line, self.buffer.line_len(line));
            self.cursor.line = line - 1;
            self.cursor.col = prev_len;
        } else {
            self.buffer.delete_range(0, 0, 0, self.buffer.line_len(0));
            self.cursor.col = 0;
        }

        self.selection = Selection::caret(self.cursor);
        self.push_transaction(EditKind::Delete { text: line_text }, start, self.cursor);
    }

    pub fn indent_line(&mut self, line: usize) {
        let indent = " ".repeat(self.tab_size);
        self.buffer.insert(line, 0, &indent);
        if self.cursor.line == line && self.cursor.col > 0 {
            self.cursor.col += self.tab_size;
        }
        self.is_dirty = true;
        self.sync_rope_to_yrs();
    }

    pub fn unindent_line(&mut self, line: usize) {
        let current_indent = self.buffer.indent_level(line);
        let remove = current_indent.min(self.tab_size);
        if remove > 0 {
            self.buffer.delete_range(line, 0, line, remove);
            if self.cursor.line == line {
                self.cursor.col = self.cursor.col.saturating_sub(remove);
            }
            self.is_dirty = true;
            self.sync_rope_to_yrs();
        }
    }

    pub fn indent_selection(&mut self) {
        let (start, end) = self.selection.normalize();
        let start_line = start.line;
        let end_line = if end.col > 0 {
            end.line
        } else {
            end.line.saturating_sub(1)
        };

        for line in start_line..=end_line.min(self.buffer.len_lines() - 1) {
            self.indent_line(line);
        }
    }

    pub fn unindent_selection(&mut self) {
        let (start, end) = self.selection.normalize();
        let start_line = start.line;
        let end_line = if end.col > 0 {
            end.line
        } else {
            end.line.saturating_sub(1)
        };

        for line in start_line..=end_line.min(self.buffer.len_lines() - 1) {
            self.unindent_line(line);
        }
    }

    pub fn toggle_line_comment(&mut self) {
        let line = self.cursor.line;
        let line_text = self.buffer.line(line);
        let stripped = line_text.trim_start_matches('\n');

        if stripped.starts_with("// ") || stripped == "//" {
            let full_prefix_len = line_text.len() - stripped.len();
            let comment_len = if stripped == "//" { 2 } else { 3 };
            self.buffer
                .delete_range(line, full_prefix_len, line, full_prefix_len + comment_len);
            self.cursor.col = self.cursor.col.saturating_sub(comment_len);
        } else {
            let indent = self.buffer.indent_level(line);
            self.buffer.insert(line, indent, "// ");
            if self.cursor.col >= indent {
                self.cursor.col += 3;
            }
        }
        self.is_dirty = true;
        self.sync_rope_to_yrs();
    }

    pub fn duplicate_line(&mut self) {
        let line = self.cursor.line;
        let line_text = self.buffer.line_content_with_newline(line);
        if !line_text.ends_with('\n') {
            self.buffer.insert(line, self.buffer.line_len(line), "\n");
        }
        self.buffer
            .insert(line + 1, 0, line_text.trim_end_matches('\n'));
        self.buffer
            .insert(line + 1, self.buffer.line_len(line + 1), "\n");
        self.cursor.line += 1;
        self.selection = Selection::caret(self.cursor);
        self.is_dirty = true;
        self.sync_rope_to_yrs();
    }

    pub fn move_line_up(&mut self) {
        if self.cursor.line == 0 {
            return;
        }
        let line = self.cursor.line;
        let current_text = self.buffer.line(line).trim_end_matches('\n').to_string();
        let prev_text = self
            .buffer
            .line(line - 1)
            .trim_end_matches('\n')
            .to_string();
        let current_col = self.cursor.col;

        let replacement = format!("{}\n{}\n", current_text, prev_text);

        self.buffer.delete_range(line - 1, 0, line + 1, 0);
        self.buffer.insert(line - 1, 0, &replacement);

        self.cursor.line -= 1;
        self.cursor.col = current_col.min(self.buffer.line_len(self.cursor.line));
    }

    pub fn move_line_down(&mut self) {
        if self.cursor.line >= self.buffer.len_lines() - 1 {
            return;
        }
        let line = self.cursor.line;
        let current_text = self.buffer.line(line).trim_end_matches('\n').to_string();
        let next_text = self
            .buffer
            .line(line + 1)
            .trim_end_matches('\n')
            .to_string();
        let current_col = self.cursor.col;

        let replacement = format!("{}\n{}\n", next_text, current_text);
        self.buffer.delete_range(line, 0, line + 2, 0);
        self.buffer.insert(line, 0, &replacement);

        self.cursor.line += 1;
        self.cursor.col = current_col.min(self.buffer.line_len(self.cursor.line));
    }

    pub fn get_wikilink_state(&self) -> Option<WikilinkState> {
        let line_text = self.buffer.line(self.cursor.line);
        let trimmed = line_text.trim_end_matches('\n');
        let before_cursor = &trimmed[..self.cursor.col.min(trimmed.len())];

        if let Some(start) = before_cursor.rfind("[[") {
            let after_start = &before_cursor[start + 2..];
            if after_start.contains("]]") {
                return None;
            }
            Some(WikilinkState {
                active: true,
                query: after_start.to_string(),
                start_line: self.cursor.line,
                start_col: start,
            })
        } else {
            None
        }
    }

    pub fn insert_wikilink(&mut self, title: &str, alias: Option<&str>) {
        if let Some(wl_state) = self.get_wikilink_state() {
            let replacement = match alias {
                Some(a) => format!("[[{}|{}]]", title, a),
                None => format!("[[{}]]", title),
            };

            self.cursor = Cursor::new(wl_state.start_line, wl_state.start_col);
            let end = Cursor::new(self.cursor.line, wl_state.start_col + 2 + wl_state.query.len());
            self.selection = Selection::range(self.cursor, end);
            self.delete_selection();

            self.insert_text(&replacement);
        }
    }

    pub fn undo(&mut self) -> bool {
        let tx = match self.undo_stack.pop_undo() {
            Some(t) => t,
            None => return false,
        };

        let inverse = match tx.kind {
            EditKind::Insert { text } => {
                let end = tx.end;
                self.buffer
                    .delete_range(tx.start.line, tx.start.col, end.line, end.col);
                self.cursor = tx.start;
                EditKind::Delete { text }
            }
            EditKind::Delete { text } => {
                self.buffer.insert(tx.start.line, tx.start.col, &text);
                let lines_added = text.matches('\n').count();
                if lines_added > 0 {
                    let last_line_len = text.rsplit('\n').next().map(|l| l.len()).unwrap_or(0);
                    self.cursor = Cursor::new(tx.start.line + lines_added, last_line_len);
                } else {
                    self.cursor = Cursor::new(tx.start.line, tx.start.col + text.chars().count());
                }
                EditKind::Insert { text }
            }
            EditKind::Replace { old_text, new_text } => {
                let end = tx.end;
                self.buffer
                    .delete_range(tx.start.line, tx.start.col, end.line, end.col);
                self.buffer.insert(tx.start.line, tx.start.col, &old_text);
                let lines_added = old_text.matches('\n').count();
                if lines_added > 0 {
                    let last_line_len = old_text.rsplit('\n').next().map(|l| l.len()).unwrap_or(0);
                    self.cursor = Cursor::new(tx.start.line + lines_added, last_line_len);
                } else {
                    self.cursor =
                        Cursor::new(tx.start.line, tx.start.col + old_text.chars().count());
                }
                EditKind::Replace {
                    old_text: new_text,
                    new_text: old_text,
                }
            }
        };

        self.undo_stack.push_redo(Transaction {
            kind: inverse,
            start: tx.start,
            end: self.cursor,
            timestamp: tx.timestamp,
        });

        self.selection = Selection::caret(self.cursor);
        true
    }

    pub fn redo(&mut self) -> bool {
        let tx = match self.undo_stack.pop_redo() {
            Some(t) => t,
            None => return false,
        };

        let inverse = match &tx.kind {
            EditKind::Insert { text } => {
                self.buffer.insert(tx.start.line, tx.start.col, text);
                let lines_added = text.matches('\n').count();
                if lines_added > 0 {
                    let last_line_len = text.rsplit('\n').next().map(|l| l.len()).unwrap_or(0);
                    self.cursor = Cursor::new(tx.start.line + lines_added, last_line_len);
                } else {
                    self.cursor = Cursor::new(tx.start.line, tx.start.col + text.chars().count());
                }
                EditKind::Delete { text: text.clone() }
            }
            EditKind::Delete { text } => {
                self.buffer.insert(tx.start.line, tx.start.col, text);
                self.cursor = tx.start;
                EditKind::Insert { text: text.clone() }
            }
            EditKind::Replace { old_text, new_text } => {
                self.buffer.insert(tx.start.line, tx.start.col, new_text);
                let lines_added = new_text.matches('\n').count();
                if lines_added > 0 {
                    let last_line_len = new_text.rsplit('\n').next().map(|l| l.len()).unwrap_or(0);
                    self.cursor = Cursor::new(tx.start.line + lines_added, last_line_len);
                } else {
                    self.cursor =
                        Cursor::new(tx.start.line, tx.start.col + new_text.chars().count());
                }
                EditKind::Replace {
                    old_text: new_text.clone(),
                    new_text: old_text.clone(),
                }
            }
        };

        self.undo_stack.push_undo(Transaction {
            kind: inverse,
            start: tx.start,
            end: self.cursor,
            timestamp: tx.timestamp,
        });

        self.selection = Selection::caret(self.cursor);
        true
    }

    pub fn can_undo(&self) -> bool {
        self.undo_stack.can_undo()
    }

    pub fn can_redo(&self) -> bool {
        self.undo_stack.can_redo()
    }

    pub fn select_all(&mut self) {
        self.cursor.move_to_buffer_end(&self.buffer);
        let end = self.cursor;
        let start = Cursor::zero();
        self.selection = Selection::range(start, end);
    }

    pub fn select_word(&mut self) {
        let line_text = self.buffer.line(self.cursor.line);
        let chars: Vec<char> = line_text.chars().collect();
        let col = self.cursor.col.min(chars.len());

        if col >= chars.len() {
            self.selection = Selection::caret(self.cursor);
            return;
        }

        let is_word_char = |c: char| c.is_alphanumeric() || c == '_';

        let mut start = col;
        while start > 0 && is_word_char(chars[start - 1]) {
            start -= 1;
        }

        let mut end = col;
        while end < chars.len() && is_word_char(chars[end]) {
            end += 1;
        }

        self.selection = Selection::range(
            Cursor::new(self.cursor.line, start),
            Cursor::new(self.cursor.line, end),
        );
    }

    pub fn select_line(&mut self) {
        let line = self.cursor.line;
        self.selection = Selection::range(
            Cursor::new(line, 0),
            Cursor::new(line, self.buffer.line_len(line)),
        );
    }

    pub fn extend_selection_to(&mut self, cursor: Cursor) {
        if self.selection.is_empty() {
            self.selection.anchor = self.cursor;
        }
        self.selection.cursor = cursor;
        self.selection.kind = SelectionKind::Range;
    }

    pub fn move_cursor_left(&mut self) {
        if self.selection.kind != SelectionKind::Caret {
            let (start, _) = self.selection.normalize();
            self.cursor = start;
            self.selection = Selection::caret(self.cursor);
            return;
        }
        self.cursor.move_left(&self.buffer);
        self.selection = Selection::caret(self.cursor);
    }

    pub fn move_cursor_right(&mut self) {
        if self.selection.kind != SelectionKind::Caret {
            let (_, end) = self.selection.normalize();
            self.cursor = end;
            self.selection = Selection::caret(self.cursor);
            return;
        }
        self.cursor.move_right(&self.buffer);
        self.selection = Selection::caret(self.cursor);
    }

    pub fn move_cursor_up(&mut self) {
        if self.selection.kind != SelectionKind::Caret {
            let (start, _) = self.selection.normalize();
            self.cursor = start;
            self.selection = Selection::caret(self.cursor);
            return;
        }
        self.cursor.move_up(&self.buffer);
        self.selection = Selection::caret(self.cursor);
    }

    pub fn move_cursor_down(&mut self) {
        if self.selection.kind != SelectionKind::Caret {
            let (_, end) = self.selection.normalize();
            self.cursor = end;
            self.selection = Selection::caret(self.cursor);
            return;
        }
        self.cursor.move_down(&self.buffer);
        self.selection = Selection::caret(self.cursor);
    }

    pub fn move_cursor_home(&mut self) {
        let line_text = self.buffer.line(self.cursor.line);
        let first_non_ws = line_text
            .chars()
            .position(|c| !c.is_whitespace())
            .unwrap_or(0);

        if self.cursor.col == first_non_ws {
            self.cursor.col = 0;
        } else {
            self.cursor.col = first_non_ws;
        }
        self.selection = Selection::caret(self.cursor);
    }

    pub fn move_cursor_end(&mut self) {
        self.cursor.move_end(&self.buffer);
        self.selection = Selection::caret(self.cursor);
    }

    pub fn move_cursor_to(&mut self, line: usize, col: usize) {
        self.cursor.line = line.min(self.buffer.len_lines() - 1);
        self.cursor.col = col.min(self.buffer.line_len(self.cursor.line));
        self.selection = Selection::caret(self.cursor);
    }

    pub fn page_up(&mut self, page_size: usize) {
        let new_line = self.cursor.line.saturating_sub(page_size);
        self.move_cursor_to(new_line, self.cursor.col);
    }

    pub fn page_down(&mut self, page_size: usize) {
        let new_line = (self.cursor.line + page_size).min(self.buffer.len_lines() - 1);
        self.move_cursor_to(new_line, self.cursor.col);
    }

    pub fn find(&mut self, query: &str) -> Vec<SearchResult> {
        self.current_search_results = self.search.find(query, &self.buffer);
        self.current_search_index = 0;
        self.current_search_results.clone()
    }

    pub fn find_next(&mut self) {
        if self.current_search_results.is_empty() {
            return;
        }
        self.current_search_index =
            (self.current_search_index + 1) % self.current_search_results.len();
        if let Some(result) = self.current_search_results.get(self.current_search_index) {
            self.cursor = Cursor::new(result.line, result.start_col);
            self.selection = Selection::caret(self.cursor);
        }
    }

    pub fn find_previous(&mut self) {
        if self.current_search_results.is_empty() {
            return;
        }
        self.current_search_index = if self.current_search_index == 0 {
            self.current_search_results.len() - 1
        } else {
            self.current_search_index - 1
        };
        if let Some(result) = self.current_search_results.get(self.current_search_index) {
            self.cursor = Cursor::new(result.line, result.start_col);
            self.selection = Selection::caret(self.cursor);
        }
    }

    pub fn replace_next(&mut self, replacement: &str) -> bool {
        if self.current_search_results.is_empty() {
            return false;
        }
        let result = match self.current_search_results.get(self.current_search_index) {
            Some(r) => r.clone(),
            None => return false,
        };

        match self
            .search
            .replace_next(&mut self.buffer, replacement, &result)
        {
            Some(tx) => {
                self.undo_stack.push(tx);
                self.cursor = Cursor::new(result.line, result.start_col + replacement.len());
                self.selection = Selection::caret(self.cursor);
                self.is_dirty = true;
                self.sync_rope_to_yrs();
                true
            }
            None => false,
        }
    }

    pub fn replace_all(&mut self, query: &str, replacement: &str) -> usize {
        let txs = self
            .search
            .replace_all(&mut self.buffer, query, replacement);
        let count = txs.len();
        for tx in txs {
            self.undo_stack.push(tx);
        }
        if count > 0 {
            self.is_dirty = true;
            self.sync_rope_to_yrs();
        }
        count
    }

    pub fn highlight_line(&mut self, line_idx: usize) -> Vec<HighlightSpan> {
        if line_idx >= self.buffer.len_lines() {
            return Vec::new();
        }
        let line = self.buffer.line(line_idx);
        let line_text = line.trim_end_matches('\n');
        self.highlighter
            .highlight_line(line_text, &mut self.in_code_block)
    }

    pub fn rehighlight_all(&mut self) -> Vec<Vec<HighlightSpan>> {
        self.in_code_block = false;
        let mut all_spans = Vec::with_capacity(self.buffer.len_lines());
        for i in 0..self.buffer.len_lines() {
            all_spans.push(self.highlight_line(i));
        }
        all_spans
    }

    pub fn scroll_offset(&self) -> (usize, usize) {
        (self.scroll_offset_lines, self.scroll_offset_cols)
    }

    pub fn set_scroll_offset(&mut self, lines: usize, cols: usize) {
        self.scroll_offset_lines = lines;
        self.scroll_offset_cols = cols;
    }

    pub fn auto_indent_newline(&mut self) {
        let line = self.cursor.line;
        let indent = self.buffer.indent_level(line);
        let line_text = self.buffer.line(line);
        let trimmed = line_text.trim_start_matches('\n').trim_start();

        let extra_indent = if trimmed.starts_with(['{', '(', '[']) {
            self.tab_size
        } else {
            0
        };

        let new_indent = " ".repeat(indent + extra_indent);
        self.insert_text(&format!("\n{}", new_indent));
    }

    pub fn auto_close_bracket(&mut self, bracket: char) {
        let pairs = [('(', ')'), ('[', ']'), ('{', '}'), ('"', '"'), ('\'', '\'')];
        let closing = pairs
            .iter()
            .find(|&&(open, _)| open == bracket)
            .map(|&(_, close)| close);

        self.insert_text(&bracket.to_string());

        if let Some(close) = closing {
            self.buffer
                .insert(self.cursor.line, self.cursor.col, &close.to_string());
        }
    }

    pub fn current_line(&self) -> usize {
        self.cursor.line
    }

    pub fn current_line_text(&self) -> String {
        self.buffer
            .line(self.cursor.line)
            .trim_end_matches('\n')
            .to_string()
    }

    pub fn line_count(&self) -> usize {
        self.buffer.len_lines()
    }

    pub fn word_count(&self) -> usize {
        let text = self.buffer.to_string();
        text.split_whitespace().count()
    }

    pub fn char_count(&self) -> usize {
        self.buffer.len_chars()
    }
}

impl Default for Editor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::HighlightToken;

    #[test]
    fn new_editor_is_empty() {
        let editor = Editor::new();
        assert_eq!(editor.content(), "");
        assert!(!editor.is_dirty());
        assert_eq!(editor.line_count(), 1);
    }

    #[test]
    fn with_content() {
        let editor = Editor::with_content("hello\nworld");
        assert_eq!(editor.content(), "hello\nworld");
        assert_eq!(editor.line_count(), 2);
    }

    #[test]
    fn insert_text_basic() {
        let mut editor = Editor::new();
        editor.insert_text("hello");
        assert_eq!(editor.content(), "hello");
        assert_eq!(editor.cursor(), &Cursor::new(0, 5));
    }

    #[test]
    fn insert_multiline() {
        let mut editor = Editor::new();
        editor.insert_text("hello\nworld");
        assert_eq!(editor.cursor(), &Cursor::new(1, 5));
        assert_eq!(editor.line_count(), 2);
    }

    #[test]
    fn delete_backwards_basic() {
        let mut editor = Editor::with_content("hello");
        editor.move_cursor_to(0, 5);
        editor.delete_backwards();
        assert_eq!(editor.content(), "hell");
        assert_eq!(editor.cursor(), &Cursor::new(0, 4));
    }

    #[test]
    fn delete_backwards_at_line_start() {
        let mut editor = Editor::with_content("hello\nworld");
        editor.move_cursor_to(1, 0);
        editor.delete_backwards();
        assert_eq!(editor.content(), "helloworld");
        assert_eq!(editor.cursor(), &Cursor::new(0, 5));
    }

    #[test]
    fn delete_forwards_basic() {
        let mut editor = Editor::with_content("hello");
        editor.move_cursor_to(0, 0);
        editor.delete_forwards();
        assert_eq!(editor.content(), "ello");
    }

    #[test]
    fn undo_insert() {
        let mut editor = Editor::new();
        editor.insert_text("hello");
        assert!(editor.can_undo());
        editor.undo();
        assert_eq!(editor.content(), "");
        assert!(!editor.can_undo());
    }

    #[test]
    fn undo_redo() {
        let mut editor = Editor::new();
        editor.insert_text("hello");
        editor.undo();
        assert_eq!(editor.content(), "");
        assert!(editor.can_redo());
        editor.redo();
        assert_eq!(editor.content(), "hello");
    }

    #[test]
    fn select_all() {
        let mut editor = Editor::with_content("hello\nworld");
        editor.select_all();
        assert!(!editor.selection().is_empty());
        let (start, end) = editor.selection().normalize();
        assert_eq!(start, Cursor::new(0, 0));
        assert_eq!(end, Cursor::new(1, 5));
    }

    #[test]
    fn delete_selection() {
        let mut editor = Editor::with_content("hello world");
        editor.selection = Selection::range(Cursor::new(0, 0), Cursor::new(0, 5));
        editor.delete_selection();
        assert_eq!(editor.content(), " world");
    }

    #[test]
    fn find_and_replace() {
        let mut editor = Editor::with_content("hello world hello");
        let results = editor.find("hello");
        assert_eq!(results.len(), 2);
        editor.replace_all("hello", "hi");
        assert_eq!(editor.content(), "hi world hi");
    }

    #[test]
    fn highlight_basic() {
        let mut editor = Editor::with_content("# Title\nhello **bold**");
        let spans = editor.rehighlight_all();
        assert_eq!(spans.len(), 2);
        assert!(spans[0].iter().any(|s| s.token == HighlightToken::Heading1));
        assert!(spans[1].iter().any(|s| s.token == HighlightToken::Bold));
    }

    #[test]
    fn auto_indent_newline() {
        let mut editor = Editor::with_content("  hello");
        editor.move_cursor_to(0, 7);
        editor.auto_indent_newline();
        assert_eq!(editor.content(), "  hello\n  ");
    }

    #[test]
    fn indent_unindent_line() {
        let mut editor = Editor::with_content("hello");
        editor.indent_line(0);
        assert!(editor.content().starts_with("  "));
        editor.unindent_line(0);
        assert_eq!(editor.content(), "hello");
    }

    #[test]
    fn duplicate_line() {
        let mut editor = Editor::with_content("hello\n");
        editor.move_cursor_to(0, 0);
        editor.duplicate_line();
        assert_eq!(editor.content(), "hello\nhello\n");
    }

    #[test]
    fn word_count() {
        let editor = Editor::with_content("hello world foo");
        assert_eq!(editor.word_count(), 3);
    }

    #[test]
    fn char_count() {
        let editor = Editor::with_content("hello");
        assert_eq!(editor.char_count(), 5);
    }

    #[test]
    fn move_line_up_down() {
        let mut editor = Editor::with_content("first\nsecond\nthird\n");
        editor.move_cursor_to(2, 0);
        editor.move_line_up();
        assert_eq!(editor.content(), "first\nthird\nsecond\n");
        editor.move_line_up();
        assert_eq!(editor.content(), "third\nfirst\nsecond\n");
    }

    #[test]
    fn delete_word_backwards() {
        let mut editor = Editor::with_content("hello world");
        editor.move_cursor_to(0, 11);
        editor.delete_word_backwards();
        assert_eq!(editor.content(), "hello ");
    }

    #[test]
    fn delete_word_forwards() {
        let mut editor = Editor::with_content("hello world");
        editor.move_cursor_to(0, 0);
        editor.delete_word_forwards();
        assert_eq!(editor.content(), "world");
    }

    #[test]
    fn select_word() {
        let mut editor = Editor::with_content("hello world");
        editor.move_cursor_to(0, 2);
        editor.select_word();
        let (start, end) = editor.selection().normalize();
        assert_eq!(start, Cursor::new(0, 0));
        assert_eq!(end, Cursor::new(0, 5));
    }

    #[test]
    fn insert_with_selection_replaces() {
        let mut editor = Editor::with_content("hello world");
        editor.selection = Selection::range(Cursor::new(0, 0), Cursor::new(0, 5));
        editor.insert_text("hi");
        assert_eq!(editor.content(), "hi world");
    }

    #[test]
    fn page_up_down() {
        let mut editor = Editor::with_content("1\n2\n3\n4\n5\n6\n7\n8\n9\n10\n");
        editor.move_cursor_to(8, 0);
        editor.page_up(3);
        assert_eq!(editor.cursor().line, 5);
        editor.page_down(3);
        assert_eq!(editor.cursor().line, 8);
    }

    #[test]
    fn crdt_sync_on_insert() {
        let mut editor = Editor::new();
        editor.insert_text("hello");
        let update = editor.encode_update();
        // After inserting text, there should be a non-empty CRDT update
        assert!(!update.is_empty());
    }

    #[test]
    fn crdt_sync_on_delete() {
        let mut editor = Editor::with_content("hello");
        editor.move_cursor_to(0, 5);
        editor.delete_backwards();
        let update = editor.encode_update();
        assert!(!update.is_empty());
    }

    #[test]
    fn crdt_set_content_syncs() {
        let mut editor = Editor::with_content("initial");
        let update1 = editor.encode_update();
        editor.set_content("replaced");
        let update2 = editor.encode_update();
        // Both should produce updates
        assert!(!update1.is_empty());
        assert!(!update2.is_empty());
    }

    #[test]
    fn crdt_remote_update() {
        let mut editor_a = Editor::new();
        editor_a.insert_text("hello world");

        let update = editor_a.encode_update();
        assert!(!update.is_empty());

        let mut editor_b = Editor::new();
        let changed = editor_b.apply_remote_update(&update);
        assert!(changed);
        assert_eq!(editor_b.content(), "hello world");
    }

    #[test]
    fn crdt_full_state() {
        let mut editor = Editor::new();
        editor.insert_text("hello\nworld");
        // encode_update captures the changes since last state
        let update = editor.encode_update();
        assert!(!update.is_empty());

        let mut editor_b = Editor::new();
        let changed = editor_b.apply_remote_update(&update);
        assert!(changed);
        assert_eq!(editor_b.content(), "hello\nworld");
    }

    #[test]
    fn crdt_state_vector_roundtrip() {
        let mut editor = Editor::new();
        editor.insert_text("test content");
        let sv = editor.state_vector();
        assert!(!sv.is_empty());
    }

    #[test]
    fn crdt_consecutive_updates() {
        let mut editor_a = Editor::new();
        editor_a.insert_text("hello");
        let update1 = editor_a.encode_update();

        editor_a.insert_text(" world");
        let update2 = editor_a.encode_update();

        let mut editor_b = Editor::new();
        editor_b.apply_remote_update(&update1);
        assert_eq!(editor_b.content(), "hello");

        editor_b.apply_remote_update(&update2);
        assert_eq!(editor_b.content(), "hello world");
    }

    #[test]
    fn test_get_wikilink_state_active() {
        let mut editor = Editor::with_content("[[hello");
        editor.move_cursor_to(0, 7);
        let state = editor.get_wikilink_state();
        assert!(state.is_some());
        let s = state.unwrap();
        assert!(s.active);
        assert_eq!(s.query, "hello");
        assert_eq!(s.start_line, 0);
        assert_eq!(s.start_col, 0);
    }

    #[test]
    fn test_get_wikilink_state_closed() {
        let mut editor = Editor::with_content("[[hello]]");
        editor.move_cursor_to(0, 9);
        let state = editor.get_wikilink_state();
        assert!(state.is_none());
    }

    #[test]
    fn test_get_wikilink_state_no_open() {
        let mut editor = Editor::with_content("some random text");
        editor.move_cursor_to(0, 16);
        let state = editor.get_wikilink_state();
        assert!(state.is_none());
    }

    #[test]
    fn test_insert_wikilink() {
        let mut editor = Editor::with_content("[[hel");
        editor.move_cursor_to(0, 5);
        editor.insert_wikilink("Hello World", None);
        assert_eq!(editor.content(), "[[Hello World]]");
    }

    #[test]
    fn test_insert_wikilink_with_alias() {
        let mut editor = Editor::with_content("some [[hel");
        editor.move_cursor_to(0, 10);
        editor.insert_wikilink("Hello World", Some("hw"));
        assert_eq!(editor.content(), "some [[Hello World|hw]]");
    }

    #[test]
    fn test_insert_wikilink_no_state() {
        let mut editor = Editor::with_content("no wikilink here");
        editor.move_cursor_to(0, 17);
        editor.insert_wikilink("Hello", None);
        assert_eq!(editor.content(), "no wikilink here");
    }
}
