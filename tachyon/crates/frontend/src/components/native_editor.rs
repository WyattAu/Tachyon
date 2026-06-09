#![allow(clippy::redundant_locals)]
use crate::components::collaborative_cursors::{AwarenessState, CollaborativeCursors};
use crate::components::wikilink_autocomplete::{WikilinkAutocomplete, WikilinkCompletion};
use leptos::prelude::*;
use leptos_use::{use_event_listener, use_resize_observer};
use tachyon_editor::{Cursor, Editor, HighlightSpan, HighlightToken, Selection};
use wasm_bindgen::JsCast;
use web_sys::{KeyboardEvent, MouseEvent};

const LINE_HEIGHT_PX: f64 = 22.0;
const CHAR_WIDTH_PX: f64 = 8.0;

fn css_class(token: &HighlightToken) -> &'static str {
    match token {
        HighlightToken::Heading1 => "ed-h1",
        HighlightToken::Heading2 => "ed-h2",
        HighlightToken::Heading3 => "ed-h3",
        HighlightToken::Heading4 => "ed-h4",
        HighlightToken::Heading5 => "ed-h5",
        HighlightToken::Heading6 => "ed-h6",
        HighlightToken::Bold => "ed-bold",
        HighlightToken::Italic => "ed-italic",
        HighlightToken::BoldItalic => "ed-bold-italic",
        HighlightToken::Strikethrough => "ed-strikethrough",
        HighlightToken::CodeInline => "ed-code-inline",
        HighlightToken::Link => "ed-link",
        HighlightToken::LinkUrl => "ed-link-url",
        HighlightToken::LinkText => "ed-link-text",
        HighlightToken::Image => "ed-image",
        HighlightToken::ImageUrl => "ed-image-url",
        HighlightToken::ImageAlt => "ed-image-alt",
        HighlightToken::WikiLink => "ed-wiki-link",
        HighlightToken::Blockquote => "ed-blockquote",
        HighlightToken::ListItem => "ed-list-item",
        HighlightToken::ListMarker => "ed-list-marker",
        HighlightToken::HorizontalRule => "ed-hr",
        HighlightToken::CodeBlock => "ed-code-block",
        HighlightToken::Frontmatter => "ed-frontmatter",
        HighlightToken::Tag => "ed-tag",
        HighlightToken::TaskMarker => "ed-task-marker",
        HighlightToken::TableHeader => "ed-table-header",
        HighlightToken::TableCell => "ed-table-cell",
        HighlightToken::TableBorder => "ed-table-border",
        HighlightToken::Text => "ed-text",
        HighlightToken::Whitespace => "ed-whitespace",
        _ => "ed-text",
    }
}

/// Shared key handler — called by both `on:keydown` and the global window listener.
/// Returns `true` if the key was handled (editor operation performed).
fn handle_editor_key(
    ctrl: bool,
    shift: bool,
    key: &str,
    editor: RwSignal<Editor>,
    find_visible: Signal<bool>,
) -> bool {
    editor
        .try_update(|e| {
            match (ctrl, shift, key) {
                (true, false, "z") => {
                    e.undo();
                    true
                }
                (true, true, "z") | (true, true, "Z") => {
                    e.redo();
                    true
                }
                (true, false, "a") | (true, false, "A") => {
                    e.select_all();
                    true
                }
                (true, false, "s") | (true, false, "S") => {
                    false /* save externally */
                }
                (true, false, "d") | (true, false, "D") => {
                    e.add_cursor_at_next_occurrence();
                    true
                }
                (false, false, "Enter") => {
                    e.auto_indent_newline();
                    true
                }
                (false, false, "Backspace") => {
                    e.delete_backwards();
                    true
                }
                (false, false, "Delete") => {
                    e.delete_forwards();
                    true
                }
                (false, false, "Tab") => {
                    e.indent_selection();
                    true
                }
                (true, false, "Tab") => {
                    e.unindent_selection();
                    true
                }

                // Arrow (no modifier)
                (false, false, "ArrowLeft") => {
                    e.move_cursor_left();
                    true
                }
                (false, false, "ArrowRight") => {
                    e.move_cursor_right();
                    true
                }
                (false, false, "ArrowUp") => {
                    e.move_cursor_up();
                    true
                }
                (false, false, "ArrowDown") => {
                    e.move_cursor_down();
                    true
                }
                (false, false, "Home") => {
                    e.move_cursor_home();
                    true
                }
                (false, false, "End") => {
                    e.move_cursor_end();
                    true
                }

                // Shift+Arrow (selection extend)
                (false, true, "ArrowLeft") => {
                    let mut t = e.cursors().active().0;
                    t.move_left(e.buffer());
                    e.extend_selection_to(t);
                    true
                }
                (false, true, "ArrowRight") => {
                    let mut t = e.cursors().active().0;
                    t.move_right(e.buffer());
                    e.extend_selection_to(t);
                    true
                }
                (false, true, "ArrowUp") => {
                    let mut t = e.cursors().active().0;
                    t.move_up(e.buffer());
                    e.extend_selection_to(t);
                    true
                }
                (false, true, "ArrowDown") => {
                    let mut t = e.cursors().active().0;
                    t.move_down(e.buffer());
                    e.extend_selection_to(t);
                    true
                }
                (false, true, "Home") => {
                    let mut t = e.cursors().active().0;
                    t.move_home();
                    e.extend_selection_to(t);
                    true
                }
                (false, true, "End") => {
                    let mut t = e.cursors().active().0;
                    t.move_end(e.buffer());
                    e.extend_selection_to(t);
                    true
                }

                // Ctrl+Arrow (word/document nav)
                (true, false, "ArrowLeft") | (true, false, "ArrowUp") => {
                    e.move_cursor_word_left();
                    true
                }
                (true, false, "ArrowRight") | (true, false, "ArrowDown") => {
                    e.move_cursor_word_right();
                    true
                }
                (true, false, "Home") => {
                    e.move_cursor_to(0, 0);
                    true
                }
                (true, false, "End") => {
                    let last = e.buffer().len_lines().saturating_sub(1);
                    e.move_cursor_to(last, e.buffer().line_len(last));
                    true
                }

                // Ctrl+Shift+Arrow (word/document selection)
                (true, true, "ArrowLeft") | (true, true, "ArrowUp") => {
                    let mut t = e.cursors().active().0;
                    t.move_word_left(e.buffer());
                    e.extend_selection_to(t);
                    true
                }
                (true, true, "ArrowRight") | (true, true, "ArrowDown") => {
                    let mut t = e.cursors().active().0;
                    t.move_word_right(e.buffer());
                    e.extend_selection_to(t);
                    true
                }
                (true, true, "Home") => {
                    e.extend_selection_to(Cursor::zero());
                    true
                }
                (true, true, "End") => {
                    let last = e.buffer().len_lines().saturating_sub(1);
                    e.extend_selection_to(Cursor::new(last, e.buffer().line_len(last)));
                    true
                }

                // Word operations
                (true, false, "Backspace") => {
                    e.delete_word_backwards();
                    true
                }
                (true, false, "Delete") => {
                    e.delete_word_forwards();
                    true
                }
                (true, false, "l") => {
                    e.select_line();
                    true
                }
                (true, false, "w") => {
                    e.select_word();
                    true
                }
                (true, _, "k") => {
                    e.delete_line();
                    true
                }

                // Line operations
                (true, false, "j") => {
                    e.join_lines();
                    true
                }
                (true, false, "/") if !find_visible.get() => {
                    e.toggle_line_comment();
                    true
                }

                // Code folding
                (true, true, "[") => {
                    e.fold_at_cursor();
                    true
                }
                (true, true, "]") => {
                    e.unfold_at_cursor();
                    true
                }

                // Clipboard
                (true, false, "c") => {
                    if let Some(sel_text) = e.get_selected_text() {
                        let _ = js_sys::eval(&format!(
                            "navigator.clipboard.writeText('{}')",
                            sel_text.replace('\\', "\\\\").replace('\'', "\\'")
                        ));
                    }
                    true
                }
                (true, false, "x") => {
                    if let Some(sel_text) = e.get_selected_text() {
                        let _ = js_sys::eval(&format!(
                            "navigator.clipboard.writeText('{}')",
                            sel_text.replace('\\', "\\\\").replace('\'', "\\'")
                        ));
                    }
                    e.delete_selection();
                    true
                }
                (true, false, "v") => {
                    // Paste handled externally via async clipboard API
                    false
                }

                // Auto-close brackets
                (false, false, "(" | "[" | "{" | "\"" | "'") => {
                    e.auto_close_bracket(key.chars().next().unwrap_or(' '));
                    true
                }

                _ => false,
            }
        })
        .unwrap_or(false)
}

