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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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
            cursor.line > start.line && cursor.line < end.line
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

#[derive(Debug, Clone)]
pub struct Cursors {
    pub(crate) entries: Vec<(Cursor, Selection)>,
    pub(crate) active: usize,
}

impl Cursors {
    pub fn new() -> Self {
        let cursor = Cursor::zero();
        let selection = Selection::caret(cursor);
        Self {
            entries: vec![(cursor, selection)],
            active: 0,
        }
    }

    pub fn add(&mut self, cursor: Cursor, selection: Selection) -> usize {
        let entry = (cursor, selection);
        let insert_pos = match self.entries.binary_search_by_key(&cursor, |&(c, _)| c) {
            Ok(i) => i,
            Err(i) => i,
        };

        self.entries.insert(insert_pos, entry);
        self.merge_overlapping();

        let new_active = self
            .entries
            .binary_search_by_key(&cursor, |&(c, _)| c)
            .unwrap_or_else(|i| i.saturating_sub(1));
        self.active = new_active;
        new_active
    }

    pub fn remove(&mut self, index: usize) {
        if index >= self.entries.len() {
            return;
        }
        if self.entries.len() <= 1 {
            let cursor = Cursor::zero();
            let selection = Selection::caret(cursor);
            self.entries = vec![(cursor, selection)];
            self.active = 0;
            return;
        }
        self.entries.remove(index);
        if self.active >= self.entries.len() {
            self.active = self.entries.len() - 1;
        } else if self.active > index {
            self.active -= 1;
        }
    }

    pub fn active(&self) -> (Cursor, Selection) {
        self.entries[self.active].clone()
    }

    pub fn active_index(&self) -> usize {
        self.active
    }

    pub fn set_active(&mut self, index: usize) {
        if !self.entries.is_empty() && index < self.entries.len() {
            self.active = index;
        }
    }

