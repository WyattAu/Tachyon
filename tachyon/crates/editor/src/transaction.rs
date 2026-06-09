use serde::{Deserialize, Serialize};

use crate::cursor::Cursor;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EditKind {
    Insert { text: String },
    Delete { text: String },
    Replace { old_text: String, new_text: String },
}

impl EditKind {
    pub fn text(&self) -> String {
        match self {
            EditKind::Insert { text } => text.clone(),
            EditKind::Delete { text } => text.clone(),
            EditKind::Replace { new_text, .. } => new_text.clone(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub kind: EditKind,
    pub start: Cursor,
    pub end: Cursor,
    pub timestamp: u64,
}

pub struct UndoStack {
    undo: Vec<Transaction>,
    redo: Vec<Transaction>,
    max_size: usize,
}

impl UndoStack {
    pub fn new(max_size: usize) -> Self {
        Self {
            undo: Vec::with_capacity(max_size),
            redo: Vec::new(),
            max_size,
        }
    }

    pub fn push(&mut self, transaction: Transaction) {
        if self.undo.len() >= self.max_size {
            self.undo.remove(0);
        }
        self.undo.push(transaction);
        self.redo.clear();
    }

    pub fn undo(&mut self) -> Option<&Transaction> {
        if self.undo.is_empty() {
            return None;
        }
        // We don't move to redo here because the caller applies the inverse.
        // The caller should call push with the inverse transaction after applying undo.
        self.undo.last()
    }

    pub fn redo(&mut self) -> Option<&Transaction> {
        if self.redo.is_empty() {
            return None;
        }
        self.redo.last()
    }

    pub fn can_undo(&self) -> bool {
        !self.undo.is_empty()
    }

    pub fn last_mut(&mut self) -> Option<&mut Transaction> {
        self.undo.last_mut()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }

    pub fn clear(&mut self) {
        self.undo.clear();
        self.redo.clear();
    }

    pub fn push_undo(&mut self, transaction: Transaction) {
        if self.undo.len() >= self.max_size {
            self.undo.remove(0);
        }
        self.undo.push(transaction);
    }

    pub fn push_redo(&mut self, transaction: Transaction) {
        self.redo.push(transaction);
    }

    pub fn pop_undo(&mut self) -> Option<Transaction> {
        self.undo.pop()
    }

    pub fn pop_redo(&mut self) -> Option<Transaction> {
        self.redo.pop()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_transaction(text: &str) -> Transaction {
        Transaction {
            kind: EditKind::Insert {
                text: text.to_string(),
            },
            start: Cursor::zero(),
            end: Cursor::zero(),
            timestamp: 0,
        }
    }

    #[test]
    fn push_and_can_undo() {
        let mut stack = UndoStack::new(100);
        stack.push(make_transaction("hello"));
        assert!(stack.can_undo());
        assert!(!stack.can_redo());
    }

    #[test]
    fn push_clears_redo() {
        let mut stack = UndoStack::new(100);
        stack.push(make_transaction("first"));
        stack.push_redo(make_transaction("redo1"));
        stack.push(make_transaction("second"));
        assert!(!stack.can_redo());
    }

    #[test]
    fn max_size_evicts_oldest() {
        let mut stack = UndoStack::new(3);
        stack.push(make_transaction("a"));
        stack.push(make_transaction("b"));
        stack.push(make_transaction("c"));
        stack.push(make_transaction("d"));
        assert_eq!(stack.undo.len(), 3);
    }

    #[test]
    fn pop_undo_and_redo() {
        let mut stack = UndoStack::new(100);
        stack.push(make_transaction("a"));
        stack.push(make_transaction("b"));
        let t = stack.pop_undo().unwrap();
        assert_eq!(
            match &t.kind {
                EditKind::Insert { text } => text.as_str(),
                _ => "",
            },
            "b"
        );
        stack.push_redo(t);
        let r = stack.pop_redo().unwrap();
        assert_eq!(
            match &r.kind {
                EditKind::Insert { text } => text.as_str(),
                _ => "",
            },
            "b"
        );
    }

    #[test]
    fn clear_resets_everything() {
        let mut stack = UndoStack::new(100);
        stack.push(make_transaction("a"));
        stack.push(make_transaction("b"));
        stack.clear();
        assert!(!stack.can_undo());
        assert!(!stack.can_redo());
    }

    #[test]
    fn undo_on_empty_returns_none() {
        let mut stack = UndoStack::new(100);
        assert!(stack.undo().is_none());
    }
}
