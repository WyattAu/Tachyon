use crate::editor::Editor;
use crate::keybindings::{KeybindingAction, KeybindingMode};
use crate::cursor::{Cursor, Selection};

pub fn execute_action(
    editor: &mut Editor,
    action: KeybindingAction,
    mode: KeybindingMode,
    visual_start: Option<Cursor>,
) -> bool {
    match action {
        // Motions
        KeybindingAction::MoveLeft => {
            let buf = editor.buffer().clone();
            let mut cursor = editor.active_cursor();
            super::motions::move_left(&buf, &mut cursor);
            editor.update_active_cursor(cursor);
            editor.update_active_selection(Selection::caret(cursor));
            true
        }
        KeybindingAction::MoveDown => {
            let buf = editor.buffer().clone();
            let mut cursor = editor.active_cursor();
            super::motions::move_down(&buf, &mut cursor);
            editor.update_active_cursor(cursor);
            editor.update_active_selection(Selection::caret(cursor));
            true
        }
        KeybindingAction::MoveUp => {
            let buf = editor.buffer().clone();
            let mut cursor = editor.active_cursor();
            super::motions::move_up(&buf, &mut cursor);
            editor.update_active_cursor(cursor);
            editor.update_active_selection(Selection::caret(cursor));
            true
        }
        KeybindingAction::MoveRight => {
            let buf = editor.buffer().clone();
            let mut cursor = editor.active_cursor();
            super::motions::move_right(&buf, &mut cursor);
            editor.update_active_cursor(cursor);
            editor.update_active_selection(Selection::caret(cursor));
            true
        }
        KeybindingAction::MoveWordForward => {
            let buf = editor.buffer().clone();
            let mut cursor = editor.active_cursor();
            super::motions::move_word_forward(&buf, &mut cursor);
            editor.update_active_cursor(cursor);
            editor.update_active_selection(Selection::caret(cursor));
            true
        }
        KeybindingAction::MoveWordBackward => {
            let buf = editor.buffer().clone();
            let mut cursor = editor.active_cursor();
            super::motions::move_word_backward(&buf, &mut cursor);
            editor.update_active_cursor(cursor);
            editor.update_active_selection(Selection::caret(cursor));
            true
        }
        KeybindingAction::MoveWordEnd => {
            let buf = editor.buffer().clone();
            let mut cursor = editor.active_cursor();
            super::motions::move_word_end(&buf, &mut cursor);
            editor.update_active_cursor(cursor);
            editor.update_active_selection(Selection::caret(cursor));
            true
        }
        KeybindingAction::MoveLineStart => {
            let buf = editor.buffer().clone();
            let mut cursor = editor.active_cursor();
            super::motions::move_line_start(&buf, &mut cursor);
            editor.update_active_cursor(cursor);
            editor.update_active_selection(Selection::caret(cursor));
            true
        }
        KeybindingAction::MoveLineEnd => {
            let buf = editor.buffer().clone();
            let mut cursor = editor.active_cursor();
            super::motions::move_line_end(&buf, &mut cursor);
            editor.update_active_cursor(cursor);
            editor.update_active_selection(Selection::caret(cursor));
            true
        }
        KeybindingAction::MoveDocStart => {
            let buf = editor.buffer().clone();
            let mut cursor = editor.active_cursor();
            super::motions::move_doc_start(&buf, &mut cursor);
            editor.update_active_cursor(cursor);
            editor.update_active_selection(Selection::caret(cursor));
            true
        }
        KeybindingAction::MoveDocEnd => {
            let buf = editor.buffer().clone();
            let mut cursor = editor.active_cursor();
            super::motions::move_doc_end(&buf, &mut cursor);
            editor.update_active_cursor(cursor);
            editor.update_active_selection(Selection::caret(cursor));
            true
        }

        // Editing
        KeybindingAction::DeleteLine => {
            editor.delete_line();
            true
        }
        KeybindingAction::YankLine => {
            let cursor = editor.active_cursor();
            let _line_text = editor.buffer().line_content_with_newline(cursor.line);
            editor.update_active_selection(Selection::caret(editor.active_cursor()));
            true
        }
        KeybindingAction::PasteAfter => {
            editor.move_cursor_right();
            true
        }
        KeybindingAction::PasteBefore => {
            editor.move_cursor_left();
            true
        }
        KeybindingAction::Undo => {
            editor.undo();
            true
        }
        KeybindingAction::Redo => {
            editor.redo();
            true
        }
        KeybindingAction::DeleteChar => {
            editor.delete_forwards();
            true
        }
        KeybindingAction::JoinLines => {
            editor.join_lines();
            true
        }
        KeybindingAction::ToggleCase => {
            let cursor = editor.active_cursor();
            let line = editor.buffer().line(cursor.line);
            let chars: Vec<char> = line.chars().collect();
            if cursor.col < chars.len() {
                let ch = chars[cursor.col];
                let toggled = if ch.is_uppercase() {
                    ch.to_lowercase().next().unwrap_or(ch)
                } else {
                    ch.to_uppercase().next().unwrap_or(ch)
                };
                let _old = ch.to_string();
                let new = toggled.to_string();
                editor.delete_selection();
                editor.insert_text(&new);
            }
            true
        }

        // Mode transitions
        KeybindingAction::EnterInsert => {
            editor.set_vim_mode(KeybindingMode::Insert);
            true
        }
        KeybindingAction::EnterInsertAtLineStart => {
            editor.set_vim_mode(KeybindingMode::Insert);
            let buf = editor.buffer().clone();
            let mut cursor = editor.active_cursor();
            super::motions::move_line_start(&buf, &mut cursor);
            editor.update_active_cursor(cursor);
            editor.update_active_selection(Selection::caret(cursor));
            true
        }
        KeybindingAction::EnterInsertAfter => {
            editor.set_vim_mode(KeybindingMode::Insert);
            editor.move_cursor_right();
            true
        }
        KeybindingAction::EnterInsertBelow => {
            editor.set_vim_mode(KeybindingMode::Insert);
            let cursor = editor.active_cursor();
            let next_line = cursor.line + 1;
            editor.move_cursor_to(next_line, 0);
            editor.insert_text("\n");
            editor.move_cursor_to(next_line, 0);
            true
        }
        KeybindingAction::EnterVisual => {
            editor.set_vim_mode(KeybindingMode::Visual);
            let cursor = editor.active_cursor();
            editor.keybinding_state_mut().state.visual_start = Some(cursor);
            editor.update_active_selection(Selection::range(cursor, cursor));
            true
        }
        KeybindingAction::EnterVisualLine => {
            editor.set_vim_mode(KeybindingMode::Visual);
            let cursor = editor.active_cursor();
            editor.keybinding_state_mut().state.visual_start = Some(cursor);
            let start = Cursor::new(cursor.line, 0);
            let end = Cursor::new(cursor.line, editor.buffer().line_len(cursor.line));
            editor.update_active_selection(Selection::range(start, end));
            true
        }
        KeybindingAction::EnterCommand => {
            editor.set_vim_mode(KeybindingMode::Command);
            editor.keybinding_state_mut().state.command_buffer.clear();
            true
        }
        KeybindingAction::ExitToNormal => {
            editor.set_vim_mode(KeybindingMode::Normal);
            editor.update_active_selection(Selection::caret(editor.active_cursor()));
            true
        }

        // Selection
        KeybindingAction::SelectChar => {
            editor.select_word();
            true
        }
        KeybindingAction::SelectLine => {
            editor.select_line();
            true
        }

        // Insert mode pass-through
        KeybindingAction::InsertChar(ch) => {
            editor.insert_text(&ch);
            true
        }
        KeybindingAction::Backspace => {
            editor.delete_backwards();
            true
        }
        KeybindingAction::Delete => {
            editor.delete_forwards();
            true
        }
        KeybindingAction::Newline => {
            editor.auto_indent_newline();
            true
        }
        KeybindingAction::Tab => {
            editor.insert_text("  ");
            true
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::editor::Editor;

    #[test]
    fn execute_move_left() {
        let mut editor = Editor::with_content("hello");
        editor.move_cursor_to(0, 3);
        execute_action(
            &mut editor,
            KeybindingAction::MoveLeft,
            KeybindingMode::Normal,
            None,
        );
        let cursor = editor.active_cursor();
        assert_eq!(cursor.col, 2);
    }

    #[test]
    fn execute_delete_line() {
        let mut editor = Editor::with_content("hello\nworld");
        execute_action(
            &mut editor,
            KeybindingAction::DeleteLine,
            KeybindingMode::Normal,
            None,
        );
        assert_eq!(editor.content(), "world");
    }

    #[test]
    fn execute_undo() {
        let mut editor = Editor::new();
        editor.insert_text("hello");
        execute_action(
            &mut editor,
            KeybindingAction::Undo,
            KeybindingMode::Normal,
            None,
        );
        assert_eq!(editor.content(), "");
    }

    #[test]
    fn execute_redo() {
        let mut editor = Editor::new();
        editor.insert_text("hello");
        editor.undo();
        execute_action(
            &mut editor,
            KeybindingAction::Redo,
            KeybindingMode::Normal,
            None,
        );
        assert_eq!(editor.content(), "hello");
    }

    #[test]
    fn execute_insert_char() {
        let mut editor = Editor::new();
        execute_action(
            &mut editor,
            KeybindingAction::InsertChar("a".to_string()),
            KeybindingMode::Insert,
            None,
        );
        assert_eq!(editor.content(), "a");
    }

    #[test]
    fn execute_backspace() {
        let mut editor = Editor::with_content("hello");
        editor.move_cursor_to(0, 5);
        execute_action(
            &mut editor,
            KeybindingAction::Backspace,
            KeybindingMode::Insert,
            None,
        );
        assert_eq!(editor.content(), "hell");
    }

    #[test]
    fn execute_newline() {
        let mut editor = Editor::with_content("hello");
        editor.move_cursor_to(0, 5);
        execute_action(
            &mut editor,
            KeybindingAction::Newline,
            KeybindingMode::Insert,
            None,
        );
        assert!(editor.content().contains('\n'));
    }

    #[test]
    fn execute_enter_insert() {
        let mut editor = Editor::new();
        execute_action(
            &mut editor,
            KeybindingAction::EnterInsert,
            KeybindingMode::Normal,
            None,
        );
        assert_eq!(editor.vim_mode(), KeybindingMode::Insert);
    }

    #[test]
    fn execute_exit_to_normal() {
        let mut editor = Editor::new();
        execute_action(
            &mut editor,
            KeybindingAction::ExitToNormal,
            KeybindingMode::Insert,
            None,
        );
        assert_eq!(editor.vim_mode(), KeybindingMode::Normal);
    }

    #[test]
    fn execute_join_lines() {
        let mut editor = Editor::with_content("hello\nworld");
        execute_action(
            &mut editor,
            KeybindingAction::JoinLines,
            KeybindingMode::Normal,
            None,
        );
        assert!(editor.content().contains("hello world"));
    }

    #[test]
    fn execute_delete_char() {
        let mut editor = Editor::with_content("hello");
        execute_action(
            &mut editor,
            KeybindingAction::DeleteChar,
            KeybindingMode::Normal,
            None,
        );
        assert_eq!(editor.content(), "ello");
    }
}
