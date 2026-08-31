// Admin Analytics Dashboard Page
// Displays analytics metrics, charts, and activity data

use crate::api::ApiClient;
use leptos::prelude::*;
use leptos::task::spawn_local;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsOverview {
    pub total_documents: i64,
    pub total_users: i64,
    pub storage_bytes: i64,
    pub active_spaces: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyActivity {
    pub date: String,
    pub created: i64,
    pub updated: i64,
    pub deleted: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyActivityResponse {
    pub entries: Vec<DailyActivity>,
    pub total: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyUserActivity {
    pub date: String,
    pub active_users: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserActivityResponse {
    pub entries: Vec<DailyUserActivity>,
    pub total: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailySearchCount {
    pub date: String,
    pub query_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchActivityResponse {
    pub entries: Vec<DailySearchCount>,
    pub total: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiRequestVolume {
    pub date: String,
    pub total_requests: i64,
    pub successful: i64,
    pub failed: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiActivityResponse {
    pub entries: Vec<ApiRequestVolume>,
    pub total: usize,
}

async fn fetch_overview() -> Result<AnalyticsOverview, String> {
    let client = ApiClient::default();
    client
        .get_analytics_overview()
        .await
        .map_err(|e| e.to_string())
}

async fn fetch_document_activity(days: i32) -> Result<DailyActivityResponse, String> {
    let client = ApiClient::default();
    client
        .get_document_activity(days)
        .await
        .map_err(|e| e.to_string())
}

async fn fetch_user_activity(days: i32) -> Result<UserActivityResponse, String> {
    let client = ApiClient::default();
    client
        .get_user_activity(days)
        .await
        .map_err(|e| e.to_string())
}

async fn fetch_search_activity(days: i32) -> Result<SearchActivityResponse, String> {
    let client = ApiClient::default();
    client
        .get_search_activity(days)
        .await
        .map_err(|e| e.to_string())
}

async fn fetch_api_activity(days: i32) -> Result<ApiActivityResponse, String> {
    let client = ApiClient::default();
    client
        .get_api_activity(days)
        .await
        .map_err(|e| e.to_string())
}

fn format_bytes(bytes: i64) -> String {
    if bytes < 1024 {
        format!("{} B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else if bytes < 1024 * 1024 * 1024 {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    } else {
        format!("{:.2} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
    }
}

fn format_number(n: i64) -> String {
    if n >= 1_000_000 {
        format!("{:.1}M", n as f64 / 1_000_000.0)
    } else if n >= 1_000 {
        format!("{:.1}K", n as f64 / 1_000.0)
    } else {
        n.to_string()
    }
}

#[component]
pub fn AnalyticsPage() -> impl IntoView {
    let (days, set_days) = signal(30);
    let (overview, set_overview) = signal(None::<AnalyticsOverview>);
    let (doc_activity, set_doc_activity) = signal(None::<DailyActivityResponse>);
    let (user_activity_data, set_user_activity) = signal(None::<UserActivityResponse>);
    let (search_activity_data, set_search_activity) = signal(None::<SearchActivityResponse>);
    let (api_activity_data, set_api_activity) = signal(None::<ApiActivityResponse>);
    let (loading, set_loading) = signal(true);
    let (error, set_error) = signal(None::<String>);

    let load_data = move |d: i32| {
        spawn_local(async move {
            set_loading.set(true);
            set_error.set(None);

            let ov = fetch_overview().await;
            let doc = fetch_document_activity(d).await;
            let usr = fetch_user_activity(d).await;
            let srch = fetch_search_activity(d).await;
            let api = fetch_api_activity(d).await;

            if let Err(e) = &ov {
                set_error.set(Some(format!("Overview: {}", e)));
            }
            if let Err(e) = &doc {
                set_error.set(Some(format!("Document activity: {}", e)));
            }
            if let Err(e) = &usr {
                set_error.set(Some(format!("User activity: {}", e)));
            }
            if let Err(e) = &srch {
                set_error.set(Some(format!("Search activity: {}", e)));
            }
            if let Err(e) = &api {
                set_error.set(Some(format!("API activity: {}", e)));
            }

            if let Ok(o) = ov {
                set_overview.set(Some(o));
            }
            if let Ok(d) = doc {
                set_doc_activity.set(Some(d));
            }
            if let Ok(u) = usr {
                set_user_activity.set(Some(u));
            }
            if let Ok(s) = srch {
                set_search_activity.set(Some(s));
            }
            if let Ok(a) = api {
                set_api_activity.set(Some(a));
            }

            set_loading.set(false);
        });
    };

    Effect::new(move |_| {
        load_data(days.get());
    });

    let handle_days_change = move |new_days: i32| {
        set_days.set(new_days);
        load_data(new_days);
    };

    view! {
        <div class="max-w-7xl mx-auto px-4 py-6 sm:px-6 lg:px-8">
            <div class="mb-8">
                <div class="flex flex-col sm:flex-row sm:items-center sm:justify-between">
                    <div>
                        <h1 class="text-2xl font-bold text-gray-900 dark:text-white">
                            "Analytics Dashboard"
                        </h1>
                        <p class="mt-1 text-sm text-gray-500 dark:text-gray-400">
                            "Overview of system usage and activity"
                        </p>
                    </div>
                    <div class="mt-4 sm:mt-0 flex items-center gap-2">
                        <span class="text-sm text-gray-500 dark:text-gray-400">"Date range:"</span>
                        <select
                            class="px-3 py-1.5 border border-gray-300 dark:border-gray-600 rounded bg-white dark:bg-gray-700 text-gray-700 dark:text-gray-200 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
                            on:change=move |ev| {
                                let val = event_target_value(&ev);
                                if let Ok(d) = val.parse::<i32>() {
                                    handle_days_change(d);
                                }
                            }
                        >
                            <option value="7" selected={days.get() == 7}>"Last 7 days"</option>
                            <option value="14" selected={days.get() == 14}>"Last 14 days"</option>
                            <option value="30" selected={days.get() == 30}>"Last 30 days"</option>
                            <option value="60" selected={days.get() == 60}>"Last 60 days"</option>
                            <option value="90" selected={days.get() == 90}>"Last 90 days"</option>
                        </select>
                    </div>
                </div>
            </div>

            {move || if loading.get() {
                Some(view! {
                    <div class="flex justify-center items-center py-12">
                        <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600"></div>
                    </div>
                })
            } else {
                None
            }}

            {move || error.get().map(|e| view! {
                <div class="mb-4 p-4 bg-red-100 text-red-700 dark:bg-red-900 dark:text-red-200 rounded">
                    {e}
                </div>
            })}

            {move || overview.get().map(|ov| view! {
                <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4 mb-8">
                    <div class="bg-white dark:bg-gray-800 rounded border border-gray-200 dark:border-gray-700 p-5">
                        <div class="flex items-center gap-3">
                            <div class="flex-shrink-0 p-2 bg-blue-50 dark:bg-blue-900/30 rounded">
                                <svg class="w-5 h-5 text-blue-600 dark:text-blue-400" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
                                    <path stroke-linecap="round" stroke-linejoin="round" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"/>
                                </svg>
                            </div>
                            <div>
                                <p class="text-sm text-gray-500 dark:text-gray-400">"Total Documents"</p>
                                <p class="text-2xl font-bold text-gray-900 dark:text-white">{format_number(ov.total_documents)}</p>
                            </div>
                        </div>
                    </div>
                    <div class="bg-white dark:bg-gray-800 rounded border border-gray-200 dark:border-gray-700 p-5">
                        <div class="flex items-center gap-3">
                            <div class="flex-shrink-0 p-2 bg-green-50 dark:bg-green-900/30 rounded">
                                <svg class="w-5 h-5 text-green-600 dark:text-green-400" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
                                    <path stroke-linecap="round" stroke-linejoin="round" d="M12 4.354a4 4 0 110 5.292M15 21H3v-1a6 6 0 0112 0v1zm0 0h6v-1a6 6 0 00-9-5.197m13.5-9a2.5 2.5 0 11-5 0 2.5 2.5 0 015 0z"/>
                                </svg>
                            </div>
                            <div>
                                <p class="text-sm text-gray-500 dark:text-gray-400">"Total Users"</p>
                                <p class="text-2xl font-bold text-gray-900 dark:text-white">{format_number(ov.total_users)}</p>
                            </div>
                        </div>
                    </div>
                    <div class="bg-white dark:bg-gray-800 rounded border border-gray-200 dark:border-gray-700 p-5">
                        <div class="flex items-center gap-3">
                            <div class="flex-shrink-0 p-2 bg-purple-50 dark:bg-purple-900/30 rounded">
                                <svg class="w-5 h-5 text-purple-600 dark:text-purple-400" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
                                    <path stroke-linecap="round" stroke-linejoin="round" d="M4 7v10c0 2.21 3.582 4 8 4s8-1.79 8-4V7M4 7c0 2.21 3.582 4 8 4s8-1.79 8-4M4 7c0-2.21 3.582-4 8-4s8 1.79 8 4"/>
                                </svg>
                            </div>
                            <div>
                                <p class="text-sm text-gray-500 dark:text-gray-400">"Storage Used"</p>
                                <p class="text-2xl font-bold text-gray-900 dark:text-white">{format_bytes(ov.storage_bytes)}</p>
                            </div>
                        </div>
                    </div>
                    <div class="bg-white dark:bg-gray-800 rounded border border-gray-200 dark:border-gray-700 p-5">
                        <div class="flex items-center gap-3">
                            <div class="flex-shrink-0 p-2 bg-amber-50 dark:bg-amber-900/30 rounded">
                                <svg class="w-5 h-5 text-amber-600 dark:text-amber-400" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
                                    <path stroke-linecap="round" stroke-linejoin="round" d="M4 6a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2H6a2 2 0 01-2-2V6zm10 0a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2h-2a2 2 0 01-2-2V6zM4 16a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2H6a2 2 0 01-2-2v-2zm10 0a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2h-2a2 2 0 01-2-2v-2z"/>
                                </svg>
                            </div>
                            <div>
                                <p class="text-sm text-gray-500 dark:text-gray-400">"Active Spaces"</p>
                                <p class="text-2xl font-bold text-gray-900 dark:text-white">{format_number(ov.active_spaces)}</p>
                            </div>
                        </div>
                    </div>
                </div>
            })}

            <div class="grid grid-cols-1 lg:grid-cols-2 gap-6 mb-8">
                {move || doc_activity.get().map(|data| view! {
                    <div class="bg-white dark:bg-gray-800 rounded border border-gray-200 dark:border-gray-700 p-5">
                        <h3 class="text-lg font-semibold text-gray-900 dark:text-white mb-4">
                            "Document Activity"
                        </h3>
                        <SvgBarChart
                            entries=data.entries.iter().map(|e| (e.date.clone(), e.created + e.updated + e.deleted)).collect::<Vec<_>>()
                            label="Documents"
                            color="#3b82f6"
                        />
                    </div>
                })}

                {move || user_activity_data.get().map(|data| view! {
                    <div class="bg-white dark:bg-gray-800 rounded border border-gray-200 dark:border-gray-700 p-5">
                        <h3 class="text-lg font-semibold text-gray-900 dark:text-white mb-4">
                            "User Activity"
                        </h3>
                        <SvgBarChart
                            entries=data.entries.iter().map(|e| (e.date.clone(), e.active_users)).collect::<Vec<_>>()
                            label="Users"
                            color="#10b981"
                        />
                    </div>
                })}

                {move || search_activity_data.get().map(|data| view! {
                    <div class="bg-white dark:bg-gray-800 rounded border border-gray-200 dark:border-gray-700 p-5">
                        <h3 class="text-lg font-semibold text-gray-900 dark:text-white mb-4">
                            "Search Volume"
                        </h3>
                        <SvgBarChart
                            entries=data.entries.iter().map(|e| (e.date.clone(), e.query_count)).collect::<Vec<_>>()
                            label="Queries"
                            color="#8b5cf6"
                        />
                    </div>
                })}

                {move || api_activity_data.get().map(|data| view! {
                    <div class="bg-white dark:bg-gray-800 rounded border border-gray-200 dark:border-gray-700 p-5">
                        <h3 class="text-lg font-semibold text-gray-900 dark:text-white mb-4">
                            "API Request Volume"
                        </h3>
                        <SvgBarChart
                            entries=data.entries.iter().map(|e| (e.date.clone(), e.total_requests)).collect::<Vec<_>>()
                            label="Requests"
                            color="#f59e0b"
                        />
                    </div>
                })}
            </div>
        </div>
    }
}

#[component]
fn SvgBarChart(
    entries: Vec<(String, i64)>,
    label: &'static str,
    color: &'static str,
) -> impl IntoView {
    if entries.is_empty() {
        return view! {
            <div class="text-center py-8 text-gray-400 dark:text-gray-500 text-sm">
                "No data available"
            </div>
        }.into_any();
    }

    let max_val = entries.iter().map(|(_, v)| *v).max().unwrap_or(1).max(1);
    let bar_width = 20;
    let gap = 4;
    let chart_height = 120;
    let chart_width = entries.len() * (bar_width + gap);
    let svg_width = chart_width + 40;

    let bars: Vec<_> = entries
        .iter()
        .enumerate()
        .map(|(i, (date, val))| {
            let bar_height = (*val as f64 / max_val as f64 * chart_height as f64) as i32;
            let x = (i * (bar_width + gap)) as i32 + 20;
            let y = chart_height - bar_height;
            let short_date = date.len().saturating_sub(5);
            let label = &date[short_date..];
            (x, y, bar_width as i32, bar_height, *val, label.to_string())
        })
        .collect();

    view! {
        <div class="overflow-x-auto">
            <svg width={svg_width.to_string()} height={(chart_height + 30).to_string()} class="w-full">
                {bars.iter().map(|(x, y, w, h, val, lbl)| view! {
                    <g>
                        <rect
                            x={x.to_string()}
                            y={y.to_string()}
                            width={w.to_string()}
                            height={h.to_string()}
                            fill={color}
                            rx="2"
                            class="opacity-80 hover:opacity-100 transition-opacity"
                        />
                        <text
                            x={(*x + w / 2).to_string()}
                            y={(chart_height + 15).to_string()}
                            text-anchor="middle"
                            class="text-xs fill-gray-500 dark:fill-gray-400"
                        >
                            {lbl}
                        </text>
                        <text
                            x={(*x + w / 2).to_string()}
                            y={(*y - 4).to_string()}
                            text-anchor="middle"
                            class="text-xs fill-gray-600 dark:fill-gray-300 font-medium"
                        >
                            {val.to_string()}
                        </text>
                    </g>
                }).collect::<Vec<_>>()}
            </svg>
        </div>
    }.into_any()
}
