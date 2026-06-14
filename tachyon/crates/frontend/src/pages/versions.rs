use crate::api::ApiClient;
use crate::api::versions::{CreateDocVersionRequest, DocVersion};
use leptos::prelude::*;
use wasm_bindgen_futures::spawn_local;

/// Versions management page — lists all documentation versions with
/// create, compare, publish, and rollback actions.
#[component]
pub fn VersionsPage() -> impl IntoView {
    let (versions, set_versions) = signal(Vec::<DocVersion>::new());
    let (loading, set_loading) = signal(true);
    let (error, set_error) = signal(None::<String>);
    let (show_create, set_show_create) = signal(false);
    let (compare_a, set_compare_a) = signal(None::<String>);
    let (compare_b, set_compare_b) = signal(None::<String>);
    let (show_diff, set_show_diff) = signal(false);

    let load_versions = move || {
        set_loading.set(true);
        set_error.set(None);
        spawn_local(async move {
            let api = ApiClient::default();
            match api.list_doc_versions().await {
                Ok(list) => set_versions.set(list),
                Err(e) => set_error.set(Some(format!("Failed to load versions: {}", e))),
            }
            set_loading.set(false);
        });
    };

    load_versions();

    let error_view = move || {
        error.get().map(|e| {
            view! {
                <div class="mb-4 p-3 bg-red-50 dark:bg-red-900/30 border border-red-200 dark:border-red-800
                            text-red-700 dark:text-red-300 text-sm rounded-none">{e}</div>
            }
        })
    };

    let versions_list = move || {
        if loading.get() {
            view! {
                <div class="p-8 text-center text-gray-500 dark:text-gray-400">
                    "Loading versions..."
                </div>
            }
            .into_any()
        } else {
            let list = versions.get();
            if list.is_empty() {
                view! {
                    <div class="p-8 text-center">
                        <p class="text-gray-700 dark:text-gray-300 font-medium">"No versions yet"</p>
                        <p class="text-sm text-gray-500 dark:text-gray-400 mt-1">"Create your first documentation version to get started"</p>
                    </div>
                }.into_any()
            } else {
                view! {
                    <div class="divide-y divide-gray-200 dark:divide-gray-700">
                        {list.into_iter().map(|v| {
                            let vid = v.id.clone();
                            let vid_a = v.id.clone();
                            let vid_b = v.id.clone();
                            let status = v.status.clone();
                            let is_latest = v.is_latest;
                            let name = v.name.clone();
                            let description = v.description.clone().unwrap_or_default();
                            let doc_count = v.document_count;
                            let created = v.created_at.clone();
                            let updated = v.updated_at.clone();
                            let status_class = match v.status.as_str() {
                                "published" => "bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200",
                                "archived" => "bg-gray-100 text-gray-800 dark:bg-gray-700 dark:text-gray-300",
                                _ => "bg-blue-100 text-blue-800 dark:bg-blue-900 dark:text-blue-200",
                            };
                            view! {
                                <div class="p-4 hover:bg-gray-50 dark:hover:bg-gray-700/50 flex items-center justify-between">
                                    <div class="flex items-center gap-4 flex-1">
                                        <input
                                            type="checkbox"
                                            class="w-4 h-4 text-blue-600 rounded"
                                            on:change={
                                                let vid_a2 = vid_a.clone();
                                                move |_| {
                                                    let current = compare_a.get();
                                                    if current.as_deref() == Some(&vid_a2) {
                                                        set_compare_a.set(None);
                                                    } else if compare_b.get().is_none() {
                                                        set_compare_a.set(Some(vid_a2.clone()));
                                                    }
                                                }
                                            }
                                            prop:checked={
                                                let vid_a3 = vid_a.clone();
                                                move || compare_a.get().as_deref() == Some(&vid_a3)
                                            }
                                        />
                                        <div class="flex-1">
                                            <div class="flex items-center gap-2">
                                                <span class="font-medium text-gray-900 dark:text-white">
                                                    {name}
                                                </span>
                                                <span class={format!("inline-flex items-center px-2 py-0.5 rounded-full text-xs font-medium {}", status_class)}>
                                                    {status}
                                                </span>
                                                {if is_latest {
                                                    view! {
                                                        <span class="inline-flex items-center px-2 py-0.5 rounded-full text-xs font-medium bg-yellow-100 text-yellow-800 dark:bg-yellow-900 dark:text-yellow-200">
                                                            "Latest"
                                                        </span>
                                                    }.into_any()
                                                } else {
                                                    view! { <span></span> }.into_any()
                                                }}
                                            </div>
                                            <div class="mt-1 text-sm text-gray-500 dark:text-gray-400">
                                                {description}
                                                " · "{doc_count}" documents"
                                            </div>
                                            <div class="mt-1 text-xs text-gray-400 dark:text-gray-500">
                                                "Created "{created}" · Updated "{updated}
                                            </div>
                                        </div>
                                    </div>
                                    <div class="flex items-center gap-2">
                                        <input
                                            type="checkbox"
                                            class="w-4 h-4 text-blue-600 rounded"
                                            on:change={
                                                let vid_b2 = vid_b.clone();
                                                move |_| {
                                                    let current = compare_b.get();
                                                    if current.as_deref() == Some(&vid_b2) {
                                                        set_compare_b.set(None);
                                                    } else if compare_a.get().is_some() {
                                                        set_compare_b.set(Some(vid_b2.clone()));
                                                    }
                                                }
                                            }
                                            prop:checked={
                                                let vid_b3 = vid_b.clone();
                                                move || compare_b.get().as_deref() == Some(&vid_b3)
                                            }
                                        />
                                        <VersionActions
                                            version_id=vid
                                            status=v.status.clone()
                                            on_refresh=load_versions
                                        />
                                    </div>
                                </div>
                            }
                        }).collect::<Vec<_>>()}
                    </div>
                }.into_any()
            }
        }
    };

    let can_compare = move || compare_a.get().is_some() && compare_b.get().is_some();

    let diff_view = move || {
        if show_diff.get() {
            if let (Some(a), Some(b)) = (compare_a.get(), compare_b.get()) {
                view! {
                    <div class="mt-6">
                        <crate::components::version_diff::VersionDiffView
                            version_a_id=a
                            version_b_id=b
                            document_slug="".to_string()
                        />
                    </div>
                }
                .into_any()
            } else {
                view! { <div></div> }.into_any()
            }
        } else {
            view! { <div></div> }.into_any()
        }
    };

    view! {
        <div>
            <div class="flex items-center justify-between mb-6">
                <div>
                    <h1 class="text-2xl font-bold text-gray-900 dark:text-white">"Version Management"</h1>
                    <p class="mt-1 text-sm text-gray-500 dark:text-gray-400">
                        "Manage documentation versions — create, compare, publish, and rollback"
                    </p>
                </div>
                <div class="flex items-center gap-3">
                    <button
                        class="px-4 py-2 text-sm font-medium text-gray-700 dark:text-gray-300 border border-gray-300 dark:border-gray-600
                               rounded-none hover:bg-gray-50 dark:hover:bg-gray-700 transition-colors disabled:opacity-50"
                        disabled=move || !can_compare()
                        on:click=move |_| set_show_diff.update(|v| *v = !*v)
                    >
                        {move || if show_diff.get() { "Hide Diff" } else { "Compare Selected" }}
                    </button>
                    <button
                        class="px-4 py-2 text-sm font-medium text-white bg-blue-600 rounded-none hover:bg-blue-700 transition-colors"
                        on:click=move |_| set_show_create.set(true)
                    >
                        "New Version"
                    </button>
                </div>
            </div>

            {error_view}

            <div class="bg-white dark:bg-gray-800 rounded-none border border-gray-200 dark:border-gray-700">
                {versions_list}
            </div>

            {diff_view}

            {move || if show_create.get() {
                view! {
                    <CreateVersionModal
                        on_close=move || set_show_create.set(false)
                        on_created=load_versions
                    />
                }.into_any()
            } else {
                view! { <div></div> }.into_any()
            }}
        </div>
    }
}

