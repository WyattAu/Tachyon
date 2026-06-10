#![allow(clippy::redundant_locals)]
// Documents Pages

use crate::api::ApiClient;
use crate::components::{
    Activity, ActivityFeed, BreadcrumbItem, Breadcrumbs, ConflictResolver, EditorSearch,
    EditorToolbar, EmptyDocuments, FocusTrap, MarkdownPreview, NativeEditor, ReviewPanel,
    TableOfContents, TemplateSelector, VersionHistory,
};
use crate::storage::sync::SyncEngine;
use crate::storage::{
    BrowserStore, LocalDocument, StoredDocument, SyncState, SyncStatus, stored_to_document,
};
use crate::types::{BacklinksResponse, Document, DocumentListResponse, DocumentTemplate};
use leptos::prelude::*;
use leptos_router::hooks::{use_navigate, use_params};
use leptos_router::params::Params;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ViewMode {
    Grid,
    List,
}

/// Documents list page
#[component]
pub fn DocumentsPage() -> impl IntoView {
    let api_client = ApiClient::default();
    let current_page = RwSignal::new(1usize);
    let page_size = 20usize;
    let view_mode = RwSignal::new(ViewMode::Grid);
    let search_text = RwSignal::new(String::new());
    let status_filter = RwSignal::new(String::new());

    let store = use_context::<BrowserStore>().unwrap_or_default();
    let sync_engine = use_context::<SyncEngine>();
    let sync_state = sync_engine.as_ref().map(|e| e.get_sync_state());

    let (show_create_modal, set_show_create_modal) = signal(false);
    let (new_doc_title, set_new_doc_title) = signal(String::new());
    let (creating, set_creating) = signal(false);
    let (create_error, set_create_error) = signal::<Option<String>>(None);

    let api_client_for_create = api_client.clone();
    let store_for_create = store.clone();

    Effect::new(move |_| {
        if let Some(ref engine) = sync_engine {
            engine.trigger_sync();
        }
    });

    let store_for_resource = store.clone();
    let documents_resource = LocalResource::new(move || {
        let client = api_client.clone();
        let page = current_page.get();
        let s = store_for_resource.clone();
        async move {
            match client.list_documents(Some(page), Some(page_size)).await {
                Ok(response) => {
                    for doc in &response.results {
                        let stored = StoredDocument {
                            document: LocalDocument::from(doc.clone()),
                            sync_status: SyncStatus::Synced,
                            local_version: 1,
                            server_version: Some(1),
                            last_modified: chrono::Utc::now().to_rfc3339(),
                        };
                        s.put(stored);
                    }
                    response
                }
                Err(_) => {
                    let local = s.get_all();
                    let total = local.len();
                    let docs: Vec<Document> = local.iter().map(stored_to_document).collect();
                    DocumentListResponse {
                        results: docs,
                        total,
                        page: 1,
                        page_size: 20,
                    }
                }
            }
        }
    });

    let total_pages = move || {
        documents_resource
            .get()
            .map(|d| {
                if d.page_size > 0 {
                    (d.total as f64 / d.page_size as f64).ceil() as usize
                } else {
                    1
                }
            })
            .unwrap_or(1)
    };

    let filtered_docs = move || {
        let response = documents_resource.get().unwrap_or(DocumentListResponse {
            results: vec![],
            total: 0,
            page: 1,
            page_size: 20,
        });
        let search = search_text.get().to_lowercase();
        let status = status_filter.get();
        let filtered: Vec<Document> = response
            .results
            .into_iter()
            .filter(|doc| {
                let matches_search = search.is_empty()
                    || doc.title.to_lowercase().contains(&search)
                    || doc.tags.iter().any(|t| t.to_lowercase().contains(&search));
                let matches_status = status.is_empty() || doc.status == status;
                matches_search && matches_status
            })
            .collect();
        filtered
    };

    let navigate = StoredValue::new(use_navigate());
    let handle_create_document = StoredValue::new({
        let api_client = api_client_for_create;
        move |_: leptos::ev::MouseEvent| {
            let title = new_doc_title.get();
            if title.trim().is_empty() {
                set_create_error.set(Some("Title is required".to_string()));
                return;
            }
            set_create_error.set(None);
            set_creating.set(true);
            let api = api_client.clone();
            let nav = navigate.get_value();
            let s = store_for_create.clone();
            spawn_local(async move {
                let body = serde_json::json!({ "title": title.trim(), "content": "", "tags": [] });
                match api.create_document(&body).await {
                    Ok(doc) => {
                        let stored = StoredDocument {
                            document: LocalDocument::from(doc.clone()),
                            sync_status: SyncStatus::Synced,
                            local_version: 1,
                            server_version: Some(1),
                            last_modified: chrono::Utc::now().to_rfc3339(),
                        };
                        s.put(stored);
                        let doc_id = doc.id.clone();
                        set_show_create_modal.set(false);
                        set_new_doc_title.set(String::new());
                        set_creating.set(false);
                        nav(&format!("/documents/{}/edit", doc_id), Default::default());
                    }
                    Err(e) => {
                        let local_id = uuid::Uuid::new_v4().to_string();
                        let now = chrono::Utc::now().to_rfc3339();
                        let local_doc = LocalDocument {
                            id: local_id,
                            title: title.trim().to_string(),
                            slug: None,
                            content: String::new(),
                            html: None,
                            status: "draft".to_string(),
                            visibility: "private".to_string(),
                            tags: vec![],
                            author_id: String::new(),
                            word_count: 0,
                            character_count: 0,
                            created_at: now.clone(),
                            updated_at: now,
                            published_at: None,
                            description: None,
                        };
                        let stored = StoredDocument {
                            document: local_doc,
                            sync_status: SyncStatus::PendingCreate,
                            local_version: 1,
                            server_version: None,
                            last_modified: chrono::Utc::now().to_rfc3339(),
                        };
                        s.put(stored);
                        set_create_error
                            .set(Some(format!("Failed to create: {}. Saved locally.", e)));
                        set_creating.set(false);
                    }
                }
            });
        }
    });

    let sync_badge = move || {
        let state = sync_state
            .as_ref()
            .map(|ss| ss.get())
            .unwrap_or(SyncState::Idle);
        match state {
            SyncState::Syncing => view! {
                <span class="flex items-center gap-1.5 text-xs text-blue-600 dark:text-blue-400 bg-blue-50 dark:bg-blue-900/20 px-2 py-0.5 rounded-full">
                    <span class="w-1.5 h-1.5 bg-blue-500 rounded-full animate-pulse"></span>"Syncing"
                </span>
            }.into_any(),
            SyncState::Offline => view! {
                <span class="flex items-center gap-1.5 text-xs text-yellow-600 dark:text-yellow-400 bg-yellow-50 dark:bg-yellow-900/20 px-2 py-0.5 rounded-full">
                    <span class="w-1.5 h-1.5 bg-yellow-500 rounded-full"></span>"Offline"
                </span>
            }.into_any(),
            SyncState::Error(_) => view! {
                <span class="flex items-center gap-1.5 text-xs text-red-600 dark:text-red-400 bg-red-50 dark:bg-red-900/20 px-2 py-0.5 rounded-full">
                    <span class="w-1.5 h-1.5 bg-red-500 rounded-full"></span>"Sync error"
                </span>
            }.into_any(),
            SyncState::Idle => view! {
                <span class="flex items-center gap-1.5 text-xs text-green-600 dark:text-green-400 bg-green-50 dark:bg-green-900/20 px-2 py-0.5 rounded-full">
                    <span class="w-1.5 h-1.5 bg-green-500 rounded-full"></span>"Synced"
                </span>
            }.into_any(),
        }
    };

    view! {
        <div>
            <div class="flex items-center justify-between mb-4">
                <div class="flex items-center gap-3">
                    <h1 class="text-2xl font-bold text-gray-900 dark:text-white">"Documents"</h1>
                    {sync_badge}
                </div>
                <button class="px-4 py-2 bg-blue-600 text-white rounded-none hover:bg-blue-700 transition-colors"
                    on:click=move |_| set_show_create_modal.set(true)>"+ New Document"</button>
            </div>

            <div class="flex flex-col sm:flex-row gap-3 mb-4">
                <div class="flex-1 relative">
                    <svg class="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-gray-400" aria-hidden="true" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
                    </svg>
                    <input type="text" placeholder="Search documents..."
                        class="w-full pl-10 pr-4 py-2 border border-gray-300 dark:border-gray-600 rounded-none bg-white dark:bg-gray-800 text-gray-900 dark:text-white focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                        prop:value={move || search_text.get()}
                        on:input=move |ev| search_text.set(event_target_value(&ev)) />
                </div>
                <select class="px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-none bg-white dark:bg-gray-800 text-gray-900 dark:text-white"
                    on:change=move |ev| status_filter.set(event_target_value(&ev))>
                    <option value="">"All Status"</option>
                    <option value="draft">"Draft"</option>
                    <option value="published">"Published"</option>
                    <option value="archived">"Archived"</option>
                </select>
                <div class="flex border border-gray-300 dark:border-gray-600 rounded-none overflow-hidden">
                    <button class={move || if view_mode.get() == ViewMode::Grid { "px-3 py-2 bg-blue-600 text-white" } else { "px-3 py-2 bg-white dark:bg-gray-800 text-gray-700 dark:text-gray-300 hover:bg-gray-50 dark:hover:bg-gray-700" }}
                        on:click=move |_| view_mode.set(ViewMode::Grid)
                        aria-label="Grid view"
                        aria-pressed=move || if view_mode.get() == ViewMode::Grid { "true" } else { "false" }
                    >
                        <svg class="h-4 w-4" aria-hidden="true" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2H6a2 2 0 01-2-2V6zm10 0a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2h-2a2 2 0 01-2-2V6zM4 16a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2H6a2 2 0 01-2-2v-2zm10 0a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2h-2a2 2 0 01-2-2v-2z" /></svg>
                    </button>
                    <button class={move || if view_mode.get() == ViewMode::List { "px-3 py-2 bg-blue-600 text-white" } else { "px-3 py-2 bg-white dark:bg-gray-800 text-gray-700 dark:text-gray-300 hover:bg-gray-50 dark:hover:bg-gray-700" }}
                        on:click=move |_| view_mode.set(ViewMode::List)
                        aria-label="List view"
                        aria-pressed=move || if view_mode.get() == ViewMode::List { "true" } else { "false" }
                    >
                        <svg class="h-4 w-4" aria-hidden="true" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6h16M4 10h16M4 14h16M4 18h16" /></svg>
                    </button>
                </div>
            </div>

            <Suspense fallback={view! { <DocumentsGridSkeleton /> }}>
                {move || {
                    let docs = filtered_docs();
                    if docs.is_empty() {
                        view! { <EmptyDocuments /> }.into_any()
                    } else {
                        let mode = view_mode.get();
                        view! {
                            <div>
                                <div class="text-sm text-gray-500 dark:text-gray-400 mb-4">
                                    {format!("{} documents", docs.len())}
                                </div>
                                {if mode == ViewMode::Grid {
                                    view! {
                                        <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
                                            {docs.into_iter().map(|doc| view! { <DocumentCard document={doc} /> }).collect::<Vec<_>>()}
                                        </div>
                                    }.into_any()
                                } else {
                                    view! {
                                        <div class="bg-white dark:bg-gray-800 rounded-none shadow border-2 border-gray-900 dark:border-gray-100 divide-y divide-gray-200 dark:divide-gray-700">
                                            {docs.into_iter().map(|doc| view! { <DocumentRow document={doc} /> }).collect::<Vec<_>>()}
                                        </div>
                                    }.into_any()
                                }}
                            </div>
                        }.into_any()
                    }
                }}
            </Suspense>

            <div class="flex items-center justify-center gap-2 mt-6">
                <button class="px-3 py-2 text-sm bg-white dark:bg-gray-800 border border-gray-300 dark:border-gray-600 rounded-none hover:bg-gray-50 dark:hover:bg-gray-700 disabled:opacity-50 disabled:cursor-not-allowed text-gray-700 dark:text-gray-300"
                    disabled={move || current_page.get() <= 1}
                    on:click=move |_| { let c = current_page.get(); if c > 1 { current_page.set(c - 1); } }>"Previous"</button>
                <span class="px-3 py-2 text-sm text-gray-700 dark:text-gray-300">{move || format!("Page {} of {}", current_page.get(), total_pages())}</span>
                <button class="px-3 py-2 text-sm bg-white dark:bg-gray-800 border border-gray-300 dark:border-gray-600 rounded-none hover:bg-gray-50 dark:hover:bg-gray-700 disabled:opacity-50 disabled:cursor-not-allowed text-gray-700 dark:text-gray-300"
                    disabled={move || current_page.get() >= total_pages()}
                    on:click=move |_| { let c = current_page.get(); if c < total_pages() { current_page.set(c + 1); } }>"Next"</button>
            </div>

            {move || if show_create_modal.get() {
                Some(view! {
                    <div class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
                        <FocusTrap active={show_create_modal.into()}>
                            <div class="bg-white dark:bg-gray-800 rounded-none p-6 w-full max-w-md" role="dialog" aria-modal="true" aria-labelledby="new-doc-title">
                                <h2 id="new-doc-title" class="text-xl font-bold text-gray-900 dark:text-white mb-4">"New Document"</h2>
                                {move || create_error.get().map(|e| view! {
                                    <div class="mb-4 p-3 bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded text-sm text-red-700 dark:text-red-300">{e}</div>
                                })}
                                <div class="space-y-4">
                                    <div>
                                        <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">"Start from template"</label>
                                        <TemplateSelector on_select={Callback::new(move |t: DocumentTemplate| set_new_doc_title.set(t.name))} category={None} />
                                    </div>
                                    <div>
                                        <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">"Title"</label>
                                        <input type="text" class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-none focus:outline-none focus:ring-2 focus:ring-blue-500 dark:bg-gray-700 dark:text-white"
                                            placeholder="Enter document title" prop:value={move || new_doc_title.get()}
                                            on:input=move |ev| set_new_doc_title.set(event_target_value(&ev)) />
                                    </div>
                                </div>
                                <div class="mt-6 flex justify-end gap-3">
                                    <button class="px-4 py-2 text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-none transition-colors"
                                        on:click=move |_| { set_show_create_modal.set(false); set_new_doc_title.set(String::new()); set_create_error.set(None); }>"Cancel"</button>
                                    <button class="px-4 py-2 bg-blue-600 text-white rounded-none hover:bg-blue-700 transition-colors disabled:opacity-50"
                                        disabled={move || creating.get()}
                                        on:click=move |ev| handle_create_document.get_value()(ev)>
                                        {move || if creating.get() { "Creating..." } else { "Create" }}
                                    </button>
                                </div>
                            </div>
                        </FocusTrap>
                    </div>
                })
            } else { None }}
        </div>
    }
}

