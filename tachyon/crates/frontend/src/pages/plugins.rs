// Plugins Page
// Plugin management - browse, install, enable/disable, uninstall

use crate::api::ApiClient;
use crate::types::{CreatePluginRequest, Plugin, UpdatePluginRequest};
use leptos::prelude::*;
use wasm_bindgen_futures::spawn_local;

// ---------------------------------------------------------------------------
// Plugins Page
// ---------------------------------------------------------------------------

#[component]
pub fn PluginsPage() -> impl IntoView {
    let api_client = ApiClient::default();
    let api_client_for_plugins = api_client.clone();

    let (show_install_modal, set_show_install_modal) = signal(false);
    let (editing_plugin, set_editing_plugin) = signal(None::<Plugin>);
    let (deleting_id, set_deleting_id) = signal(None::<String>);
    let (show_enabled_only, set_show_enabled_only) = signal(false);
    let (refresh_counter, set_refresh_counter) = signal(0u32);

    // Fetch plugins
    let plugins_resource = LocalResource::new(move || {
        let client = api_client_for_plugins.clone();
        let enabled_only = show_enabled_only.get();
        let _ = refresh_counter.get();
        async move {
            client
                .list_plugins(if enabled_only { Some(true) } else { None })
                .await
                .unwrap_or_default()
        }
    });

    // Plugins grid view (extracted before view!)
    let plugins_view = move || {
        plugins_resource.get().map(|plugins| {
            let total = plugins.len();
            let enabled_count = plugins.iter().filter(|p| p.enabled).count();
            if total == 0 {
                view! {
                    <div class="text-center py-16">
                        <svg class="w-16 h-16 mx-auto text-gray-300 dark:text-gray-600 mb-4"
                             fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5"
                                  d="M11 4a2 2 0 114 0v1a1 1 0 001 1h3a1 1 0 011 1v3a1 1 0 01-1 1h-1a2 2 0 100 4h1a1 1 0 011 1v3a1 1 0 01-1 1h-3a1 1 0 01-1-1v-1a2 2 0 10-4 0v1a1 1 0 01-1 1H7a1 1 0 01-1-1v-3a1 1 0 00-1-1H4a2 2 0 110-4h1a1 1 0 001-1V7a1 1 0 011-1h3a1 1 0 001-1V4z" />
                        </svg>
                        <h3 class="text-lg font-medium text-gray-900 dark:text-white mb-1">
                            "No plugins installed"
                        </h3>
                        <p class="text-gray-500 dark:text-gray-400 mb-4">
                            "Install plugins to extend Tachyon's capabilities"
                        </p>
                        <button
                            class="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700"
                            on:click={move |_| set_show_install_modal.set(true)}
                        >
                            "Install Plugin"
                        </button>
                    </div>
                }.into_any()
            } else {
                view! {
                    <div>
                        <div class="flex items-center gap-4 mb-4">
                            <p class="text-sm text-gray-500 dark:text-gray-400">
                                {format!("{} plugin{} installed, {} enabled", total, if total == 1 { "" } else { "s" }, enabled_count)}
                            </p>
                            <button
                                class={move || format!(
                                    "text-sm px-3 py-1 rounded-full transition-colors {}",
                                    if show_enabled_only.get() {
                                        "bg-blue-100 dark:bg-blue-900/50 text-blue-700 dark:text-blue-300"
                                    } else {
                                        "bg-gray-100 dark:bg-gray-700 text-gray-600 dark:text-gray-300 hover:bg-gray-200 dark:hover:bg-gray-600"
                                    }
                                )}
                                on:click={move |_| set_show_enabled_only.update(|v| { *v = !*v; })}
                            >
                                {move || if show_enabled_only.get() { "Showing enabled" } else { "Show all" } }
                            </button>
                        </div>
                        <div class="space-y-3">
                            {plugins.into_iter().map(|plugin| {
                                let t_edit = plugin.clone();
                                let t_delete_id = plugin.id.clone();
                                let t_toggle_id = plugin.id.clone();
                                let t_enabled = plugin.enabled;
                                view! {
                                    <PluginRow
                                        plugin=plugin
                                        on_edit={Callback::new(move |_| set_editing_plugin.set(Some(t_edit.clone())))}
                                        on_delete={Callback::new(move |_| set_deleting_id.set(Some(t_delete_id.clone())))}
                                        on_toggle={Callback::new(move |_| {
                                            let api = ApiClient::default();
                                            let id = t_toggle_id.clone();
                                            let new_enabled = !t_enabled;
                                            spawn_local(async move {
                                                if api.update_plugin(&id, &UpdatePluginRequest {
                                                    enabled: Some(new_enabled),
                                                    ..Default::default()
                                                }).await.is_ok() { set_refresh_counter.update(|n| *n += 1) }
                                            });
                                        })}
                                    />
                                }
                            }).collect::<Vec<_>>()}
                        </div>
                    </div>
                }.into_any()
            }
        })
    };

    // Skeleton
    let skeleton = view! {
        <div class="space-y-3">
            <PluginRowSkeleton />
            <PluginRowSkeleton />
            <PluginRowSkeleton />
        </div>
    };

    // Install modal
    let install_modal_view = move || {
        show_install_modal.get().then(|| {
            let save_cb = Callback::new(move |_| {
                set_refresh_counter.update(|n| *n += 1);
                set_show_install_modal.set(false);
            });
            let cancel_cb = Callback::new(move |_| set_show_install_modal.set(false));
            view! {
                <InstallPluginModal on_save={save_cb} on_cancel={cancel_cb} />
            }
        })
    };

    // Edit modal
    let edit_modal_view = move || {
        editing_plugin.get().map(|plugin| {
            let save_cb = Callback::new(move |_| {
                set_refresh_counter.update(|n| *n += 1);
                set_editing_plugin.set(None);
            });
            let cancel_cb = Callback::new(move |_| set_editing_plugin.set(None));
            view! {
                <EditPluginModal plugin=plugin on_save={save_cb} on_cancel={cancel_cb} />
            }
        })
    };

    // Delete modal
    let delete_modal_view = move || {
        deleting_id.get().map(|id| {
            let confirm_cb = Callback::new(move |_| {
                set_refresh_counter.update(|n| *n += 1);
                set_deleting_id.set(None);
            });
            let cancel_cb = Callback::new(move |_| set_deleting_id.set(None));
            view! {
                <DeletePluginModal id=id on_confirm={confirm_cb} on_cancel={cancel_cb} />
            }
        })
    };

    view! {
        <div>
            // Header
            <div class="flex items-center justify-between mb-6">
                <div>
                    <h1 class="text-2xl font-bold text-gray-900 dark:text-white">"Plugins"</h1>
                    <p class="mt-1 text-sm text-gray-500 dark:text-gray-400">
                        "Extend Tachyon with community and custom plugins"
                    </p>
                </div>
                <button
                    class="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors
                           flex items-center gap-2"
                    on:click={move |_| set_show_install_modal.set(true)}
                >
                    <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                              d="M12 4v16m8-8H4" />
                    </svg>
                    "Install Plugin"
                </button>
            </div>

            <Suspense fallback={skeleton}>
                {plugins_view}
            </Suspense>

            {install_modal_view}
            {edit_modal_view}
            {delete_modal_view}
        </div>
    }
}

