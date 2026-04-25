use crate::api::ApiClient;
use crate::types::{ActivityListResponse, Document};
use chrono::Timelike;
use leptos::prelude::*;
use leptos_router::hooks::use_navigate;

fn format_relative_time(ts: &str) -> String {
    let dt = chrono::DateTime::parse_from_rfc3339(ts);
    let Ok(past) = dt else {
        return ts.split('T').next().unwrap_or("Unknown").to_string();
    };
    let dur = chrono::Utc::now().signed_duration_since(past.with_timezone(&chrono::Utc));
    if dur.num_seconds() < 60 {
        "just now".into()
    } else if dur.num_minutes() < 60 {
        format!("{}m ago", dur.num_minutes())
    } else if dur.num_hours() < 24 {
        format!("{}h ago", dur.num_hours())
    } else if dur.num_days() < 7 {
        format!("{}d ago", dur.num_days())
    } else {
        past.format("%b %d").to_string()
    }
}

fn status_class(s: &str) -> &'static str {
    match s.to_lowercase().as_str() {
        "published" => "bg-green-100 text-green-700 dark:bg-green-900/40 dark:text-green-400",
        "draft" => "bg-yellow-100 text-yellow-700 dark:bg-yellow-900/40 dark:text-yellow-400",
        "archived" => "bg-gray-200 text-gray-600 dark:bg-gray-700 dark:text-gray-400",
        _ => "bg-blue-100 text-blue-700 dark:bg-blue-900/40 dark:text-blue-400",
    }
}

#[component]
pub fn DashboardPage() -> impl IntoView {
    let navigate = use_navigate();
    let nav_doc = navigate.clone();
    let nav_space = navigate.clone();
    let nav_import = navigate.clone();

    let on_new_doc = Callback::new(move |_: leptos::ev::MouseEvent| {
        let _ = nav_doc("/documents", Default::default());
    });
    let on_new_space = Callback::new(move |_: leptos::ev::MouseEvent| {
        let _ = nav_space("/spaces", Default::default());
    });
    let on_import = Callback::new(move |_: leptos::ev::MouseEvent| {
        let _ = nav_import("/documents", Default::default());
    });

    let api = ApiClient::default();
    let api_docs = api.clone();
    let api_spaces = api.clone();
    let api_activity = api.clone();

    let docs_res = LocalResource::new(move || {
        let c = api_docs.clone();
        async move {
            c.list_documents(Some(1), Some(8))
                .await
                .map(|r| r.results)
                .unwrap_or_default()
        }
    });

    let spaces_res = LocalResource::new(move || {
        let c = api_spaces.clone();
        async move { c.list_spaces(None).await.unwrap_or_default() }
    });

    let activity_res = LocalResource::new(move || {
        let c = api_activity.clone();
        async move {
            c.list_activity(Some(10), None)
                .await
                .unwrap_or(ActivityListResponse::default())
        }
    });

    let stat_fb1 = view! { <StatSkeleton /> };
    let stat_fb2 = view! { <StatSkeleton /> };
    let stat_fb3 = view! { <StatSkeleton /> };
    let doc_fb = view! { <DocListSkeleton /> };

    let hour = chrono::Utc::now().hour();
    let greeting = if hour < 12 {
        "Good morning"
    } else if hour < 17 {
        "Good afternoon"
    } else {
        "Good evening"
    };

    view! {
        <div class="max-w-6xl mx-auto px-4 py-6 sm:px-6 lg:px-8">
            <div class="mb-8">
                <h1 class="text-2xl font-bold text-gray-900 dark:text-white">
                    {greeting}", welcome back"
                </h1>
                <p class="mt-1 text-sm text-gray-500 dark:text-gray-400">
                    "Here's what's happening with your knowledge base."
                </p>
            </div>

            <div class="mb-8 flex flex-wrap gap-3">
                <button on:click={move |ev| on_new_doc.run(ev)}
                    class="inline-flex items-center gap-2 px-4 py-2.5 bg-blue-600 hover:bg-blue-700 text-white text-sm font-medium rounded-lg transition-colors shadow-sm">
                    <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
                        <path stroke-linecap="round" stroke-linejoin="round" d="M12 4v16m8-8H4"/>
                    </svg>
                    "New Document"
                </button>
                <button on:click={move |ev| on_new_space.run(ev)}
                    class="inline-flex items-center gap-2 px-4 py-2.5 bg-white dark:bg-gray-800 text-gray-700 dark:text-gray-200 text-sm font-medium rounded-lg border border-gray-200 dark:border-gray-700 hover:bg-gray-50 dark:hover:bg-gray-700 transition-colors shadow-sm">
                    <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
                        <path stroke-linecap="round" stroke-linejoin="round" d="M4 6a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2H6a2 2 0 01-2-2V6zm10 0a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2h-2a2 2 0 01-2-2V6zM4 16a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2H6a2 2 0 01-2-2v-2zm10 0a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2h-2a2 2 0 01-2-2v-2z"/>
                    </svg>
                    "New Space"
                </button>
                <button on:click={move |ev| on_import.run(ev)}
                    class="inline-flex items-center gap-2 px-4 py-2.5 bg-white dark:bg-gray-800 text-gray-700 dark:text-gray-200 text-sm font-medium rounded-lg border border-gray-200 dark:border-gray-700 hover:bg-gray-50 dark:hover:bg-gray-700 transition-colors shadow-sm">
                    <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
                        <path stroke-linecap="round" stroke-linejoin="round" d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-8l-4-4m0 0L8 8m4-4v12"/>
                    </svg>
                    "Import"
                </button>
            </div>

            <div class="grid grid-cols-1 sm:grid-cols-3 gap-4 mb-8">
                <Suspense fallback={stat_fb1}>
                    {move || {
                        docs_res.get().map(|docs| {
                            view! { <StatCard label="Total Documents" value=docs.len() icon="doc" /> }
                        })
                    }}
                </Suspense>
                <Suspense fallback={stat_fb2}>
                    {move || {
                        activity_res.get().map(|a| {
                            view! { <StatCard label="Recent Edits" value=a.events.len() icon="edit" /> }
                        })
                    }}
                </Suspense>
                <Suspense fallback={stat_fb3}>
                    {move || {
                        spaces_res.get().map(|s| {
                            view! { <StatCard label="Spaces" value=s.len() icon="space" /> }
                        })
                    }}
                </Suspense>
            </div>

            <div class="mb-8">
                <div class="flex items-center justify-between mb-4">
                    <h2 class="text-lg font-semibold text-gray-900 dark:text-white">"Recent Documents"</h2>
                    <a href="/documents" class="text-sm text-blue-600 dark:text-blue-400 hover:underline">"View all"</a>
                </div>
                <Suspense fallback={doc_fb}>
                    {move || {
                        docs_res.get().map(|docs| {
                            if docs.is_empty() {
                                view! {
                                    <div class="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 p-12 text-center">
                                        <svg class="mx-auto h-12 w-12 text-gray-300 dark:text-gray-600" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="1">
                                            <path stroke-linecap="round" stroke-linejoin="round" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"/>
                                        </svg>
                                        <p class="mt-3 text-sm text-gray-500 dark:text-gray-400">"No documents yet"</p>
                                        <p class="mt-1 text-xs text-gray-400 dark:text-gray-500">"Create your first document to get started"</p>
                                    </div>
                                }.into_any()
                            } else {
                                view! {
                                    <div class="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 divide-y divide-gray-100 dark:divide-gray-700">
                                        {docs.into_iter().take(5).map(|d| {
                                            view! { <DocRow document=d /> }
                                        }).collect::<Vec<_>>()}
                                    </div>
                                }.into_any()
                            }
                        })
                    }}
                </Suspense>
            </div>
        </div>
    }
}

