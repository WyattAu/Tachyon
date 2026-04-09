// App Shell - Main layout with sidebar navigation

use leptos::prelude::*;
use leptos_router::hooks::use_navigate;

/// App shell component with sidebar and main content area
#[component]
pub fn AppShell<F>(theme: ReadSignal<String>, toggle_theme: F, children: Children) -> impl IntoView
where
    F: Fn() + 'static,
{
    let _ = theme;
    let (sidebar_collapsed, set_sidebar_collapsed) = signal(false);

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

    view! {
        <div class="min-h-screen bg-gray-50 dark:bg-gray-900">
            {/* Sidebar */}
            <aside class={
                let collapsed = sidebar_collapsed.get();
                move || {
                    if collapsed {
                        "fixed inset-y-0 left-0 z-30 w-16 transition-all duration-300 bg-white dark:bg-gray-800 border-r border-gray-200 dark:border-gray-700"
                    } else {
                        "fixed inset-y-0 left-0 z-30 w-64 transition-all duration-300 bg-white dark:bg-gray-800 border-r border-gray-200 dark:border-gray-700"
                    }
                }
            }>
                <div class="h-full flex flex-col">
                    {/* Logo */}
                    <div class="p-4 border-b border-gray-200 dark:border-gray-700">
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
                    </div>

                    {/* Navigation */}
                    <nav class="flex-1 p-2 space-y-1 overflow-y-auto">
                        <NavLink href="/" label="Home" collapsed={sidebar_collapsed.get()} />
                        <NavLink href="/dashboard" label="Dashboard" collapsed={sidebar_collapsed.get()} />
                        <NavLink href="/documents" label="Documents" collapsed={sidebar_collapsed.get()} />
                        <NavLink href="/graph" label="Graph" collapsed={sidebar_collapsed.get()} />
                        <NavLink href="/teams" label="Teams" collapsed={sidebar_collapsed.get()} />
                        <NavLink href="/search" label="Search" collapsed={sidebar_collapsed.get()} />
                        <NavLink href="/catalog" label="Catalog" collapsed={sidebar_collapsed.get()} />
                        <NavLink href="/admin/roles" label="Admin" collapsed={sidebar_collapsed.get()} />
                        <NavLink href="/settings" label="Settings" collapsed={sidebar_collapsed.get()} />
                    </nav>

                    {/* Collapse toggle */}
                    <div class="p-2 border-t border-gray-200 dark:border-gray-700">
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
                        "ml-16 transition-all duration-300"
                    } else {
                        "ml-64 transition-all duration-300"
                    }
                }
            }>
                {/* Top bar */}
                <header class="sticky top-0 z-20 bg-white dark:bg-gray-800 border-b border-gray-200 dark:border-gray-700">
                    <div class="px-6 py-4 flex items-center justify-between">
                        <h1 class="text-xl font-semibold text-gray-900 dark:text-white">
                            Tachyon
                        </h1>
                        <div class="flex items-center space-x-4">
                            {/* Theme toggle button - simplified */}
                            <button
                                on:click=move |_| toggle_theme()
                                class="p-2 text-gray-500 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-lg"
                            >
                                <svg class="w-5 h-5" fill="currentColor" viewBox="0 0 20 20">
                                    <path fill-rule="evenodd" d="M10 2a1 1 0 011 1v1a1 1 0 11-2 0V3a1 1 0 011-1zm4 8a4 4 0 11-8 0 4 4 0 018 0zm-.464 4.95l.707.707a1 1 0 001.414-1.414l-.707-.707a1 1 0 00-1.414 1.414zm2.12-10.607a1 1 0 010 1.414l-.706.707a1 1 0 11-1.414-1.414l.707-.707a1 1 0 011.414 0zM17 11a1 1 0 100-2h-1a1 1 0 100 2h1zm-7 4a1 1 0 011 1v1a1 1 0 11-2 0v-1a1 1 0 011-1zM5.05 6.464A1 1 0 106.465 5.05l-.708-.707a1 1 0 00-1.414 1.414l.707.707zm1.414 8.486l-.707.707a1 1 0 01-1.414-1.414l.707-.707a1 1 0 011.414 1.414zM4 11a1 1 0 100-2H3a1 1 0 000 2h1z" clip-rule="evenodd" />
                                </svg>
                            </button>
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
                <main class="p-6">
                    {children()}
                </main>
            </div>
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
