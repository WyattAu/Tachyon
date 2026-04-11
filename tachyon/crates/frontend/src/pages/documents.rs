// Documents Pages

use leptos::prelude::*;
use leptos_router::hooks::{use_params, use_navigate};
use leptos_router::params::Params;
use crate::api::ApiClient;
use crate::types::{Document, DocumentListResponse, DocumentTemplate, BacklinksResponse};
use crate::components::{DocumentEditor, ActivityFeed, Activity, ActivityType, VersionHistory, TemplateSelector, ReviewPanel, ConflictResolver};
use crate::websocket::{WebSocketClient, WsMessage};
use std::rc::Rc;
use wasm_bindgen_futures::spawn_local;

/// Documents list page
#[component]
pub fn DocumentsPage() -> impl IntoView {
    let api_client = ApiClient::default();
    let current_page = RwSignal::new(1usize);
    let page_size = 20usize;

    // New document modal state
    let (show_create_modal, set_show_create_modal) = signal(false);
    let (new_doc_title, set_new_doc_title) = signal(String::new());
    let (creating, set_creating) = signal(false);
    let (create_error, set_create_error) = signal::<Option<String>>(None);

    let api_client_for_create = api_client.clone();

    let documents_resource = LocalResource::new(move || {
        let client = api_client.clone();
        let page = current_page.get();
        async move {
            client
                .list_documents(Some(page), Some(page_size))
                .await
                .unwrap_or(DocumentListResponse {
                    results: vec![],
                    total: 0,
                    page: 1,
                    page_size: 20,
                })
        }
    });

    let total_pages = move || {
        documents_resource.get().map(|d| {
            if d.page_size > 0 {
                (d.total as f64 / d.page_size as f64).ceil() as usize
            } else {
                1
            }
        }).unwrap_or(1)
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

            spawn_local(async move {
                let body = serde_json::json!({
                    "title": title.trim(),
                    "content": "",
                    "tags": [],
                });
                match api.create_document(&body).await {
                    Ok(doc) => {
                        let doc_id = doc.id.clone();
                        set_show_create_modal.set(false);
                        set_new_doc_title.set(String::new());
                        set_creating.set(false);
                        nav(&format!("/documents/{}/edit", doc_id), Default::default());
                    }
                    Err(e) => {
                        set_create_error.set(Some(format!("Failed to create document: {}", e)));
                        set_creating.set(false);
                    }
                }
            });
        }
    });

    view! {
        <div>
            <div class="flex items-center justify-between mb-6">
                <h1 class="text-2xl font-bold text-gray-900 dark:text-white">"Documents"</h1>
                <button class="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors" on:click=move |_| set_show_create_modal.set(true)>
                    "+ New Document"
                </button>
            </div>

            <Suspense fallback={view! { <DocumentsGridSkeleton /> }}>
                {move || {
                    documents_resource.get().map(|response| {
                        if response.results.is_empty() {
                            view! {
                                <div class="bg-white dark:bg-gray-800 rounded-lg shadow border border-gray-200 dark:border-gray-700 p-6">
                                    <p class="text-gray-500 dark:text-gray-400 text-center">
                                        "No documents found. Create your first document to get started!"
                                    </p>
                                </div>
                            }.into_any()
                        } else {
                            view! {
                                <div>
                                    <div class="text-sm text-gray-500 dark:text-gray-400 mb-4">
                                        {format!("Showing {} of {} documents", response.results.len(), response.total)}
                                    </div>
                                    <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
                                        {response.results.into_iter().map(|doc| {
                                            view! {
                                                <DocumentCard document={doc} />
                                            }
                                        }).collect::<Vec<_>>()}
                                    </div>
                                </div>
                            }.into_any()
                        }
                    })
                }}
            </Suspense>

            <div class="flex items-center justify-center gap-2 mt-6">
                <button
                    class="px-3 py-2 text-sm bg-white dark:bg-gray-800 border border-gray-300 dark:border-gray-600 rounded-lg hover:bg-gray-50 dark:hover:bg-gray-700 disabled:opacity-50 disabled:cursor-not-allowed text-gray-700 dark:text-gray-300"
                    disabled={move || current_page.get() <= 1}
                    on:click={move |_| {
                        let current = current_page.get();
                        if current > 1 {
                            current_page.set(current - 1);
                        }
                    }}
                >
                    "Previous"
                </button>

                <span class="px-3 py-2 text-sm text-gray-700 dark:text-gray-300">
                    {move || format!("Page {} of {}", current_page.get(), total_pages())}
                </span>

                <button
                    class="px-3 py-2 text-sm bg-white dark:bg-gray-800 border border-gray-300 dark:border-gray-600 rounded-lg hover:bg-gray-50 dark:hover:bg-gray-700 disabled:opacity-50 disabled:cursor-not-allowed text-gray-700 dark:text-gray-300"
                    disabled={move || current_page.get() >= total_pages()}
                    on:click={move |_| {
                        let current = current_page.get();
                        if current < total_pages() {
                            current_page.set(current + 1);
                        }
                    }}
                >
                    "Next"
                </button>
            </div>

            // Create document modal
            {move || if show_create_modal.get() {
                Some(view! {
                    <div class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
                        <div class="bg-white dark:bg-gray-800 rounded-lg p-6 w-full max-w-md">
                            <h2 class="text-xl font-bold text-gray-900 dark:text-white mb-4">"New Document"</h2>

                            {move || create_error.get().map(|e| view! {
                                <div class="mb-4 p-3 bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded text-sm text-red-700 dark:text-red-300">
                                    {e}
                                </div>
                            })}

                            <div class="space-y-4">
                                <div>
                                    <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                                        "Start from template"
                                    </label>
                                    <TemplateSelector
                                        on_select={Callback::new(move |template: DocumentTemplate| {
                                            let title = template.name.clone();
                                            set_new_doc_title.set(title);
                                        })}
                                        category={None}
                                    />
                                </div>
                                <div>
                                    <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                                        "Title"
                                    </label>
                                    <input
                                        type="text"
                                        class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 dark:bg-gray-700 dark:text-white"
                                        placeholder="Enter document title"
                                        prop:value={move || new_doc_title.get()}
                                        on:input=move |ev| set_new_doc_title.set(event_target_value(&ev))
                                    />
                                </div>
                            </div>

                            <div class="mt-6 flex justify-end gap-3">
                                <button
                                    class="px-4 py-2 text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-md transition-colors"
                                    on:click=move |_| {
                                        set_show_create_modal.set(false);
                                        set_new_doc_title.set(String::new());
                                        set_create_error.set(None);
                                    }
                                >
                                    "Cancel"
                                </button>
                                <button
                                    class="px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 transition-colors disabled:opacity-50"
                                    disabled={move || creating.get()}
                                    on:click=move |ev| handle_create_document.get_value()(ev)
                                >
                                    {move || if creating.get() { "Creating..." } else { "Create" }}
                                </button>
                            </div>
                        </div>
                    </div>
                })
            } else {
                None
            }}
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
                        <div class="bg-white dark:bg-gray-800 rounded-lg shadow border border-gray-200 dark:border-gray-700 p-6 animate-pulse">
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
    let created_date = document.created_at.split('T').next().unwrap_or("Unknown").to_string();
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
        navigate(&format!("/documents/{}/edit", doc_id_for_click), Default::default());
    };

    view! {
        <div 
            class="bg-white dark:bg-gray-800 rounded-lg shadow border border-gray-200 dark:border-gray-700 p-6 hover:border-blue-500 transition-colors cursor-pointer"
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
        params.with(|p| p.as_ref().map(|p| p.id.clone()).unwrap_or_default())
    };

    let api_client = ApiClient::default();
    let doc_id = document_id();
    let (loading, set_loading) = signal(true);
    let (load_error, set_load_error) = signal::<Option<String>>(None);
    let (doc, set_doc) = signal::<Option<Document>>(None);

    let set_ld = set_loading.clone();
    let set_le = set_load_error.clone();
    let set_d = set_doc.clone();

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
            // Back link
            <a href="/documents" class="inline-flex items-center gap-1 text-sm text-blue-600 dark:text-blue-400 hover:underline mb-6">
                <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
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
                        let _ = nav(&format!("/documents/{}/edit", edit_id_clone), Default::default());
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
                                    class="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors flex items-center gap-2"
                                >
                                    <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z" />
                                    </svg>
                                    "Edit"
                                </button>
                            </div>

                            // Metadata
                            <div class="flex flex-wrap gap-4 mb-6 text-sm text-gray-500 dark:text-gray-400">
                                <span class="flex items-center gap-1">
                                    <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 7V3m8 4V3m-9 8h10M5 21h14a2 2 0 002-2V7a2 2 0 00-2-2H5a2 2 0 00-2 2v12a2 2 0 002 2z" />
                                    </svg>
                                    "Created: "{created_at.split('T').next().unwrap_or("Unknown")}
                                </span>
                                <span class="flex items-center gap-1">
                                    <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z" />
                                    </svg>
                                    "Updated: "{updated_at.split('T').next().unwrap_or("Unknown")}
                                </span>
                                <span class="flex items-center gap-1">
                                    <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
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
                            <div class="bg-white dark:bg-gray-800 rounded-lg shadow border border-gray-200 dark:border-gray-700 p-6">
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