// ---------------------------------------------------------------------------
// Plugin Row
// ---------------------------------------------------------------------------

#[component]
fn PluginRow(
    plugin: Plugin,
    on_edit: Callback<()>,
    on_delete: Callback<()>,
    on_toggle: Callback<()>,
) -> impl IntoView {
    let desc = plugin.description.clone().unwrap_or_default();
    let installed = plugin
        .installed_at
        .split('T')
        .next()
        .unwrap_or("")
        .to_string();

    let runtime_badge_class = match plugin.runtime_type.as_str() {
        "builtin" => "bg-purple-100 dark:bg-purple-900/50 text-purple-700 dark:text-purple-300",
        "wasm" => "bg-green-100 dark:bg-green-900/50 text-green-700 dark:text-green-300",
        _ => "bg-gray-100 dark:bg-gray-700 text-gray-600 dark:text-gray-300",
    };

    let desc_view = if !desc.is_empty() {
        let d = desc.clone();
        view! {
            <p class="text-sm text-gray-500 dark:text-gray-400 mt-1 line-clamp-1">{d}</p>
        }
        .into_any()
    } else {
        view! { <div class="mt-1"></div> }.into_any()
    };

    let tags_view = if !plugin.extension_points.is_empty() {
        let tags = plugin.extension_points.clone();
        view! {
            <div class="flex flex-wrap gap-1 mt-2">
                {tags.into_iter().take(4).map(|tag| {
                    view! {
                        <span class="px-2 py-0.5 text-xs bg-gray-100 dark:bg-gray-700
                                     text-gray-600 dark:text-gray-300 rounded">{tag}</span>
                    }
                }).collect::<Vec<_>>()}
            </div>
        }
        .into_any()
    } else {
        view! { <div></div> }.into_any()
    };

    view! {
        <div class={move || format!(
            "bg-white dark:bg-gray-800 rounded-lg border p-4 transition-colors {}",
            if plugin.enabled {
                "border-gray-200 dark:border-gray-700"
            } else {
                "border-gray-200 dark:border-gray-700 opacity-60"
            }
        )}>
            <div class="flex items-start justify-between gap-4">
                <div class="flex-1 min-w-0">
                    <div class="flex items-center gap-2 flex-wrap">
                        <h3 class="font-semibold text-gray-900 dark:text-white truncate">
                            {plugin.name.clone()}
                        </h3>
                        <span class="text-xs text-gray-400 font-mono">{format!("v{}", plugin.version)}</span>
                        <span class={runtime_badge_class.to_string()}>
                            {plugin.runtime_type.clone()}
                        </span>
                    </div>
                    {desc_view}
                    {tags_view}
                    <div class="flex items-center gap-4 mt-2 text-xs text-gray-400">
                        {if let Some(author) = &plugin.author {
                            let a = author.clone();
                            view! { <span>{a}</span> }.into_any()
                        } else {
                            view! { <div></div> }.into_any()
                        }}
                        <span>{format!("Installed {}", installed)}</span>
                    </div>
                </div>

                <div class="flex items-center gap-2 flex-shrink-0">
                    <button
                        class={move || format!(
                            "relative inline-flex h-6 w-11 items-center rounded-full transition-colors {}",
                            if plugin.enabled {
                                "bg-blue-600"
                            } else {
                                "bg-gray-300 dark:bg-gray-600"
                            }
                        )}
                        on:click={move |_| on_toggle.run(())}
                    >
                        <span class={move || format!(
                            "inline-block h-4 w-4 transform rounded-full bg-white transition-transform {}",
                            if plugin.enabled { "translate-x-6" } else { "translate-x-1" }
                        )}></span>
                    </button>
                    <button
                        class="p-1.5 text-gray-400 hover:text-gray-600 dark:hover:text-gray-200
                               hover:bg-gray-100 dark:hover:bg-gray-700 rounded transition-colors"
                        on:click={move |_| on_edit.run(())}
                    >
                        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                                  d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z" />
                        </svg>
                    </button>
                    <button
                        class="p-1.5 text-red-400 hover:text-red-600 hover:bg-red-50
                               dark:hover:bg-red-900/30 rounded transition-colors"
                        on:click={move |_| on_delete.run(())}
                    >
                        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                                  d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
                        </svg>
                    </button>
                </div>
            </div>
        </div>
    }
}

