// Search Page
// Global search across documents and projects with advanced filtering

use std::sync::Arc;
use leptos::prelude::*;
use wasm_bindgen::JsCast;
use crate::api::ApiClient;
use crate::components::EmptySearch;
use crate::types::{
    GlobalSearchResponse, SearchResultItem, ProjectSearchResultItem,
    CreateSavedSearchRequest, SearchFilters,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchCategory {
    All,
    Documents,
    Projects,
}

impl SearchCategory {
    pub fn label(&self) -> &'static str {
        match self {
            SearchCategory::All => "All",
            SearchCategory::Documents => "Documents",
            SearchCategory::Projects => "Projects",
        }
    }
}

#[component]
pub fn SearchPage() -> impl IntoView {
    let query = RwSignal::new(String::new());
    let category = RwSignal::new(SearchCategory::All);
    let current_page = RwSignal::new(1i64);
    let page_size = RwSignal::new(20i64);
    let show_filters = RwSignal::new(false);
    let show_saved = RwSignal::new(false);

    let filters = RwSignal::new(SearchFilters::default());
    let selected_status = RwSignal::new(None::<String>);
    let selected_visibility = RwSignal::new(None::<String>);
    let selected_tags = RwSignal::new(Vec::<String>::new());
    let date_from = RwSignal::new(String::new());
    let date_to = RwSignal::new(String::new());

    let api_client = ApiClient::default();
    let api_client_search = api_client.clone();
    let api_client_saved = api_client.clone();

    let suggestions = RwSignal::new(Vec::<String>::new());
    let show_suggestions = RwSignal::new(false);

    let search_resource = LocalResource::new(move || {
        let client = api_client_search.clone();
        let q = query.get();
        let cat = category.get();
        let page = current_page.get();
        let ps = page_size.get();
        let mut f = SearchFilters::default();
        
        if let Some(s) = selected_status.get() {
            f.status = Some(s);
        }
        if let Some(v) = selected_visibility.get() {
            f.visibility = Some(v);
        }
        let tags = selected_tags.get();
        if !tags.is_empty() {
            f.tags = Some(tags);
        }
        let df = date_from.get();
        if !df.is_empty() {
            f.date_from = Some(df);
        }
        let dt = date_to.get();
        if !dt.is_empty() {
            f.date_to = Some(dt);
        }

        async move {
            if q.is_empty() {
                None
            } else if cat == SearchCategory::All {
                client.global_search(&q, Some(&f), Some(page), Some(ps)).await.ok()
            } else {
                client.search(&q, Some(&f), Some(page), Some(ps)).await.ok().map(|r| {
                    GlobalSearchResponse {
                        documents: r,
                        projects: vec![],
                    }
                })
            }
        }
    });

    let saved_searches_resource = LocalResource::new(move || {
        let client = api_client_saved.clone();
        async move {
            client.list_saved_searches().await.unwrap_or_default()
        }
    });

    let update_filters = move || {
        let mut f = SearchFilters::default();
        if let Some(s) = selected_status.get() {
            f.status = Some(s);
        }
        if let Some(v) = selected_visibility.get() {
            f.visibility = Some(v);
        }
        let tags = selected_tags.get();
        if !tags.is_empty() {
            f.tags = Some(tags);
        }
        let df = date_from.get();
        if !df.is_empty() {
            f.date_from = Some(df);
        }
        let dt = date_to.get();
        if !dt.is_empty() {
            f.date_to = Some(dt);
        }
        filters.set(f);
    };

    Effect::new(move |_| {
        let _ = query.get();
        let _ = selected_status.get();
        let _ = selected_visibility.get();
        let _ = selected_tags.get();
        let _ = date_from.get();
        let _ = date_to.get();
        current_page.set(1);
        update_filters();
    });

    let api_client_for_input = api_client.clone();
    let on_input = move |ev| {
        let val = event_target_value(&ev);
        query.set(val.clone());

        let client = api_client_for_input.clone();
        let show_sugg = show_suggestions.clone();
        let suggs = suggestions.clone();

        let cb = wasm_bindgen::prelude::Closure::<dyn Fn()>::new(move || {
            let q = val.clone();
            let client_inner = client.clone();
            wasm_bindgen_futures::spawn_local(async move {
                if q.len() < 2 {
                    suggs.set(Vec::new());
                    return;
                }
                match client_inner.search_suggest(&q, Some(5)).await {
                    Ok(items) => {
                        suggs.set(items);
                        show_sugg.set(true);
                    }
                    Err(_) => {
                        suggs.set(Vec::new());
                    }
                }
            });
        });
        if let Some(window) = web_sys::window() {
            let fn_ref: &js_sys::Function = cb.as_ref().unchecked_ref();
            let _ = window.set_timeout_with_callback(fn_ref);
        }
        cb.forget();
    };

    let select_suggestion = move |suggestion: String| {
        query.set(suggestion.clone());
        show_suggestions.set(false);
        suggestions.set(Vec::new());
    };

    let close_suggestions = move |_| {
        show_suggestions.set(false);
    };

    let save_search: Arc<dyn Fn() + Send + Sync> = {
        let api_client = api_client.clone();
        let query = query.clone();
        let filters = filters.clone();
        Arc::new(move || {
            let client = api_client.clone();
            let q = query.get();
            let f = filters.get();
            
            if q.is_empty() {
                return;
            }
            
            let name = q.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let request = CreateSavedSearchRequest {
                    name,
                    query: q,
                    filters: Some(f),
                };
                let _ = client.create_saved_search(&request).await;
            });
        })
    };

    view! {
        <div class="flex gap-6" on:click=close_suggestions>
            // Saved Searches Sidebar
            {move || {
                if show_saved.get() {
                    view! {
                        <div class="w-64 flex-shrink-0">
                            <div class="bg-white dark:bg-gray-800 rounded-lg shadow border border-gray-200 dark:border-gray-700 p-4">
                                <h3 class="text-sm font-semibold text-gray-900 dark:text-white mb-3">
                                    "Saved Searches"
                                </h3>
                                <Suspense fallback={view! { <div class="animate-pulse">"Loading..."</div> }}>
                                    {move || {
                                        let saved = saved_searches_resource.get().unwrap_or_default();
                                        if saved.is_empty() {
                                            view! {
                                                <p class="text-sm text-gray-500 dark:text-gray-400">
                                                    "No saved searches yet"
                                                </p>
                                            }.into_any()
                                        } else {
                                            view! {
                                                <ul class="space-y-2">
                                                    {saved.into_iter().map(|s| {
                                                        view! {
                                                            <li>
                                                                <button
                                                                    class="w-full text-left px-3 py-2 text-sm rounded-lg hover:bg-gray-100 dark:hover:bg-gray-700 text-gray-700 dark:text-gray-300"
                                                                    on:click=move |_| {
                                                                        query.set(s.query.clone());
                                                                        if let Some(f) = s.filters.clone() {
                                                                            selected_status.set(f.status);
                                                                            selected_visibility.set(f.visibility);
                                                                            if let Some(t) = f.tags {
                                                                                selected_tags.set(t);
                                                                            }
                                                                            if let Some(df) = f.date_from {
                                                                                date_from.set(df);
                                                                            }
                                                                            if let Some(dt) = f.date_to {
                                                                                date_to.set(dt);
                                                                            }
                                                                        }
                                                                    }
                                                                >
                                                                    {s.name}
                                                                </button>
                                                            </li>
                                                        }
                                                    }).collect::<Vec<_>>()}
                                                </ul>
                                            }.into_any()
                                        }
                                    }}
                                </Suspense>
                            </div>
                        </div>
                    }.into_any()
                } else {
                    ().into_any()
                }
            }}

            // Main Search Content
            <div class="flex-1">
                <div class="flex items-center justify-between mb-6">
                    <h1 class="text-2xl font-bold text-gray-900 dark:text-white">"Search"</h1>
                    <button
                        class="text-sm text-gray-500 dark:text-gray-400 hover:text-gray-700 dark:hover:text-gray-300"
                        on:click=move |_| show_saved.update(|s| *s = !*s)
                    >
                        {move || if show_saved.get() { "Hide Saved" } else { "Show Saved" }}
                    </button>
                </div>

                // Search Input with autocomplete
                <div class="mb-4 relative">
                    <div class="flex gap-2">
                        <div class="flex-1 relative">
                            <input
                                type="text"
                                placeholder="Search documents and projects..."
                                class="w-full p-3 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-800 text-gray-900 dark:text-white focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                                on:input=on_input
                                prop:value=move || query.get()
                            />
                            // Suggestions dropdown
                            <div
                                class=move || {
                                    if show_suggestions.get() && !suggestions.get().is_empty() {
                                        "absolute top-full left-0 right-0 z-50 mt-1 bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-lg shadow-lg max-h-60 overflow-auto"
                                    } else {
                                        "hidden"
                                    }
                                }
                                on:click=move |ev| ev.stop_propagation()
                            >
                                {move || {
                                    let suggs = suggestions.get();
                                    suggs.into_iter().map(|s| {
                                        let s_clone = s.clone();
                                        let s_display = s.clone();
                                        let sel = select_suggestion.clone();
                                        view! {
                                            <button
                                                class="w-full text-left px-4 py-2 text-sm text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 border-b border-gray-100 dark:border-gray-700 last:border-b-0"
                                                on:click=move |_| sel(s_clone.clone())
                                            >
                                                {s_display}
                                            </button>
                                        }
                                    }).collect::<Vec<_>>()
                                }}
                            </div>
                        </div>
                        <button
                            class="px-4 py-2 bg-gray-200 dark:bg-gray-700 text-gray-700 dark:text-gray-300 rounded-lg hover:bg-gray-300 dark:hover:bg-gray-600"
                            on:click=move |_| show_filters.update(|f| *f = !*f)
                        >
                            "Filters"
                        </button>
                    </div>
                </div>

                // Advanced Filters Panel
                {move || {
                    if show_filters.get() {
                        view! {
                            <div class="mb-6 p-4 bg-gray-50 dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700">
                                <div class="grid grid-cols-2 md:grid-cols-4 gap-4">
                                    // Status Filter
                                    <div>
                                        <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                                            "Status"
                                        </label>
                                        <select
                                            class="w-full p-2 border border-gray-300 dark:border-gray-600 rounded bg-white dark:bg-gray-700 text-gray-900 dark:text-white"
                                            on:change=move |ev| {
                                                let val = event_target_value(&ev);
                                                selected_status.set(if val.is_empty() { None } else { Some(val) });
                                            }
                                        >
                                            <option value="">"Any"</option>
                                            <option value="draft">"Draft"</option>
                                            <option value="published">"Published"</option>
                                            <option value="archived">"Archived"</option>
                                        </select>
                                    </div>

                                    // Visibility Filter
                                    <div>
                                        <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                                            "Visibility"
                                        </label>
                                        <select
                                            class="w-full p-2 border border-gray-300 dark:border-gray-600 rounded bg-white dark:bg-gray-700 text-gray-900 dark:text-white"
                                            on:change=move |ev| {
                                                let val = event_target_value(&ev);
                                                selected_visibility.set(if val.is_empty() { None } else { Some(val) });
                                            }
                                        >
                                            <option value="">"Any"</option>
                                            <option value="public">"Public"</option>
                                            <option value="private">"Private"</option>
                                            <option value="restricted">"Restricted"</option>
                                        </select>
                                    </div>

                                    // Date From
                                    <div>
                                        <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                                            "From Date"
                                        </label>
                                        <input
                                            type="date"
                                            class="w-full p-2 border border-gray-300 dark:border-gray-600 rounded bg-white dark:bg-gray-700 text-gray-900 dark:text-white"
                                            on:input=move |ev| date_from.set(event_target_value(&ev))
                                            prop:value=move || date_from.get()
                                        />
                                    </div>

                                    // Date To
                                    <div>
                                        <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                                            "To Date"
                                        </label>
                                        <input
                                            type="date"
                                            class="w-full p-2 border border-gray-300 dark:border-gray-600 rounded bg-white dark:bg-gray-700 text-gray-900 dark:text-white"
                                            on:input=move |ev| date_to.set(event_target_value(&ev))
                                            prop:value=move || date_to.get()
                                        />
                                    </div>
                                </div>

                                <div class="mt-4 flex justify-end">
                                    <button
                                        class="px-3 py-1 text-sm bg-gray-200 dark:bg-gray-600 text-gray-700 dark:text-gray-300 rounded hover:bg-gray-300 dark:hover:bg-gray-500"
                                        on:click=move |_| {
                                            selected_status.set(None);
                                            selected_visibility.set(None);
                                            selected_tags.set(Vec::new());
                                            date_from.set(String::new());
                                            date_to.set(String::new());
                                        }
                                    >
                                        "Clear Filters"
                                    </button>
                                </div>
                            </div>
                        }.into_any()
                    } else {
                        ().into_any()
                    }
                }}

                // Category Filters
                <div class="flex gap-2 mb-6">
                    {vec![SearchCategory::All, SearchCategory::Documents, SearchCategory::Projects]
                        .into_iter()
                        .map(|cat| {
                            let label = cat.label();
                            view! {
                                <button
                                    class=move || {
                                        let current = category.get();
                                        if current == cat {
                                            "px-4 py-2 bg-blue-600 text-white rounded-lg transition-colors"
                                        } else {
                                            "px-4 py-2 bg-gray-200 dark:bg-gray-700 text-gray-700 dark:text-gray-300 rounded-lg hover:bg-gray-300 dark:hover:bg-gray-600 transition-colors"
                                        }
                                    }
                                    on:click=move |_| {
                                        category.set(cat);
                                    }
                                >
                                    {label}
                                </button>
                            }
                        })
                        .collect::<Vec<_>>()
                    }
                </div>

                // Results
                <div>
                    {move || {
                        let search_query = query.get();
                        let save_search_clone = save_search.clone();

                        if search_query.is_empty() {
                            view! {
                                <div class="bg-white dark:bg-gray-800 rounded-lg shadow border border-gray-200 dark:border-gray-700 p-6">
                                    <p class="text-gray-500 dark:text-gray-400 text-center">
                                        "Enter a search query to find documents and projects"
                                    </p>
                                </div>
                            }.into_any()
                        } else {
                            let search_query_for_inner = search_query.clone();
                            view! {
                                <Suspense fallback={view! { <SearchResultsSkeleton /> }}>
                                    {move || {
                                        let result = search_resource.get();

                                        match result {
                                            None | Some(None) => {
                                                view! {
                                                    <div class="bg-white dark:bg-gray-800 rounded-lg shadow border border-gray-200 dark:border-gray-700 p-6">
                                                        <p class="text-gray-500 dark:text-gray-400 text-center">
                                                            "Searching..."
                                                        </p>
                                                    </div>
                                                }.into_any()
                                            }
                                            Some(Some(response)) => {
                                                let docs = response.documents.results.clone();
                                                let projects = response.projects.clone();
                                                let total = response.documents.total;
                                                let facets = response.documents.facets.clone();

                                                if total == 0 && projects.is_empty() {
                                                    let sq = search_query_for_inner.clone();
                                                    view! {
                                                        <EmptySearch query=sq />
                                                    }.into_any()
                                                } else {
                                                    let save_search_for_btn = save_search_clone.clone();
                                                    view! {
                                                        <div>
                                                            // Save Search Button
                                                            {move || {
                                                                if !query.get().is_empty() {
                                                                    let save_search_fn = save_search_for_btn.clone();
                                                                    view! {
                                                                        <div class="mb-4 flex justify-end">
                                                                            <button
                                                                                class="px-3 py-1 text-sm bg-blue-600 text-white rounded hover:bg-blue-700"
                                                                                on:click=move |_| {
                                                                                    save_search_fn.clone()()
                                                                                }
                                                                            >
                                                                                "Save Search"
                                                                            </button>
                                                                        </div>
                                                                    }.into_any()
                                                                } else {
                                                                    ().into_any()
                                                                }
                                                            }}

                                                            <p class="text-sm text-gray-500 dark:text-gray-400 mb-4">
                                                                {format!("{} document result", total)}{if total != 1 { "s" } else { "" }}
                                                                {if !projects.is_empty() { format!(" and {} project(s)", projects.len()) } else { String::new() }}
                                                            </p>

                                                            // Facets
                                                            {if !facets.tags.is_empty() {
                                                                view! {
                                                                    <div class="mb-4 flex flex-wrap gap-2">
                                                                        <span class="text-sm text-gray-500 dark:text-gray-400">"Popular tags:"</span>
                                                                        {facets.tags.into_iter().take(10).map(|f| {
                                                                            let tag_value = f.value.clone();
                                                                            let tag_value_for_click = f.value.clone();
                                                                            let tag_count = f.count;
                                                                            view! {
                                                                                <button
                                                                                    class="px-2 py-1 text-xs bg-gray-100 dark:bg-gray-700 text-gray-600 dark:text-gray-300 rounded hover:bg-gray-200 dark:hover:bg-gray-600"
                                                                                    on:click=move |_| {
                                                                                        let mut tags = selected_tags.get();
                                                                                        if !tags.contains(&tag_value_for_click) {
                                                                                            tags.push(tag_value_for_click.clone());
                                                                                            selected_tags.set(tags);
                                                                                        }
                                                                                    }
                                                                                >
                                                                                    {tag_value}" ("{tag_count}")"
                                                                                </button>
                                                                            }
                                                                        }).collect::<Vec<_>>()}
                                                                    </div>
                                                                }.into_any()
                                                            } else {
                                                                ().into_any()
                                                            }}

                                                            // Document Results
                                                            {if !docs.is_empty() {
                                                                view! {
                                                                    <div class="mb-6">
                                                                        <h2 class="text-lg font-semibold text-gray-900 dark:text-white mb-3">
                                                                            "Documents"
                                                                        </h2>
                                                                        <div class="space-y-3">
                                                                            {docs.into_iter().map(|result| {
                                                                                view! {
                                                                                    <SearchResultCard result={result} />
                                                                                }
                                                                            }).collect::<Vec<_>>()}
                                                                        </div>
                                                                    </div>
                                                                }.into_any()
                                                            } else {
                                                                ().into_any()
                                                            }}

                                                            // Project Results
                                                            {if !projects.is_empty() {
                                                                view! {
                                                                    <div>
                                                                        <h2 class="text-lg font-semibold text-gray-900 dark:text-white mb-3">
                                                                            "Projects"
                                                                        </h2>
                                                                        <div class="space-y-3">
                                                                            {projects.into_iter().map(|result| {
                                                                                view! {
                                                                                    <ProjectResultCard result={result} />
                                                                                }
                                                                            }).collect::<Vec<_>>()}
                                                                        </div>
                                                                    </div>
                                                                }.into_any()
                                                            } else {
                                                                ().into_any()
                                                            }}
                                                        </div>
                                                    }.into_any()
                                                }
                                            }
                                        }
                                    }}
                                </Suspense>
                            }.into_any()
                        }
                    }}
                </div>
            </div>
        </div>
    }
}

