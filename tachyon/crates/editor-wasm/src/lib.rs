//! WASM bindings for the Tachyon editor model (ADR-113 Phase A).
//!
//! The MODEL is `tachyon-editor` (shared with desktop/CLI — one source of
//! truth). This crate is the JS seam: it implements the FROZEN bridge
//! contract from `apps/web/src/lib/editor/contract.ts` so the Solid editor
//! view and the Leptos view are drop-in compatible.
//!
//! Design mirrors engine-wasm: real logic in plain Rust functions (natively
//! unit-tested), thin `#[wasm_bindgen]` wrappers on top.

use tachyon_editor::Editor;
use wasm_bindgen::prelude::*;

/// Internal editor model handle exposed to JS.
#[wasm_bindgen]
pub struct TachyonEditor {
    inner: Editor,
    /// Content snapshot at last save — `is_dirty` compares against it
    /// (Editor::set_content resets its own dirty flag, but callers save
    /// asynchronously and must not reload content just to mark clean).
    last_saved: String,
}

#[wasm_bindgen]
impl TachyonEditor {
    #[wasm_bindgen(constructor)]
    pub fn new(content: String) -> TachyonEditor {
        TachyonEditor {
            inner: Editor::with_content(&content),
            last_saved: content,
        }
    }

    // ------------------------------------------------------------------
    // Frozen bridge contract
    // ------------------------------------------------------------------

    /// Insert text at the caret (bridge: `insertText`).
    #[wasm_bindgen(js_name = insertText)]
    pub fn insert_text(&mut self, text: String) {
        self.inner.insert_text(&text);
    }

    /// Named-key dispatch (bridge: `key`).
    ///
    /// Semantics frozen in contract.ts:
    /// - recognized names map to editor commands
    /// - single characters insert literally
    /// - unrecognized multi-char names are IGNORED (regression guard: a
    ///   previous bug typed "ControlEnd" into a production document)
    pub fn key(&mut self, key: String) {
        dispatch_key(&mut self.inner, &key);
    }

    /// Full document text (bridge: `content`).
    pub fn content(&self) -> String {
        self.inner.content()
    }

    // ------------------------------------------------------------------
    // View API (rendering + status bar)
    // ------------------------------------------------------------------

    /// Number of lines in the document.
    #[wasm_bindgen(js_name = lineCount)]
    pub fn line_count(&self) -> usize {
        self.inner.buffer().len_lines()
    }

    /// Content of one line (no trailing newline).
    pub fn line(&self, idx: usize) -> String {
        self.inner
            .buffer()
            .line(idx)
            .trim_end_matches('\n')
            .to_string()
    }

    /// Cursor position as `[line, col]` (0-based).
    pub fn cursor(&self) -> Vec<usize> {
        let (c, _) = self.inner.cursors().active();
        vec![c.line, c.col]
    }

    /// Unsaved-changes flag: content differs from the last `mark_saved`.
    #[wasm_bindgen(js_name = isDirty)]
    pub fn is_dirty(&self) -> bool {
        self.inner.content() != self.last_saved
    }

    /// Mark the current content as saved (after a successful PUT/autosave).
    #[wasm_bindgen(js_name = markSaved)]
    pub fn mark_saved(&mut self) {
        self.last_saved = self.inner.content();
    }

    /// Replace the whole document (external sync / version restore).
    /// Resets cursor to start and the dirty flag (matches desktop behavior).
    #[wasm_bindgen(js_name = setContent)]
    pub fn set_content(&mut self, content: String) {
        self.inner.set_content(&content);
        self.last_saved = content;
    }

    #[wasm_bindgen(js_name = wordCount)]
    pub fn word_count(&self) -> usize {
        self.inner.word_count()
    }

    #[wasm_bindgen(js_name = charCount)]
    pub fn char_count(&self) -> usize {
        self.inner.char_count()
    }

    // ------------------------------------------------------------------
    // CRDT sync (collab Phase E — thin pass-throughs)
    // ------------------------------------------------------------------

    /// Encode a local-update delta for the sync server.
    #[wasm_bindgen(js_name = encodeUpdate)]
    pub fn encode_update(&mut self) -> Vec<u8> {
        self.inner.encode_update()
    }

