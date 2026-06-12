use crate::keybindings::{KeybindingAction, KeybindingMode, KeybindingState};

pub struct KeybindingStateMachine {
    pub state: KeybindingState,
    pub timeout_ms: u64,
}

impl KeybindingStateMachine {
    pub fn new() -> Self {
        Self {
            state: KeybindingState::new(),
            timeout_ms: 1000,
        }
    }

    pub fn process_key(&mut self, key: &str, ctrl: bool, shift: bool) -> Option<KeybindingAction> {
        match self.state.mode {
            KeybindingMode::Normal => self.process_normal(key, ctrl, shift),
            KeybindingMode::Insert => self.process_insert(key, ctrl, shift),
            KeybindingMode::Visual => self.process_visual(key, ctrl, shift),
            KeybindingMode::Command => self.process_command(key),
        }
    }

    fn process_normal(&mut self, key: &str, ctrl: bool, _shift: bool) -> Option<KeybindingAction> {
        // Handle Ctrl sequences first (they don't go into command buffer)
        if ctrl {
            return match key {
                "r" => Some(KeybindingAction::Redo),
                _ => None,
            };
        }

        self.state.command_buffer.push_str(key);

        // Check for register prefix
        if self.state.command_buffer.len() == 1 && key.starts_with('"') {
            let reg = key.chars().nth(1);
            self.state.register = reg;
            return None;
        }

        // Check for count prefix
        if self.state.command_buffer.len() == 1 && key.chars().all(|c| c.is_ascii_digit()) {
            return None;
        }

        let buf = &self.state.command_buffer;

        let action = match buf.as_str() {
            // Single-key motions
            "h" => Some(KeybindingAction::MoveLeft),
            "j" => Some(KeybindingAction::MoveDown),
            "k" => Some(KeybindingAction::MoveUp),
            "l" => Some(KeybindingAction::MoveRight),
            "w" => Some(KeybindingAction::MoveWordForward),
            "b" => Some(KeybindingAction::MoveWordBackward),
            "e" => Some(KeybindingAction::MoveWordEnd),
            "0" => Some(KeybindingAction::MoveLineStart),
            "$" => Some(KeybindingAction::MoveLineEnd),
            "x" => Some(KeybindingAction::DeleteChar),
            "p" => Some(KeybindingAction::PasteAfter),
            "P" => Some(KeybindingAction::PasteBefore),
            "u" => Some(KeybindingAction::Undo),
            "J" => Some(KeybindingAction::JoinLines),
            "~" => Some(KeybindingAction::ToggleCase),
            "v" => Some(KeybindingAction::EnterVisual),
            "V" => Some(KeybindingAction::EnterVisualLine),
            ":" => Some(KeybindingAction::EnterCommand),
            "i" => Some(KeybindingAction::EnterInsert),
            "I" => Some(KeybindingAction::EnterInsertAtLineStart),
            "a" => Some(KeybindingAction::EnterInsertAfter),
            "o" => Some(KeybindingAction::EnterInsertBelow),

            // Two-key sequences
            "dd" => Some(KeybindingAction::DeleteLine),
            "yy" => Some(KeybindingAction::YankLine),
            "gg" => Some(KeybindingAction::MoveDocStart),

            _ => {
                // Check if incomplete (might be prefix of longer sequence)
                if "dgy".contains(buf.as_str()) || buf == "gg" {
                    return None;
                }
                // Invalid sequence - clear and ignore
                self.state.command_buffer.clear();
                return None;
            }
        };

        self.state.command_buffer.clear();

        // Update mode based on action
        if let Some(ref a) = action {
            match a {
                KeybindingAction::EnterInsert | KeybindingAction::EnterInsertAtLineStart | KeybindingAction::EnterInsertAfter | KeybindingAction::EnterInsertBelow => {
                    self.state.mode = KeybindingMode::Insert;
                }
                KeybindingAction::EnterVisual | KeybindingAction::EnterVisualLine => {
                    self.state.mode = KeybindingMode::Visual;
                }
                KeybindingAction::EnterCommand => {
                    self.state.mode = KeybindingMode::Command;
                }
                _ => {}
            }
        }

        action
    }

    fn process_insert(
        &mut self,
        key: &str,
        ctrl: bool,
        _shift: bool,
    ) -> Option<KeybindingAction> {
        match key {
            "Escape" => {
                self.state.mode = KeybindingMode::Normal;
                Some(KeybindingAction::ExitToNormal)
            }
            "Backspace" => Some(KeybindingAction::Backspace),
            "Delete" => Some(KeybindingAction::Delete),
            "Enter" => Some(KeybindingAction::Newline),
            "Tab" => Some(KeybindingAction::Tab),
            "Ctrl-r" if ctrl => Some(KeybindingAction::Redo),
            _ if ctrl => None,
            _ => Some(KeybindingAction::InsertChar(key.to_string())),
        }
    }

