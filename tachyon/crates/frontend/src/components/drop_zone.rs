#![allow(dead_code)]

use leptos::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{DataTransfer, DragEvent};

#[derive(Debug, Clone)]
pub struct DroppedFile {
    pub name: String,
    pub size: f64,
    pub content_type: String,
    pub file: web_sys::File,
}

#[component]
pub fn DropZone(
    #[prop(default = "Drop files here or click to browse".to_string())]
    label: String,
    #[prop(default = "*".to_string())]
    accept: String,
    #[prop(default = false)]
    multiple: bool,
    on_files: Callback<Vec<DroppedFile>>,
    #[prop(default = false)]
    disabled: bool,
) -> impl IntoView {
    let (is_dragging, set_is_dragging) = signal(false);
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

    let border_class = move || {
        if is_dragging.get() {
            "border-blue-500 bg-blue-50 dark:bg-blue-900/20"
        } else {
            "border-gray-300 dark:border-gray-600 hover:border-gray-400 dark:hover:border-gray-500"
        }
    };

    view! {
        <div
            class=move || format!(
                "border-2 border-dashed rounded-lg p-8 text-center cursor-pointer transition-colors {}",
                border_class()
            )
            on:dragover=handle_drag_over
            on:dragleave=handle_drag_leave
            on:drop=handle_drop
            on:click=handle_click
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
                    "Drag and drop or click to browse"
                </p>
            </div>
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
