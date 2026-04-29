// Catalog Page
// Project/Service Catalog page with API integration

use crate::api::ApiClient;
use crate::types::{CatalogStats, Project};
use leptos::prelude::*;

/// Catalog page component - displays project catalog
#[component]
pub fn CatalogPage() -> impl IntoView {
    // Create API client - clone for each resource
    let api_client = ApiClient::default();
    let api_client_stats = api_client.clone();
    let api_client_projects = api_client.clone();

    // Fetch catalog stats
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

    // Fetch projects
    let projects_resource = LocalResource::new(move || {
        let client = api_client_projects.clone();
        async move { client.list_projects().await.unwrap_or_default() }
    });

    view! {
        <div>
            <h1 class="text-2xl font-bold mb-6 text-gray-900 dark:text-white">"Catalog"</h1>

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
                                    description="Total members"
                                />
                            }
                        })
                    }}
                </Suspense>
            </div>

            // Projects Section
            <div class="mb-6">
                <div class="flex items-center justify-between mb-4">
                    <h2 class="text-xl font-semibold text-gray-900 dark:text-white">"Projects"</h2>
                    <button class="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors">
                        "+ New Project"
                    </button>
                </div>

                // Project Cards Grid
                <Suspense fallback={view! { <ProjectsGridSkeleton /> }}>
                    {move || {
                        projects_resource.get().map(|projects| {
                            if projects.is_empty() {
                                view! {
                                    <div class="bg-white dark:bg-gray-800 rounded-lg shadow border border-gray-200 dark:border-gray-700 p-6">
                                        <p class="text-gray-500 dark:text-gray-400 text-center">
                                            "No projects found. Create your first project to get started!"
                                        </p>
                                    </div>
                                }.into_any()
                            } else {
                                view! {
                                    <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
                                        {projects.into_iter().map(|project| {
                                            view! {
                                                <ProjectCard project={project} />
                                            }
                                        }).collect::<Vec<_>>()}
                                    </div>
                                }.into_any()
                            }
                        })
                    }}
                </Suspense>
            </div>

            // Components Section
            <div>
                <div class="flex items-center justify-between mb-4">
                    <h2 class="text-xl font-semibold text-gray-900 dark:text-white">"Components"</h2>
                    <button class="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors">
                        "+ New Component"
                    </button>
                </div>

                <div class="bg-white dark:bg-gray-800 rounded-lg shadow border border-gray-200 dark:border-gray-700 p-6">
                    <p class="text-gray-500 dark:text-gray-400">
                        "No components registered yet. Components will appear here when added to projects."
                    </p>
                </div>
            </div>
        </div>
    }
}

/// Stats card component
#[component]
fn StatsCard(label: &'static str, value: String, description: &'static str) -> impl IntoView {
    view! {
        <div class="bg-white dark:bg-gray-800 rounded-lg shadow p-6 border border-gray-200 dark:border-gray-700">
            <div class="text-sm text-gray-500 dark:text-gray-400">{label}</div>
            <div class="text-2xl font-bold text-gray-900 dark:text-white mt-1">{value}</div>
            <div class="text-sm text-gray-400 mt-1">{description}</div>
        </div>
    }
}

/// Stats card skeleton for loading state
#[component]
fn StatsCardSkeleton() -> impl IntoView {
    view! {
        <div class="bg-white dark:bg-gray-800 rounded-lg shadow p-6 border border-gray-200 dark:border-gray-700 animate-pulse">
            <div class="h-4 bg-gray-200 dark:bg-gray-700 rounded w-1/2"></div>
            <div class="h-8 bg-gray-200 dark:bg-gray-700 rounded w-3/4 mt-2"></div>
            <div class="h-3 bg-gray-200 dark:bg-gray-700 rounded w-2/3 mt-2"></div>
        </div>
    }
}

/// Projects grid skeleton for loading state
#[component]
fn ProjectsGridSkeleton() -> impl IntoView {
    view! {
        <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
            {(0..3).map(|_| {
                view! {
                    <div class="bg-white dark:bg-gray-800 rounded-lg shadow border border-gray-200 dark:border-gray-700 p-6 animate-pulse">
                        <div class="h-5 bg-gray-200 dark:bg-gray-700 rounded w-3/4"></div>
                        <div class="h-3 bg-gray-200 dark:bg-gray-700 rounded w-1/2 mt-2"></div>
                        <div class="h-4 bg-gray-200 dark:bg-gray-700 rounded w-full mt-4"></div>
                        <div class="flex gap-2 mt-4">
                            <div class="h-5 bg-gray-200 dark:bg-gray-700 rounded w-12"></div>
                            <div class="h-5 bg-gray-200 dark:bg-gray-700 rounded w-16"></div>
                        </div>
                    </div>
                }
            }).collect::<Vec<_>>()}
        </div>
    }
}

/// Project card component
#[component]
fn ProjectCard(project: Project) -> impl IntoView {
    let tags = project.tags.clone();
    let description = project
        .description
        .clone()
        .unwrap_or_else(|| "No description".to_string());
    let truncated_desc = if description.len() > 100 {
        format!("{}...", &description[..100])
    } else {
        description
    };

    view! {
        <div class="bg-white dark:bg-gray-800 rounded-lg shadow border border-gray-200 dark:border-gray-700 p-6 hover:border-blue-500 transition-colors cursor-pointer">
            <h3 class="text-lg font-semibold text-gray-900 dark:text-white">{project.name}</h3>
            <p class="text-sm text-gray-500 dark:text-gray-400">{project.project_type}</p>
            <p class="text-sm text-gray-600 dark:text-gray-300 mt-2">{truncated_desc}</p>
            <div class="flex flex-wrap gap-2 mt-3">
                {project.language.map(|lang| {
                    view! {
                        <span class="px-2 py-1 text-xs bg-blue-100 dark:bg-blue-900 text-blue-600 dark:text-blue-300 rounded">
                            {lang}
                        </span>
                    }
                })}
                {project.framework.map(|fw| {
                    view! {
                        <span class="px-2 py-1 text-xs bg-green-100 dark:bg-green-900 text-green-600 dark:text-green-300 rounded">
                            {fw}
                        </span>
                    }
                })}
                {tags.into_iter().take(2).map(|tag| {
                    view! {
                        <span class="px-2 py-1 text-xs bg-gray-100 dark:bg-gray-700 text-gray-600 dark:text-gray-300 rounded">
                            {tag}
                        </span>
                    }
                }).collect::<Vec<_>>()}
            </div>
        </div>
    }
}
