use crate::components::collaborative_cursors::{AwarenessState, CollaborativeCursors};
use leptos::prelude::*;
use tachyon_editor::{Cursor, Editor, HighlightToken};
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
    }
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
) -> impl IntoView {
    let editor = match editor {
        Some(sig) => sig,
        None => RwSignal::new(Editor::with_content(&content)),
    };
    let scroll_offset = RwSignal::new((0usize, 0usize));
    let visible_lines = RwSignal::new(50usize);
    let is_focused = RwSignal::new(false);

    let editor_for_change = editor;
    let on_change_cb = on_change;

    Effect::new(move |_| {
        let content = editor_for_change.with(|e| e.content());
        let dirty = editor_for_change.with(|e| e.is_dirty());
        if dirty {
            on_change_cb.run(content);
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

    let handle_keydown =
        move |ev: KeyboardEvent| {
            let key = ev.key();
            let ctrl = ev.ctrl_key() || ev.meta_key();
            let shift = ev.shift_key();

            let should_prevent = !ctrl
                || matches!(
                    key.as_str(),
                    "z" | "Z" | "a" | "A" | "f" | "F" | "d" | "D" | "s" | "S"
                );

            if should_prevent {
                ev.prevent_default();
            }

            let _handled =
                {
                    editor.update(|e| {
                match (ctrl, shift, key.as_str()) {
                    (true, false, "z") => { e.undo(); }
                    (true, true, "z") | (true, true, "Z") => { e.redo(); }
                    (true, false, "a") | (true, false, "A") => { e.select_all(); }
                    (true, false, "s") | (true, false, "S") => { /* save handled externally */ }
                    (true, false, "d") | (true, false, "D") => { e.delete_line(); }
                    (false, false, "Enter") => { e.auto_indent_newline(); }
                    (false, false, "Backspace") => { e.delete_backwards(); }
                    (false, false, "Delete") => { e.delete_forwards(); }
                    (false, false, "Tab") => { e.indent_selection(); }
                    (true, false, "Tab") => { e.unindent_selection(); }
                    (false, false, "ArrowLeft") => {
                        if shift {
                            let cur = e.cursor().clone();
                            let mut target = cur;
                            target.move_left(e.buffer());
                            e.extend_selection_to(target);
                        } else {
                            e.move_cursor_left();
                        }
                    }
                    (false, false, "ArrowRight") => {
                        if shift {
                            let cur = e.cursor().clone();
                            let mut target = cur;
                            target.move_right(e.buffer());
                            e.extend_selection_to(target);
                        } else {
                            e.move_cursor_right();
                        }
                    }
                    (false, false, "ArrowUp") => {
                        if shift {
                            let cur = e.cursor().clone();
                            let mut target = cur;
                            target.move_up(e.buffer());
                            e.extend_selection_to(target);
                        } else {
                            e.move_cursor_up();
                        }
                    }
                    (false, false, "ArrowDown") => {
                        if shift {
                            let cur = e.cursor().clone();
                            let mut target = cur;
                            target.move_down(e.buffer());
                            e.extend_selection_to(target);
                        } else {
                            e.move_cursor_down();
                        }
                    }
                    (false, false, "Home") => { e.move_cursor_home(); }
                    (false, false, "End") => { e.move_cursor_end(); }
                    (true, false, "Backspace") => { e.delete_word_backwards(); }
                    (true, false, "Delete") => { e.delete_word_forwards(); }
                    (true, false, "ArrowLeft") | (true, false, "ArrowUp") => {
                        e.move_cursor_to(0, 0);
                    }
                    (true, false, "ArrowRight") | (true, false, "ArrowDown") => {
                        let last_line = e.buffer().len_lines().saturating_sub(1);
                        let last_col = e.buffer().line_len(last_line);
                        e.move_cursor_to(last_line, last_col);
                    }
                    (true, _shift, "ArrowLeft") | (true, _shift, "ArrowUp") => {
                        e.extend_selection_to(Cursor::zero());
                    }
                    (true, _shift, "ArrowRight") | (true, _shift, "ArrowDown") => {
                        let last_line = e.buffer().len_lines().saturating_sub(1);
                        let last_col = e.buffer().line_len(last_line);
                        e.extend_selection_to(Cursor::new(last_line, last_col));
                    }
                    (true, false, "l") => { e.select_line(); }
                    (true, false, "w") => { e.select_word(); }
                    (true, _shift, "k") => { e.delete_line(); }
                    (true, false, "/") => { /* search */ }
                    (false, false, "a" | "b" | "c" | "e" | "f" | "g" | "h" | "i" | "j" | "k" | "l"
                        | "m" | "n" | "o" | "p" | "q" | "r" | "s" | "t" | "u" | "v" | "w" | "x"
                        | "y" | "z" | "A" | "B" | "C" | "D" | "E" | "F" | "G" | "H" | "I" | "J"
                        | "K" | "L" | "M" | "N" | "O" | "P" | "Q" | "R" | "S" | "T" | "U" | "V"
                        | "W" | "X" | "Y" | "Z"
                        | "0" | "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9"
                        | " " | "!" | "@" | "#" | "$" | "%" | "^" | "&" | "*" | "(" | ")"
                        | "-" | "_" | "=" | "+" | "[" | "]" | "{" | "}" | "|" | "\\" | ";"
                        | ":" | "'" | "\"" | "," | "<" | "." | ">" | "/" | "?" | "`" | "~"
                    ) => {
                        e.insert_text(&key);
                    }
                    _ => {}
                }
            });
                    true
                };
        };

    let handle_mousedown = move |ev: MouseEvent| {
        if let Some(_target) = ev.target() {
            is_focused.set(true);
        }
    };

    let handle_click_on_line = move |line_idx: usize, col: usize| {
        editor.update(|e| {
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

    view! {
        <div
            class={format!("{} {}", container_class, class_name)}
            tabindex="0"
            on:keydown={handle_keydown}
            on:scroll={handle_scroll}
            on:mousedown={handle_mousedown}
            style:font-size={font_size.clone()}
            style:outline={move || if is_focused.get() { "none" } else { "none" }}
            style:cursor="text"
            prop:contenteditable="false"
        >
            // Spacer to ensure scroll height matches content
            <div class="editor-scroll-spacer" style:height={move || {
                let line_count = editor.with(|e| e.buffer().len_lines());
                format!("{}px", line_count as f64 * LINE_HEIGHT_PX)
            }}></div>

            // Visible lines
            {
                let editor = editor;
                let wrap_class = wrap_class;
                let handle_click = handle_click_on_line;
                let placeholder = placeholder.clone();
                let line_numbers = line_numbers;

                move || {
                    let total_lines = editor.with(|e| e.buffer().len_lines());
                    let (scroll_line, _) = scroll_offset.get();
                    let vis = visible_lines.get();
                    let start = scroll_line;
                    let end = (start + vis + 5).min(total_lines);
                    let is_empty = editor.with(|e| e.content().trim().is_empty());

                    if is_empty {
                        return view! {
                            <div class="editor-placeholder">
                                {placeholder.clone()}
                            </div>
                        }.into_any();
                    }

                    let lines: Vec<usize> = (start..end).collect();

                    lines.into_iter().map(|line_idx| {
                        let line_text = editor.with(|e| {
                            e.buffer().line(line_idx).trim_end_matches('\n').to_string()
                        });
                        let highlights = {
                            let line_idx = line_idx;
                            let mut spans = Vec::new();
                            editor.update(|e| {
                                spans = e.highlight_line(line_idx);
                            });
                            spans
                        };
                        let cursor_line = editor.with(|e| e.cursor().line);
                        let is_active = cursor_line == line_idx;
                        let line_num = line_idx + 1;
                        let h = handle_click.clone();
                        let wc = wrap_class.clone();
                        let ln = line_numbers;

                        view! {
                            <div class="editor-line" class:active-line={is_active}
                                 style:position="absolute"
                                 style:top={format!("{}px", line_idx as f64 * LINE_HEIGHT_PX)}
                                 style:left="0"
                                 style:right="0"
                                 style:height={format!("{}px", LINE_HEIGHT_PX)}
                            >
                                {if ln {
                                    view! {
                                        <div class="line-number">{line_num.to_string()}</div>
                                    }.into_any()
                                } else {
                                    ().into_any()
                                }}
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
                    let (cursor_line, cursor_col) = editor.with(|e| {
                        (e.cursor().line, e.cursor().col)
                    });
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
                    let (has_selection, sel_start, sel_end) = editor.with(|e| {
                        let sel = e.selection();
                        if sel.is_empty() {
                            (false, Cursor::zero(), Cursor::zero())
                        } else {
                            let (s, e) = sel.normalize();
                            (true, s, e)
                        }
                    });

                    if !has_selection {
                        return ().into_any();
                    }

                    let gutter_w = if ln { 50.0 } else { 0.0 };
                    let top_px = sel_start.line as f64 * LINE_HEIGHT_PX;
                    let left_px = gutter_w + sel_start.col as f64 * CHAR_WIDTH_PX;
                    let height_px = if sel_end.line > sel_start.line {
                        (sel_end.line - sel_start.line + 1) as f64 * LINE_HEIGHT_PX
                    } else {
                        LINE_HEIGHT_PX
                    };
                    let width_px = if sel_end.line == sel_start.line {
                        (sel_end.col - sel_start.col) as f64 * CHAR_WIDTH_PX
                    } else {
                        0.0
                    };

                    view! {
                        <div class="editor-selection"
                             style:position="absolute"
                             style:top={format!("{}px", top_px)}
                             style:left={format!("{}px", left_px)}
                             style:width={format!("{}px", width_px)}
                             style:height={format!("{}px", height_px)}
                        ></div>
                    }.into_any()
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
        </div>
    }
}

pub fn insert_markdown_syntax(
    editor: RwSignal<Editor>,
    prefix: &str,
    suffix: &str,
    default_text: &str,
) {
    editor.update(|e| {
        if !e.selection().is_empty() {
            let selected = {
                let (start, end) = e.selection().normalize();
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
            let cursor = e.cursor();
            let _offset = prefix.len();
            e.move_cursor_to(
                cursor.line,
                cursor.col.saturating_sub(default_text.len() + suffix.len()),
            );
        }
    });
}

pub fn insert_line_prefix(editor: RwSignal<Editor>, prefix: &str) {
    editor.update(|e| {
        let line = e.cursor().line;
        let line_text = e.current_line_text();
        let already = line_text.trim_start().starts_with(prefix.trim());
        if already {
            let remove_len = prefix.len();
            e.buffer_mut().delete_range(line, 0, line, remove_len);
            e.move_cursor_to(line, e.cursor().col.saturating_sub(remove_len));
        } else {
            e.buffer_mut().insert(line, 0, prefix);
            e.move_cursor_to(line, e.cursor().col + prefix.len());
        }
    });
}
