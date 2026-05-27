use crate::api::ApiClient;
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SettingsTab {
    Profile,
    Account,
    Preferences,
    Danger,
}

#[component]
pub fn SettingsPage() -> impl IntoView {
    let active_tab = RwSignal::new(SettingsTab::Profile);

    let tabs = [
        SettingsTab::Profile,
        SettingsTab::Account,
        SettingsTab::Preferences,
        SettingsTab::Danger,
    ];

    let tab_ids: [(&'static str, &'static str); 4] = [
        ("settings-tab-profile", "settings-panel-profile"),
        ("settings-tab-account", "settings-panel-account"),
        ("settings-tab-preferences", "settings-panel-preferences"),
        ("settings-tab-danger", "settings-panel-danger"),
    ];

    view! {
        <div class="max-w-3xl">
            <h1 class="text-2xl font-bold mb-6 text-gray-900 dark:text-white">"Settings"</h1>
            <div class="flex border-b border-gray-200 dark:border-gray-700 mb-6 overflow-x-auto" role="tablist"
                on:keydown=move |ev: web_sys::KeyboardEvent| {
                    let key = ev.key();
                    if key == "ArrowRight" || key == "ArrowLeft" {
                        ev.prevent_default();
                        let current = active_tab.get();
                        let current_idx = tabs.iter().position(|&t| t == current).unwrap_or(0);
                        let new_idx = if key == "ArrowRight" {
                            (current_idx + 1) % tabs.len()
                        } else {
                            current_idx.checked_sub(1).unwrap_or(tabs.len() - 1)
                        };
                        active_tab.set(tabs[new_idx]);
                        if let Some(window) = web_sys::window() {
                            if let Some(doc) = window.document() {
                                let _ = doc.get_element_by_id(tab_ids[new_idx].0)
                                    .and_then(|el| el.dyn_into::<web_sys::HtmlElement>().ok())
                                    .and_then(|el| el.focus().ok());
                            }
                        }
                    }
                }
            >
                {tabs.iter().enumerate().map(|(i, tab)| {
                    let label = tab.label();
                    let t = *tab;
                    let tab_id = tab_ids[i].0;
                    let is_active = move || active_tab.get() == t;
                    view! {
                        <button
                            id={tab_id}
                            class={move || tab_button_class(active_tab.get(), t)}
                            role="tab"
                            attr:aria-selected=move || if is_active() { "true" } else { "false" }
                            attr:aria-controls={tab_ids[i].1}
                            tabindex=move || if is_active() { 0 } else { -1 }
                            on:click=move |_| active_tab.set(t)
                        >{label}</button>
                    }
                }).collect::<Vec<_>>()}
            </div>
            <Show when={move || active_tab.get() == SettingsTab::Profile}>
                <div id="settings-panel-profile" role="tabpanel" attr:aria-labelledby="settings-tab-profile">
                    <ProfileTab />
                </div>
            </Show>
            <Show when={move || active_tab.get() == SettingsTab::Account}>
                <div id="settings-panel-account" role="tabpanel" attr:aria-labelledby="settings-tab-account">
                    <AccountTab />
                </div>
            </Show>
            <Show when={move || active_tab.get() == SettingsTab::Preferences}>
                <div id="settings-panel-preferences" role="tabpanel" attr:aria-labelledby="settings-tab-preferences">
                    <PreferencesTab />
                </div>
            </Show>
            <Show when={move || active_tab.get() == SettingsTab::Danger}>
                <div id="settings-panel-danger" role="tabpanel" attr:aria-labelledby="settings-tab-danger">
                    <DangerTab />
                </div>
            </Show>
        </div>
    }
}

