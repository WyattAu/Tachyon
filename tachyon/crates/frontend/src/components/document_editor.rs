// Document Editor Component
// Real-time collaborative document editor with markdown preview,
// formatting toolbar, keyboard shortcuts, auto-save, and file export.

#![allow(dead_code)]
#![allow(unused_imports)]

use crate::api::ApiClient;
use crate::types::Document;
use crate::websocket::{
    ConnectionState, DocumentEditMessage, EditOperation, WebSocketClient, WsMessage,
};
use leptos::ev::Event;
use leptos::prelude::*;
use wasm_bindgen_futures::spawn_local;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use uuid::Uuid;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use web_sys::HtmlTextAreaElement;

// ============================================================================
// Types
// ============================================================================

#[derive(Clone, Debug)]
pub struct PresenceUser {
    pub user_id: String,
    pub user_name: String,
    pub color: String,
}

#[derive(Clone, Default)]
pub struct EditorState {
    pub content: String,
    pub version: u64,
    pub is_saving: bool,
    pub last_saved: Option<String>,
    pub presence_users: Vec<PresenceUser>,
    pub preview_html: String,
    pub word_count: usize,
    pub character_count: usize,
    pub show_preview: bool,
    pub render_error: Option<String>,
    pub auto_save_enabled: bool,
    pub dirty: bool,
}

