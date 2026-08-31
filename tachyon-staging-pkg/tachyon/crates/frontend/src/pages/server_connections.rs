//! Server connections management page.
//!
//! Allows users to add, remove, and switch between multiple Tachyon servers.
//! Each server has its own auth token and document namespace.

use crate::servers::{ServerEntry, ServerRegistry};
use leptos::ev;
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::hooks::use_navigate;

/// Server connections management page.
#[component]
pub fn ServerConnectionsPage() -> impl IntoView {
    let registry = RwSignal::new(ServerRegistry::load());
    let show_add_form = RwSignal::new(false);
    let new_name = RwSignal::new(String::new());
    let new_url = RwSignal::new(String::new());
    let error = RwSignal::new(None::<String>);
    let connecting = RwSignal::new(false);
    let nav = StoredValue::new(use_navigate());

    let on_add_server = move |ev: ev::SubmitEvent| {
        ev.prevent_default();
        let name = new_name.get();
        let url = new_url.get().trim().to_string();

        if name.is_empty() || url.is_empty() {
            error.set(Some("Name and URL are required.".to_string()));
            return;
        }

        // Normalize URL: remove trailing slash, ensure scheme
        let normalized_url = if !url.starts_with("http://") && !url.starts_with("https://") {
            format!("http://{}", url)
        } else {
            url
        }
        .trim_end_matches('/')
        .to_string();

        let server_id = uuid::Uuid::new_v4().to_string();
        let entry = ServerEntry {
            id: server_id.clone(),
            name: name.clone(),
            base_url: normalized_url,
            auth_token: None,
            last_connected: None,
        };

        registry.update(|r| {
            r.add_server(entry);
            r.save();
        });

        // Clear form
        new_name.set(String::new());
        new_url.set(String::new());
        show_add_form.set(false);
        error.set(None);
    };

    let on_remove = move |server_id: String| {
        registry.update(|r| {
            r.remove_server(&server_id);
            r.save();
        });
    };

    let on_connect = move |server_id: String| {
        connecting.set(true);
        error.set(None);
        let server_id_clone = server_id.clone();

        spawn_local(async move {
            // Check if already has a token
            let has_token = registry.with(|r| {
                r.servers
                    .get(&server_id_clone)
                    .and_then(|s| s.auth_token.as_ref())
                    .is_some()
            });

            if has_token {
                // Already authenticated — just switch
                registry.update(|r| {
                    r.set_active(&server_id_clone);
                    r.save();
                });
                nav.update_value(|n| n("/dashboard", Default::default()));
                connecting.set(false);
                return;
            }

            // Navigate to login page with server context
            // The login page will read the active server and set the token
            registry.update(|r| {
                r.set_active(&server_id_clone);
                r.save();
            });
            nav.update_value(|n| n("/login", Default::default()));
            connecting.set(false);
        });
    };

    let _on_edit_url = move |server_id: String| {
        // For now, remove and re-add. A full edit modal would be better long-term.
        // This is a placeholder — in practice, we'd open an inline edit form.
        let _ = server_id; // TODO: implement inline edit
    };

    let sorted = move || registry.with(|r| r.sorted_servers());
    let active_id = move || registry.with(|r| r.active_server_id.clone());

    view! {
        <div class="p-6 max-w-4xl mx-auto">
            <div class="flex items-center justify-between mb-6">
                <div>
                    <h1 class="text-2xl font-bold text-gray-900 dark:text-white">"Server Connections"</h1>
                    <p class="mt-1 text-sm text-gray-500 dark:text-gray-400">
                        "Manage your Tachyon server connections. Each server has its own documents and authentication."
                    </p>
                </div>
                <button
                    type="button"
                    on:click=move |_| show_add_form.set(!show_add_form.get())
                    class="px-4 py-2 bg-blue-600 hover:bg-blue-700 text-white text-sm font-medium transition-colors"
                >
                    "Add Server"
                </button>
            </div>

            // Error display
            <Show when=move || error.get().is_some()>
                <div role="alert" class="mb-4 p-3 bg-red-100 dark:bg-red-900 border border-red-400 dark:border-red-700 text-red-700 dark:text-red-200 rounded">
                    {move || error.get().unwrap_or_default()}
                </div>
            </Show>

            // Add server form
            <Show when=move || show_add_form.get()>
                <div class="mb-6 p-4 border border-gray-200 dark:border-gray-700 bg-gray-50 dark:bg-gray-800">
                    <h3 class="text-lg font-medium text-gray-900 dark:text-white mb-3">"Add New Server"</h3>
                    <form on:submit=on_add_server class="space-y-3">
                        <div>
                            <label for="server-name" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                                "Server Name"
                            </label>
                            <input
                                id="server-name"
                                type="text"
                                class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-700 text-gray-900 dark:text-white text-sm"
                                placeholder="My Tachyon Server"
                                on:input=move |ev| new_name.set(event_target_value(&ev))
                                prop:value=move || new_name.get()
                            />
                        </div>
                        <div>
                            <label for="server-url" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                                "Server URL"
                            </label>
                            <input
                                id="server-url"
                                type="text"
                                class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-700 text-gray-900 dark:text-white text-sm"
                                placeholder="http://localhost:8080"
                                on:input=move |ev| new_url.set(event_target_value(&ev))
                                prop:value=move || new_url.get()
                            />
                        </div>
                        <div class="flex gap-2">
                            <button
                                type="submit"
                                class="px-4 py-2 bg-blue-600 hover:bg-blue-700 text-white text-sm font-medium transition-colors"
                            >
                                "Save"
                            </button>
                            <button
                                type="button"
                                on:click=move |_| show_add_form.set(false)
                                class="px-4 py-2 bg-gray-200 hover:bg-gray-300 dark:bg-gray-700 dark:hover:bg-gray-600 text-gray-700 dark:text-gray-300 text-sm font-medium transition-colors"
                            >
                                "Cancel"
                            </button>
                        </div>
                    </form>
                </div>
            </Show>

            // Server list
            <Show
                when=move || sorted().is_empty()
                fallback=move || view! {
                    <div class="space-y-3">
                        {sorted().into_iter().map(|server| {
                            let _sid = server.id.clone();
                            let sid_active = server.id.clone();
                            let sid_remove = server.id.clone();
                            let is_active = active_id().as_deref() == Some(&server.id);
                            let has_token = server.auth_token.is_some();
                            let server_name = server.name.clone();
                            let server_base = server.base_url.clone();
                            let last_conn_check = server.last_connected.clone();
                            let last_conn_val = server.last_connected.clone();

                            view! {
                                <div
                                    class=format!(
                                        "p-4 border {} transition-colors",
                                        if is_active { "border-blue-500 dark:border-blue-400 bg-blue-50 dark:bg-blue-900/20" } else { "border-gray-200 dark:border-gray-700" }
                                    )
                                >
                                    <div class="flex items-start justify-between">
                                        <div class="flex-1">
                                            <div class="flex items-center gap-2">
                                                <h3 class="font-medium text-gray-900 dark:text-white">
                                                    {server_name}
                                                </h3>
                                                <Show when=move || is_active>
                                                    <span class="px-2 py-0.5 text-xs bg-blue-100 dark:bg-blue-900 text-blue-700 dark:text-blue-300 font-medium">
                                                        "Active"
                                                    </span>
                                                </Show>
                                                <Show when=move || has_token>
                                                    <span class="px-2 py-0.5 text-xs bg-green-100 dark:bg-green-900 text-green-700 dark:text-green-300">
                                                        "Connected"
                                                    </span>
                                                </Show>
                                            </div>
                                            <p class="text-sm text-gray-500 dark:text-gray-400 mt-1">
                                                {server_base}
                                            </p>
                                            <Show when=move || last_conn_check.is_some()>
                                                <p class="text-xs text-gray-400 dark:text-gray-500 mt-1">
                                                    "Last connected: "
                                                    {last_conn_val.clone().unwrap_or_default()}
                                                </p>
                                            </Show>
                                        </div>
                                        <div class="flex gap-2 ml-4">
                                            <button
                                                type="button"
                                                on:click=move |_| on_connect(sid_active.clone())
                                                disabled=move || connecting.get()
                                                class=move || format!(
                                                    "px-3 py-1.5 text-sm font-medium transition-colors {}",
                                                    if is_active { "bg-gray-200 dark:bg-gray-700 text-gray-500 dark:text-gray-400 cursor-default" } else { "bg-blue-600 hover:bg-blue-700 text-white" }
                                                )
                                            >
                                                {move || if is_active { "Current" } else { "Connect" }}
                                            </button>
                                            <button
                                                type="button"
                                                on:click=move |_| on_remove(sid_remove.clone())
                                                class="px-3 py-1.5 text-sm font-medium text-red-600 hover:text-red-700 dark:text-red-400 transition-colors"
                                            >
                                                "Remove"
                                            </button>
                                        </div>
                                    </div>
                                </div>
                            }
                        }).collect::<Vec<_>>()}
                    </div>
                }
            >
                <div class="text-center py-16">
                    <svg class="w-16 h-16 mx-auto text-gray-300 dark:text-gray-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M5 12h14M5 12a2 2 0 01-2-2V6a2 2 0 012-2h14a2 2 0 012 2v4a2 2 0 01-2 2M5 12a2 2 0 00-2 2v4a2 2 0 002 2h14a2 2 0 002-2v-4a2 2 0 00-2-2"></path>
                    </svg>
                    <h3 class="mt-4 text-lg font-medium text-gray-900 dark:text-white">"No servers configured"</h3>
                    <p class="mt-2 text-sm text-gray-500 dark:text-gray-400">
                        "Add a server connection to start syncing documents."
                    </p>
                </div>
            </Show>
        </div>
    }
}
