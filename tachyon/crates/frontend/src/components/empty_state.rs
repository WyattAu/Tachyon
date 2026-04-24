#![allow(dead_code)]

use leptos::prelude::*;

#[component]
pub fn EmptyDocuments() -> impl IntoView {
    view! {
        <div class="bg-white dark:bg-gray-800 rounded-lg shadow border border-gray-200 dark:border-gray-700 p-12 text-center">
            <svg class="mx-auto h-16 w-16 text-gray-300 dark:text-gray-600 mb-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
            </svg>
            <h3 class="text-lg font-medium text-gray-900 dark:text-white mb-2">"No documents yet"</h3>
            <p class="text-sm text-gray-500 dark:text-gray-400 mb-6">"Create your first document to get started."</p>
            <button
                class="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors text-sm font-medium"
                on:click=move |_| {
                    let _ = leptos_router::hooks::use_navigate()("/documents", Default::default());
                }
            >
                "+ New Document"
            </button>
        </div>
    }
}

#[component]
pub fn EmptySearch(query: String) -> impl IntoView {
    view! {
        <div class="bg-white dark:bg-gray-800 rounded-lg shadow border border-gray-200 dark:border-gray-700 p-12 text-center">
            <svg class="mx-auto h-16 w-16 text-gray-300 dark:text-gray-600 mb-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
            </svg>
            <h3 class="text-lg font-medium text-gray-900 dark:text-white mb-2">
                "No results for \"" {query.clone()} "\""
            </h3>
            <p class="text-sm text-gray-500 dark:text-gray-400 mb-4">"Try adjusting your search or filter to find what you're looking for."</p>
            <div class="text-left max-w-sm mx-auto space-y-2">
                <p class="text-xs font-medium text-gray-700 dark:text-gray-300">"Suggestions:"</p>
                <ul class="text-xs text-gray-500 dark:text-gray-400 space-y-1 list-disc list-inside">
                    <li>"Check your spelling"</li>
                    <li>"Use more general terms"</li>
                    <li>"Try removing filters"</li>
                    <li>"Search by title or content"</li>
                </ul>
            </div>
        </div>
    }
}

#[component]
pub fn EmptyNotifications() -> impl IntoView {
    view! {
        <div class="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 p-12 text-center">
            <svg class="mx-auto h-16 w-16 text-gray-300 dark:text-gray-600 mb-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M15 17h5l-1.405-1.405A2.032 2.032 0 0118 14.158V11a6.002 6.002 0 00-4-5.659V5a2 2 0 10-4 0v.341C7.67 6.165 6 8.388 6 11v3.159c0 .538-.214 1.055-.595 1.436L4 17h5m6 0v1a3 3 0 11-6 0v-1m6 0H9" />
            </svg>
            <h3 class="text-lg font-medium text-gray-900 dark:text-white mb-2">"You're all caught up!"</h3>
            <p class="text-sm text-gray-500 dark:text-gray-400">"Notifications about reviews and activity will appear here."</p>
        </div>
    }
}