fn tab_button_class(active: SettingsTab, tab: SettingsTab) -> String {
    let base = "px-4 py-2.5 text-sm font-medium border-b-2 whitespace-nowrap transition-colors";
    if active == tab {
        format!("{} border-blue-500 text-blue-600 dark:text-blue-400", base)
    } else {
        format!(
            "{} border-transparent text-gray-500 dark:text-gray-400 hover:text-gray-700 dark:hover:text-gray-300 hover:border-gray-300",
            base
        )
    }
}

impl SettingsTab {
    fn label(&self) -> &'static str {
        match self {
            SettingsTab::Profile => "Profile",
            SettingsTab::Account => "Account",
            SettingsTab::Preferences => "Preferences",
            SettingsTab::Danger => "Danger Zone",
        }
    }
}

#[component]
fn ProfileTab() -> impl IntoView {
    let (display_name, set_display_name) = signal(String::new());
    let (email, set_email) = signal(String::new());
    let (profile_loaded, set_profile_loaded) = signal(false);
    let (profile_msg, set_profile_msg) = signal(String::new());
    let (saving_profile, set_saving_profile) = signal(false);

    Effect::new(move |_| {
        let api = ApiClient::default();
        wasm_bindgen_futures::spawn_local(async move {
            match api.get_current_user().await {
                Ok(user) => {
                    let name = user
                        .get("display_name")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();
                    let mail = user
                        .get("email")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();
                    set_display_name.set(name);
                    set_email.set(mail);
                    set_profile_loaded.set(true);
                }
                Err(_) => {
                    set_profile_loaded.set(true);
                }
            }
        });
    });

    let on_save_profile = move |_| {
        let client = ApiClient::default();
        let name = display_name.get();
        let mail = email.get();
        set_saving_profile.set(true);
        set_profile_msg.set(String::new());
        let set_msg = set_profile_msg;
        let set_saving = set_saving_profile;
        wasm_bindgen_futures::spawn_local(async move {
            let dn = if name.is_empty() {
                None
            } else {
                Some(name.as_str())
            };
            let em = if mail.is_empty() {
                None
            } else {
                Some(mail.as_str())
            };
            match client.update_profile(dn, em).await {
                Ok(_) => set_msg.set("Profile saved successfully.".to_string()),
                Err(e) => set_msg.set(format!("Failed to save: {}", e)),
            }
            set_saving.set(false);
        });
    };

    view! {
        <div class="space-y-6">
            <SettingsSection title="Profile" description="Your account information">
                <div class="space-y-4">
                    <div>
                        <label for="settings-display-name" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">"Display Name"</label>
                        <input id="settings-display-name" type="text" prop:value={move || display_name.get()}
                            on:input=move |ev| set_display_name.set(event_target_value(&ev))
                            class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-none bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100"
                            disabled=move || !profile_loaded.get() />
                    </div>
                    <div>
                        <label for="settings-email" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">"Email"</label>
                        <input id="settings-email" type="email" prop:value={move || email.get()} readonly
                            class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-none bg-gray-100 dark:bg-gray-600 text-gray-500 dark:text-gray-400 cursor-not-allowed" />
                        <p class="text-xs text-gray-500 mt-1">"Email is read-only and managed by your authentication provider."</p>
                    </div>
                    <div>
                        <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">"Avatar"</label>
                        <div class="w-16 h-16 bg-blue-600 rounded-full flex items-center justify-center">
                            <span class="text-white text-xl font-medium">{move || display_name.get().chars().next().unwrap_or('U').to_uppercase().to_string()}</span>
                        </div>
                    </div>
                    <div class="flex items-center gap-3">
                        <button on:click=on_save_profile disabled=move || saving_profile.get() || !profile_loaded.get()
                            class="px-4 py-2 bg-blue-600 hover:bg-blue-700 disabled:bg-blue-400 text-white rounded-none transition-colors">
                            {move || if saving_profile.get() { "Saving..." } else { "Save Profile" }}
                        </button>
                        <span class="text-sm text-gray-600 dark:text-gray-400">{move || profile_msg.get()}</span>
                    </div>
                </div>
            </SettingsSection>
        </div>
    }
}

