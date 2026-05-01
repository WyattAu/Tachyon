#![allow(dead_code)]

use leptos::prelude::*;

#[component]
pub fn AppErrorBoundary(children: Children) -> impl IntoView {
    view! {
        <ErrorBoundary fallback=move |errors| {
            let errors_val = errors.get();
            let error_list: Vec<String> = errors_val
                .iter()
                .map(|(_id, e)| e.to_string())
                .collect();
            let error_display = error_list.join("\n");
            log::error!("AppErrorBoundary caught error: {}", error_display);

            let reload = move |_| {
                if let Some(window) = web_sys::window() {
                    let _ = window.location().reload();
                }
            };

            view! {
                <div class="min-h-[50vh] flex items-center justify-center p-8">
                    <div class="max-w-md w-full text-center">
                        <div class="mx-auto w-16 h-16 flex items-center justify-center rounded-full bg-red-100 dark:bg-red-900/30 mb-4">
                            <svg class="w-8 h-8 text-red-600 dark:text-red-400" aria-hidden="true" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-2.5L13.732 4c-.77-.833-1.964-.833-2.732 0L4.082 16.5c-.77.833.192 2.5 1.732 2.5z" />
                            </svg>
                        </div>
                        <h2 class="text-xl font-bold text-gray-900 dark:text-white mb-2">"Something went wrong"</h2>
                        <p class="text-sm text-gray-500 dark:text-gray-400 mb-4">
                            "An unexpected error occurred. Please try again."
                        </p>
                        <pre class="text-xs text-left bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded p-3 mb-4 overflow-auto max-h-32 text-red-700 dark:text-red-300 whitespace-pre-wrap break-words">
                            {error_display}
                        </pre>
                        <div class="flex items-center justify-center gap-3">
                            <button
                                class="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors text-sm font-medium"
                                on:click=reload
                            >
                                "Retry"
                            </button>
                            <a
                                href="https://github.com/WyattAu/Tachyon/issues/new"
                                target="_blank"
                                rel="noopener noreferrer"
                                class="px-4 py-2 bg-white dark:bg-gray-800 text-gray-700 dark:text-gray-300 border border-gray-300 dark:border-gray-600 rounded-lg hover:bg-gray-50 dark:hover:bg-gray-700 transition-colors text-sm font-medium inline-block"
                            >
                                "Report Bug"
                            </a>
                        </div>
                    </div>
                </div>
            }
        }>
            {children()}
        </ErrorBoundary>
    }
}
