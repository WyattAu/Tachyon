// Journal page — daily outliner (journal mode) with auto-created dated entries

use crate::api::ApiClient;
use crate::components::{BreadcrumbItem, Breadcrumbs, OutlinerView};
use leptos::prelude::*;
use leptos_router::hooks::{use_navigate, use_params_map};
use tachyon_editor::outliner::OutlinerState;
use wasm_bindgen::JsValue;

/// Format a NaiveDate as YYYY-MM-DD
fn format_date(d: chrono::NaiveDate) -> String {
    d.format("%Y-%m-%d").to_string()
}

/// Parse a YYYY-MM-DD string to NaiveDate
fn parse_date(s: &str) -> Option<chrono::NaiveDate> {
    chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d").ok()
}

/// Find or create a journal entry for the given date, returns (document_id, content)
async fn find_or_create_journal(api: &ApiClient, date: &str) -> Result<(String, String), String> {
    let title = format!("Journal: {}", date);

    match api.search(&title, None, Some(1), Some(5)).await {
        Ok(results) => {
            for item in &results.results {
                if item.title == title {
                    match api.get_document(&item.id).await {
                        Ok(doc) => return Ok((doc.id, doc.content)),
                        Err(e) => return Err(format!("Failed to load journal: {}", e)),
                    }
                }
            }
            // No existing journal — create with default outliner content
            let default_content = "- Morning\n  - \n- Afternoon\n  - \n- Evening\n  - \n";
            let body = serde_json::json!({
                "title": title,
                "content": default_content,
                "tags": vec!["journal"],
            });
            match api.create_document(&body).await {
                Ok(doc) => Ok((doc.id, default_content.to_string())),
                Err(e) => Err(format!("Failed to create journal: {}", e)),
            }
        }
        Err(e) => Err(format!("Failed to search for journal: {}", e)),
    }
}