#[component]
fn AccountTab() -> impl IntoView {
    let (old_password, set_old_password) = signal(String::new());
    let (new_password, set_new_password) = signal(String::new());
    let (confirm_password, set_confirm_password) = signal(String::new());
    let (password_msg, set_password_msg) = signal(String::new());
    let (saving_password, set_saving_password) = signal(false);

    let on_change_password = move |_| {
        let old_pw = old_password.get();
        let new_pw = new_password.get();
        let confirm = confirm_password.get();
        if old_pw.is_empty() || new_pw.is_empty() {
            set_password_msg.set("All fields are required.".to_string());
            return;
        }
        if new_pw != confirm {
            set_password_msg.set("New passwords do not match.".to_string());
            return;
        }
        set_saving_password.set(true);
        set_password_msg.set(String::new());
        let set_msg = set_password_msg;
        let set_saving = set_saving_password;
        let set_old = set_old_password;
        let set_new = set_new_password;
        let set_confirm = set_confirm_password;
        wasm_bindgen_futures::spawn_local(async move {
            match ApiClient::default().change_password(&old_pw, &new_pw).await {
                Ok(_) => {
                    set_msg.set("Password changed successfully.".to_string());
                    set_old.set(String::new());
                    set_new.set(String::new());
                    set_confirm.set(String::new());
                }
                Err(e) => set_msg.set(format!("Failed to change password: {}", e)),
            }
            set_saving.set(false);
        });
    };

    let on_logout = move |_| {
        if let Some(window) = web_sys::window() {
            let _ = window.location().set_href("/login");
        }
    };

    view! {
        <div class="space-y-6">
            <SettingsSection title="Change Password" description="Update your password">
                <div class="space-y-4">
                    <div>
                        <label for="settings-current-password" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">"Current Password"</label>
                        <input id="settings-current-password" type="password" prop:value={move || old_password.get()}
                            on:input=move |ev| set_old_password.set(event_target_value(&ev))
                            class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-none bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100" />
                    </div>
                    <div>
                        <label for="settings-new-password" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">"New Password"</label>
                        <input id="settings-new-password" type="password" prop:value={move || new_password.get()}
                            on:input=move |ev| set_new_password.set(event_target_value(&ev))
                            class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-none bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100" />
                    </div>
                    <div>
                        <label for="settings-confirm-password" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">"Confirm New Password"</label>
                        <input id="settings-confirm-password" type="password" prop:value={move || confirm_password.get()}
                            on:input=move |ev| set_confirm_password.set(event_target_value(&ev))
                            class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-none bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100" />
                    </div>
                    <div class="flex items-center gap-3">
                        <button on:click=on_change_password disabled=move || saving_password.get()
                            class="px-4 py-2 bg-blue-600 hover:bg-blue-700 disabled:bg-blue-400 text-white rounded-none transition-colors">
                            {move || if saving_password.get() { "Changing..." } else { "Change Password" }}
                        </button>
                        <span class="text-sm text-gray-600 dark:text-gray-400">{move || password_msg.get()}</span>
                    </div>
                </div>
            </SettingsSection>
            <SettingsSection title="Session" description="Manage your session">
                <button on:click=on_logout class="px-4 py-2 bg-gray-200 dark:bg-gray-700 hover:bg-gray-300 dark:hover:bg-gray-600 text-gray-700 dark:text-gray-300 rounded-none transition-colors">
                    "Logout"
                </button>
            </SettingsSection>
        </div>
    }
}

