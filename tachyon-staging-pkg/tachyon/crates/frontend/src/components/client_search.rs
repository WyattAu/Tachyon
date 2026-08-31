#![allow(dead_code)]

use crate::storage::{BrowserStore, StoredDocument};
use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::JsCast;

#[component]
pub fn ClientSearch(open: ReadSignal<bool>, set_open: WriteSignal<bool>) -> impl IntoView {
    let (query, set_query) = signal(String::new());
    let (results, set_results) = signal(Vec::<StoredDocument>::new());
    let (searching, set_searching) = signal(false);

    let store = use_context::<BrowserStore>().unwrap_or_default();

    let timeout_handle: Rc<RefCell<Option<i32>>> = Rc::new(RefCell::new(None));

    Effect::new(move |_| {
        if let Some(id) = timeout_handle.borrow_mut().take() {
            if let Some(window) = web_sys::window() {
                window.clear_timeout_with_handle(id);
            }
        }

        let q = query.get();
        if q.is_empty() {
            set_results.set(Vec::new());
            set_searching.set(false);
            return;
        }

        set_searching.set(true);

        let store = store.clone();
        let set_r = set_results;
        let set_s = set_searching;
        let handle = timeout_handle.clone();
        let handle_for_cb = timeout_handle.clone();

        let cb = wasm_bindgen::closure::Closure::<dyn Fn()>::new(move || {
            let r = store.search(&q);
            set_r.set(r);
            set_s.set(false);
            handle_for_cb.borrow_mut().take();
        });

        if let Some(window) = web_sys::window() {
            match window.set_timeout_with_callback_and_timeout_and_arguments_0(
                cb.as_ref().unchecked_ref(),
                300,
            ) {
                Ok(id) => {
                    *handle.borrow_mut() = Some(id);
                }
                Err(_) => {
                    set_searching.set(false);
                }
            }
            cb.forget();
        }
    });

    Effect::new(move |_| {
        if open.get() {
            if let Some(window) = web_sys::window() {
                if let Some(doc) = window.document() {
                    if let Ok(Some(el)) = doc.query_selector("#client-search-input") {
                        if let Ok(input) = el.dyn_into::<web_sys::HtmlInputElement>() {
                            let _ = input.focus();
                        }
                    }
                }
            }
        }
    });

    let on_input = move |ev| {
        set_query.set(event_target_value(&ev));
    };

    let on_clear = move |_: leptos::ev::MouseEvent| {
        set_query.set(String::new());
    };

    let on_backdrop_click = move |_: leptos::ev::MouseEvent| {
        set_open.set(false);
    };

    let on_modal_click = move |ev: leptos::ev::MouseEvent| {
        ev.stop_propagation();
    };

    let on_escape = move |ev: web_sys::KeyboardEvent| {
        if ev.key() == "Escape" {
            ev.prevent_default();
            set_open.set(false);
        }
    };

    view! {
        <div
            class="fixed inset-0 z-50 bg-black/50"
            style={move || if open.get() { "" } else { "display: none;" }}
            on:click=on_backdrop_click
        ></div>

        <div
            class="fixed inset-0 z-50 flex items-start sm:justify-center pt-[10vh] sm:pt-[15vh] px-4"
            style={move || if open.get() { "" } else { "display: none;" }}
        >
            <div
                class="w-full max-w-2xl bg-white dark:bg-gray-800 rounded-none border-2 border-gray-900 dark:border-gray-100 spatial-3 overflow-hidden"
                role="dialog"
                aria-label="Search local documents"
                on:click=on_modal_click
                on:keydown=on_escape
            >
                <div class="flex items-center px-4 border-b border-gray-200 dark:border-gray-700 gap-3">
                    <svg class="h-5 w-5 text-gray-400 flex-shrink-0" aria-hidden="true" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
                    </svg>
                    <input
                        id="client-search-input"
                        type="text"
                        placeholder="Search local documents..."
                        class="w-full min-h-[44px] px-3 py-3 bg-transparent text-gray-900 dark:text-gray-100 outline-none placeholder-gray-400"
                        prop:value={move || query.get()}
                        on:input=on_input
                        aria-label="Search local documents"
                    />
                    <Show when={move || !query.get().is_empty()}>
                        <button
                            on:click=on_clear
                            class="p-1.5 text-gray-400 hover:text-gray-600 dark:hover:text-gray-300 rounded transition-colors"
                            aria-label="Clear search"
                        >
                            <svg class="h-4 w-4" aria-hidden="true" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
                            </svg>
                        </button>
                    </Show>
                    <Show when={move || searching.get()}>
                        <svg class="h-4 w-4 text-gray-400 animate-spin flex-shrink-0" aria-hidden="true" fill="none" viewBox="0 0 24 24">
                            <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                            <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                        </svg>
                    </Show>
                </div>

                <div class="max-h-96 overflow-y-auto">
                    {move || {
                        let items = results.get();
                        let q = query.get();
                        let is_searching = searching.get();

                        if q.is_empty() {
                            view! {
                                <div class="px-4 py-8 text-center text-sm text-gray-500 dark:text-gray-400">
                                    "Type to search your local documents"
                                </div>
                            }.into_any()
                        } else if items.is_empty() && !is_searching {
                            view! {
                                <div class="px-4 py-8 text-center text-sm text-gray-500 dark:text-gray-400">
                                    "No results found"
                                </div>
                            }.into_any()
                        } else if !items.is_empty() {
                            let count = items.len();
                            view! {
                                <div>
                                    <div class="px-4 py-2 text-xs text-gray-500 dark:text-gray-400 border-b border-gray-100 dark:border-gray-700" aria-live="polite">
                                        {format!(
                                            "{} result{} found",
                                            count,
                                            if count == 1 { "" } else { "s" }
                                        )}
                                    </div>
                                    <div class="divide-y divide-gray-100 dark:divide-gray-700/50">
                                        {items.into_iter().map(|doc| {
                                            let id = doc.document.id.clone();
                                            let title = doc.document.title.clone();
                                            let chars: Vec<char> = doc.document.content.chars().collect();
                                            let preview = if chars.len() > 150 {
                                                format!("{}...", chars[..150].iter().collect::<String>())
                                            } else {
                                                doc.document.content.clone()
                                            };
                                            let tags = doc.document.tags.clone();
                                            let date = doc.document.updated_at.split('T').next().unwrap_or("").to_string();
                                            let set_o = set_open;
                                            view! {
                                                <button
                                                    class="w-full px-4 py-3 text-left hover:bg-gray-50 dark:hover:bg-gray-700/50 transition-colors"
                                                    on:click=move |_| {
                                                        let navigate = use_navigate();
                                                        set_o.set(false);
                                                        navigate(&format!("/documents/{}", id), Default::default());
                                                    }
                                                >
                                                    <p class="text-sm font-medium text-gray-900 dark:text-white truncate">
                                                        {title.clone()}
                                                    </p>
                                                    <p class="text-xs text-gray-500 dark:text-gray-400 mt-0.5 line-clamp-2">
                                                        {preview.clone()}
                                                    </p>
                                                    <div class="flex items-center gap-2 mt-1.5">
                                                        {if !tags.is_empty() {
                                                            view! {
                                                                <div class="flex gap-1 flex-wrap">
                                                                    {tags.iter().take(3).map(|tag| {
                                                                        view! {
                                                                            <span class="inline-block text-xs px-1.5 py-0.5 bg-blue-50 dark:bg-blue-900/30 text-blue-700 dark:text-blue-300 rounded">
                                                                                {tag.clone()}
                                                                            </span>
                                                                        }
                                                                    }).collect::<Vec<_>>()}
                                                                </div>
                                                            }.into_any()
                                                        } else {
                                                            view! { <span></span> }.into_any()
                                                        }}
                                                        <span class="text-xs text-gray-400 dark:text-gray-500 ml-auto">
                                                            {date}
                                                        </span>
                                                    </div>
                                                </button>
                                            }
                                        }).collect::<Vec<_>>()}
                                    </div>
                                </div>
                            }.into_any()
                        } else {
                            view! {
                                <div class="px-4 py-8 text-center text-sm text-gray-500 dark:text-gray-400">
                                    <svg class="h-5 w-5 animate-spin mx-auto mb-2" aria-hidden="true" fill="none" viewBox="0 0 24 24">
                                        <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                                        <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                                    </svg>
                                    "Searching..."
                                </div>
                            }.into_any()
                        }
                    }}
                </div>

                <div class="px-4 py-2 border-t border-gray-200 dark:border-gray-700 text-xs text-gray-400 text-center flex items-center justify-center gap-3">
                    <span>"Searching local documents"</span>
                    <span class="text-gray-300 dark:text-gray-600">"\u{00b7}"</span>
                    <span>"Esc to close"</span>
                </div>
            </div>
        </div>
    }
}
