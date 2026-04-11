// Review Panel Component
// Displays review workflow status, comments, and approve/reject actions

#![allow(dead_code)]

use leptos::prelude::*;
use crate::api::ApiClient;
use crate::types::{DocumentReview, ReviewComment};
use wasm_bindgen::JsCast;
use std::sync::{Arc, Mutex};

/// Review status badge with color coding
fn status_badge(status: &str) -> (String, String) {
    match status {
        "pending" => ("bg-yellow-100 text-yellow-800 dark:bg-yellow-900 dark:text-yellow-200".to_string(), "Pending".to_string()),
        "approved" => ("bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200".to_string(), "Approved".to_string()),
        "rejected" => ("bg-red-100 text-red-800 dark:bg-red-900 dark:text-red-200".to_string(), "Rejected".to_string()),
        "changes_requested" => ("bg-orange-100 text-orange-800 dark:bg-orange-900 dark:text-orange-200".to_string(), "Changes Requested".to_string()),
        "cancelled" => ("bg-gray-100 text-gray-600 dark:bg-gray-700 dark:text-gray-300".to_string(), "Cancelled".to_string()),
        _ => ("bg-gray-100 text-gray-600".to_string(), status.to_string()),
    }
}

/// Review Panel — shows review status, list, and actions for a document
#[component]
pub fn ReviewPanel(
    document_id: String,
) -> impl IntoView {
    let api_client = Arc::new(Mutex::new(ApiClient::default()));

    let (refresh_counter, set_refresh_counter) = signal(0u32);

    // Pre-clone for closures that need api_client after move
    let api_client_for_submit = api_client.clone();

    let reviews_resource = LocalResource::new({
        let api_client = api_client.clone();
        let document_id = document_id.clone();
        move || {
            let _ = refresh_counter.get();
            let client = api_client.lock().ok().map(|g| g.clone()).unwrap_or_default();
            let doc_id = document_id.clone();
            async move {
                client.list_reviews(&doc_id).await.unwrap_or_default()
            }
        }
    });

    let status_resource = LocalResource::new({
        let api_client = api_client.clone();
        let document_id = document_id.clone();
        move || {
            let _ = refresh_counter.get();
            let client = api_client.lock().ok().map(|g| g.clone()).unwrap_or_default();
            let doc_id = document_id.clone();
            async move {
                client.get_review_status(&doc_id).await.ok()
            }
        }
    });

    let doc_id_for_new = document_id.clone();
    let set_rc_for_new = set_refresh_counter.clone();

    let on_submit_review = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let doc_id = doc_id_for_new.clone();
        let set_rc = set_rc_for_new.clone();
        let client = match api_client_for_submit.lock().ok() {
            Some(guard) => guard.clone(),
            None => return,
        };

        wasm_bindgen_futures::spawn_local(async move {
            // Use the author from localStorage or a default
            let reviewer_id = crate::components::auth_guard::get_user_id().unwrap_or_else(|| "anonymous".to_string());
            let _ = client.create_review(&doc_id, &reviewer_id, Some("Submitted for review")).await;
            set_rc.update(|n| *n += 1);
        });
    };

    view! {
        <div class="bg-white dark:bg-gray-800 rounded-lg shadow border border-gray-200 dark:border-gray-700">
            <div class="p-4 border-b border-gray-200 dark:border-gray-700 flex justify-between items-center">
                <h3 class="text-lg font-semibold text-gray-900 dark:text-white">"Reviews"</h3>
                <form on:submit=on_submit_review>
                    <button
                        type="submit"
                        class="px-3 py-1.5 text-sm bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors"
                    >
                        "Submit for Review"
                    </button>
                </form>
            </div>

            // Status summary
            <Suspense fallback=view! { <div class="p-3 text-gray-500 text-sm">"Loading..."</div> }>
                {move || {
                    status_resource.get().map(|maybe_status| {
                        match maybe_status {
                            Some(status) => view! {
                                <div class="px-4 py-2 border-b border-gray-100 dark:border-gray-700 flex items-center gap-4 text-sm">
                                    {move || {
                                        if status.pending_count > 0 {
                                            view! {
                                                <span class="inline-flex items-center px-2 py-0.5 rounded-full text-xs font-medium bg-yellow-100 text-yellow-800 dark:bg-yellow-900 dark:text-yellow-200">
                                                    {status.pending_count} "pending"
                                                </span>
                                            }.into_any()
                                        } else {
                                            view! { <span></span> }.into_any()
                                        }
                                    }}
                                    {move || {
                                        if let Some(ref latest) = status.latest_status {
                                            let (cls, label) = status_badge(latest);
                                            view! {
                                                <span class={format!("inline-flex items-center px-2 py-0.5 rounded-full text-xs font-medium {}", cls)}>
                                                    {label}
                                                </span>
                                            }.into_any()
                                        } else {
                                            view! { <span class="text-gray-400">"No reviews yet"</span> }.into_any()
                                        }
                                    }}
                                </div>
                            }.into_any(),
                            None => view! { <div></div> }.into_any(),
                        }
                    }).unwrap_or_else(|| view! { <div></div> }.into_any())
                }}
            </Suspense>

            // Reviews list
            <Suspense fallback=view! { <div class="p-4 text-gray-500">"Loading reviews..."</div> }>
                <ReviewList
                    document_id=document_id.clone()
                    reviews_resource=reviews_resource
                    api_client=api_client.clone()
                    set_refresh_counter=set_refresh_counter
                />
            </Suspense>
        </div>
    }
}