#[component]
fn DocumentRow(document: Document) -> impl IntoView {
    let navigate = use_navigate();
    let doc_id = document.id.clone();
    let title = document.title;
    let status = document.status.clone();
    let visibility = document.visibility.clone();
    let word_count = document.word_count;
    let tags = document.tags;
    let updated = document
        .updated_at
        .split('T')
        .next()
        .unwrap_or("Unknown")
        .to_string();

    let status_class = match status.as_str() {
        "published" => "bg-green-100 dark:bg-green-900 text-green-600 dark:text-green-300",
        "archived" => "bg-gray-100 dark:bg-gray-700 text-gray-600 dark:text-gray-300",
        _ => "bg-yellow-100 dark:bg-yellow-900 text-yellow-600 dark:text-yellow-300",
    };
    let status_text = match status.as_str() {
        "published" => "Published",
        "archived" => "Archived",
        _ => "Draft",
    };
    let vis_text = match visibility.as_str() {
        "public" => "Public",
        "restricted" => "Restricted",
        _ => "Private",
    };

    let wc_text = if word_count == 1 {
        "1 word".to_string()
    } else {
        format!("{} words", word_count)
    };

    view! {
        <div class="flex items-center gap-4 px-4 py-3 hover:bg-gray-50 dark:hover:bg-gray-700/50 transition-colors cursor-pointer"
            on:click=move |_| navigate(&format!("/documents/{}/edit", doc_id), Default::default())>
            <div class="flex-1 min-w-0">
                <div class="flex items-center gap-2">
                    <h3 class="text-sm font-medium text-gray-900 dark:text-white truncate">{title}</h3>
                    <span class={format!("px-2 py-0.5 text-xs rounded {}", status_class)}>{status_text}</span>
                    <span class="px-2 py-0.5 text-xs rounded bg-gray-100 dark:bg-gray-700 text-gray-600 dark:text-gray-300">{vis_text}</span>
                </div>
                <div class="flex items-center gap-3 mt-1">
                    <span class="text-xs text-gray-500 dark:text-gray-400">{wc_text}</span>
                    <span class="text-xs text-gray-500 dark:text-gray-400">{updated}</span>
                    {tags.into_iter().take(3).map(|tag| {
                        view! { <span class="px-2 py-0.5 text-xs bg-purple-100 dark:bg-purple-900 text-purple-600 dark:text-purple-300 rounded">{tag}</span> }
                    }).collect::<Vec<_>>()}
                </div>
            </div>
            <svg class="h-4 w-4 text-gray-400 flex-shrink-0" aria-hidden="true" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7" />
            </svg>
        </div>
    }
}

