#![allow(dead_code, clippy::redundant_locals)]

use crate::api::ApiClient;
use crate::types::Attachment;
use leptos::prelude::*;
use std::sync::{Arc, Mutex};
use wasm_bindgen_futures::spawn_local;

#[component]
pub fn AttachmentManager(document_id: String) -> impl IntoView {
    let api_client = Arc::new(Mutex::new(ApiClient::default()));
    let (uploading, set_uploading) = signal(false);
    let (error_msg, set_error_msg) = signal(None::<String>);
    let (refresh_counter, set_refresh_counter) = signal(0u32);
    let (is_dragging, set_is_dragging) = signal(false);

    let file_input_ref = NodeRef::<leptos::html::Input>::new();

    let attachments_resource = LocalResource::new({
        let api_client = api_client.clone();
        let document_id = document_id.clone();
        move || {
            let _ = refresh_counter.get();
            let client = api_client.lock().unwrap_or_else(|e| e.into_inner()).clone();
            let doc_id = document_id.clone();
            async move { client.list_attachments(&doc_id).await.unwrap_or_default() }
        }
    });

    let upload_single_file = {
        let api_client = api_client.clone();
        let document_id = document_id.clone();
        move |file: web_sys::File| {
            let doc_id = document_id.clone();
            let client = api_client.lock().unwrap_or_else(|e| e.into_inner()).clone();

            set_uploading.set(true);
            set_error_msg.set(None);

            spawn_local(async move {
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
            });
        }
    };

    let _on_file_upload = {
        let file_input_ref = file_input_ref;
        let upload_single_file = upload_single_file.clone();
        move || {
            let input = file_input_ref.get();
            if let Some(input) = input {
                if let Some(files) = input.files() {
                    if let Some(file) = files.item(0) {
                        upload_single_file(file);
                    } else {
                        set_error_msg.set(Some("No file selected".to_string()));
                    }
                }
            } else {
                set_error_msg.set(Some("File input not found".to_string()));
            }
        }
    };

    let open_file_picker = {
        let file_input_ref = file_input_ref;
        move |_| {
            if let Some(input) = file_input_ref.get() {
                input.click();
            }
        }
    };

    let on_dragover = move |ev: web_sys::DragEvent| {
        ev.prevent_default();
        ev.stop_propagation();
        set_is_dragging.set(true);
    };

    let on_dragleave = move |ev: web_sys::DragEvent| {
        ev.prevent_default();
        set_is_dragging.set(false);
    };

    let on_drop = {
        let upload_single_file = upload_single_file.clone();
        move |ev: web_sys::DragEvent| {
            ev.prevent_default();
            ev.stop_propagation();
            set_is_dragging.set(false);
            if let Some(data_transfer) = ev.data_transfer() {
                if let Some(files) = data_transfer.files() {
                    let file_count = files.length();
                    if file_count > 0 {
                        if let Some(file) = files.item(0) {
                            upload_single_file(file);
                        }
                    }
                }
            }
        }
    };

    let on_delete = {
        let api_client = api_client.clone();
        let document_id = document_id.clone();
        move |attachment_id: String| {
            let client = api_client.lock().unwrap_or_else(|e| e.into_inner()).clone();
            let doc_id = document_id.clone();
            spawn_local(async move {
                let _ = client.delete_attachment(&doc_id, &attachment_id).await;
                set_refresh_counter.update(|n| *n += 1);
            });
        }
    };

    view! {
        <div class="bg-white dark:bg-gray-800 rounded-none shadow border-2 border-gray-900 dark:border-gray-100">
            <div class="p-4 border-b border-gray-200 dark:border-gray-700">
                <h3 class="text-lg font-semibold text-gray-900 dark:text-white">"Attachments"</h3>
            </div>

            <div class="p-4">
                <div
                    class={move || {
                        if is_dragging.get() {
                            "border-2 border-dashed border-blue-500 bg-blue-50 dark:bg-blue-900/20 rounded-none p-6 text-center transition-colors"
                        } else {
                            "border-2 border-dashed border-gray-300 dark:border-gray-600 bg-gray-50 dark:bg-gray-700/50 rounded-none p-6 text-center transition-colors hover:border-gray-400 dark:hover:border-gray-500"
                        }
                    }}
                    on:dragover=on_dragover
                    on:dragleave=on_dragleave
                    on:drop=on_drop
                    on:click=open_file_picker
                >
                    <input
                        node_ref=file_input_ref
                        type="file"
                        class="hidden"
                    ></input>
                    <svg class="mx-auto h-10 w-10 text-gray-400 dark:text-gray-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M7 16a4 4 0 01-.88-7.903A5 5 0 1115.9 6L16 6a5 5 0 011 9.9M15 13l-3-3m0 0l-3 3m3-3v12"></path>
                    </svg>
                    <p class="mt-2 text-sm text-gray-600 dark:text-gray-400">
                        {move || if is_dragging.get() { "Drop file here..." } else { "Drop files here or click to upload" }}
                    </p>
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

                {move || if uploading.get() {
                    view! {
                        <div class="mt-3 flex items-center gap-2 text-sm text-blue-600 dark:text-blue-400">
                            <div class="w-4 h-4 border-2 border-blue-400 border-t-transparent rounded-full animate-spin"></div>
                            "Uploading..."
                        </div>
                    }.into_any()
                } else {
                    ().into_any()
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