#[component]
fn SearchResultCard(result: SearchResultItem) -> impl IntoView {
    view! {
        <div class="bg-white dark:bg-gray-800 rounded-lg shadow border border-gray-200 dark:border-gray-700 p-4 hover:border-blue-500 transition-colors cursor-pointer">
            <div class="flex items-start justify-between">
                <h3 class="text-lg font-semibold text-gray-900 dark:text-white">{result.title.clone()}</h3>
                <div class="flex items-center gap-2">
                    <span class="px-2 py-1 text-xs bg-blue-100 dark:bg-blue-900 text-blue-600 dark:text-blue-300 rounded">
                        {result.status.clone()}
                    </span>
                    <span class="text-xs text-gray-500 dark:text-gray-400">
                        "Rank: "{format!("{:.2}", result.rank)}
                    </span>
                </div>
            </div>
            // Show headline with highlighting if available
            {if let Some(ref headline) = result.headline {
                view! {
                    <div class="text-sm text-gray-600 dark:text-gray-300 mt-2"
                        inner_html={headline.clone()}
                    />
                }.into_any()
            } else if let Some(ref desc) = result.description {
                view! {
                    <p class="text-sm text-gray-600 dark:text-gray-300 mt-2">{desc.clone()}</p>
                }.into_any()
            } else {
                ().into_any()
            }}
            {if !result.tags.is_empty() {
                view! {
                    <div class="flex flex-wrap gap-2 mt-3">
                        {result.tags.into_iter().take(3).map(|tag| {
                            view! {
                                <span class="px-2 py-1 text-xs bg-gray-100 dark:bg-gray-700 text-gray-600 dark:text-gray-300 rounded">
                                    {tag}
                                </span>
                            }
                        }).collect::<Vec<_>>()}
                    </div>
                }.into_any()
            } else {
                ().into_any()
            }}
            <div class="mt-2 text-xs text-gray-400 dark:text-gray-500">
                {format!("{} words", result.word_count)}
                " · "
                {result.updated_at.clone()}
            </div>
        </div>
    }
}

