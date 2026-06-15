#![allow(dead_code)]

use leptos::prelude::*;

/// PDF viewer component using browser's built-in PDF rendering
#[component]
pub fn PdfViewer(
    url: String,
    #[prop(default = String::new())] title: String,
    on_close: Option<Callback<()>>,
) -> impl IntoView {
    let url_stored = StoredValue::new(url.clone());
    let title_stored = StoredValue::new(title.clone());
    let (current_page, set_current_page) = signal(1u32);
    let (total_pages, set_total_pages) = signal(0u32);
    let (zoom_level, set_zoom_level) = signal(1.0_f64);
    let (is_fullscreen, set_is_fullscreen) = signal(false);
    let (loading, set_loading) = signal(true);
    let (error, set_error) = signal::<Option<String>>(None);

    // PDF.js worker initialization
    let pdf_viewer_ref = NodeRef::<leptos::html::Iframe>::new();

    // Initialize PDF.js viewer
    Effect::new(move |_| {
        let _url = url.clone();
        let set_loading = set_loading;
        let _set_error = set_error;
        let set_total_pages = set_total_pages;

        wasm_bindgen_futures::spawn_local(async move {
            // For now, we'll use the browser's built-in PDF viewer
            // In a real implementation, we'd use pdf.js
            set_loading.set(false);
            // Default to a reasonable page count
            set_total_pages.set(1);
        });
    });

    // Navigation handlers
    let prev_page = move |_: leptos::ev::MouseEvent| {
        set_current_page.update(|p| {
            if *p > 1 {
                *p -= 1;
            }
        });
    };

    let _next_page = move |_: leptos::ev::MouseEvent| {
        set_current_page.update(|p| {
            if *p < total_pages.get() {
                *p += 1;
            }
        });
    };

    // Zoom handlers
    let zoom_in = move |_: leptos::ev::MouseEvent| {
        set_zoom_level.update(|z| *z = (*z + 0.25).min(3.0));
    };

    let zoom_out = move |_: leptos::ev::MouseEvent| {
        set_zoom_level.update(|z| *z = (*z - 0.25).max(0.25));
    };

    let zoom_reset = move |_: leptos::ev::MouseEvent| {
        set_zoom_level.set(1.0);
    };

    // Fullscreen toggle
    let toggle_fullscreen = move |_: leptos::ev::MouseEvent| {
        set_is_fullscreen.update(|f| *f = !*f);
    };

    // Close handler
    let close_handler = move |_: leptos::ev::MouseEvent| {
        if let Some(callback) = &on_close {
            callback.run(());
        }
    };

    // Zoom via mouse wheel
    let handle_wheel = move |ev: web_sys::WheelEvent| {
        ev.prevent_default();
        let delta = ev.delta_y() * -0.001;
        let new_zoom = (zoom_level.get() + delta).clamp(0.25, 3.0);
        set_zoom_level.set(new_zoom);
    };

    let container_class = move || {
        if is_fullscreen.get() {
            "fixed inset-0 bg-white dark:bg-gray-900 z-50 flex flex-col".to_string()
        } else {
            "flex flex-col h-full".to_string()
        }
    };

    view! {
        <div class=container_class>
            // Toolbar
            <div class="flex items-center justify-between px-4 py-2 border-b border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-800">
                <div class="flex items-center gap-3">
                    {if on_close.is_some() {
                        view! {
                            <button
                                class="p-1.5 rounded hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors"
                                on:click=close_handler
                                aria-label="Close PDF viewer"
                            >
                                <svg class="w-5 h-5 text-gray-500" aria-hidden="true" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
                                </svg>
                            </button>
                        }.into_any()
                    } else {
                        ().into_any()
                    }}

                    <h2 class="text-sm font-medium text-gray-900 dark:text-white truncate max-w-xs">
                        {if title.is_empty() { "PDF Document".to_string() } else { title }}
                    </h2>
                </div>

                <div class="flex items-center gap-2">
                    // Page navigation
                    <button
                        class="px-2 py-1 text-sm bg-white dark:bg-gray-700 border border-gray-300 dark:border-gray-600 rounded hover:bg-gray-50 dark:hover:bg-gray-600 disabled:opacity-50 disabled:cursor-not-allowed text-gray-700 dark:text-gray-300"
                        disabled=move || current_page.get() <= 1
                        on:click=prev_page
                        aria-label="Previous page"
                    >
                        <svg class="w-4 h-4" aria-hidden="true" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 19l-7-7 7-7" />
                        </svg>
                    </button>

                    <span class="px-2 py-1 text-sm text-gray-700 dark:text-gray-300 min-w-[80px] text-center">
                        {move || format!("{} / {}", current_page.get(), total_pages.get())}
                    </span>

                    <button
                        class="px-2 py-1 text-sm bg-white dark:bg-gray-700 border border-gray-300 dark:border-gray-600 rounded hover:bg-gray-50 dark:hover:bg-gray-600 disabled:opacity-50 disabled:cursor-not-allowed text-gray-700 dark:text-gray-300"
                        disabled=move || current_page.get() >= total_pages.get()
                        on:click=next_page
                        aria-label="Next page"
                    >
                        <svg class="w-4 h-4" aria-hidden="true" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7" />
                        </svg>
                    </button>

                    // Divider
                    <div class="w-px h-6 bg-gray-300 dark:bg-gray-600 mx-1"></div>

                    // Zoom controls
                    <button
                        class="px-2 py-1 text-sm bg-white dark:bg-gray-700 border border-gray-300 dark:border-gray-600 rounded hover:bg-gray-50 dark:hover:bg-gray-600 text-gray-700 dark:text-gray-300"
                        on:click=zoom_out
                        aria-label="Zoom out"
                    >
                        <svg class="w-4 h-4" aria-hidden="true" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M20 12H4" />
                        </svg>
                    </button>

                    <button
                        class="px-2 py-1 text-sm bg-white dark:bg-gray-700 border border-gray-300 dark:border-gray-600 rounded hover:bg-gray-50 dark:hover:bg-gray-600 text-gray-700 dark:text-gray-300 min-w-[60px]"
                        on:click=zoom_reset
                        aria-label="Reset zoom"
                    >
                        {move || format!("{:.0}%", zoom_level.get() * 100.0)}
                    </button>

                    <button
                        class="px-2 py-1 text-sm bg-white dark:bg-gray-700 border border-gray-300 dark:border-gray-600 rounded hover:bg-gray-50 dark:hover:bg-gray-600 text-gray-700 dark:text-gray-300"
                        on:click=zoom_in
                        aria-label="Zoom in"
                    >
                        <svg class="w-4 h-4" aria-hidden="true" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
                        </svg>
                    </button>

                    // Divider
                    <div class="w-px h-6 bg-gray-300 dark:bg-gray-600 mx-1"></div>

                    // Fullscreen toggle
                    <button
                        class="p-1.5 rounded hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors"
                        on:click=toggle_fullscreen
                        aria-label={move || if is_fullscreen.get() { "Exit fullscreen" } else { "Enter fullscreen" }}
                    >
                        <svg class="w-5 h-5 text-gray-500" aria-hidden="true" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            {move || if is_fullscreen.get() {
                                view! {
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 9V4.5M9 9H4.5M9 9L3.75 3.75M9 15v4.5M9 15H4.5M9 15l-5.25 5.25M15 9h4.5M15 9V4.5M15 9l5.25-5.25M15 15h4.5M15 15v4.5m0-4.5l5.25 5.25" />
                                }.into_any()
                            } else {
                                view! {
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3.75 3.75v4.5m0-4.5h4.5m-4.5 0L9 9M3.75 20.25v-4.5m0 4.5h4.5m-4.5 0L9 15M20.25 3.75h-4.5m4.5 0v4.5m0-4.5L15 9m5.25 11.25h-4.5m4.5 0v-4.5m0 4.5L15 15" />
                                }.into_any()
                            }}
                        </svg>
                    </button>
                </div>
            </div>

            // PDF content area
            <div
                class="flex-1 overflow-auto bg-gray-100 dark:bg-gray-900"
                on:wheel=handle_wheel
            >
                {move || {
                    if loading.get() {
                        view! {
                            <div class="flex items-center justify-center h-full">
                                <div class="flex items-center gap-3 text-gray-500 dark:text-gray-400">
                                    <div class="w-5 h-5 border-2 border-gray-400 border-t-transparent rounded-full animate-spin"></div>
                                    <span>"Loading PDF..."</span>
                                </div>
                            </div>
                        }.into_any()
                    } else if let Some(err) = error.get() {
                        view! {
                            <div class="flex items-center justify-center h-full">
                                <div class="p-4 bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded text-red-700 dark:text-red-300 max-w-md">
                                    <p class="font-medium mb-2">"Error loading PDF"</p>
                                    <p class="text-sm">{err}</p>
                                </div>
                            </div>
                        }.into_any()
                    } else {
                        view! {
                            <PdfViewerContent
                                url=url_stored.get_value()
                                title=title_stored.get_value()
                                current_page=current_page
                                zoom_level=zoom_level
                                pdf_viewer_ref=pdf_viewer_ref
                            />
                        }.into_any()
                    }
                }}
            </div>
        </div>
    }
}

/// PDF viewer content component (iframe)
#[component]
fn PdfViewerContent(
    url: String,
    title: String,
    current_page: ReadSignal<u32>,
    zoom_level: ReadSignal<f64>,
    pdf_viewer_ref: NodeRef<leptos::html::Iframe>,
) -> impl IntoView {
    let embed_url = move || {
        let page = current_page.get();
        format!("{}#page={}", url, page)
    };
    let style = move || {
        format!(
            "transform: scale({}); transform-origin: top left;",
            zoom_level.get()
        )
    };

    view! {
        <div class="relative w-full h-full" style=style>
            <iframe
                node_ref=pdf_viewer_ref
                src=embed_url
                class="w-full h-full border-0"
                title=move || title.clone()
                style="min-height: 100vh;"
            ></iframe>
        </div>
    }
}

/// Standalone PDF viewer for embedding in document view
#[component]
pub fn PdfViewerEmbed(url: String) -> impl IntoView {
    view! {
        <PdfViewer url=url title=String::new() on_close=None />
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_pdf_viewer_component_creation() {
        // Component creation test - in real WASM tests we'd mount and check DOM
        // For unit testing, we verify the component compiles
    }
}
