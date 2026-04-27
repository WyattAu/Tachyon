#![allow(dead_code)]

use leptos::prelude::*;

#[component]
pub fn MobileNav(
    /// Whether mobile nav is open
    open: Signal<bool>,
    /// Close callback
    on_close: Callback<()>,
) -> impl IntoView {
    view! {
        <Show when=move || open.get()>
            <div
                class="fixed inset-0 bg-black/50 z-40 md:hidden"
                on:click=move |_| on_close.run(())
            ></div>

            <div
                class=move || format!(
                    "fixed top-0 left-0 h-full w-64 bg-white dark:bg-gray-800 z-50 transform transition-transform duration-200 ease-in-out md:hidden {}",
                    if open.get() { "translate-x-0" } else { "-translate-x-full" }
                )
            >
                <div class="flex items-center justify-between p-4 border-b border-gray-200 dark:border-gray-700">
                    <span class="font-bold text-lg">"Tachyon"</span>
                    <button
                        on:click=move |_| on_close.run(())
                        class="p-3 text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-200 min-h-[44px] min-w-[44px] flex items-center justify-center"
                        aria-label="Close navigation"
                    >
                        <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" viewBox="0 0 20 20" fill="currentColor">
                            <path fill-rule="evenodd" d="M4.293 4.293a1 1 0 011.414 0L10 8.586l4.293-4.293a1 1 0 111.414 1.414L11.414 10l4.293 4.293a1 1 0 01-1.414 1.414L10 11.414l-4.293 4.293a1 1 0 01-1.414-1.414L8.586 10 4.293 5.707a1 1 0 010-1.414z" clip-rule="evenodd" />
                        </svg>
                    </button>
                </div>

                <nav class="p-4 space-y-2" aria-label="Mobile navigation">
                    <a href="/" class="block px-3 py-3 rounded-md text-gray-700 dark:text-gray-200 hover:bg-gray-100 dark:hover:bg-gray-700">"Home"</a>
                    <a href="/documents" class="block px-3 py-3 rounded-md text-gray-700 dark:text-gray-200 hover:bg-gray-100 dark:hover:bg-gray-700">"Documents"</a>
                    <a href="/spaces" class="block px-3 py-3 rounded-md text-gray-700 dark:text-gray-200 hover:bg-gray-100 dark:hover:bg-gray-700">"Spaces"</a>
                    <a href="/search" class="block px-3 py-3 rounded-md text-gray-700 dark:text-gray-200 hover:bg-gray-100 dark:hover:bg-gray-700">"Search"</a>
                    <a href="/teams" class="block px-3 py-3 rounded-md text-gray-700 dark:text-gray-200 hover:bg-gray-100 dark:hover:bg-gray-700">"Teams"</a>
                    <a href="/settings" class="block px-3 py-3 rounded-md text-gray-700 dark:text-gray-200 hover:bg-gray-100 dark:hover:bg-gray-700">"Settings"</a>
                </nav>
            </div>
        </Show>
    }
}