/// Documents grid skeleton for loading state
#[component]
fn DocumentsGridSkeleton() -> impl IntoView {
    view! {
        <div>
            <div class="h-4 bg-gray-200 dark:bg-gray-700 rounded w-48 mb-4 animate-pulse"></div>
            <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
                {(0..6).map(|_| {
                    view! {
                        <div class="bg-white dark:bg-gray-800 rounded-none shadow border-2 border-gray-900 dark:border-gray-100 p-6 animate-pulse">
                            <div class="h-5 bg-gray-200 dark:bg-gray-700 rounded w-3/4"></div>
                            <div class="h-3 bg-gray-200 dark:bg-gray-700 rounded w-1/2 mt-2"></div>
                            <div class="h-4 bg-gray-200 dark:bg-gray-700 rounded w-full mt-4"></div>
                            <div class="h-4 bg-gray-200 dark:bg-gray-700 rounded w-5/6 mt-2"></div>
                            <div class="flex gap-2 mt-4">
                                <div class="h-5 bg-gray-200 dark:bg-gray-700 rounded w-12"></div>
                                <div class="h-5 bg-gray-200 dark:bg-gray-700 rounded w-16"></div>
                            </div>
                            <div class="flex justify-between mt-4">
                                <div class="h-3 bg-gray-200 dark:bg-gray-700 rounded w-20"></div>
                                <div class="h-3 bg-gray-200 dark:bg-gray-700 rounded w-20"></div>
                            </div>
                        </div>
                    }
                }).collect::<Vec<_>>()}
            </div>
        </div>
    }
}