pub fn get_user_color(user_id: &str) -> String {
    let colors = [
        "#3B82F6", "#10B981", "#F59E0B", "#EF4444", "#8B5CF6", "#EC4899", "#06B6D4", "#84CC16",
        "#F97316", "#6366F1",
    ];
    let hash = user_id.chars().map(|c| c as usize).sum::<usize>();
    colors[hash % colors.len()].to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PresenceUserInfo {
    user_id: String,
    user_name: String,
}

// ============================================================================
// Editor Component
// ============================================================================

#[component]
pub fn DocumentEditor(
    document_id: String,
    user_id: String,
    user_name: String,
    #[prop(optional)] initial_content: String,
) -> impl IntoView {
    let api_client = ApiClient::default();
    let ws_client = WebSocketClient::new("");

    let (editor_state, set_editor_state) = signal(EditorState {
        content: initial_content.clone(),
        preview_html: String::new(),
        word_count: 0,
        character_count: 0,
        show_preview: true,
        render_error: None,
        version: 0,
        is_saving: false,
        last_saved: None,
        presence_users: Vec::new(),
        auto_save_enabled: true,
        dirty: false,
    });

    let (connection_state, set_connection_state) = signal(ConnectionState::Disconnected);
    let (local_edit_version, set_local_edit_version) = signal(0u64);

    let debounce_handle: Rc<RefCell<Option<i32>>> = Rc::new(RefCell::new(None));
    let render_debounce: Rc<RefCell<Option<i32>>> = Rc::new(RefCell::new(None));
    let auto_save_debounce: Rc<RefCell<Option<i32>>> = Rc::new(RefCell::new(None));
    let document_content = RwSignal::new(initial_content);

    // Pre-clone for move closures that need ownership
    // api_client is moved into trigger_render, so clone for later closures
    let api_client_for_hi = api_client.clone();
    let api_client_for_save = api_client.clone();
    let api_client_for_save_ks = api_client.clone();

    // --- Markdown rendering via API ---
    let trigger_render = move |content: String| {
        let render_dh = render_debounce.clone();
        let api = api_client.clone();
        let set_es = set_editor_state.clone();

        {
            let handle = render_dh.borrow().clone();
            if let Some(h) = handle {
                let _ = web_sys::window().map(|w| {
                    let _ = w.clear_timeout_with_handle(h);
                });
            }
        }

        let closure = Closure::<dyn Fn()>::new(move || {
            let api = api.clone();
            let set_es = set_es.clone();
            let content = content.clone();

            spawn_local(async move {
                match api.render_markdown(&content).await {
                    Ok(response) => {
                        set_es.update(|s| {
                            s.preview_html = response.html;
                            s.word_count = response.word_count;
                            s.character_count = response.character_count;
                            s.render_error = None;
                        });
                    }
                    Err(e) => {
                        set_es.update(|s| {
                            s.render_error = Some(format!("Render error: {}", e));
                        });
                    }
                }
            });
        });

        let timeout = web_sys::window()
            .and_then(|w| {
                w.set_timeout_with_callback_and_timeout_and_arguments_0(
                    closure.as_ref().unchecked_ref(),
                    400,
                )
                .ok()
            })
            .unwrap_or(0);

        *render_dh.borrow_mut() = Some(timeout);
        closure.forget();
    };

    // --- Initial render on mount ---
    {
        let initial = document_content.get_untracked();
        if !initial.is_empty() {
            trigger_render(initial);
        }
    }

    // --- WebSocket collaborative editing ---
    let ws_clone = ws_client.clone();
    let user_id_for_effect = user_id.clone();
    let doc_id_for_effect = document_id.clone();
    let user_name_for_effect = user_name.clone();

    Effect::new(move || {
        let ws = ws_clone.clone();
        let uid_for_join = user_id_for_effect.clone();
        let uid_for_msg = user_id_for_effect.clone();
        let did = doc_id_for_effect.clone();
        let uname = user_name_for_effect.clone();
        let doc_content = document_content;
        let set_es = set_editor_state.clone();

        ws.on_message(Rc::new(move |msg: WsMessage| {
            match msg.message_type.as_str() {
                "edit" => {
                    if msg.user_id.as_deref() != Some(&uid_for_msg) {
                        if let Some(data) = &msg.data {
                            if let Ok(edit_msg) =
                                serde_json::from_value::<DocumentEditMessage>(data.clone())
                            {
                                let op = edit_msg.operation;
                                let new_content = apply_operation(&doc_content.get(), &op);
                                doc_content.set(new_content);
                                set_es.update(|s| {
                                    s.version = edit_msg.version;
                                });
                            }
                        }
                    }
                }
                "join" => {
                    if let (Some(join_user_id), Some(data)) = (&msg.user_id, &msg.data) {
                        if let Some(join_user_name) = data.get("user_name").and_then(|n| n.as_str())
                        {
                            if join_user_id != &uid_for_msg {
                                set_es.update(|s| {
                                    if !s.presence_users.iter().any(|u| u.user_id == *join_user_id)
                                    {
                                        s.presence_users.push(PresenceUser {
                                            user_id: join_user_id.clone(),
                                            user_name: join_user_name.to_string(),
                                            color: get_user_color(join_user_id),
                                        });
                                    }
                                });
                            }
                        }
                    }
                }
                "leave" => {
                    if let Some(leave_user_id) = &msg.user_id {
                        set_es.update(|s| {
                            s.presence_users.retain(|u| &u.user_id != leave_user_id);
                        });
                    }
                }
                "presence" => {
                    if let Some(data) = &msg.data {
                        if let Ok(users) =
                            serde_json::from_value::<Vec<PresenceUserInfo>>(data.clone())
                        {
                            let uid_for_filter = uid_for_msg.clone();
                            set_es.update(|s| {
                                s.presence_users = users
                                    .into_iter()
                                    .filter(|u| u.user_id != uid_for_filter)
                                    .map(|u| {
                                        let color = get_user_color(&u.user_id);
                                        PresenceUser {
                                            user_id: u.user_id,
                                            user_name: u.user_name,
                                            color,
                                        }
                                    })
                                    .collect();
                            });
                        }
                    }
                }
                _ => {}
            }
        }));

        ws.on_state_change(Rc::new(move |state| {
            set_connection_state.set(state);
        }));

        ws.connect();

        let _ = ws.join_document(&did, &uid_for_join, &uname);
    });

    Effect::new(move || {
        let content = document_content.get();
        set_editor_state.update(|s| {
            s.content = content;
        });
    });

    // --- Input handler ---
    let debounce_handle_clone = debounce_handle.clone();
    let auto_save_dh_clone = auto_save_debounce.clone();
    let trigger_render_for_input = trigger_render;

    // Pre-clone for closures that capture after handle_input moves these in
    let document_id_for_save = document_id.clone();
    let document_id_for_save_ks = document_id.clone();
    let auto_save_debounce_for_save = auto_save_debounce.clone();
    let auto_save_debounce_for_save_ks = auto_save_debounce.clone();

    let handle_input = move |ev: Event| {
        let target: HtmlTextAreaElement = ev.target().unwrap().unchecked_into();
        let new_content = target.value();
        let old_content = document_content.get();

        let operation = compute_diff(&old_content, &new_content);
        document_content.set(new_content.clone());

        // Mark as dirty and trigger auto-save
        set_editor_state.update(|s| {
            s.dirty = true;
        });

        // Debounce auto-save (3 seconds)
        {
            let asdh = auto_save_dh_clone.borrow().clone();
            if let Some(h) = asdh {
                let _ = web_sys::window().map(|w| {
                    let _ = w.clear_timeout_with_handle(h);
                });
            }
        }

        let api_as = api_client_for_hi.clone();
        let doc_id_as = document_id.clone();
        let set_es_as = set_editor_state.clone();
        let asdh2 = auto_save_debounce.clone();

        let auto_save_closure = Closure::<dyn Fn()>::new(move || {
            let api = api_as.clone();
            let doc_id = doc_id_as.clone();
            let content = document_content.get_untracked();
            let set_es = set_es_as.clone();

            spawn_local(async move {
                let body = serde_json::json!({ "content": content });
                match api.update_document(&doc_id, &body).await {
                    Ok(_) => {
                        let now = chrono::Utc::now().format("%H:%M:%S").to_string();
                        set_es.update(|s| {
                            s.is_saving = false;
                            s.last_saved = Some(format!("Auto-saved {}", now));
                            s.dirty = false;
                        });
                    }
                    Err(_) => {
                        set_es.update(|s| {
                            s.is_saving = false;
                        });
                    }
                }
            });
        });

        let auto_timeout = web_sys::window()
            .and_then(|w| {
                w.set_timeout_with_callback_and_timeout_and_arguments_0(
                    auto_save_closure.as_ref().unchecked_ref(),
                    3000,
                )
                .ok()
            })
            .unwrap_or(0);

        *asdh2.borrow_mut() = Some(auto_timeout);
        auto_save_closure.forget();

        // Debounce WebSocket broadcast
        {
            let handle = debounce_handle_clone.borrow().clone();
            if let Some(h) = handle {
                let _ = web_sys::window().map(|w| {
                    let _ = w.clear_timeout_with_handle(h);
                });
            }
        }

        let ws = ws_client.clone();
        let doc_id = document_id.clone();
        let uid = user_id.clone();
        let op = operation.clone();
        let version = local_edit_version.get();
        let set_lev = set_local_edit_version.clone();
        let set_es = set_editor_state.clone();
        let dh = debounce_handle_clone.clone();

        let closure = Closure::<dyn Fn()>::new(move || {
            let op_id = Uuid::new_v4().to_string();
            let edit_msg = DocumentEditMessage {
                operation_id: op_id,
                operation: op.clone(),
                version,
            };
            let _ = ws.send_edit(
                &doc_id,
                &uid,
                serde_json::to_value(&edit_msg).unwrap_or(serde_json::Value::Null),
            );
            set_lev.update(|v| *v += 1);
            set_es.update(|s| {
                s.is_saving = true;
            });
        });

        let timeout = web_sys::window()
            .and_then(|w| {
                w.set_timeout_with_callback_and_timeout_and_arguments_0(
                    closure.as_ref().unchecked_ref(),
                    300,
                )
                .ok()
            })
            .unwrap_or(0);

        *dh.borrow_mut() = Some(timeout);
        closure.forget();

        // Trigger markdown preview render
        trigger_render_for_input(new_content);
    };

    // --- Manual save ---
    let save_document = move |_: leptos::ev::MouseEvent| {
        let api = api_client_for_save.clone();
        let doc_id = document_id_for_save.clone();
        let content = document_content.get();
        let set_es = set_editor_state.clone();

        // Clear any pending auto-save
        {
            let handle = auto_save_debounce_for_save.borrow().clone();
            if let Some(h) = handle {
                let _ = web_sys::window().map(|w| {
                    let _ = w.clear_timeout_with_handle(h);
                });
            }
        }

        spawn_local(async move {
            set_es.update(|s| s.is_saving = true);
            let body = serde_json::json!({
                "content": content,
            });
            match api.update_document(&doc_id, &body).await {
                Ok(_) => {
                    let now = chrono::Utc::now().format("%H:%M:%S").to_string();
                    set_es.update(|s| {
                        s.is_saving = false;
                        s.last_saved = Some(now);
                        s.dirty = false;
                    });
                }
                Err(e) => {
                    set_es.update(|s| {
                        s.is_saving = false;
                    });
                    web_sys::console::error_1(&format!("Save failed: {}", e).into());
                }
            }
        });
    };

    // --- Ctrl+S keyboard shortcut ---
    let save_document_ks = move || {
        let api = api_client_for_save_ks.clone();
        let doc_id = document_id_for_save_ks.clone();
        let content = document_content.get_untracked();
        let set_es = set_editor_state.clone();

        {
            let handle = auto_save_debounce_for_save_ks.borrow().clone();
            if let Some(h) = handle {
                let _ = web_sys::window().map(|w| {
                    let _ = w.clear_timeout_with_handle(h);
                });
            }
        }

        spawn_local(async move {
            set_es.update(|s| s.is_saving = true);
            let body = serde_json::json!({ "content": content });
            match api.update_document(&doc_id, &body).await {
                Ok(_) => {
                    let now = chrono::Utc::now().format("%H:%M:%S").to_string();
                    set_es.update(|s| {
                        s.is_saving = false;
                        s.last_saved = Some(now);
                        s.dirty = false;
                    });
                }
                Err(_) => {
                    set_es.update(|s| s.is_saving = false);
                }
            }
        });
    };

    // --- Formatting toolbar actions ---
    let wrap_selection = move |prefix: &str, suffix: &str| {
        if let Some(window) = web_sys::window() {
            if let Some(document) = window.document() {
                if let Some(textarea) = document.query_selector("textarea.editor-textarea")
                    .ok()
                    .flatten()
                {
                    let ta: HtmlTextAreaElement = textarea.unchecked_into();
                    let start = ta.selection_start().unwrap_or(Some(0)).unwrap_or(0);
                    let end = ta.selection_end().unwrap_or(Some(0)).unwrap_or(0);
                    let start = start as usize;
                    let end = end as usize;
                    let value = ta.value();
                    let (before, selected, after) = (
                        value[..start].to_string(),
                        value[start..end].to_string(),
                        value[end..].to_string(),
                    );

                    // If nothing selected, provide a default placeholder
                    let placeholder = if selected.is_empty() { "text" } else { "" };
                    let new_value = format!("{}{}{}{}{}", before, prefix, selected, suffix, after);

                    ta.set_value(&new_value);
                    let _ = ta.focus();

                    // Select the inserted text (minus prefix/suffix)
                    let new_start = start + prefix.len();
                    let new_end = new_start + if selected.is_empty() { placeholder.len() } else { selected.len() };
                    let _ = ta.set_selection_start(Some(new_start as u32));
                    let _ = ta.set_selection_end(Some(new_end as u32));

                    // Dispatch input event so Leptos picks up the change
                    let event = web_sys::Event::new("input").unwrap();
                    let _ = ta.dispatch_event(&event);
                }
            }
        }
    };

    let insert_at_line_start = move |prefix: &str| {
        if let Some(window) = web_sys::window() {
            if let Some(document) = window.document() {
                if let Some(textarea) = document.query_selector("textarea.editor-textarea")
                    .ok()
                    .flatten()
                {
                    let ta: HtmlTextAreaElement = textarea.unchecked_into();
                    let start = ta.selection_start().unwrap_or(Some(0)).unwrap_or(0);
                    let start = start as usize;
                    let value = ta.value();

                    // Find the start of the current line
                    let line_start = value[..start].rfind('\n').map(|n| n + 1).unwrap_or(0);
                    let new_value = format!("{}{}{}", &value[..line_start], prefix, &value[line_start..]);

                    ta.set_value(&new_value);
                    let _ = ta.focus();

                    // Place cursor after the prefix
                    let new_cursor = start + prefix.len();
                    let _ = ta.set_selection_start(Some(new_cursor as u32));
                    let _ = ta.set_selection_end(Some(new_cursor as u32));

                    let event = web_sys::Event::new("input").unwrap();
                    let _ = ta.dispatch_event(&event);
                }
            }
        }
    };

    // Toolbar button handlers
    let bold_action = move |_: leptos::ev::MouseEvent| { wrap_selection("**", "**"); };
    let italic_action = move |_: leptos::ev::MouseEvent| { wrap_selection("*", "*"); };
    let code_action = move |_: leptos::ev::MouseEvent| { wrap_selection("`", "`"); };
    let code_block_action = move |_: leptos::ev::MouseEvent| { wrap_selection("```\n", "\n```"); };
    let link_action = move |_: leptos::ev::MouseEvent| { wrap_selection("[", "](url)"); };
    let h1_action = move |_: leptos::ev::MouseEvent| { insert_at_line_start("# "); };
    let h2_action = move |_: leptos::ev::MouseEvent| { insert_at_line_start("## "); };
    let h3_action = move |_: leptos::ev::MouseEvent| { insert_at_line_start("### "); };
    let ul_action = move |_: leptos::ev::MouseEvent| { insert_at_line_start("- "); };
    let ol_action = move |_: leptos::ev::MouseEvent| { insert_at_line_start("1. "); };
    let blockquote_action = move |_: leptos::ev::MouseEvent| { insert_at_line_start("> "); };

    // --- Toggle preview ---
    let toggle_preview = move |_| {
        set_editor_state.update(|s| {
            s.show_preview = !s.show_preview;
        });
    };

    // --- Toggle auto-save ---
    let toggle_auto_save = move |_| {
        set_editor_state.update(|s| {
            s.auto_save_enabled = !s.auto_save_enabled;
        });
    };

    // --- File export (.md download) ---
    let export_markdown = move |_: leptos::ev::MouseEvent| {
        let content = document_content.get_untracked();
        if let Some(window) = web_sys::window() {
            if let Some(document) = window.document() {
                let parts = js_sys::Array::of1(&content.into());
                let options = web_sys::BlobPropertyBag::new();
                options.set_type("text/markdown");
                let blob = web_sys::Blob::new_with_str_sequence_and_options(
                    &parts.into(),
                    &options,
                )
                .ok();
                if let Some(blob) = blob {
                    let url = web_sys::Url::create_object_url_with_blob(&blob).unwrap_or_default();
                    let a = document.create_element("a").unwrap();
                    let a: web_sys::HtmlAnchorElement = a.unchecked_into();
                    a.set_href(&url);
                    a.set_download("document.md");
                    a.click();
                    let _ = web_sys::Url::revoke_object_url(&url);
                }
            }
        }
    };

    // --- Keyboard shortcut handler ---
    {
        let window = web_sys::window().unwrap();
        let closure = Closure::<dyn Fn(web_sys::KeyboardEvent)>::new(move |e: web_sys::KeyboardEvent| {
            let key = e.key();
            let ctrl = e.ctrl_key() || e.meta_key();

            if ctrl && key == "s" {
                e.prevent_default();
                save_document_ks();
            }
        });
        window
            .add_event_listener_with_callback("keydown", closure.as_ref().unchecked_ref())
            .unwrap();
        closure.forget();
    }

    // --- Connection indicator ---
    let connection_indicator = move || {
        let state = connection_state.get();
        let (color, text) = match state {
            ConnectionState::Connected => ("bg-green-500", "Connected"),
            ConnectionState::Connecting => ("bg-yellow-500", "Connecting..."),
            ConnectionState::Reconnecting => ("bg-orange-500", "Reconnecting..."),
            ConnectionState::Disconnected => ("bg-red-500", "Disconnected"),
        };
        view! {
            <div class="flex items-center gap-2">
                <div class={format!("w-2 h-2 rounded-full {}", color)}></div>
                <span class="text-xs text-gray-500 dark:text-gray-400">{text}</span>
            </div>
        }
    };

    view! {
        <div class="flex flex-col h-full">
            // Formatting toolbar
            <div class="flex items-center gap-1 px-3 py-1.5 border-b border-gray-200 dark:border-gray-700 bg-gray-50 dark:bg-gray-850 overflow-x-auto flex-wrap flex-shrink-0 no-print toolbar">
                // Text formatting
                <ToolbarButton title="Bold (Ctrl+B)" on_click=bold_action>{"B"}</ToolbarButton>
                <ToolbarButton title="Italic (Ctrl+I)" on_click=italic_action>
                    <span class="italic">{"I"}</span>
                </ToolbarButton>
                <ToolbarButton title="Inline Code" on_click=code_action>
                    <span class="font-mono text-xs">{"<>"}</span>
                </ToolbarButton>
                <ToolbarButton title="Code Block" on_click=code_block_action>
                    <span class="font-mono text-xs">{"{ }"}</span>
                </ToolbarButton>

                <ToolbarDivider />

                // Headings
                <ToolbarButton title="Heading 1" on_click=h1_action>
                    <span class="font-bold text-xs">{"H1"}</span>
                </ToolbarButton>
                <ToolbarButton title="Heading 2" on_click=h2_action>
                    <span class="font-bold text-xs">{"H2"}</span>
                </ToolbarButton>
                <ToolbarButton title="Heading 3" on_click=h3_action>
                    <span class="font-bold text-xs">{"H3"}</span>
                </ToolbarButton>

                <ToolbarDivider />

                // Lists
                <ToolbarButton title="Bullet List" on_click=ul_action>{"\u{2022}"}</ToolbarButton>
                <ToolbarButton title="Numbered List" on_click=ol_action>{"1."}</ToolbarButton>
                <ToolbarButton title="Blockquote" on_click=blockquote_action>{"\u{201C}"}</ToolbarButton>

                <ToolbarDivider />

                // Link
                <ToolbarButton title="Link" on_click=link_action>
                    <svg class="w-4 h-4" viewBox="0 0 20 20" fill="currentColor">
                        <path d="M12.586 4.586a2 2 0 112.828 2.828l-3 3a2 2 0 01-2.828 0 1 1 0 00-1.414 1.414 4 4 0 005.656 0l3-3a4 4 0 00-5.656-5.656l-1.5 1.5a1 1 0 101.414 1.414l1.5-1.5a2 2 0 012.828 0z" />
                        <path d="M3.414 15.414a2 2 0 102.828-2.828l3-3a2 2 0 00-2.828 0 1 1 0 10-1.414-1.414 4 4 0 105.656 0l3-3a4 4 0 00-5.656 5.656l-1.5-1.5a1 1 0 00-1.414 1.414l1.5 1.5a2 2 0 01-2.828 0z" />
                    </svg>
                </ToolbarButton>

                // Spacer
                <div class="flex-1"></div>

                // Status indicators
                {connection_indicator}
                <div
                    class="w-2 h-2 rounded-full"
                    title={move || if editor_state.get().dirty { "Unsaved changes" } else { "All changes saved" }}
                    class=("bg-yellow-400", move || editor_state.get().dirty)
                    class=("bg-green-500", move || !editor_state.get().dirty)
                ></div>
            </div>

            // Top toolbar (actions)
            <div class="flex items-center justify-between px-4 py-2 border-b border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-800 flex-wrap gap-2 no-print toolbar">
                <div class="flex items-center gap-2">
                    <button
                        class="px-2 py-1 text-xs rounded border border-gray-300 dark:border-gray-600 hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors"
                        on:click=toggle_preview
                    >
                        {move || {
                            if editor_state.get().show_preview {
                                "Hide Preview"
                            } else {
                                "Show Preview"
                            }
                        }}
                    </button>
                    <button
                        class="px-2 py-1 text-xs rounded border border-gray-300 dark:border-gray-600 hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors"
                        on:click=toggle_auto_save
                    >
                        {move || {
                            if editor_state.get().auto_save_enabled {
                                "Auto-save: On"
                            } else {
                                "Auto-save: Off"
                            }
                        }}
                    </button>
                    <button
                        class="px-2 py-1 text-xs rounded border border-gray-300 dark:border-gray-600 hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors"
                        on:click=export_markdown
                    >
                        "Export .md"
                    </button>
                </div>

                <div class="flex items-center gap-2">
                    <span class="text-xs text-gray-400 hidden sm:inline">"Ctrl+S to save"</span>
                    <button
                        class="px-3 py-1 text-xs rounded bg-blue-600 text-white hover:bg-blue-700 transition-colors disabled:opacity-50"
                        on:click=save_document
                    >
                        {move || if editor_state.get().is_saving {
                            "Saving..."
                        } else {
                            "Save"
                        }}
                    </button>
                    {move || {
                        editor_state.get().presence_users.iter().map(|user| {
                            let initial = user.user_name.chars().next().unwrap_or('?').to_uppercase().to_string();
                            let bg_color = user.color.clone();
                            let title = user.user_name.clone();
                            view! {
                                <div
                                    class="w-7 h-7 rounded-full flex items-center justify-center text-white text-xs font-medium"
                                    style={format!("background-color: {}", bg_color)}
                                    title={title}
                                >
                                    {initial}
                                </div>
                            }
                        }).collect::<Vec<_>>()
                    }}
                </div>
            </div>

            // Editor + Preview area
            <div class="flex-1 overflow-hidden flex">
                // Textarea pane
                <div
                    class="flex-1 overflow-hidden"
                    style={move || if editor_state.get().show_preview { "min-width: 0px".to_string() } else { "min-width: 100%".to_string() }}
                >
                    <textarea
                        class="editor-textarea w-full h-full p-4 resize-none focus:outline-none bg-white dark:bg-gray-900 text-gray-900 dark:text-gray-100 font-mono text-sm leading-relaxed"
                        placeholder="Start writing markdown..."
                        on:input=handle_input
                        prop:value={move || editor_state.get().content}
                    />
                </div>

                // Divider (only when preview visible)
                {move || {
                    if editor_state.get().show_preview {
                        view! {
                            <div class="w-px bg-gray-200 dark:bg-gray-700 flex-shrink-0"></div>
                        }.into_any()
                    } else {
                        ().into_any()
                    }
                }}

                // Preview pane
                {move || {
                    if editor_state.get().show_preview {
                        view! {
                            <div class="flex-1 overflow-auto p-6 bg-white dark:bg-gray-50">
                                {move || {
                                    let state = editor_state.get();
                                    if let Some(err) = &state.render_error {
                                        view! {
                                            <div class="p-3 mb-4 bg-yellow-50 dark:bg-yellow-900/20 border border-yellow-200 dark:border-yellow-800 rounded text-sm text-yellow-700 dark:text-yellow-300">
                                                {err.clone()}
                                            </div>
                                        }.into_any()
                                    } else if state.preview_html.is_empty() && state.content.is_empty() {
                                        view! {
                                            <div class="flex items-center justify-center h-full text-gray-400 dark:text-gray-500">
                                                <div class="text-center">
                                                    <p class="text-lg mb-1">"No content yet"</p>
                                                    <p class="text-sm">"Start typing in the editor to see a preview"</p>
                                                </div>
                                            </div>
                                        }.into_any()
                                    } else if state.preview_html.is_empty() {
                                        view! {
                                            <div class="flex items-center justify-center h-full text-gray-400 dark:text-gray-500">
                                                <p class="text-sm">"Rendering preview..."</p>
                                            </div>
                                        }.into_any()
                                    } else {
                                        view! {
                                            <div
                                                class="prose prose-sm dark:prose-invert max-w-none prose-headings:font-semibold prose-a:text-blue-600 dark:prose-a:text-blue-400 prose-code:text-pink-600 dark:prose-code:text-pink-400 prose-pre:bg-gray-100 dark:prose-pre:bg-gray-800"
                                                inner_html={state.preview_html.clone()}
                                            ></div>
                                        }.into_any()
                                    }
                                }}
                            </div>
                        }.into_any()
                    } else {
                        ().into_any()
                    }
                }}
            </div>

            // Bottom status bar
            <div class="flex items-center justify-between px-4 py-2 border-t border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-800 text-xs text-gray-500 dark:text-gray-400">
                <div>
                    {move || {
                        let state = editor_state.get();
                        format!("{} words \u{00b7} {} chars", state.word_count, state.character_count)
                    }}
                </div>
                <div>
                    {move || {
                        let state = editor_state.get();
                        if state.is_saving {
                            "Saving...".to_string()
                        } else if let Some(time) = &state.last_saved {
                            format!("Last saved: {}", time)
                        } else {
                            "Not saved".to_string()
                        }
                    }}
                </div>
            </div>
        </div>
    }
}

// ============================================================================
// Toolbar Components
// ============================================================================

#[component]
fn ToolbarButton(
    title: &'static str,
    on_click: impl Fn(leptos::ev::MouseEvent) + 'static,
    children: Children,
) -> impl IntoView {
    let child_views = children();
    view! {
        <button
            class="p-1.5 rounded hover:bg-gray-200 dark:hover:bg-gray-600 text-gray-600 dark:text-gray-300 transition-colors flex-shrink-0"
            on:click=on_click
            title=title
        >
            {child_views}
        </button>
    }
}

#[component]
fn ToolbarDivider() -> impl IntoView {
    view! {
        <div class="w-px h-5 bg-gray-300 dark:bg-gray-600 mx-1 flex-shrink-0"></div>
    }
}

// ============================================================================
// Diff & Apply Operations (unchanged from v0.7.0)
// ============================================================================

fn compute_diff(old_content: &str, new_content: &str) -> EditOperation {
    let old_len = old_content.len();
    let new_len = new_content.len();

    if new_len > old_len {
        let mut insert_pos = 0;
        let mut insert_text = String::new();

        for (i, (old_c, new_c)) in old_content.chars().zip(new_content.chars()).enumerate() {
            if old_c != new_c {
                insert_pos = i;
                break;
            }
            insert_pos = i + 1;
        }

        if insert_text.is_empty() && new_len > old_len {
            insert_text = new_content[old_len..new_len].to_string();
        }

        if insert_text.is_empty() {
            for (i, (old_c, new_c)) in old_content
                .chars()
                .rev()
                .zip(new_content.chars().rev())
                .enumerate()
            {
                if old_c != new_c {
                    insert_pos = old_len - i;
                    insert_text = new_content[insert_pos..].to_string();
                    break;
                }
            }
        }

        if insert_text.is_empty() {
            insert_pos = old_content
                .chars()
                .take_while(|c| new_content.starts_with(&c.to_string()))
                .count();
            insert_text = new_content[insert_pos..new_len].to_string();
        }

        EditOperation::insert(insert_pos, insert_text)
    } else if new_len < old_len {
        let mut delete_pos = 0;
        let delete_len = old_len - new_len;

        for (i, (old_c, new_c)) in old_content.chars().zip(new_content.chars()).enumerate() {
            if old_c != new_c {
                delete_pos = i;
                break;
            }
            delete_pos = i + 1;
        }

        EditOperation::delete(delete_pos, delete_len)
    } else {
        for (i, (old_c, new_c)) in old_content.chars().zip(new_content.chars()).enumerate() {
            if old_c != new_c {
                return EditOperation::replace(i, 1, new_c.to_string());
            }
        }
        EditOperation::insert(0, String::new())
    }
}

fn apply_operation(content: &str, op: &EditOperation) -> String {
    match op.operation_type.as_str() {
        "insert" => {
            let pos = op.position.min(content.len());
            let text = op.text.as_deref().unwrap_or("");
            let mut result = String::with_capacity(content.len() + text.len());
            result.push_str(&content[..pos]);
            result.push_str(text);
            result.push_str(&content[pos..]);
            result
        }
        "delete" => {
            let pos = op.position.min(content.len());
            let len = op.length.unwrap_or(0);
            let end = (pos + len).min(content.len());
            let mut result = String::with_capacity(content.len().saturating_sub(end - pos));
            result.push_str(&content[..pos]);
            result.push_str(&content[end..]);
            result
        }
        "replace" => {
            let pos = op.position.min(content.len());
            let len = op.length.unwrap_or(0);
            let end = (pos + len).min(content.len());
            let text = op.text.as_deref().unwrap_or("");
            let mut result = String::with_capacity(content.len() - (end - pos) + text.len());
            result.push_str(&content[..pos]);
            result.push_str(text);
            result.push_str(&content[end..]);
            result
        }
        _ => content.to_string(),
    }
}

// ============================================================================
// Presence Indicators Component
// ============================================================================

#[component]
pub fn PresenceIndicators(users: Vec<PresenceUser>) -> impl IntoView {
    view! {
        <div class="flex items-center -space-x-2">
            {users.into_iter().map(|user| {
                let initial = user.user_name.chars().next().unwrap_or('?').to_uppercase().to_string();
                let bg_color = user.color.clone();
                let title = user.user_name.clone();
                view! {
                    <div
                        class="w-8 h-8 rounded-full flex items-center justify-center text-white text-xs font-medium ring-2 ring-white dark:ring-gray-800"
                        style={format!("background-color: {}", bg_color)}
                        title={title}
                    >
                        {initial}
                    </div>
                }
            }).collect::<Vec<_>>()}
        </div>
    }
}
