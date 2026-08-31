#![allow(dead_code)]

use crate::api::ApiClient;
use crate::types::AuditLogEntry;
use leptos::prelude::*;
use leptos::task::spawn_local;
use wasm_bindgen::JsCast;

fn format_timestamp(ts: &str) -> String {
    let dt = chrono::DateTime::parse_from_rfc3339(ts);
    let Ok(past) = dt else {
        return ts.split('T').next().unwrap_or("Unknown").to_string();
    };
    past.format("%Y-%m-%d %H:%M").to_string()
}

async fn fetch_audit_logs(
    page: u32,
    page_size: u32,
    action: Option<&str>,
    actor_id: Option<&str>,
) -> Result<(Vec<AuditLogEntry>, usize), String> {
    let client = ApiClient::default();
    let raw = client
        .list_audit_logs(Some(page), Some(page_size), action, actor_id)
        .await
        .map_err(|e| e.to_string())?;

    let entries: Vec<AuditLogEntry> = raw
        .get("entries")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|item| serde_json::from_value(item.clone()).ok())
                .collect()
        })
        .unwrap_or_default();

    let total = raw.get("total").and_then(|v| v.as_u64()).unwrap_or(0) as usize;

    Ok((entries, total))
}

#[component]
pub fn AuditPage() -> impl IntoView {
    let (entries, set_entries) = signal(Vec::<AuditLogEntry>::new());
    let (total, set_total) = signal(0usize);
    let (loading, set_loading) = signal(true);
    let (error, set_error) = signal(None::<String>);
    let (page, set_page) = signal(1u32);
    let page_size = 20u32;
    let (filter_action, set_filter_action) = signal(String::new());
    let (filter_actor, set_filter_actor) = signal(String::new());

    let load_data = move || {
        let action = filter_action.get();
        let actor = filter_actor.get();
        let p = page.get();
        let act_filter = if action.is_empty() {
            None
        } else {
            Some(action)
        };
        let actor_filter = if actor.is_empty() { None } else { Some(actor) };
        spawn_local(async move {
            set_loading.set(true);
            set_error.set(None);
            match fetch_audit_logs(p, page_size, act_filter.as_deref(), actor_filter.as_deref())
                .await
            {
                Ok((ents, tot)) => {
                    set_entries.set(ents);
                    set_total.set(tot);
                }
                Err(e) => set_error.set(Some(e)),
            }
            set_loading.set(false);
        });
    };

    Effect::new(move |_| {
        load_data();
    });

    let apply_filters = move |_: leptos::ev::MouseEvent| {
        set_page.set(1);
        load_data();
    };

    let clear_filters = move |_: leptos::ev::MouseEvent| {
        set_filter_action.set(String::new());
        set_filter_actor.set(String::new());
        set_page.set(1);
        load_data();
    };

    let prev_page = move |_: leptos::ev::MouseEvent| {
        if page.get() > 1 {
            set_page.update(|p| *p -= 1);
            load_data();
        }
    };

    let next_page = move |_: leptos::ev::MouseEvent| {
        let max_page = ((total.get() as f64) / (page_size as f64)).ceil() as u32;
        if page.get() < max_page {
            set_page.update(|p| *p += 1);
            load_data();
        }
    };

    let export_csv = move |_: leptos::ev::MouseEvent| {
        let ents = entries.get();
        let mut csv = String::from("ID,Action,Actor,Target Type,Target ID,Details,Timestamp\n");
        for e in &ents {
            csv.push_str(&format!(
                "\"{}\",\"{}\",\"{}\",\"{}\",\"{}\",\"{}\",\"{}\"\n",
                e.id,
                e.action,
                e.actor_name.as_deref().unwrap_or(&e.actor_id),
                e.target_type,
                e.target_id,
                e.details.as_deref().unwrap_or(""),
                e.timestamp,
            ));
        }
        if let Some(window) = web_sys::window() {
            let arr = js_sys::Array::new();
            arr.push(&js_sys::JsString::from(csv.as_str()));
            let blob = web_sys::Blob::new_with_str_sequence(&arr).ok();
            if let Some(blob) = blob {
                let url = web_sys::Url::create_object_url_with_blob(&blob).unwrap_or_default();
                if let Some(document) = window.document() {
                    let a = document.create_element("a").unwrap();
                    a.set_attribute("href", &url).unwrap();
                    a.set_attribute("download", "audit-log.csv").unwrap();
                    if let Some(body) = document.body() {
                        body.append_child(&a).unwrap();
                        let _ = a
                            .dyn_ref::<web_sys::HtmlElement>()
                            .map(|el: &web_sys::HtmlElement| el.click());
                        body.remove_child(&a).unwrap();
                    }
                }
                let _ = web_sys::Url::revoke_object_url(&url);
            }
        }
    };

    let total_pages = move || ((total.get() as f64) / (page_size as f64)).ceil() as u32;
    let current_page = move || page.get();

    view! {
        <div class="p-4 md:p-6 max-w-6xl mx-auto">
            <div class="flex flex-col sm:flex-row justify-between items-start sm:items-center mb-6">
                <div>
                    <h1 class="text-xl sm:text-2xl font-bold text-gray-900 dark:text-white">"Audit Log"</h1>
                    <p class="text-gray-600 dark:text-gray-400 mt-1">"Track all actions and changes across your organization."</p>
                </div>
                <button
                    class="min-h-[44px] px-4 py-2 text-sm border border-gray-300 dark:border-gray-600 text-gray-700 dark:text-gray-300 rounded-none hover:bg-gray-50 dark:hover:bg-gray-700 transition-colors"
                    on:click=export_csv
                >
                    "Export CSV"
                </button>
            </div>

            // Filters
            <div class="bg-white dark:bg-gray-800 shadow rounded-none p-4 mb-6 border border-gray-900 dark:border-gray-100">
                <div class="flex flex-wrap items-end gap-4">
                    <div class="flex-1 min-w-[200px]">
                        <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">"Action"</label>
                        <input
                            type="text"
                            class="w-full min-h-[44px] px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-none focus:outline-none focus:ring-2 focus:ring-blue-500 dark:bg-gray-700 dark:text-white text-sm"
                            placeholder="e.g. document.create"
                            value={filter_action.get()}
                            on:input=move |ev| set_filter_action.set(event_target_value(&ev))
                        />
                    </div>
                    <div class="flex-1 min-w-[200px]">
                        <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">"Actor"</label>
                        <input
                            type="text"
                            class="w-full min-h-[44px] px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-none focus:outline-none focus:ring-2 focus:ring-blue-500 dark:bg-gray-700 dark:text-white text-sm"
                            placeholder="User ID"
                            value={filter_actor.get()}
                            on:input=move |ev| set_filter_actor.set(event_target_value(&ev))
                        />
                    </div>
                    <div class="flex gap-2">
                        <button
                            class="min-h-[44px] px-4 py-2 bg-blue-600 text-white rounded-none hover:bg-blue-700 transition-colors text-sm"
                            on:click=apply_filters
                        >
                            "Filter"
                        </button>
                        <button
                            class="min-h-[44px] px-4 py-2 text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-none transition-colors text-sm"
                            on:click=clear_filters
                        >
                            "Clear"
                        </button>
                    </div>
                </div>
            </div>

            {move || error.get().map(|e| view! {
                <div class="mb-4 p-4 bg-red-100 text-red-700 dark:bg-red-900 dark:text-red-200 rounded-none">
                    {e}
                </div>
            })}

            <div class="bg-white dark:bg-gray-800 shadow rounded-none overflow-hidden border border-gray-900 dark:border-gray-100">
                {move || if loading.get() {
                    Some(view! {
                        <div class="flex justify-center items-center py-12">
                            <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600"></div>
                        </div>
                    }.into_any())
                } else {
                    let ents = entries.get();
                    if ents.is_empty() {
                        Some(view! {
                            <div class="text-center py-12">
                                <p class="text-gray-500 dark:text-gray-400">"No audit log entries found."</p>
                            </div>
                        }.into_any())
                    } else {
                        Some(view! {
                            <div class="overflow-x-auto">
                                <table class="w-full text-sm">
                                    <thead>
                                        <tr class="border-b border-gray-200 dark:border-gray-700">
                                            <th class="text-left py-3 px-4 font-medium text-gray-500 dark:text-gray-400">"Timestamp"</th>
                                            <th class="text-left py-3 px-4 font-medium text-gray-500 dark:text-gray-400">"Action"</th>
                                            <th class="text-left py-3 px-4 font-medium text-gray-500 dark:text-gray-400">"Actor"</th>
                                            <th class="text-left py-3 px-4 font-medium text-gray-500 dark:text-gray-400">"Target"</th>
                                            <th class="text-left py-3 px-4 font-medium text-gray-500 dark:text-gray-400">"Details"</th>
                                        </tr>
                                    </thead>
                                    <tbody class="divide-y divide-gray-200 dark:divide-gray-700">
                                        {ents.into_iter().map(|entry| {
                                            let action = entry.action.clone();
                                            let action_color = if action.contains("create") || action.contains("add") {
                                                "bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200"
                                            } else if action.contains("delete") || action.contains("remove") {
                                                "bg-red-100 text-red-800 dark:bg-red-900 dark:text-red-200"
                                            } else if action.contains("update") {
                                                "bg-blue-100 text-blue-800 dark:bg-blue-900 dark:text-blue-200"
                                            } else {
                                                "bg-gray-100 text-gray-800 dark:bg-gray-700 dark:text-gray-200"
                                            };
                                            view! {
                                                <tr class="hover:bg-gray-50 dark:hover:bg-gray-700/50">
                                                    <td class="py-3 px-4 text-gray-500 dark:text-gray-400 whitespace-nowrap">
                                                        {format_timestamp(&entry.timestamp)}
                                                    </td>
                                                    <td class="py-3 px-4">
                                                        <span class={format!("px-2 py-1 rounded text-xs font-medium {}", action_color)}>
                                                            {action}
                                                        </span>
                                                    </td>
                                                    <td class="py-3 px-4 text-gray-900 dark:text-white">
                                                        {entry.actor_name.unwrap_or(entry.actor_id)}
                                                    </td>
                                                    <td class="py-3 px-4 text-gray-600 dark:text-gray-400">
                                                        {entry.target_type}"/"{entry.target_id}
                                                    </td>
                                                    <td class="py-3 px-4 text-gray-500 dark:text-gray-400 max-w-xs truncate">
                                                        {entry.details.unwrap_or("-".to_string())}
                                                    </td>
                                                </tr>
                                            }
                                        }).collect::<Vec<_>>()}
                                    </tbody>
                                </table>
                            </div>

                            // Pagination
                            <div class="border-t border-gray-200 dark:border-gray-700 px-4 py-3 flex items-center justify-between">
                                <p class="text-sm text-gray-500 dark:text-gray-400">
                                    "Page "{current_page()}" of "{total_pages()}
                                    " ("{total.get()}" entries)"
                                </p>
                                <div class="flex gap-2">
                                    <button
                                        class="min-h-[44px] px-3 py-2 text-sm border border-gray-300 dark:border-gray-600 rounded-none hover:bg-gray-50 dark:hover:bg-gray-700 disabled:opacity-50 disabled:cursor-not-allowed text-gray-700 dark:text-gray-300"
                                        disabled={page.get() <= 1}
                                        on:click=prev_page
                                    >
                                        "Previous"
                                    </button>
                                    <button
                                        class="min-h-[44px] px-3 py-2 text-sm border border-gray-300 dark:border-gray-600 rounded-none hover:bg-gray-50 dark:hover:bg-gray-700 disabled:opacity-50 disabled:cursor-not-allowed text-gray-700 dark:text-gray-300"
                                        disabled={page.get() >= total_pages()}
                                        on:click=next_page
                                    >
                                        "Next"
                                    </button>
                                </div>
                            </div>
                        }.into_any())
                    }
                }}
            </div>
        </div>
    }
}
