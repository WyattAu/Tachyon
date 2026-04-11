// Dashboard Page
// Main dashboard with stats, quick actions, and recent items

use crate::api::ApiClient;
use crate::types::{ActivityEvent, ActivityListResponse, CatalogStats, Document, Project, TagsResponse};
use crate::components::{BreadcrumbItem, Breadcrumbs};
use leptos::prelude::*;
use leptos_router::hooks::use_navigate;

fn format_relative_time(timestamp: &str) -> String {
    let dt = chrono::DateTime::parse_from_rfc3339(timestamp);
    let Ok(past) = dt else {
        return timestamp.split('T').next().unwrap_or("Unknown").to_string();
    };
    let now = chrono::Utc::now();
    let duration = now.signed_duration_since(past.with_timezone(&chrono::Utc));

    if duration.num_seconds() < 60 {
        "just now".to_string()
    } else if duration.num_minutes() < 60 {
        format!("{}m ago", duration.num_minutes())
    } else if duration.num_hours() < 24 {
        format!("{}h ago", duration.num_hours())
    } else if duration.num_days() < 7 {
        format!("{}d ago", duration.num_days())
    } else {
        past.format("%b %d").to_string()
    }
}

fn event_type_dot_color(event_type: &str) -> &'static str {
    if event_type.contains("created") {
        "bg-green-500"
    } else if event_type.contains("updated") || event_type.contains("edited") {
        "bg-blue-500"
    } else if event_type.contains("deleted") || event_type.contains("removed") {
        "bg-red-500"
    } else if event_type.contains("published") {
        "bg-emerald-500"
    } else {
        "bg-gray-400"
    }
}

