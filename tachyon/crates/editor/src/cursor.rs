use serde::{Deserialize, Serialize};

use crate::buffer::TextBuffer;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Cursor {
    pub line: usize,
    pub col: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SelectionKind {
    Caret,
    Range,
    Word,
    Line,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Selection {
    pub cursor: Cursor,
    pub anchor: Cursor,
    pub kind: SelectionKind,
}

impl Cursor {
    pub fn new(line: usize, col: usize) -> Self {
        Self { line, col }
    }

    pub fn zero() -> Self {
        Self { line: 0, col: 0 }
    }

    pub fn end_of_line(buffer: &TextBuffer) -> Self {
        let last_line = buffer.len_lines().saturating_sub(1);
        Self {
            line: last_line,
            col: buffer.line_len(last_line),
        }
    }

    pub fn start_of_line(line: usize) -> Self {
        Self { line, col: 0 }
    }

    pub fn move_left(&mut self, buffer: &TextBuffer) {
        if self.col > 0 {
            self.col -= 1;
        } else if self.line > 0 {
            self.line -= 1;
            self.col = buffer.line_len(self.line);
        }
    }

    pub fn move_right(&mut self, buffer: &TextBuffer) {
        let line_len = buffer.line_len(self.line);
        if self.col < line_len {
            self.col += 1;
        } else if self.line < buffer.len_lines() - 1 {
            self.line += 1;
            self.col = 0;
        }
    }

    pub fn move_up(&mut self, buffer: &TextBuffer) {
        if self.line > 0 {
            self.line -= 1;
            self.col = self.col.min(buffer.line_len(self.line));
        }
    }

    pub fn move_down(&mut self, buffer: &TextBuffer) {
        if self.line < buffer.len_lines() - 1 {
            self.line += 1;
            self.col = self.col.min(buffer.line_len(self.line));
        }
    }

    pub fn move_home(&mut self) {
        self.col = 0;
    }

    pub fn move_end(&mut self, buffer: &TextBuffer) {
        self.col = buffer.line_len(self.line);
    }

    pub fn move_to_line_start(&mut self) {
        self.col = 0;
    }

    pub fn move_to_line_end(&mut self, buffer: &TextBuffer) {
        self.col = buffer.line_len(self.line);
    }

    pub fn move_to_buffer_start(&mut self) {
        self.line = 0;
        self.col = 0;
    }

    pub fn move_to_buffer_end(&mut self, buffer: &TextBuffer) {
        self.line = buffer.len_lines().saturating_sub(1);
        self.col = buffer.line_len(self.line);
    }

    pub fn move_word_left(&mut self, buffer: &TextBuffer) {
        let text = buffer.line(self.line);
        if self.col == 0 {
            if self.line > 0 {
                self.line -= 1;
                self.col = buffer.line_len(self.line);
                self.move_word_left(buffer);
            }
            return;
        }

        let chars: Vec<char> = text.chars().collect();
        let mut pos = self.col.min(chars.len());

        while pos > 0 && chars[pos - 1].is_whitespace() {
            pos -= 1;
        }
        while pos > 0 && !chars[pos - 1].is_whitespace() {
            pos -= 1;
        }
        self.col = pos;
    }

    pub fn move_word_right(&mut self, buffer: &TextBuffer) {
        let text = buffer.line(self.line);
        let line_len = buffer.line_len(self.line);
        if self.col >= line_len {
            if self.line < buffer.len_lines() - 1 {
                self.line += 1;
                self.col = 0;
                self.move_word_right(buffer);
            }
            return;
        }

        let chars: Vec<char> = text.chars().collect();
        let mut pos = self.col.min(chars.len());

        while pos < chars.len() && !chars[pos].is_whitespace() {
            pos += 1;
        }
        while pos < chars.len() && chars[pos].is_whitespace() {
            pos += 1;
        }
        self.col = pos;
    }
}

impl Selection {
    pub fn caret(cursor: Cursor) -> Self {
        Self {
            cursor,
            anchor: cursor,
            kind: SelectionKind::Caret,
        }
    }

    pub fn range(anchor: Cursor, cursor: Cursor) -> Self {
        Self {
            cursor,
            anchor,
            kind: SelectionKind::Range,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.cursor == self.anchor
    }

    pub fn normalize(&self) -> (Cursor, Cursor) {
        if self.anchor <= self.cursor {
            (self.anchor, self.cursor)
        } else {
            (self.cursor, self.anchor)
        }
    }

    pub fn selected_text(&self, buffer: &mut TextBuffer) -> String {
        if self.is_empty() {
            return String::new();
        }
        let (start, end) = self.normalize();
        buffer.delete_range(start.line, start.col, end.line, end.col)
    }

    pub fn contains(&self, cursor: Cursor) -> bool {
        if self.is_empty() {
            return false;
        }
        let (start, end) = self.normalize();
        if start.line == end.line {
            cursor.line == start.line && cursor.col >= start.col && cursor.col <= end.col
        } else {
            cursor.line > start.line
                && cursor.line < end.line
                || (cursor.line == start.line && cursor.col >= start.col)
                || (cursor.line == end.line && cursor.col <= end.col)
        }
    }

    pub fn merge(&mut self, other: Selection) {
        let (s1, e1) = self.normalize();
        let (s2, e2) = other.normalize();
        let new_start = if s1 <= s2 { s1 } else { s2 };
        let new_end = if e1 >= e2 { e1 } else { e2 };
        self.anchor = new_start;
        self.cursor = new_end;
        self.kind = SelectionKind::Range;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cursor_zero() {
        let c = Cursor::zero();
        assert_eq!(c.line, 0);
        assert_eq!(c.col, 0);
    }

    #[test]
    fn cursor_move_left_wraps_line() {
        let buf = TextBuffer::from_str("hello\nworld");
        let mut c = Cursor::new(1, 0);
        c.move_left(&buf);
        assert_eq!(c, Cursor::new(0, 5));
    }

    #[test]
    fn cursor_move_right_wraps_line() {
        let buf = TextBuffer::from_str("hello\nworld");
        let mut c = Cursor::new(0, 5);
        c.move_right(&buf);
        assert_eq!(c, Cursor::new(1, 0));
    }

    #[test]
    fn cursor_move_up_clamps_col() {
        let buf = TextBuffer::from_str("hi\nworld");
        let mut c = Cursor::new(1, 5);
        c.move_up(&buf);
        assert_eq!(c, Cursor::new(0, 2));
    }

    #[test]
    fn cursor_move_down_clamps_col() {
        let buf = TextBuffer::from_str("hello\nhi");
        let mut c = Cursor::new(0, 5);
        c.move_down(&buf);
        assert_eq!(c, Cursor::new(1, 2));
    }

    #[test]
    fn cursor_move_up_at_top_noop() {
        let buf = TextBuffer::from_str("hello");
        let mut c = Cursor::new(0, 3);
        c.move_up(&buf);
        assert_eq!(c, Cursor::new(0, 3));
    }

    #[test]
    fn cursor_move_down_at_bottom_noop() {
        let buf = TextBuffer::from_str("hello");
        let mut c = Cursor::new(0, 3);
        c.move_down(&buf);
        assert_eq!(c, Cursor::new(0, 3));
    }

    #[test]
    fn cursor_move_word_left_basic() {
        let buf = TextBuffer::from_str("hello world foo");
        let mut c = Cursor::new(0, 11);
        c.move_word_left(&buf);
        assert_eq!(c.col, 6);
        c.move_word_left(&buf);
        assert_eq!(c.col, 0);
    }

    #[test]
    fn cursor_move_word_right_basic() {
        let buf = TextBuffer::from_str("hello world foo");
        let mut c = Cursor::new(0, 0);
        c.move_word_right(&buf);
        assert_eq!(c.col, 6);
        c.move_word_right(&buf);
        assert_eq!(c.col, 12);
    }

    #[test]
    fn selection_caret_is_empty() {
        let s = Selection::caret(Cursor::new(1, 2));
        assert!(s.is_empty());
    }

    #[test]
    fn selection_range_not_empty() {
        let s = Selection::range(Cursor::new(0, 0), Cursor::new(0, 5));
        assert!(!s.is_empty());
    }

    #[test]
    fn selection_normalize() {
        let s = Selection::range(Cursor::new(0, 5), Cursor::new(0, 2));
        let (start, end) = s.normalize();
        assert_eq!(start, Cursor::new(0, 2));
        assert_eq!(end, Cursor::new(0, 5));
    }

    #[test]
    fn selection_selected_text() {
        let mut buf = TextBuffer::from_str("hello world");
        let s = Selection::range(Cursor::new(0, 0), Cursor::new(0, 5));
        assert_eq!(s.selected_text(&mut buf), "hello");
    }

    #[test]
    fn selection_contains() {
        let _buf = TextBuffer::from_str("hello world");
        let s = Selection::range(Cursor::new(0, 0), Cursor::new(0, 5));
        assert!(s.contains(Cursor::new(0, 3)));
        assert!(!s.contains(Cursor::new(0, 7)));
    }

    #[test]
    fn selection_merge() {
        let mut s1 = Selection::range(Cursor::new(0, 0), Cursor::new(0, 3));
        let s2 = Selection::range(Cursor::new(0, 2), Cursor::new(0, 6));
        s1.merge(s2);
        let (start, end) = s1.normalize();
        assert_eq!(start, Cursor::new(0, 0));
        assert_eq!(end, Cursor::new(0, 6));
    }

    #[test]
    fn cursor_end_of_line() {
        let buf = TextBuffer::from_str("hello\nworld");
        let c = Cursor::end_of_line(&buf);
        assert_eq!(c, Cursor::new(1, 5));
    }
}
