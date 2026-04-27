#![allow(dead_code)]

use leptos::prelude::*;

#[derive(Debug, Clone)]
pub struct UploadItem {
    pub id: String,
    pub filename: String,
    pub size: f64,
    pub status: UploadStatus,
    pub progress: f64,
    pub error: Option<String>,
    pub url: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UploadStatus {
    Pending,
    Uploading,
    Complete,
    Error,
}

impl UploadStatus {
    pub fn label(&self) -> &'static str {
        match self {
            UploadStatus::Pending => "Pending",
            UploadStatus::Uploading => "Uploading...",
            UploadStatus::Complete => "Complete",
            UploadStatus::Error => "Failed",
        }
    }
}

#[component]
pub fn UploadProgress(uploads: Signal<Vec<UploadItem>>) -> impl IntoView {
    view! {
        <div class="space-y-2" role="list" aria-label="Upload progress">
            <For
                each=move || uploads.get()
                key=|item| item.id.clone()
                let:item
            >
                {move || {
                    let status_class = match &item.status {
                        UploadStatus::Uploading => "text-blue-600 dark:text-blue-400",
                        UploadStatus::Complete => "text-green-600 dark:text-green-400",
                        UploadStatus::Error => "text-red-600 dark:text-red-400",
                        UploadStatus::Pending => "text-gray-500",
                    };

                    let is_uploading = item.status == UploadStatus::Uploading;
                    let is_complete = item.status == UploadStatus::Complete;
                    let is_error = item.status == UploadStatus::Error;
                    let filename = item.filename.clone();

                    view! {
                        <div class="flex items-center gap-3 p-2 bg-white dark:bg-gray-800 rounded border border-gray-200 dark:border-gray-700" role="listitem">
                            <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5 text-gray-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
                            </svg>

                            <div class="flex-1 min-w-0">
                                <div class="text-sm font-medium truncate">{filename}</div>
                                <div class=format!("text-xs text-gray-500 {} — {}", status_class, item.status.label())>
                                    {format_size(item.size)}
                                </div>
                                <Show when=move || is_uploading>
                                    <div class="mt-1 w-full bg-gray-200 dark:bg-gray-700 rounded-full h-1.5">
                                        <div
                                            class="bg-blue-600 h-1.5 rounded-full transition-all duration-300"
                                            style=format!("width: {}%", item.progress)
                                            role="progressbar"
                                            aria-valuenow=item.progress as i32
                                            aria-valuemin=0
                                            aria-valuemax=100
                                        ></div>
                                    </div>
                                </Show>
                            </div>

                            <Show when=move || is_complete>
                                <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5 text-green-500" viewBox="0 0 20 20" fill="currentColor">
                                    <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z" clip-rule="evenodd" />
                                </svg>
                            </Show>

                            <Show when=move || is_error>
                                <span class="text-xs text-red-500" title="Upload failed">
                                    "Error"
                                </span>
                            </Show>
                        </div>
                    }
                }}
            </For>
        </div>
    }
}

fn format_size(bytes: f64) -> String {
    if bytes < 1024.0 {
        format!("{} B", bytes)
    } else if bytes < 1024.0 * 1024.0 {
        format!("{:.1} KB", bytes / 1024.0)
    } else if bytes < 1024.0 * 1024.0 * 1024.0 {
        format!("{:.1} MB", bytes / (1024.0 * 1024.0))
    } else {
        format!("{:.1} GB", bytes / (1024.0 * 1024.0 * 1024.0))
    }
}
