use crate::i18n::{Locale, use_locale};
use leptos::prelude::*;

/// Language picker dropdown component.
///
/// Displays a dropdown to select the preferred language.
/// Persists the selection in localStorage and reloads the page.
#[component]
pub fn LanguagePicker() -> impl IntoView {
    let locale = use_locale();
    let (is_open, set_is_open) = signal(false);

    let on_select = Callback::new(move |new_locale: Locale| {
        let code = new_locale.code().to_string();
        crate::storage::set_locale(&code);
        // Reload page to apply language change
        if let Some(window) = web_sys::window() {
            let _ = window.location().reload();
        }
    });

    let toggle_dropdown = move |ev: leptos::ev::MouseEvent| {
        ev.stop_propagation();
        set_is_open.update(|open| *open = !*open);
    };

    let close_dropdown = move |_: leptos::ev::MouseEvent| {
        set_is_open.set(false);
    };

    view! {
        <div class="relative">
            <button
                on:click=toggle_dropdown
                class="flex items-center gap-1.5 px-2.5 py-1.5 text-sm text-gray-500 dark:text-gray-400 bg-gray-100 dark:bg-gray-700 hover:bg-gray-200 dark:hover:bg-gray-600 rounded border border-gray-200 dark:border-gray-600 transition-colors"
                aria-haspopup="true"
                aria-expanded=move || if is_open.get() { "true" } else { "false" }
                aria-label="Select language"
            >
                <svg class="w-4 h-4" aria-hidden="true" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 12a9 9 0 01-9 9m9-9a9 9 0 00-9-9m9 9H3m9 9a9 9 0 01-9-9m9 9c1.657 0 3-4.03 3-9s-1.343-9-3-9m0 18c-1.657 0-3-4.03-3-9s1.343-9 3-9m-9 9a9 9 0 019-9" />
                </svg>
                <span class="hidden md:inline">{move || locale.get().native_name()}</span>
                <svg class="w-3 h-3" aria-hidden="true" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
                </svg>
            </button>

            <Show when=move || is_open.get()>
                <div
                    class="absolute right-0 mt-2 w-40 bg-white dark:bg-gray-800 rounded-none shadow-lg border-2 border-gray-900 dark:border-gray-100 py-1 z-50"
                    on:click=close_dropdown
                >
                    {Locale::ALL.iter().map(|&loc| {
                        let is_current = move || locale.get() == loc;
                        let on_click = move |_: leptos::ev::MouseEvent| {
                            on_select.run(loc);
                        };
                        view! {
                            <button
                                on:click=on_click
                                class=move || {
                                    let base = "w-full text-left px-4 py-2 text-sm ";
                                    if is_current() {
                                        format!("{}bg-blue-50 dark:bg-blue-900/20 text-blue-600 dark:text-blue-400 font-medium", base)
                                    } else {
                                        format!("{}text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700", base)
                                    }
                                }
                            >
                                {loc.native_name()}
                            </button>
                        }
                    }).collect::<Vec<_>>()}
                </div>
            </Show>
        </div>
    }
}