#[component]
fn PreferencesTab() -> impl IntoView {
    let (theme, set_theme) = signal(get_stored_theme());
    let (language, set_language) = signal("en".to_string());
    let (font_size, set_font_size) = signal(16u32);
    let (notifications_enabled, set_notifications_enabled) = signal(true);

    let on_theme_change = move |new_theme: String| {
        set_theme.set(new_theme.clone());
        apply_theme(&new_theme);
        save_theme_to_storage(&new_theme);
    };

    view! {
        <div class="space-y-6">
            <SettingsSection title="Appearance" description="Customize how Tachyon looks">
                <div class="space-y-4">
                    <div>
                        <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-3">"Theme"</label>
                        <div class="flex gap-4">
                            <ThemeOption label="Light" value="light" current_theme=theme.get() on_change=on_theme_change />
                            <ThemeOption label="Dark" value="dark" current_theme=theme.get() on_change=on_theme_change />
                            <ThemeOption label="System" value="system" current_theme=theme.get() on_change=on_theme_change />
                        </div>
                    </div>
                </div>
            </SettingsSection>
            <SettingsSection title="Editor" description="Configure editor settings">
                <div class="space-y-4">
                    <div>
                        <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                            {move || format!("Font Size: {}px", font_size.get())}
                        </label>
                        <input type="range" min="12" max="24" step="1" prop:value={move || font_size.get().to_string()}
                            on:input=move |ev| { if let Ok(v) = event_target_value(&ev).parse::<u32>() { set_font_size.set(v); } }
                            class="w-full max-w-xs" />
                    </div>
                </div>
            </SettingsSection>
            <SettingsSection title="Notifications" description="Configure notifications">
                <div class="flex items-center justify-between">
                    <div>
                        <p class="text-sm font-medium text-gray-700 dark:text-gray-300">"Email Notifications"</p>
                        <p class="text-sm text-gray-500 dark:text-gray-400">"Receive email updates about your activity"</p>
                    </div>
                    <ToggleSwitch enabled=notifications_enabled.get() on_toggle=move |_| set_notifications_enabled.update(|e| *e = !*e) label="Email Notifications".to_string() />
                </div>
            </SettingsSection>
            <SettingsSection title="Language" description="Select your preferred language">
                <select class="w-full max-w-xs px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-none bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100"
                    on:change=move |ev| set_language.set(event_target_value(&ev))>
                    <option value="en" selected={language.get() == "en"}>"English"</option>
                    <option value="zh" selected={language.get() == "zh"}>"中文"</option>
                    <option value="es" selected={language.get() == "es"}>"Español"</option>
                    <option value="ja" selected={language.get() == "ja"}>"日本語"</option>
                </select>
            </SettingsSection>
        </div>
    }
}

#[component]
fn DangerTab() -> impl IntoView {
    let (delete_confirm, set_delete_confirm) = signal(String::new());
    let (deleting, set_deleting) = signal(false);
    let (danger_msg, set_danger_msg) = signal(String::new());

    let on_delete_account = move |_| {
        let confirm = delete_confirm.get();
        if confirm != "DELETE" {
            set_danger_msg.set("Type DELETE to confirm.".to_string());
            return;
        }
        set_deleting.set(true);
        set_danger_msg.set(String::new());
        let set_del = set_deleting;
        wasm_bindgen_futures::spawn_local(async move {
            let client = ApiClient::default();
            let _ = client.delete_account().await;
            client.clear_auth_token();
            if let Some(window) = web_sys::window() {
                if let Ok(Some(s)) = window.local_storage() {
                    let _ = s.remove_item("tachyon_token");
                }
                let _ = window.location().set_href("/login");
            }
            set_del.set(false);
        });
    };

    view! {
        <div class="space-y-6">
            <SettingsSection title="Danger Zone" description="Irreversible actions">
                <div class="space-y-4">
                    <div class="p-4 border border-red-200 dark:border-red-800 rounded-none bg-red-50 dark:bg-red-900/10">
                        <h3 class="text-sm font-semibold text-red-800 dark:text-red-300 mb-2">"Delete Account"</h3>
                        <p class="text-sm text-red-700 dark:text-red-400 mb-4">"Once you delete your account, there is no going back. All your data will be permanently removed."</p>
                        <div class="space-y-3">
                            <label for="settings-delete-confirm" class="sr-only">"Type DELETE to confirm account deletion"</label>
                            <input id="settings-delete-confirm" type="text" placeholder="Type DELETE to confirm"
                                prop:value={move || delete_confirm.get()}
                                on:input=move |ev| set_delete_confirm.set(event_target_value(&ev))
                                class="w-full px-3 py-2 border border-red-300 dark:border-red-700 rounded-none bg-white dark:bg-gray-800 text-gray-900 dark:text-white focus:ring-2 focus:ring-red-500" />
                            <div class="flex items-center gap-3">
                                <button on:click=on_delete_account disabled=move || deleting.get() || delete_confirm.get() != "DELETE"
                                    class="px-4 py-2 bg-red-600 hover:bg-red-700 disabled:bg-red-400 text-white rounded-none transition-colors">
                                    {move || if deleting.get() { "Deleting..." } else { "Delete Account" }}
                                </button>
                                <span class="text-sm text-red-600 dark:text-red-400">{move || danger_msg.get()}</span>
                            </div>
                        </div>
                    </div>
                </div>
            </SettingsSection>
        </div>
    }
}