// ---------------------------------------------------------------------------
// Install Plugin Modal
// ---------------------------------------------------------------------------

#[component]
fn InstallPluginModal(on_save: Callback<()>, on_cancel: Callback<()>) -> impl IntoView {
    let (name, set_name) = signal(String::new());
    let (version, set_version) = signal(String::new());
    let (description, set_description) = signal(String::new());
    let (author, set_author) = signal(String::new());
    let (homepage, set_homepage) = signal(String::new());
    let (license, set_license) = signal(String::new());
    let (ext_points, set_ext_points) = signal(String::new());
    let (runtime_type, set_runtime_type) = signal("wasm".to_string());
    let (error, set_error) = signal(None::<String>);
    let (submitting, set_submitting) = signal(false);

    let handle_submit = move |_| {
        let n = name.get();
        let v = version.get();
        if n.trim().is_empty() {
            set_error.set(Some("Plugin name is required".to_string()));
            return;
        }
        if v.trim().is_empty() {
            set_error.set(Some("Version is required".to_string()));
            return;
        }
        set_error.set(None);
        set_submitting.set(true);

        let api = ApiClient::default();
        let ep_str = ext_points.get();
        let parsed_eps: Vec<String> = ep_str
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        let desc_val = description.get();
        let auth_val = author.get();
        let home_val = homepage.get();
        let lic_val = license.get();
        let rt_val = runtime_type.get();
        let save_cb = on_save;

        spawn_local(async move {
            let result = api
                .create_plugin(&CreatePluginRequest {
                    name: n,
                    description: if desc_val.is_empty() {
                        None
                    } else {
                        Some(desc_val)
                    },
                    version: v,
                    author: if auth_val.is_empty() {
                        None
                    } else {
                        Some(auth_val)
                    },
                    homepage: if home_val.is_empty() {
                        None
                    } else {
                        Some(home_val)
                    },
                    license: if lic_val.is_empty() {
                        None
                    } else {
                        Some(lic_val)
                    },
                    extension_points: if parsed_eps.is_empty() {
                        None
                    } else {
                        Some(parsed_eps)
                    },
                    manifest: None,
                    runtime_type: Some(rt_val),
                    entry_point: None,
                    enabled: Some(false),
                })
                .await;
            set_submitting.set(false);
            match result {
                Ok(_) => save_cb.run(()),
                Err(e) => set_error.set(Some(format!("Failed: {}", e))),
            }
        });
    };

    let error_view = move || {
        error.get().map(|e| {
            view! {
                <div class="p-3 bg-red-50 dark:bg-red-900/30 border border-red-200 dark:border-red-800 
                            text-red-700 dark:text-red-300 text-sm rounded-lg">{e}</div>
            }
        })
    };

    let btn_label = move || {
        if submitting.get() {
            "Installing..."
        } else {
            "Install"
        }
    };
    let btn_class = move || {
        format!(
            "px-4 py-2 text-sm text-white rounded-lg {}",
            if submitting.get() {
                "bg-blue-400 cursor-not-allowed"
            } else {
                "bg-blue-600 hover:bg-blue-700"
            }
        )
    };

    view! {
        <div class="fixed inset-0 bg-black/50 flex items-center justify-center z-50"
             on:click={move |_| on_cancel.run(())}>
            <div class="bg-white dark:bg-gray-800 rounded-xl shadow-2xl w-full max-w-lg max-h-[90vh] overflow-hidden"
                 on:click={move |ev| ev.stop_propagation()}>
                <div class="flex items-center justify-between px-6 py-4 border-b border-gray-200 dark:border-gray-700">
                    <h2 class="text-lg font-semibold text-gray-900 dark:text-white">"Install Plugin"</h2>
                    <button class="p-1 text-gray-400 hover:text-gray-600 dark:hover:text-gray-200 rounded-lg
                               hover:bg-gray-100 dark:hover:bg-gray-700"
                            on:click={move |_| on_cancel.run(())}>
                        <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
                        </svg>
                    </button>
                </div>

                <div class="px-6 py-4 space-y-4 overflow-y-auto max-h-[60vh]">
                    {error_view}

                    <div class="grid grid-cols-2 gap-4">
                        <div>
                            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                                "Name" <span class="text-red-500">"*"</span>
                            </label>
                            <input type="text"
                                class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg
                                       bg-white dark:bg-gray-700 text-gray-900 dark:text-white 
                                       focus:ring-2 focus:ring-blue-500 outline-none"
                                placeholder="my-plugin" prop:value={name.get()}
                                on:input={move |ev| set_name.set(event_target_value(&ev))} />
                        </div>
                        <div>
                            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                                "Version" <span class="text-red-500">"*"</span>
                            </label>
                            <input type="text"
                                class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg
                                       bg-white dark:bg-gray-700 text-gray-900 dark:text-white 
                                       focus:ring-2 focus:ring-blue-500 outline-none"
                                placeholder="0.1.0" prop:value={version.get()}
                                on:input={move |ev| set_version.set(event_target_value(&ev))} />
                        </div>
                    </div>

                    <div>
                        <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">"Description"</label>
                        <input type="text"
                            class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg
                                   bg-white dark:bg-gray-700 text-gray-900 dark:text-white 
                                   focus:ring-2 focus:ring-blue-500 outline-none"
                            placeholder="What does this plugin do?" prop:value={description.get()}
                            on:input={move |ev| set_description.set(event_target_value(&ev))} />
                    </div>

                    <div class="grid grid-cols-2 gap-4">
                        <div>
                            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">"Author"</label>
                            <input type="text"
                                class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg
                                       bg-white dark:bg-gray-700 text-gray-900 dark:text-white 
                                       focus:ring-2 focus:ring-blue-500 outline-none"
                                placeholder="Author name" prop:value={author.get()}
                                on:input={move |ev| set_author.set(event_target_value(&ev))} />
                        </div>
                        <div>
                            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">"License"</label>
                            <input type="text"
                                class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg
                                       bg-white dark:bg-gray-700 text-gray-900 dark:text-white 
                                       focus:ring-2 focus:ring-blue-500 outline-none"
                                placeholder="MIT" prop:value={license.get()}
                                on:input={move |ev| set_license.set(event_target_value(&ev))} />
                        </div>
                    </div>

                    <div>
                        <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">"Homepage"</label>
                        <input type="text"
                            class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg
                                   bg-white dark:bg-gray-700 text-gray-900 dark:text-white 
                                   focus:ring-2 focus:ring-blue-500 outline-none"
                            placeholder="https://github.com/user/plugin" prop:value={homepage.get()}
                            on:input={move |ev| set_homepage.set(event_target_value(&ev))} />
                    </div>

                    <div>
                        <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">"Runtime"</label>
                        <select
                            class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg
                                   bg-white dark:bg-gray-700 text-gray-900 dark:text-white 
                                   focus:ring-2 focus:ring-blue-500 outline-none"
                            prop:value={runtime_type.get()}
                            on:change={move |ev| set_runtime_type.set(event_target_value(&ev))}>
                            <option value="wasm">"WASM (WebAssembly)"</option>
                            <option value="native">"Native (Rust dylib)"</option>
                            <option value="builtin">"Built-in"</option>
                        </select>
                    </div>

                    <div>
                        <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">"Extension Points"</label>
                        <input type="text"
                            class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg
                                   bg-white dark:bg-gray-700 text-gray-900 dark:text-white 
                                   focus:ring-2 focus:ring-blue-500 outline-none"
                            placeholder="Comma-separated: editor:command, document:on-save"
                            prop:value={ext_points.get()}
                            on:input={move |ev| set_ext_points.set(event_target_value(&ev))} />
                        <p class="mt-1 text-xs text-gray-400">
                            "Hooks this plugin registers. E.g. editor:command, sidebar:panel, document:on-save"
                        </p>
                    </div>
                </div>

                <div class="flex items-center justify-end gap-3 px-6 py-4 border-t border-gray-200 dark:border-gray-700">
                    <button class="px-4 py-2 text-sm text-gray-700 dark:text-gray-300 border border-gray-300
                                   dark:border-gray-600 rounded-lg hover:bg-gray-50 dark:hover:bg-gray-700"
                        on:click={move |_| on_cancel.run(())}>"Cancel"</button>
                    <button class={btn_class} disabled={submitting.get()} on:click={handle_submit}>{btn_label}</button>
                </div>
            </div>
        </div>
    }
}