#[component]
pub fn DashboardPage() -> impl IntoView {
    let navigate = use_navigate();
    let nav_for_project = navigate.clone();
    let nav_for_document = navigate.clone();
    let on_click_new_project = Callback::new(move |_: leptos::ev::MouseEvent| {
        let _ = nav_for_project("/catalog", Default::default());
    });
    let on_click_new_document = Callback::new(move |_: leptos::ev::MouseEvent| {
        let _ = nav_for_document("/documents", Default::default());
    });
    let on_click_search = Callback::new(move |_: leptos::ev::MouseEvent| {
        let _ = navigate("/search", Default::default());
    });

    let api_client = ApiClient::default();
    let api_client_stats = api_client.clone();
    let api_client_projects = api_client.clone();
    let api_client_documents = api_client.clone();
    let api_client_activity = api_client.clone();
    let api_client_tags = api_client.clone();

    let stats_resource = LocalResource::new(move || {
        let client = api_client_stats.clone();
        async move {
            client.get_catalog_stats().await.unwrap_or(CatalogStats {
                project_count: 0,
                component_count: 0,
                member_count: 0,
            })
        }
    });

    let projects_resource = LocalResource::new(move || {
        let client = api_client_projects.clone();
        async move {
            client.list_projects().await.unwrap_or_default()
        }
    });

    let documents_resource = LocalResource::new(move || {
        let client = api_client_documents.clone();
        async move {
            client
                .list_documents(Some(1), Some(5))
                .await
                .map(|r| r.results)
                .unwrap_or_default()
        }
    });

    let activity_resource = LocalResource::new(move || {
        let client = api_client_activity.clone();
        async move {
            client.list_activity(Some(20), None).await.unwrap_or(ActivityListResponse::default())
        }
    });

    let tags_resource = LocalResource::new(move || {
        let client = api_client_tags.clone();
        async move {
            client.list_tags().await.unwrap_or(TagsResponse::default())
        }
    });

    view! {
        <div>
            <Breadcrumbs items={vec![
                BreadcrumbItem { label: "Dashboard".into(), href: None },
            ]}/>
            <h1 class="text-2xl font-bold mb-6 text-gray-900 dark:text-white">"Dashboard"</h1>

            // Stats Overview
            <div class="grid grid-cols-1 md:grid-cols-3 gap-4 mb-8">
                <Suspense fallback={view! { <StatsCardSkeleton /> }}>
                    {move || {
                        stats_resource.get().map(|stats| {
                            view! {
                                <StatsCard
                                    label="Projects"
                                    value={stats.project_count.to_string()}
                                    description="Total projects"
                                    icon="folder"
                                />
                            }
                        })
                    }}
                </Suspense>

                <Suspense fallback={view! { <StatsCardSkeleton /> }}>
                    {move || {
                        stats_resource.get().map(|stats| {
                            view! {
                                <StatsCard
                                    label="Components"
                                    value={stats.component_count.to_string()}
                                    description="Total components"
                                    icon="cube"
                                />
                            }
                        })
                    }}
                </Suspense>

                <Suspense fallback={view! { <StatsCardSkeleton /> }}>
                    {move || {
                        stats_resource.get().map(|stats| {
                            view! {
                                <StatsCard
                                    label="Members"
                                    value={stats.member_count.to_string()}
                                    description="Team members"
                                    icon="users"
                                />
                            }
                        })
                    }}
                </Suspense>
            </div>

            // Quick Actions
            <div class="mb-8">
                <h2 class="text-lg font-semibold text-gray-900 dark:text-white mb-4">"Quick Actions"</h2>
                <div class="flex flex-wrap gap-3">
                    <QuickActionButton label="New Project" icon="plus" on_click={on_click_new_project} />
                    <QuickActionButton label="New Document" icon="document" on_click={on_click_new_document} />
                    <QuickActionButton label="Search" icon="search" on_click={on_click_search} />
                </div>
            </div>

            // Popular Tags
            <div class="mb-8">
                <h2 class="text-lg font-semibold text-gray-900 dark:text-white mb-4">"Popular Tags"</h2>
                <Suspense fallback={view! { <div class="animate-pulse h-8 bg-gray-200 dark:bg-gray-700 rounded w-64"></div> }}>
                    {move || tags_resource.get().map(|tags_response| {
                        if tags_response.tags.is_empty() {
                            view! {
                                <p class="text-sm text-gray-500 dark:text-gray-400">"No tags yet"</p>
                            }.into_any()
                        } else {
                            view! {
                                <div class="flex flex-wrap gap-2">
                                    {tags_response.tags.into_iter().map(|tag_info| {
                                        view! {
                                            <span class="inline-flex items-center px-3 py-1 rounded-full text-sm bg-gray-100 dark:bg-gray-700 text-gray-700 dark:text-gray-300 hover:bg-blue-100 dark:hover:bg-blue-900 hover:text-blue-700 dark:hover:text-blue-300 cursor-pointer transition-colors">
                                                {tag_info.tag}
                                                <span class="ml-1.5 text-xs text-gray-400">{tag_info.count}</span>
                                            </span>
                                        }
                                    }).collect::<Vec<_>>()}
                                </div>
                            }.into_any()
                        }
                    })}
                </Suspense>
            </div>

            // Recent Activity
            <div class="mb-8">
                <h2 class="text-lg font-semibold text-gray-900 dark:text-white mb-4">"Recent Activity"</h2>
                <div class="bg-white dark:bg-gray-800 rounded-lg shadow border border-gray-200 dark:border-gray-700">
                    <Suspense fallback={view! { <ActivityListSkeleton /> }}>
                        {move || {
                            activity_resource.get().map(|activity| {
                                if activity.events.is_empty() {
                                    view! {
                                        <div class="p-6 text-center">
                                            <p class="text-gray-500 dark:text-gray-400">"No recent activity"</p>
                                        </div>
                                    }.into_any()
                                } else {
                                    view! {
                                        <div class="divide-y divide-gray-200 dark:divide-gray-700">
                                            {activity.events.into_iter().map(|event| {
                                                view! {
                                                    <ActivityEventRow event=event />
                                                }
                                            }).collect::<Vec<_>>()}
                                        </div>
                                    }.into_any()
                                }
                            })
                        }}
                    </Suspense>
                </div>
            </div>

            // Recent Documents and Projects
            <div class="grid grid-cols-1 lg:grid-cols-2 gap-6">
                // Recent Documents
                <div>
                    <div class="flex items-center justify-between mb-4">
                        <h2 class="text-lg font-semibold text-gray-900 dark:text-white">"Recent Documents"</h2>
                        <a href="/documents" class="text-sm text-blue-600 dark:text-blue-400 hover:underline">"View all"</a>
                    </div>
                    <Suspense fallback={view! { <DocumentListSkeleton /> }}>
                        {move || {
                            documents_resource.get().map(|documents| {
                                if documents.is_empty() {
                                    view! {
                                        <div class="bg-white dark:bg-gray-800 rounded-lg shadow border border-gray-200 dark:border-gray-700 p-6">
                                            <p class="text-gray-500 dark:text-gray-400 text-center">"No documents yet"</p>
                                        </div>
                                    }.into_any()
                                } else {
                                    view! {
                                        <div class="space-y-3">
                                            {documents.into_iter().take(5).map(|doc| {
                                                view! {
                                                    <DocumentListItem document={doc} />
                                                }
                                            }).collect::<Vec<_>>()}
                                        </div>
                                    }.into_any()
                                }
                            })
                        }}
                    </Suspense>
                </div>

                // Recent Projects
                <div>
                    <div class="flex items-center justify-between mb-4">
                        <h2 class="text-lg font-semibold text-gray-900 dark:text-white">"Recent Projects"</h2>
                        <a href="/catalog" class="text-sm text-blue-600 dark:text-blue-400 hover:underline">"View all"</a>
                    </div>
                    <Suspense fallback={view! { <ProjectListSkeleton /> }}>
                        {move || {
                            projects_resource.get().map(|projects| {
                                if projects.is_empty() {
                                    view! {
                                        <div class="bg-white dark:bg-gray-800 rounded-lg shadow border border-gray-200 dark:border-gray-700 p-6">
                                            <p class="text-gray-500 dark:text-gray-400 text-center">"No projects yet"</p>
                                        </div>
                                    }.into_any()
                                } else {
                                    view! {
                                        <div class="space-y-3">
                                            {projects.into_iter().take(5).map(|project| {
                                                view! {
                                                    <ProjectListItem project={project} />
                                                }
                                            }).collect::<Vec<_>>()}
                                        </div>
                                    }.into_any()
                                }
                            })
                        }}
                    </Suspense>
                </div>
            </div>
        </div>
    }
}