#[component]
pub fn NativeEditor(
    #[prop(default = String::new())] content: String,
    #[prop(optional)] editor: Option<RwSignal<Editor>>,
    #[prop(default = String::new())]
    #[allow(unused)]
    document_id: String,
    #[prop(default = true)]
    #[allow(unused)]
    editable: bool,
    #[prop(default = "Start writing...".to_string())] placeholder: String,
    #[prop(default = Callback::new(|_: String| {}))] on_change: Callback<String>,
    #[prop(default = "native-editor".to_string())] class_name: String,
    #[prop(default = true)] line_numbers: bool,
    #[prop(default = true)] word_wrap: bool,
    #[prop(default = "14px".to_string())] font_size: String,
    #[prop(optional)] awareness: Option<RwSignal<AwarenessState>>,
    #[prop(optional)] render_trigger: Option<RwSignal<u64>>,
) -> impl IntoView {
    let editor = match editor {
        Some(sig) => sig,
        None => RwSignal::new(Editor::with_content(&content)),
    };
    let container_ref = NodeRef::<leptos::html::Div>::new();
    let scroll_offset = RwSignal::new((0usize, 0usize));
    let visible_lines = RwSignal::new(50usize);
    let container_height = RwSignal::new(500.0f64);
    let is_focused = RwSignal::new(false);
    let is_mouse_dragging = RwSignal::new(false);
    // Render tick: bumped by editor operations to trigger view re-render.
    // The view! closure tracks this so it re-reads editor state on each tick.
    let render_tick = match render_trigger {
        Some(sig) => sig,
        None => RwSignal::new(0u64),
    };

    let (wl_visible, set_wl_visible) = signal(false);
    let (wl_query, set_wl_query) = signal(String::new());
    let (wl_position, set_wl_position) = signal((0.0, 0.0));

    // Find/Replace state
    let (find_visible, set_find_visible) = signal(false);
    let (find_query, set_find_query) = signal(String::new());
    let (find_replace, set_find_replace) = signal(String::new());
    let (find_match_count, set_find_match_count) = signal(0usize);
    let (find_current_idx, set_find_current_idx) = signal(0usize);
    let (find_case_sensitive, set_find_case_sensitive) = signal(false);
    let (find_whole_word, set_find_whole_word) = signal(false);
    let (find_show_replace, set_find_show_replace) = signal(false);

    let editor_for_change = editor;
    let on_change_cb = on_change;

    Effect::new(move |_| {
        let content = editor_for_change.with(|e| e.content());
        let dirty = editor_for_change.with(|e| e.is_dirty());
        if dirty {
            on_change_cb.run(content);
        }
    });

    let editor_for_wl = editor;
    let wl_line_numbers = line_numbers;
    Effect::new(move |_| {
        let wl_state = editor_for_wl.with(|e| e.get_wikilink_state());
        if let Some(state) = wl_state {
            if !state.query.is_empty() {
                set_wl_query.set(state.query.clone());
                set_wl_visible.set(true);
                let gutter_w = if wl_line_numbers { 50.0 } else { 0.0 };
                let top_px = state.start_line as f64 * LINE_HEIGHT_PX + LINE_HEIGHT_PX;
                let left_px = gutter_w + state.start_col as f64 * CHAR_WIDTH_PX;
                set_wl_position.set((left_px, top_px));
            } else {
                set_wl_visible.set(false);
            }
        } else {
            set_wl_visible.set(false);
        }
    });

    let _editor_for_scroll = editor;
    let handle_scroll = move |ev: web_sys::Event| {
        if let Some(target) = ev.target() {
            if let Ok(el) = target.dyn_into::<web_sys::HtmlElement>() {
                let scroll_top = el.scroll_top() as usize;
                let line = scroll_top / LINE_HEIGHT_PX as usize;
                let line = line.saturating_sub(2);
                scroll_offset.set((line, 0));
            }
        }
    };

    let handle_keydown = move |ev: KeyboardEvent| {
        let key = ev.key();
        let ctrl = ev.ctrl_key() || ev.meta_key();
        let shift = ev.shift_key();

        let should_prevent = !ctrl
            || matches!(
                key.as_str(),
                "z" | "Z"
                    | "a"
                    | "A"
                    | "f"
                    | "F"
                    | "d"
                    | "D"
                    | "s"
                    | "S"
                    | "c"
                    | "C"
                    | "x"
                    | "X"
                    | "v"
                    | "V"
                    | "ArrowLeft"
                    | "ArrowRight"
                    | "ArrowUp"
                    | "ArrowDown"
                    | "Home"
                    | "End"
            );

        // Ctrl+F: toggle find panel (intercept before editor key handling)
        if ctrl && matches!(key.as_str(), "f" | "F") {
            ev.prevent_default();
            let new_visible = !find_visible.get();
            set_find_visible.set(new_visible);
            // Focus the find input after toggling
            if new_visible {
                let _ = js_sys::eval(
                    "setTimeout(() => { let el = document.querySelector('.find-input'); if (el) el.focus(); }, 50)",
                );
                // Seed search with current selection or word at cursor
                let seed = editor.with_untracked(|e| {
                    if let Some(sel) = e.get_selected_text() {
                        if !sel.is_empty() && !sel.contains('\n') {
                            return sel;
                        }
                    }
                    // No selection: get word at cursor
                    let cursor = e.cursors().active().0;
                    let line = e.buffer().line(cursor.line);
                    let chars: Vec<char> = line.chars().collect();
                    let mut start = cursor.col.min(chars.len());
                    // Find word boundary start
                    while start > 0
                        && chars
                            .get(start - 1)
                            .map(|c| {
                                !c.is_whitespace()
                                    && !matches!(
                                        c,
                                        '(' | ')'
                                            | '['
                                            | ']'
                                            | '{'
                                            | '}'
                                            | ','
                                            | ';'
                                            | ':'
                                            | '.'
                                            | '"'
                                            | '\''
                                    )
                            })
                            .unwrap_or(false)
                    {
                        start -= 1;
                    }
                    let mut end = cursor.col.min(chars.len());
                    while end < chars.len()
                        && chars
                            .get(end)
                            .map(|c| {
                                !c.is_whitespace()
                                    && !matches!(
                                        c,
                                        '(' | ')'
                                            | '['
                                            | ']'
                                            | '{'
                                            | '}'
                                            | ','
                                            | ';'
                                            | ':'
                                            | '.'
                                            | '"'
                                            | '\''
                                    )
                            })
                            .unwrap_or(false)
                    {
                        end += 1;
                    }
                    if end > start {
                        chars[start..end].iter().collect()
                    } else {
                        String::new()
                    }
                });
                set_find_query.set(seed.clone());
                if !seed.is_empty() {
                    let _ = editor.try_update(|e| {
                        e.find(&seed);
                    });
                    render_tick.update(|t| *t += 1);
                }
            }
            return;
        }

        // Escape: close find panel
        if key.as_str() == "Escape" && find_visible.get() {
            set_find_visible.set(false);
            ev.prevent_default();
            // Refocus editor
            if let Some(el) = container_ref.get() {
                let _ = el.focus();
            }
            return;
        }

        if should_prevent {
            ev.prevent_default();
        }

        let handled = handle_editor_key(ctrl, shift, &key, editor, find_visible.into());
        if !handled && !ctrl {
            // Auto-close bracket fallback was handled, or printable char
            let _ = editor.try_update(|e| {
                let is_printable = key.len() == 1
                    && !matches!(
                        key.as_str(),
                        "Escape"
                            | "F1"
                            | "F2"
                            | "F3"
                            | "F4"
                            | "F5"
                            | "F6"
                            | "F7"
                            | "F8"
                            | "F9"
                            | "F10"
                            | "F11"
                            | "F12"
                            | "Shift"
                            | "Control"
                            | "Alt"
                            | "Meta"
                            | "CapsLock"
                            | "NumLock"
                            | "ScrollLock"
                    );
                if is_printable {
                    e.insert_text(&key);
                }
            });
        }
        render_tick.update(|t| *t += 1);
    };

    let container_ref_for_focus = container_ref;
    let editor_for_mousedown = editor;
    let render_tick_mousedown = render_tick;
    let handle_mousedown = move |ev: MouseEvent| {
        if let Some(_target) = ev.target() {
            is_focused.set(true);
            // Explicitly focus the container so it receives keydown events.
            // contenteditable="false" means the browser won't auto-focus on click.
            if let Some(el) = container_ref_for_focus.get() {
                let _ = el.focus();
            }

            // Mouse selection: convert click position to line/col
            if let Some(container_el) = container_ref_for_focus.get() {
                let rect = container_el.get_bounding_client_rect();
                let x = (ev.client_x() as f64) - rect.left();
                let y = (ev.client_y() as f64) - rect.top();

                let scroll_top = container_el.scroll_top() as f64;
                let adjusted_y = y + scroll_top;

                let line = (adjusted_y / LINE_HEIGHT_PX).floor().max(0.0) as usize;
                let col = (x / CHAR_WIDTH_PX).floor().max(0.0) as usize;

                let _ = editor_for_mousedown.try_update(|e| {
                    let max_line = e.buffer().len_lines().saturating_sub(1);
                    let clamped_line = line.min(max_line);
                    let max_col = e.buffer().line_len(clamped_line);
                    let clamped_col = col.min(max_col);

                    // Alt+Click: add cursor at position (multi-cursor)
                    if ev.alt_key() {
                        let pos = Cursor::new(clamped_line, clamped_col);
                        e.add_cursor(pos, Selection::caret(pos));
                    } else {
                        e.move_cursor_to(clamped_line, clamped_col);
                    }

                    // Double-click: select word, Triple-click: select line
                    let detail = ev.detail();
                    if detail == 2 {
                        e.select_word();
                    } else if detail >= 3 {
                        e.select_line();
                    }
                });
                render_tick_mousedown.update(|t| *t += 1);

                // Start mouse drag tracking
                is_mouse_dragging.set(true);
            }
        }
    };

    // Mouse drag selection: extend selection as mouse moves while button is held
    let container_ref_drag = container_ref;
    let editor_for_drag = editor;
    let render_tick_drag = render_tick;
    let handle_mousemove = move |ev: MouseEvent| {
        if !is_mouse_dragging.get() {
            return;
        }
        // Only process while buttons are pressed (button 0 = primary)
        if ev.buttons() == 0 {
            is_mouse_dragging.set(false);
            return;
        }

        if let Some(container_el) = container_ref_drag.get() {
            let rect = container_el.get_bounding_client_rect();
            let x = (ev.client_x() as f64) - rect.left();
            let y = (ev.client_y() as f64) - rect.top();

            let scroll_top = container_el.scroll_top() as f64;
            let adjusted_y = y + scroll_top;

            let line = (adjusted_y / LINE_HEIGHT_PX).floor().max(0.0) as usize;
            let _col = (x / CHAR_WIDTH_PX).floor().max(0.0) as usize;
            let col = (x / CHAR_WIDTH_PX).max(0.0) as usize;

            let _ = editor_for_drag.try_update(|e| {
                let max_line = e.buffer().len_lines().saturating_sub(1);
                let clamped_line = line.min(max_line);
                let max_col = e.buffer().line_len(clamped_line);
                let clamped_col = col.min(max_col);
                e.extend_selection_to(Cursor::new(clamped_line, clamped_col));
            });
            render_tick_drag.update(|t| *t += 1);

            // Auto-scroll when dragging near edges
            let container_height_val = container_el.get_bounding_client_rect().height();
            let edge_zone = 30.0f64;
            if y < edge_zone {
                let current = container_el.scroll_top() as f64;
                let new_scroll = (current - 3.0).max(0.0);
                container_el.set_scroll_top(new_scroll as i32);
            } else if y > container_height_val - edge_zone {
                let max_scroll = container_el.scroll_height() - container_el.client_height();
                let new_scroll = (container_el.scroll_top() + 3).min(max_scroll);
                container_el.set_scroll_top(new_scroll);
            }
        }
    };

    let handle_mouseup = move |_ev: MouseEvent| {
        is_mouse_dragging.set(false);
    };

    // WebKitGTK quirk: on:keydown on a div with contenteditable="false"
    // often doesn't fire. Work around by attaching a global window keydown listener.
    // Uses manual Closure::forget() for WebKitGTK compatibility (useEventListener
    // caused Tauri IPC timing issues).
    {
        let editor_gl = editor;
        let is_focused_gl = is_focused;
        let container_ref_gl = container_ref;
        let render_tick_gl = render_tick;
        let find_vis_gl = find_visible;

        let closure = wasm_bindgen::closure::Closure::<dyn Fn(web_sys::Event)>::new(
            move |ev: web_sys::Event| {
                let ev = match ev.dyn_into::<web_sys::KeyboardEvent>() {
                    Ok(e) => e,
                    Err(_) => return,
                };
                if !is_focused_gl.get() {
                    return;
                }

                let key = ev.key();
                let ctrl = ev.ctrl_key() || ev.meta_key();
                let shift = ev.shift_key();

                let should_prevent = !ctrl
                    || matches!(
                        key.as_str(),
                        "z" | "Z"
                            | "a"
                            | "A"
                            | "f"
                            | "F"
                            | "d"
                            | "D"
                            | "s"
                            | "S"
                            | "c"
                            | "C"
                            | "x"
                            | "X"
                            | "v"
                            | "V"
                            | "ArrowLeft"
                            | "ArrowRight"
                            | "ArrowUp"
                            | "ArrowDown"
                            | "Home"
                            | "End"
                    );
                if should_prevent {
                    ev.prevent_default();
                }

                let handled = handle_editor_key(ctrl, shift, &key, editor_gl, find_vis_gl.into());
                if !handled {
                    let is_printable = key.len() == 1
                        && !ctrl
                        && !matches!(
                            key.as_str(),
                            "Escape"
                                | "F1"
                                | "F2"
                                | "F3"
                                | "F4"
                                | "F5"
                                | "F6"
                                | "F7"
                                | "F8"
                                | "F9"
                                | "F10"
                                | "F11"
                                | "F12"
                                | "Shift"
                                | "Control"
                                | "Alt"
                                | "Meta"
                                | "CapsLock"
                                | "NumLock"
                                | "ScrollLock"
                        );
                    if is_printable {
                        let _ = editor_gl.try_update(|e| {
                            e.insert_text(&key);
                        });
                    }
                }
                render_tick_gl.update(|t| *t += 1);

                // Auto-scroll: ensure cursor is visible after any key action
                if let Some(container) = container_ref_gl.get() {
                    if let Ok(el) = container.dyn_into::<web_sys::HtmlElement>() {
                        let cursor_line = editor_gl.with_untracked(|e| e.cursors().active().0.line);
                        let cursor_top = cursor_line as f64 * LINE_HEIGHT_PX;
                        let scroll_top = el.scroll_top() as f64;
                        let container_h = el.client_height() as f64;
                        if cursor_top + LINE_HEIGHT_PX > scroll_top + container_h {
                            el.set_scroll_top(
                                (cursor_top - container_h + LINE_HEIGHT_PX * 2.0) as i32,
                            );
                        }
                        if cursor_top < scroll_top {
                            el.set_scroll_top((cursor_top - LINE_HEIGHT_PX).max(0.0) as i32);
                        }
                    }
                }
            },
        );

        if let Some(window) = web_sys::window() {
            let _ = window
                .add_event_listener_with_callback("keydown", closure.as_ref().unchecked_ref());
            closure.forget();
        }
    }

    // Global mouseup: stop drag selection even if mouse released outside editor
    // Uses leptos-use useEventListener for automatic cleanup.
    if let Some(window) = web_sys::window() {
        let drag_sig = is_mouse_dragging;
        let _cleanup = use_event_listener(
            window,
            leptos::ev::mouseup,
            move |_ev: web_sys::MouseEvent| {
                drag_sig.set(false);
            },
        );
    }

    // Expose editor bridge for programmatic access (e.g. testing, automation)
    // Accessible via window.__tachyonEditor.insertText("text") etc.
    if let Some(window) = web_sys::window() {
        let editor_sig = editor;
        let render_tick1 = render_tick;
        let insert_clos =
            wasm_bindgen::closure::Closure::<dyn Fn(String)>::new(move |text: String| {
                let _ = editor_sig.try_update(|e| {
                    e.insert_text(&text);
                });
                render_tick1.update(|t| *t += 1);
            });
        let insert_fn: js_sys::Function = insert_clos
            .as_ref()
            .unchecked_ref::<js_sys::Function>()
            .clone();
        insert_clos.forget();

        let editor_sig2 = editor;
        let render_tick2 = render_tick;
        let key_clos = wasm_bindgen::closure::Closure::<dyn Fn(String)>::new(move |key: String| {
            let _ = editor_sig2.try_update(|e| match key.as_str() {
                "Enter" => e.auto_indent_newline(),
                "Backspace" => e.delete_backwards(),
                "Delete" => e.delete_forwards(),
                "Tab" => e.indent_selection(),
                "Home" => e.move_cursor_home(),
                "End" => e.move_cursor_end(),
                "ArrowLeft" => e.move_cursor_left(),
                "ArrowRight" => e.move_cursor_right(),
                "ArrowUp" => e.move_cursor_up(),
                "ArrowDown" => e.move_cursor_down(),
                _ => e.insert_text(&key),
            });
            render_tick2.update(|t| *t += 1);
        });
        let key_fn: js_sys::Function = key_clos
            .as_ref()
            .unchecked_ref::<js_sys::Function>()
            .clone();
        key_clos.forget();

        let editor_sig3 = editor;
        let content_clos = wasm_bindgen::closure::Closure::<dyn Fn() -> String>::new(move || {
            editor_sig3.with_untracked(|e| e.content().to_string())
        });
        let content_fn: js_sys::Function = content_clos
            .as_ref()
            .unchecked_ref::<js_sys::Function>()
            .clone();
        content_clos.forget();

        // toggleFind: open/close find panel
        let find_vis_bridge = find_visible;
        let set_find_bridge = set_find_visible;
        let editor_find_bridge = editor;
        let rt_find_bridge = render_tick;
        let toggle_find_clos = wasm_bindgen::closure::Closure::<dyn Fn()>::new(move || {
            let new_vis = !find_vis_bridge.get();
            set_find_bridge.set(new_vis);
            if new_vis {
                // Seed with selection or word at cursor
                let seed = editor_find_bridge.with_untracked(|e| {
                    if let Some(sel) = e.get_selected_text() {
                        if !sel.is_empty() && !sel.contains('\n') {
                            return sel;
                        }
                    }
                    let cursor = e.cursors().active().0;
                    let line = e.buffer().line(cursor.line);
                    let chars: Vec<char> = line.chars().collect();
                    let mut start = cursor.col.min(chars.len());
                    while start > 0
                        && chars
                            .get(start - 1)
                            .map(|c| {
                                !c.is_whitespace()
                                    && !matches!(
                                        c,
                                        '(' | ')'
                                            | '['
                                            | ']'
                                            | '{'
                                            | '}'
                                            | ','
                                            | ';'
                                            | ':'
                                            | '.'
                                            | '"'
                                            | '\''
                                    )
                            })
                            .unwrap_or(false)
                    {
                        start -= 1;
                    }
                    let mut end = cursor.col.min(chars.len());
                    while end < chars.len()
                        && chars
                            .get(end)
                            .map(|c| {
                                !c.is_whitespace()
                                    && !matches!(
                                        c,
                                        '(' | ')'
                                            | '['
                                            | ']'
                                            | '{'
                                            | '}'
                                            | ','
                                            | ';'
                                            | ':'
                                            | '.'
                                            | '"'
                                            | '\''
                                    )
                            })
                            .unwrap_or(false)
                    {
                        end += 1;
                    }
                    if end > start {
                        chars[start..end].iter().collect()
                    } else {
                        String::new()
                    }
                });
                let _ = editor_find_bridge.try_update(|e| {
                    e.find(&seed);
                });
                rt_find_bridge.update(|t| *t += 1);
                // Focus find input
                let _ = js_sys::eval(
                    "setTimeout(() => { var el = document.querySelector('.find-input'); if (el) el.focus(); }, 100)",
                );
            }
        });
        let toggle_find_fn: js_sys::Function = toggle_find_clos
            .as_ref()
            .unchecked_ref::<js_sys::Function>()
            .clone();
        toggle_find_clos.forget();

        // find: run a search query
        let editor_find2 = editor;
        let rt_find2 = render_tick;
        let find_clos =
            wasm_bindgen::closure::Closure::<dyn Fn(String)>::new(move |query: String| {
                let _ = editor_find2.try_update(|e| {
                    e.find(&query);
                });
                rt_find2.update(|t| *t += 1);
            });
        let find_fn: js_sys::Function = find_clos
            .as_ref()
            .unchecked_ref::<js_sys::Function>()
            .clone();
        find_clos.forget();

        // findNext / findPrevious
        let editor_fn3 = editor;
        let rt_fn3 = render_tick;
        let find_next_clos = wasm_bindgen::closure::Closure::<dyn Fn()>::new(move || {
            let _ = editor_fn3.try_update(|e| {
                e.find_next();
            });
            rt_fn3.update(|t| *t += 1);
        });
        let find_next_fn: js_sys::Function = find_next_clos
            .as_ref()
            .unchecked_ref::<js_sys::Function>()
            .clone();
        find_next_clos.forget();

        let editor_fn4 = editor;
        let rt_fn4 = render_tick;
        let find_prev_clos = wasm_bindgen::closure::Closure::<dyn Fn()>::new(move || {
            let _ = editor_fn4.try_update(|e| {
                e.find_previous();
            });
            rt_fn4.update(|t| *t += 1);
        });
        let find_prev_fn: js_sys::Function = find_prev_clos
            .as_ref()
            .unchecked_ref::<js_sys::Function>()
            .clone();
        find_prev_clos.forget();

        let obj = js_sys::Object::new();
        let _ = js_sys::Reflect::set(&obj, &"insertText".into(), &insert_fn);
        let _ = js_sys::Reflect::set(&obj, &"key".into(), &key_fn);
        let _ = js_sys::Reflect::set(&obj, &"content".into(), &content_fn);
        let _ = js_sys::Reflect::set(&obj, &"toggleFind".into(), &toggle_find_fn);
        let _ = js_sys::Reflect::set(&obj, &"find".into(), &find_fn);
        let _ = js_sys::Reflect::set(&obj, &"findNext".into(), &find_next_fn);
        let _ = js_sys::Reflect::set(&obj, &"findPrevious".into(), &find_prev_fn);
        let _ = js_sys::Reflect::set(&window, &"__tachyonEditor".into(), &obj);
    }

    let handle_click_on_line = move |line_idx: usize, col: usize| {
        editor.try_update(|e| {
            e.move_cursor_to(line_idx, col);
        });
    };

    let container_class = if line_numbers {
        "native-editor show-line-numbers".to_string()
    } else {
        "native-editor".to_string()
    };

    let wrap_class = if word_wrap {
        "line-content word-wrap".to_string()
    } else {
        "line-content".to_string()
    };

    let editor_for_wl_select = editor;
    let on_wikilink_select = Callback::new(move |completion: WikilinkCompletion| {
        editor_for_wl_select.update(|e| {
            e.insert_wikilink(&completion.title, None);
        });
        set_wl_visible.set(false);
    });

    // Dynamic visible_lines computation via leptos-use
    use_resize_observer(container_ref, move |entries, _observer| {
        if let Some(entry) = entries.first() {
            let rect = entry.content_rect();
            let h = rect.height();
            container_height.set(h);
            let computed = (h / LINE_HEIGHT_PX).max(10.0) as usize;
            visible_lines.set(computed);
        }
    });

    // Find/Replace actions (must be before view! macro)
    let editor_find = editor;
    let rt_find = render_tick;
    let do_find = move |query: String| {
        let _ = editor_find.try_update(|e| {
            e.find(&query);
        });
        rt_find.update(|t| *t += 1);
    };

    let editor_find_next = editor;
    let rt_find_next = render_tick;
    let on_find_next = move |_: ()| {
        let count = find_match_count.get();
        if count == 0 {
            return;
        }
        let _ = editor_find_next.try_update(|e| {
            e.find_next();
        });
        rt_find_next.update(|t| *t += 1);
    };

    let editor_find_prev = editor;
    let rt_find_prev = render_tick;
    let on_find_prev = move |_: ()| {
        let count = find_match_count.get();
        if count == 0 {
            return;
        }
        let _ = editor_find_prev.try_update(|e| {
            e.find_previous();
        });
        rt_find_prev.update(|t| *t += 1);
    };

    let editor_replace = editor;
    let rt_replace = render_tick;
    let on_replace_one = move |_: web_sys::MouseEvent| {
        let rep = find_replace.get();
        let q = find_query.get();
        let _ = editor_replace.try_update(|e| {
            if e.replace_next(&rep) {
                e.find(&q);
            }
        });
        rt_replace.update(|t| *t += 1);
    };

    let editor_replace_all = editor;
    let rt_replace_all = render_tick;
    let on_replace_all = move |_: web_sys::MouseEvent| {
        let rep = find_replace.get();
        let q = find_query.get();
        let _ = editor_replace_all.try_update(|e| {
            e.replace_all(&q, &rep);
            e.find(&q);
        });
        rt_replace_all.update(|t| *t += 1);
    };

    let on_close_find = move |_: ()| {
        set_find_visible.set(false);
    };

    let on_toggle_case = move |_: web_sys::MouseEvent| {
        set_find_case_sensitive.set(!find_case_sensitive.get());
    };

    let on_toggle_whole = move |_: web_sys::MouseEvent| {
        set_find_whole_word.set(!find_whole_word.get());
    };

    let on_toggle_replace = move |_: web_sys::MouseEvent| {
        set_find_show_replace.set(!find_show_replace.get());
    };

    // Effect: sync find UI with editor search state
    {
        let editor_sync = editor;
        Effect::new(move |_| {
            let (count, idx) =
                editor_sync.with_untracked(|e| (e.search_match_count(), e.search_current_index()));
            set_find_match_count.set(count);
            set_find_current_idx.set(idx);
        });
    }

    view! {
        // Editor container wrapper (fragment: editor + status bar)
        <div class="native-editor-wrapper h-full min-h-0">
        // Find/Replace panel (overlay at top)
        {move || if find_visible.get() {
            view! {
                <div class="find-replace-panel">
                    <div class="find-row">
                        <input
                            type="text"
                            class="find-input"
                            placeholder="Find..."
                            prop:value={find_query.get()}
                            on:input={move |ev: web_sys::Event| {
                                if let Some(input) = ev.target().and_then(|t| t.dyn_into::<web_sys::HtmlInputElement>().ok()) {
                                    let val = input.value();
                                    set_find_query.set(val.clone());
                                    do_find(val);
                                }
                            }}
                            on:keydown={move |ev: web_sys::KeyboardEvent| {
                                match ev.key().as_str() {
                                    "Enter" => {
                                        ev.prevent_default();
                                        if ev.shift_key() {
                                            on_find_prev(());
                                        } else {
                                            on_find_next(());
                                        }
                                    }
                                    "Escape" => {
                                        ev.prevent_default();
                                        on_close_find(());
                                    }
                                    _ => {}
                                }
                            }}
                        />
                        <span class="find-count">
                            {move || {
                                let count = find_match_count.get();
                                let idx = find_current_idx.get();
                                if count == 0 { "No results".to_string() } else { format!("{} of {}", idx + 1, count) }
                            }}
                        </span>
                        <button class="find-btn" title="Next" on:click=move |_| on_find_next(())>v</button>
                        <button class="find-btn" title="Previous" on:click=move |_| on_find_prev(())>^</button>
                        <button class={move || if find_case_sensitive.get() { "find-btn find-btn-active" } else { "find-btn" }} title="Match Case" on:click=on_toggle_case>Aa</button>
                        <button class={move || if find_whole_word.get() { "find-btn find-btn-active" } else { "find-btn" }} title="Whole Word" on:click=on_toggle_whole>W</button>
                        <button class="find-btn" title="Replace" on:click=on_toggle_replace>R</button>
                        <button class="find-btn" title="Close" on:click=move |_| on_close_find(())>x</button>
                    </div>
                    {move || if find_show_replace.get() {
                        view! {
                            <div class="replace-row">
                                <input
                                    type="text"
                                    class="find-input"
                                    placeholder="Replace..."
                                    prop:value={find_replace.get()}
                                    on:input={move |ev: web_sys::Event| {
                                        if let Some(input) = ev.target().and_then(|t| t.dyn_into::<web_sys::HtmlInputElement>().ok()) {
                                            set_find_replace.set(input.value());
                                        }
                                    }}
                                />
                                <button class="find-btn replace-btn" title="Replace" on:click=on_replace_one>Rep</button>
                                <button class="find-btn replace-btn" title="Replace All" on:click=on_replace_all>All</button>
                            </div>
                        }.into_any()
                    } else {
                        ().into_any()
                    }}
                </div>
            }.into_any()
        } else {
            ().into_any()
        }}

        <div
            class={move || {
                let mut cls = format!("{} {}", container_class, class_name);
                if is_mouse_dragging.get() { cls.push_str(" dragging"); }
                cls
            }}
            tabindex="0"
            role="textbox"
            aria-label="Document editor"
            aria-multiline="true"
            on:keydown={handle_keydown}
            on:scroll={handle_scroll}
            on:mousedown={handle_mousedown}
            on:mousemove={handle_mousemove}
            on:mouseup={handle_mouseup}
            style:font-size={font_size.clone()}
            style:cursor="text"
            prop:contenteditable="false"
            node_ref=container_ref
        >
            // Spacer to ensure scroll height matches content
            // Read editor line count without tracking (no reactive loop)
            <div class="editor-scroll-spacer" style:height={move || {
                let line_count = editor.with_untracked(|e| e.buffer().len_lines());
                format!("{}px", line_count as f64 * LINE_HEIGHT_PX + 200.0)
            }}></div>

            // Visible lines
            {
                let editor = editor;
                let wrap_class = wrap_class;
                let handle_click = handle_click_on_line;
                let placeholder = placeholder.clone();
                let line_numbers = line_numbers;

                move || {
                    // Track render_tick so edits trigger re-render
                    let _ = render_tick.get();
                    // Read all editor state WITHOUT tracking the editor signal.
                    // Tracking editor.with() here would create a reactive loop:
                    //   render → editor.update() → editor notifies → re-render → ...
                    // Use with_untracked/update_untracked so editor changes don't
                    // trigger re-renders — only scroll_offset and visible_lines do.
                    let (scroll_line, vis, total_lines, is_empty, line_texts, cursor_line, search_highlights, fold_starts) = {
                        let (sl, _) = scroll_offset.get_untracked();
                        let v = visible_lines.get_untracked();
                        let (tl, ie, txts, cl, sh, fs) = editor.with_untracked(|e| {
                            let total = e.buffer().len_lines();
                            let empty = e.content().trim().is_empty();
                            let end = (sl + v + 5).min(total);
                            let texts: Vec<String> = (sl..end).map(|i| {
                                e.buffer().line(i).trim_end_matches('\n').to_string()
                            }).collect();
                            let cl = e.cursors().active().0.line;
                            let sh = e.search_highlights();
                            let fs: std::collections::HashSet<usize> = e.fold_regions().iter().map(|r| r.start_line).collect();
                            (total, empty, texts, cl, sh, fs)
                        });
                        (sl, v, tl, ie, txts, cl, sh, fs)
                    };

                    let end = (scroll_line + vis + 5).min(total_lines);

                    // Syntax highlighting — uses update_untracked to avoid
                    // notifying reactive subscribers (which would loop).
                    let highlights_vec: Vec<Vec<HighlightSpan>> = editor.update_untracked(|e| {
                        (scroll_line..end).map(|i| e.highlight_line(i)).collect()
                    });

                    if is_empty {
                        return view! {
                            <div class="editor-placeholder">
                                {placeholder.clone()}
                            </div>
                        }.into_any();
                    }

                    let lines: Vec<usize> = (scroll_line..end).collect();

                    lines.into_iter().enumerate().map(|(vec_idx, line_idx)| {
                        let line_text = line_texts.get(vec_idx).cloned().unwrap_or_default();
                        let highlights = highlights_vec.get(vec_idx).cloned().unwrap_or_default();
                        let is_active = cursor_line == line_idx;
                        let line_num = line_idx + 1;
                        let h = handle_click;
                        let wc = wrap_class.clone();
                        let ln = line_numbers;
                        let gutter_w = if ln { 50.0f64 } else { 0.0f64 };

                        // Collect search highlights for this line
                        let line_search: Vec<(usize, usize, bool)> = search_highlights.iter()
                            .filter(|(l, _, _, _)| *l == line_idx)
                            .map(|(_, s, e, cur)| (*s, *e, *cur))
                            .collect();

                        view! {
                            <div class="editor-line" class:active-line={is_active}
                                 style:position="absolute"
                                 style:top={format!("{}px", line_idx as f64 * LINE_HEIGHT_PX)}
                                 style:left="0"
                                 style:right="0"
                                 style:height={format!("{}px", LINE_HEIGHT_PX)}
                            >
                                // Fold indicator
                                {if fold_starts.contains(&line_idx) {
                                    let editor_fold = editor;
                                    let fold_line = line_idx;
                                    let rt_fold = render_tick;
                                    view! {
                                        <div class="fold-indicator"
                                             style:position="absolute"
                                             style:left="2px"
                                             style:top="0"
                                             style:width="16px"
                                             style:height="100%"
                                             style:cursor="pointer"
                                             style:display="flex"
                                             style:align-items="center"
                                             style:justify-content="center"
                                             style:font-size="10px"
                                             style:color="#666"
                                             style:z-index="5"
                                             on:click=move |_: MouseEvent| {
                                                 editor_fold.try_update(|e| { e.toggle_fold(fold_line); });
                                                 rt_fold.update(|t| *t += 1);
                                             }
                                        >{"\u{25BC}"}</div>
                                    }.into_any()
                                } else {
                                    ().into_any()
                                }}
                                {if ln {
                                    view! {
                                        <div class="line-number">{line_num.to_string()}</div>
                                    }.into_any()
                                } else {
                                    ().into_any()
                                }}
                                // Indent guides: subtle vertical lines at each indent level
                                {
                                    let indent_count = line_text.chars()
                                        .take_while(|&c| c == ' ').count();
                                    let indent_levels = indent_count / 4; // Assuming tab_size=4
                                    (0..indent_levels).map(|level| {
                                        let left_px = if ln { 50.0 } else { 0.0 } + level as f64 * 4.0 * CHAR_WIDTH_PX;
                                        view! {
                                            <div class="editor-indent-guide"
                                                 style:position="absolute"
                                                 style:top="0"
                                                 style:left={format!("{}px", left_px)}
                                                 style:width="1px"
                                                 style:height="100%"
                                            ></div>
                                        }.into_any()
                                    }).collect::<Vec<_>>().into_any()
                                }
                                <div class={wc}
                                     on:click=move |ev: MouseEvent| {
                                         let x = ev.client_x() as f64;
                                         let col = {
                                             let gutter_w = if ln { 50.0 } else { 0.0 };
                                             ((x - gutter_w).max(0.0) / CHAR_WIDTH_PX) as usize
                                         };
                                         h(line_idx, col);
                                     }
                                >
                                    {highlights.iter().map(|span| {
                                        let start_c = span.start_col.min(line_text.len());
                                        let end_c = span.end_col.min(line_text.len());
                                        if start_c >= end_c {
                                            return view! { <span></span> }.into_any();
                                        }
                                        let text = line_text[start_c..end_c].to_string();
                                        let class = css_class(&span.token);
                                        view! {
                                            <span class={class}>{text}</span>
                                        }.into_any()
                                    }).collect::<Vec<_>>()}
                                    // Ensure line ends with a newline-like space for click target
                                    <span class="ed-whitespace">{"\u{00a0}"}</span>
                                </div>
                                // Search match highlight overlays
                                {line_search.into_iter().map(|(start_col, end_col, is_current)| {
                                    let left_px = gutter_w + start_col as f64 * CHAR_WIDTH_PX;
                                    let width_px = (end_col - start_col) as f64 * CHAR_WIDTH_PX;
                                    let bg_class = if is_current { "search-highlight-current" } else { "search-highlight" };
                                    view! {
                                        <div class=bg_class
                                             style:position="absolute"
                                             style:top="2px"
                                             style:left={format!("{}px", left_px)}
                                             style:width={format!("{}px", width_px)}
                                             style:height={format!("{}px", LINE_HEIGHT_PX - 4.0)}
                                             style:pointer-events="none"
                                        ></div>
                                    }.into_any()
                                }).collect::<Vec<_>>()}
                            </div>
                        }.into_any()
                    }).collect::<Vec<_>>().into_any()
                }
            }

            // Cursor overlay
            {
                let editor = editor;
                let scroll_off = scroll_offset;
                let ln = line_numbers;

                move || {
                    let (cursor_line, cursor_col) = editor.try_with(|e| {
                        let c = e.cursors().active().0;
                        (c.line, c.col)
                    }).unwrap_or((0, 0));
                    let (_scroll_line, _) = scroll_off.get();
                    let gutter_w = if ln { 50.0 } else { 0.0 };
                    let top_px = cursor_line as f64 * LINE_HEIGHT_PX;
                    let left_px = gutter_w + cursor_col as f64 * CHAR_WIDTH_PX;

                    view! {
                        <div class="editor-cursor"
                             style:position="absolute"
                             style:top={format!("{}px", top_px)}
                             style:left={format!("{}px", left_px)}
                        ></div>
                    }.into_any()
                }
            }

            // Selection overlay
            {
                let editor = editor;
                let ln = line_numbers;

                move || {
                    let (has_selection, sel_start, sel_end) = editor.try_with(|e| {
                        let sel = e.cursors().active().1;
                        if sel.is_empty() {
                            (false, Cursor::zero(), Cursor::zero())
                        } else {
                            let (s, e) = sel.normalize();
                            (true, s, e)
                        }
                    }).unwrap_or((false, Cursor::zero(), Cursor::zero()));

                    if !has_selection {
                        return ().into_any();
                    }

                    let gutter_w = if ln { 50.0 } else { 0.0 };
                    let top_px = sel_start.line as f64 * LINE_HEIGHT_PX;
                    let left_px = gutter_w + sel_start.col as f64 * CHAR_WIDTH_PX;
                    let sel_lines = sel_end.line - sel_start.line;
                    let _height_px = if sel_lines > 0 {
                        (sel_lines + 1) as f64 * LINE_HEIGHT_PX
                    } else {
                        LINE_HEIGHT_PX
                    };

                    // Multi-line selection: full-width background + start/end specific width
                    let rects: Vec<(f64, f64, f64, f64)> = if sel_lines == 0 {
                        // Single line: one rect from sel_start.col to sel_end.col
                        let w = (sel_end.col - sel_start.col) as f64 * CHAR_WIDTH_PX;
                        vec![(top_px, left_px, w, LINE_HEIGHT_PX)]
                    } else {
                        let mut rects = Vec::new();
                        // First line: from sel_start.col to right edge (use 9999 as "infinity")
                        let first_h = LINE_HEIGHT_PX;
                        rects.push((top_px, left_px, 9999.0, first_h));
                        // Middle lines: full width
                        for i in 1..sel_lines {
                            let y = (sel_start.line + i) as f64 * LINE_HEIGHT_PX;
                            rects.push((y, gutter_w, 9999.0, LINE_HEIGHT_PX));
                        }
                        // Last line: from gutter to sel_end.col
                        let last_y = sel_end.line as f64 * LINE_HEIGHT_PX;
                        let last_w = sel_end.col as f64 * CHAR_WIDTH_PX;
                        rects.push((last_y, gutter_w, last_w, LINE_HEIGHT_PX));
                        rects
                    };

                    rects.into_iter().map(|(y, x, w, h)| {
                        view! {
                            <div class="editor-selection"
                                 style:position="absolute"
                                 style:top={format!("{}px", y)}
                                 style:left={format!("{}px", x)}
                                 style:width={format!("{}px", w)}
                                 style:height={format!("{}px", h)}
                            ></div>
                        }.into_any()
                    }).collect::<Vec<_>>().into_any()
                }
            }

            // Bracket matching highlight overlay
            {
                let editor_bracket = editor;
                let ln_b = line_numbers;
                move || {
                    let bracket_info = editor_bracket.try_with(|e| {
                        let cursor = e.cursors().active().0;
                        let check_col = if cursor.col > 0 { cursor.col - 1 } else { return None; };
                        e.find_matching_bracket(cursor.line, check_col)
                            .map(|(ml, mc)| (cursor.line, check_col, ml, mc))
                    }).flatten();

                    if let Some((al, ac, bl, bc)) = bracket_info {
                        let gutter_w = if ln_b { 50.0 } else { 0.0 };
                        view! {
                            <div class="editor-bracket-match"
                                 style:position="absolute"
                                 style:top={format!("{}px", al as f64 * LINE_HEIGHT_PX)}
                                 style:left={format!("{}px", gutter_w + ac as f64 * CHAR_WIDTH_PX)}
                                 style:width={format!("{}px", CHAR_WIDTH_PX)}
                                 style:height={format!("{}px", LINE_HEIGHT_PX)}
                            ></div>
                            <div class="editor-bracket-match"
                                 style:position="absolute"
                                 style:top={format!("{}px", bl as f64 * LINE_HEIGHT_PX)}
                                 style:left={format!("{}px", gutter_w + bc as f64 * CHAR_WIDTH_PX)}
                                 style:width={format!("{}px", CHAR_WIDTH_PX)}
                                 style:height={format!("{}px", LINE_HEIGHT_PX)}
                            ></div>
                        }.into_any()
                    } else {
                        ().into_any()
                    }
                }
            }

            // Collaborative cursors overlay
            {
                let aw = awareness;
                let ln = line_numbers;
                move || {
                    let Some(aw_signal) = aw else {
                        return ().into_any();
                    };
                    view! {
                        <CollaborativeCursors awareness={aw_signal} line_numbers={ln} />
                    }.into_any()
                }
            }

            // Wikilink autocomplete dropdown
            <WikilinkAutocomplete
                query=wl_query
                visible=wl_visible
                position=wl_position
                on_select=on_wikilink_select
            />
        </div>
        // Status bar: line:col, selection info, language
        {
            let editor_sb = editor;
            let _ln_sb = line_numbers;
            move || {
                let (cursor_line, cursor_col) = editor_sb.try_with(|e| {
                    let c = e.cursors().active().0;
                    (c.line + 1, c.col + 1) // 1-indexed for display
                }).unwrap_or((1, 1));
                let sel_info = editor_sb.try_with(|e| {
                    let sel = e.cursors().active().1;
                    if sel.is_empty() {
                        String::new()
                    } else {
                        let (s, en) = sel.normalize();
                        let chars = if s.line == en.line {
                            (en.col - s.col).to_string()
                        } else {
                            format!("{}:{}-{}:{}", s.line+1, s.col+1, en.line+1, en.col+1)
                        };
                        format!(" ({})", chars)
                    }
                }).unwrap_or_default();
                let (total_lines, lang_name) = editor_sb.with_untracked(|e| {
                    let tl = e.buffer().len_lines();
                    let ln = match e.language() {
                        tachyon_editor::Language::Markdown => "Markdown",
                        tachyon_editor::Language::Rust => "Rust",
                        tachyon_editor::Language::JavaScript => "JavaScript",
                        tachyon_editor::Language::TypeScript => "TypeScript",
                        tachyon_editor::Language::Python => "Python",
                        tachyon_editor::Language::Html => "HTML",
                        tachyon_editor::Language::Css => "CSS",
                        tachyon_editor::Language::Json => "JSON",
                        tachyon_editor::Language::Yaml => "YAML",
                        tachyon_editor::Language::Bash => "Shell",
                        tachyon_editor::Language::PlainText => "Plain Text",
                        tachyon_editor::Language::Unknown => "Unknown",
                        _ => "Text",
                    };
                    (tl, ln)
                });
                view! {
                    <div class="editor-status-bar">
                        <span class="status-left">
                            {"Ln "}{cursor_line}{", Col "}{cursor_col}{sel_info}
                        </span>
                        <span class="status-right">
                            {lang_name}{" — "}{total_lines}{" lines"}
                        </span>
                    </div>
                }.into_any()
            }
        }
        </div>
    }
}