// ---------------------------------------------------------------------------
// Edit Plugin Modal
// ---------------------------------------------------------------------------

#[component]
fn EditPluginModal(
    plugin: Plugin,
    on_save: Callback<()>,
    on_cancel: Callback<()>,
) -> impl IntoView {
    let (description, set_description) = signal(plugin.description.clone().unwrap_or_default());
    let (version, set_version) = signal(plugin.version.clone());
    let (author, set_author) = signal(plugin.author.clone().unwrap_or_default());
    let (homepage, _set_homepage) = signal(plugin.homepage.clone().unwrap_or_default());
    let (license, set_license) = signal(plugin.license.clone().unwrap_or_default());
    let (ext_points, set_ext_points) = signal(plugin.extension_points.join(", "));
    let (error, set_error) = signal(None::<String>);
    let (submitting, set_submitting) = signal(false);
    let plugin_id = plugin.id.clone();

    let handle_submit = move |_| {
        set_error.set(None);
        set_submitting.set(true);
        let api = ApiClient::default();
        let ep_str = ext_points.get();
        let parsed_eps: Vec<String> = ep_str
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        let desc_val = description.get();
        let ver_val = version.get();
        let auth_val = author.get();
        let home_val = homepage.get();
        let lic_val = license.get();
        let tid = plugin_id.clone();
        let save_cb = on_save;

        spawn_local(async move {
            let result = api
                .update_plugin(
                    &tid,
                    &UpdatePluginRequest {
                        description: if desc_val.is_empty() {
                            None
                        } else {
                            Some(desc_val)
                        },
                        version: Some(ver_val),
                        author: if auth_val.is_empty() {
                            None
                        } else {
                            Some(auth_val)
                        },
                        homepage: if home_val.is_empty() {
                            None
                        } else {
                            Some(home_val)
                        },
                        license: if lic_val.is_empty() {
                            None
                        } else {
                            Some(lic_val)
                        },
                        extension_points: if parsed_eps.is_empty() {
                            None
                        } else {
                            Some(parsed_eps)
                        },
                        ..Default::default()
                    },
                )
                .await;
            set_submitting.set(false);
            match result {
                Ok(_) => save_cb.run(()),
                Err(e) => set_error.set(Some(format!("Failed: {}", e))),
            }
        });
    };

    let error_view = move || {
        error.get().map(|e| {
            view! {
                <div class="p-3 bg-red-50 dark:bg-red-900/30 border border-red-200 dark:border-red-800 
                            text-red-700 dark:text-red-300 text-sm rounded-lg">{e}</div>
            }
        })
    };

    let btn_label = move || {
        if submitting.get() {
            "Saving..."
        } else {
            "Save"
        }
    };
    let btn_class = move || {
        format!(
            "px-4 py-2 text-sm text-white rounded-lg {}",
            if submitting.get() {
                "bg-blue-400 cursor-not-allowed"
            } else {
                "bg-blue-600 hover:bg-blue-700"
            }
        )
    };

    view! {
        <div class="fixed inset-0 bg-black/50 flex items-center justify-center z-50"
             on:click={move |_| on_cancel.run(())}>
            <div class="bg-white dark:bg-gray-800 rounded-xl shadow-2xl w-full max-w-lg max-h-[90vh] overflow-hidden"
                 on:click={move |ev| ev.stop_propagation()}>
                <div class="flex items-center justify-between px-6 py-4 border-b border-gray-200 dark:border-gray-700">
                    <div>
                        <h2 class="text-lg font-semibold text-gray-900 dark:text-white">
                            {format!("Edit {}", plugin.name)}
                        </h2>
                        <p class="text-xs text-gray-400 font-mono">{format!("v{}", plugin.version)}</p>
                    </div>
                    <button class="p-1 text-gray-400 hover:text-gray-600 dark:hover:text-gray-200 rounded-lg
                               hover:bg-gray-100 dark:hover:bg-gray-700"
                        on:click={move |_| on_cancel.run(())}>
                        <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
                        </svg>
                    </button>
                </div>
                <div class="px-6 py-4 space-y-4 overflow-y-auto max-h-[60vh]">
                    {error_view}
                    <div>
                        <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">"Version"</label>
                        <input type="text"
                            class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg
                                   bg-white dark:bg-gray-700 text-gray-900 dark:text-white 
                                   focus:ring-2 focus:ring-blue-500 outline-none"
                            prop:value={version.get()}
                            on:input={move |ev| set_version.set(event_target_value(&ev))} />
                    </div>
                    <div>
                        <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">"Description"</label>
                        <input type="text"
                            class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg
                                   bg-white dark:bg-gray-700 text-gray-900 dark:text-white 
                                   focus:ring-2 focus:ring-blue-500 outline-none"
                            prop:value={description.get()}
                            on:input={move |ev| set_description.set(event_target_value(&ev))} />
                    </div>
                    <div class="grid grid-cols-2 gap-4">
                        <div>
                            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">"Author"</label>
                            <input type="text"
                                class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg
                                       bg-white dark:bg-gray-700 text-gray-900 dark:text-white 
                                       focus:ring-2 focus:ring-blue-500 outline-none"
                                prop:value={author.get()}
                                on:input={move |ev| set_author.set(event_target_value(&ev))} />
                        </div>
                        <div>
                            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">"License"</label>
                            <input type="text"
                                class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg
                                       bg-white dark:bg-gray-700 text-gray-900 dark:text-white 
                                       focus:ring-2 focus:ring-blue-500 outline-none"
                                prop:value={license.get()}
                                on:input={move |ev| set_license.set(event_target_value(&ev))} />
                        </div>
                    </div>
                    <div>
                        <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">"Extension Points"</label>
                        <input type="text"
                            class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg
                                   bg-white dark:bg-gray-700 text-gray-900 dark:text-white 
                                   focus:ring-2 focus:ring-blue-500 outline-none"
                            placeholder="Comma-separated" prop:value={ext_points.get()}
                            on:input={move |ev| set_ext_points.set(event_target_value(&ev))} />
                    </div>
                </div>
                <div class="flex items-center justify-end gap-3 px-6 py-4 border-t border-gray-200 dark:border-gray-700">
                    <button class="px-4 py-2 text-sm text-gray-700 dark:text-gray-300 border border-gray-300
                                   dark:border-gray-600 rounded-lg hover:bg-gray-50 dark:hover:bg-gray-700"
                        on:click={move |_| on_cancel.run(())}>"Cancel"</button>
                    <button class={btn_class} disabled={submitting.get()} on:click={handle_submit}>{btn_label}</button>
                </div>
            </div>
        </div>
    }
}

