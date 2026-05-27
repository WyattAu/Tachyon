// Tags Page
// Browse all tags, view documents per tag, search/filter

use crate::api::ApiClient;
use crate::components::{BreadcrumbItem, Breadcrumbs};
use crate::types::{SearchResultItem, TagInfo};
use leptos::prelude::*;
use leptos::task::spawn_local;

// ============================================================================
// Page Component
// ============================================================================

#[component]
pub fn TagsPage() -> impl IntoView {
    let (tags, set_tags) = signal(Vec::<TagInfo>::new());
    let (selected_tag, set_selected_tag) = signal(None::<String>);
    let (documents, set_documents) = signal(Vec::<SearchResultItem>::new());
    let (loading, set_loading) = signal(false);
    let (loading_docs, set_loading_docs) = signal(false);
    let (error, set_error) = signal(None::<String>);
    let (search_query, set_search_query) = signal(String::new());

    // Load all tags on mount
    Effect::new(move |_| {
        set_loading.set(true);
        set_error.set(None);
        spawn_local(async move {
            let client = ApiClient::default();
            match client.list_tags().await {
                Ok(response) => {
                    set_tags.set(response.tags);
                    set_loading.set(false);
                }
                Err(e) => {
                    set_error.set(Some(format!("Failed to load tags: {}", e)));
                    set_loading.set(false);
                }
            }
        });
    });

    // Load documents when a tag is selected
    let on_tag_click = move |tag: String| {
        set_selected_tag.set(Some(tag.clone()));
        set_loading_docs.set(true);
        spawn_local(async move {
            let client = ApiClient::default();
            match client.list_documents_by_tag(&tag, Some(1), Some(50)).await {
                Ok(response) => {
                    set_documents.set(response.results);
                    set_loading_docs.set(false);
                }
                Err(e) => {
                    set_error.set(Some(format!("Failed to load documents: {}", e)));
                    set_loading_docs.set(false);
                }
            }
        });
    };

    let clear_selection = move |_: ()| {
        set_selected_tag.set(None);
        set_documents.set(Vec::new());
    };

    // Filter tags by search query
    let filtered_tags = move || {
        let query = search_query.get().to_lowercase();
        let all_tags = tags.get();
        if query.is_empty() {
            all_tags
        } else {
            all_tags
                .into_iter()
                .filter(|t| t.tag.to_lowercase().contains(&query))
                .collect::<Vec<_>>()
        }
    };

    view! {
        <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
            <Breadcrumbs items={vec![
                BreadcrumbItem { label: "Tags".into(), href: None },
            ]}/>

            <div class="mb-8">
                <h1 class="text-3xl font-bold text-gray-900 dark:text-white">"Tags"</h1>
                <p class="mt-2 text-gray-600 dark:text-gray-400">
                    "Browse and filter documents by tag."
                </p>
            </div>

            <Show when=move || error.get().is_some()>
                <div class="mb-6 p-4 bg-red-100 dark:bg-red-900 border-2 border-red-400 dark:border-red-700 text-red-700 dark:text-red-200 rounded-none">
                    {move || error.get().unwrap_or_default()}
                </div>
            </Show>

            <Show when=move || loading.get()>
                <div class="flex items-center justify-center py-16">
                    <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600"></div>
                    <span class="ml-3 text-gray-500">"Loading tags..."</span>
                </div>
            </Show>

            <Show when=move || !loading.get()>
                // Search bar
                <div class="mb-6">
                    <input
                        type="text"
                        placeholder="Filter tags..."
                        prop:value={move || search_query.get()}
                        on:input=move |ev| {
                            let val = event_target_value(&ev);
                            set_search_query.set(val);
                        }
                        class="w-full max-w-md px-4 py-2 border border-gray-300 dark:border-gray-600 rounded-none bg-white dark:bg-gray-800 text-gray-900 dark:text-white focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                    />
                </div>

                // Tag cloud
                <div class="mb-8">
                    <div class="flex flex-wrap gap-2">
                        <For
                            each=filtered_tags
                            key=|t| t.tag.clone()
                            let:tag
                        >
                            {
                                let selected = selected_tag;
                                let tag_name = tag.tag.clone();
                                let is_selected = move || selected.get().as_deref() == Some(&tag_name);
                                let is_selected_click = is_selected.clone();
                                let tag_for_click = tag.tag.clone();
                                let on_click = move |_| {
                                    if is_selected_click() {
                                        clear_selection(());
                                    } else {
                                        on_tag_click(tag_for_click.clone());
                                    }
                                };
                                let tag_classes = move || {
                                    let size = if tag.count > 10 { "text-lg px-4 py-2" }
                                    else if tag.count > 5 { "text-base px-3 py-1.5" }
                                    else if tag.count > 2 { "text-sm px-3 py-1" }
                                    else { "text-xs px-2 py-1" };
                                    if is_selected() {
                                        format!("{} rounded-full font-medium transition-colors bg-blue-600 text-white", size)
                                    } else {
                                        format!("{} rounded-full font-medium transition-colors bg-blue-100 dark:bg-blue-900 text-blue-800 dark:text-blue-200 hover:bg-blue-200 dark:hover:bg-blue-800", size)
                                    }
                                };
                                view! {
                                    <button
                                        class=tag_classes
                                        on:click=on_click
                                    >
                                        {tag.tag.clone()}
                                        <span class="ml-1 opacity-70">"("{tag.count}")"</span>
                                    </button>
                                }
                            }
                        </For>
                    </div>
                    <Show when=move || filtered_tags().is_empty() && !tags.get().is_empty()>
                        <p class="mt-4 text-sm text-gray-500 dark:text-gray-400">"No tags match your filter."</p>
                    </Show>
                    <Show when=move || tags.get().is_empty()>
                        <p class="mt-4 text-sm text-gray-500 dark:text-gray-400">"No tags found. Add tags to your documents to see them here."</p>
                    </Show>
                </div>

                // Documents for selected tag
                <Show when=move || selected_tag.get().is_some()>
                    <div class="border-t border-gray-200 dark:border-gray-700 pt-6">
                        <div class="flex items-center justify-between mb-4">
                            <h2 class="text-xl font-semibold text-gray-900 dark:text-white">
                                "Documents tagged \""
                                {move || selected_tag.get().unwrap_or_default()}
                                "\""
                            </h2>
                            <button
                                class="text-sm text-blue-600 dark:text-blue-400 hover:underline"
                                on:click=move |_: leptos::ev::MouseEvent| clear_selection(())
                            >
                                "Clear filter"
                            </button>
                        </div>

                        <Show when=move || loading_docs.get()>
                            <div class="flex items-center justify-center py-8">
                                <div class="animate-spin rounded-full h-6 w-6 border-b-2 border-blue-600"></div>
                                <span class="ml-2 text-gray-500">"Loading documents..."</span>
                            </div>
                        </Show>

                        <Show when=move || !loading_docs.get() && !documents.get().is_empty()>
                            <div class="space-y-3">
                                <For
                                    each=move || documents.get()
                                    key=|d| d.id.clone()
                                    let:doc
                                >
                                    {
                                        let doc_id = doc.id.clone();
                                        let updated = doc.updated_at.split('T').next().unwrap_or("Unknown").to_string();
                                        let doc_status = doc.status.clone();
                                        let doc_status_for_class = doc.status.clone();
                                        let doc_tags = doc.tags.clone();
                                        let status_class = move || {
                                            let base = "ml-3 inline-flex items-center px-2 py-0.5 rounded text-xs font-medium flex-shrink-0";
                                            match doc_status_for_class.as_str() {
                                                "published" => format!("{} bg-green-100 dark:bg-green-900 text-green-800 dark:text-green-200", base),
                                                "draft" => format!("{} bg-yellow-100 dark:bg-yellow-900 text-yellow-800 dark:text-yellow-200", base),
                                                "archived" => format!("{} bg-gray-100 dark:bg-gray-700 text-gray-800 dark:text-gray-200", base),
                                                _ => format!("{} bg-gray-100 dark:bg-gray-700 text-gray-600 dark:text-gray-300", base),
                                            }
                                        };
                                        view! {
                                            <a
                                                href=format!("/documents/{}", doc_id)
                                                class="block p-4 bg-white dark:bg-gray-800 rounded-none border-2 border-gray-900 dark:border-gray-100 hover:border-blue-400 dark:hover:border-blue-500 transition-colors no-underline text-inherit"
                                            >
                                                <div class="flex items-start justify-between">
                                                    <div class="min-w-0 flex-1">
                                                        <h3 class="text-base font-medium text-gray-900 dark:text-white truncate">
                                                            {doc.title.clone()}
                                                        </h3>
                                                        <p class="mt-1 text-sm text-gray-500 dark:text-gray-400 line-clamp-2">
                                                            {doc.description.clone().unwrap_or_default()}
                                                        </p>
                                                    </div>
                                                    <span class=status_class>
                                                        {doc_status.clone()}
                                                    </span>
                                                </div>
                                                <div class="mt-3 flex items-center gap-4 text-xs text-gray-400">
                                                    <span>{doc.word_count}" words"</span>
                                                    <span>"Updated "{updated}</span>
                                                    {doc_tags.into_iter().map(|tag| {
                                                        view! {
                                                            <span class="px-1.5 py-0.5 bg-gray-100 dark:bg-gray-700 rounded text-gray-600 dark:text-gray-400">
                                                                {tag}
                                                            </span>
                                                        }
                                                    }).collect::<Vec<_>>()}
                                                </div>
                                            </a>
                                        }
                                    }
                                </For>
                            </div>
                        </Show>

                        <Show when=move || !loading_docs.get() && documents.get().is_empty()>
                            <p class="text-sm text-gray-500 dark:text-gray-400 py-4">"No documents found with this tag."</p>
                        </Show>
                    </div>
                </Show>

                // Stats summary
                <Show when=move || selected_tag.get().is_none() && !tags.get().is_empty()>
                    <div class="border-t border-gray-200 dark:border-gray-700 pt-6">
                        <div class="grid grid-cols-2 md:grid-cols-4 gap-4">
                            <div class="bg-white dark:bg-gray-800 rounded-none p-4 border-2 border-gray-900 dark:border-gray-100">
                                <div class="text-2xl font-bold text-blue-600 dark:text-blue-400">
                                    {move || tags.get().len().to_string()}
                                </div>
                                <div class="text-sm text-gray-500 dark:text-gray-400">"Total Tags"</div>
                            </div>
                            <div class="bg-white dark:bg-gray-800 rounded-none p-4 border-2 border-gray-900 dark:border-gray-100">
                                <div class="text-2xl font-bold text-green-600 dark:text-green-400">
                                    {move || tags.get().iter().filter(|t| t.count > 5).count().to_string()}
                                </div>
                                <div class="text-sm text-gray-500 dark:text-gray-400">"Popular (5+)"</div>
                            </div>
                            <div class="bg-white dark:bg-gray-800 rounded-none p-4 border-2 border-gray-900 dark:border-gray-100">
                                <div class="text-2xl font-bold text-purple-600 dark:text-purple-400">
                                    {move || tags.get().iter().map(|t| t.count).max().unwrap_or(0).to_string()}
                                </div>
                                <div class="text-sm text-gray-500 dark:text-gray-400">"Most Used"</div>
                            </div>
                            <div class="bg-white dark:bg-gray-800 rounded-none p-4 border-2 border-gray-900 dark:border-gray-100">
                                <div class="text-2xl font-bold text-amber-600 dark:text-amber-400">
                                    {move || tags.get().iter().filter(|t| t.count == 1).count().to_string()}
                                </div>
                                <div class="text-sm text-gray-500 dark:text-gray-400">"Unique Tags"</div>
                            </div>
                        </div>
                    </div>
                </Show>
            </Show>
        </div>
    }
}
