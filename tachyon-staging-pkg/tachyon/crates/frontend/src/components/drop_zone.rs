#![allow(dead_code)]

use leptos::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{ClipboardEvent, DataTransfer, DragEvent, HtmlCanvasElement, ImageBitmap};

use crate::api::ApiClient;
use crate::components::upload_progress::{UploadItem, UploadProgress, UploadStatus};
use wasm_bindgen_futures::spawn_local;

const IMAGE_COMPRESS_THRESHOLD: u64 = 1024 * 1024;
const PROGRESS_DISMISS_MS: u32 = 3000;
const MAX_COMPRESS_WIDTH: u32 = 1920;
const MAX_COMPRESS_HEIGHT: u32 = 1080;

#[derive(Debug, Clone)]
pub struct DroppedFile {
    pub name: String,
    pub size: f64,
    pub content_type: String,
    pub file: web_sys::File,
}

fn is_image(content_type: &str) -> bool {
    content_type.starts_with("image/")
}

fn is_video(content_type: &str) -> bool {
    content_type.starts_with("video/")
}

fn blob_to_file(blob: &web_sys::Blob, name: &str) -> Result<web_sys::File, String> {
    let arr = js_sys::Array::new();
    arr.push(blob);
    web_sys::File::new_with_blob_sequence(&arr, name)
        .map_err(|e| format!("Failed to create File from Blob: {:?}", e))
}

async fn maybe_compress_image(file: &web_sys::File) -> Result<web_sys::Blob, String> {
    let ct = file.type_();
    let size = file.size() as u64;

    if !is_image(&ct) || size <= IMAGE_COMPRESS_THRESHOLD || ct == "image/svg+xml" {
        return Ok(file.clone().into());
    }

    let blob_ref: &web_sys::Blob = file;
    let window = web_sys::window().ok_or("No window")?;
    let bitmap_future = window
        .create_image_bitmap_with_blob(blob_ref)
        .map_err(|e| format!("createImageBitmap failed: {:?}", e))?;
    let bitmap: ImageBitmap = wasm_bindgen_futures::JsFuture::from(bitmap_future)
        .await
        .map_err(|e| format!("ImageBitmap promise failed: {:?}", e))?
        .dyn_into()
        .map_err(|_| "Failed to cast to ImageBitmap")?;

    let natural_w = bitmap.width();
    let natural_h = bitmap.height();
    let (w, h) = if natural_w > MAX_COMPRESS_WIDTH || natural_h > MAX_COMPRESS_HEIGHT {
        let scale = (MAX_COMPRESS_WIDTH as f64 / natural_w as f64)
            .min(MAX_COMPRESS_HEIGHT as f64 / natural_h as f64);
        (
            (natural_w as f64 * scale) as u32,
            (natural_h as f64 * scale) as u32,
        )
    } else {
        (natural_w, natural_h)
    };

    let canvas: HtmlCanvasElement = web_sys::window()
        .ok_or("No window")?
        .document()
        .ok_or("No document")?
        .create_element("canvas")
        .map_err(|e| format!("createElement failed: {:?}", e))?
        .dyn_into()
        .map_err(|_| "Failed to cast to HtmlCanvasElement")?;

    canvas.set_width(w);
    canvas.set_height(h);

    let ctx = canvas
        .get_context("2d")
        .map_err(|e| format!("getContext failed: {:?}", e))?
        .ok_or("No 2d context")?
        .dyn_into::<web_sys::CanvasRenderingContext2d>()
        .map_err(|_| "Failed to cast to CanvasRenderingContext2d")?;

    ctx.draw_image_with_image_bitmap_and_sw_and_sh_and_dx_and_dy_and_dw_and_dh(
        &bitmap,
        0.0,
        0.0,
        natural_w as f64,
        natural_h as f64,
        0.0,
        0.0,
        w as f64,
        h as f64,
    )
    .map_err(|e| format!("drawImage failed: {:?}", e))?;

    let blob_promise = js_sys::Promise::new(&mut |resolve, _reject| {
        canvas.to_blob_with_type(&resolve, "image/jpeg").ok();
    });

    let blob: web_sys::Blob = wasm_bindgen_futures::JsFuture::from(blob_promise)
        .await
        .map_err(|e| format!("toBlob promise failed: {:?}", e))?
        .dyn_into()
        .map_err(|_| "Failed to cast to Blob")?;

    Ok(blob)
}

