// Settings Page
// User settings, preferences, and application configuration

use leptos::prelude::*;
use wasm_bindgen::prelude::*;
use crate::api::ApiClient;

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

    let (display_name, set_display_name) = signal(String::new());
    let (email, set_email) = signal(String::new());
    let (profile_loaded, set_profile_loaded) = signal(false);
    let (save_message, set_save_message) = signal(String::new());
    let (saving, set_saving) = signal(false);

    // Collaboration settings
    let (collaboration_cursor_sharing, set_collaboration_cursor_sharing) = signal(true);
    let (collaboration_presence, set_collaboration_presence) = signal(true);
    let (collaboration_auto_connect, set_collaboration_auto_connect) = signal(false);

    let api_client = ApiClient::default();
    let api_client_for_load = api_client.clone();

    Effect::new(move |_| {
        let client = api_client_for_load.clone();
        wasm_bindgen_futures::spawn_local(async move {
            match client.get_current_user().await {
                Ok(user) => {
                    let name = user.get("display_name")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();
                    let mail = user.get("email")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();
                    set_display_name.set(name);
                    set_email.set(mail);
                    set_profile_loaded.set(true);
                }
                Err(_) => {
                    set_display_name.set(String::new());
                    set_email.set(String::new());
                    set_profile_loaded.set(true);
                }
            }
        });
    });

    let on_save_profile = move |_| {
        let client = api_client.clone();
        let name = display_name.get();
        let mail = email.get();
        set_saving.set(true);
        set_save_message.set(String::new());
        wasm_bindgen_futures::spawn_local(async move {
            let dn = if name.is_empty() { None } else { Some(name.as_str()) };
            let em = if mail.is_empty() { None } else { Some(mail.as_str()) };
            match client.update_profile(dn, em).await {
                Ok(_) => {
                    set_save_message.set("Profile saved successfully.".to_string());
                }
                Err(e) => {
                    set_save_message.set(format!("Failed to save: {}", e));
                }
            }
            set_saving.set(false);
        });
    };

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
                            prop:value=move || display_name.get()
                            on:input=move |ev| set_display_name.set(event_target_value(&ev))
                            class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100"
                            disabled=move || !profile_loaded.get()
                        />
                    </div>
                    <div>
                        <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                            "Email"
                        </label>
                        <input
                            type="email"
                            prop:value=move || email.get()
                            on:input=move |ev| set_email.set(event_target_value(&ev))
                            class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100"
                            disabled=move || !profile_loaded.get()
                        />
                    </div>
                    <div class="flex items-center gap-3">
                        <button
                            on:click=on_save_profile
                            disabled=move || saving.get() || !profile_loaded.get()
                            class="px-4 py-2 bg-blue-600 hover:bg-blue-700 disabled:bg-blue-400 text-white rounded-lg transition-colors"
                        >
                            {move || if saving.get() { "Saving..." } else { "Save" }}
                        </button>
                        <span class=move || {
                            let msg = save_message.get();
                            if msg.is_empty() {
                                "hidden".to_string()
                            } else {
                                "text-sm text-gray-600 dark:text-gray-400".to_string()
                            }
                        }>
                            {move || save_message.get()}
                        </span>
                    </div>
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
                        <span class="text-sm font-medium text-gray-900 dark:text-white">"0.16.0"</span>
                    </div>
                    <div class="flex gap-4 pt-2">
                        <a
                            href="https://github.com/WyattAu/Tachyon"
                            target="_blank"
                            rel="noopener noreferrer"
                            class="inline-flex items-center text-sm text-blue-600 dark:text-blue-400 hover:underline"
                        >
                            <ExternalLinkIcon />
                            "GitHub"
                        </a>
                        <a
                            href="/api/docs"
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

            // Webhooks Section
            <WebhooksSection />

            // Collaboration Settings
            <SettingsSection title="Collaboration" description="Configure real-time collaboration settings">
                <div class="space-y-4">
                    <div class="flex items-center justify-between">
                        <div>
                            <p class="text-sm font-medium text-gray-900 dark:text-gray-100">"Cursor Sharing"</p>
                            <p class="text-xs text-gray-500 dark:text-gray-400">"Show your cursor position to other collaborators"</p>
                        </div>
                        <ToggleSwitch
                            enabled=collaboration_cursor_sharing.get()
                            on_toggle={move |_| set_collaboration_cursor_sharing.update(|v| *v = !*v)}
                        />
                    </div>
                    <div class="flex items-center justify-between">
                        <div>
                            <p class="text-sm font-medium text-gray-900 dark:text-gray-100">"Presence Indicators"</p>
                            <p class="text-xs text-gray-500 dark:text-gray-400">"Show when other users are viewing the same document"</p>
                        </div>
                        <ToggleSwitch
                            enabled=collaboration_presence.get()
                            on_toggle={move |_| set_collaboration_presence.update(|v| *v = !*v)}
                        />
                    </div>
                    <div class="flex items-center justify-between">
                        <div>
                            <p class="text-sm font-medium text-gray-900 dark:text-gray-100">"Auto-Connect Collaboration"</p>
                            <p class="text-xs text-gray-500 dark:text-gray-400">"Automatically start collaboration when opening a shared document"</p>
                        </div>
                        <ToggleSwitch
                            enabled=collaboration_auto_connect.get()
                            on_toggle={move |_| set_collaboration_auto_connect.update(|v| *v = !*v)}
                        />
                    </div>
                </div>
            </SettingsSection>

            // OAuth2 / Connected Accounts
            <SettingsSection title="Connected Accounts" description="Manage third-party authentication providers">
                <div class="space-y-3">
                    <div class="flex items-center justify-between p-3 bg-gray-50 dark:bg-gray-800 rounded-lg">
                        <div class="flex items-center gap-3">
                            <div class="w-8 h-8 bg-white dark:bg-gray-700 rounded-lg flex items-center justify-center shadow-sm">
                                <span class="text-lg font-bold">"G"</span>
                            </div>
                            <div>
                                <p class="text-sm font-medium text-gray-900 dark:text-gray-100">"Google"</p>
                                <p class="text-xs text-gray-500 dark:text-gray-400">"Sign in with Google OAuth2"</p>
                            </div>
                        </div>
                        <a
                            href="/api/auth/google/login"
                            class="px-3 py-1 text-xs rounded bg-blue-600 text-white hover:bg-blue-700 transition-colors"
                        >
                            "Connect"
                        </a>
                    </div>
                    <div class="flex items-center justify-between p-3 bg-gray-50 dark:bg-gray-800 rounded-lg">
                        <div class="flex items-center gap-3">
                            <div class="w-8 h-8 bg-white dark:bg-gray-700 rounded-lg flex items-center justify-center shadow-sm">
                                <span class="text-lg font-bold text-gray-800 dark:text-gray-200">"GH"</span>
                            </div>
                            <div>
                                <p class="text-sm font-medium text-gray-900 dark:text-gray-100">"GitHub"</p>
                                <p class="text-xs text-gray-500 dark:text-gray-400">"Sign in with GitHub OAuth2"</p>
                            </div>
                        </div>
                        <a
                            href="/api/auth/github/login"
                            class="px-3 py-1 text-xs rounded bg-gray-800 text-white hover:bg-gray-900 dark:bg-gray-600 dark:hover:bg-gray-500 transition-colors"
                        >
                            "Connect"
                        </a>
                    </div>
                    <p class="text-xs text-gray-400 dark:text-gray-500">
                        "OAuth2 providers must be configured on the server. Contact your administrator for setup instructions."
                    </p>
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