#[component]
fn ActivityEventRow(event: ActivityEvent) -> impl IntoView {
    let dot_color = event_type_dot_color(&event.event_type);
    let relative_time = format_relative_time(&event.created_at);

    view! {
        <div class="flex items-center gap-3 px-4 py-3 hover:bg-gray-50 dark:hover:bg-gray-700/50 transition-colors">
            <span class={format!("flex-shrink-0 w-2.5 h-2.5 rounded-full {}", dot_color)}></span>
            <div class="flex-1 min-w-0">
                <p class="text-sm text-gray-900 dark:text-white truncate">{event.description}</p>
            </div>
            <span class="flex-shrink-0 text-xs text-gray-400 dark:text-gray-500">{relative_time}</span>
        </div>
    }
}

#[component]
fn ActivityListSkeleton() -> impl IntoView {
    view! {
        <div class="divide-y divide-gray-200 dark:divide-gray-700">
            {(0..5).map(|_| {
                view! {
                    <div class="flex items-center gap-3 px-4 py-3 animate-pulse">
                        <div class="flex-shrink-0 w-2.5 h-2.5 bg-gray-200 dark:bg-gray-700 rounded-full"></div>
                        <div class="flex-1 h-4 bg-gray-200 dark:bg-gray-700 rounded w-3/4"></div>
                        <div class="flex-shrink-0 h-3 bg-gray-200 dark:bg-gray-700 rounded w-16"></div>
                    </div>
                }
            }).collect::<Vec<_>>()}
        </div>
    }
}

