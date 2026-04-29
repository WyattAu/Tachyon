#![allow(clippy::redundant_locals)]
use crate::api::ApiClient;
use crate::components::EmptySearch;
use crate::types::{
    CreateSavedSearchRequest, GlobalSearchResponse, ProjectSearchResultItem, SearchFilters,
    SearchResultItem,
};
use leptos::prelude::*;
use std::sync::Arc;
use wasm_bindgen::JsCast;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SearchCategory {
    All,
    Documents,
    Projects,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SortBy {
    Relevance,
    Updated,
    Created,
}

#[component]
pub fn SearchPage() -> impl IntoView {
    let query = RwSignal::new(String::new());
    let category = RwSignal::new(SearchCategory::All);
    let current_page = RwSignal::new(1i64);
    let sort_by = RwSignal::new(SortBy::Relevance);
    let show_filters = RwSignal::new(false);
    let show_saved = RwSignal::new(false);

    let selected_status = RwSignal::new(None::<String>);
    let selected_visibility = RwSignal::new(None::<String>);
    let selected_tags = RwSignal::new(Vec::<String>::new());
    let date_from = RwSignal::new(String::new());
    let date_to = RwSignal::new(String::new());

    let api_client = ApiClient::default();
    let suggestions = RwSignal::new(Vec::<String>::new());
    let show_suggestions = RwSignal::new(false);

    let search_resource = LocalResource::new({
        let client = api_client.clone();
        move || {
            let client = client.clone();
            let q = query.get();
            let cat = category.get();
            let page = current_page.get();
            let _sort = sort_by.get();
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
                    client
                        .global_search(&q, Some(&f), Some(page), Some(20))
                        .await
                        .ok()
                } else {
                    client
                        .search(&q, Some(&f), Some(page), Some(20))
                        .await
                        .ok()
                        .map(|r| GlobalSearchResponse {
                            documents: r,
                            projects: vec![],
                        })
                }
            }
        }
    });

    let api_client_for_saved = api_client.clone();
    let saved_searches_resource = LocalResource::new(move || {
        let client = api_client_for_saved.clone();
        async move { client.list_saved_searches().await.unwrap_or_default() }
    });

    Effect::new(move |_| {
        let _ = query.get();
        let _ = selected_status.get();
        let _ = selected_visibility.get();
        let _ = selected_tags.get();
        let _ = date_from.get();
        let _ = date_to.get();
        current_page.set(1);
    });

    let api_client_for_input = api_client.clone();
    let on_input = move |ev| {
        let val = event_target_value(&ev);
        query.set(val.clone());
        let client = api_client_for_input.clone();
        let show_sugg = show_suggestions;
        let suggs = suggestions;
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
            let _ = window.set_timeout_with_callback_and_timeout_and_arguments_0(fn_ref, 300);
        }
        cb.forget();
    };

    let select_suggestion = move |suggestion: String| {
        query.set(suggestion);
        show_suggestions.set(false);
        suggestions.set(Vec::new());
    };

    let save_search: Arc<dyn Fn() + Send + Sync> = {
        let api_client = api_client.clone();
        let query = query;
        let selected_status = selected_status;
        let selected_visibility = selected_visibility;
        let selected_tags = selected_tags;
        let date_from = date_from;
        let date_to = date_to;
        Arc::new(move || {
            let client = api_client.clone();
            let q = query.get();
            if q.is_empty() {
                return;
            }
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
            let name = q.clone();
            let q2 = q.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let request = CreateSavedSearchRequest {
                    name,
                    query: q2,
                    filters: Some(f),
                };
                let _ = client.create_saved_search(&request).await;
            });
        })
    };

    view! {
        <div class="flex gap-6" on:click=move |_| show_suggestions.set(false)>
            {move || if show_saved.get() {
                view! {
                    <div class="w-64 flex-shrink-0">
                        <div class="bg-white dark:bg-gray-800 rounded-lg shadow border border-gray-200 dark:border-gray-700 p-4">
                            <h3 class="text-sm font-semibold text-gray-900 dark:text-white mb-3">"Saved Searches"</h3>
                            <Suspense fallback={view! { <div class="animate-pulse">"Loading..."</div> }}>
                                {move || {
                                    let saved = saved_searches_resource.get().unwrap_or_default();
                                    if saved.is_empty() {
                                        view! { <p class="text-sm text-gray-500 dark:text-gray-400">"No saved searches yet"</p> }.into_any()
                                    } else {
                                        view! {
                                            <ul class="space-y-2">
                                                {saved.into_iter().map(|s| {
                                                    let q = s.query.clone();
                                                    let f = s.filters.clone();
                                                    view! {
                                                        <li><button class="w-full text-left px-3 py-2 text-sm rounded-lg hover:bg-gray-100 dark:hover:bg-gray-700 text-gray-700 dark:text-gray-300"
                                                            on:click=move |_| {
                                                                query.set(q.clone());
                                                                if let Some(ref fl) = f {
                                                                    selected_status.set(fl.status.clone());
                                                                    selected_visibility.set(fl.visibility.clone());
                                                                    if let Some(ref t) = fl.tags { selected_tags.set(t.clone()); }
                                                                    if let Some(ref d) = fl.date_from { date_from.set(d.clone()); }
                                                                    if let Some(ref d) = fl.date_to { date_to.set(d.clone()); }
                                                                }
                                                            }>{s.name}</button></li>
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
            } else { ().into_any() }}

            <div class="flex-1">
                <div class="flex items-center justify-between mb-6">
                    <h1 class="text-2xl font-bold text-gray-900 dark:text-white">"Search"</h1>
                    <button class="text-sm text-gray-500 dark:text-gray-400 hover:text-gray-700 dark:hover:text-gray-300"
                        on:click=move |_| show_saved.update(|s| *s = !*s)>
                        {move || if show_saved.get() { "Hide Saved" } else { "Show Saved" }}
                    </button>
                </div>

                <div class="mb-4 relative">
                    <div class="flex gap-2">
                        <div class="flex-1 relative">
                            <input type="text" placeholder="Search documents and projects..." autofocus=true
                                class="w-full p-3 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-800 text-gray-900 dark:text-white focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                                on:input=on_input prop:value={move || query.get()} />
                            <div class=move || {
                                if show_suggestions.get() && !suggestions.get().is_empty() {
                                    "absolute top-full left-0 right-0 z-50 mt-1 bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-lg shadow-lg max-h-60 overflow-auto"
                                } else { "hidden" }
                            } on:click=move |ev| ev.stop_propagation()>
                                {move || {
                                    suggestions.get().into_iter().map(|s| {
                                        let sc = s.clone();
                                        let sel = select_suggestion;
                                        view! {
                                            <button class="w-full text-left px-4 py-2 text-sm text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 border-b border-gray-100 dark:border-gray-700 last:border-b-0"
                                                on:click=move |_| sel(sc.clone())>{s}</button>
                                        }
                                    }).collect::<Vec<_>>()
                                }}
                            </div>
                        </div>
                        <button class="px-4 py-2 bg-gray-200 dark:bg-gray-700 text-gray-700 dark:text-gray-300 rounded-lg hover:bg-gray-300 dark:hover:bg-gray-600"
                            on:click=move |_| show_filters.update(|f| *f = !*f)>"Filters"</button>
                    </div>
                </div>

                {move || if show_filters.get() {
                    view! {
                        <div class="mb-4 p-4 bg-gray-50 dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700">
                            <div class="grid grid-cols-2 md:grid-cols-4 gap-4">
                                <div>
                                    <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">"Status"</label>
                                    <select class="w-full p-2 border border-gray-300 dark:border-gray-600 rounded bg-white dark:bg-gray-700 text-gray-900 dark:text-white"
                                        on:change=move |ev| { let v = event_target_value(&ev); selected_status.set(if v.is_empty() { None } else { Some(v) }); }>
                                        <option value="">"Any"</option>
                                        <option value="draft">"Draft"</option>
                                        <option value="published">"Published"</option>
                                        <option value="archived">"Archived"</option>
                                    </select>
                                </div>
                                <div>
                                    <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">"Visibility"</label>
                                    <select class="w-full p-2 border border-gray-300 dark:border-gray-600 rounded bg-white dark:bg-gray-700 text-gray-900 dark:text-white"
                                        on:change=move |ev| { let v = event_target_value(&ev); selected_visibility.set(if v.is_empty() { None } else { Some(v) }); }>
                                        <option value="">"Any"</option>
                                        <option value="public">"Public"</option>
                                        <option value="private">"Private"</option>
                                    </select>
                                </div>
                                <div>
                                    <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">"From"</label>
                                    <input type="date" class="w-full p-2 border border-gray-300 dark:border-gray-600 rounded bg-white dark:bg-gray-700 text-gray-900 dark:text-white"
                                        on:input=move |ev| date_from.set(event_target_value(&ev)) prop:value={move || date_from.get()} />
                                </div>
                                <div>
                                    <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">"To"</label>
                                    <input type="date" class="w-full p-2 border border-gray-300 dark:border-gray-600 rounded bg-white dark:bg-gray-700 text-gray-900 dark:text-white"
                                        on:input=move |ev| date_to.set(event_target_value(&ev)) prop:value={move || date_to.get()} />
                                </div>
                            </div>
                            <div class="mt-3 flex justify-end">
                                <button class="px-3 py-1 text-sm bg-gray-200 dark:bg-gray-600 text-gray-700 dark:text-gray-300 rounded hover:bg-gray-300 dark:hover:bg-gray-500"
                                    on:click=move |_| { selected_status.set(None); selected_visibility.set(None); selected_tags.set(Vec::new()); date_from.set(String::new()); date_to.set(String::new()); }>"Clear"</button>
                            </div>
                        </div>
                    }.into_any()
                } else { ().into_any() }}

                <div class="flex flex-wrap items-center gap-2 mb-4">
                    <div class="flex gap-2 mr-4">
                        {vec![SearchCategory::All, SearchCategory::Documents, SearchCategory::Projects].into_iter().map(|cat| {
                            let label = match cat { SearchCategory::All => "All", SearchCategory::Documents => "Docs", SearchCategory::Projects => "Projects" };
                            let c = cat;
                            view! {
                                <button class=move || if category.get() == c { "px-3 py-1.5 bg-blue-600 text-white rounded-lg text-sm transition-colors" } else { "px-3 py-1.5 bg-gray-200 dark:bg-gray-700 text-gray-700 dark:text-gray-300 rounded-lg text-sm hover:bg-gray-300 dark:hover:bg-gray-600 transition-colors" }
                                    on:click=move |_| category.set(c)>{label}</button>
                            }
                        }).collect::<Vec<_>>()}
                    </div>
                    <select class="px-2 py-1.5 text-sm border border-gray-300 dark:border-gray-600 rounded bg-white dark:bg-gray-800 text-gray-700 dark:text-gray-300"
                        on:change=move |ev| {
                            let v = event_target_value(&ev);
                            sort_by.set(match v.as_str() { "updated" => SortBy::Updated, "created" => SortBy::Created, _ => SortBy::Relevance });
                        }>
                        <option value="relevance" selected={sort_by.get() == SortBy::Relevance}>"Relevance"</option>
                        <option value="updated" selected={sort_by.get() == SortBy::Updated}>"Last Modified"</option>
                        <option value="created" selected={sort_by.get() == SortBy::Created}>"Date Created"</option>
                    </select>
                </div>

                <div>
                    {move || {
                        let search_query = query.get();
                        let save_search_clone = save_search.clone();
                        if search_query.is_empty() {
                            view! {
                                <div class="bg-white dark:bg-gray-800 rounded-lg shadow border border-gray-200 dark:border-gray-700 p-6">
                                    <p class="text-gray-500 dark:text-gray-400 text-center">"Enter a search query to find documents and projects"</p>
                                </div>
                            }.into_any()
                        } else {
                            let sq = search_query.clone();
                            view! {
                                <Suspense fallback={view! { <SearchResultsSkeleton /> }}>
                                    {move || {
                                        let result = search_resource.get();
                                        match result {
                                            None | Some(None) => view! {
                                                <div class="bg-white dark:bg-gray-800 rounded-lg shadow border border-gray-200 dark:border-gray-700 p-6">
                                                    <p class="text-gray-500 dark:text-gray-400 text-center">"Searching..."</p>
                                                </div>
                                            }.into_any(),
                                            Some(Some(response)) => {
                                                let docs = response.documents.results.clone();
                                                let projects = response.projects.clone();
                                                let total = response.documents.total;
                                                let facets = response.documents.facets.clone();
                                                if total == 0 && projects.is_empty() {
                                                    let sq2 = sq.clone();
                                                    view! { <EmptySearch query=sq2 /> }.into_any()
                                                } else {
                                                    let save_fn = save_search_clone.clone();
                                                    view! {
                                                        <div>
                                                            {move || if !query.get().is_empty() {
                                                                let sf = save_fn.clone();
                                                                view! {
                                                                    <div class="mb-4 flex justify-end">
                                                                        <button class="px-3 py-1 text-sm bg-blue-600 text-white rounded hover:bg-blue-700"
                                                                            on:click=move |_| sf()>"Save Search"</button>
                                                                    </div>
                                                                }.into_any()
                                                            } else { ().into_any() }}
                                                            <p class="text-sm text-gray-500 dark:text-gray-400 mb-4">
                                                                {format!("{} result{}", total, if total != 1 { "s" } else { "" })}
                                                                {if !projects.is_empty() { format!(" and {} project(s)", projects.len()) } else { String::new() }}
                                                            </p>
                                                            {if !facets.tags.is_empty() {
                                                                view! {
                                                                    <div class="mb-4 flex flex-wrap gap-2">
                                                                        <span class="text-sm text-gray-500 dark:text-gray-400">"Tags:"</span>
                                                                        {facets.tags.into_iter().take(10).map(|f| {
                                                                            let label = format!("{} ({})", f.value, f.count);
                                                                            let tv = f.value.clone();
                                                                            view! {
                                                                                <button class="px-2 py-1 text-xs bg-gray-100 dark:bg-gray-700 text-gray-600 dark:text-gray-300 rounded hover:bg-gray-200 dark:hover:bg-gray-600"
                                                                                    on:click=move |_| { let mut t = selected_tags.get(); if !t.contains(&tv) { t.push(tv.clone()); } selected_tags.set(t); }>
                                                                                    {label}
                                                                                </button>
                                                                            }
                                                                        }).collect::<Vec<_>>()}
                                                                    </div>
                                                                }.into_any()
                                                            } else { ().into_any() }}
                                                            {if !docs.is_empty() {
                                                                view! {
                                                                    <div class="mb-6">
                                                                        <h2 class="text-lg font-semibold text-gray-900 dark:text-white mb-3">"Documents"</h2>
                                                                        <div class="space-y-3">
                                                                            {docs.into_iter().map(|r| view! { <SearchResultCard result={r} /> }).collect::<Vec<_>>()}
                                                                        </div>
                                                                    </div>
                                                                }.into_any()
                                                            } else { ().into_any() }}
                                                            {if !projects.is_empty() {
                                                                view! {
                                                                    <div>
                                                                        <h2 class="text-lg font-semibold text-gray-900 dark:text-white mb-3">"Projects"</h2>
                                                                        <div class="space-y-3">
                                                                            {projects.into_iter().map(|r| view! { <ProjectResultCard result={r} /> }).collect::<Vec<_>>()}
                                                                        </div>
                                                                    </div>
                                                                }.into_any()
                                                            } else { ().into_any() }}
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
    let title = result.title;
    let status = result.status;
    let rank = result.rank;
    let headline = result.headline;
    let description = result.description;
    let tags = result.tags;
    let wc = result.word_count;
    let updated = result.updated_at;
    view! {
        <div class="bg-white dark:bg-gray-800 rounded-lg shadow border border-gray-200 dark:border-gray-700 p-4 hover:border-blue-500 transition-colors cursor-pointer">
            <div class="flex items-start justify-between">
                <h3 class="text-lg font-semibold text-gray-900 dark:text-white">{title}</h3>
                <div class="flex items-center gap-2">
                    <span class="px-2 py-1 text-xs bg-blue-100 dark:bg-blue-900 text-blue-600 dark:text-blue-300 rounded">{status}</span>
                    <span class="text-xs text-gray-500 dark:text-gray-400">"Rank: "{format!("{:.2}", rank)}</span>
                </div>
            </div>
            {if let Some(ref hl) = headline {
                view! { <div class="text-sm text-gray-600 dark:text-gray-300 mt-2" inner_html={hl.clone()} /> }.into_any()
            } else if let Some(ref desc) = description {
                view! { <p class="text-sm text-gray-600 dark:text-gray-300 mt-2">{desc.clone()}</p> }.into_any()
            } else { ().into_any() }}
            {if !tags.is_empty() {
                view! {
                    <div class="flex flex-wrap gap-2 mt-3">
                        {tags.into_iter().take(3).map(|tag| {
                            view! { <span class="px-2 py-1 text-xs bg-gray-100 dark:bg-gray-700 text-gray-600 dark:text-gray-300 rounded">{tag}</span> }
                        }).collect::<Vec<_>>()}
                    </div>
                }.into_any()
            } else { ().into_any() }}
            <div class="mt-2 text-xs text-gray-400 dark:text-gray-500">{format!("{} words", wc)}" · "{updated}</div>
        </div>
    }
}

#[component]
fn ProjectResultCard(result: ProjectSearchResultItem) -> impl IntoView {
    let name = result.name;
    let ptype = result.project_type;
    let rank = result.rank;
    let desc = result.description;
    let status = result.status;
    view! {
        <div class="bg-white dark:bg-gray-800 rounded-lg shadow border border-gray-200 dark:border-gray-700 p-4 hover:border-blue-500 transition-colors cursor-pointer">
            <div class="flex items-start justify-between">
                <h3 class="text-lg font-semibold text-gray-900 dark:text-white">{name}</h3>
                <div class="flex items-center gap-2">
                    <span class="px-2 py-1 text-xs bg-green-100 dark:bg-green-900 text-green-600 dark:text-green-300 rounded">{ptype}</span>
                    <span class="text-xs text-gray-500 dark:text-gray-400">"Rank: "{format!("{:.2}", rank)}</span>
                </div>
            </div>
            {if let Some(ref d) = desc {
                view! { <p class="text-sm text-gray-600 dark:text-gray-300 mt-2">{d.clone()}</p> }.into_any()
            } else { ().into_any() }}
            <div class="mt-2 text-xs text-gray-400 dark:text-gray-500">{status}</div>
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
                        <div class="h-5 bg-gray-200 dark:bg-gray-700 rounded w-1/3"></div>
                        <div class="h-4 bg-gray-200 dark:bg-gray-700 rounded w-full mt-3"></div>
                        <div class="h-4 bg-gray-200 dark:bg-gray-700 rounded w-2/3 mt-2"></div>
                    </div>
                }
            }).collect::<Vec<_>>()}
        </div>
    }
}