#[component]
fn WebhooksSection() -> impl IntoView {
    let (webhooks, set_webhooks) = signal(Vec::<crate::types::WebhookInfo>::new());
    let (new_url, set_new_url) = signal(String::new());
    let (new_secret, set_new_secret) = signal(String::new());
    let (new_events, set_new_events) = signal(Vec::<String>::new());
    let (message, set_message) = signal(String::new());

    let event_options = vec![
        "document_created",
        "document_updated",
        "document_deleted",
        "review_created",
        "review_approved",
        "review_rejected",
    ];

    let api_client = ApiClient::default();

    Effect::new({
        let api = api_client.clone();
        let set_w = set_webhooks.clone();
        let set_m = set_message.clone();
        move |_| {
            let api = api.clone();
            let set_w = set_w.clone();
            let set_m = set_m.clone();
            wasm_bindgen_futures::spawn_local(async move {
                match api.list_webhooks().await {
                    Ok(hooks) => set_w.set(hooks),
                    Err(e) => set_m.set(format!("Failed to load webhooks: {}", e)),
                }
            });
        }
    });

    let on_toggle_event = move |event: String| {
        set_new_events.update(|events| {
            if let Some(pos) = events.iter().position(|e| e == &event) {
                events.remove(pos);
            } else {
                events.push(event);
            }
        });
    };

    let api_client_for_delete = api_client.clone();
    let on_add_webhook = move |_: leptos::ev::MouseEvent| {
        let api = api_client.clone();
        let url = new_url.get();
        let secret = new_secret.get();
        let events = new_events.get();
        let set_m = set_message.clone();
        let set_u = set_new_url.clone();
        let set_s = set_new_secret.clone();
        let set_e = set_new_events.clone();
        let set_w = set_webhooks.clone();

        if url.is_empty() || events.is_empty() {
            set_m.set("URL and at least one event are required.".to_string());
            return;
        }

        wasm_bindgen_futures::spawn_local(async move {
            let events_ref: Vec<&str> = events.iter().map(|s| s.as_str()).collect();
            let secret_ref = if secret.is_empty() { None } else { Some(secret.as_str()) };
            match api.create_webhook(&url, events_ref, secret_ref).await {
                Ok(_) => {
                    set_m.set("Webhook created.".to_string());
                    set_u.set(String::new());
                    set_s.set(String::new());
                    set_e.set(Vec::new());
                    match api.list_webhooks().await {
                        Ok(hooks) => set_w.set(hooks),
                        Err(e) => set_m.set(format!("Reload failed: {}", e)),
                    }
                }
                Err(e) => set_m.set(format!("Failed to create webhook: {}", e)),
            }
        });
    };

    let on_delete_webhook = move |id: String| {
        let api = api_client_for_delete.clone();
        let set_m = set_message.clone();
        let set_w = set_webhooks.clone();
        wasm_bindgen_futures::spawn_local(async move {
            match api.delete_webhook(&id).await {
                Ok(_) => {
                    set_m.set("Webhook deleted.".to_string());
                    match api.list_webhooks().await {
                        Ok(hooks) => set_w.set(hooks),
                        Err(e) => set_m.set(format!("Reload failed: {}", e)),
                    }
                }
                Err(e) => set_m.set(format!("Failed to delete webhook: {}", e)),
            }
        });
    };

    view! {
        <SettingsSection title="Webhooks" description="Manage webhook endpoints for event notifications">
            <div class="space-y-4">
                <div>
                    <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">"Webhook URL"</label>
                    <input
                        type="url"
                        prop:value=move || new_url.get()
                        on:input=move |ev| set_new_url.set(event_target_value(&ev))
                        class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100"
                        placeholder="https://example.com/webhook"
                    />
                </div>
                <div>
                    <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">"Secret (optional)"</label>
                    <input
                        type="text"
                        prop:value=move || new_secret.get()
                        on:input=move |ev| set_new_secret.set(event_target_value(&ev))
                        class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100"
                        placeholder="HMAC signing secret"
                    />
                </div>
                <div>
                    <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">"Events"</label>
                    <div class="flex flex-wrap gap-2">
                        {event_options.into_iter().map(|event| {
                            let event_str = event.to_string();
                            let is_checked = {
                                let evts = new_events.get();
                                evts.contains(&event_str)
                            };
                            let on_toggle = on_toggle_event.clone();
                            let event_label = event.to_string();
                            let event_for_toggle = event.to_string();
                            view! {
                                <label
                                    class=move || {
                                        let base = "inline-flex items-center gap-1.5 px-3 py-1.5 rounded-lg border cursor-pointer text-sm transition-colors";
                                        if is_checked {
                                            format!("{} border-blue-500 bg-blue-50 dark:bg-blue-900/20 text-blue-700 dark:text-blue-300", base)
                                        } else {
                                            format!("{} border-gray-300 dark:border-gray-600 text-gray-600 dark:text-gray-400 hover:border-gray-400", base)
                                        }
                                    }>
                                    <input
                                        type="checkbox"
                                        class="sr-only"
                                        checked=is_checked
                                        on:change=move |_| on_toggle(event_for_toggle.clone())
                                    />
                                    {event_label}
                                </label>
                            }
                        }).collect::<Vec<_>>()}
                    </div>
                </div>
                <div class="flex items-center gap-3">
                    <button
                        on:click=on_add_webhook
                        class="px-4 py-2 bg-blue-600 hover:bg-blue-700 text-white rounded-lg transition-colors"
                    >
                        "Add Webhook"
                    </button>
                    {move || {
                        let msg = message.get();
                        if msg.is_empty() {
                            view! { <span></span> }.into_any()
                        } else {
                            view! { <span class="text-sm text-gray-600 dark:text-gray-400">{msg}</span> }.into_any()
                        }
                    }}
                </div>

                // Existing webhooks list
                <div class="mt-4">
                    <h3 class="text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">"Active Webhooks"</h3>
                    {move || {
                        let hooks = webhooks.get();
                        if hooks.is_empty() {
                            view! {
                                <p class="text-sm text-gray-500 dark:text-gray-400">"No webhooks configured"</p>
                            }.into_any()
                        } else {
                            view! {
                                <div class="space-y-2">
                                    {hooks.into_iter().map(|hook| {
                                        let hook_id = hook.id.clone();
                                        let on_del = on_delete_webhook.clone();
                                        view! {
                                            <div class="flex items-center justify-between p-3 bg-gray-50 dark:bg-gray-700/50 rounded-lg">
                                                <div class="min-w-0 flex-1">
                                                    <p class="text-sm font-medium text-gray-900 dark:text-white truncate">{hook.url}</p>
                                                    <p class="text-xs text-gray-500 dark:text-gray-400 mt-0.5">
                                                        {hook.events.join(", ")}
                                                    </p>
                                                </div>
                                                <button
                                                    on:click=move |_| on_del(hook_id.clone())
                                                    class="ml-3 px-2 py-1 text-xs text-red-600 dark:text-red-400 hover:bg-red-50 dark:hover:bg-red-900/20 rounded transition-colors"
                                                >
                                                    "Delete"
                                                </button>
                                            </div>
                                        }
                                    }).collect::<Vec<_>>()}
                                </div>
                            }.into_any()
                        }
                    }}
                </div>
            </div>
        </SettingsSection>
    }
}
