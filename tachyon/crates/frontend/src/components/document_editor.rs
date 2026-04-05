// Document Editor Component
// Real-time collaborative document editor with markdown preview

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
    });

    let (connection_state, set_connection_state) = signal(ConnectionState::Disconnected);
    let (local_edit_version, set_local_edit_version) = signal(0u64);

    let debounce_handle: Rc<RefCell<Option<i32>>> = Rc::new(RefCell::new(None));
    let render_debounce: Rc<RefCell<Option<i32>>> = Rc::new(RefCell::new(None));
    let document_content = RwSignal::new(initial_content);

    // Clone for save handler before trigger_render consumes api_client and document_id
    let save_api = api_client.clone();
    let save_doc_id = document_id.clone();

    // --- Markdown rendering via API ---
    let trigger_render = move |content: String| {
        let render_dh = render_debounce.clone();
        let api = api_client.clone();
        let set_es = set_editor_state.clone();

        // Clear any pending render timeout
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
    let trigger_render_for_input = trigger_render;
    let handle_input = move |ev: Event| {
        let target: HtmlTextAreaElement = ev.target().unwrap().unchecked_into();
        let new_content = target.value();
        let old_content = document_content.get();

        let operation = compute_diff(&old_content, &new_content);
        document_content.set(new_content.clone());

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

    // --- Save document ---
    let save_document = move |_: leptos::ev::MouseEvent| {
        let api = save_api.clone();
        let doc_id = save_doc_id.clone();
        let content = document_content.get();
        let set_es = set_editor_state.clone();

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

    // --- Toggle preview ---
    let toggle_preview = move |_| {
        set_editor_state.update(|s| {
            s.show_preview = !s.show_preview;
        });
    };

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
            // Top toolbar
            <div class="flex items-center justify-between px-4 py-2 border-b border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-800">
                <div class="flex items-center gap-4">
                    {connection_indicator}
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
                </div>

                <div class="flex items-center gap-2">
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
                        class="w-full h-full p-4 resize-none focus:outline-none bg-white dark:bg-gray-900 text-gray-900 dark:text-gray-100 font-mono text-sm leading-relaxed"
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
                        format!("{} words, {} chars", state.word_count, state.character_count)
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