/// Document edit page with real-time collaboration
#[component]
pub fn DocumentEditPage() -> impl IntoView {
    let params = use_params::<DocumentEditParams>();
    let document_id = move || {
        params.with(|p| p.as_ref().map(|p| p.id.clone()).unwrap_or_default())
    };
    
    let user_id = "user-".to_string() + &uuid::Uuid::new_v4().to_string()[..8];
    let user_name = "User".to_string();
    
    // Fetch document content on mount
    let (doc_content, set_doc_content) = signal(String::new());
    let (doc_title, set_doc_title) = signal(String::new());
    let (loading, set_loading) = signal(true);
    let (load_error, set_load_error) = signal::<Option<String>>(None);
    let (sidebar_tab, set_sidebar_tab) = signal("activity".to_string());

    let api_client = ApiClient::default();
    let fetch_doc_id = document_id();
    let set_dc = set_doc_content.clone();
    let set_dt = set_doc_title.clone();
    let set_ld = set_loading.clone();
    let set_le = set_load_error.clone();

    Effect::new(move || {
        let did = fetch_doc_id.clone();
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
    
    let (activities, set_activities) = signal(Vec::<Activity>::new());
    let ws_client = WebSocketClient::new("");
    
    let ws_for_activity = ws_client.clone();
    let set_activities_clone = set_activities;
    Effect::new(move || {
        let ws = ws_for_activity.clone();
        ws.on_message(Rc::new(move |msg: WsMessage| {
            if msg.message_type == "activity" {
                if let Some(data) = &msg.data {
                    if let Ok(activity) = serde_json::from_value::<ActivityData>(data.clone()) {
                        let new_activity = Activity::new(
                            ActivityType::from(activity.activity_type.as_str()),
                            activity.user_id,
                            activity.user_name,
                            activity.description,
                        );
                        set_activities_clone.update(|acts| {
                            acts.insert(0, new_activity);
                            if acts.len() > 50 {
                                acts.pop();
                            }
                        });
                    }
                }
            }
        }));
    });

    view! {
        <div class="flex h-[calc(100vh-4rem)]">
            <div class="flex-1 flex flex-col overflow-hidden">
                <div class="p-4 border-b border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-800">
                    <h1 class="text-xl font-semibold text-gray-900 dark:text-white">
                        {move || {
                            let title = doc_title.get();
                            if title.is_empty() {
                                format!("Editing Document: {}", document_id())
                            } else {
                                title
                            }
                        }}
                    </h1>
                </div>
                
                <div class="flex-1 overflow-hidden">
                    {move || {
                        let doc_id = document_id();
                        if doc_id.is_empty() {
                            view! {
                                <div class="flex items-center justify-center h-full">
                                    <p class="text-gray-500 dark:text-gray-400">"Select a document to edit"</p>
                                </div>
                            }.into_any()
                        } else if loading.get() {
                            view! {
                                <div class="flex items-center justify-center h-full">
                                    <div class="flex items-center gap-3 text-gray-500 dark:text-gray-400">
                                        <div class="w-5 h-5 border-2 border-gray-400 border-t-transparent rounded-full animate-spin"></div>
                                        <span>"Loading document..."</span>
                                    </div>
                                </div>
                            }.into_any()
                        } else if let Some(err) = load_error.get() {
                            view! {
                                <div class="flex items-center justify-center h-full">
                                    <div class="p-4 bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded text-red-700 dark:text-red-300">
                                        {err}
                                    </div>
                                </div>
                            }.into_any()
                        } else {
                            view! {
                                <DocumentEditor 
                                    document_id={doc_id}
                                    user_id={user_id.clone()}
                                    user_name={user_name.clone()}
                                    initial_content={doc_content.get()}
                                />
                            }.into_any()
                        }
                    }}
                </div>
            </div>
            
            <div class="w-80 flex-shrink-0 border-l border-gray-200 dark:border-gray-700">
                <div class="flex border-b border-gray-200 dark:border-gray-700">
                    <button
                        class="flex-1 px-3 py-2 text-sm font-medium border-b-2 transition-colors "
                        on:click=move |_| set_sidebar_tab.set("activity".to_string())
                    >
                        "Activity"
                    </button>
                    <button
                        class="flex-1 px-3 py-2 text-sm font-medium border-b-2 transition-colors "
                        on:click=move |_| set_sidebar_tab.set("history".to_string())
                    >
                        "History"
                    </button>
                    <button
                        class="flex-1 px-3 py-2 text-sm font-medium border-b-2 transition-colors "
                        on:click=move |_| set_sidebar_tab.set("review".to_string())
                    >
                        "Review"
                    </button>
                    <button
                        class="flex-1 px-3 py-2 text-sm font-medium border-b-2 transition-colors "
                        on:click=move |_| set_sidebar_tab.set("conflicts".to_string())
                    >
                        "Conflicts"
                    </button>
                    <button
                        class="flex-1 px-3 py-2 text-sm font-medium border-b-2 transition-colors "
                        on:click=move |_| set_sidebar_tab.set("backlinks".to_string())
                    >
                        "Backlinks"
                    </button>
                </div>
                {move || {
                    let tab = sidebar_tab.get();
                    let doc_id = document_id();
                    if tab == "history" {
                        view! {
                            <div class="p-4 overflow-y-auto h-[calc(100vh-6rem)]">
                                <VersionHistory document_id={doc_id} on_rollback=None />
                            </div>
                        }.into_any()
                    } else if tab == "review" {
                        view! {
                            <div class="p-4 overflow-y-auto h-[calc(100vh-6rem)]">
                                <ReviewPanel document_id={doc_id} />
                            </div>
                        }.into_any()
                    } else if tab == "conflicts" {
                        view! {
                            <div class="p-4 overflow-y-auto h-[calc(100vh-6rem)]">
                                <ConflictResolver document_id={doc_id} />
                            </div>
                        }.into_any()
                    } else if tab == "backlinks" {
                        view! {
                            <div class="p-4 overflow-y-auto h-[calc(100vh-6rem)]">
                                <BacklinksPanel document_id={doc_id} />
                            </div>
                        }.into_any()
                    } else {
                        view! {
                            <div class="p-4 overflow-y-auto h-[calc(100vh-6rem)]">
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

#[derive(serde::Deserialize)]
struct ActivityData {
    activity_type: String,
    user_id: String,
    user_name: String,
    description: String,
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