#[component]
fn SettingsSection(
    title: &'static str,
    description: &'static str,
    children: Children,
) -> impl IntoView {
    view! {
        <div class="bg-white dark:bg-gray-800 rounded-none shadow border-2 border-gray-900 dark:border-gray-100 p-6">
            <div class="mb-4">
                <h2 class="text-lg font-semibold text-gray-900 dark:text-white">{title}</h2>
                <p class="text-sm text-gray-500 dark:text-gray-400">{description}</p>
            </div>
            {children()}
        </div>
    }
}

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
            class=move || if is_selected {
                "flex-1 px-4 py-3 border-2 border-blue-500 rounded-none bg-blue-50 dark:bg-blue-900/20 text-blue-700 dark:text-blue-300 font-medium transition-colors"
            } else {
                "flex-1 px-4 py-3 border border-gray-300 dark:border-gray-600 rounded-none bg-white dark:bg-gray-700 text-gray-700 dark:text-gray-300 hover:border-gray-400 dark:hover:border-gray-500 transition-colors"
            }
            on:click=move |_| on_change_clone(value_str.clone())
        >{label}</button>
    }
}

#[component]
fn ToggleSwitch<F>(enabled: bool, on_toggle: F, label: String) -> impl IntoView
where
    F: Fn(leptos::ev::MouseEvent) + 'static,
{
    view! {
        <button type="button" on:click=on_toggle
            role="switch"
            attr:aria-checked=move || if enabled { "true" } else { "false" }
            attr:aria-label=label.clone()
            class=move || if enabled {
                "relative inline-flex h-6 w-11 flex-shrink-0 cursor-pointer rounded-full border-2 border-transparent bg-blue-600 transition-colors"
            } else {
                "relative inline-flex h-6 w-11 flex-shrink-0 cursor-pointer rounded-full border-2 border-transparent bg-gray-200 dark:bg-gray-600 transition-colors"
            }>
            <span class=move || if enabled {
                "pointer-events-none inline-block h-5 w-5 transform rounded-full bg-white shadow ring-0 transition translate-x-5"
            } else {
                "pointer-events-none inline-block h-5 w-5 transform rounded-full bg-white shadow ring-0 transition translate-x-0"
            } />
        </button>
    }
}

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

fn save_theme_to_storage(theme: &str) {
    if let Some(window) = web_sys::window() {
        if let Ok(Some(storage)) = window.local_storage() {
            let _ = storage.set_item("tachyon-theme", theme);
        }
    }
}

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

fn get_system_theme() -> String {
    if let Ok(Some(media_query)) = match_media("(prefers-color-scheme: dark)") {
        if media_query.matches() {
            return "dark".to_string();
        }
    }
    "light".to_string()
}