    fn process_visual(
        &mut self,
        key: &str,
        ctrl: bool,
        _shift: bool,
    ) -> Option<KeybindingAction> {
        self.state.command_buffer.push_str(key);

        let buf = &self.state.command_buffer;

        let action = match buf.as_str() {
            "Escape" => {
                self.state.mode = KeybindingMode::Normal;
                Some(KeybindingAction::ExitToNormal)
            }
            "h" => Some(KeybindingAction::MoveLeft),
            "j" => Some(KeybindingAction::MoveDown),
            "k" => Some(KeybindingAction::MoveUp),
            "l" => Some(KeybindingAction::MoveRight),
            "w" => Some(KeybindingAction::MoveWordForward),
            "b" => Some(KeybindingAction::MoveWordBackward),
            "e" => Some(KeybindingAction::MoveWordEnd),
            "0" => Some(KeybindingAction::MoveLineStart),
            "$" => Some(KeybindingAction::MoveLineEnd),
            "x" | "d" => Some(KeybindingAction::DeleteLine),
            "y" => Some(KeybindingAction::YankLine),
            "p" => Some(KeybindingAction::PasteAfter),
            "P" => Some(KeybindingAction::PasteBefore),
            "u" => Some(KeybindingAction::Undo),
            "J" => Some(KeybindingAction::JoinLines),
            "~" => Some(KeybindingAction::ToggleCase),
            "gg" => Some(KeybindingAction::MoveDocStart),
            "G" => Some(KeybindingAction::MoveDocEnd),
            _ => {
                if buf.len() <= 2 && "dgy".contains(buf.as_str()) {
                    return None;
                }
                self.state.command_buffer.clear();
                return None;
            }
        };

        if ctrl {
            self.state.command_buffer.clear();
            return None;
        }

        self.state.command_buffer.clear();
        action
    }

    fn process_command(&mut self, key: &str) -> Option<KeybindingAction> {
        match key {
            "Escape" => {
                self.state.mode = KeybindingMode::Normal;
                self.state.command_buffer.clear();
                None
            }
            "Enter" => {
                let cmd = self.state.command_buffer.clone();
                self.state.command_buffer.clear();
                self.state.mode = KeybindingMode::Normal;
                self.execute_command(&cmd)
            }
            _ => {
                self.state.command_buffer.push_str(key);
                None
            }
        }
    }

    fn execute_command(&mut self, cmd: &str) -> Option<KeybindingAction> {
        match cmd.trim() {
            "w" => None,
            "q" => None,
            "wq" => None,
            _ => None,
        }
    }

    pub fn mode(&self) -> KeybindingMode {
        self.state.mode
    }

    pub fn set_mode(&mut self, mode: KeybindingMode) {
        self.state.mode = mode;
    }
}

impl Default for KeybindingStateMachine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normal_mode_single_key() {
        let mut sm = KeybindingStateMachine::new();
        assert_eq!(sm.process_key("h", false, false), Some(KeybindingAction::MoveLeft));
    }

    #[test]
    fn normal_to_insert() {
        let mut sm = KeybindingStateMachine::new();
        assert_eq!(sm.process_key("i", false, false), Some(KeybindingAction::EnterInsert));
        assert_eq!(sm.mode(), KeybindingMode::Insert);
    }

    #[test]
    fn insert_to_normal_escape() {
        let mut sm = KeybindingStateMachine::new();
        sm.set_mode(KeybindingMode::Insert);
        assert_eq!(sm.process_key("Escape", false, false), Some(KeybindingAction::ExitToNormal));
        assert_eq!(sm.mode(), KeybindingMode::Normal);
    }

    #[test]
    fn insert_passthrough_char() {
        let mut sm = KeybindingStateMachine::new();
        sm.set_mode(KeybindingMode::Insert);
        assert_eq!(
            sm.process_key("a", false, false),
            Some(KeybindingAction::InsertChar("a".to_string()))
        );
    }

    #[test]
    fn double_key_sequence_dd() {
        let mut sm = KeybindingStateMachine::new();
        assert_eq!(sm.process_key("d", false, false), None);
        assert_eq!(sm.process_key("d", false, false), Some(KeybindingAction::DeleteLine));
    }

    #[test]
    fn double_key_sequence_yy() {
        let mut sm = KeybindingStateMachine::new();
        assert_eq!(sm.process_key("y", false, false), None);
        assert_eq!(sm.process_key("y", false, false), Some(KeybindingAction::YankLine));
    }

    #[test]
    fn double_key_sequence_gg() {
        let mut sm = KeybindingStateMachine::new();
        assert_eq!(sm.process_key("g", false, false), None);
        assert_eq!(sm.process_key("g", false, false), Some(KeybindingAction::MoveDocStart));
    }

    #[test]
    fn ctrl_r_redo() {
        let mut sm = KeybindingStateMachine::new();
        assert_eq!(sm.process_key("r", true, false), Some(KeybindingAction::Redo));
    }

    #[test]
    fn visual_mode_transition() {
        let mut sm = KeybindingStateMachine::new();
        assert_eq!(sm.process_key("v", false, false), Some(KeybindingAction::EnterVisual));
        assert_eq!(sm.mode(), KeybindingMode::Visual);
    }

    #[test]
    fn normal_to_visual_escape() {
        let mut sm = KeybindingStateMachine::new();
        sm.set_mode(KeybindingMode::Visual);
        assert_eq!(sm.process_key("Escape", false, false), Some(KeybindingAction::ExitToNormal));
        assert_eq!(sm.mode(), KeybindingMode::Normal);
    }

    #[test]
    fn invalid_sequence_resets() {
        let mut sm = KeybindingStateMachine::new();
        // "q" is not a valid sequence prefix
        assert_eq!(sm.process_key("q", false, false), None);
        // Buffer should be cleared, so next key is fresh
        assert_eq!(sm.process_key("h", false, false), Some(KeybindingAction::MoveLeft));
    }
}