#[component]
fn VersionActions(
    version_id: String,
    status: String,
    on_refresh: impl Fn() + Clone + 'static,
) -> impl IntoView {
    let (publishing, set_publishing) = signal(false);
    let (rolling_back, set_rolling_back) = signal(false);

    let handle_publish = {
        let vid = version_id.clone();
        let refresh = on_refresh.clone();
        move |_| {
            if publishing.get() {
                return;
            }
            set_publishing.set(true);
            let id = vid.clone();
            let refresh = refresh.clone();
            spawn_local(async move {
                let api = ApiClient::default();
                let _ = api.publish_doc_version(&id).await;
                set_publishing.set(false);
                refresh();
            });
        }
    };

    let handle_rollback = {
        let vid = version_id.clone();
        let refresh = on_refresh.clone();
        move |_| {
            if rolling_back.get() {
                return;
            }
            set_rolling_back.set(true);
            let id = vid.clone();
            let refresh = refresh.clone();
            spawn_local(async move {
                let api = ApiClient::default();
                let _ = api.rollback_doc_version(&id).await;
                set_rolling_back.set(false);
                refresh();
            });
        }
    };

    view! {
        <div class="flex items-center gap-1">
            {if status != "published" {
                view! {
                    <button
                        class="px-2 py-1 text-xs font-medium text-green-700 dark:text-green-300 hover:bg-green-50 dark:hover:bg-green-900/30 rounded transition-colors disabled:opacity-50"
                        disabled=move || publishing.get()
                        on:click=handle_publish
                    >
                        {move || if publishing.get() { "Publishing..." } else { "Publish" }}
                    </button>
                }.into_any()
            } else {
                view! { <span></span> }.into_any()
            }}
            <button
                class="px-2 py-1 text-xs font-medium text-orange-700 dark:text-orange-300 hover:bg-orange-50 dark:hover:bg-orange-900/30 rounded transition-colors disabled:opacity-50"
                disabled=move || rolling_back.get()
                on:click=handle_rollback
            >
                {move || if rolling_back.get() { "Rolling back..." } else { "Rollback" }}
            </button>
        </div>
    }
}