#[component]
fn StatsCard(label: &'static str, value: String, description: &'static str, icon: &'static str) -> impl IntoView {
    let icon_svg = match icon {
        "folder" => r#"<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z" />"#,
        "cube" => r#"<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M20 7l-8-4-8 4m16 0l-8 4m8-4v10l-8 4m0-10L4 7m8 4v10M4 7v10l8 4" />"#,
        "users" => r#"<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4.354a4 4 0 110 5.292M15 21H3v-1a6 6 0 0112 0v1zm0 0h6v-1a6 6 0 00-9-5.197M13 7a4 4 0 11-8 0 4 4 0 018 0z" />"#,
        _ => r#"<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />"#,
    };

    view! {
        <div class="bg-white dark:bg-gray-800 rounded-lg shadow p-6 border border-gray-200 dark:border-gray-700">
            <div class="flex items-center">
                <div class="flex-shrink-0">
                    <svg class="h-8 w-8 text-blue-600 dark:text-blue-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                        {icon_svg}
                    </svg>
                </div>
                <div class="ml-4">
                    <div class="text-sm text-gray-500 dark:text-gray-400">{label}</div>
                    <div class="text-2xl font-bold text-gray-900 dark:text-white">{value}</div>
                    <div class="text-sm text-gray-400 mt-1">{description}</div>
                </div>
            </div>
        </div>
    }
}

#[component]
fn StatsCardSkeleton() -> impl IntoView {
    view! {
        <div class="bg-white dark:bg-gray-800 rounded-lg shadow p-6 border border-gray-200 dark:border-gray-700 animate-pulse">
            <div class="flex items-center">
                <div class="h-8 w-8 bg-gray-200 dark:bg-gray-700 rounded"></div>
                <div class="ml-4 flex-1">
                    <div class="h-4 bg-gray-200 dark:bg-gray-700 rounded w-1/2"></div>
                    <div class="h-6 bg-gray-200 dark:bg-gray-700 rounded w-3/4 mt-2"></div>
                    <div class="h-3 bg-gray-200 dark:bg-gray-700 rounded w-2/3 mt-2"></div>
                </div>
            </div>
        </div>
    }
}

#[component]
fn QuickActionButton(label: &'static str, icon: &'static str, on_click: Callback<leptos::ev::MouseEvent>) -> impl IntoView {
    let icon_svg = match icon {
        "plus" => r#"<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />"#,
        "document" => r#"<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />"#,
        "search" => r#"<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />"#,
        _ => r#"<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />"#,
    };

    view! {
        <button on:click={move |ev| on_click.run(ev)} class="flex items-center gap-2 px-4 py-2 bg-white dark:bg-gray-800 text-gray-700 dark:text-gray-200 rounded-lg border border-gray-200 dark:border-gray-700 hover:bg-gray-50 dark:hover:bg-gray-700 hover:border-blue-500 dark:hover:border-blue-400 transition-colors shadow-sm">
            <svg class="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                {icon_svg}
            </svg>
            <span>{label}</span>
        </button>
    }
}

#[component]
fn DocumentListItem(document: Document) -> impl IntoView {
    let truncated_content = if document.content.len() > 80 {
        format!("{}...", &document.content.chars().take(80).collect::<String>())
    } else {
        document.content.clone()
    };

    view! {
        <div class="bg-white dark:bg-gray-800 rounded-lg shadow border border-gray-200 dark:border-gray-700 p-4 hover:border-blue-500 transition-colors cursor-pointer">
            <div class="flex items-start justify-between">
                <div class="flex-1 min-w-0">
                    <h3 class="text-sm font-medium text-gray-900 dark:text-white truncate">{document.title}</h3>
                    <p class="text-xs text-gray-500 dark:text-gray-400 mt-1 truncate">{truncated_content}</p>
                </div>
                <span class="ml-2 px-2 py-1 text-xs font-medium rounded bg-blue-100 dark:bg-blue-900 text-blue-600 dark:text-blue-300">
                    {document.status}
                </span>
            </div>
            <div class="flex items-center gap-2 mt-2">
                <span class="text-xs text-gray-400">{document.word_count}" words"</span>
                <span class="text-xs text-gray-400">"·"</span>
                <span class="text-xs text-gray-400">{document.updated_at}</span>
            </div>
        </div>
    }
}