pub fn insert_markdown_syntax(
    editor: RwSignal<Editor>,
    prefix: &str,
    suffix: &str,
    default_text: &str,
) {
    editor.try_update(|e| {
        if !e.cursors().active().1.is_empty() {
            let selected = {
                let sel = e.cursors().active().1;
                let (start, end) = sel.normalize();
                let line = e.buffer().line(start.line);
                let trimmed = line.trim_end_matches('\n');
                if start.line == end.line {
                    trimmed[start.col.min(trimmed.len())..end.col.min(trimmed.len())].to_string()
                } else {
                    String::new()
                }
            };
            e.insert_text(&format!("{}{}{}", prefix, selected, suffix));
        } else {
            e.insert_text(&format!("{}{}{}", prefix, default_text, suffix));
            let cursor = e.cursors().active().0;
            let _offset = prefix.len();
            e.move_cursor_to(
                cursor.line,
                cursor.col.saturating_sub(default_text.len() + suffix.len()),
            );
        }
    });
}

pub fn insert_line_prefix(editor: RwSignal<Editor>, prefix: &str) {
    editor.try_update(|e| {
        let line = e.cursors().active().0.line;
        let line_text = e.current_line_text();
        let already = line_text.trim_start().starts_with(prefix.trim());
        if already {
            let remove_len = prefix.len();
            e.buffer_mut().delete_range(line, 0, line, remove_len);
            e.move_cursor_to(line, e.cursors().active().0.col.saturating_sub(remove_len));
        } else {
            e.buffer_mut().insert(line, 0, prefix);
            e.move_cursor_to(line, e.cursors().active().0.col + prefix.len());
        }
    });
}
