use crate::buffer::TextBuffer;
use crate::cursor::Cursor;

pub fn move_left(buffer: &TextBuffer, cursor: &mut Cursor) {
    cursor.move_left(buffer);
}

pub fn move_down(buffer: &TextBuffer, cursor: &mut Cursor) {
    cursor.move_down(buffer);
}

pub fn move_up(buffer: &TextBuffer, cursor: &mut Cursor) {
    cursor.move_up(buffer);
}

pub fn move_right(buffer: &TextBuffer, cursor: &mut Cursor) {
    cursor.move_right(buffer);
}

pub fn move_word_forward(buffer: &TextBuffer, cursor: &mut Cursor) {
    cursor.move_word_right(buffer);
}

pub fn move_word_backward(buffer: &TextBuffer, cursor: &mut Cursor) {
    cursor.move_word_left(buffer);
}

pub fn move_word_end(buffer: &TextBuffer, cursor: &mut Cursor) {
    let text = buffer.line(cursor.line);
    let chars: Vec<char> = text.chars().collect();
    let mut pos = cursor.col.min(chars.len());

    // Skip current word
    while pos < chars.len() && !chars[pos].is_whitespace() {
        pos += 1;
    }
    // Skip whitespace
    while pos < chars.len() && chars[pos].is_whitespace() {
        pos += 1;
    }
    // Move to end of word
    if pos < chars.len() {
        while pos + 1 < chars.len() && !chars[pos + 1].is_whitespace() {
            pos += 1;
        }
    }

    cursor.col = pos;
}

pub fn move_line_start(buffer: &TextBuffer, cursor: &mut Cursor) {
    let _ = buffer;
    cursor.col = 0;
}

pub fn move_line_end(buffer: &TextBuffer, cursor: &mut Cursor) {
    cursor.move_to_line_end(buffer);
}

pub fn move_doc_start(buffer: &TextBuffer, cursor: &mut Cursor) {
    cursor.move_to_buffer_start();
}

pub fn move_doc_end(buffer: &TextBuffer, cursor: &mut Cursor) {
    cursor.move_to_buffer_end(buffer);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::buffer::TextBuffer;

    #[test]
    fn left_motion() {
        let buf = TextBuffer::from_str("hello");
        let mut c = Cursor::new(0, 3);
        move_left(&buf, &mut c);
        assert_eq!(c, Cursor::new(0, 2));
    }

    #[test]
    fn left_motion_at_start() {
        let buf = TextBuffer::from_str("hello\nworld");
        let mut c = Cursor::new(1, 0);
        move_left(&buf, &mut c);
        assert_eq!(c, Cursor::new(0, 5));
    }

    #[test]
    fn right_motion() {
        let buf = TextBuffer::from_str("hello");
        let mut c = Cursor::new(0, 2);
        move_right(&buf, &mut c);
        assert_eq!(c, Cursor::new(0, 3));
    }

    #[test]
    fn down_motion() {
        let buf = TextBuffer::from_str("hello\nworld\nfoo");
        let mut c = Cursor::new(0, 2);
        move_down(&buf, &mut c);
        assert_eq!(c, Cursor::new(1, 2));
    }

    #[test]
    fn down_motion_clamps() {
        let buf = TextBuffer::from_str("hello\nhi");
        let mut c = Cursor::new(0, 5);
        move_down(&buf, &mut c);
        assert_eq!(c, Cursor::new(1, 2));
    }

    #[test]
    fn up_motion() {
        let buf = TextBuffer::from_str("hello\nworld");
        let mut c = Cursor::new(1, 2);
        move_up(&buf, &mut c);
        assert_eq!(c, Cursor::new(0, 2));
    }

    #[test]
    fn word_forward() {
        let buf = TextBuffer::from_str("hello world foo");
        let mut c = Cursor::new(0, 0);
        move_word_forward(&buf, &mut c);
        assert_eq!(c.col, 6);
        move_word_forward(&buf, &mut c);
        assert_eq!(c.col, 12);
    }

    #[test]
    fn word_backward() {
        let buf = TextBuffer::from_str("hello world foo");
        let mut c = Cursor::new(0, 11);
        move_word_backward(&buf, &mut c);
        assert_eq!(c.col, 6);
        move_word_backward(&buf, &mut c);
        assert_eq!(c.col, 0);
    }

    #[test]
    fn line_start() {
        let buf = TextBuffer::from_str("hello");
        let mut c = Cursor::new(0, 3);
        move_line_start(&buf, &mut c);
        assert_eq!(c.col, 0);
    }

    #[test]
    fn line_end() {
        let buf = TextBuffer::from_str("hello");
        let mut c = Cursor::new(0, 0);
        move_line_end(&buf, &mut c);
        assert_eq!(c.col, 5);
    }

    #[test]
    fn doc_start() {
        let buf = TextBuffer::from_str("hello\nworld");
        let mut c = Cursor::new(1, 3);
        move_doc_start(&buf, &mut c);
        assert_eq!(c, Cursor::new(0, 0));
    }

    #[test]
    fn doc_end() {
        let buf = TextBuffer::from_str("hello\nworld");
        let mut c = Cursor::new(0, 0);
        move_doc_end(&buf, &mut c);
        assert_eq!(c, Cursor::new(1, 5));
    }
}
