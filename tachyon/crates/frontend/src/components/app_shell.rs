// App Shell - Main layout with sidebar navigation

use leptos::prelude::*;

/// App shell component with sidebar and main content area
#[component]
pub fn AppShell<F>(theme: ReadSignal<String>, toggle_theme: F, children: Children) -> impl IntoView
where
    F: Fn() + 'static,
{
    // Theme is applied via toggle_theme which updates document class
    // Future: Use theme signal to show sun/moon icon based on current theme
    let _ = theme;
    let (sidebar_collapsed, set_sidebar_collapsed) = signal(false);

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
                            <a href="/login" class="text-sm text-gray-500 hover:text-gray-700 dark:hover:text-gray-300">
                                Sign In
                            </a>
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
