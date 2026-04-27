#![allow(dead_code)]

use leptos::prelude::*;
use wasm_bindgen::JsCast;

#[derive(Debug, Clone, PartialEq)]
pub enum Theme {
    Light,
    Dark,
    System,
}

impl Theme {
    pub fn label(&self) -> &'static str {
        match self {
            Theme::Light => "Light",
            Theme::Dark => "Dark",
            Theme::System => "System",
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Theme::Light => "light",
            Theme::Dark => "dark",
            Theme::System => "system",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "light" => Theme::Light,
            "dark" => Theme::Dark,
            _ => Theme::System,
        }
    }
}

fn get_stored_theme() -> Theme {
    if let Some(window) = web_sys::window() {
        if let Ok(Some(storage)) = window.local_storage() {
            if let Ok(Some(theme)) = storage.get("tachyon-theme") {
                return Theme::from_str(&theme);
            }
        }
    }
    Theme::System
}

fn store_theme(theme: &Theme) {
    if let Some(window) = web_sys::window() {
        if let Ok(Some(storage)) = window.local_storage() {
            let _ = storage.set("tachyon-theme", theme.as_str());
        }
    }
}

fn apply_theme(theme: &Theme) {
    if let Some(window) = web_sys::window() {
        if let Some(document) = window.document() {
            let html = document.document_element().unwrap();
            match theme {
                Theme::Light => {
                    let _ = html.class_list().remove_1("dark");
                }
                Theme::Dark => {
                    let _ = html.class_list().add_1("dark");
                }
                Theme::System => {
                    let effective = get_system_theme();
                    if effective == "dark" {
                        let _ = html.class_list().add_1("dark");
                    } else {
                        let _ = html.class_list().remove_1("dark");
                    }
                }
            }
        }
    }
}

fn get_system_theme() -> &'static str {
    if let Some(window) = web_sys::window() {
        if let Ok(Some(mql)) = window.match_media("(prefers-color-scheme: dark)") {
            if mql.matches() {
                return "dark";
            }
        }
    }
    "light"
}

#[component]
pub fn ThemeToggle() -> impl IntoView {
    let (theme, set_theme) = signal(get_stored_theme());

    let toggle = move |_| {
        let new_theme = match theme.get() {
            Theme::Light => Theme::Dark,
            Theme::Dark => Theme::System,
            Theme::System => Theme::Light,
        };
        set_theme.set(new_theme.clone());
        apply_theme(&new_theme);
        store_theme(&new_theme);
    };

    view! {
        <button
            on:click=toggle
            class="p-3 min-h-[44px] min-w-[44px] flex items-center justify-center rounded-lg hover:bg-gray-100 dark:hover:bg-gray-700 text-gray-600 dark:text-gray-400 transition-colors"
            aria-label=move || format!("Theme: {}", theme.get().label())
            title=move || format!("Current: {} (click to switch)", theme.get().label())
        >
            {move || match theme.get() {
                Theme::Light => view! {
                    <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" viewBox="0 0 20 20" fill="currentColor">
                        <path fill-rule="evenodd" d="M10 2a1 1 0 011 1v1a1 1 0 11-2 0V3a1 1 0 011-1zm4 8a4 4 0 11-8 0 4 4 0 018 0zm-.464 4.95l.707.707a1 1 0 001.414-1.414l-.707-.707a1 1 0 00-1.414 1.414zm2.12-10.607a1 1 0 010 1.414l-.706.707a1 1 0 11-1.414-1.414l.707-.707a1 1 0 011.414 0zM17 11a1 1 0 100-2h-1a1 1 0 100 2h1zm-7 4a1 1 0 011 1v1a1 1 0 11-2 0v-1a1 1 0 011-1zM5.05 6.464A1 1 0 106.465 5.05l-.708-.707a1 1 0 00-1.414 1.414l.707.707zm1.414 8.486l-.707.707a1 1 0 01-1.414-1.414l.707-.707a1 1 0 011.414 1.414zM4 11a1 1 0 100-2H3a1 1 0 000 2h1z" clip-rule="evenodd" />
                    </svg>
                }.into_any(),
                Theme::Dark => view! {
                    <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" viewBox="0 0 20 20" fill="currentColor">
                        <path d="M17.293 13.293A8 8 0 016.707 2.707a8.001 8.001 0 1010.586 10.586z" />
                    </svg>
                }.into_any(),
                Theme::System => view! {
                    <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" viewBox="0 0 20 20" fill="currentColor">
                        <path fill-rule="evenodd" d="M3 5a2 2 0 012-2h10a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2V5zm11 1H6v4h8V6zm-3 7h2v2H9v-2zm-4 3h10a1 1 0 011 1v1a1 1 0 01-1 1H5a1 1 0 01-1-1v-1a1 1 0 011-1z" clip-rule="evenodd" />
                    </svg>
                }.into_any(),
            }}
        </button>
    }
}

#[component]
pub fn ThemeInitializer() -> impl IntoView {
    Effect::new(move |_| {
        let theme = get_stored_theme();
        apply_theme(&theme);

        if let Some(window) = web_sys::window() {
            if let Ok(Some(mql)) = window.match_media("(prefers-color-scheme: dark)") {
                let callback = wasm_bindgen::closure::Closure::<dyn Fn(web_sys::Event)>::new(move |_e: web_sys::Event| {
                    let stored = get_stored_theme();
                    if stored == Theme::System {
                        if let Some(window) = web_sys::window() {
                            if let Some(document) = window.document() {
                                let html = document.document_element().unwrap();
                                if let Ok(Some(sys_mql)) = window.match_media("(prefers-color-scheme: dark)") {
                                    if sys_mql.matches() {
                                        let _ = html.class_list().add_1("dark");
                                    } else {
                                        let _ = html.class_list().remove_1("dark");
                                    }
                                }
                            }
                        }
                    }
                });
                let _ = mql.add_event_listener_with_callback("change", callback.as_ref().unchecked_ref());
                callback.forget();
            }
        }
    });

    view! { <></> }
}

pub fn get_current_theme() -> Theme {
    get_stored_theme()
}

pub fn get_current_theme_label() -> String {
    get_stored_theme().label().to_string()
}
