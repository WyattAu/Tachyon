#![allow(dead_code)]

use leptos::prelude::*;
use crate::api::ApiClient;
use crate::types::ConflictInfo;
use std::sync::{Arc, Mutex};

#[component]
pub fn ConflictResolver(
    document_id: String,
    #[prop(optional)] on_resolved: Option<Callback<String>>,
) -> impl IntoView {
    let api_client = Arc::new(Mutex::new(ApiClient::default()));

    let (conflict_info, set_conflict_info) = signal::<Option<ConflictInfo>>(None);
    let (loading, set_loading) = signal(true);
    let (error_msg, set_error_msg) = signal::<Option<String>>(None);
    let (resolving, set_resolving) = signal(false);

    let (has_conflict, set_has_conflict) = signal(false);
    let (is_clean, set_is_clean) = signal(false);
    let (conflict_count, set_conflict_count) = signal(0usize);
    let (merge_content, set_merge_content) = signal(String::new());
    let (current_content, set_current_content) = signal(String::new());
    let (incoming_content, set_incoming_content) = signal(String::new());

    let doc_id_load = document_id.clone();
    let api_load = api_client.clone();
    let set_ci = set_conflict_info.clone();
    let set_ld = set_loading.clone();
    let set_er = set_error_msg.clone();

    wasm_bindgen_futures::spawn_local(async move {
        let client = api_load.lock().unwrap().clone();
        match client.get_conflict_info(&doc_id_load).await {
            Ok(info) => {
                set_ci.set(Some(info));
            }
            Err(e) => {
                set_er.set(Some(format!("Failed to load conflict info: {}", e)));
            }
        }
        set_ld.set(false);
    });

    let _ = Effect::new(move || {
        let info = conflict_info.get();
        match info.as_ref() {
            Some(i) if i.has_conflict => {
                set_has_conflict.set(true);
                let merge = i.merge_result.as_ref();
                set_is_clean.set(merge.as_ref().map_or(false, |m| m.status == "clean"));
                set_conflict_count.set(merge.as_ref().map_or(0, |m| m.conflict_count));
                set_merge_content.set(merge.as_ref().map(|m| m.content.clone()).unwrap_or_default());
                set_current_content.set(i.current_content.clone().unwrap_or_default());
                set_incoming_content.set(i.incoming_content.clone().unwrap_or_default());
            }
            _ => {
                set_has_conflict.set(false);
            }
        }
    });

    let on_accept_ours = {
        let doc_id = document_id.clone();
        let api = api_client.clone();
        let on_resolved_cb = on_resolved.clone();
        let set_rs = set_resolving.clone();
        let set_er = set_error_msg.clone();
        move |_: leptos::ev::MouseEvent| {
            let doc_id = doc_id.clone();
            let api = api.lock().unwrap().clone();
            let on_resolved_cb = on_resolved_cb.clone();
            let set_rs = set_rs.clone();
            let set_er = set_er.clone();
            wasm_bindgen_futures::spawn_local(async move {
                set_rs.set(true);
                match api.resolve_conflict(&doc_id, "ours", None).await {
                    Ok(_) => {
                        set_er.set(None);
                        if let Some(cb) = on_resolved_cb {
                            cb.run(doc_id);
                        }
                    }
                    Err(e) => {
                        set_er.set(Some(format!("Failed to resolve: {}", e)));
                    }
                }
                set_rs.set(false);
            });
        }
    };

    let on_accept_theirs = {
        let doc_id = document_id.clone();
        let api = api_client.clone();
        let on_resolved_cb = on_resolved.clone();
        let set_rs = set_resolving.clone();
        let set_er = set_error_msg.clone();
        move |_: leptos::ev::MouseEvent| {
            let doc_id = doc_id.clone();
            let api = api.lock().unwrap().clone();
            let on_resolved_cb = on_resolved_cb.clone();
            let set_rs = set_rs.clone();
            let set_er = set_er.clone();
            wasm_bindgen_futures::spawn_local(async move {
                set_rs.set(true);
                match api.resolve_conflict(&doc_id, "theirs", None).await {
                    Ok(_) => {
                        set_er.set(None);
                        if let Some(cb) = on_resolved_cb {
                            cb.run(doc_id);
                        }
                    }
                    Err(e) => {
                        set_er.set(Some(format!("Failed to resolve: {}", e)));
                    }
                }
                set_rs.set(false);
            });
        }
    };

    let on_accept_merge = {
        let doc_id = document_id.clone();
        let api = api_client.clone();
        let on_resolved_cb = on_resolved.clone();
        let set_rs = set_resolving.clone();
        let set_er = set_error_msg.clone();
        move |_: leptos::ev::MouseEvent| {
            let doc_id = doc_id.clone();
            let content = doc_id.clone();
            let api = api.lock().unwrap().clone();
            let on_resolved_cb = on_resolved_cb.clone();
            let set_rs = set_rs.clone();
            let set_er = set_er.clone();
            wasm_bindgen_futures::spawn_local(async move {
                set_rs.set(true);
                match api.resolve_conflict(&doc_id, "manual", Some(&content)).await {
                    Ok(_) => {
                        set_er.set(None);
                        if let Some(cb) = on_resolved_cb {
                            cb.run(doc_id);
                        }
                    }
                    Err(e) => {
                        set_er.set(Some(format!("Failed to resolve: {}", e)));
                    }
                }
                set_rs.set(false);
            });
        }
    };

    let loading_cls = move || if loading.get() { "" } else { "hidden" };
    let _error_cls = move || if error_msg.get().is_some() && !loading.get() { "" } else { "hidden" };
    let no_conflict_cls = move || if !loading.get() && error_msg.get().is_none() && !has_conflict.get() { "" } else { "hidden" };
    let conflict_cls = move || if !loading.get() && error_msg.get().is_none() && has_conflict.get() { "" } else { "hidden" };

    view! {
        <div class="bg-white dark:bg-gray-800 rounded-lg shadow border border-gray-200 dark:border-gray-700">
            <div class="p-4 border-b border-gray-200 dark:border-gray-700">
                <h3 class="text-lg font-semibold text-gray-900 dark:text-white">"Conflict Resolution"</h3>
            </div>

            <div class="p-4">
                <div class={loading_cls}>
                    <div class="text-gray-500 text-sm">"Loading conflict information..."</div>
                </div>

                {move || {
                    error_msg.get().map(|err| {
                        view! {
                            <div class="text-red-600 dark:text-red-400 text-sm">{err}</div>
                        }.into_any()
                    }).unwrap_or_else(|| view! { <span></span> }.into_any())
                }}

                <div class={no_conflict_cls}>
                    <div class="text-green-600 dark:text-green-400 text-sm">"No conflicts detected for this document."</div>
                </div>

                <div class={conflict_cls}>
                    <div class="mb-4">
                        {move || {
                            if is_clean.get() {
                                view! {
                                    <span class="inline-flex items-center px-2 py-0.5 rounded-full text-xs font-medium bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200">
                                        "Clean Merge"
                                    </span>
                                }.into_any()
                            } else {
                                view! {
                                    <span class="inline-flex items-center px-2 py-0.5 rounded-full text-xs font-medium bg-red-100 text-red-800 dark:bg-red-900 dark:text-red-200">
                                        {move || conflict_count.get()} " conflict(s)"
                                    </span>
                                }.into_any()
                            }
                        }}
                    </div>

                    <div class="grid grid-cols-1 lg:grid-cols-3 gap-4 mb-4">
                        <div>
                            <h4 class="text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">"Current (Server)"</h4>
                            <pre class="p-3 bg-gray-50 dark:bg-gray-900 rounded text-xs overflow-auto max-h-96 text-gray-800 dark:text-gray-200">{move || current_content.get()}</pre>
                        </div>
                        <div>
                            <h4 class="text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">"Merged Result"</h4>
                            <pre class="p-3 bg-gray-50 dark:bg-gray-900 rounded text-xs overflow-auto max-h-96 text-gray-800 dark:text-gray-200">{move || merge_content.get()}</pre>
                        </div>
                        <div>
                            <h4 class="text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">"Incoming (File)"</h4>
                            <pre class="p-3 bg-gray-50 dark:bg-gray-900 rounded text-xs overflow-auto max-h-96 text-gray-800 dark:text-gray-200">{move || incoming_content.get()}</pre>
                        </div>
                    </div>

                    <div class="flex items-center gap-3">
                        <button
                            class="px-3 py-1.5 text-sm bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors"
                            on:click=on_accept_ours
                        >
                            "Accept Ours"
                        </button>
                        <button
                            class="px-3 py-1.5 text-sm bg-purple-600 text-white rounded-lg hover:bg-purple-700 transition-colors"
                            on:click=on_accept_theirs
                        >
                            "Accept Theirs"
                        </button>
                        <button
                            class="px-3 py-1.5 text-sm bg-green-600 text-white rounded-lg hover:bg-green-700 transition-colors"
                            on:click=on_accept_merge
                        >
                            "Accept Merge"
                        </button>
                        {move || if resolving.get() {
                            view! {
                                <span class="text-sm text-gray-500">"Resolving..."</span>
                            }.into_any()
                        } else {
                            view! { <span></span> }.into_any()
                        }}
                    </div>
                </div>
            </div>
        </div>
    }
}
