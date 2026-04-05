#![allow(dead_code)]

use chrono::{DateTime, Utc};
use leptos::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ActivityType {
    Edit,
    Comment,
    Join,
    Leave,
    Create,
    Delete,
    Publish,
}

impl std::fmt::Display for ActivityType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ActivityType::Edit => write!(f, "edit"),
            ActivityType::Comment => write!(f, "comment"),
            ActivityType::Join => write!(f, "join"),
            ActivityType::Leave => write!(f, "leave"),
            ActivityType::Create => write!(f, "create"),
            ActivityType::Delete => write!(f, "delete"),
            ActivityType::Publish => write!(f, "publish"),
        }
    }
}

impl From<&str> for ActivityType {
    fn from(s: &str) -> Self {
        match s {
            "edit" => ActivityType::Edit,
            "comment" => ActivityType::Comment,
            "join" => ActivityType::Join,
            "leave" => ActivityType::Leave,
            "create" => ActivityType::Create,
            "delete" => ActivityType::Delete,
            "publish" => ActivityType::Publish,
            _ => ActivityType::Edit,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Activity {
    pub id: String,
    pub activity_type: ActivityType,
    pub user_id: String,
    pub user_name: String,
    pub document_id: Option<String>,
    pub description: String,
    pub timestamp: DateTime<Utc>,
    pub metadata: Option<serde_json::Value>,
}

impl Activity {
    pub fn new(
        activity_type: ActivityType,
        user_id: String,
        user_name: String,
        description: String,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            activity_type,
            user_id,
            user_name,
            document_id: None,
            description,
            timestamp: Utc::now(),
            metadata: None,
        }
    }

    pub fn with_document(mut self, document_id: String) -> Self {
        self.document_id = Some(document_id);
        self
    }

    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = Some(metadata);
        self
    }
}

fn get_activity_icon(activity_type: &ActivityType) -> &'static str {
    match activity_type {
        ActivityType::Edit => "✏️",
        ActivityType::Comment => "💬",
        ActivityType::Join => "👋",
        ActivityType::Leave => "🚪",
        ActivityType::Create => "📝",
        ActivityType::Delete => "🗑️",
        ActivityType::Publish => "🚀",
    }
}

fn get_activity_color(activity_type: &ActivityType) -> &'static str {
    match activity_type {
        ActivityType::Edit => "bg-blue-100 dark:bg-blue-900 text-blue-600 dark:text-blue-300",
        ActivityType::Comment => {
            "bg-purple-100 dark:bg-purple-900 text-purple-600 dark:text-purple-300"
        }
        ActivityType::Join => "bg-green-100 dark:bg-green-900 text-green-600 dark:text-green-300",
        ActivityType::Leave => "bg-gray-100 dark:bg-gray-700 text-gray-600 dark:text-gray-300",
        ActivityType::Create => {
            "bg-emerald-100 dark:bg-emerald-900 text-emerald-600 dark:text-emerald-300"
        }
        ActivityType::Delete => "bg-red-100 dark:bg-red-900 text-red-600 dark:text-red-300",
        ActivityType::Publish => {
            "bg-amber-100 dark:bg-amber-900 text-amber-600 dark:text-amber-300"
        }
    }
}

fn format_relative_time(timestamp: &DateTime<Utc>) -> String {
    let now = Utc::now();
    let duration = now.signed_duration_since(*timestamp);

    if duration.num_seconds() < 60 {
        "just now".to_string()
    } else if duration.num_minutes() < 60 {
        let mins = duration.num_minutes();
        format!("{} minute{} ago", mins, if mins == 1 { "" } else { "s" })
    } else if duration.num_hours() < 24 {
        let hours = duration.num_hours();
        format!("{} hour{} ago", hours, if hours == 1 { "" } else { "s" })
    } else if duration.num_days() < 7 {
        let days = duration.num_days();
        format!("{} day{} ago", days, if days == 1 { "" } else { "s" })
    } else {
        timestamp.format("%b %d, %Y").to_string()
    }
}