#[component]
fn ProjectResultCard(result: ProjectSearchResultItem) -> impl IntoView {
    view! {
        <div class="bg-white dark:bg-gray-800 rounded-lg shadow border border-gray-200 dark:border-gray-700 p-4 hover:border-blue-500 transition-colors cursor-pointer">
            <div class="flex items-start justify-between">
                <h3 class="text-lg font-semibold text-gray-900 dark:text-white">{result.name.clone()}</h3>
                <div class="flex items-center gap-2">
                    <span class="px-2 py-1 text-xs bg-green-100 dark:bg-green-900 text-green-600 dark:text-green-300 rounded">
                        {result.project_type.clone()}
                    </span>
                    <span class="text-xs text-gray-500 dark:text-gray-400">
                        "Rank: "{format!("{:.2}", result.rank)}
                    </span>
                </div>
            </div>
            {if let Some(ref desc) = result.description {
                view! {
                    <p class="text-sm text-gray-600 dark:text-gray-300 mt-2">{desc.clone()}</p>
                }.into_any()
            } else {
                ().into_any()
            }}
            <div class="mt-2 text-xs text-gray-400 dark:text-gray-500">
                {result.status.clone()}
            </div>
        </div>
    }
}

#[component]
fn SearchResultsSkeleton() -> impl IntoView {
    view! {
        <div class="space-y-4">
            {(0..3).map(|_| {
                view! {
                    <div class="bg-white dark:bg-gray-800 rounded-lg shadow border border-gray-200 dark:border-gray-700 p-4 animate-pulse">
                        <div class="flex items-start justify-between">
                            <div class="h-5 bg-gray-200 dark:bg-gray-700 rounded w-1/3"></div>
                            <div class="h-5 bg-gray-200 dark:bg-gray-700 rounded w-16"></div>
                        </div>
                        <div class="h-4 bg-gray-200 dark:bg-gray-700 rounded w-full mt-3"></div>
                        <div class="h-4 bg-gray-200 dark:bg-gray-700 rounded w-2/3 mt-2"></div>
                        <div class="flex gap-2 mt-3">
                            <div class="h-5 bg-gray-200 dark:bg-gray-700 rounded w-12"></div>
                            <div class="h-5 bg-gray-200 dark:bg-gray-700 rounded w-16"></div>
                            <div class="h-5 bg-gray-200 dark:bg-gray-700 rounded w-14"></div>
                        </div>
                    </div>
                }
            }).collect::<Vec<_>>()}
        </div>
    }
}