    pub fn set_active_cursor(&mut self, cursor: Cursor, selection: Selection) {
        if let Some(idx) = self.entries.iter().position(|(c, _)| *c == cursor) {
            self.active = idx;
            self.entries[idx] = (cursor, selection);
        } else {
            let insert_pos = match self.entries.binary_search_by_key(&cursor, |&(c, _)| c) {
                Ok(i) => i,
                Err(i) => i,
            };
            self.entries.insert(insert_pos, (cursor, selection));
            self.merge_overlapping();
            if let Ok(idx) = self.entries.binary_search_by_key(&cursor, |&(c, _)| c) {
                self.active = idx;
            }
        }
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = &(Cursor, Selection)> {
        self.entries.iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut (Cursor, Selection)> {
        self.entries.iter_mut()
    }

    pub fn clear(&mut self, cursor: Cursor, selection: Selection) {
        self.entries = vec![(cursor, selection)];
        self.active = 0;
    }

    fn merge_overlapping(&mut self) {
        if self.entries.len() <= 1 {
            return;
        }

        let mut merged: Vec<(Cursor, Selection)> = Vec::new();
        let mut current = self.entries[0].clone();

        for i in 1..self.entries.len() {
            let (cursor, selection) = self.entries[i].clone();
            if current.1.overlaps_or_adjacent(&selection) {
                if !selection.is_empty() && current.1.is_empty() {
                    current.0 = cursor;
                    current.1 = selection;
                } else if !selection.is_empty() {
                    current.1.merge(selection);
                    if cursor > current.0 {
                        current.0 = cursor;
                    }
                }
            } else {
                merged.push(current);
                current = (cursor, selection);
            }
        }
        merged.push(current);

        self.entries = merged;
        if self.active >= self.entries.len() {
            self.active = self.entries.len().saturating_sub(1);
        }
    }
}

impl Default for Cursors {
    fn default() -> Self {
        Self::new()
    }
}

impl Selection {
    fn overlaps_or_adjacent(&self, other: &Selection) -> bool {
        if self.is_empty() && other.is_empty() {
            return false;
        }
        let (s1, e1) = self.normalize();
        let (s2, e2) = other.normalize();
        if s1.line == e1.line && s2.line == e2.line && s1.line == s2.line {
            s1.col <= e2.col && s2.col <= e1.col
        } else {
            let start1 = (s1.line, s1.col);
            let end1 = (e1.line, e1.col);
            let start2 = (s2.line, s2.col);
            let end2 = (e2.line, e2.col);
            start1 <= end2 && start2 <= end1
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cursors_new_single_at_zero() {
        let c = Cursors::new();
        assert_eq!(c.len(), 1);
        assert_eq!(
            c.active(),
            (Cursor::zero(), Selection::caret(Cursor::zero()))
        );
        assert_eq!(c.active_index(), 0);
        assert!(!c.is_empty());
    }

    #[test]
    fn cursors_add_sorted() {
        let mut c = Cursors::new();
        c.add(Cursor::new(2, 0), Selection::caret(Cursor::new(2, 0)));
        c.add(Cursor::new(0, 3), Selection::caret(Cursor::new(0, 3)));
        let cursors: Vec<_> = c.iter().map(|(cur, _)| *cur).collect();
        assert_eq!(cursors[0], Cursor::new(0, 0));
        assert_eq!(cursors[1], Cursor::new(0, 3));
        assert_eq!(cursors[2], Cursor::new(2, 0));
    }

    #[test]
    fn cursors_remove_adjusts_active() {
        let mut c = Cursors::new();
        c.add(Cursor::new(2, 0), Selection::caret(Cursor::new(2, 0)));
        c.add(Cursor::new(4, 0), Selection::caret(Cursor::new(4, 0)));
        assert_eq!(c.len(), 3);
        c.remove(1);
        assert_eq!(c.len(), 2);
        assert_eq!(c.active_index(), 1);
    }

    #[test]
    fn cursors_remove_last_keeps_one() {
        let mut c = Cursors::new();
        c.remove(0);
        assert_eq!(c.len(), 1);
        assert_eq!(
            c.active(),
            (Cursor::zero(), Selection::caret(Cursor::zero()))
        );
    }

    #[test]
    fn cursors_merge_overlapping_selections() {
        let mut c = Cursors::new();
        c.add(
            Cursor::new(0, 0),
            Selection::range(Cursor::new(0, 0), Cursor::new(0, 5)),
        );
        c.add(
            Cursor::new(0, 3),
            Selection::range(Cursor::new(0, 3), Cursor::new(0, 8)),
        );
        assert_eq!(c.len(), 1);
        let (_, sel) = c.active();
        let (start, end) = sel.normalize();
        assert_eq!(start, Cursor::new(0, 0));
        assert_eq!(end, Cursor::new(0, 8));
    }

    #[test]
    fn cursors_clear() {
        let mut c = Cursors::new();
        c.add(Cursor::new(5, 0), Selection::caret(Cursor::new(5, 0)));
        c.clear(Cursor::new(1, 1), Selection::caret(Cursor::new(1, 1)));
        assert_eq!(c.len(), 1);
        assert_eq!(c.active_index(), 0);
        assert_eq!(c.active().0, Cursor::new(1, 1));
    }

    #[test]
    fn cursors_set_active() {
        let mut c = Cursors::new();
        c.add(Cursor::new(2, 0), Selection::caret(Cursor::new(2, 0)));
        c.set_active(0);
        assert_eq!(c.active_index(), 0);
    }

    #[test]
    fn cursors_set_active_out_of_bounds_is_noop() {
        let mut c = Cursors::new();
        c.set_active(99);
        assert_eq!(c.active_index(), 0);
    }

    #[test]
    fn cursors_iter_mut() {
        let mut c = Cursors::new();
        for (_, sel) in c.iter_mut() {
            sel.kind = SelectionKind::Word;
        }
        assert_eq!(c.active().1.kind, SelectionKind::Word);
    }

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
