use crate::components::FocusTrap;
use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use wasm_bindgen::JsCast;

#[derive(Debug, Clone)]
pub struct CommandItem {
    pub id: String,
    pub label: String,
    pub category: String,
    pub action_type: String,
    pub action_value: String,
}

#[component]
pub fn CommandPalette(open: ReadSignal<bool>, set_open: WriteSignal<bool>) -> impl IntoView {
    let (query, set_query) = signal(String::new());
    let (selected, set_selected) = signal(0usize);
    let (filtered, set_filtered) = signal(Vec::<CommandItem>::new());
    let previous_focus: RwSignal<Option<web_sys::Element>> = RwSignal::new(None);

    let all_commands: Vec<CommandItem> = vec![
        CommandItem {
            id: "nav-dashboard".into(),
            label: "Go to Dashboard".into(),
            category: "Navigation".into(),
            action_type: "navigate".into(),
            action_value: "/dashboard".into(),
        },
        CommandItem {
            id: "nav-documents".into(),
            label: "Go to Documents".into(),
            category: "Navigation".into(),
            action_type: "navigate".into(),
            action_value: "/documents".into(),
        },
        CommandItem {
            id: "nav-graph".into(),
            label: "Go to Graph".into(),
            category: "Navigation".into(),
            action_type: "navigate".into(),
            action_value: "/graph".into(),
        },
        CommandItem {
            id: "nav-search".into(),
            label: "Go to Search".into(),
            category: "Navigation".into(),
            action_type: "navigate".into(),
            action_value: "/search".into(),
        },
        CommandItem {
            id: "nav-settings".into(),
            label: "Go to Settings".into(),
            category: "Navigation".into(),
            action_type: "navigate".into(),
            action_value: "/settings".into(),
        },
        CommandItem {
            id: "nav-catalog".into(),
            label: "Go to Catalog".into(),
            category: "Navigation".into(),
            action_type: "navigate".into(),
            action_value: "/catalog".into(),
        },
        CommandItem {
            id: "action-create-doc".into(),
            label: "Create Document".into(),
            category: "Actions".into(),
            action_type: "navigate".into(),
            action_value: "/documents".into(),
        },
        CommandItem {
            id: "action-toggle-theme".into(),
            label: "Toggle Dark Mode".into(),
            category: "Actions".into(),
            action_type: "toggle_theme".into(),
            action_value: String::new(),
        },
        CommandItem {
            id: "action-sign-out".into(),
            label: "Sign Out".into(),
            category: "Actions".into(),
            action_type: "sign_out".into(),
            action_value: String::new(),
        },
    ];

    let all_commands_for_effect = all_commands.clone();
    Effect::new(move |_| {
        let q = query.get().to_lowercase();
        let f: Vec<CommandItem> = if q.is_empty() {
            all_commands_for_effect.clone()
        } else {
            all_commands_for_effect
                .iter()
                .filter(|c| {
                    c.label.to_lowercase().contains(&q) || c.category.to_lowercase().contains(&q)
                })
                .cloned()
                .collect()
        };
        set_filtered.set(f);
        set_selected.set(0);
    });

    let set_open_for_listener = set_open;
    let set_query_for_listener = set_query;
    let set_selected_for_listener = set_selected;
    let previous_focus_for_listener = previous_focus;

    Effect::new(move |_| {
        let cb = wasm_bindgen::closure::Closure::<dyn Fn(wasm_bindgen::JsValue)>::new(
            move |event: wasm_bindgen::JsValue| {
                let ev = event.unchecked_into::<web_sys::KeyboardEvent>();
                let target_tag = ev
                    .target()
                    .and_then(|t| t.dyn_into::<web_sys::Element>().ok())
                    .map(|el| el.tag_name().to_lowercase())
                    .unwrap_or_default();
                if target_tag == "input" || target_tag == "textarea" || target_tag == "select" {
                    return;
                }
                let key = ev.key();
                if (ev.meta_key() || ev.ctrl_key()) && key == "k" {
                    ev.prevent_default();
                    set_open_for_listener.update(|o| {
                        if !*o {
                            if let Some(doc) = web_sys::window().and_then(|w| w.document()) {
                                previous_focus_for_listener.set(doc.active_element());
                            }
                        }
                        *o = !*o;
                    });
                    set_query_for_listener.set(String::new());
                    set_selected_for_listener.set(0);
                }
            },
        );
        if let Some(window) = web_sys::window() {
            let _ = window.add_event_listener_with_callback("keydown", cb.as_ref().unchecked_ref());
            cb.forget();
        }
    });

    let execute_command = move |cmd: CommandItem| {
        match cmd.action_type.as_str() {
            "navigate" => {
                let navigate = use_navigate();
                navigate(&cmd.action_value, Default::default());
            }
            "toggle_theme" => {
                if let Some(window) = web_sys::window() {
                    if let Ok(Some(storage)) = window.local_storage() {
                        let current = storage
                            .get_item("tachyon-theme")
                            .ok()
                            .flatten()
                            .unwrap_or_else(|| "light".to_string());
                        let new_theme = if current == "dark" { "light" } else { "dark" };
                        let _ = storage.set_item("tachyon-theme", new_theme);
                        if let Some(document) = window.document() {
                            if let Some(html) = document.document_element() {
                                if new_theme == "dark" {
                                    let _ = html.class_list().add_1("dark");
                                } else {
                                    let _ = html.class_list().remove_1("dark");
                                }
                            }
                        }
                    }
                }
            }
            "sign_out" => {
                if let Some(window) = web_sys::window() {
                    if let Ok(Some(storage)) = window.local_storage() {
                        let _ = storage.remove_item("tachyon_token");
                        let _ = storage.remove_item("tachyon_remember");
                    }
                }
                let navigate = use_navigate();
                navigate("/login", Default::default());
            }
            _ => {}
        }
        set_open.set(false);
        if let Some(el) = previous_focus.get() {
            let _ = el.unchecked_into::<web_sys::HtmlElement>().focus();
        }
    };

    let on_input = move |ev| {
        set_query.set(event_target_value(&ev));
        set_selected.set(0);
    };

    let on_keydown = move |ev: web_sys::KeyboardEvent| {
        let key = ev.key();
        let len = filtered.with(|f| f.len());
        if key == "ArrowDown" {
            ev.prevent_default();
            let new = if selected.get() + 1 >= len {
                0
            } else {
                selected.get() + 1
            };
            set_selected.set(new);
        } else if key == "ArrowUp" {
            ev.prevent_default();
            let new = if selected.get() == 0 {
                len.saturating_sub(1)
            } else {
                selected.get() - 1
            };
            set_selected.set(new);
        } else if key == "Enter" {
            ev.prevent_default();
            let idx = selected.get();
            let cmd = filtered.with(|f| f.get(idx).cloned());
            if let Some(cmd) = cmd {
                execute_command(cmd);
            }
        }
    };

    let on_backdrop_click = move |_: leptos::ev::MouseEvent| {
        set_open.set(false);
        if let Some(el) = previous_focus.get() {
            let _ = el.unchecked_into::<web_sys::HtmlElement>().focus();
        }
    };

    let on_modal_click = move |ev: leptos::ev::MouseEvent| {
        ev.stop_propagation();
    };

    let on_escape_keydown = move |ev: web_sys::KeyboardEvent| {
        if ev.key() == "Escape" {
            set_open.set(false);
            if let Some(el) = previous_focus.get() {
                let _ = el.unchecked_into::<web_sys::HtmlElement>().focus();
            }
        }
    };

    view! {
        <div
            class="fixed inset-0 z-50 bg-black/50"
            style={move || if open.get() { "" } else { "display: none;" }}
            on:click=on_backdrop_click
        ></div>

        <div
            class="fixed inset-0 z-50 flex items-start sm:justify-center pt-[10vh] sm:pt-[20vh] px-4"
            style={move || if open.get() { "" } else { "display: none;" }}
        >
            <FocusTrap active=open.into()>
                <div
                    class="w-full max-w-lg bg-white dark:bg-gray-800 rounded-none border-2 border-gray-900 dark:border-gray-100 spatial-3 overflow-hidden"
                    role="dialog"
                    attr:aria-modal="true"
                    attr:aria-label="Command palette"
                    on:click=on_modal_click
                    on:keydown=on_escape_keydown
                >
                    <div class="flex items-center px-4 border-b border-gray-200 dark:border-gray-700">
                        <svg class="h-5 w-5 text-gray-400 flex-shrink-0" aria-hidden="true" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
                        </svg>
                        <input
                            type="text"
                            placeholder="Search documents, commands..."
                            attr:aria-label="Search commands"
                            class="w-full min-h-[44px] px-3 py-3 bg-transparent text-gray-900 dark:text-gray-100 outline-none"
                            prop:value={move || query.get()}
                            on:input=on_input
                            on:keydown=on_keydown
                        />
                        <kbd class="text-xs text-gray-400 border border-gray-300 dark:border-gray-600 rounded px-1.5 py-0.5 flex-shrink-0">"ESC"</kbd>
                    </div>

                    <div class="max-h-80 overflow-y-auto py-2" role="listbox" attr:aria-label="Command results">
                        {move || {
                            let items = filtered.get();
                            if items.is_empty() {
                                view! {
                                    <div class="px-4 py-8 text-center text-sm text-gray-500 dark:text-gray-400">
                                        "No results found"
                                    </div>
                                }.into_any()
                            } else {
                                let mut last_category = String::new();
                                view! {
                                    <div>
                                        {items.into_iter().enumerate().map(|(idx, cmd)| {
                                            let show_category = cmd.category != last_category;
                                            if show_category {
                                                last_category = cmd.category.clone();
                                            }
                                            let is_selected = move || idx == selected.get();
                                            let cmd_clone = cmd.clone();
                                            let cmd_for_click = cmd.clone();
                                            let on_click = move |_: leptos::ev::MouseEvent| {
                                                execute_command(cmd_for_click.clone());
                                            };
                                            let category_label = if show_category { cmd.category.clone() } else { String::new() };
                                            view! {
                                                <>
                                                    {if show_category {
                                                        view! {
                                                            <div class="px-4 py-1.5 text-xs font-semibold text-gray-400 dark:text-gray-500 uppercase tracking-wider">
                                                                {category_label}
                                                            </div>
                                                        }.into_any()
                                                    } else {
                                                        view! { <div style="display:none"></div> }.into_any()
                                                    }}
                                                    <button
                                                        class="w-full px-4 py-2 text-left text-sm flex items-center gap-3 transition-colors "
                                                        class=("bg-blue-50 dark:bg-blue-900/30 text-blue-900 dark:text-blue-100", is_selected)
                                                        class=("text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700/50", move || !is_selected())
                                                        role="option"
                                                        attr:aria-selected=move || if is_selected() { "true" } else { "false" }
                                                        on:click=on_click
                                                        on:mouseenter=move |_| set_selected.set(idx)
                                                    >
                                                        {match cmd_clone.action_type.as_str() {
                                                            "navigate" => view! {
                                                                <svg class="h-4 w-4 text-gray-400 flex-shrink-0" aria-hidden="true" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 7l5 5m0 0l-5 5m5-5H6" />
                                                                </svg>
                                                            }.into_any(),
                                                            "toggle_theme" => view! {
                                                                <svg class="h-4 w-4 text-gray-400 flex-shrink-0" aria-hidden="true" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M20.354 15.354A9 9 0 018.646 3.646 9.003 9.003 0 0012 21a9.003 9.003 0 008.354-5.646z" />
                                                                </svg>
                                                            }.into_any(),
                                                            "sign_out" => view! {
                                                                <svg class="h-4 w-4 text-gray-400 flex-shrink-0" aria-hidden="true" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M17 16l4-4m0 0l-4-4m4 4H7m6 4v1a3 3 0 01-3 3H6a3 3 0 01-3-3V7a3 3 0 013-3h4a3 3 0 013 3v1" />
                                                                </svg>
                                                            }.into_any(),
                                                            _ => view! {
                                                                <svg class="h-4 w-4 text-gray-400 flex-shrink-0" aria-hidden="true" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 10V3L4 14h7v7l9-11h-7z" />
                                                                </svg>
                                                            }.into_any(),
                                                        }}
                                                        <span>{cmd_clone.label}</span>
                                                    </button>
                                                </>
                                            }
                                        }).collect::<Vec<_>>()}
                                    </div>
                                }.into_any()
                            }
                        }}
                    </div>

                    <div class="px-4 py-2 border-t border-gray-200 dark:border-gray-700 text-xs text-gray-400 text-center">
                        <span class="hidden sm:inline">"\u{2191}\u{2193} Navigate \u{00b7} \u{21b5} Select \u{00b7} Esc Close"</span>
                        <span class="sm:hidden">"Tap to select"</span>
                    </div>
                </div>
            </FocusTrap>
        </div>
    }
}