#[component]
fn ReviewList(
    document_id: String,
    reviews_resource: LocalResource<Vec<DocumentReview>>,
    api_client: Arc<Mutex<ApiClient>>,
    set_refresh_counter: WriteSignal<u32>,
) -> impl IntoView {
    move || {
        let reviews = reviews_resource.get();
        reviews.map(|reviews| {
            if reviews.is_empty() {
                view! {
                    <div class="p-4 text-center text-gray-500 dark:text-gray-400">
                        "No reviews submitted yet"
                    </div>
                }.into_any()
            } else {
                let doc_id = document_id.clone();
                let api = api_client.clone();
                let set_rc = set_refresh_counter.clone();
                view! {
                    <ul class="divide-y divide-gray-200 dark:border-gray-700">
                        <For
                            each=move || reviews.clone()
                            key=|r| r.id.clone()
                            let:review
                        >
                            <ReviewItem
                                document_id=doc_id.clone()
                                review=review
                                api_client=api.clone()
                                set_refresh_counter=set_rc.clone()
                            />
                        </For>
                    </ul>
                }.into_any()
            }
        }).unwrap_or_else(|| view! { <div></div> }.into_any())
    }
}

#[component]
fn ReviewItem(
    document_id: String,
    review: DocumentReview,
    api_client: Arc<Mutex<ApiClient>>,
    set_refresh_counter: WriteSignal<u32>,
) -> impl IntoView {
    let review_id = review.id.clone();
    let status = review.status.clone();
    let summary = review.summary.clone().unwrap_or_else(|| "No summary".to_string());
    let reviewer_id = review.reviewer_id.clone();
    let created_at = format_timestamp(&review.created_at);

    let (show_comments, set_show_comments) = signal(false);

    let (comments, set_comments) = signal(Vec::<ReviewComment>::new());
    let (comment_text, set_comment_text) = signal(String::new());

    // Compute review state flags early (needed by closures below)
    let is_pending = status == "pending";
    let is_changes_requested = status == "changes_requested";

    let doc_id_approve = document_id.clone();
    let review_id_approve = review_id.clone();
    let api_approve = api_client.clone();
    let set_rc_approve = set_refresh_counter.clone();
    let is_pending_a = is_pending;
    let is_cr_a = is_changes_requested;

    let on_approve = move |_: leptos::ev::MouseEvent| {
        if !is_pending_a && !is_cr_a {
            return;
        }
        let doc_id = doc_id_approve.clone();
        let rid = review_id_approve.clone();
        let api = match api_approve.lock().ok() {
            Some(guard) => guard.clone(),
            None => return,
        };
        let set_rc = set_rc_approve.clone();
        wasm_bindgen_futures::spawn_local(async move {
            let _ = api.update_review(&doc_id, &rid, "approved", Some("Approved")).await;
            set_rc.update(|n| *n += 1);
        });
    };

    let doc_id_reject = document_id.clone();
    let review_id_reject = review_id.clone();
    let api_reject = api_client.clone();
    let set_rc_reject = set_refresh_counter.clone();
    let is_pending_r = is_pending;
    let is_cr_r = is_changes_requested;

    let on_reject = move |_: leptos::ev::MouseEvent| {
        if !is_pending_r && !is_cr_r {
            return;
        }
        let doc_id = doc_id_reject.clone();
        let rid = review_id_reject.clone();
        let api = match api_reject.lock().ok() {
            Some(guard) => guard.clone(),
            None => return,
        };
        let set_rc = set_rc_reject.clone();
        wasm_bindgen_futures::spawn_local(async move {
            let _ = api.update_review(&doc_id, &rid, "rejected", Some("Rejected")).await;
            set_rc.update(|n| *n += 1);
        });
    };

    let doc_id_changes = document_id.clone();
    let review_id_changes = review_id.clone();
    let api_changes = api_client.clone();
    let set_rc_changes = set_refresh_counter.clone();
    let is_pending_c = is_pending;
    let is_cr_c = is_changes_requested;

    let on_request_changes = move |_: leptos::ev::MouseEvent| {
        if !is_pending_c && !is_cr_c {
            return;
        }
        let doc_id = doc_id_changes.clone();
        let rid = review_id_changes.clone();
        let api = match api_changes.lock().ok() {
            Some(guard) => guard.clone(),
            None => return,
        };
        let set_rc = set_rc_changes.clone();
        wasm_bindgen_futures::spawn_local(async move {
            let _ = api.update_review(&doc_id, &rid, "changes_requested", Some("Changes requested")).await;
            set_rc.update(|n| *n += 1);
        });
    };

    let doc_id_comments = document_id.clone();
    let review_id_comments = review_id.clone();
    let api_comments = api_client.clone();
    let set_comments_clone = set_comments.clone();
    let _set_rc_comments = set_refresh_counter.clone();

    let doc_id_post = document_id;
    let review_id_post = review_id.clone();
    let api_post = api_client.clone();
    let set_rc_post = set_refresh_counter;

    let on_post_comment = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let text = comment_text.get_untracked();
        if text.trim().is_empty() {
            return;
        }
        let doc_id = doc_id_post.clone();
        let rid = review_id_post.clone();
        let api = match api_post.lock().ok() {
            Some(guard) => guard.clone(),
            None => return,
        };
        let set_rc = set_rc_post.clone();
        let set_c = set_comments.clone();
        let content = text;

        wasm_bindgen_futures::spawn_local(async move {
            let author_id = crate::components::auth_guard::get_user_id().unwrap_or_else(|| "anonymous".to_string());
            if let Ok(comment) = api.create_review_comment(&doc_id, &rid, &author_id, &content).await {
                set_c.update(|c| {
                    c.push(comment);
                });
                set_rc.update(|n| *n += 1);
            }
        });
        set_comment_text.set(String::new());
    };

    let (badge_cls, badge_label) = status_badge(&status);

    view! {
        <li class="p-4 hover:bg-gray-50 dark:hover:bg-gray-700/50">
            <div class="flex items-start justify-between">
                <div class="flex-1">
                    <div class="flex items-center gap-2 mb-1">
                        <span class={format!("inline-flex items-center px-2 py-0.5 rounded-full text-xs font-medium {}", badge_cls)}>
                            {badge_label}
                        </span>
                        <span class="text-xs text-gray-500 dark:text-gray-400">
                            {created_at}
                        </span>
                        <span class="text-xs text-gray-400 dark:text-gray-500">
                            "by "{reviewer_id}
                        </span>
                    </div>
                    <p class="text-sm text-gray-700 dark:text-gray-300">{summary}</p>
                </div>

                // Action buttons (always rendered; handlers are no-ops when not applicable)
                <div class="flex items-center gap-2 ml-4">
                    <button
                        class="px-2 py-1 text-xs rounded bg-green-600 text-white hover:bg-green-700 transition-colors"
                        on:click=on_approve
                    >
                        "Approve"
                    </button>
                    <button
                        class="px-2 py-1 text-xs rounded bg-orange-500 text-white hover:bg-orange-600 transition-colors"
                        on:click=on_request_changes
                    >
                        "Request Changes"
                    </button>
                    <button
                        class="px-2 py-1 text-xs rounded bg-red-600 text-white hover:bg-red-700 transition-colors"
                        on:click=on_reject
                    >
                        "Reject"
                    </button>
                </div>

                // Comments toggle
                <button
                    class="ml-2 text-xs text-blue-600 dark:text-blue-400 hover:underline"
                    on:click=move |_| {
                        let new_show = !show_comments.get_untracked();
                        set_show_comments.set(new_show);
                        if new_show {
                            let doc_id = doc_id_comments.clone();
                            let rid = review_id_comments.clone();
                            let api = match api_comments.lock().ok() {
                                Some(guard) => guard.clone(),
                                None => return,
                            };
                            let set_c = set_comments_clone.clone();
                            wasm_bindgen_futures::spawn_local(async move {
                                if let Ok(coms) = api.list_review_comments(&doc_id, &rid).await {
                                    set_c.set(coms);
                                }
                            });
                        }
                    }
                >
                    {move || if show_comments.get() { "Hide Comments" } else { "Comments" }}
                </button>
            </div>

            // Comments section (always rendered; visibility controlled via class)
            <div class={move || if show_comments.get() { "mt-3 ml-4 border-l-2 border-gray-200 dark:border-gray-600 pl-3" } else { "mt-3 ml-4 border-l-2 border-gray-200 dark:border-gray-600 pl-3 hidden" }}>
                            // Existing comments (always render ul; For handles empty case)
                            <ul class="space-y-2 mb-3">
                                <For
                                    each=move || comments.get().clone()
                                    key=|c| c.id.clone()
                                    let:comment
                                >
                                    <li class="text-sm">
                                        <div class="flex items-center gap-2 text-xs text-gray-500 dark:text-gray-400">
                                            <span class="font-medium">{comment.author_id.clone()}</span>
                                            <span>{format_timestamp(&comment.created_at)}</span>
                                        </div>
                                        <p class="mt-0.5 text-gray-700 dark:text-gray-300">{comment.content.clone()}</p>
                                    </li>
                                </For>
                            </ul>

                            // New comment form
                            <form on:submit=on_post_comment class="flex gap-2">
                                <input
                                    type="text"
                                    placeholder="Add a comment..."
                                    class="flex-1 px-2 py-1 text-sm border border-gray-300 dark:border-gray-600 rounded bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 focus:outline-none focus:ring-1 focus:ring-blue-500"
                                    prop:value={move || comment_text.get()}
                                    on:input=move |ev: leptos::ev::Event| {
                                        let target: web_sys::HtmlInputElement = match ev.target() {
                                            Some(t) => t.unchecked_into(),
                                            None => return,
                                        };
                                        set_comment_text.set(target.value());
                                    }
                                />
                                <button
                                    type="submit"
                                    class="px-2 py-1 text-xs bg-blue-600 text-white rounded hover:bg-blue-700 transition-colors"
                                >
                                    "Send"
                                </button>
                            </form>
                         </div>
        </li>
    }
}

fn format_timestamp(timestamp: &str) -> String {
    if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(timestamp) {
        dt.format("%Y-%m-%d %H:%M").to_string()
    } else {
        timestamp.to_string()
    }
}
