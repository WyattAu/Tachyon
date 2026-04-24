// Admin Users Page
// User management with listing, creation, role editing, and deactivation

use leptos::prelude::*;
use crate::api::ApiClient;

/// Admin users management page
#[component]
pub fn UsersPage() -> impl IntoView {
    let (users, set_users) = signal(Vec::<serde_json::Value>::new());
    let (total, set_total) = signal(0i64);
    let (page, set_page) = signal(1usize);
    let (loading, set_loading) = signal(true);
    let (message, set_message) = signal(String::new());
    let (role_filter, set_role_filter) = signal(String::new());

    // Create user modal
    let (show_create, set_show_create) = signal(false);
    let (new_username, set_new_username) = signal(String::new());
    let (new_display_name, set_new_display_name) = signal(String::new());
    let (new_email, set_new_email) = signal(String::new());
    let (new_password, set_new_password) = signal(String::new());
    let (new_role, set_new_role) = signal(String::from("reader"));
    let (creating, set_creating) = signal(false);

    let fetch_users = move || {
        let api = ApiClient::default();
        let set_u = set_users.clone();
        let set_t = set_total.clone();
        let set_l = set_loading.clone();
        let current_page = page.get();
        let filter = role_filter.get();
        wasm_bindgen_futures::spawn_local(async move {
            set_l.set(true);
            let role_arg = if filter.is_empty() { None } else { Some(filter.as_str()) };
            match api.list_users(Some(current_page), Some(20), role_arg).await {
                Ok(resp) => {
                    let u = resp.get("users")
                        .and_then(|v| v.as_array())
                        .map(|arr| arr.clone())
                        .unwrap_or_default();
                    let t = resp.get("total").and_then(|v| v.as_i64()).unwrap_or(0);
                    set_u.set(u);
                    set_t.set(t);
                }
                Err(e) => {
                    set_message.set(format!("Failed to load users: {}", e));
                }
            }
            set_l.set(false);
        });
    };

    // Load on mount
    Effect::new(move |_| {
        fetch_users();
    });

    let on_prev_page = move |_: leptos::ev::MouseEvent| {
        if page.get() > 1 {
            set_page.update(|p| { *p -= 1; });
            fetch_users();
        }
    };

    let on_next_page = move |_: leptos::ev::MouseEvent| {
        let max_page = ((total.get() as usize) + 19) / 20;
        if page.get() < max_page {
            set_page.update(|p| { *p += 1; });
            fetch_users();
        }
    };

    let on_filter_change = move |ev: leptos::ev::Event| {
        let val = event_target_value(&ev);
        set_role_filter.set(val);
        set_page.set(1);
        fetch_users();
    };

    let on_create_user = move |_: leptos::ev::MouseEvent| {
        let username = new_username.get();
        let display_name = new_display_name.get();
        let email = new_email.get();
        let password = new_password.get();
        let role = new_role.get();

        if username.len() < 3 {
            set_message.set("Username must be at least 3 characters.".to_string());
            return;
        }
        if display_name.is_empty() {
            set_message.set("Display name is required.".to_string());
            return;
        }
        if password.len() < 8 {
            set_message.set("Password must be at least 8 characters.".to_string());
            return;
        }

        set_creating.set(true);
        set_message.set(String::new());
        let api = ApiClient::default();
        wasm_bindgen_futures::spawn_local(async move {
            let body = serde_json::json!({
                "username": username,
                "display_name": display_name,
                "email": if email.is_empty() { serde_json::Value::Null } else { serde_json::Value::String(email) },
                "password": password,
                "role": role,
            });
            match api.create_user(&body).await {
                Ok(_) => {
                    set_message.set("User created successfully.".to_string());
                    set_show_create.set(false);
                    set_new_username.set(String::new());
                    set_new_display_name.set(String::new());
                    set_new_email.set(String::new());
                    set_new_password.set(String::new());
                    set_new_role.set(String::from("reader"));
                    fetch_users();
                }
                Err(e) => {
                    set_message.set(format!("Failed to create user: {}", e));
                }
            }
            set_creating.set(false);
        });
    };

    let on_deactivate = move |user_id: String, username: String| {
        let api = ApiClient::default();
        let set_m = set_message.clone();
        let fetch = {
            let api = ApiClient::default();
            let set_u = set_users.clone();
            let set_t = set_total.clone();
            let set_l = set_loading.clone();
            let current_page = page.get();
            let filter = role_filter.get();
            move || {
                let api = api.clone();
                let set_u = set_u.clone();
                let set_t = set_t.clone();
                let set_l = set_l.clone();
                let filter = filter.clone();
                wasm_bindgen_futures::spawn_local(async move {
                    set_l.set(true);
                    let role_arg = if filter.is_empty() { None } else { Some(filter.as_str()) };
                    if let Ok(resp) = api.list_users(Some(current_page), Some(20), role_arg).await {
                        let u = resp.get("users").and_then(|v| v.as_array()).map(|arr| arr.clone()).unwrap_or_default();
                        let t = resp.get("total").and_then(|v| v.as_i64()).unwrap_or(0);
                        set_u.set(u);
                        set_t.set(t);
                    }
                    set_l.set(false);
                });
            }
        };
        wasm_bindgen_futures::spawn_local(async move {
            match api.delete_user(&user_id).await {
                Ok(_) => {
                    set_m.set(format!("User '{}' deactivated.", username));
                    fetch();
                }
                Err(e) => {
                    set_m.set(format!("Failed to deactivate user: {}", e));
                }
            }
        });
    };

    let on_change_role = move |user_id: String, new_role: String| {
        let api = ApiClient::default();
        let set_m = set_message.clone();
        wasm_bindgen_futures::spawn_local(async move {
            match api.update_user(&user_id, &serde_json::json!({ "role": new_role })).await {
                Ok(_) => {
                    set_m.set("User role updated.".to_string());
                    // Refresh list
                    let api2 = ApiClient::default();
                    if let Ok(resp) = api2.list_users(Some(page.get()), Some(20), None).await {
                        let u = resp.get("users").and_then(|v| v.as_array()).map(|arr| arr.clone()).unwrap_or_default();
                        set_users.set(u);
                    }
                }
                Err(e) => {
                    set_m.set(format!("Failed to update role: {}", e));
                }
            }
        });
    };

    let max_page = move || ((total.get() as usize) + 19) / 20;

    let role_badge_class = |role: &str| -> String {
        match role {
            "admin" => "px-2 py-0.5 text-xs font-medium rounded-full bg-red-100 dark:bg-red-900/30 text-red-700 dark:text-red-300".to_string(),
            "editor" => "px-2 py-0.5 text-xs font-medium rounded-full bg-blue-100 dark:bg-blue-900/30 text-blue-700 dark:text-blue-300".to_string(),
            "writer" => "px-2 py-0.5 text-xs font-medium rounded-full bg-green-100 dark:bg-green-900/30 text-green-700 dark:text-green-300".to_string(),
            _ => "px-2 py-0.5 text-xs font-medium rounded-full bg-gray-100 dark:bg-gray-700 text-gray-700 dark:text-gray-300".to_string(),
        }
    };

    view! {
        <div class="max-w-5xl">
            <div class="flex items-center justify-between mb-6">
                <h1 class="text-2xl font-bold text-gray-900 dark:text-white">"User Management"</h1>
                <button
                    on:click=move |_| set_show_create.set(true)
                    class="px-4 py-2 bg-blue-600 hover:bg-blue-700 text-white rounded-lg transition-colors text-sm font-medium"
                >
                    "Create User"
                </button>
            </div>

            // Message bar
            {move || {
                let msg = message.get();
                if msg.is_empty() {
                    view! { <div></div> }.into_any()
                } else {
                    view! {
                        <div class="mb-4 px-4 py-3 bg-blue-50 dark:bg-blue-900/20 border border-blue-200 dark:border-blue-800 rounded-lg text-sm text-blue-700 dark:text-blue-300">
                            {msg}
                            <button
                                on:click=move |_| set_message.set(String::new())
                                class="ml-2 text-blue-500 hover:text-blue-700 dark:hover:text-blue-200"
                            >
                                "×"
                            </button>
                        </div>
                    }.into_any()
                }
            }}

            // Filters
            <div class="flex items-center gap-4 mb-4">
                <select
                    class="px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 text-sm"
                    on:change=on_filter_change
                    prop:value=move || role_filter.get()
                >
                    <option value="">"All Roles"</option>
                    <option value="admin">"Admin"</option>
                    <option value="editor">"Editor"</option>
                    <option value="writer">"Writer"</option>
                    <option value="reader">"Reader"</option>
                </select>
                <span class="text-sm text-gray-500 dark:text-gray-400">
                    {move || format!("{} user{}", total.get(), if total.get() == 1 { "" } else { "s" })}
                </span>
            </div>

            // Loading skeleton
            {move || if loading.get() {
                view! {
                    <div class="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 overflow-hidden">
                        <div class="divide-y divide-gray-200 dark:divide-gray-700">
                            {vec![0, 1, 2, 3, 4].into_iter().map(|_| {
                                view! {
                                    <div class="px-6 py-4 animate-pulse">
                                        <div class="flex items-center gap-4">
                                            <div class="h-4 bg-gray-200 dark:bg-gray-700 rounded w-24"></div>
                                            <div class="h-4 bg-gray-200 dark:bg-gray-700 rounded w-32"></div>
                                            <div class="h-4 bg-gray-200 dark:bg-gray-700 rounded w-40"></div>
                                            <div class="h-4 bg-gray-200 dark:bg-gray-700 rounded w-16"></div>
                                        </div>
                                    </div>
                                }
                            }).collect::<Vec<_>>()}
                        </div>
                    </div>
                }.into_any()
            } else {
                view! {
                    // Users table
                    <div class="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 overflow-hidden">
                        <div class="overflow-x-auto">
                            <table class="min-w-full divide-y divide-gray-200 dark:divide-gray-700">
                                <thead class="bg-gray-50 dark:bg-gray-900/50">
                                    <tr>
                                        <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">"Username"</th>
                                        <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">"Display Name"</th>
                                        <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">"Email"</th>
                                        <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">"Role"</th>
                                        <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">"Status"</th>
                                        <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">"Created"</th>
                                        <th class="px-6 py-3 text-right text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">"Actions"</th>
                                    </tr>
                                </thead>
                                <tbody class="divide-y divide-gray-200 dark:divide-gray-700">
                                    {move || {
                                        let us = users.get();
                                        if us.is_empty() {
                                            view! {
                                                <tr>
                                                    <td colspan="7" class="px-6 py-12 text-center text-gray-500 dark:text-gray-400">
                                                        "No users found"
                                                    </td>
                                                </tr>
                                            }.into_any()
                                        } else {
                                            view! {
                                                {us.into_iter().map(|u| {
                                                    let uid = u.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string();
                                                    let username = u.get("username").and_then(|v| v.as_str()).unwrap_or("").to_string();
                                                    let display_name = u.get("display_name").and_then(|v| v.as_str()).unwrap_or("").to_string();
                                                    let email = u.get("email").and_then(|v| v.as_str()).unwrap_or("").to_string();
                                                    let role = u.get("role").and_then(|v| v.as_str()).unwrap_or("reader").to_string();
                                                    let is_active = u.get("is_active").and_then(|v| v.as_bool()).unwrap_or(true);
                                                    let created = u.get("created_at").and_then(|v| v.as_str()).unwrap_or("");
                                                    // Format date
                                                    let date_str = if created.is_empty() {
                                                        "N/A".to_string()
                                                    } else {
                                                        created.split('T').next().unwrap_or("N/A").to_string()
                                                    };

                                                    let on_deact = on_deactivate.clone();
                                                    let uid_for_deact = uid.clone();
                                                    let username_for_deact = username.clone();

                                                    view! {
                                                        <tr class="hover:bg-gray-50 dark:hover:bg-gray-700/50 transition-colors">
                                                            <td class="px-6 py-4 whitespace-nowrap text-sm font-medium text-gray-900 dark:text-white">
                                                                {username}
                                                            </td>
                                                            <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-600 dark:text-gray-300">
                                                                {display_name}
                                                            </td>
                                                            <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-500 dark:text-gray-400">
                                                                {if email.is_empty() { view! { <span class="text-gray-300 dark:text-gray-600">"—"</span> }.into_any() } else { view! { <span>{email}</span> }.into_any() }}
                                                            </td>
                                                            <td class="px-6 py-4 whitespace-nowrap">
                                                                <select
                                                                    class={role_badge_class(&role)}
                                                                    on:change=move |ev| {
                                                                        let new_role = event_target_value(&ev);
                                                                        on_change_role(uid.clone(), new_role);
                                                                    }
                                                                    prop:value=role
                                                                >
                                                                    <option value="admin" selected={role == "admin"}>"Admin"</option>
                                                                    <option value="editor" selected={role == "editor"}>"Editor"</option>
                                                                    <option value="writer" selected={role == "writer"}>"Writer"</option>
                                                                    <option value="reader" selected={role == "reader"}>"Reader"</option>
                                                                </select>
                                                            </td>
                                                            <td class="px-6 py-4 whitespace-nowrap">
                                                                {if is_active {
                                                                    view! {
                                                                        <span class="px-2 py-0.5 text-xs font-medium rounded-full bg-green-100 dark:bg-green-900/30 text-green-700 dark:text-green-300">"Active"</span>
                                                                    }.into_any()
                                                                } else {
                                                                    view! {
                                                                        <span class="px-2 py-0.5 text-xs font-medium rounded-full bg-gray-100 dark:bg-gray-700 text-gray-500 dark:text-gray-400">"Inactive"</span>
                                                                    }.into_any()
                                                                }}
                                                            </td>
                                                            <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-500 dark:text-gray-400">
                                                                {date_str}
                                                            </td>
                                                            <td class="px-6 py-4 whitespace-nowrap text-right">
                                                                {if is_active {
                                                                    view! {
                                                                        <button
                                                                            on:click=move |_| on_deact(uid_for_deact.clone(), username_for_deact.clone())
                                                                            class="text-xs text-red-600 dark:text-red-400 hover:underline"
                                                                        >
                                                                            "Deactivate"
                                                                        </button>
                                                                    }.into_any()
                                                                } else {
                                                                    view! { <span class="text-xs text-gray-400">"—"</span> }.into_any()
                                                                }}
                                                            </td>
                                                        </tr>
                                                    }
                                                }).collect::<Vec<_>>()}
                                            }.into_any()
                                        }
                                    }}
                                </tbody>
                            </table>
                        </div>

                        // Pagination
                        {move || {
                            let mp = max_page();
                            let cp = page.get();
                            if mp <= 1 {
                                view! { <div></div> }.into_any()
                            } else {
                                view! {
                                    <div class="px-6 py-3 border-t border-gray-200 dark:border-gray-700 flex items-center justify-between">
                                        <span class="text-sm text-gray-500 dark:text-gray-400">
                                            {format!("Page {} of {}", cp, mp)}
                                        </span>
                                        <div class="flex gap-2">
                                            <button
                                                on:click=on_prev_page
                                                disabled=move || cp <= 1
                                                class="px-3 py-1 text-sm rounded border border-gray-300 dark:border-gray-600 text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
                                            >
                                                "Previous"
                                            </button>
                                            <button
                                                on:click=on_next_page
                                                disabled=move || cp >= mp
                                                class="px-3 py-1 text-sm rounded border border-gray-300 dark:border-gray-600 text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
                                            >
                                                "Next"
                                            </button>
                                        </div>
                                    </div>
                                }.into_any()
                            }
                        }}
                    </div>
                }.into_any()
            }}

            // Create user modal
            {move || if show_create.get() {
                view! {
                    <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/50">
                        <div class="bg-white dark:bg-gray-800 rounded-xl shadow-xl border border-gray-200 dark:border-gray-700 w-full max-w-md mx-4 p-6">
                            <h2 class="text-lg font-semibold text-gray-900 dark:text-white mb-4">"Create User"</h2>
                            <div class="space-y-4">
                                <div>
                                    <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">"Username"</label>
                                    <input
                                        type="text"
                                        prop:value=move || new_username.get()
                                        on:input=move |ev| set_new_username.set(event_target_value(&ev))
                                        class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100"
                                        placeholder="johndoe"
                                    />
                                </div>
                                <div>
                                    <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">"Display Name"</label>
                                    <input
                                        type="text"
                                        prop:value=move || new_display_name.get()
                                        on:input=move |ev| set_new_display_name.set(event_target_value(&ev))
                                        class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100"
                                        placeholder="John Doe"
                                    />
                                </div>
                                <div>
                                    <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">"Email"</label>
                                    <input
                                        type="email"
                                        prop:value=move || new_email.get()
                                        on:input=move |ev| set_new_email.set(event_target_value(&ev))
                                        class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100"
                                        placeholder="john@example.com (optional)"
                                    />
                                </div>
                                <div>
                                    <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">"Password"</label>
                                    <input
                                        type="password"
                                        prop:value=move || new_password.get()
                                        on:input=move |ev| set_new_password.set(event_target_value(&ev))
                                        class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100"
                                        placeholder="Min 8 characters"
                                    />
                                </div>
                                <div>
                                    <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">"Role"</label>
                                    <select
                                        prop:value=move || new_role.get()
                                        on:change=move |ev| set_new_role.set(event_target_value(&ev))
                                        class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100"
                                    >
                                        <option value="reader">"Reader"</option>
                                        <option value="writer">"Writer"</option>
                                        <option value="editor">"Editor"</option>
                                        <option value="admin">"Admin"</option>
                                    </select>
                                </div>
                            </div>
                            <div class="flex justify-end gap-3 mt-6">
                                <button
                                    on:click=move |_| set_show_create.set(false)
                                    class="px-4 py-2 text-sm text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-lg transition-colors"
                                >
                                    "Cancel"
                                </button>
                                <button
                                    on:click=on_create_user
                                    disabled=move || creating.get()
                                    class="px-4 py-2 text-sm bg-blue-600 hover:bg-blue-700 disabled:bg-blue-400 text-white rounded-lg transition-colors"
                                >
                                    {move || if creating.get() { "Creating..." } else { "Create" }}
                                </button>
                            </div>
                        </div>
                    </div>
                }.into_any()
            } else {
                view! { <div></div> }.into_any()
            }}
        </div>
    }
}