#[component]
fn StatCard(label: &'static str, value: usize, icon: &'static str) -> impl IntoView {
    let path = match icon {
        "doc" => "M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z",
        "edit" => "M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z",
        "space" => "M4 6a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2H6a2 2 0 01-2-2V6zm10 0a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2h-2a2 2 0 01-2-2V6zM4 16a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2H6a2 2 0 01-2-2v-2zm10 0a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2h-2a2 2 0 01-2-2v-2z",
        _ => "M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z",
    };
    view! {
        <div class="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 p-5">
            <div class="flex items-center gap-3">
                <div class="flex-shrink-0 p-2 bg-blue-50 dark:bg-blue-900/30 rounded-lg">
                    <svg class="w-5 h-5 text-blue-600 dark:text-blue-400" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
                        <path stroke-linecap="round" stroke-linejoin="round" d={path} />
                    </svg>
                </div>
                <div>
                    <p class="text-sm text-gray-500 dark:text-gray-400">{label}</p>
                    <p class="text-2xl font-bold text-gray-900 dark:text-white">{value}</p>
                </div>
            </div>
        </div>
    }
}

#[component]
fn StatSkeleton() -> impl IntoView {
    view! {
        <div class="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 p-5 animate-pulse">
            <div class="flex items-center gap-3">
                <div class="w-10 h-10 bg-gray-200 dark:bg-gray-700 rounded-lg"></div>
                <div class="space-y-2 flex-1">
                    <div class="h-3 bg-gray-200 dark:bg-gray-700 rounded w-24"></div>
                    <div class="h-7 bg-gray-200 dark:bg-gray-700 rounded w-12"></div>
                </div>
            </div>
        </div>
    }
}

#[component]
fn DocRow(document: Document) -> impl IntoView {
    let excerpt = document.content.chars().take(100).collect::<String>();
    let excerpt = if document.content.len() > 100 {
        format!("{}...", excerpt)
    } else {
        excerpt
    };
    let time = format_relative_time(&document.updated_at);
    let cls = status_class(&document.status);

    view! {
        <div class="flex items-center gap-4 px-5 py-3.5 hover:bg-gray-50 dark:hover:bg-gray-700/30 transition-colors cursor-pointer">
            <div class="flex-1 min-w-0">
                <p class="text-sm font-medium text-gray-900 dark:text-white truncate">
                    {document.title}
                </p>
                <p class="text-xs text-gray-500 dark:text-gray-400 mt-0.5 truncate">
                    {excerpt}
                </p>
            </div>
            <div class="flex items-center gap-3 flex-shrink-0">
                <span class={format!("px-2 py-0.5 text-xs font-medium rounded-full {}", cls)}>
                    {document.status}
                </span>
                <span class="text-xs text-gray-400 dark:text-gray-500 w-16 text-right">
                    {time}
                </span>
            </div>
        </div>
    }
}

#[component]
fn DocListSkeleton() -> impl IntoView {
    view! {
        <div class="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 divide-y divide-gray-100 dark:divide-gray-700">
            {(0..4).map(|_| {
                view! {
                    <div class="flex items-center gap-4 px-5 py-3.5 animate-pulse">
                        <div class="flex-1 space-y-2">
                            <div class="h-4 bg-gray-200 dark:bg-gray-700 rounded w-1/3"></div>
                            <div class="h-3 bg-gray-200 dark:bg-gray-700 rounded w-2/3"></div>
                        </div>
                        <div class="h-5 bg-gray-200 dark:bg-gray-700 rounded w-16"></div>
                        <div class="h-3 bg-gray-200 dark:bg-gray-700 rounded w-14"></div>
                    </div>
                }
            }).collect::<Vec<_>>()}
        </div>
    }
}