// ---------------------------------------------------------------------------
// Delete Plugin Modal
// ---------------------------------------------------------------------------

#[component]
fn DeletePluginModal(
    id: String,
    on_confirm: Callback<()>,
    on_cancel: Callback<()>,
) -> impl IntoView {
    let (error, set_error) = signal(None::<String>);
    let (submitting, set_submitting) = signal(false);

    let handle_delete = move |_| {
        set_error.set(None);
        set_submitting.set(true);
        let api = ApiClient::default();
        let tid = id.clone();
        let confirm_cb = on_confirm;
        spawn_local(async move {
            match api.delete_plugin(&tid).await {
                Ok(_) => confirm_cb.run(()),
                Err(e) => {
                    set_error.set(Some(format!("Failed: {}", e)));
                    set_submitting.set(false);
                }
            }
        });
    };

    let error_view = move || {
        error.get().map(|e| {
            view! {
                <div class="mb-4 p-3 bg-red-50 dark:bg-red-900/30 border border-red-200 
                            dark:border-red-800 text-red-700 dark:text-red-300 text-sm rounded-lg">{e}</div>
            }
        })
    };

    let btn_class = move || {
        format!(
            "px-4 py-2 text-sm text-white rounded-lg {}",
            if submitting.get() {
                "bg-red-400 cursor-not-allowed"
            } else {
                "bg-red-600 hover:bg-red-700"
            }
        )
    };

    view! {
        <div class="fixed inset-0 bg-black/50 flex items-center justify-center z-50"
             on:click={move |_| on_cancel.run(())}>
            <div class="bg-white dark:bg-gray-800 rounded-xl shadow-2xl w-full max-w-md"
                 on:click={move |ev| ev.stop_propagation()}>
                <div class="p-6">
                    <div class="flex items-center gap-3 mb-4">
                        <div class="flex-shrink-0 w-10 h-10 bg-red-100 dark:bg-red-900/30 rounded-full
                                    flex items-center justify-center">
                            <svg class="w-5 h-5 text-red-600 dark:text-red-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                                      d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-2.5L13.732 4c-.77-.833-1.964-.833-2.732 0L3.34 16.5c-.77.833.192 2.5 1.732 2.5z" />
                            </svg>
                        </div>
                        <div>
                            <h3 class="text-lg font-semibold text-gray-900 dark:text-white">"Uninstall Plugin"</h3>
                            <p class="text-sm text-gray-500 dark:text-gray-400 mt-1">
                                "This will remove the plugin from Tachyon."
                            </p>
                        </div>
                    </div>
                    {error_view}
                    <div class="flex justify-end gap-3 mt-6">
                        <button class="px-4 py-2 text-sm text-gray-700 dark:text-gray-300 border border-gray-300
                                       dark:border-gray-600 rounded-lg hover:bg-gray-50 dark:hover:bg-gray-700"
                            on:click={move |_| on_cancel.run(())}>"Cancel"</button>
                        <button class={btn_class} disabled={submitting.get()} on:click={handle_delete}>
                            {move || if submitting.get() { "Uninstalling..." } else { "Uninstall" } }
                        </button>
                    </div>
                </div>
            </div>
        </div>
    }
}

// ---------------------------------------------------------------------------
// Skeleton
// ---------------------------------------------------------------------------

#[component]
fn PluginRowSkeleton() -> impl IntoView {
    view! {
        <div class="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 p-4 animate-pulse">
            <div class="flex items-center gap-3">
                <div class="h-5 bg-gray-200 dark:bg-gray-700 rounded w-1/4"></div>
                <div class="h-4 bg-gray-200 dark:bg-gray-700 rounded w-16"></div>
                <div class="h-4 bg-gray-200 dark:bg-gray-700 rounded w-14"></div>
            </div>
            <div class="h-3 bg-gray-200 dark:bg-gray-700 rounded w-2/3 mt-3"></div>
        </div>
    }
}