    /// Apply a remote-update delta; true when it changed the document.
    #[wasm_bindgen(js_name = applyRemoteUpdate)]
    pub fn apply_remote_update(&mut self, update: Vec<u8>) -> bool {
        self.inner.apply_remote_update(&update)
    }

    /// Full-state encoding (initial sync payload).
    #[wasm_bindgen(js_name = encodeFullState)]
    pub fn encode_full_state(&self) -> Vec<u8> {
        self.inner.encode_full_state()
    }
}

/// Plain (natively testable) key dispatch — frozen contract semantics.
pub fn dispatch_key(editor: &mut Editor, key: &str) {
    match key {
        "Enter" => editor.auto_indent_newline(),
        "Backspace" => editor.delete_backwards(),
        "Delete" => editor.delete_forwards(),
        "Tab" => editor.indent_selection(),
        "Home" => editor.move_cursor_home(),
        "End" => editor.move_cursor_end(),
        "ArrowLeft" => editor.move_cursor_left(),
        "ArrowRight" => editor.move_cursor_right(),
        "ArrowUp" => editor.move_cursor_up(),
        "ArrowDown" => editor.move_cursor_down(),
        // Single characters are typing shortcuts; multi-char names are
        // unrecognized key identifiers (e.g. "ControlEnd") — ignore them
        // instead of typing them literally into the document.
        k if k.chars().count() == 1 => editor.insert_text(k),
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ed(content: &str) -> Editor {
        Editor::with_content(content)
    }

    #[test]
    fn single_char_types() {
        let mut e = ed("");
        dispatch_key(&mut e, "a");
        assert_eq!(e.content(), "a");
    }

    #[test]
    fn unknown_key_names_ignored() {
        let mut e = ed("keep");
        dispatch_key(&mut e, "ControlEnd");
        dispatch_key(&mut e, "ShiftHome");
        dispatch_key(&mut e, "MetaArrowUp");
        assert_eq!(e.content(), "keep");
    }

    #[test]
    fn named_keys_do_not_type() {
        let mut e = ed("line1\nline2");
        for k in [
            "End",
            "ArrowDown",
            "Home",
            "ArrowLeft",
            "ArrowUp",
            "ArrowRight",
        ] {
            dispatch_key(&mut e, k);
        }
        assert_eq!(e.content(), "line1\nline2");
    }

    #[test]
    fn enter_creates_line() {
        let mut e = ed("ab");
        dispatch_key(&mut e, "Home");
        dispatch_key(&mut e, "ArrowRight");
        dispatch_key(&mut e, "ArrowRight");
        dispatch_key(&mut e, "Enter");
        assert_eq!(e.content(), "ab\n");
    }

    #[test]
    fn backspace_deletes() {
        let mut e = ed("abc");
        dispatch_key(&mut e, "End");
        dispatch_key(&mut e, "Backspace");
        assert_eq!(e.content(), "ab");
    }

    #[test]
    fn tab_indents() {
        let mut e = ed("x");
        dispatch_key(&mut e, "Home");
        dispatch_key(&mut e, "Tab");
        assert!(e.content().starts_with(' ') || e.content().starts_with('\t'));
    }

    #[test]
    fn insert_and_content_round_trip() {
        let mut e = ed("# Title");
        e.insert_text("\n\nBODY");
        assert!(e.content().contains("# Title"));
        assert!(e.content().contains("BODY"));
    }

    #[test]
    fn lines_and_cursor() {
        let mut w = TachyonEditor::new("one\ntwo\nthree".into());
        assert_eq!(w.line_count(), 3);
        assert_eq!(w.line(1), "two");
        w.key("End".into());
        w.key("ArrowDown".into());
        let c = w.cursor();
        assert_eq!(c[0], 1);
        assert_eq!(c[1], 3); // desired col 3 clamped to len("two") = 3
    }

    #[test]
    fn dirty_tracking() {
        let mut w = TachyonEditor::new("hello".into());
        assert!(!w.is_dirty());
        w.insert_text("!".into());
        assert!(w.is_dirty());
        w.mark_saved();
        assert!(!w.is_dirty());
        w.set_content("fresh".into());
        assert!(!w.is_dirty());
        assert_eq!(w.content(), "fresh");
    }
}
