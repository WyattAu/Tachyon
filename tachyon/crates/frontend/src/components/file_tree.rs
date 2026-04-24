// File Tree Component
// Displays a navigable list of documents in the editor sidebar

use crate::api::ApiClient;
use leptos::prelude::*;
use wasm_bindgen_futures::spawn_local;

#[derive(Debug, Clone, PartialEq)]
pub struct FileTreeItem {
    pub id: String,
    pub title: String,
    pub updated_at: String,
    pub is_active: bool,
}

#[component]
pub fn FileTree(
    current_document_id: String,
    repository_id: Option<String>,
    on_select: Callback<String>,
) -> impl IntoView {
    let (files, set_files) = signal(Vec::<FileTreeItem>::new());
    let (loading, set_loading) = signal(true);
    let (filter_text, set_filter_text) = signal(String::new());

    // Fetch documents for the current repository/space
    let fetch_files = {
        let repo_id = repository_id.clone().unwrap_or_default();
        let current_id = current_document_id.clone();
        let set_f = set_files.clone();
        let set_l = set_loading.clone();
        move || {
            let api = ApiClient::default();
            let rid = repo_id.clone();
            let cid = current_id.clone();
            let set_f = set_f.clone();
            let set_l = set_l.clone();
            spawn_local(async move {
                set_l.set(true);
                let result = if rid.is_empty() {
                    // No repository filter — list all documents
                    api.list_documents(Some(1), Some(100)).await
                } else {
                    // Filter by repository/project
                    api.list_documents_by_project(&rid, Some(1), Some(100)).await
                };
                match result {
                    Ok(resp) => {
                        let items: Vec<FileTreeItem> = resp.results
                            .into_iter()
                            .map(|doc| {
                                let doc_id = doc.id.clone();
                                let is_active = doc_id == cid;
                                FileTreeItem {
                                    id: doc_id,
                                    title: if doc.title.is_empty() { "Untitled".to_string() } else { doc.title },
                                    updated_at: doc.updated_at,
                                    is_active,
                                }
                            })
                            .collect();
                        set_f.set(items);
                    }
                    Err(_) => {
                        set_f.set(Vec::new());
                    }
                }
                set_l.set(false);
            });
        }
    };

    // Fetch on mount
    Effect::new(move |_| {
        fetch_files();
    });

    // Filtered file list
    let filtered_files = move || {
        let filter = filter_text.get().to_lowercase();
        let all = files.get();
        if filter.is_empty() {
            all
        } else {
            all.into_iter()
                .filter(|f| f.title.to_lowercase().contains(&filter))
                .collect::<Vec<_>>()
        }
    };

    // Sort: active first, then by updated_at descending
    let sorted_files = move || {
        let mut items = filtered_files();
        items.sort_by(|a, b| {
            match (a.is_active, b.is_active) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => b.updated_at.cmp(&a.updated_at),
            }
        });
        items
    };

    let on_select_ref = on_select.clone();

    view! {
        <div>
            // Search/filter input
            <div class="mb-3">
                <div class="relative">
                    <svg class="absolute left-2.5 top-2.5 h-3.5 w-3.5 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
                    </svg>
                    <input
                        type="text"
                        placeholder="Filter documents..."
                        class="w-full pl-8 pr-3 py-1.5 text-xs bg-gray-50 dark:bg-gray-900 border border-gray-200 dark:border-gray-700 rounded focus:outline-none focus:ring-1 focus:ring-blue-500 focus:border-blue-500 text-gray-900 dark:text-gray-100 placeholder-gray-400"
                        prop:value={move || filter_text.get()}
                        on:input=move |ev| {
                            let val = event_target_value(&ev);
                            set_filter_text.set(val);
                        }
                    />
                </div>
            </div>

            // File list
            <div class="space-y-0.5">
                {move || if loading.get() {
                    view! {
                        <div class="flex items-center justify-center py-6">
                            <div class="w-4 h-4 border-2 border-gray-400 border-t-transparent rounded-full animate-spin"></div>
                        </div>
                    }.into_any()
                } else {
                    let items = sorted_files();
                    if items.is_empty() {
                        view! {
                            <p class="text-xs text-gray-500 dark:text-gray-400 py-4 text-center">
                                "No documents found"
                            </p>
                        }.into_any()
                    } else {
                        view! {
                            <div class="space-y-0.5">
                                {items.into_iter().map(|item| {
                                    let item_id = item.id.clone();
                                    let item_title = item.title.clone();
                                    let is_active = item.is_active;
                                    let updated = item.updated_at.split('T').next().unwrap_or("").to_string();
                                    let on_click = on_select_ref.clone();
                                    view! {
                                        <button
                                            class={
                                                let active = is_active;
                                                move || if active {
                                                    "w-full text-left px-2 py-1.5 rounded text-xs bg-blue-50 dark:bg-blue-900/30 text-blue-700 dark:text-blue-300 font-medium transition-colors"
                                                } else {
                                                    "w-full text-left px-2 py-1.5 rounded text-xs text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors"
                                                }
                                            }
                                            on:click=move |_| on_click.run(item_id.clone())
                                        >
                                            <div class="flex items-center gap-1.5">
                                                // Document icon
                                                <svg class="w-3.5 h-3.5 flex-shrink-0 opacity-60" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
                                                </svg>
                                                <span class="truncate flex-1">{item_title}</span>
                                            </div>
                                            // Updated date
                                            <div class="text-[10px] text-gray-400 dark:text-gray-500 mt-0.5 ml-5">
                                                {updated}
                                            </div>
                                        </button>
                                    }
                                }).collect::<Vec<_>>()}
                            </div>
                        }.into_any()
                    }
                }}
            </div>
        </div>
    }
}
