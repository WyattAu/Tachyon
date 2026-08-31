// Home Page

use leptos::prelude::*;

/// Home page component
#[component]
pub fn HomePage() -> impl IntoView {
    view! {
        <div>
            <h1 class="text-3xl font-bold mb-4 text-gray-900 dark:text-white">Welcome to Tachyon</h1>
            <p class="text-gray-600 dark:text-gray-400 mb-8">
                A high-performance knowledge management platform
            </p>

            {/* Quick Links */}
            <h2 class="text-xl font-semibold mb-4 text-gray-900 dark:text-white">Quick Links</h2>
            <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
                <QuickLinkCard
                    href="/documents"
                    title="Documents"
                    description="View and manage your documents"
                />
                <QuickLinkCard
                    href="/search"
                    title="Search"
                    description="Search across all content"
                />
                <QuickLinkCard
                    href="/catalog"
                    title="Catalog"
                    description="Browse the project catalog"
                />
            </div>
        </div>
    }
}

/// Quick link card component
#[component]
fn QuickLinkCard(
    href: &'static str,
    title: &'static str,
    description: &'static str,
) -> impl IntoView {
    view! {
        <a
            href=href
            class="block bg-white dark:bg-gray-800 rounded-none shadow p-6 border-2 border-gray-900 dark:border-gray-100 hover:border-blue-500 dark:hover:border-blue-500 transition-colors"
        >
            <div class="flex items-center mb-3">
                <div class="w-10 h-10 bg-blue-100 dark:bg-blue-900 rounded-none flex items-center justify-center mr-3">
                    <svg class="w-5 h-5 text-blue-600 dark:text-blue-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 10V3L4 14h7v7l9-11h-7z" />
                    </svg>
                </div>
                <h3 class="text-lg font-semibold text-gray-900 dark:text-white">{title}</h3>
            </div>
            <p class="text-gray-500 dark:text-gray-400">{description}</p>
        </a>
    }
}