#[component]
pub fn ActivityFeed(
    activities: Vec<Activity>,
    #[prop(optional)] filter: Option<String>,
    #[prop(optional)] max_items: Option<usize>,
) -> impl IntoView {
    let filter_type = RwSignal::new(filter.unwrap_or_else(|| "all".to_string()));
    let activities_signal = RwSignal::new(activities);

    let filtered_activities = Memo::new(move |_| {
        let items = activities_signal.get();
        let ft = filter_type.get();

        let filtered: Vec<Activity> = if ft.is_empty() || ft == "all" {
            items
        } else {
            items
                .into_iter()
                .filter(|a| a.activity_type.to_string() == ft)
                .collect()
        };

        if let Some(max) = max_items {
            filtered.into_iter().take(max).collect()
        } else {
            filtered
        }
    });

    view! {
        <div class="flex flex-col h-full bg-white dark:bg-gray-800 border-l border-gray-200 dark:border-gray-700">
            <div class="px-4 py-3 border-b border-gray-200 dark:border-gray-700">
                <h3 class="text-sm font-semibold text-gray-900 dark:text-white">"Activity"</h3>
                <div class="mt-2 flex flex-wrap gap-1">
                    <FilterButton label="All".to_string() value="all".to_string() active=move || filter_type.get() == "all" on_click=move || filter_type.set("all".to_string()) />
                    <FilterButton label="Edits".to_string() value="edit".to_string() active=move || filter_type.get() == "edit" on_click=move || filter_type.set("edit".to_string()) />
                    <FilterButton label="Comments".to_string() value="comment".to_string() active=move || filter_type.get() == "comment" on_click=move || filter_type.set("comment".to_string()) />
                    <FilterButton label="Presence".to_string() value="presence".to_string() active=move || filter_type.get() == "presence" on_click=move || filter_type.set("presence".to_string()) />
                </div>
            </div>

            <div class="flex-1 overflow-y-auto">
                <div class="divide-y divide-gray-100 dark:divide-gray-700">
                    {move || {
                        filtered_activities.get().into_iter().map(|activity| {
                            view! {
                                <ActivityItem activity={activity} />
                            }
                        }).collect::<Vec<_>>()
                    }}
                </div>

                {move || {
                    if filtered_activities.get().is_empty() {
                        view! {
                            <div class="p-4 text-center text-gray-500 dark:text-gray-400 text-sm">
                                "No activity yet"
                            </div>
                        }.into_any()
                    } else {
                        view! { <div></div> }.into_any()
                    }
                }}
            </div>
        </div>
    }
}

#[component]
fn FilterButton(
    label: String,
    value: String,
    active: impl Fn() -> bool + 'static + Clone + Send + Sync,
    on_click: impl Fn() + 'static + Clone + Send + Sync,
) -> impl IntoView {
    let _ = value;
    let class_str = move || {
        if active() {
            "px-2 py-1 text-xs rounded bg-blue-600 text-white".to_string()
        } else {
            "px-2 py-1 text-xs rounded bg-gray-100 dark:bg-gray-700 text-gray-600 dark:text-gray-300 hover:bg-gray-200 dark:hover:bg-gray-600".to_string()
        }
    };

    view! {
        <button
            class={class_str}
            on:click={move |_| on_click()}
        >
            {label}
        </button>
    }
}

#[component]
fn ActivityItem(activity: Activity) -> impl IntoView {
    let icon = get_activity_icon(&activity.activity_type);
    let color_class = get_activity_color(&activity.activity_type);
    let time_str = format_relative_time(&activity.timestamp);

    view! {
        <div class="p-3 hover:bg-gray-50 dark:hover:bg-gray-700/50 transition-colors">
            <div class="flex items-start gap-3">
                <div class={format!("w-8 h-8 rounded-full flex items-center justify-center text-sm {}", color_class)}>
                    {icon}
                </div>

                <div class="flex-1 min-w-0">
                    <div class="flex items-center gap-2">
                        <span class="text-xs font-medium text-gray-900 dark:text-white truncate">
                            {activity.user_name.clone()}
                        </span>
                        <span class="text-xs text-gray-400 dark:text-gray-500">
                            {time_str}
                        </span>
                    </div>

                    <p class="mt-1 text-sm text-gray-600 dark:text-gray-300">
                        {activity.description.clone()}
                    </p>

                    {if let Some(doc_id) = &activity.document_id {
                        let display_id = format!("Document: {}...", &doc_id[..8.min(doc_id.len())]);
                        view! {
                            <div class="mt-1">
                                <span class="text-xs text-blue-600 dark:text-blue-400 hover:underline cursor-pointer">
                                    {display_id}
                                </span>
                            </div>
                        }.into_any()
                    } else {
                        view! { <div></div> }.into_any()
                    }}
                </div>
            </div>
        </div>
    }
}

#[component]
pub fn ActivityFeedCompact(
    activities: Vec<Activity>,
    #[prop(default = 5)] max_items: usize,
) -> impl IntoView {
    let display_activities: Vec<Activity> = activities.into_iter().take(max_items).collect();

    view! {
        <div class="bg-white dark:bg-gray-800 rounded-lg shadow border border-gray-200 dark:border-gray-700">
            <div class="px-4 py-3 border-b border-gray-200 dark:border-gray-700">
                <h3 class="text-sm font-semibold text-gray-900 dark:text-white">"Recent Activity"</h3>
            </div>

            <div class="divide-y divide-gray-100 dark:divide-gray-700">
                {display_activities.into_iter().map(|activity| {
                    let icon = get_activity_icon(&activity.activity_type);
                    let time_str = format_relative_time(&activity.timestamp);

                    view! {
                        <div class="px-4 py-2 flex items-center gap-2">
                            <span class="text-sm">{icon}</span>
                            <span class="text-sm text-gray-600 dark:text-gray-300 truncate flex-1">
                                {activity.description}
                            </span>
                            <span class="text-xs text-gray-400 dark:text-gray-500">
                                {time_str}
                            </span>
                        </div>
                    }
                }).collect::<Vec<_>>()}
            </div>
        </div>
    }
}