#[component]
pub fn JournalPage() -> impl IntoView {
    let params = use_params_map();
    let navigate = use_navigate();

    let today = chrono::Utc::now().date_naive();
    let initial_date =
        move || params.with(|p| p.get("date").clone().unwrap_or_else(|| format_date(today)));

    let (selected_date, set_selected_date) = signal(initial_date());
    let (doc_id, set_doc_id) = signal(None::<String>);
    let (outliner_state, set_outliner_state) = signal(OutlinerState::new());
    let (loading, set_loading) = signal(false);
    let (error, set_error) = signal(None::<String>);
    let (dirty, set_dirty) = signal(false);

    // When date changes, load or create the journal entry
    Effect::new(move |_| {
        let date = selected_date.get();
        if date.is_empty() {
            return;
        }
        set_loading.set(true);
        set_error.set(None);
        set_dirty.set(false);
        let api = ApiClient::default();
        let date_clone = date.clone();

        wasm_bindgen_futures::spawn_local(async move {
            match find_or_create_journal(&api, &date_clone).await {
                Ok((id, content)) => {
                    let state = OutlinerState::from_text(&content);
                    set_doc_id.set(Some(id));
                    set_outliner_state.set(state);
                    set_loading.set(false);
                    // Update URL
                    if let Some(window) = web_sys::window() {
                        if let Ok(history) = window.history() {
                            let state: &JsValue = &JsValue::from(js_sys::Object::new());
                            let _ = history.push_state_with_url(
                                state,
                                "",
                                Some(&format!("/journal/{}", date_clone)),
                            );
                        }
                    }
                }
                Err(e) => {
                    set_error.set(Some(e));
                    set_loading.set(false);
                }
            }
        });
    });

    // Save when dirty and navigating away (debounced)
    let save_journal = move || {
        if !dirty.get() {
            return;
        }
        if let Some(id) = doc_id.get() {
            let api = ApiClient::default();
            let content = outliner_state.get().to_text();
            let id_clone = id.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let body = serde_json::json!({
                    "content": content,
                });
                let _ = api.update_document(&id_clone, &body).await;
            });
            set_dirty.set(false);
        }
    };

    // Navigate to previous/next day
    let go_prev = move |_: web_sys::MouseEvent| {
        save_journal();
        if let Some(d) = parse_date(&selected_date.get()) {
            let prev = d - chrono::Duration::days(1);
            set_selected_date.set(format_date(prev));
        }
    };
    let go_next = move |_: web_sys::MouseEvent| {
        save_journal();
        if let Some(d) = parse_date(&selected_date.get()) {
            let next = d + chrono::Duration::days(1);
            set_selected_date.set(format_date(next));
        }
    };
    let go_today = move |_: web_sys::MouseEvent| {
        save_journal();
        set_selected_date.set(format_date(chrono::Utc::now().date_naive()));
    };

    view! {
        <div class="max-w-4xl mx-auto">
            <Breadcrumbs items={vec![
                BreadcrumbItem { label: "Journal".into(), href: None },
            ]}/>

            // Header with date navigation
            <div class="mb-6">
                <div class="flex items-center justify-between mb-4">
                    <h1 class="text-2xl font-bold text-gray-900 dark:text-white">"Journal"</h1>
                    {move || {
                        if dirty.get() {
                            view! {
                                <span class="text-xs text-amber-600 dark:text-amber-400">"Unsaved changes"</span>
                            }.into_any()
                        } else {
                            view! { <span/> }.into_any()
                        }
                    }}
                </div>
                <div class="flex items-center gap-3 bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 p-3">
                    <button
                        class="px-3 py-1.5 text-sm border border-gray-300 dark:border-gray-600 hover:bg-gray-50 dark:hover:bg-gray-700 text-gray-700 dark:text-gray-300 transition-colors"
                        on:click=go_prev
                        aria-label="Previous day"
                    >
                        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 19l-7-7 7-7" />
                        </svg>
                    </button>

                    <input
                        type="date"
                        class="flex-1 px-3 py-1.5 text-sm border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-700 text-gray-900 dark:text-white focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                        prop:value={move || selected_date.get()}
                        on:input=move |ev| {
                            let val = event_target_value(&ev);
                            if !val.is_empty() {
                                save_journal();
                                set_selected_date.set(val);
                            }
                        }
                    />

                    <button
                        class="px-3 py-1.5 text-sm border border-gray-300 dark:border-gray-600 hover:bg-gray-50 dark:hover:bg-gray-700 text-gray-700 dark:text-gray-300 transition-colors"
                        on:click=go_next
                        aria-label="Next day"
                    >
                        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7" />
                        </svg>
                    </button>

                    <button
                        class="px-3 py-1.5 text-sm bg-blue-600 text-white hover:bg-blue-700 transition-colors"
                        on:click=go_today
                    >
                        "Today"
                    </button>
                </div>

                // Day display
                <div class="text-sm text-gray-500 dark:text-gray-400 mt-2">
                    {move || {
                        if let Some(d) = parse_date(&selected_date.get()) {
                            let weekday = d.format("%A").to_string();
                            let formatted = d.format("%B %-d, %Y").to_string();
                            format!("{}, {}", weekday, formatted)
                        } else {
                            "Select a date".to_string()
                        }
                    }}
                </div>
            </div>

            // Outliner content area
            <div class="bg-white dark:bg-gray-800 rounded-none shadow border border-gray-200 dark:border-gray-700 min-h-[400px]">
                {move || {
                    if loading.get() {
                        view! {
                            <div class="flex items-center justify-center py-16">
                                <div class="flex items-center gap-3 text-gray-500 dark:text-gray-400">
                                    <div class="w-5 h-5 border-2 border-gray-400 border-t-transparent rounded-full animate-spin"></div>
                                    <span>"Loading journal..."</span>
                                </div>
                            </div>
                        }.into_any()
                    } else if let Some(err) = error.get() {
                        view! {
                            <div class="p-6">
                                <div class="p-4 bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded text-red-700 dark:text-red-300">
                                    {err}
                                </div>
                            </div>
                        }.into_any()
                    } else {
                        let state = outliner_state.get();
                        view! {
                            <OutlinerView initial_state=state />
                        }.into_any()
                    }
                }}
            </div>

            // Keyboard shortcuts help
            <div class="mt-4 text-xs text-gray-400 dark:text-gray-500 space-y-1">
                <p>"Tab / Shift+Tab — indent / outdent"</p>
                <p>"Alt+↑ / Alt+↓ — move item up / down"</p>
                <p>"Enter — new sibling item"</p>
                <p>"Double-click — edit item text"</p>
                <p>"Backspace on empty item — delete"</p>
            </div>
        </div>
    }
}
