#![allow(dead_code)]

use leptos::prelude::*;

#[component]
pub fn CatalogPage() -> impl IntoView {
    view! {
        <div class="min-h-screen bg-white dark:bg-gray-950">
            <header class="border-b border-gray-200 dark:border-gray-800 px-6 py-4">
                <h1 class="text-2xl font-bold text-gray-900 dark:text-gray-100">{"Template Catalog"}</h1>
                <p class="mt-1 text-sm text-gray-500 dark:text-gray-400">{"Browse and apply templates to your workspace"}</p>
            </header>
            <main class="p-6">
                <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6" id="catalog-grid">
                </div>
                <div class="mt-8 text-center text-gray-400 dark:text-gray-600">
                    <p>{"Templates loaded from your workspace."}</p>
                </div>
            </main>
        </div>
    }
}