/// Document card component
#[component]
fn DocumentCard(document: Document) -> impl IntoView {
    let title = document.title.clone();
    let tags = document.tags.clone();
    let word_count = document.word_count;
    let created_date = document
        .created_at
        .split('T')
        .next()
        .unwrap_or("Unknown")
        .to_string();
    let doc_id = document.id.clone();

    let status_class = match document.status.as_str() {
        "published" => "bg-green-100 dark:bg-green-900 text-green-600 dark:text-green-300",
        "archived" => "bg-gray-100 dark:bg-gray-700 text-gray-600 dark:text-gray-300",
        "deleted" => "bg-red-100 dark:bg-red-900 text-red-600 dark:text-red-300",
        _ => "bg-yellow-100 dark:bg-yellow-900 text-yellow-600 dark:text-yellow-300",
    };

    let visibility_class = match document.visibility.as_str() {
        "public" => "bg-blue-100 dark:bg-blue-900 text-blue-600 dark:text-blue-300",
        "restricted" => "bg-orange-100 dark:bg-orange-900 text-orange-600 dark:text-orange-300",
        _ => "bg-gray-100 dark:bg-gray-700 text-gray-600 dark:text-gray-300",
    };

    let status_text = match document.status.as_str() {
        "published" => "Published",
        "archived" => "Archived",
        "deleted" => "Deleted",
        _ => "Draft",
    };

    let visibility_text = match document.visibility.as_str() {
        "public" => "Public",
        "restricted" => "Restricted",
        _ => "Private",
    };

    let word_count_text = if word_count == 1 {
        "1 word".to_string()
    } else {
        format!("{} words", word_count)
    };

    let navigate = use_navigate();
    let doc_id_for_click = doc_id.clone();
    let on_click = move |_| {
        navigate(
            &format!("/documents/{}/edit", doc_id_for_click),
            Default::default(),
        );
    };

    view! {
        <div
            class="bg-white dark:bg-gray-800 rounded-none shadow border-2 border-gray-900 dark:border-gray-100 p-6 hover:border-blue-500 transition-colors cursor-pointer"
            on:click={on_click}
        >
            <div class="flex items-start justify-between mb-2">
                <h3 class="text-lg font-semibold text-gray-900 dark:text-white line-clamp-1">{title}</h3>
            </div>

            <div class="flex gap-2 mb-3">
                <span class={format!("px-2 py-0.5 text-xs rounded {}", status_class)}>{status_text}</span>
                <span class={format!("px-2 py-0.5 text-xs rounded {}", visibility_class)}>{visibility_text}</span>
            </div>

            {if !tags.is_empty() {
                view! {
                    <div class="flex flex-wrap gap-1.5 mb-3">
                        {tags.into_iter().take(3).map(|tag| {
                            view! {
                                <span class="px-2 py-0.5 text-xs bg-purple-100 dark:bg-purple-900 text-purple-600 dark:text-purple-300 rounded">
                                    {tag}
                                </span>
                            }
                        }).collect::<Vec<_>>()}
                    </div>
                }.into_any()
            } else {
                view! { <div class="mb-3"></div> }.into_any()
            }}

            <div class="flex items-center justify-between text-xs text-gray-500 dark:text-gray-400">
                <span>{word_count_text}</span>
                <span>{created_date}</span>
            </div>
        </div>
    }
}

