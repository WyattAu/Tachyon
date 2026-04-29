#![allow(dead_code, clippy::redundant_locals)]

use crate::api::ApiClient;
use leptos::prelude::*;
use wasm_bindgen_futures::spawn_local;

#[component]
pub fn ProfilePage() -> impl IntoView {
    let (display_name, set_display_name) = signal(String::new());
    let (username, set_username) = signal(String::new());
    let (email, set_email) = signal(String::new());
    let (bio, set_bio) = signal(String::new());
    let (avatar_url, set_avatar_url) = signal(String::new());
    let (timezone, set_timezone) = signal("UTC".to_string());
    let (mfa_enabled, set_mfa_enabled) = signal(false);
    let (loading, set_loading) = signal(true);
    let (profile_msg, set_profile_msg) = signal(None::<(String, bool)>);
    let (saving, set_saving) = signal(false);
    let (delete_confirm, set_delete_confirm) = signal(String::new());
    let (deleting, set_deleting) = signal(false);
    let (show_delete_confirm, set_show_delete_confirm) = signal(false);

    Effect::new(move |_| {
        let api = ApiClient::default();
        spawn_local(async move {
            if let Ok(user) = api.get_current_user().await {
                let name = user
                    .get("display_name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let uname = user
                    .get("username")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let mail = user
                    .get("email")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let b = user
                    .get("bio")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let tz = user
                    .get("timezone")
                    .and_then(|v| v.as_str())
                    .unwrap_or("UTC")
                    .to_string();
                let mfa = user
                    .get("mfa_enabled")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                let av = user
                    .get("avatar_url")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                set_display_name.set(name);
                set_username.set(uname);
                set_email.set(mail);
                set_bio.set(b);
                set_timezone.set(tz);
                set_mfa_enabled.set(mfa);
                set_avatar_url.set(av);
            }
            set_loading.set(false);
        });
    });

    let on_save_profile = move |_| {
        let client = ApiClient::default();
        let name = display_name.get();
        let b = bio.get();
        let tz = timezone.get();
        set_saving.set(true);
        set_profile_msg.set(None);
        let set_msg = set_profile_msg;
        let set_saving = set_saving;
        spawn_local(async move {
            let dn = if name.is_empty() {
                None
            } else {
                Some(name.as_str())
            };
            let em: Option<&str> = None;
            match client.update_profile(dn, em).await {
                Ok(_) => {
                    let mut body = serde_json::Map::new();
                    body.insert("bio".to_string(), serde_json::json!(b));
                    body.insert("timezone".to_string(), serde_json::json!(tz));
                    match client.update_user_settings(&body).await {
                        Ok(_) => {
                            set_msg.set(Some(("Profile saved successfully.".to_string(), true)))
                        }
                        Err(e) => {
                            set_msg.set(Some((format!("Failed to save settings: {}", e), false)))
                        }
                    }
                }
                Err(e) => set_msg.set(Some((format!("Failed to save: {}", e), false))),
            }
            set_saving.set(false);
        });
    };

    let on_toggle_mfa = move |_| {
        let api = ApiClient::default();
        let current_mfa = mfa_enabled.get();
        spawn_local(async move {
            if current_mfa {
                let _ = api.disable_mfa().await;
            } else {
                let _ = api.enable_mfa().await;
            }
            if let Ok(user) = api.get_current_user().await {
                let mfa = user
                    .get("mfa_enabled")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                set_mfa_enabled.set(mfa);
            }
        });
    };

    let on_delete_account = move |_| {
        let confirm = delete_confirm.get();
        if confirm != "DELETE" {
            set_profile_msg.set(Some(("Type DELETE to confirm.".to_string(), false)));
            return;
        }
        set_deleting.set(true);
        spawn_local(async move {
            let client = ApiClient::default();
            let _ = client.delete_account().await;
            client.clear_auth_token();
            if let Some(window) = web_sys::window() {
                if let Ok(Some(s)) = window.local_storage() {
                    let _ = s.remove_item("tachyon_token");
                }
                let _ = window.location().set_href("/login");
            }
        });
    };

    let timezone_options = [
        "UTC",
        "America/New_York",
        "America/Chicago",
        "America/Denver",
        "America/Los_Angeles",
        "Europe/London",
        "Europe/Paris",
        "Europe/Berlin",
        "Asia/Tokyo",
        "Asia/Shanghai",
        "Asia/Kolkata",
        "Australia/Sydney",
    ];

    view! {
        <div class="max-w-3xl">
            <h1 class="text-2xl font-bold mb-6 text-gray-900 dark:text-white">"Profile"</h1>

            {move || loading.get().then(|| view! {
                <div class="flex justify-center items-center py-12">
                    <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600"></div>
                </div>
            })}

            {move || if !loading.get() {
                Some(view! {
                    <div class="space-y-6">

                        {move || profile_msg.get().map(|(msg, ok)| view! {
                            <div class={
                                if ok { "p-4 bg-green-100 text-green-700 dark:bg-green-900 dark:text-green-200 rounded-lg" }
                                else { "p-4 bg-red-100 text-red-700 dark:bg-red-900 dark:text-red-200 rounded-lg" }
                            }>
                                <div class="flex items-center justify-between">
                                    <span>{msg}</span>
                                    <button on:click=move |_| set_profile_msg.set(None) class="text-sm underline">"Dismiss"</button>
                                </div>
                            </div>
                        })}

                        <ProfileCard
                            display_name=display_name
                            username=username
                            email=email
                            avatar_url=avatar_url
                        />

                        <ProfileSection title="Edit Profile" description="Update your personal information">
                            <div class="space-y-4">
                                <div>
                                    <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">"Display Name"</label>
                                    <input type="text" prop:value={move || display_name.get()}
                                        on:input=move |ev| set_display_name.set(event_target_value(&ev))
                                        class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 focus:ring-2 focus:ring-blue-500 focus:border-blue-500" />
                                </div>
                                <div>
                                    <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">"Username"</label>
                                    <input type="text" prop:value={move || username.get()} readonly
                                        class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-gray-100 dark:bg-gray-600 text-gray-500 dark:text-gray-400 cursor-not-allowed" />
                                    <p class="text-xs text-gray-400 mt-1">"Username is read-only."</p>
                                </div>
                                <div>
                                    <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">"Email"</label>
                                    <input type="email" prop:value={move || email.get()} readonly
                                        class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-gray-100 dark:bg-gray-600 text-gray-500 dark:text-gray-400 cursor-not-allowed" />
                                    <p class="text-xs text-gray-400 mt-1">"Email is managed by your authentication provider."</p>
                                </div>
                                <div>
                                    <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">"Bio"</label>
                                    <textarea rows="3" prop:value={move || bio.get()}
                                        on:input=move |ev| set_bio.set(event_target_value(&ev))
                                        class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 focus:ring-2 focus:ring-blue-500 focus:border-blue-500 resize-none"
                                        placeholder="Tell us a bit about yourself..."
                                    ></textarea>
                                </div>
                                <div>
                                    <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">"Timezone"</label>
                                    <select
                                        class="w-full max-w-xs px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
                                        on:change=move |ev| set_timezone.set(event_target_value(&ev))
                                    >
                                        {timezone_options.iter().map(|tz| {
                                            let tz_s = tz.to_string();
                                            let current = timezone.get();
                                            let selected = current == tz_s;
                                            let label = tz_s.clone();
                                            view! {
                                                <option value=tz_s.clone() selected={selected}>{label}</option>
                                            }
                                        }).collect::<Vec<_>>()}
                                    </select>
                                </div>
                                <div class="flex items-center gap-3">
                                    <button on:click=on_save_profile disabled=move || saving.get()
                                        class="px-4 py-2 bg-blue-600 hover:bg-blue-700 disabled:bg-blue-400 text-white rounded-lg transition-colors">
                                        {move || if saving.get() { "Saving..." } else { "Save Profile" }}
                                    </button>
                                </div>
                            </div>
                        </ProfileSection>

                        <ProfileSection title="Two-Factor Authentication" description="Add an extra layer of security to your account">
                            <div class="flex items-center justify-between">
                                <div>
                                    <p class="text-sm font-medium text-gray-700 dark:text-gray-300">
                                        {move || if mfa_enabled.get() { "MFA is enabled" } else { "MFA is disabled" }}
                                    </p>
                                    <p class="text-sm text-gray-500 dark:text-gray-400">
                                        {move || if mfa_enabled.get() { "Your account is protected with two-factor authentication." } else { "Enable MFA for enhanced account security." }}
                                    </p>
                                </div>
                                <button
                                    on:click=on_toggle_mfa
                                    class={
                                        move || if mfa_enabled.get() {
                                            "px-4 py-2 bg-red-600 hover:bg-red-700 text-white rounded-lg transition-colors text-sm font-medium"
                                        } else {
                                            "px-4 py-2 bg-blue-600 hover:bg-blue-700 text-white rounded-lg transition-colors text-sm font-medium"
                                        }
                                    }
                                >
                                    {move || if mfa_enabled.get() { "Disable MFA" } else { "Enable MFA" }}
                                </button>
                            </div>
                        </ProfileSection>

                        <ProfileSection title="Danger Zone" description="Irreversible and destructive actions">
                            <div class="p-4 border border-red-200 dark:border-red-800 rounded-lg bg-red-50 dark:bg-red-900/10">
                                <h3 class="text-sm font-semibold text-red-800 dark:text-red-300 mb-2">"Delete Account"</h3>
                                <p class="text-sm text-red-700 dark:text-red-400 mb-4">"Once you delete your account, there is no going back. All your data will be permanently removed."</p>
                                {move || if show_delete_confirm.get() {
                                    Some(view! {
                                        <div class="space-y-3">
                                            <input type="text" placeholder="Type DELETE to confirm"
                                                prop:value={move || delete_confirm.get()}
                                                on:input=move |ev| set_delete_confirm.set(event_target_value(&ev))
                                                class="w-full px-3 py-2 border border-red-300 dark:border-red-700 rounded-lg bg-white dark:bg-gray-800 text-gray-900 dark:text-white focus:ring-2 focus:ring-red-500"
                                            />
                                            <div class="flex items-center gap-3">
                                                <button on:click=on_delete_account disabled=move || deleting.get() || delete_confirm.get() != "DELETE"
                                                    class="px-4 py-2 bg-red-600 hover:bg-red-700 disabled:bg-red-400 text-white rounded-lg transition-colors">
                                                    {move || if deleting.get() { "Deleting..." } else { "Delete Account" }}
                                                </button>
                                                <button on:click=move |_| { set_show_delete_confirm.set(false); set_delete_confirm.set(String::new()); }
                                                    class="px-4 py-2 text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-lg transition-colors">
                                                    "Cancel"
                                                </button>
                                            </div>
                                        </div>
                                    }.into_any())
                                } else {
                                    Some(view! {
                                        <button on:click=move |_| set_show_delete_confirm.set(true)
                                            class="px-4 py-2 bg-red-600 hover:bg-red-700 text-white rounded-lg transition-colors">
                                            "Delete Account"
                                        </button>
                                    }.into_any())
                                }}
                            </div>
                        </ProfileSection>
                    </div>
                }.into_any())
            } else {
                None
            }}
        </div>
    }
}

#[component]
fn ProfileCard(
    display_name: ReadSignal<String>,
    username: ReadSignal<String>,
    email: ReadSignal<String>,
    avatar_url: ReadSignal<String>,
) -> impl IntoView {
    let _ = avatar_url;
    view! {
        <div class="bg-white dark:bg-gray-800 rounded-lg shadow border border-gray-200 dark:border-gray-700 p-6">
            <div class="flex items-center gap-4">
                {move || {
                    let dn = display_name.get();
                    let name = if dn.is_empty() { username.get() } else { dn };
                    let initial = name.chars().next().unwrap_or('U').to_uppercase().to_string();
                    view! {
                        <div class="w-16 h-16 bg-blue-600 rounded-full flex items-center justify-center">
                            <span class="text-white text-xl font-medium">{initial}</span>
                        </div>
                    }
                }}
                <div>
                    <h2 class="text-lg font-semibold text-gray-900 dark:text-white">{move || display_name.get()}</h2>
                    <p class="text-sm text-gray-500 dark:text-gray-400">"@"{move || username.get()}</p>
                    <p class="text-sm text-gray-500 dark:text-gray-400">{move || email.get()}</p>
                </div>
            </div>
        </div>
    }
}

#[component]
fn ProfileSection(
    title: &'static str,
    description: &'static str,
    children: Children,
) -> impl IntoView {
    view! {
        <div class="bg-white dark:bg-gray-800 rounded-lg shadow border border-gray-200 dark:border-gray-700 p-6">
            <div class="mb-4">
                <h2 class="text-lg font-semibold text-gray-900 dark:text-white">{title}</h2>
                <p class="text-sm text-gray-500 dark:text-gray-400">{description}</p>
            </div>
            {children()}
        </div>
    }
}
