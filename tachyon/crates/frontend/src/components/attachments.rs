#![allow(dead_code)]

use leptos::prelude::*;
use crate::api::ApiClient;
use crate::types::Attachment;
use std::sync::{Arc, Mutex};

#[component]
pub fn AttachmentManager(
    document_id: String,
) -> impl IntoView {
    let api_client = Arc::new(Mutex::new(ApiClient::default()));
    let (uploading, set_uploading) = signal(false);
    let (error_msg, set_error_msg) = signal(None::<String>);
    let (refresh_counter, set_refresh_counter) = signal(0u32);

    let file_input_ref = NodeRef::<leptos::html::Input>::new();

    let attachments_resource = LocalResource::new({
        let api_client = api_client.clone();
        let document_id = document_id.clone();
        move || {
            let _ = refresh_counter.get();
            let client = api_client.lock().unwrap().clone();
            let doc_id = document_id.clone();
            async move {
                client.list_attachments(&doc_id).await.unwrap_or_default()
            }
        }
    });

    let on_file_upload = {
        let api_client = api_client.clone();
        let document_id = document_id.clone();
        let file_input_ref = file_input_ref;
        move || {
            let doc_id = document_id.clone();
            let client = api_client.lock().unwrap().clone();
            let input = file_input_ref.get();

            set_uploading.set(true);
            set_error_msg.set(None);

            wasm_bindgen_futures::spawn_local(async move {
                if let Some(input) = input {
                    if let Some(files) = input.files() {
                        if let Some(file) = files.item(0) {
                            match client.upload_attachment(&doc_id, &file).await {
                                Ok(_) => {
                                    set_uploading.set(false);
                                    set_refresh_counter.update(|n| *n += 1);
                                }
                                Err(e) => {
                                    set_error_msg.set(Some(e.to_string()));
                                    set_uploading.set(false);
                                }
                            }
                        } else {
                            set_error_msg.set(Some("No file selected".to_string()));
                            set_uploading.set(false);
                        }
                    }
                } else {
                    set_error_msg.set(Some("File input not found".to_string()));
                    set_uploading.set(false);
                }
            });
        }
    };

    let on_delete = {
        let api_client = api_client.clone();
        let document_id = document_id.clone();
        move |attachment_id: String| {
            let client = api_client.lock().unwrap().clone();
            let doc_id = document_id.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let _ = client.delete_attachment(&doc_id, &attachment_id).await;
                set_refresh_counter.update(|n| *n += 1);
            });
        }
    };

    view! {
        <div class="bg-white dark:bg-gray-800 rounded-lg shadow border border-gray-200 dark:border-gray-700">
            <div class="p-4 border-b border-gray-200 dark:border-gray-700">
                <h3 class="text-lg font-semibold text-gray-900 dark:text-white">"Attachments"</h3>
            </div>

            <div class="p-4">
                <div class="flex items-center gap-3">
                    <input
                        node_ref=file_input_ref
                        type="file"
                        class="block w-full text-sm text-gray-500
                               file:mr-4 file:py-2 file:px-4
                               file:rounded file:border-0
                               file:text-sm file:font-semibold
                               file:bg-blue-50 file:text-blue-700
                               hover:file:bg-blue-100
                               dark:file:bg-blue-900 dark:file:text-blue-300"
                    />
                    <button
                        type="button"
                        disabled=move || uploading.get()
                        class="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 
                               disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
                        on:click=move |_| on_file_upload()
                    >
                        {move || if uploading.get() { "Uploading..." } else { "Upload" }}
                    </button>
                </div>

                {move || {
                    error_msg.get().map(|msg| {
                        view! {
                            <div class="mt-3 p-2 bg-red-100 dark:bg-red-900 text-red-700 dark:text-red-300 rounded text-sm">
                                {msg}
                            </div>
                        }
                    })
                }}
            </div>

            <Suspense fallback=view! { <div class="p-4">"Loading attachments..."</div> }>
                <AttachmentList
                    document_id=document_id.clone()
                    on_delete=on_delete
                    attachments_resource=attachments_resource
                />
            </Suspense>
        </div>
    }
}

#[component]
fn AttachmentList(
    document_id: String,
    on_delete: impl Fn(String) + 'static + Send + Sync + Clone,
    attachments_resource: LocalResource<Vec<Attachment>>,
) -> impl IntoView {
    move || {
        let doc_id = document_id.clone();
        let on_delete_clone = on_delete.clone();
        attachments_resource.get().map(|attachments| {
            if attachments.is_empty() {
                view! {
                    <div class="px-4 pb-4 text-gray-500 dark:text-gray-400 text-center">
                        "No attachments"
                    </div>
                }.into_any()
            } else {
                view! {
                    <ul class="divide-y divide-gray-200 dark:divide-gray-700">
                        {attachments.into_iter().map(|attachment| {
                            let attachment_id = attachment.id.clone();
                            let filename = attachment.filename.clone();
                            let size = format_size(attachment.size);
                            let download_url = format!("/api/v1/documents/{}/attachments/{}", doc_id, attachment_id);
                            let aid_for_delete = attachment_id.clone();
                            let on_delete_for_item = on_delete_clone.clone();

                            view! {
                                <li class="p-4 flex items-center justify-between hover:bg-gray-50 dark:hover:bg-gray-700">
                                    <div class="flex items-center gap-3">
                                        <svg class="w-5 h-5 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"></path>
                                        </svg>
                                        <div>
                                            <p class="font-medium text-gray-900 dark:text-white">{filename}</p>
                                            <p class="text-sm text-gray-500 dark:text-gray-400">{size}</p>
                                        </div>
                                    </div>
                                    <div class="flex items-center gap-2">
                                        <a
                                            href=download_url
                                            class="px-3 py-1 text-sm text-blue-600 dark:text-blue-400 hover:underline"
                                            download
                                        >
                                            "Download"
                                        </a>
                                        <button
                                            class="px-3 py-1 text-sm text-red-600 dark:text-red-400 hover:underline"
                                            on:click=move |_| on_delete_for_item(aid_for_delete.clone())
                                        >
                                            "Delete"
                                        </button>
                                    </div>
                                </li>
                            }
                        }).collect::<Vec<_>>()}
                    </ul>
                }.into_any()
            }
        })
    }
}

fn format_size(bytes: i64) -> String {
    const KB: i64 = 1024;
    const MB: i64 = 1024 * KB;
    const GB: i64 = 1024 * MB;

    if bytes >= GB {
        format!("{:.1} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}