/// Single document page
#[component]
pub fn DocumentPage() -> impl IntoView {
    let params = use_params::<DocumentViewParams>();
    let document_id = move || {
        params
            .try_with(|p| p.as_ref().map(|p| p.id.clone()).unwrap_or_default())
            .unwrap_or_default()
    };

    let api_client = ApiClient::default();
    let doc_id = document_id();
    let (loading, set_loading) = signal(true);
    let (load_error, set_load_error) = signal::<Option<String>>(None);
    let (doc, set_doc) = signal::<Option<Document>>(None);

    let set_ld = set_loading;
    let set_le = set_load_error;
    let set_d = set_doc;

    Effect::new(move || {
        let did = doc_id.clone();
        if did.is_empty() {
            set_ld.set(false);
            return;
        }
        set_ld.set(true);
        set_le.set(None);
        let api = api_client.clone();
        wasm_bindgen_futures::spawn_local(async move {
            match api.get_document(&did).await {
                Ok(document) => {
                    set_d.set(Some(document));
                    set_ld.set(false);
                }
                Err(e) => {
                    set_le.set(Some(format!("Failed to load document: {}", e)));
                    set_ld.set(false);
                }
            }
        });
    });

    let navigate = use_navigate();
    let doc_id_for_edit = document_id();

    view! {
        <div class="max-w-4xl mx-auto">
            <Breadcrumbs items={vec![
                BreadcrumbItem { label: "Documents".into(), href: Some("/documents".into()) },
            ]}/>
            // Back link
            <a href="/documents" class="inline-flex items-center gap-1 text-sm text-blue-600 dark:text-blue-400 hover:underline mb-6">
                <svg class="w-4 h-4" aria-hidden="true" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 19l-7-7 7-7" />
                </svg>
                "Back to Documents"
            </a>

            {move || {
                let did = document_id();
                if did.is_empty() {
                    view! {
                        <div class="p-8 text-center">
                            <p class="text-gray-500 dark:text-gray-400">"No document ID specified"</p>
                        </div>
                    }.into_any()
                } else if loading.get() {
                    view! {
                        <div class="flex items-center justify-center py-12">
                            <div class="flex items-center gap-3 text-gray-500 dark:text-gray-400">
                                <div class="w-5 h-5 border-2 border-gray-400 border-t-transparent rounded-full animate-spin"></div>
                                <span>"Loading document..."</span>
                            </div>
                        </div>
                    }.into_any()
                } else if let Some(err) = load_error.get() {
                    view! {
                        <div class="p-4 bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded text-red-700 dark:text-red-300">
                            {err}
                        </div>
                    }.into_any()
                } else if let Some(document) = doc.get() {
                    let title = document.title.clone();
                    let tags = document.tags.clone();
                    let word_count = document.word_count;
                    let created_at = document.created_at.clone();
                    let updated_at = document.updated_at.clone();
                    let content = document.content.clone();
                    let status = document.status.clone();
                    let visibility = document.visibility.clone();
                    let edit_id = doc_id_for_edit.clone();

                    let status_class = match status.as_str() {
                        "published" => "bg-green-100 dark:bg-green-900 text-green-600 dark:text-green-300",
                        "archived" => "bg-gray-100 dark:bg-gray-700 text-gray-600 dark:text-gray-300",
                        "deleted" => "bg-red-100 dark:bg-red-900 text-red-600 dark:text-red-300",
                        _ => "bg-yellow-100 dark:bg-yellow-900 text-yellow-600 dark:text-yellow-300",
                    };
                    let status_text = match status.as_str() {
                        "published" => "Published",
                        "archived" => "Archived",
                        "deleted" => "Deleted",
                        _ => "Draft",
                    };
                    let visibility_text = match visibility.as_str() {
                        "public" => "Public",
                        "restricted" => "Restricted",
                        _ => "Private",
                    };
                    let word_count_text = if word_count == 1 { "1 word".to_string() } else { format!("{} words", word_count) };

                    let nav = navigate.clone();
                    let edit_id_clone = edit_id.clone();
                    let on_edit = Callback::new(move |_: leptos::ev::MouseEvent| {
                        nav(&format!("/documents/{}/edit", edit_id_clone), Default::default());
                    });

                    view! {
                        <div>
                            // Title and actions
                            <div class="flex items-start justify-between mb-6">
                                <div>
                                    <h1 class="text-3xl font-bold text-gray-900 dark:text-white mb-2">{title}</h1>
                                    <div class="flex gap-2">
                                        <span class={format!("px-2 py-0.5 text-xs rounded {}", status_class)}>{status_text}</span>
                                        <span class="px-2 py-0.5 text-xs rounded bg-gray-100 dark:bg-gray-700 text-gray-600 dark:text-gray-300">{visibility_text}</span>
                                    </div>
                                </div>
                                <button
                                    on:click={move |ev| on_edit.run(ev)}
                                    class="px-4 py-2 bg-blue-600 text-white rounded-none hover:bg-blue-700 transition-colors flex items-center gap-2"
                                >
                                    <svg class="w-4 h-4" aria-hidden="true" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z" />
                                    </svg>
                                    "Edit"
                                </button>
                            </div>

                            // Metadata
                            <div class="flex flex-wrap gap-4 mb-6 text-sm text-gray-500 dark:text-gray-400">
                                <span class="flex items-center gap-1">
                                    <svg class="w-4 h-4" aria-hidden="true" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 7V3m8 4V3m-9 8h10M5 21h14a2 2 0 002-2V7a2 2 0 00-2-2H5a2 2 0 00-2 2v12a2 2 0 002 2z" />
                                    </svg>
                                    "Created: "{created_at.split('T').next().unwrap_or("Unknown")}
                                </span>
                                <span class="flex items-center gap-1">
                                    <svg class="w-4 h-4" aria-hidden="true" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z" />
                                    </svg>
                                    "Updated: "{updated_at.split('T').next().unwrap_or("Unknown")}
                                </span>
                                <span class="flex items-center gap-1">
                                    <svg class="w-4 h-4" aria-hidden="true" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
                                    </svg>
                                    {word_count_text}
                                </span>
                            </div>

                            // Tags
                            {if !tags.is_empty() {
                                view! {
                                    <div class="flex flex-wrap gap-2 mb-6">
                                        {tags.into_iter().map(|tag| {
                                            view! {
                                                <span class="px-2 py-0.5 text-xs bg-purple-100 dark:bg-purple-900 text-purple-600 dark:text-purple-300 rounded">
                                                    {tag}
                                                </span>
                                            }
                                        }).collect::<Vec<_>>()}
                                    </div>
                                }.into_any()
                            } else {
                                view! { <div class="mb-6"></div> }.into_any()
                            }}

                            // Content
                            <div class="bg-white dark:bg-gray-800 rounded-none shadow border-2 border-gray-900 dark:border-gray-100 p-6">
                                <div class="prose dark:prose-invert max-w-none">
                                    <pre class="whitespace-pre-wrap font-sans text-gray-900 dark:text-white bg-transparent p-0 m-0">{content}</pre>
                                </div>
                            </div>
                        </div>
                    }.into_any()
                } else {
                    view! {
                        <div class="p-8 text-center">
                            <p class="text-gray-500 dark:text-gray-400">"Document not found"</p>
                        </div>
                    }.into_any()
                }
            }}
        </div>
    }
}

#[derive(Params, PartialEq, Clone)]
struct DocumentViewParams {
    id: String,
}

