use crate::api::ApiClient;
use crate::types::WebhookInfo;
use leptos::prelude::*;
use wasm_bindgen_futures::spawn_local;

const AVAILABLE_EVENTS: &[&str] = &[
    "document_created",
    "document_updated",
    "document_deleted",
    "document_published",
    "team_member_added",
    "team_member_removed",
    "space_created",
    "space_updated",
];

#[component]
pub fn WebhooksPage() -> impl IntoView {
    let api_client = ApiClient::default();
    let api_client_for_list = api_client.clone();

    let (show_create_modal, set_show_create_modal) = signal(false);
    let (deleting_id, set_deleting_id) = signal(None::<String>);
    let (_testing_id, set_testing_id) = signal(None::<String>);
    let (test_result, set_test_result) = signal(None::<(String, bool)>);
    let (refresh_counter, set_refresh_counter) = signal(0u32);

    let webhooks_resource = LocalResource::new(move || {
        let client = api_client_for_list.clone();
        let _ = refresh_counter.get();
        async move { client.list_webhooks().await.unwrap_or_default() }
    });

    let webhooks_view = move || {
        webhooks_resource.get().map(|webhooks| {
            if webhooks.is_empty() {
                view! {
                    <div class="text-center py-16">
                        <svg class="w-16 h-16 mx-auto text-gray-300 dark:text-gray-600 mb-4"
                             fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5"
                                  d="M13.828 10.172a4 4 0 00-5.656 0l-4 4a4 4 0 105.656 5.656l1.102-1.101m-.758-4.899a4 4 0 005.656 0l4-4a4 4 0 00-5.656-5.656l-1.1 1.1" />
                        </svg>
                        <h3 class="text-lg font-medium text-gray-900 dark:text-white mb-1">
                            "No webhooks configured"
                        </h3>
                        <p class="text-gray-500 dark:text-gray-400 mb-4">
                            "Create webhooks to receive notifications when events occur in Tachyon"
                        </p>
                        <button
                            class="px-4 py-2 bg-blue-600 text-white rounded-none hover:bg-blue-700"
                            on:click={move |_| set_show_create_modal.set(true)}
                        >
                            "Create Webhook"
                        </button>
                    </div>
                }.into_any()
            } else {
                let count = webhooks.len();
                let active_count = webhooks.iter().filter(|w| w.active).count();
                view! {
                    <div>
                        <div class="flex items-center gap-4 mb-4">
                            <p class="text-sm text-gray-500 dark:text-gray-400">
                                {format!("{} webhook{}, {} active", count, if count == 1 { "" } else { "s" }, active_count)}
                            </p>
                        </div>
                        <div class="space-y-3">
                            {webhooks.into_iter().map(|webhook| {
                                let w_id = webhook.id.clone();
                                let w_id2 = webhook.id.clone();
                                let w_id3 = webhook.id.clone();
                                let w_active = webhook.active;
                                view! {
                                    <WebhookRow
                                        webhook=webhook
                                        on_delete={Callback::new(move |_| set_deleting_id.set(Some(w_id.clone())))}
                                        on_test={Callback::new(move |_| {
                                            set_testing_id.set(Some(w_id2.clone()));
                                            set_test_result.set(None);
                                            let api = ApiClient::default();
                                            let id = w_id2.clone();
                                            spawn_local(async move {
                                                let result = api.test_webhook(&id).await;
                                                let success = result.is_ok();
                                                let msg = if success {
                                                    "Test payload sent successfully".to_string()
                                                } else {
                                                    format!("Test failed: {:?}", result.err())
                                                };
                                                set_test_result.set(Some((msg, success)));
                                                set_testing_id.set(None);
                                            });
                                        })}
                                        on_toggle_active={Callback::new(move |_| {
                                            let api = ApiClient::default();
                                            let id = w_id3.clone();
                                            let new_active = !w_active;
                                            spawn_local(async move {
                                                let body = serde_json::json!({
                                                    "url": "",
                                                    "events": [],
                                                    "active": new_active,
                                                });
                                                let _ = api.update_webhook(&id, &body).await;
                                                set_refresh_counter.update(|n| *n += 1);
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

    let skeleton = view! {
        <div class="space-y-3">
            {vec![0, 1, 2].into_iter().map(|_| {
                view! {
                    <div class="bg-white dark:bg-gray-800 rounded-none border-2 border-gray-900 dark:border-gray-100 p-4 animate-pulse">
                        <div class="flex items-center gap-3">
                            <div class="h-5 bg-gray-200 dark:bg-gray-700 rounded w-1/3"></div>
                            <div class="h-4 bg-gray-200 dark:bg-gray-700 rounded w-16"></div>
                        </div>
                        <div class="h-3 bg-gray-200 dark:bg-gray-700 rounded w-1/2 mt-3"></div>
                    </div>
                }
            }).collect::<Vec<_>>()}
        </div>
    };

    let create_modal_view = move || {
        show_create_modal.get().then(|| {
            let save_cb = Callback::new(move |_| {
                set_refresh_counter.update(|n| *n += 1);
                set_show_create_modal.set(false);
            });
            let cancel_cb = Callback::new(move |_| set_show_create_modal.set(false));
            view! {
                <CreateWebhookModal on_save={save_cb} on_cancel={cancel_cb} />
            }
        })
    };

    let delete_modal_view = move || {
        deleting_id.get().map(|id| {
            let confirm_cb = Callback::new(move |_| {
                set_refresh_counter.update(|n| *n += 1);
                set_deleting_id.set(None);
            });
            let cancel_cb = Callback::new(move |_| set_deleting_id.set(None));
            view! {
                <DeleteWebhookModal id=id on_confirm={confirm_cb} on_cancel={cancel_cb} />
            }
        })
    };

    let test_result_view = move || {
        test_result.get().map(|(msg, success)| {
            let class = if success {
                "p-3 bg-green-50 dark:bg-green-900/30 border border-green-200 dark:border-green-800 text-green-700 dark:text-green-300 text-sm rounded-none"
            } else {
                "p-3 bg-red-50 dark:bg-red-900/30 border border-red-200 dark:border-red-800 text-red-700 dark:text-red-300 text-sm rounded-none"
            };
            view! {
                <div class=class>{msg}</div>
            }
        })
    };

    view! {
        <div>
            <div class="flex items-center justify-between mb-6">
                <div>
                    <h1 class="text-2xl font-bold text-gray-900 dark:text-white">"Webhooks"</h1>
                    <p class="mt-1 text-sm text-gray-500 dark:text-gray-400">
                        "Manage outgoing webhook integrations"
                    </p>
                </div>
                <button
                    class="px-4 py-2 bg-blue-600 text-white rounded-none hover:bg-blue-700 transition-colors
                           flex items-center gap-2"
                    on:click={move |_| set_show_create_modal.set(true)}
                >
                    <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                              d="M12 4v16m8-8H4" />
                    </svg>
                    "Create Webhook"
                </button>
            </div>

            {test_result_view}

            <Suspense fallback={skeleton}>
                {webhooks_view}
            </Suspense>

            {create_modal_view}
            {delete_modal_view}
        </div>
    }
}

#[component]
fn WebhookRow(
    webhook: WebhookInfo,
    on_delete: Callback<()>,
    on_test: Callback<()>,
    on_toggle_active: Callback<()>,
) -> impl IntoView {
    let url = webhook.url.clone();
    let events = webhook.events.clone();
    let active = webhook.active;
    let last_triggered = webhook
        .last_triggered_at
        .as_ref()
        .and_then(|t| t.split('T').next())
        .unwrap_or("Never")
        .to_string();
    let created = webhook
        .created_at
        .split('T')
        .next()
        .unwrap_or("")
        .to_string();

    view! {
        <div class="bg-white dark:bg-gray-800 rounded-none border-2 border-gray-900 dark:border-gray-100 p-4 transition-colors">
            <div class="flex items-start justify-between gap-4">
                <div class="flex-1 min-w-0">
                    <div class="flex items-center gap-2 flex-wrap">
                        <span class="text-sm font-mono text-gray-900 dark:text-white truncate max-w-md">
                            {url}
                        </span>
                        <span class={if active {
                            "px-2 py-0.5 text-xs bg-green-100 dark:bg-green-900/50 text-green-700 dark:text-green-300 rounded"
                        } else {
                            "px-2 py-0.5 text-xs bg-gray-100 dark:bg-gray-700 text-gray-500 dark:text-gray-400 rounded"
                        }}>
                            {if active { "Active" } else { "Inactive" }}
                        </span>
                    </div>
                    <div class="flex flex-wrap gap-1 mt-2">
                        {events.into_iter().map(|ev| {
                            view! {
                                <span class="px-2 py-0.5 text-xs bg-blue-50 dark:bg-blue-900/30 text-blue-700 dark:text-blue-300 rounded">
                                    {ev}
                                </span>
                            }
                        }).collect::<Vec<_>>()}
                    </div>
                    <div class="flex items-center gap-4 mt-2 text-xs text-gray-400">
                        <span>{format!("Created {}", created)}</span>
                        <span>{format!("Last triggered {}", last_triggered)}</span>
                    </div>
                </div>

                <div class="flex items-center gap-2 flex-shrink-0">
                    <button
                        class={move || format!(
                            "relative inline-flex h-6 w-11 items-center rounded-full transition-colors {}",
                            if active {
                                "bg-blue-600"
                            } else {
                                "bg-gray-300 dark:bg-gray-600"
                            }
                        )}
                        aria-label=move || if active { "Disable webhook" } else { "Enable webhook" }
                        on:click={move |_| on_toggle_active.run(())}
                    >
                        <span class={move || format!(
                            "inline-block h-4 w-4 transform rounded-full bg-white transition-transform {}",
                            if active { "translate-x-6" } else { "translate-x-1" }
                        )}></span>
                    </button>
                    <button
                        class="p-1.5 text-gray-400 hover:text-blue-600 dark:hover:text-blue-400
                               hover:bg-blue-50 dark:hover:bg-blue-900/30 rounded transition-colors"
                        aria-label="Test webhook"
                        on:click={move |_| on_test.run(())}
                    >
                        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                                  d="M13 10V3L4 14h7v7l9-11h-7z" />
                        </svg>
                    </button>
                    <button
                        class="p-1.5 text-gray-400 hover:text-red-600 hover:bg-red-50
                               dark:hover:bg-red-900/30 rounded transition-colors"
                        aria-label="Delete webhook"
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

#[component]
fn CreateWebhookModal(on_save: Callback<()>, on_cancel: Callback<()>) -> impl IntoView {
    let (url, set_url) = signal(String::new());
    let (events_str, set_events_str) = signal(String::new());
    let (secret, set_secret) = signal(String::new());
    let (error, set_error) = signal(None::<String>);
    let (submitting, set_submitting) = signal(false);

    let handle_submit = move |_| {
        let u = url.get();
        if u.trim().is_empty() {
            set_error.set(Some("URL is required".to_string()));
            return;
        }
        let ev = events_str.get();
        let parsed_events: Vec<String> = ev
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        if parsed_events.is_empty() {
            set_error.set(Some("At least one event is required".to_string()));
            return;
        }
        set_error.set(None);
        set_submitting.set(true);

        let api = ApiClient::default();
        let s = secret.get();
        let events_owned: Vec<String> = parsed_events;
        let sec_owned = if s.is_empty() { None } else { Some(s) };
        let save_cb = on_save;

        spawn_local(async move {
            let events_refs: Vec<&str> = events_owned.iter().map(|s| s.as_str()).collect();
            let sec_ref = sec_owned.as_deref();
            let result = api.create_webhook(&u, events_refs, sec_ref).await;
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
                            text-red-700 dark:text-red-300 text-sm rounded-none">{e}</div>
            }
        })
    };

    let btn_class = move || {
        format!(
            "px-4 py-2 text-sm text-white rounded-none {}",
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
            <div class="bg-white dark:bg-gray-800 rounded-none shadow-2xl w-full max-w-lg max-h-[90vh] overflow-hidden border border-gray-900 dark:border-gray-100"
                 on:click={move |ev| ev.stop_propagation()}>
                <div class="flex items-center justify-between px-6 py-4 border-b border-gray-200 dark:border-gray-700">
                    <h2 class="text-lg font-semibold text-gray-900 dark:text-white">"Create Webhook"</h2>
                    <button class="p-1 text-gray-400 hover:text-gray-600 dark:hover:text-gray-200 rounded-none
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
                        <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                            "URL" <span class="text-red-500">"*"</span>
                        </label>
                        <input type="url"
                            class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-none
                                   bg-white dark:bg-gray-700 text-gray-900 dark:text-white
                                   focus:ring-2 focus:ring-blue-500 outline-none"
                            placeholder="https://example.com/webhook" prop:value={url.get()}
                            on:input={move |ev| set_url.set(event_target_value(&ev))} />
                    </div>

                    <div>
                        <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                            "Events" <span class="text-red-500">"*"</span>
                        </label>
                        <input type="text"
                            class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-none
                                   bg-white dark:bg-gray-700 text-gray-900 dark:text-white
                                   focus:ring-2 focus:ring-blue-500 outline-none"
                            placeholder="Comma-separated: document_created, document_updated"
                            prop:value={events_str.get()}
                            on:input={move |ev| set_events_str.set(event_target_value(&ev))} />
                        <p class="mt-1 text-xs text-gray-400">
                            "Available events: document_created, document_updated, document_deleted, document_published, team_member_added, team_member_removed, space_created, space_updated"
                        </p>
                    </div>

                    <div>
                        <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                            "Secret"
                        </label>
                        <input type="password"
                            class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-none
                                   bg-white dark:bg-gray-700 text-gray-900 dark:text-white
                                   focus:ring-2 focus:ring-blue-500 outline-none"
                            placeholder="Optional signing secret"
                            prop:value={secret.get()}
                            on:input={move |ev| set_secret.set(event_target_value(&ev))} />
                        <p class="mt-1 text-xs text-gray-400">
                            "Used to verify webhook payloads via HMAC signature"
                        </p>
                    </div>
                </div>

                <div class="flex items-center justify-end gap-3 px-6 py-4 border-t border-gray-200 dark:border-gray-700">
                    <button class="px-4 py-2 text-sm text-gray-700 dark:text-gray-300 border border-gray-300
                                   dark:border-gray-600 rounded-none hover:bg-gray-50 dark:hover:bg-gray-700"
                        on:click={move |_| on_cancel.run(())}>"Cancel"</button>
                    <button class={btn_class} disabled={submitting.get()} on:click={handle_submit}>
                        {move || if submitting.get() { "Creating..." } else { "Create Webhook" }}
                    </button>
                </div>
            </div>
        </div>
    }
}

#[component]
fn DeleteWebhookModal(
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
            match api.delete_webhook(&tid).await {
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
                            dark:border-red-800 text-red-700 dark:text-red-300 text-sm rounded-none">{e}</div>
            }
        })
    };

    let btn_class = move || {
        format!(
            "px-4 py-2 text-sm text-white rounded-none {}",
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
            <div class="bg-white dark:bg-gray-800 rounded-none shadow-2xl w-full max-w-md border border-gray-900 dark:border-gray-100"
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
                            <h3 class="text-lg font-semibold text-gray-900 dark:text-white">"Delete Webhook"</h3>
                            <p class="text-sm text-gray-500 dark:text-gray-400 mt-1">
                                "This will permanently remove this webhook. This action cannot be undone."
                            </p>
                        </div>
                    </div>
                    {error_view}
                    <div class="flex justify-end gap-3 mt-6">
                        <button class="px-4 py-2 text-sm text-gray-700 dark:text-gray-300 border border-gray-300
                                       dark:border-gray-600 rounded-none hover:bg-gray-50 dark:hover:bg-gray-700"
                            on:click={move |_| on_cancel.run(())}>"Cancel"</button>
                        <button class={btn_class} disabled={submitting.get()} on:click={handle_delete}>
                            {move || if submitting.get() { "Deleting..." } else { "Delete" } }
                        </button>
                    </div>
                </div>
            </div>
        </div>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_available_events_count() {
        assert!(AVAILABLE_EVENTS.len() >= 8);
    }

    #[test]
    fn test_webhook_info_fields() {
        let w = WebhookInfo {
            id: "abc".to_string(),
            url: "https://example.com".to_string(),
            events: vec!["document_created".to_string()],
            active: true,
            created_at: "2026-01-01T00:00:00Z".to_string(),
            last_triggered_at: None,
        };
        assert!(w.active);
        assert_eq!(w.events.len(), 1);
    }
}