#[component]
pub fn DropZone(
    #[prop(default = "Drop files here or click to browse".to_string())] label: String,
    #[prop(default = "*".to_string())] accept: String,
    #[prop(default = false)] multiple: bool,
    on_files: Callback<Vec<DroppedFile>>,
    #[prop(default = false)] disabled: bool,
    #[prop(optional)] editor_insert: Option<Callback<String>>,
) -> impl IntoView {
    let (is_dragging, set_is_dragging) = signal(false);
    let (uploads, set_uploads) = signal(Vec::<UploadItem>::new());
    let input_ref = NodeRef::<leptos::html::Input>::new();
    let input_ref_key = input_ref;

    let handle_drag_over = move |ev: DragEvent| {
        ev.prevent_default();
        ev.stop_propagation();
        set_is_dragging.set(true);
    };

    let handle_drag_leave = move |ev: DragEvent| {
        ev.prevent_default();
        ev.stop_propagation();
        set_is_dragging.set(false);
    };

    let handle_drop = move |ev: DragEvent| {
        ev.prevent_default();
        ev.stop_propagation();
        set_is_dragging.set(false);

        if let Some(data_transfer) = ev.data_transfer() {
            let files = extract_files(&data_transfer);
            if !files.is_empty() {
                on_files.run(files);
            }
        }
    };

    let input_ref_click = input_ref;
    let handle_click = move |_: web_sys::MouseEvent| {
        if let Some(input) = input_ref_click.get() {
            input.click();
        }
    };

    let handle_change = move |ev: web_sys::Event| {
        let input: web_sys::HtmlInputElement = ev
            .target()
            .and_then(|t| t.dyn_into().ok())
            .expect("Event target should be input");

        let files = extract_files_from_input(&input);
        if !files.is_empty() {
            on_files.run(files);
        }
    };

    let paste_insert = editor_insert;
    let handle_paste = move |ev: ClipboardEvent| {
        let clipboard = match ev.clipboard_data() {
            Some(data) => data,
            None => return,
        };

        let files_result = clipboard.files();
        let file_list = match files_result {
            Some(fl) => fl,
            None => return,
        };
        if file_list.length() > 0 {
            ev.prevent_default();
            for i in 0..file_list.length() {
                if let Some(file) = file_list.get(i) {
                    let content_type = file.type_();
                    let name = file.name();
                    let file_clone = file.clone();
                    let insert_cb = paste_insert;

                    let item_id = format!("paste-{}", js_sys::Date::now() as u64);
                    let item = UploadItem {
                        id: item_id.clone(),
                        filename: name.clone(),
                        size: file.size(),
                        status: UploadStatus::Pending,
                        progress: 0.0,
                        error: None,
                        url: None,
                    };
                    set_uploads.update(|u| u.push(item));

                    let api = ApiClient::default();
                    let set_uploads_inner = set_uploads;
                    let item_id_inner = item_id.clone();
                    let ct = content_type.clone();

                    spawn_local(async move {
                        set_uploads_inner.update(|u| {
                            if let Some(item) = u.iter_mut().find(|i| i.id == item_id_inner) {
                                item.status = UploadStatus::Uploading;
                                item.progress = 10.0;
                            }
                        });

                        let upload_file = match maybe_compress_image(&file_clone).await {
                            Ok(blob) => {
                                set_uploads_inner.update(|u| {
                                    if let Some(item) = u.iter_mut().find(|i| i.id == item_id_inner)
                                    {
                                        item.progress = 25.0;
                                    }
                                });
                                blob_to_file(&blob, &name).unwrap_or(file_clone)
                            }
                            Err(_) => file_clone,
                        };

                        set_uploads_inner.update(|u| {
                            if let Some(item) = u.iter_mut().find(|i| i.id == item_id_inner) {
                                item.progress = 40.0;
                            }
                        });

                        match api.upload_file(&upload_file).await {
                            Ok(response) => {
                                let url = response.url.clone();
                                set_uploads_inner.update(|u| {
                                    if let Some(item) = u.iter_mut().find(|i| i.id == item_id_inner)
                                    {
                                        item.status = UploadStatus::Complete;
                                        item.progress = 100.0;
                                        item.url = Some(url.clone());
                                    }
                                });

                                let md = if is_image(&ct) || is_video(&ct) {
                                    format!("![{}]({})", name, url)
                                } else {
                                    format!("[{}]({})", name, url)
                                };
                                if let Some(cb) = insert_cb {
                                    cb.run(md);
                                }

                                let id_clone = item_id_inner.clone();
                                let dismiss = set_uploads_inner;
                                wasm_bindgen_futures::spawn_local(async move {
                                    gloo_timers::future::TimeoutFuture::new(PROGRESS_DISMISS_MS)
                                        .await;
                                    dismiss.update(|u| u.retain(|i| i.id != id_clone));
                                });
                            }
                            Err(e) => {
                                set_uploads_inner.update(|u| {
                                    if let Some(item) = u.iter_mut().find(|i| i.id == item_id_inner)
                                    {
                                        item.status = UploadStatus::Error;
                                        item.error = Some(e.to_string());
                                    }
                                });
                                let id_clone = item_id_inner.clone();
                                let dismiss = set_uploads_inner;
                                wasm_bindgen_futures::spawn_local(async move {
                                    gloo_timers::future::TimeoutFuture::new(PROGRESS_DISMISS_MS)
                                        .await;
                                    dismiss.update(|u| u.retain(|i| i.id != id_clone));
                                });
                            }
                        }
                    });
                }
            }
        }
    };

    let border_class = move || {
        if is_dragging.get() {
            "border-blue-500 bg-blue-50 dark:bg-blue-900/20"
        } else {
            "border-gray-300 dark:border-gray-600 hover:border-gray-400 dark:hover:border-gray-500"
        }
    };

    view! {
        <div class="drop-zone-wrapper">
            <div
                class=move || format!(
                    "border-2 border-dashed rounded-none p-8 text-center cursor-pointer transition-colors {}",
                    border_class()
                )
                on:dragover=handle_drag_over
                on:dragleave=handle_drag_leave
                on:drop=handle_drop
                on:click=handle_click
                on:paste=handle_paste
                role="button"
                aria-label="File upload area"
                tabindex=0
                on:keydown=move |ev: web_sys::KeyboardEvent| {
                    if ev.key() == "Enter" || ev.key() == " " {
                        ev.prevent_default();
                        if let Some(input) = input_ref_key.get() {
                            input.click();
                        }
                    }
                }
            >
                <input
                    node_ref=input_ref
                    type="file"
                    accept=accept
                    multiple=multiple
                    class="hidden"
                    on:change=handle_change
                    disabled=disabled
                    aria-hidden="true"
                />
                <div class="space-y-2">
                    <div class="text-4xl text-gray-400">
                        <svg xmlns="http://www.w3.org/2000/svg" class="h-12 w-12 mx-auto text-gray-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M7 16a4 4 0 01-.88-7.903A5 5 0 1115.9 6L16 6a5 5 0 011 9.9M15 13l-3-3m0 0l-3 3m3-3v12" />
                        </svg>
                    </div>
                    <p class="text-sm text-gray-600 dark:text-gray-400">{label}</p>
                    <p class="text-xs text-gray-500 dark:text-gray-500">
                        "Drag and drop, click to browse, or Ctrl+V to paste"
                    </p>
                </div>
            </div>
            {move || {
                let items = uploads.get();
                if items.is_empty() {
                    return ().into_any();
                }
                view! {
                    <div class="mt-2">
                        <UploadProgress uploads={Signal::derive(move || uploads.get())} />
                    </div>
                }.into_any()
            }}
        </div>
    }
}