/// Document edit page with native editor
#[component]
pub fn DocumentEditPage() -> impl IntoView {
    let params = use_params::<DocumentEditParams>();
    let document_id = move || {
        params
            .try_with(|p| p.as_ref().map(|p| p.id.clone()).unwrap_or_default())
            .unwrap_or_default()
    };

    let _user_id = "user-".to_string() + &uuid::Uuid::new_v4().to_string()[..8];
    let _user_name = "User".to_string();

    // Fetch document content on mount
    let (doc_content, set_doc_content) = signal(String::new());
    let (doc_title, set_doc_title) = signal(String::new());
    let (loading, set_loading) = signal(true);
    let (load_error, set_load_error) = signal::<Option<String>>(None);
    let (sidebar_tab, set_sidebar_tab) = signal("activity".to_string());
    let (sidebar_open, set_sidebar_open) = signal(true);
    let show_search = RwSignal::new(false);
    let (show_preview, set_show_preview) = signal(false);
    let (is_saving, set_is_saving) = signal(false);
    let (last_saved, set_last_saved) = signal::<Option<String>>(None);
    let (dirty, set_dirty) = signal(false);

    let api_client = ApiClient::default();
    let set_dc = set_doc_content;
    let set_dt = set_doc_title;
    let set_ld = set_loading;
    let set_le = set_load_error;

    Effect::new(move || {
        let did = document_id();
        if did.is_empty() {
            set_ld.set(false);
            return;
        }
        set_ld.set(true);
        set_le.set(None);
        let api = api_client.clone();
        spawn_local(async move {
            match api.get_document(&did).await {
                Ok(doc) => {
                    set_dc.set(doc.content);
                    set_dt.set(doc.title);
                    set_ld.set(false);
                }
                Err(e) => {
                    set_le.set(Some(format!("Failed to load document: {}", e)));
                    set_ld.set(false);
                }
            }
        });
    });

    let (activities, _set_activities) = signal(Vec::<Activity>::new());

    let document_content = RwSignal::new(String::new());

    let on_editor_change = Callback::new(move |content: String| {
        document_content.set(content);
        set_dirty.set(true);
    });

    // Auto-save debounce via Effect
    {
        let auto_save_debounce: std::rc::Rc<std::cell::RefCell<Option<i32>>> =
            std::rc::Rc::new(std::cell::RefCell::new(None));

        Effect::new(move |_| {
            let content = document_content.get();
            if content.is_empty() {
                return;
            }

            let doc_id_val = document_id();
            if doc_id_val.is_empty() {
                return;
            }

            {
                let handle = *auto_save_debounce.borrow();
                if let Some(h) = handle {
                    let _ = web_sys::window().map(|w| {
                        w.clear_timeout_with_handle(h);
                    });
                }
            }

            let api = ApiClient::default();
            let did = doc_id_val;
            let set_is_saving = set_is_saving;
            let set_last_saved = set_last_saved;
            let set_dirty = set_dirty;
            let dh = auto_save_debounce.clone();

            let closure = wasm_bindgen::closure::Closure::<dyn Fn()>::new(move || {
                let api = api.clone();
                let did = did.clone();
                let content_val = document_content
                    .try_with_untracked(|c| c.clone())
                    .unwrap_or_default();
                let set_is_saving = set_is_saving;
                let set_last_saved = set_last_saved;
                let set_dirty = set_dirty;

                wasm_bindgen_futures::spawn_local(async move {
                    let body = serde_json::json!({ "content": content_val });
                    match api.update_document(&did, &body).await {
                        Ok(_) => {
                            let now = chrono::Utc::now().format("%H:%M:%S").to_string();
                            set_is_saving.set(false);
                            set_last_saved.set(Some(format!("Auto-saved {}", now)));
                            set_dirty.set(false);
                        }
                        Err(_) => {
                            set_is_saving.set(false);
                        }
                    }
                });
            });

            let timeout = web_sys::window()
                .and_then(|w| {
                    w.set_timeout_with_callback_and_timeout_and_arguments_0(
                        closure.as_ref().unchecked_ref(),
                        3000,
                    )
                    .ok()
                })
                .unwrap_or(0);

            *dh.borrow_mut() = Some(timeout);
            closure.forget();
        });
    }

    // Manual save
    let manual_save = Callback::new(move |_: ()| {
        let doc_id_val = document_id();
        if doc_id_val.is_empty() {
            return;
        }

        let api = ApiClient::default();
        let did = doc_id_val;
        let set_is_saving = set_is_saving;
        let set_last_saved = set_last_saved;
        let set_dirty = set_dirty;

        wasm_bindgen_futures::spawn_local(async move {
            set_is_saving.set(true);
            let content = document_content.get_untracked();
            let body = serde_json::json!({ "content": content });
            match api.update_document(&did, &body).await {
                Ok(_) => {
                    let now = chrono::Utc::now().format("%H:%M:%S").to_string();
                    set_is_saving.set(false);
                    set_last_saved.set(Some(now));
                    set_dirty.set(false);
                }
                Err(_) => {
                    set_is_saving.set(false);
                }
            }
        });
    });

    // Ctrl+S handler
    {
        let save_fn = manual_save;
        if let Some(window) = web_sys::window() {
            let closure = wasm_bindgen::closure::Closure::<dyn Fn(web_sys::KeyboardEvent)>::new(
                move |e: web_sys::KeyboardEvent| {
                    if (e.ctrl_key() || e.meta_key()) && e.key() == "s" {
                        e.prevent_default();
                        save_fn.run(());
                    }
                },
            );
            let _ = window
                .add_event_listener_with_callback("keydown", closure.as_ref().unchecked_ref());
            closure.forget();
        }
    }

    // Editor signal for sharing between toolbar and search
    let editor = RwSignal::new(tachyon_editor::Editor::with_content(""));
    // Render trigger: bump to force NativeEditor to re-render after content sync
    let render_trigger = RwSignal::new(0u64);

    // CRDT collaboration sync via WebSocket.
    // DISABLED: The tachyon-server does not have a WebSocket endpoint at /ws.
    // Re-enable once the server supports real-time collaboration.
    // CRDT collaboration disabled: tachyon-server has no /ws endpoint yet.
    // Re-enable the WebSocket sync once the server supports real-time collaboration.
    {
        let _id = document_id();
        // Effect::new(move || { let _ = document_id(); }); // placeholder
    }

    view! {
        <div class="flex" style="height: calc(100vh - 4rem);">
            <div class="flex-1 flex flex-col overflow-hidden min-w-0">
                <div class="p-4 border-b border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-800 flex items-center gap-3 no-print toolbar">
                    <h1 class="text-xl font-semibold text-gray-900 dark:text-white min-w-0 truncate">
                        {move || {
                            let title = doc_title.get();
                            if title.is_empty() {
                                format!("Editing: {}", document_id())
                            } else {
                                title.to_string()
                            }
                        }}
                    </h1>
                    <button
                        class="hidden md:flex p-1.5 rounded border border-gray-300 dark:border-gray-600 hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors flex-shrink-0"
                        on:click=move |_| set_sidebar_open.update(|o| *o = !*o)
                        title={move || if sidebar_open.get() { "Hide sidebar" } else { "Show sidebar" }}
                        aria-label={move || if sidebar_open.get() { "Hide sidebar" } else { "Show sidebar" }}
                    >
                        <svg class="w-4 h-4 text-gray-500 dark:text-gray-400" aria-hidden="true" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 5l7 7-7 7M5 5l7 7-7 7" />
                        </svg>
                    </button>
                </div>

                <div class="flex-1 overflow-hidden min-h-0 flex flex-col">
                    {move || {
                        let doc_id = document_id();
                        let loading_val = loading.get();
                        let error_val = load_error.get();
                        if doc_id.is_empty() {
                            view! {
                                <div class="flex items-center justify-center h-full">
                                    <p class="text-gray-500 dark:text-gray-400">No document selected</p>
                                </div>
                            }.into_any()
                        } else if loading_val {
                            view! {
                                <div class="flex items-center justify-center h-full">
                                    <div class="flex items-center gap-3 text-gray-500 dark:text-gray-400">
                                        <div class="w-5 h-5 border-2 border-gray-400 border-t-transparent rounded-full animate-spin"></div>
                                        <span>"Loading..."</span>
                                    </div>
                                </div>
                            }.into_any()
                        } else if let Some(err) = error_val {
                            view! {
                                <div class="flex items-center justify-center h-full">
                                    <div class="p-4 bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded text-red-700 dark:text-red-300">
                                        {err}
                                    </div>
                                </div>
                            }.into_any()
                         } else {
                            let content = doc_content.get();
                            // Sync loaded content to shared editor (only if different)
                            let _ = editor.try_update(|e| {
                                if !content.is_empty() && e.content() != content {
                                    e.set_content(&content);
                                }
                            });
                            // Force NativeEditor to re-render with the new content
                            render_trigger.update(|t| *t += 1);
                            let ed = editor;
                            let on_change = on_editor_change;
                            let on_save = manual_save;
                            let on_search = Callback::new(move |_: ()| show_search.update(|s| *s = !*s));
                            let on_preview = Callback::new(move |_: ()| set_show_preview.update(|s| *s = !*s));

                            view! {
                                <div class="flex flex-col min-h-0 flex-1">
                                    <EditorToolbar
                                        editor={ed}
                                        on_save={on_save}
                                        on_preview={on_preview}
                                        on_search={on_search}
                                    />

                                     <div class="flex-1 overflow-auto relative flex min-h-0">
                                         <div class={move || if show_preview.get() { "flex-1 overflow-hidden border-r border-gray-200 dark:border-gray-700 min-h-[30vh] sm:min-h-[200px]" } else { "flex-1 overflow-hidden min-h-[30vh] sm:min-h-[200px]" }}>
                                             <NativeEditor
                                                 editor={ed}
                                                 document_id={doc_id}
                                                 editable=true
                                                 placeholder="Start writing markdown...".into()
                                                 on_change={on_change}
                                                 render_trigger={render_trigger}
                                             />
                                            <EditorSearch
                                                editor={ed}
                                                show={show_search}
                                            />
                                        </div>
                                        <Show when=move || show_preview.get()>
                <div class="flex-1 overflow-hidden min-h-0">
                                                <MarkdownPreview content={document_content.get()} render_toc=true />
                                            </div>
                                        </Show>
                                    </div>

                                    // Status bar
                                    <div class="flex-shrink-0 flex items-center justify-between px-4 py-2 border-t border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-800 text-xs text-gray-500 dark:text-gray-400">
                                        <div class="flex items-center gap-3">
                                            <div
                                                class=move || if dirty.get() { "w-2 h-2 rounded-full bg-yellow-400" } else { "w-2 h-2 rounded-full bg-green-500" }
                                                title={move || if dirty.get() { "Unsaved changes" } else { "All changes saved" }}
                                            ></div>
                                            <span>{move || {
                                                let (wc, cc) = editor.try_with(|e| (e.word_count(), e.char_count())).unwrap_or((0, 0));
                                                format!("{} words \u{00b7} {} chars", wc, cc)
                                            }}</span>
                                        </div>
                                        <div>
                                            {move || {
                                                if is_saving.get() {
                                                    "Saving...".to_string()
                                                } else if let Some(time) = last_saved.get() {
                                                    format!("Last saved: {}", time)
                                                } else {
                                                    "Not saved".to_string()
                                                }
                                            }}
                                        </div>
                                    </div>
                                </div>
                            }.into_any()
                        }
                    }}
                </div>
            </div>

            <div
                class={move || if sidebar_open.get() {
                    "w-96 flex-shrink-0 border-l border-gray-200 dark:border-gray-700 transition-all duration-200 overflow-hidden hidden md:block flex flex-col"
                } else {
                    "w-0 flex-shrink-0 border-l border-gray-200 dark:border-gray-700 transition-all duration-200 overflow-hidden"
                }}
            >
                <div class="flex overflow-x-auto flex-shrink-0 border-b border-gray-200 dark:border-gray-700" role="tablist"
                    on:keydown=move |ev: web_sys::KeyboardEvent| {
                        let key = ev.key();
                        if key == "ArrowRight" || key == "ArrowLeft" {
                            ev.prevent_default();
                            let sidebar_tabs = ["activity", "history", "review", "conflicts", "backlinks", "outline"];
                            let current = sidebar_tab.get();
                            let current_idx = sidebar_tabs.iter().position(|&t| t == current).unwrap_or(0);
                            let new_idx = if key == "ArrowRight" {
                                (current_idx + 1) % sidebar_tabs.len()
                            } else {
                                current_idx.checked_sub(1).unwrap_or(sidebar_tabs.len() - 1)
                            };
                            let new_tab = sidebar_tabs[new_idx];
                            set_sidebar_tab.set(new_tab.to_string());
                            if let Some(window) = web_sys::window() {
                                if let Some(doc) = window.document() {
                                    let btn_id = format!("sidebar-tab-{}", new_tab);
                                    let _ = doc.get_element_by_id(&btn_id)
                                        .and_then(|el| el.dyn_into::<web_sys::HtmlElement>().ok())
                                        .and_then(|el| el.focus().ok());
                                }
                            }
                        }
                    }
                >
                    {["activity", "history", "review", "conflicts", "backlinks", "outline"].iter().map(|tab_name| {
                        let name = (*tab_name).to_string();
                        let name_for_active = name.clone();
                        let name_for_click = name.clone();
                        let name_for_onclick = name.clone();
                        let btn_id = format!("sidebar-tab-{}", tab_name);
                        let panel_id = format!("sidebar-panel-{}", tab_name);
                        let (label, full_label) = match *tab_name {
                            "activity" => ("Act", "Activity"),
                            "history" => ("Hist", "History"),
                            "review" => ("Rev", "Review"),
                            "conflicts" => ("Conf", "Conflicts"),
                            "backlinks" => ("Back", "Backlinks"),
                            "outline" => ("Out", "Outline"),
                            _ => ("Tab", "Tab"),
                        };
                        let full_label = full_label.to_string();
                        let is_active = move || sidebar_tab.get() == name_for_active;
                        let is_active_for_tabindex = move || sidebar_tab.get() == name_for_click;
                        view! {
                            <button
                                id={btn_id}
                                class="flex-shrink-0 px-1.5 py-2 text-xs font-medium border-b-2 transition-colors whitespace-nowrap"
                                role="tab"
                                title={full_label}
                                aria-selected=move || if is_active() { "true" } else { "false" }
                                aria-controls={panel_id.clone()}
                                tabindex=move || if is_active_for_tabindex() { 0 } else { -1 }
                                on:click=move |_| set_sidebar_tab.set(name_for_onclick.clone())
                            >
                                {label}
                            </button>
                        }
                    }).collect::<Vec<_>>()}
                </div>
                {move || {
                    let tab = sidebar_tab.get();
                    let doc_id = document_id();
                    if tab == "history" {
                        view! {
                            <div class="flex-1 overflow-y-auto" role="tabpanel" aria-labelledby="sidebar-tab-history">
                                <VersionHistory document_id={doc_id} on_rollback=None />
                            </div>
                        }.into_any()
                    } else if tab == "review" {
                        view! {
                            <div id="sidebar-panel-review" class="p-4 flex-1 overflow-y-auto" role="tabpanel" aria-labelledby="sidebar-tab-review">
                                <ReviewPanel document_id={doc_id} />
                            </div>
                        }.into_any()
                    } else if tab == "conflicts" {
                        view! {
                            <div id="sidebar-panel-conflicts" class="p-4 flex-1 overflow-y-auto" role="tabpanel" aria-labelledby="sidebar-tab-conflicts">
                                <ConflictResolver document_id={doc_id} />
                            </div>
                        }.into_any()
                    } else if tab == "backlinks" {
                        view! {
                            <div id="sidebar-panel-backlinks" class="p-4 flex-1 overflow-y-auto" role="tabpanel" aria-labelledby="sidebar-tab-backlinks">
                                <BacklinksPanel document_id={doc_id} />
                            </div>
                        }.into_any()
                    } else if tab == "outline" {
                        let content = doc_content.get();
                        view! {
                            <div id="sidebar-panel-outline" class="p-4 flex-1 overflow-y-auto" role="tabpanel" aria-labelledby="sidebar-tab-outline">
                                <TableOfContents markdown_content={content} />
                            </div>
                        }.into_any()
                    } else {
                        view! {
                            <div id="sidebar-panel-activity" class="p-4 flex-1 overflow-y-auto" role="tabpanel" aria-labelledby="sidebar-tab-activity">
                                <ActivityFeed
                                    activities={activities.get()}
                                    max_items=20
                                />
                            </div>
                        }.into_any()
                    }
                }}
            </div>
        </div>
    }
}
#[derive(Params, PartialEq, Clone)]
struct DocumentEditParams {
    id: String,
}

