// Settings Page
// User settings, preferences, and application configuration

use leptos::prelude::*;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_name = "matchMedia", catch)]
    fn match_media(query: &str) -> Result<Option<MediaQueryList>, JsValue>;
}

#[wasm_bindgen]
extern "C" {
    type MediaQueryList;

    #[wasm_bindgen(method, getter)]
    fn matches(this: &MediaQueryList) -> bool;
}

/// Settings page component
#[component]
pub fn SettingsPage() -> impl IntoView {
    let (theme, set_theme) = signal(get_stored_theme());
    let (notifications_enabled, set_notifications_enabled) = signal(true);
    let (language, set_language) = signal("en".to_string());

    let on_theme_change = move |new_theme: String| {
        set_theme.set(new_theme.clone());
        apply_theme(&new_theme);
        save_theme_to_storage(&new_theme);
    };

    let on_logout = move |_| {
        if let Some(window) = web_sys::window() {
            let _ = window.location().set_href("/login");
        }
    };

    view! {
        <div class="max-w-3xl">
            <h1 class="text-2xl font-bold mb-6 text-gray-900 dark:text-white">"Settings"</h1>

            // Profile Section
            <SettingsSection title="Profile" description="Your account information">
                <div class="space-y-4">
                    <div>
                        <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                            "Display Name"
                        </label>
                        <input
                            type="text"
                            value="User"
                            readonly=true
                            class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-gray-50 dark:bg-gray-700 text-gray-900 dark:text-gray-100 cursor-not-allowed"
                        />
                    </div>
                    <div>
                        <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                            "Email"
                        </label>
                        <input
                            type="email"
                            value="user@example.com"
                            readonly=true
                            class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-gray-50 dark:bg-gray-700 text-gray-900 dark:text-gray-100 cursor-not-allowed"
                        />
                    </div>
                    <p class="text-sm text-gray-500 dark:text-gray-400">
                        "Profile editing will be available in a future update."
                    </p>
                </div>
            </SettingsSection>

            // Appearance Section
            <SettingsSection title="Appearance" description="Customize how Tachyon looks">
                <div class="space-y-4">
                    <div>
                        <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-3">
                            "Theme"
                        </label>
                        <div class="flex gap-4">
                            <ThemeOption
                                label="Light"
                                value="light"
                                current_theme=theme.get()
                                on_change=on_theme_change.clone()
                            />
                            <ThemeOption
                                label="Dark"
                                value="dark"
                                current_theme=theme.get()
                                on_change=on_theme_change.clone()
                            />
                            <ThemeOption
                                label="System"
                                value="system"
                                current_theme=theme.get()
                                on_change=on_theme_change.clone()
                            />
                        </div>
                    </div>
                </div>
            </SettingsSection>

            // Preferences Section
            <SettingsSection title="Preferences" description="Configure your preferences">
                <div class="space-y-4">
                    <div class="flex items-center justify-between">
                        <div>
                            <label class="text-sm font-medium text-gray-700 dark:text-gray-300">
                                "Email Notifications"
                            </label>
                            <p class="text-sm text-gray-500 dark:text-gray-400">
                                "Receive email updates about your activity"
                            </p>
                        </div>
                        <ToggleSwitch
                            enabled=notifications_enabled.get()
                            on_toggle=move |_| set_notifications_enabled.update(|e| *e = !*e)
                        />
                    </div>
                    <div>
                        <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                            "Language"
                        </label>
                        <select
                            class="w-full max-w-xs px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100"
                            on:change=move |ev| {
                                let value = event_target_value(&ev);
                                set_language.set(value);
                            }
                        >
                            <option value="en" selected={language.get() == "en"}>"English"</option>
                            <option value="zh" selected={language.get() == "zh"}>"中文"</option>
                            <option value="es" selected={language.get() == "es"}>"Español"</option>
                            <option value="ja" selected={language.get() == "ja"}>"日本語"</option>
                        </select>
                    </div>
                </div>
            </SettingsSection>

            // About Section
            <SettingsSection title="About" description="Application information">
                <div class="space-y-3">
                    <div class="flex justify-between items-center">
                        <span class="text-sm text-gray-600 dark:text-gray-400">"Version"</span>
                        <span class="text-sm font-medium text-gray-900 dark:text-white">"0.1.0"</span>
                    </div>
                    <div class="flex gap-4 pt-2">
                        <a
                            href="https://github.com/example/tachyon"
                            target="_blank"
                            rel="noopener noreferrer"
                            class="inline-flex items-center text-sm text-blue-600 dark:text-blue-400 hover:underline"
                        >
                            <ExternalLinkIcon />
                            "GitHub"
                        </a>
                        <a
                            href="https://docs.tachyon.example.com"
                            target="_blank"
                            rel="noopener noreferrer"
                            class="inline-flex items-center text-sm text-blue-600 dark:text-blue-400 hover:underline"
                        >
                            <ExternalLinkIcon />
                            "Documentation"
                        </a>
                    </div>
                </div>
            </SettingsSection>

            // Account Actions
            <SettingsSection title="Account" description="Manage your account">
                <button
                    on:click=on_logout
                    class="px-4 py-2 bg-red-600 hover:bg-red-700 text-white rounded-lg transition-colors"
                >
                    "Logout"
                </button>
            </SettingsSection>
        </div>
    }
}