/// Editor-aware drop zone that handles file drops and inserts markdown.
#[component]
pub fn EditorDropZone<T: leptos::IntoView + 'static>(
    editor_insert: Callback<String>,
    children: T,
) -> impl IntoView {
    let (is_dragging, set_is_dragging) = signal(false);
    let (uploads, set_uploads) = signal(Vec::<UploadItem>::new());

    let insert_cb = editor_insert;

    let handle_drag_over = move |ev: DragEvent| {
        ev.prevent_default();
        ev.stop_propagation();
        set_is_dragging.set(true);
    };

    let handle_drag_leave = move |ev: DragEvent| {
        ev.prevent_default();
        ev.stop_propagation();
        set_is_dragging.set(false);
    };

    let handle_drop = {
        move |ev: DragEvent| {
            ev.prevent_default();
            ev.stop_propagation();
            set_is_dragging.set(false);

            if let Some(data_transfer) = ev.data_transfer() {
                let files = extract_files(&data_transfer);
                for file in files {
                    let name = file.name.clone();
                    let ct = file.content_type.clone();
                    let web_file = file.file.clone();
                    let item_id = format!("drop-{}", js_sys::Date::now() as u64);

                    set_uploads.update(|u| {
                        u.push(UploadItem {
                            id: item_id.clone(),
                            filename: name.clone(),
                            size: file.size,
                            status: UploadStatus::Uploading,
                            progress: 10.0,
                            error: None,
                            url: None,
                        });
                    });

                    let api = ApiClient::default();
                    let set_uploads_inner = set_uploads;
                    let insert_cb_inner = insert_cb;
                    let item_id_inner = item_id;
                    let name_inner = name;
                    let ct_inner = ct;

                    spawn_local(async move {
                        let upload_file = match maybe_compress_image(&web_file).await {
                            Ok(blob) => {
                                set_uploads_inner.update(|u| {
                                    if let Some(item) = u.iter_mut().find(|i| i.id == item_id_inner)
                                    {
                                        item.progress = 25.0;
                                    }
                                });
                                blob_to_file(&blob, &name_inner).unwrap_or(web_file)
                            }
                            Err(_) => web_file,
                        };

                        set_uploads_inner.update(|u| {
                            if let Some(item) = u.iter_mut().find(|i| i.id == item_id_inner) {
                                item.progress = 40.0;
                            }
                        });

                        match api.upload_file(&upload_file).await {
                            Ok(response) => {
                                let url = response.url.clone();
                                set_uploads_inner.update(|u| {
                                    if let Some(item) = u.iter_mut().find(|i| i.id == item_id_inner)
                                    {
                                        item.status = UploadStatus::Complete;
                                        item.progress = 100.0;
                                        item.url = Some(url.clone());
                                    }
                                });

                                let md = if is_image(&ct_inner) || is_video(&ct_inner) {
                                    format!("![{}]({})", name_inner, url)
                                } else {
                                    format!("[{}]({})", name_inner, url)
                                };
                                insert_cb_inner.run(md);

                                let id_clone = item_id_inner.clone();
                                let dismiss = set_uploads_inner;
                                wasm_bindgen_futures::spawn_local(async move {
                                    gloo_timers::future::TimeoutFuture::new(PROGRESS_DISMISS_MS)
                                        .await;
                                    dismiss.update(|u| u.retain(|i| i.id != id_clone));
                                });
                            }
                            Err(e) => {
                                set_uploads_inner.update(|u| {
                                    if let Some(item) = u.iter_mut().find(|i| i.id == item_id_inner)
                                    {
                                        item.status = UploadStatus::Error;
                                        item.error = Some(e.to_string());
                                    }
                                });
                                let id_clone = item_id_inner.clone();
                                let dismiss = set_uploads_inner;
                                wasm_bindgen_futures::spawn_local(async move {
                                    gloo_timers::future::TimeoutFuture::new(PROGRESS_DISMISS_MS)
                                        .await;
                                    dismiss.update(|u| u.retain(|i| i.id != id_clone));
                                });
                            }
                        }
                    });
                }
            }
        }
    };

    let handle_paste = {
        move |ev: ClipboardEvent| {
            let clipboard = match ev.clipboard_data() {
                Some(data) => data,
                None => return,
            };

            let files_opt = clipboard.files();
            let files = match files_opt {
                Some(fl) => fl,
                None => return,
            };
            if files.length() == 0 {
                return;
            }
            ev.prevent_default();

            for i in 0..files.length() {
                if let Some(file) = files.get(i) {
                    let ct = file.type_();
                    let name = file.name();
                    let item_id = format!("paste-{}", js_sys::Date::now() as u64);

                    set_uploads.update(|u| {
                        u.push(UploadItem {
                            id: item_id.clone(),
                            filename: name.clone(),
                            size: file.size(),
                            status: UploadStatus::Uploading,
                            progress: 10.0,
                            error: None,
                            url: None,
                        });
                    });

                    let api = ApiClient::default();
                    let set_uploads_inner = set_uploads;
                    let insert_cb_inner = insert_cb;
                    let item_id_inner = item_id;
                    let name_inner = name;
                    let ct_inner = ct;

                    spawn_local(async move {
                        let upload_file = match maybe_compress_image(&file).await {
                            Ok(blob) => {
                                set_uploads_inner.update(|u| {
                                    if let Some(item) = u.iter_mut().find(|i| i.id == item_id_inner)
                                    {
                                        item.progress = 25.0;
                                    }
                                });
                                blob_to_file(&blob, &name_inner).unwrap_or(file)
                            }
                            Err(_) => file,
                        };

                        set_uploads_inner.update(|u| {
                            if let Some(item) = u.iter_mut().find(|i| i.id == item_id_inner) {
                                item.progress = 40.0;
                            }
                        });

                        match api.upload_file(&upload_file).await {
                            Ok(response) => {
                                let url = response.url.clone();
                                set_uploads_inner.update(|u| {
                                    if let Some(item) = u.iter_mut().find(|i| i.id == item_id_inner)
                                    {
                                        item.status = UploadStatus::Complete;
                                        item.progress = 100.0;
                                        item.url = Some(url.clone());
                                    }
                                });

                                let md = if is_image(&ct_inner) || is_video(&ct_inner) {
                                    format!("![{}]({})", name_inner, url)
                                } else {
                                    format!("[{}]({})", name_inner, url)
                                };
                                insert_cb_inner.run(md);

                                let id_clone = item_id_inner.clone();
                                let dismiss = set_uploads_inner;
                                wasm_bindgen_futures::spawn_local(async move {
                                    gloo_timers::future::TimeoutFuture::new(PROGRESS_DISMISS_MS)
                                        .await;
                                    dismiss.update(|u| u.retain(|i| i.id != id_clone));
                                });
                            }
                            Err(e) => {
                                set_uploads_inner.update(|u| {
                                    if let Some(item) = u.iter_mut().find(|i| i.id == item_id_inner)
                                    {
                                        item.status = UploadStatus::Error;
                                        item.error = Some(e.to_string());
                                    }
                                });
                                let id_clone = item_id_inner.clone();
                                let dismiss = set_uploads_inner;
                                wasm_bindgen_futures::spawn_local(async move {
                                    gloo_timers::future::TimeoutFuture::new(PROGRESS_DISMISS_MS)
                                        .await;
                                    dismiss.update(|u| u.retain(|i| i.id != id_clone));
                                });
                            }
                        }
                    });
                }
            }
        }
    };

    view! {
        <div
            class=move || format!(
                "editor-drop-zone relative {}",
                if is_dragging.get() {
                    "ring-2 ring-blue-500 ring-offset-2"
                } else {
                    ""
                }
            )
            on:dragover=handle_drag_over
            on:dragleave=handle_drag_leave
            on:drop=handle_drop
            on:paste=handle_paste
        >
            {children}
            {move || {
                let items = uploads.get();
                if items.is_empty() {
                    return ().into_any();
                }
                view! {
                    <div class="absolute bottom-2 right-2 z-50 w-72">
                        <UploadProgress uploads={Signal::derive(move || uploads.get())} />
                    </div>
                }.into_any()
            }}
            {move || {
                if is_dragging.get() {
                    view! {
                        <div class="absolute inset-0 bg-blue-500/10 border-2 border-dashed border-blue-500 rounded flex items-center justify-center pointer-events-none z-40">
                            <span class="text-blue-600 dark:text-blue-400 font-medium text-sm">
                                "Drop files to upload"
                            </span>
                        </div>
                    }.into_any()
                } else {
                    ().into_any()
                }
            }}
        </div>
    }
}

fn extract_files(data_transfer: &DataTransfer) -> Vec<DroppedFile> {
    let file_list = match data_transfer.files() {
        Some(files) => files,
        None => return vec![],
    };

    let mut files = vec![];
    for i in 0..file_list.length() {
        if let Some(file) = file_list.get(i) {
            files.push(DroppedFile {
                name: file.name(),
                size: file.size(),
                content_type: file.type_(),
                file,
            });
        }
    }
    files
}

fn extract_files_from_input(input: &web_sys::HtmlInputElement) -> Vec<DroppedFile> {
    let file_list = match input.files() {
        Some(files) => files,
        None => return vec![],
    };

    let mut files = vec![];
    for i in 0..file_list.length() {
        if let Some(file) = file_list.get(i) {
            files.push(DroppedFile {
                name: file.name(),
                size: file.size(),
                content_type: file.type_(),
                file,
            });
        }
    }
    files
}