#[component]
fn CreateVersionModal(
    on_close: impl Fn() + Clone + 'static,
    on_created: impl Fn() + Clone + 'static,
) -> impl IntoView {
    let (id, set_id) = signal(String::new());
    let (name, set_name) = signal(String::new());
    let (description, set_description) = signal(String::new());
    let (submitting, set_submitting) = signal(false);
    let (error, set_error) = signal(None::<String>);

    let on_close_for_submit = on_close.clone();
    let on_close_for_cancel = on_close.clone();
    let on_close_for_x = on_close.clone();

    let handle_submit = move |ev: web_sys::SubmitEvent| {
        ev.prevent_default();
        if submitting.get() {
            return;
        }
        let version_id = id.get();
        let version_name = name.get();
        if version_id.is_empty() || version_name.is_empty() {
            set_error.set(Some("ID and Name are required".to_string()));
            return;
        }
        set_submitting.set(true);
        set_error.set(None);
        let desc = description.get();
        let refresh = on_created.clone();
        let close = on_close_for_submit.clone();
        spawn_local(async move {
            let api = ApiClient::default();
            let req = CreateDocVersionRequest {
                id: version_id,
                name: version_name,
                description: if desc.is_empty() { None } else { Some(desc) },
                parent_id: None,
            };
            match api.create_doc_version(&req).await {
                Ok(_) => {
                    refresh();
                    close();
                }
                Err(e) => set_error.set(Some(format!("Failed to create version: {}", e))),
            }
            set_submitting.set(false);
        });
    };

    view! {
        <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/50">
            <div class="bg-white dark:bg-gray-800 rounded-none border border-gray-200 dark:border-gray-700 w-full max-w-md shadow-xl">
                <div class="p-4 border-b border-gray-200 dark:border-gray-700 flex items-center justify-between">
                    <h2 class="text-lg font-semibold text-gray-900 dark:text-white">"New Documentation Version"</h2>
                    <button
                        class="p-1 text-gray-400 hover:text-gray-600 dark:hover:text-gray-300"
                        on:click=move |_| on_close_for_x()
                    >
                        <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
                        </svg>
                    </button>
                </div>

                <form on:submit=handle_submit class="p-4 space-y-4">
                    {move || error.get().map(|e| {
                        view! {
                            <div class="p-2 bg-red-50 dark:bg-red-900/30 border border-red-200 dark:border-red-800
                                        text-red-700 dark:text-red-300 text-sm rounded-none">{e}</div>
                        }
                    })}

                    <div>
                        <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">"Version ID"</label>
                        <input type="text"
                            class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-none
                                   bg-white dark:bg-gray-700 text-gray-900 dark:text-white
                                   focus:ring-2 focus:ring-blue-500 outline-none"
                            placeholder="e.g., 1.0, 2.0-beta"
                            prop:value=move || id.get()
                            on:input=move |ev| set_id.set(event_target_value(&ev)) />
                    </div>

                    <div>
                        <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">"Display Name"</label>
                        <input type="text"
                            class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-none
                                   bg-white dark:bg-gray-700 text-gray-900 dark:text-white
                                   focus:ring-2 focus:ring-blue-500 outline-none"
                            placeholder="e.g., v1.0, Version 2.0 Beta"
                            prop:value=move || name.get()
                            on:input=move |ev| set_name.set(event_target_value(&ev)) />
                    </div>

                    <div>
                        <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">"Description (optional)"</label>
                        <textarea
                            class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-none
                                   bg-white dark:bg-gray-700 text-gray-900 dark:text-white
                                   focus:ring-2 focus:ring-blue-500 outline-none"
                            rows="2"
                            placeholder="What is this version for?"
                            prop:value=move || description.get()
                            on:input=move |ev| set_description.set(event_target_value(&ev)) />
                    </div>

                    <div class="flex justify-end gap-3 pt-2">
                        <button type="button"
                            class="px-4 py-2 text-sm font-medium text-gray-700 dark:text-gray-300 border border-gray-300 dark:border-gray-600
                                   rounded-none hover:bg-gray-50 dark:hover:bg-gray-700 transition-colors"
                            on:click=move |_| on_close_for_cancel()>
                            "Cancel"
                        </button>
                        <button type="submit"
                            class="px-4 py-2 text-sm font-medium text-white bg-blue-600 rounded-none hover:bg-blue-700 transition-colors
                                   disabled:opacity-50"
                            disabled=move || submitting.get()>
                            {move || if submitting.get() { "Creating..." } else { "Create Version" }}
                        </button>
                    </div>
                </form>
            </div>
        </div>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_doc_version_request_serialization() {
        let req = CreateDocVersionRequest {
            id: "1.0".to_string(),
            name: "v1.0".to_string(),
            description: Some("First release".to_string()),
            parent_id: None,
        };
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("1.0"));
        assert!(json.contains("v1.0"));
        assert!(json.contains("First release"));
    }

    #[test]
    fn test_doc_version_serialization() {
        let v = DocVersion {
            id: "1.0".to_string(),
            name: "v1.0".to_string(),
            description: Some("First release".to_string()),
            status: "draft".to_string(),
            parent_id: None,
            document_count: 5,
            is_latest: false,
            created_at: "2024-01-01T00:00:00Z".to_string(),
            updated_at: "2024-01-02T00:00:00Z".to_string(),
        };
        let json = serde_json::to_string(&v).unwrap();
        assert!(json.contains("draft"));
        assert!(json.contains("\"document_count\":5"));
    }
}