/// Settings section wrapper component
#[component]
fn SettingsSection(
    title: &'static str,
    description: &'static str,
    children: Children,
) -> impl IntoView {
    view! {
        <div class="bg-white dark:bg-gray-800 rounded-lg shadow border border-gray-200 dark:border-gray-700 p-6 mb-6">
            <div class="mb-4">
                <h2 class="text-lg font-semibold text-gray-900 dark:text-white">{title}</h2>
                <p class="text-sm text-gray-500 dark:text-gray-400">{description}</p>
            </div>
            {children()}
        </div>
    }
}

/// Theme option radio button component
#[component]
fn ThemeOption<F>(
    label: &'static str,
    value: &'static str,
    current_theme: String,
    on_change: F,
) -> impl IntoView
where
    F: Fn(String) + 'static + Clone,
{
    let is_selected = current_theme == value;
    let value_str = value.to_string();
    let on_change_clone = on_change.clone();

    view! {
        <button
            class=move || {
                if is_selected {
                    "flex-1 px-4 py-3 border-2 border-blue-500 rounded-lg bg-blue-50 dark:bg-blue-900/20 text-blue-700 dark:text-blue-300 font-medium transition-colors"
                } else {
                    "flex-1 px-4 py-3 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-700 dark:text-gray-300 hover:border-gray-400 dark:hover:border-gray-500 transition-colors"
                }
            }
            on:click=move |_| on_change_clone(value_str.clone())
        >
            {label}
        </button>
    }
}

/// Toggle switch component
#[component]
fn ToggleSwitch<F>(enabled: bool, on_toggle: F) -> impl IntoView
where
    F: Fn(leptos::ev::MouseEvent) + 'static,
{
    view! {
        <button
            type="button"
            on:click=on_toggle
            class=move || {
                if enabled {
                    "relative inline-flex h-6 w-11 flex-shrink-0 cursor-pointer rounded-full border-2 border-transparent bg-blue-600 transition-colors"
                } else {
                    "relative inline-flex h-6 w-11 flex-shrink-0 cursor-pointer rounded-full border-2 border-transparent bg-gray-200 dark:bg-gray-600 transition-colors"
                }
            }
        >
            <span
                class=move || {
                    if enabled {
                        "pointer-events-none inline-block h-5 w-5 transform rounded-full bg-white shadow ring-0 transition translate-x-5"
                    } else {
                        "pointer-events-none inline-block h-5 w-5 transform rounded-full bg-white shadow ring-0 transition translate-x-0"
                    }
                }
            />
        </button>
    }
}

/// External link icon component
#[component]
fn ExternalLinkIcon() -> impl IntoView {
    view! {
        <svg
            class="w-4 h-4 mr-1"
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
        >
            <path
                stroke-linecap="round"
                stroke-linejoin="round"
                stroke-width="2"
                d="M10 6H6a2 2 0 00-2 2v10a2 2 0 002 2h10a2 2 0 002-2v-4M14 4h6m0 0v6m0-6L10 14"
            />
        </svg>
    }
}

// Helper functions for theme management

/// Get stored theme from localStorage, defaulting to "light"
fn get_stored_theme() -> String {
    if let Some(window) = web_sys::window() {
        if let Ok(Some(storage)) = window.local_storage() {
            if let Ok(Some(theme)) = storage.get_item("tachyon-theme") {
                return theme;
            }
        }
    }
    "light".to_string()
}

/// Save theme to localStorage
fn save_theme_to_storage(theme: &str) {
    if let Some(window) = web_sys::window() {
        if let Ok(Some(storage)) = window.local_storage() {
            let _ = storage.set_item("tachyon-theme", theme);
        }
    }
}

/// Apply theme to document
fn apply_theme(theme: &str) {
    let effective_theme = if theme == "system" {
        get_system_theme()
    } else {
        theme.to_string()
    };

    if let Some(window) = web_sys::window() {
        if let Some(document) = window.document() {
            if let Some(html) = document.document_element() {
                if effective_theme == "dark" {
                    let _ = html.class_list().add_1("dark");
                } else {
                    let _ = html.class_list().remove_1("dark");
                }
            }
        }
    }
}

/// Get system preferred theme
fn get_system_theme() -> String {
    if let Ok(Some(media_query)) = match_media("(prefers-color-scheme: dark)") {
        if media_query.matches() {
            return "dark".to_string();
        }
    }
    "light".to_string()
}