#[component]
fn BacklinksPanel(document_id: String) -> impl IntoView {
    let api_client = ApiClient::default();
    let doc_id_for_fetch = document_id.clone();

    let backlinks_resource = LocalResource::new(move || {
        let client = api_client.clone();
        let did = doc_id_for_fetch.clone();
        async move {
            client
                .get_backlinks(&did)
                .await
                .unwrap_or(BacklinksResponse {
                    backlinks: vec![],
                    count: 0,
                })
        }
    });

    let navigate = use_navigate();

    view! {
        <div>
            <h3 class="text-sm font-semibold text-gray-900 dark:text-white mb-3">"Backlinks"</h3>
            <Suspense fallback={view! { <div class="flex items-center justify-center py-4"><div class="w-4 h-4 border-2 border-gray-400 border-t-transparent rounded-full animate-spin"></div></div> }}>
                {move || {
                    backlinks_resource.get().map(|response| {
                        if response.backlinks.is_empty() {
                            view! {
                                <p class="text-sm text-gray-500 dark:text-gray-400">"No documents link here yet"</p>
                            }.into_any()
                        } else {
                            view! {
                                <div class="space-y-2">
                                    {response.backlinks.into_iter().map(|item| {
                                        let link_id = item.id.clone();
                                        let nav = navigate.clone();
                                        let on_click = move |_: leptos::ev::MouseEvent| {
                                            nav(&format!("/documents/{}", link_id), Default::default());
                                        };
                                        let updated = item.updated_at.split('T').next().unwrap_or("Unknown").to_string();
                                        view! {
                                            <div
                                                class="p-2 rounded hover:bg-gray-100 dark:hover:bg-gray-700 cursor-pointer transition-colors"
                                                on:click={on_click}
                                            >
                                                <div class="text-sm font-medium text-gray-900 dark:text-white">{item.title}</div>
                                                <div class="text-xs text-gray-500 dark:text-gray-400">"Updated: "{updated}</div>
                                            </div>
                                        }
                                    }).collect::<Vec<_>>()}
                                </div>
                            }.into_any()
                        }
                    })
                }}
            </Suspense>
        </div>
    }
}
