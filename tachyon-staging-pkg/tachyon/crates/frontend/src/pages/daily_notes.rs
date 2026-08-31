// Daily Notes page — auto-creates a note per day, navigation via date picker

use crate::api::ApiClient;
use crate::components::{BreadcrumbItem, Breadcrumbs};
use leptos::prelude::*;
use leptos_router::hooks::{use_navigate, use_params_map};
use wasm_bindgen::JsValue;

/// Format a NaiveDate as YYYY-MM-DD
fn format_date(d: chrono::NaiveDate) -> String {
    d.format("%Y-%m-%d").to_string()
}

/// Parse a YYYY-MM-DD string to NaiveDate
fn parse_date(s: &str) -> Option<chrono::NaiveDate> {
    chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d").ok()
}

/// Find or create a daily note for the given date, returns (document_id, title)
async fn find_or_create_daily_note(
    api: &ApiClient,
    date: &str,
) -> Result<(String, String), String> {
    let title = format!("Daily Note: {}", date);

    // Search for existing daily note by title
    match api.search(&title, None, Some(1), Some(5)).await {
        Ok(results) => {
            // Look for an exact title match
            for item in &results.results {
                if item.title == title {
                    return Ok((item.id.clone(), item.title.clone()));
                }
            }
            // No exact match — create a new daily note
            let body = serde_json::json!({
                "title": title,
                "content": format!("# Daily Note: {}\n\nWrite your notes for today here.\n", date),
                "tags": vec!["daily-note"],
            });
            match api.create_document(&body).await {
                Ok(doc) => Ok((doc.id, doc.title)),
                Err(e) => Err(format!("Failed to create daily note: {}", e)),
            }
        }
        Err(e) => Err(format!("Failed to search for daily note: {}", e)),
    }
}

#[component]
pub fn DailyNotesPage() -> impl IntoView {
    let params = use_params_map();
    let navigate = use_navigate();

    // Determine the date from URL params, default to today
    let today = chrono::Utc::now().date_naive();
    let initial_date =
        move || params.with(|p| p.get("date").clone().unwrap_or_else(|| format_date(today)));

    let (selected_date, set_selected_date) = signal(initial_date());
    let (doc_id, set_doc_id) = signal(None::<String>);
    let (doc_content, set_doc_content) = signal(String::new());
    let (doc_title, set_doc_title) = signal(String::new());
    let (loading, set_loading) = signal(false);
    let (error, set_error) = signal(None::<String>);

    // When date changes, load or create the daily note
    Effect::new(move |_| {
        let date = selected_date.get();
        if date.is_empty() {
            return;
        }
        set_loading.set(true);
        set_error.set(None);
        let api = ApiClient::default();
        let date_clone = date.clone();

        wasm_bindgen_futures::spawn_local(async move {
            match find_or_create_daily_note(&api, &date_clone).await {
                Ok((id, title)) => {
                    // Fetch the full document content
                    match api.get_document(&id).await {
                        Ok(doc) => {
                            set_doc_id.set(Some(doc.id));
                            set_doc_content.set(doc.content);
                            set_doc_title.set(title);
                            set_loading.set(false);
                            // Update URL without triggering reload
                            if let Some(window) = web_sys::window() {
                                if let Ok(history) = window.history() {
                                    let state: &JsValue = &JsValue::from(js_sys::Object::new());
                                    let _ = history.push_state_with_url(
                                        state,
                                        "",
                                        Some(&format!("/daily/{}", date_clone)),
                                    );
                                }
                            }
                        }
                        Err(e) => {
                            set_error.set(Some(format!("Failed to load document: {}", e)));
                            set_loading.set(false);
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

    // Navigate to previous/next day
    let go_prev = move |_| {
        if let Some(d) = parse_date(&selected_date.get()) {
            let prev = d - chrono::Duration::days(1);
            set_selected_date.set(format_date(prev));
        }
    };
    let go_next = move |_| {
        if let Some(d) = parse_date(&selected_date.get()) {
            let next = d + chrono::Duration::days(1);
            set_selected_date.set(format_date(next));
        }
    };
    let go_today = move |_| {
        set_selected_date.set(format_date(chrono::Utc::now().date_naive()));
    };

    view! {
        <div class="max-w-4xl mx-auto">
            <Breadcrumbs items={vec![
                BreadcrumbItem { label: "Daily Notes".into(), href: None },
            ]}/>

            // Header with date navigation
            <div class="mb-8">
                <div class="flex items-center justify-between mb-4">
                    <h1 class="text-2xl font-bold text-gray-900 dark:text-white">"Daily Notes"</h1>
                </div>
                <div class="flex items-center gap-3 bg-white dark:bg-gray-800 border-2 border-gray-900 dark:border-gray-100 p-3">
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

                // Date display
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

            // Content area
            <div class="bg-white dark:bg-gray-800 rounded-none shadow border-2 border-gray-900 dark:border-gray-100 min-h-[400px]">
                {move || {
                    if loading.get() {
                        view! {
                            <div class="flex items-center justify-center py-16">
                                <div class="flex items-center gap-3 text-gray-500 dark:text-gray-400">
                                    <div class="w-5 h-5 border-2 border-gray-400 border-t-transparent rounded-full animate-spin"></div>
                                    <span>"Loading daily note..."</span>
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
                    } else if let Some(id) = doc_id.get() {
                        let doc_content_val = doc_content.get();
                        let doc_title_val = doc_title.get();
                        let nav = navigate.clone();
                        let id_clone = id.clone();
                        view! {
                            <div class="p-6">
                                <div class="flex items-center justify-between mb-4">
                                    <h2 class="text-xl font-semibold text-gray-900 dark:text-white">{doc_title_val}</h2>
                                    <button
                                        class="px-3 py-1.5 text-sm bg-blue-600 text-white hover:bg-blue-700 transition-colors"
                                        on:click=move |_| {
                                            nav(&format!("/documents/{}/edit", id_clone), Default::default());
                                        }
                                    >
                                        "Edit"
                                    </button>
                                </div>
                                <div class="prose dark:prose-invert max-w-none">
                                    <pre class="whitespace-pre-wrap font-sans text-gray-900 dark:text-white bg-transparent p-0 m-0">{doc_content_val}</pre>
                                </div>
                            </div>
                        }.into_any()
                    } else {
                        view! {
                            <div class="flex items-center justify-center py-16 text-gray-400 dark:text-gray-500">
                                <p>"Select a date to view or create a daily note"</p>
                            </div>
                        }.into_any()
                    }
                }}
            </div>
        </div>
    }
}
