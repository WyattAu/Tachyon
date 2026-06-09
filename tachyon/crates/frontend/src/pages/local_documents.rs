//! Local documents page — shows documents that exist only in the browser's
//! local store (IndexedDB / localStorage), without requiring a server connection.

use crate::storage::{BrowserStore, SyncStatus};
use crate::types::Document;
use leptos::prelude::*;
use leptos::task::spawn_local;

/// Page component that lists local (offline) documents.
#[component]
pub fn LocalDocumentsPage() -> impl IntoView {
    let store = BrowserStore::new();
    let documents = RwSignal::new(Vec::<Document>::new());

    // Load local documents on mount
    let s = store.clone();
    spawn_local(async move {
        let all_stored = s.get_all();
        // Filter: only show non-synced (local-only) documents
        let local_docs: Vec<Document> = all_stored
            .into_iter()
            .filter(|d| {
                matches!(
                    d.sync_status,
                    SyncStatus::PendingCreate
                        | SyncStatus::PendingUpdate
                        | SyncStatus::PendingDelete
                        | SyncStatus::Conflict
                )
            })
            .map(|d| crate::storage::stored_to_document(&d))
            .collect();
        documents.set(local_docs);
    });

    view! {
        <div class="p-6 max-w-4xl mx-auto">
            <div class="flex items-center justify-between mb-6">
                <div>
                    <h1 class="text-2xl font-bold text-gray-900 dark:text-white">"Local Documents"</h1>
                    <p class="mt-1 text-sm text-gray-500 dark:text-gray-400">
                        "Documents stored only on this device. These have not been synced to any server."
                    </p>
                </div>
            </div>

            <Show
                when=move || documents.with(|d| d.is_empty())
                fallback=move || view! {
                    <div class="space-y-2">
                        <For
                            each=move || documents.get()
                            key=|d| d.id.clone()
                            children=move |doc: Document| {
                                let doc_id = doc.id.clone();
                                view! {
                                    <a
                                        href=format!("/documents/{}/edit", doc_id)
                                        class="block p-4 border border-gray-200 dark:border-gray-700 hover:bg-gray-50 dark:hover:bg-gray-800 transition-colors"
                                    >
                                        <div class="flex items-center justify-between">
                                            <div>
                                                <h3 class="font-medium text-gray-900 dark:text-white">
                                                    {move || {
                                                        let t = doc.title.clone();
                                                        if t.is_empty() { "Untitled".to_string() } else { t }
                                                    }}
                                                </h3>
                                                <p class="text-sm text-gray-500 dark:text-gray-400 mt-1">
                                                    {move || {
                                                        let c = doc.content.clone();
                                                        if c.len() > 120 { c.chars().take(120).collect() } else { c }
                                                    }}
                                                </p>
                                            </div>
                                            <div class="text-xs text-gray-400 dark:text-gray-500">
                                                {doc.updated_at.clone()}
                                            </div>
                                        </div>
                                    </a>
                                }
                            }
                        />
                    </div>
                }
            >
                <div class="text-center py-16">
                    <svg class="w-16 h-16 mx-auto text-gray-300 dark:text-gray-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"></path>
                    </svg>
                    <h3 class="mt-4 text-lg font-medium text-gray-900 dark:text-white">"No local documents"</h3>
                    <p class="mt-2 text-sm text-gray-500 dark:text-gray-400">
                        "Documents you create offline will appear here."
                    </p>
                    <a href="/documents" class="mt-4 inline-block text-blue-600 hover:underline dark:text-blue-400 text-sm">
                        "Go to server documents →"
                    </a>
                </div>
            </Show>
        </div>
    }
}
