pub mod actions;
pub mod motions;
pub mod state_machine;

use crate::cursor::Cursor;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum KeybindingMode {
    #[default]
    Normal,
    Insert,
    Visual,
    Command,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KeybindingAction {
    // Motions
    MoveLeft,
    MoveDown,
    MoveUp,
    MoveRight,
    MoveWordForward,
    MoveWordBackward,
    MoveWordEnd,
    MoveLineStart,
    MoveLineEnd,
    MoveDocStart,
    MoveDocEnd,

    // Editing
    DeleteLine,
    YankLine,
    PasteAfter,
    PasteBefore,
    Undo,
    Redo,
    DeleteChar,
    JoinLines,
    ToggleCase,

    // Mode transitions
    EnterInsert,
    EnterInsertAtLineStart,
    EnterInsertAfter,
    EnterInsertBelow,
    EnterVisual,
    EnterVisualLine,
    EnterCommand,
    ExitToNormal,

    // Selection
    SelectChar,
    SelectLine,

    // Pass-through (insert mode)
    InsertChar(String),
    Backspace,
    Delete,
    Newline,
    Tab,
}

pub struct KeybindingState {
    pub mode: KeybindingMode,
    pub command_buffer: String,
    pub register: Option<char>,
    pub last_motion_count: usize,
    pub visual_start: Option<Cursor>,
}

impl KeybindingState {
    pub fn new() -> Self {
        Self {
            mode: KeybindingMode::Normal,
            command_buffer: String::new(),
            register: None,
            last_motion_count: 1,
            visual_start: None,
        }
    }
}

impl Default for KeybindingState {
    fn default() -> Self {
        Self::new()
    }
}