#[component]
fn ProjectListItem(project: Project) -> impl IntoView {
    let description = project.description.clone().unwrap_or_else(|| "No description".to_string());
    let truncated_desc = if description.len() > 60 {
        format!("{}...", &description[..60])
    } else {
        description
    };

    view! {
        <div class="bg-white dark:bg-gray-800 rounded-lg shadow border border-gray-200 dark:border-gray-700 p-4 hover:border-blue-500 transition-colors cursor-pointer">
            <div class="flex items-start justify-between">
                <div class="flex-1 min-w-0">
                    <h3 class="text-sm font-medium text-gray-900 dark:text-white truncate">{project.name}</h3>
                    <p class="text-xs text-gray-500 dark:text-gray-400 mt-1 truncate">{truncated_desc}</p>
                </div>
                <span class="ml-2 px-2 py-1 text-xs font-medium rounded bg-green-100 dark:bg-green-900 text-green-600 dark:text-green-300">
                    {project.project_type}
                </span>
            </div>
            <div class="flex items-center gap-2 mt-2">
                {project.language.map(|lang| {
                    view! {
                        <span class="px-2 py-0.5 text-xs bg-blue-50 dark:bg-blue-900/50 text-blue-600 dark:text-blue-300 rounded">
                            {lang}
                        </span>
                    }
                })}
                {project.framework.map(|fw| {
                    view! {
                        <span class="px-2 py-0.5 text-xs bg-purple-50 dark:bg-purple-900/50 text-purple-600 dark:text-purple-300 rounded">
                            {fw}
                        </span>
                    }
                })}
            </div>
        </div>
    }
}

#[component]
fn DocumentListSkeleton() -> impl IntoView {
    view! {
        <div class="space-y-3">
            {(0..3).map(|_| {
                view! {
                    <div class="bg-white dark:bg-gray-800 rounded-lg shadow border border-gray-200 dark:border-gray-700 p-4 animate-pulse">
                        <div class="flex items-start justify-between">
                            <div class="flex-1">
                                <div class="h-4 bg-gray-200 dark:bg-gray-700 rounded w-3/4"></div>
                                <div class="h-3 bg-gray-200 dark:bg-gray-700 rounded w-full mt-2"></div>
                            </div>
                            <div class="h-5 bg-gray-200 dark:bg-gray-700 rounded w-16"></div>
                        </div>
                        <div class="flex gap-2 mt-2">
                            <div class="h-3 bg-gray-200 dark:bg-gray-700 rounded w-16"></div>
                            <div class="h-3 bg-gray-200 dark:bg-gray-700 rounded w-24"></div>
                        </div>
                    </div>
                }
            }).collect::<Vec<_>>()}
        </div>
    }
}

#[component]
fn ProjectListSkeleton() -> impl IntoView {
    view! {
        <div class="space-y-3">
            {(0..3).map(|_| {
                view! {
                    <div class="bg-white dark:bg-gray-800 rounded-lg shadow border border-gray-200 dark:border-gray-700 p-4 animate-pulse">
                        <div class="flex items-start justify-between">
                            <div class="flex-1">
                                <div class="h-4 bg-gray-200 dark:bg-gray-700 rounded w-3/4"></div>
                                <div class="h-3 bg-gray-200 dark:bg-gray-700 rounded w-full mt-2"></div>
                            </div>
                            <div class="h-5 bg-gray-200 dark:bg-gray-700 rounded w-16"></div>
                        </div>
                        <div class="flex gap-2 mt-2">
                            <div class="h-4 bg-gray-200 dark:bg-gray-700 rounded w-12"></div>
                            <div class="h-4 bg-gray-200 dark:bg-gray-700 rounded w-16"></div>
                        </div>
                    </div>
                }
            }).collect::<Vec<_>>()}
        </div>
    }
}
