// App Shell - Main layout with sidebar navigation

use crate::api::ApiClient;
use crate::types::Notification;
use crate::components::{should_show_onboarding, CommandPalette, OnboardingWizard};
use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use wasm_bindgen_futures::spawn_local;
use wasm_bindgen::JsCast;

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
        past.format("%b %d").to_string()
    }
}

/// App shell component with sidebar and main content area
#[component]
pub fn AppShell<F>(theme: ReadSignal<String>, toggle_theme: F, children: Children) -> impl IntoView
where
    F: Fn() + 'static,
{
    let _ = theme;
    let (sidebar_collapsed, set_sidebar_collapsed) = signal(false);
    let (mobile_menu_open, set_mobile_menu_open) = signal(false);
    let (palette_open, set_palette_open) = signal(false);
    let (show_onboarding, set_show_onboarding) = signal(should_show_onboarding());

    // Onboarding wizard view (extracted to avoid nested view! in closure)
    let onboarding_view = move || {
        if show_onboarding.get() {
            let on_finish = Callback::new(move |()| set_show_onboarding.set(false));
            view! {
                <OnboardingWizard on_complete={on_finish} />
            }.into_any()
        } else {
            ().into_any()
        }
    };

    let (user_id, set_user_id) = signal(None::<String>);
    let (show_user_menu, set_show_user_menu) = signal(false);

    Effect::new(move |_| {
        if let Some(id) = crate::components::auth_guard::get_user_id().filter(|s| !s.is_empty()) {
            set_user_id.set(Some(id));
        }
    });

    let navigate = use_navigate();

    let on_toggle_menu = Callback::new(move |ev: leptos::ev::MouseEvent| {
        ev.stop_propagation();
        set_show_user_menu.update(|show| *show = !*show);
    });

    let on_sign_out = Callback::new(move |_: leptos::ev::MouseEvent| {
        if let Some(window) = web_sys::window() {
            let _ = window.local_storage().unwrap().map(|storage| {
                storage.remove_item("tachyon_token").ok();
                storage.remove_item("tachyon_remember").ok();
            });
        }
        let client = crate::api::ApiClient::default();
        client.clear_auth_token();
        let _ = navigate("/login", Default::default());
    });

    let (show_notifications, set_show_notifications) = signal(false);
    let (notifications, set_notifications) = signal(Vec::<Notification>::new());
    let (unread_count, set_unread_count) = signal(0u32);

    let set_notifications_ref = set_notifications.clone();
    let set_unread_count_ref = set_unread_count.clone();

    let fetch_notifications = move || {
        let api = ApiClient::default();
        let set_n = set_notifications_ref.clone();
        let set_uc = set_unread_count_ref.clone();
        spawn_local(async move {
            if let Ok(count) = api.get_unread_notification_count().await {
                set_uc.set(count);
            }
            if let Ok(resp) = api.list_notifications(Some(10), true).await {
                set_n.set(resp.notifications);
            }
        });
    };

    Effect::new(move |_| {
        fetch_notifications();
    });

    let on_toggle_notifications = Callback::new(move |ev: leptos::ev::MouseEvent| {
        ev.stop_propagation();
        set_show_notifications.update(|show| *show = !*show);
    });

    let fetch_for_all = {
        let set_n = set_notifications.clone();
        let set_uc = set_unread_count.clone();
        move || {
            let api = ApiClient::default();
            let set_n = set_n.clone();
            let set_uc = set_uc.clone();
            spawn_local(async move {
                if let Ok(count) = api.get_unread_notification_count().await {
                    set_uc.set(count);
                }
                if let Ok(resp) = api.list_notifications(Some(10), true).await {
                    set_n.set(resp.notifications);
                }
            });
        }
    };

    let on_mark_all_read = Callback::new(move |ev: leptos::ev::MouseEvent| {
        ev.stop_propagation();
        let api = ApiClient::default();
        spawn_local(async move {
            let _ = api.mark_all_notifications_read().await;
        });
        fetch_for_all();
    });

    let on_toggle_mobile_menu = move |_: leptos::ev::MouseEvent| {
        set_mobile_menu_open.update(|open| *open = !*open);
    };

    let on_close_mobile_menu = move |_: leptos::ev::MouseEvent| {
        set_mobile_menu_open.set(false);
    };

    Effect::new(move |_| {
        let set_um = set_show_user_menu.clone();
        let set_notif = set_show_notifications.clone();
        let set_mm = set_mobile_menu_open.clone();
        let closure = wasm_bindgen::closure::Closure::<dyn Fn(wasm_bindgen::JsValue)>::new(move |_event: wasm_bindgen::JsValue| {
            set_um.set(false);
            set_notif.set(false);
            set_mm.set(false);
        });
        if let Some(window) = web_sys::window() {
            if let Some(document) = window.document() {
                let _ = document.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref());
                closure.forget();
            }
        }
    });

    view! {
        <div class="min-h-screen bg-gray-50 dark:bg-gray-900">
            <div
                class="fixed inset-0 z-40 bg-black bg-opacity-50 md:hidden transition-opacity duration-200"
                style={move || if mobile_menu_open.get() { "" } else { "display: none;" }}
                on:click=on_close_mobile_menu
            ></div>

            {/* Sidebar */}
            <aside class={
                let collapsed = sidebar_collapsed.get();
                move || {
                    let width = if collapsed { "w-16" } else { "w-64" };
                    let mobile_transform = if mobile_menu_open.get() { "translate-x-0" } else { "-translate-x-full" };
                    format!(
                        "fixed inset-y-0 left-0 z-50 {} transform transition-transform duration-200 ease-in-out {} md:translate-x-0 transition-all duration-300 bg-white dark:bg-gray-800 border-r border-gray-200 dark:border-gray-700",
                        width,
                        mobile_transform,
                    )
                }
            }>
                <div class="h-full flex flex-col">
                    {/* Logo */}
                    <div class="p-4 border-b border-gray-200 dark:border-gray-700">
                        <div class="flex items-center justify-between">
                            <div class="flex items-center">
                                <div class="w-8 h-8 bg-blue-600 rounded-lg flex items-center justify-center">
                                    <span class="text-white font-bold text-lg">T</span>
                                </div>
                                <Show when={move || !sidebar_collapsed.get()}>
                                    <span class="ml-3 text-lg font-semibold text-gray-900 dark:text-white">
                                        Tachyon
                                    </span>
                                </Show>
                            </div>
                            <button
                                class="md:hidden p-1 text-gray-500 hover:bg-gray-100 dark:hover:bg-gray-700 rounded"
                                on:click=on_close_mobile_menu
                            >
                                <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
                                </svg>
                            </button>
                        </div>
                    </div>

                    {/* Navigation */}
                    <nav class="flex-1 p-2 space-y-1 overflow-y-auto no-print">
                        <NavLink href="/" label="Home" collapsed={sidebar_collapsed.get()} />
                        <NavLink href="/dashboard" label="Dashboard" collapsed={sidebar_collapsed.get()} />
                        <NavLink href="/documents" label="Documents" collapsed={sidebar_collapsed.get()} />
                        <NavLink href="/graph" label="Graph" collapsed={sidebar_collapsed.get()} />
                        <NavLink href="/tags" label="Tags" collapsed={sidebar_collapsed.get()} />
                        <NavLink href="/teams" label="Teams" collapsed={sidebar_collapsed.get()} />
                        <NavLink href="/search" label="Search" collapsed={sidebar_collapsed.get()} />
                        <NavLink href="/catalog" label="Catalog" collapsed={sidebar_collapsed.get()} />
                        <NavLink href="/admin/roles" label="Admin" collapsed={sidebar_collapsed.get()} />
                        <NavLink href="/settings" label="Settings" collapsed={sidebar_collapsed.get()} />
                    </nav>

                    {/* Collapse toggle */}
                    <div class="hidden md:block p-2 border-t border-gray-200 dark:border-gray-700 no-print">
                        <button
                            on:click=move |_| set_sidebar_collapsed.update(|c| *c = !*c)
                            class="w-full p-2 text-gray-500 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-lg flex items-center justify-center"
                        >
                            <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6h16M4 12h16M4 18h16" />
                            </svg>
                        </button>
                    </div>
                </div>
            </aside>

            {/* Main content */}
            <div class={
                let collapsed = sidebar_collapsed.get();
                move || {
                    if collapsed {
                        "md:ml-16 transition-all duration-300"
                    } else {
                        "md:ml-64 transition-all duration-300"
                    }
                }
            }>
                {/* Top bar */}
                <header class="sticky top-0 z-20 bg-white dark:bg-gray-800 border-b border-gray-200 dark:border-gray-700 no-print">
                    <div class="px-4 md:px-6 py-4 flex items-center justify-between">
                        <div class="flex items-center gap-3">
                            <button
                                class="md:hidden p-2 text-gray-500 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-lg"
                                on:click=on_toggle_mobile_menu
                            >
                                <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6h16M4 12h16M4 18h16" />
                                </svg>
                            </button>
                            <h1 class="text-xl font-semibold text-gray-900 dark:text-white">
                                Tachyon
                            </h1>
                        </div>
                        <div class="flex items-center space-x-4">
                            <button
                                on:click=move |_| set_palette_open.set(true)
                                class="hidden sm:flex items-center gap-1.5 px-2.5 py-1.5 text-sm text-gray-500 dark:text-gray-400 bg-gray-100 dark:bg-gray-700 hover:bg-gray-200 dark:hover:bg-gray-600 rounded-lg border border-gray-200 dark:border-gray-600 transition-colors"
                            >
                                <svg class="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
                                </svg>
                                <span class="hidden md:inline">"Search..."</span>
                                <kbd class="hidden md:inline text-xs bg-white dark:bg-gray-800 border border-gray-300 dark:border-gray-500 rounded px-1.5 py-0.5 font-mono">"\u{2318}K"</kbd>
                            </button>
                            <a
                                href="/api/docs"
                                target="_blank"
                                rel="noopener noreferrer"
                                class="hidden sm:inline text-sm text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-200 transition-colors"
                            >
                                "API Docs"
                            </a>
                            {/* Theme toggle button - simplified */}
                            <button
                                on:click=move |_| toggle_theme()
                                class="p-2 text-gray-500 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-lg"
                            >
                                <svg class="w-5 h-5" fill="currentColor" viewBox="0 0 20 20">
                                    <path fill-rule="evenodd" d="M10 2a1 1 0 011 1v1a1 1 0 11-2 0V3a1 1 0 011-1zm4 8a4 4 0 11-8 0 4 4 0 018 0zm-.464 4.95l.707.707a1 1 0 001.414-1.414l-.707-.707a1 1 0 00-1.414 1.414zm2.12-10.607a1 1 0 010 1.414l-.706.707a1 1 0 11-1.414-1.414l.707-.707a1 1 0 011.414 0zM17 11a1 1 0 100-2h-1a1 1 0 100 2h1zm-7 4a1 1 0 011 1v1a1 1 0 11-2 0v-1a1 1 0 011-1zM5.05 6.464A1 1 0 106.465 5.05l-.708-.707a1 1 0 00-1.414 1.414l.707.707zm1.414 8.486l-.707.707a1 1 0 01-1.414-1.414l.707-.707a1 1 0 011.414 1.414zM4 11a1 1 0 100-2H3a1 1 0 000 2h1z" clip-rule="evenodd" />
                                </svg>
                            </button>
                            <div class="relative">
                                <button
                                    on:click={move |ev| on_toggle_notifications.run(ev)}
                                    class="relative p-2 text-gray-500 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-lg"
                                >
                                    <svg class="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 17h5l-1.405-1.405A2.032 2.032 0 0118 14.158V11a6.002 6.002 0 00-4-5.659V5a2 2 0 10-4 0v.341C7.67 6.165 6 8.388 6 11v3.159c0 .538-.214 1.055-.595 1.436L4 17h5m6 0v1a3 3 0 11-6 0v-1m6 0H9" />
                                    </svg>
                                    {move || if unread_count.get() > 0 {
                                        view! {
                                            <span class="absolute -top-1 -right-1 h-4 w-4 rounded-full bg-red-500 text-white text-xs flex items-center justify-center font-medium">
                                                {move || unread_count.get().to_string()}
                                            </span>
                                        }.into_any()
                                    } else {
                                        view! { <span></span> }.into_any()
                                    }}
                                </button>
                                <div
                                    class="absolute right-0 mt-2 w-80 bg-white dark:bg-gray-800 rounded-lg shadow-lg border border-gray-200 dark:border-gray-700 z-50"
                                    style={move || if show_notifications.get() { "" } else { "display: none;" }}
                                >
                                    <div class="px-4 py-3 border-b border-gray-200 dark:border-gray-700 flex items-center justify-between">
                                        <span class="text-sm font-semibold text-gray-900 dark:text-white">"Notifications"</span>
                                        {move || if unread_count.get() > 0 {
                                            view! {
                                                <button
                                                    on:click={move |ev| on_mark_all_read.run(ev)}
                                                    class="text-xs text-blue-600 dark:text-blue-400 hover:underline"
                                                >
                                                    "Mark all as read"
                                                </button>
                                            }.into_any()
                                        } else {
                                            view! { <span></span> }.into_any()
                                        }}
                                    </div>
                                    <div class="max-h-64 overflow-y-auto">
                                        {move || {
                                            let notifs = notifications.get();
                                            if notifs.is_empty() {
                                                view! {
                                                    <div class="px-4 py-6 text-center text-sm text-gray-500 dark:text-gray-400">
                                                        "No notifications"
                                                    </div>
                                                }.into_any()
                                            } else {
                                                view! {
                                                    <div class="divide-y divide-gray-100 dark:divide-gray-700">
                                                        {notifs.into_iter().map(|n| {
                                                            let nid = n.id.clone();
                                                            let title = n.title.clone();
                                                            let body = n.body.clone().unwrap_or_default();
                                                            let truncated = if body.len() > 60 { format!("{}...", &body.chars().take(60).collect::<String>()) } else { body };
                                                            let time = format_notification_time(&n.created_at);
                                                            let is_read = n.read;
                                                            let fetch_ref = fetch_for_all.clone();
                                                            view! {
                                                                <div
                                                                    class={
                                                                        if is_read {
                                                                            "px-4 py-3 hover:bg-gray-50 dark:hover:bg-gray-700/50 transition-colors"
                                                                        } else {
                                                                            "px-4 py-3 bg-blue-50/50 dark:bg-blue-900/20 hover:bg-gray-50 dark:hover:bg-gray-700/50 transition-colors"
                                                                        }
                                                                    }
                                                                >
                                                                    <div class="flex items-start justify-between gap-2">
                                                                        <div class="flex-1 min-w-0">
                                                                            <p class={if is_read { "text-sm text-gray-700 dark:text-gray-300 truncate" } else { "text-sm font-medium text-gray-900 dark:text-white truncate" }}>{title}</p>
                                                                            {if !truncated.is_empty() {
                                                                                view! {
                                                                                    <p class="text-xs text-gray-500 dark:text-gray-400 mt-0.5 truncate">{truncated}</p>
                                                                                }.into_any()
                                                                            } else {
                                                                                view! { <div></div> }.into_any()
                                                                            }}
                                                                            <span class="text-xs text-gray-400 dark:text-gray-500 mt-1 block">{time}</span>
                                                                        </div>
                                                                        {if !is_read {
                                                                            let nid_clone = nid.clone();
                                                                            view! {
                                                                                <button
                                                                                    on:click={move |ev| {
                                                                                        ev.stop_propagation();
                                                                                        let api = ApiClient::default();
                                                                                        let nid = nid_clone.clone();
                                                                                        let fetch = fetch_ref.clone();
                                                                                        spawn_local(async move {
                                                                                            let _ = api.mark_notification_read(&nid).await;
                                                                                            fetch();
                                                                                        });
                                                                                    }}
                                                                                    class="flex-shrink-0 text-xs text-blue-600 dark:text-blue-400 hover:underline mt-0.5"
                                                                                >
                                                                                    "Mark read"
                                                                                </button>
                                                                            }.into_any()
                                                                        } else {
                                                                            view! { <div></div> }.into_any()
                                                                        }}
                                                                    </div>
                                                                </div>
                                                            }
                                                        }).collect::<Vec<_>>()}
                                                    </div>
                                                }.into_any()
                                            }
                                        }}
                                    </div>
                                </div>
                            </div>
                            <Show when=move || user_id.get().is_some()>
                                <div class="relative">
                                    <button
                                        on:click={move |ev| on_toggle_menu.run(ev)}
                                        class="flex items-center gap-2 p-1 rounded-lg hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors"
                                    >
                                        <div class="w-8 h-8 bg-blue-600 rounded-full flex items-center justify-center">
                                            <span class="text-white text-sm font-medium">
                                                {move || {
                                                    user_id.get()
                                                        .map(|id| id.chars().next().unwrap_or('U').to_uppercase().to_string())
                                                        .unwrap_or_else(|| "U".to_string())
                                                }}
                                            </span>
                                        </div>
                                    </button>
                                    <div
                                        class="absolute right-0 mt-2 w-48 bg-white dark:bg-gray-800 rounded-lg shadow-lg border border-gray-200 dark:border-gray-700 py-1 z-50"
                                        style={move || if show_user_menu.get() { "" } else { "display: none;" }}
                                    >
                                        <a href="/settings" class="block px-4 py-2 text-sm text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700">
                                            "Settings"
                                        </a>
                                        <button
                                            on:click={move |ev| on_sign_out.run(ev)}
                                            class="w-full text-left px-4 py-2 text-sm text-red-600 dark:text-red-400 hover:bg-gray-100 dark:hover:bg-gray-700"
                                        >
                                            "Sign Out"
                                        </button>
                                    </div>
                                </div>
                            </Show>
                            <Show when=move || user_id.get().is_none()>
                                <a href="/login" class="text-sm text-gray-500 hover:text-gray-700 dark:hover:text-gray-300">
                                    Sign In
                                </a>
                            </Show>
                        </div>
                    </div>
                </header>

                {/* Page content */}
                <main class="p-4 md:p-6">
                    {children()}
                </main>
            </div>
            <CommandPalette open=palette_open set_open=set_palette_open />
            {onboarding_view}
        </div>
    }
}

/// Navigation link component
#[component]
pub fn NavLink(href: &'static str, label: &'static str, collapsed: bool) -> impl IntoView {
    view! {
        <a
            href=href
            class="flex items-center p-3 rounded-lg transition-colors text-gray-600 hover:bg-gray-100 dark:text-gray-400 dark:hover:bg-gray-700"
            title=label
        >
            <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5H7a2 2 0 00-2 2v12a2 2 0 002 2h10a2 2 0 002-2V7a2 2 0 00-2-2h-2M9 5a2 2 0 002 2h2a2 2 0 002-2M9 5a2 2 0 012-2h2a2 2 0 012 2" />
            </svg>
            <Show when={move || !collapsed}>
                <span class="ml-3">{label}</span>
            </Show>
        </a>
    }
}
