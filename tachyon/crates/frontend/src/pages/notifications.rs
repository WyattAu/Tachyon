// Notifications Page
// Dedicated notification center with full list, mark-read, and clickable links

use leptos::prelude::*;
use crate::api::ApiClient;
use crate::types::Notification;

/// Format a timestamp into a human-readable relative time string.
fn format_notification_time(timestamp: &str) -> String {
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
        past.format("%b %d, %Y").to_string()
    }
}

/// Notifications page component
#[component]
pub fn NotificationsPage() -> impl IntoView {
    let (notifications, set_notifications) = signal(Vec::<Notification>::new());
    let (loading, set_loading) = signal(true);
    let (show_unread_only, set_show_unread_only) = signal(false);
    let (total_count, set_total_count) = signal(0usize);

    let fetch_notifications = move || {
        let api = ApiClient::default();
        let set_n = set_notifications.clone();
        let set_l = set_loading.clone();
        let set_tc = set_total_count.clone();
        let unread_only = show_unread_only.get();
        wasm_bindgen_futures::spawn_local(async move {
            set_l.set(true);
            match api.list_notifications(Some(50), !unread_only).await {
                Ok(resp) => {
                    set_tc.set(resp.count);
                    set_n.set(resp.notifications);
                }
                Err(_) => {
                    set_n.set(Vec::new());
                }
            }
            set_l.set(false);
        });
    };

    // Load notifications on mount
    Effect::new(move |_| {
        fetch_notifications();
    });

    // Re-fetch when filter changes
    let show_unread_only_ref = show_unread_only.clone();
    Effect::new(move |_| {
        // This runs whenever show_unread_only changes (second invocation)
        let _ = show_unread_only_ref.get();
    });

    let on_toggle_filter = move |_: leptos::ev::MouseEvent| {
        set_show_unread_only.update(|v| { *v = !*v; });
        fetch_notifications();
    };

    let on_mark_all_read = move |_: leptos::ev::MouseEvent| {
        let api = ApiClient::default();
        wasm_bindgen_futures::spawn_local(async move {
            let _ = api.mark_all_notifications_read().await;
        });
        fetch_notifications();
    };

    let on_mark_read = move |notification_id: String| {
        let api = ApiClient::default();
        wasm_bindgen_futures::spawn_local(async move {
            let _ = api.mark_notification_read(&notification_id).await;
        });
        fetch_notifications();
    };

    view! {
        <div class="max-w-3xl">
            <div class="flex items-center justify-between mb-6">
                <h1 class="text-2xl font-bold text-gray-900 dark:text-white">"Notifications"</h1>
                <div class="flex items-center gap-3">
                    <button
                        on:click=on_toggle_filter
                        class={
                            move || {
                                if show_unread_only.get() {
                                    "px-3 py-1.5 text-sm rounded-lg bg-blue-100 dark:bg-blue-900/30 text-blue-700 dark:text-blue-300 font-medium transition-colors"
                                } else {
                                    "px-3 py-1.5 text-sm rounded-lg bg-gray-100 dark:bg-gray-700 text-gray-600 dark:text-gray-400 hover:bg-gray-200 dark:hover:bg-gray-600 transition-colors"
                                }
                            }
                        }
                    >
                        {move || if show_unread_only.get() { "Unread only" } else { "All" }}
                    </button>
                    <button
                        on:click=on_mark_all_read
                        class="px-3 py-1.5 text-sm rounded-lg bg-gray-100 dark:bg-gray-700 text-gray-600 dark:text-gray-400 hover:bg-gray-200 dark:hover:bg-gray-600 transition-colors"
                    >
                        "Mark all read"
                    </button>
                </div>
            </div>

            // Notification count
            <p class="text-sm text-gray-500 dark:text-gray-400 mb-4">
                {move || {
                    let count = total_count.get();
                    if count == 0 {
                        "No notifications".to_string()
                    } else if show_unread_only.get() {
                        format!("Showing unread notifications")
                    } else {
                        format!("{} notification{}", count, if count == 1 { "" } else { "s" })
                    }
                }}
            </p>

            // Loading state
            {move || if loading.get() {
                view! {
                    <div class="space-y-3">
                        {vec![0, 1, 2].into_iter().map(|_| {
                            view! {
                                <div class="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 p-4 animate-pulse">
                                    <div class="h-4 bg-gray-200 dark:bg-gray-700 rounded w-3/4 mb-2"></div>
                                    <div class="h-3 bg-gray-200 dark:bg-gray-700 rounded w-1/2"></div>
                                </div>
                            }
                        }).collect::<Vec<_>>()}
                    </div>
                }.into_any()
            } else {
                view! {
                    <div class="space-y-2">
                        {move || {
                            let notifs = notifications.get();
                            if notifs.is_empty() {
                                view! {
                                    <div class="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 p-12 text-center">
                                        <svg class="mx-auto h-12 w-12 text-gray-300 dark:text-gray-600 mb-3" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M15 17h5l-1.405-1.405A2.032 2.032 0 0118 14.158V11a6.002 6.002 0 00-4-5.659V5a2 2 0 10-4 0v.341C7.67 6.165 6 8.388 6 11v3.159c0 .538-.214 1.055-.595 1.436L4 17h5m6 0v1a3 3 0 11-6 0v-1m6 0H9" />
                                        </svg>
                                        <p class="text-gray-500 dark:text-gray-400">"You're all caught up!"</p>
                                        <p class="text-sm text-gray-400 dark:text-gray-500 mt-1">"Notifications about reviews and activity will appear here."</p>
                                    </div>
                                }.into_any()
                            } else {
                                view! {
                                    {notifs.into_iter().map(|n| {
                                        let nid = n.id.clone();
                                        let title = n.title.clone();
                                        let body = n.body.clone().unwrap_or_default();
                                        let link = n.link.clone();
                                        let time = format_notification_time(&n.created_at);
                                        let is_read = n.read;
                                        let ntype = n.notification_type.clone();
                                        let on_mark = on_mark_read.clone();
                                        let nid_for_mark = nid.clone();

                                        // Icon based on notification type
                                        let icon = {
                                            if ntype.contains("review_requested") {
                                                "📋".to_string()
                                            } else if ntype.contains("review_approved") {
                                                "✅".to_string()
                                            } else if ntype.contains("review_rejected") {
                                                "❌".to_string()
                                            } else if ntype.contains("review_commented") {
                                                "💬".to_string()
                                            } else {
                                                "🔔".to_string()
                                            }
                                        };

                                        view! {
                                            <div
                                                class={
                                                    if is_read {
                                                        "bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 p-4 transition-colors"
                                                    } else {
                                                        "bg-blue-50/50 dark:bg-blue-900/10 rounded-lg border border-blue-200 dark:border-blue-800 p-4 transition-colors"
                                                    }
                                                }
                                            >
                                                <div class="flex items-start gap-3">
                                                    <span class="text-lg mt-0.5">{icon}</span>
                                                    <div class="flex-1 min-w-0">
                                                        <div class="flex items-start justify-between gap-2">
                                                            <div class="flex-1 min-w-0">
                                                                {
                                                                    if let Some(ref link_url) = link {
                                                                        let link_clone = link_url.clone();
                                                                        view! {
                                                                            <a
                                                                                href=link_clone
                                                                                class={
                                                                                    if is_read {
                                                                                        "text-sm text-gray-700 dark:text-gray-300 hover:text-blue-600 dark:hover:text-blue-400 font-medium"
                                                                                    } else {
                                                                                        "text-sm text-gray-900 dark:text-white hover:text-blue-600 dark:hover:text-blue-400 font-semibold"
                                                                                    }
                                                                                }
                                                                            >
                                                                                {title}
                                                                            </a>
                                                                        }.into_any()
                                                                    } else {
                                                                        view! {
                                                                            <p class={
                                                                                if is_read {
                                                                                    "text-sm text-gray-700 dark:text-gray-300"
                                                                                } else {
                                                                                    "text-sm text-gray-900 dark:text-white font-semibold"
                                                                                }
                                                                            }>
                                                                                {title}
                                                                            </p>
                                                                        }.into_any()
                                                                    }
                                                                }
                                                                {if !body.is_empty() {
                                                                    view! {
                                                                        <p class="text-sm text-gray-500 dark:text-gray-400 mt-1 line-clamp-2">{body}</p>
                                                                    }.into_any()
                                                                } else {
                                                                    view! { <div></div> }.into_any()
                                                                }}
                                                            </div>
                                                            <div class="flex items-center gap-2 flex-shrink-0">
                                                                <span class="text-xs text-gray-400 dark:text-gray-500 whitespace-nowrap">{time}</span>
                                                                {if !is_read {
                                                                    view! {
                                                                        <button
                                                                            on:click={move |ev| {
                                                                                ev.stop_propagation();
                                                                                on_mark(nid_for_mark.clone());
                                                                            }}
                                                                            class="text-xs text-blue-600 dark:text-blue-400 hover:underline whitespace-nowrap"
                                                                        >
                                                                            "Mark read"
                                                                        </button>
                                                                    }.into_any()
                                                                } else {
                                                                    view! { <div></div> }.into_any()
                                                                }}
                                                            </div>
                                                        </div>
                                                    </div>
                                                </div>
                                            </div>
                                        }
                                    }).collect::<Vec<_>>()}
                                }.into_any()
                            }
                        }}
                    </div>
                }.into_any()
            }}
        </div>
    }
}
