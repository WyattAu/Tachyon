use ropey::Rope;

#[derive(Debug, Clone)]
pub struct TextBuffer {
    rope: Rope,
}

impl TextBuffer {
    pub fn new() -> Self {
        Self {
            rope: Rope::new(),
        }
    }

    pub fn from_str(text: &str) -> Self {
        Self {
            rope: Rope::from_str(text),
        }
    }

    pub fn to_string(&self) -> String {
        self.rope.to_string()
    }

    pub fn len_lines(&self) -> usize {
        self.rope.len_lines()
    }

    pub fn len_chars(&self) -> usize {
        self.rope.len_chars()
    }

    pub fn line(&self, line_idx: usize) -> String {
        if line_idx >= self.rope.len_lines() {
            return String::new();
        }
        self.rope.line(line_idx).to_string()
    }

    pub fn line_len(&self, line_idx: usize) -> usize {
        if line_idx >= self.rope.len_lines() {
            return 0;
        }
        let line = self.rope.line(line_idx);
        let mut len = line.len_chars();
        if len > 0 && line.char(len - 1) == '\n' {
            len -= 1;
        }
        len
    }

    pub fn char_at(&self, line: usize, col: usize) -> Option<char> {
        if line >= self.rope.len_lines() {
            return None;
        }
        let line_rope = self.rope.line(line);
        if col >= line_rope.len_chars() {
            return None;
        }
        Some(line_rope.char(col))
    }

    pub fn insert(&mut self, line: usize, col: usize, text: &str) {
        let char_idx = self.line_col_to_char_idx(line, col);
        self.rope.insert(char_idx, text);
    }

    pub fn delete_range(
        &mut self,
        start_line: usize,
        start_col: usize,
        end_line: usize,
        end_col: usize,
    ) -> String {
        let start_char_idx = self.line_col_to_char_idx(start_line, start_col);
        let end_char_idx = self.line_col_to_char_idx(end_line, end_col);
        if start_char_idx >= end_char_idx {
            return String::new();
        }
        let deleted = self.rope.slice(start_char_idx..end_char_idx).to_string();
        self.rope.remove(start_char_idx..end_char_idx);
        deleted
    }

    pub fn line_content_with_newline(&self, line_idx: usize) -> String {
        if line_idx >= self.rope.len_lines() {
            return String::new();
        }
        self.rope.line(line_idx).to_string()
    }

    pub fn is_line_empty(&self, line_idx: usize) -> bool {
        self.line_len(line_idx) == 0
    }

    pub fn indent_level(&self, line_idx: usize) -> usize {
        let line = self.line(line_idx);
        line.chars().take_while(|&c| c == ' ').count()
    }

    fn line_col_to_char_idx(&self, line: usize, col: usize) -> usize {
        if line >= self.rope.len_lines() {
            return self.rope.len_chars();
        }
        let line_start = self.rope.line_to_char(line);
        let line_len = self.line_len(line);
        let col = col.min(line_len);
        line_start + col
    }
}

impl Default for TextBuffer {
    fn default() -> Self {
        Self::new()
    }
}

impl PartialEq for TextBuffer {
    fn eq(&self, other: &Self) -> bool {
        self.rope.to_string() == other.rope.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_buffer_is_empty() {
        let buf = TextBuffer::new();
        assert_eq!(buf.len_lines(), 1);
        assert_eq!(buf.len_chars(), 0);
        assert_eq!(buf.to_string(), "");
    }

    #[test]
    fn from_str_basic() {
        let buf = TextBuffer::from_str("hello\nworld");
        assert_eq!(buf.len_lines(), 2);
        assert_eq!(buf.line(0), "hello\n");
        assert_eq!(buf.line_len(0), 5);
        assert_eq!(buf.line_len(1), 5);
        assert_eq!(buf.char_at(0, 0), Some('h'));
        assert_eq!(buf.char_at(0, 4), Some('o'));
        assert_eq!(buf.char_at(1, 0), Some('w'));
        assert_eq!(buf.char_at(2, 0), None);
    }

    #[test]
    fn insert_text() {
        let mut buf = TextBuffer::from_str("hello\nworld");
        buf.insert(0, 5, " there");
        assert_eq!(buf.line(0), "hello there\n");
    }

    #[test]
    fn insert_multiline() {
        let mut buf = TextBuffer::from_str("hello");
        buf.insert(0, 5, "\nworld");
        assert_eq!(buf.len_lines(), 2);
        assert_eq!(buf.line(0), "hello\n");
        assert_eq!(buf.line(1), "world");
    }

    #[test]
    fn delete_range_same_line() {
        let mut buf = TextBuffer::from_str("hello world");
        let deleted = buf.delete_range(0, 5, 0, 11);
        assert_eq!(deleted, " world");
        assert_eq!(buf.to_string(), "hello");
    }

    #[test]
    fn delete_range_multiline() {
        let mut buf = TextBuffer::from_str("hello\nworld\nfoo");
        let deleted = buf.delete_range(0, 5, 2, 0);
        assert_eq!(deleted, "\nworld\n");
        assert_eq!(buf.to_string(), "hellofoo");
    }

    #[test]
    fn delete_range_noop_when_inverted() {
        let mut buf = TextBuffer::from_str("hello");
        let deleted = buf.delete_range(0, 3, 0, 1);
        assert_eq!(deleted, "");
        assert_eq!(buf.to_string(), "hello");
    }

    #[test]
    fn indent_level() {
        let buf = TextBuffer::from_str("    hello\n  world\nnoindent");
        assert_eq!(buf.indent_level(0), 4);
        assert_eq!(buf.indent_level(1), 2);
        assert_eq!(buf.indent_level(2), 0);
    }

    #[test]
    fn is_line_empty() {
        let buf = TextBuffer::from_str("hello\n\nworld");
        assert!(!buf.is_line_empty(0));
        assert!(buf.is_line_empty(1));
        assert!(!buf.is_line_empty(2));
    }

    #[test]
    fn line_content_with_newline() {
        let buf = TextBuffer::from_str("hello\nworld");
        assert_eq!(buf.line_content_with_newline(0), "hello\n");
        assert_eq!(buf.line_content_with_newline(1), "world");
    }

    #[test]
    fn insert_at_end_of_empty_buffer() {
        let mut buf = TextBuffer::new();
        buf.insert(0, 0, "hello");
        assert_eq!(buf.to_string(), "hello");
    }

    #[test]
    fn insert_beyond_line_end_clamps() {
        let mut buf = TextBuffer::from_str("hi");
        buf.insert(0, 100, "!");
        assert_eq!(buf.to_string(), "hi!");
    }
}
