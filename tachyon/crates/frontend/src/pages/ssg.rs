use crate::api::ApiClient;
use crate::types::{SsgBuildRequest, SsgBuildResponse};
use leptos::prelude::*;
use wasm_bindgen_futures::spawn_local;

#[component]
pub fn SsgPage() -> impl IntoView {
    let (title, set_title) = signal("Tachyon Docs".to_string());
    let (description, set_description) = signal("A knowledge base built with Tachyon".to_string());
    let (base_url, set_base_url) = signal("https://docs.example.com".to_string());
    let (theme, set_theme) = signal("auto".to_string());
    let (building, set_building) = signal(false);
    let (build_result, set_build_result) = signal(None::<SsgBuildResponse>);
    let (error, set_error) = signal(None::<String>);

    let handle_build = move |_| {
        set_error.set(None);
        set_building.set(true);
        set_build_result.set(None);

        let req = SsgBuildRequest {
            title: Some(title.get()),
            description: Some(description.get()),
            base_url: Some(base_url.get()),
            theme: Some(theme.get()),
            custom_css: None,
            nav_links: None,
            group_by_tag: None,
            project_id: None,
            limit: None,
        };

        spawn_local(async move {
            let api = ApiClient::default();
            match api.build_site(&req).await {
                Ok(resp) => set_build_result.set(Some(resp)),
                Err(e) => set_error.set(Some(format!("Build failed: {}", e))),
            }
            set_building.set(false);
        });
    };

    let handle_download = move |_| {
        let api = ApiClient::default();
        spawn_local(async move {
            if let Err(e) = api.download_ssg_build().await {
                set_error.set(Some(format!("Download failed: {}", e)));
            }
        });
    };

    let size_str = move || {
        build_result.get().map(|r| {
            let bytes = r.result.output_size_bytes;
            if bytes < 1024 {
                format!("{} B", bytes)
            } else if bytes < 1024 * 1024 {
                format!("{:.1} KB", bytes as f64 / 1024.0)
            } else {
                format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
            }
        })
    };

    let error_view = move || {
        error.get().map(|e| {
            view! {
                <div class="p-3 bg-red-50 dark:bg-red-900/30 border border-red-200 dark:border-red-800 
                            text-red-700 dark:text-red-300 text-sm rounded-lg">{e}</div>
            }
        })
    };

    let result_view = move || {
        build_result.get().map(|r| {
            let size = size_str().unwrap_or_default();
            view! {
                <div class="mt-6 bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 p-6">
                    <h3 class="text-lg font-semibold text-gray-900 dark:text-white mb-4">
                        "Build Successful"
                    </h3>
                    <div class="grid grid-cols-2 md:grid-cols-4 gap-4">
                        <div>
                            <p class="text-sm text-gray-500 dark:text-gray-400">"Pages"</p>
                            <p class="text-xl font-bold text-gray-900 dark:text-white">{r.result.pages}</p>
                        </div>
                        <div>
                            <p class="text-sm text-gray-500 dark:text-gray-400">"Category Pages"</p>
                            <p class="text-xl font-bold text-gray-900 dark:text-white">{r.result.category_pages}</p>
                        </div>
                        <div>
                            <p class="text-sm text-gray-500 dark:text-gray-400">"Total Files"</p>
                            <p class="text-xl font-bold text-gray-900 dark:text-white">{r.result.total_files}</p>
                        </div>
                        <div>
                            <p class="text-sm text-gray-500 dark:text-gray-400">"Build Time"</p>
                            <p class="text-xl font-bold text-gray-900 dark:text-white">{format!("{}ms", r.result.build_time_ms)}</p>
                        </div>
                    </div>
                    <div class="mt-4 flex items-center justify-between">
                        <span class="text-sm text-gray-500 dark:text-gray-400">{format!("Output size: {}", size)}</span>
                        <div class="flex gap-3">
                            <button
                                class="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors 
                                       flex items-center gap-2"
                                on:click={handle_download}
                            >
                                <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                                          d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4" />
                                </svg>
                                "Download ZIP"
                            </button>
                        </div>
                    </div>
                </div>
            }
        })
    };

    let btn_label = move || {
        if building.get() {
            "Building..."
        } else {
            "Build Site"
        }
    };
    let btn_class = move || {
        format!(
        "px-6 py-3 text-white rounded-lg font-medium transition-colors flex items-center gap-2 {}",
        if building.get() { "bg-blue-400 cursor-not-allowed" } else { "bg-blue-600 hover:bg-blue-700" }
    )
    };

    view! {
        <div>
            <div class="flex items-center justify-between mb-6">
                <div>
                    <h1 class="text-2xl font-bold text-gray-900 dark:text-white">"Static Site Generator"</h1>
                    <p class="mt-1 text-sm text-gray-500 dark:text-gray-400">
                        "Generate a static site from your published documents"
                    </p>
                </div>
            </div>

            {error_view}

            <div class="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 p-6">
                <h2 class="text-lg font-semibold text-gray-900 dark:text-white mb-4">"Site Configuration"</h2>

                <div class="space-y-4">
                    <div>
                        <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                            "Site Title"
                        </label>
                        <input type="text"
                            class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg
                                   bg-white dark:bg-gray-700 text-gray-900 dark:text-white 
                                   focus:ring-2 focus:ring-blue-500 outline-none"
                            prop:value={title.get()}
                            on:input={move |ev| set_title.set(event_target_value(&ev))} />
                    </div>

                    <div>
                        <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                            "Description"
                        </label>
                        <input type="text"
                            class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg
                                   bg-white dark:bg-gray-700 text-gray-900 dark:text-white 
                                   focus:ring-2 focus:ring-blue-500 outline-none"
                            prop:value={description.get()}
                            on:input={move |ev| set_description.set(event_target_value(&ev))} />
                    </div>

                    <div>
                        <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                            "Base URL"
                        </label>
                        <input type="url"
                            class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg
                                   bg-white dark:bg-gray-700 text-gray-900 dark:text-white 
                                   focus:ring-2 focus:ring-blue-500 outline-none"
                            prop:value={base_url.get()}
                            on:input={move |ev| set_base_url.set(event_target_value(&ev))} />
                    </div>

                    <div>
                        <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                            "Theme"
                        </label>
                        <select
                            class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg
                                   bg-white dark:bg-gray-700 text-gray-900 dark:text-white 
                                   focus:ring-2 focus:ring-blue-500 outline-none"
                            prop:value={theme.get()}
                            on:change={move |ev| set_theme.set(event_target_value(&ev))}>
                            <option value="auto">"Auto (system preference)"</option>
                            <option value="light">"Light"</option>
                            <option value="dark">"Dark"</option>
                        </select>
                    </div>
                </div>

                <div class="mt-6 flex gap-3">
                    <button class={btn_class} disabled={building.get()} on:click={handle_build}>
                        {move || if building.get() {
                            view! {
                                <svg class="w-5 h-5 animate-spin" fill="none" viewBox="0 0 24 24">
                                    <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                                    <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                                </svg>
                            }.into_any()
                        } else {
                            view! {
                                <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                                          d="M10 20l4-16m4 4l4 4-4 4M6 16l-4-4 4-4" />
                                </svg>
                            }.into_any()
                        }}
                        {btn_label}
                    </button>
                    <button
                        class="px-6 py-3 text-gray-700 dark:text-gray-300 border border-gray-300 dark:border-gray-600
                               rounded-lg hover:bg-gray-50 dark:hover:bg-gray-700 transition-colors font-medium"
                        on:click={handle_download}
                    >
                        "Download Latest"
                    </button>
                </div>
            </div>

            {result_view}
        </div>
    }
}
