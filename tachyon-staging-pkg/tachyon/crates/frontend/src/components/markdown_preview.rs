#![allow(dead_code)]

use crate::markdown::{
    MarkdownHeading, extract_headings as md_extract_headings, render_markdown_to_html,
};
use leptos::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;

/// Set up a delegated click handler on the container element so that
/// clicks on `.wikilink` anchor elements navigate via the Leptos router
/// instead of triggering a full-page reload.
fn setup_wikilink_handler(container: web_sys::Element) {
    let closure = wasm_bindgen::closure::Closure::<dyn Fn(web_sys::MouseEvent)>::new(
        move |ev: web_sys::MouseEvent| {
            if let Some(target) = ev.target() {
                let el: web_sys::Element = match target.dyn_into() {
                    Ok(e) => e,
                    Err(_) => return,
                };

                // Walk up from the click target to find the nearest .wikilink anchor
                let anchor: Option<web_sys::HtmlAnchorElement> = el
                    .dyn_ref::<web_sys::HtmlAnchorElement>()
                    .cloned()
                    .or_else(|| {
                        el.closest("a.wikilink")
                            .ok()
                            .flatten()
                            .and_then(|a| a.dyn_into().ok())
                    });

                if let Some(a) = anchor {
                    if let Some(href) = a.get_attribute("href") {
                        if !href.is_empty() && !href.starts_with('#') && !href.starts_with("http") {
                            ev.prevent_default();
                            ev.stop_propagation();

                            let window = web_sys::window().unwrap();
                            let loc = window.location();
                            let current_path = loc.pathname().unwrap_or_default();
                            let base = if let Some(pos) = current_path.rfind('/') {
                                &current_path[..pos]
                            } else {
                                ""
                            };
                            let full_path = format!("{}{}", base, href);

                            // Use history.pushState for SPA navigation
                            if let Ok(history) = window.history() {
                                let state: &JsValue = &JsValue::from(js_sys::Object::new());
                                let _ = history.push_state_with_url(state, "", Some(&full_path));

                                // Dispatch popstate so Leptos router picks it up
                                if let Ok(event) = web_sys::Event::new("popstate") {
                                    let _ = window.dispatch_event(&event);
                                }
                            }
                        }
                    }
                }
            }
        },
    );

    let _ = container.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref());
    closure.forget();
}

#[component]
pub fn MarkdownPreview(content: String, #[prop(default = true)] render_toc: bool) -> impl IntoView {
    let (html_output, set_html_output) = signal(String::new());
    let (headings, set_headings) = signal(Vec::<MarkdownHeading>::new());

    Effect::new(move |_| {
        let rendered = render_markdown_to_html(&content);
        set_html_output.set(rendered);

        if render_toc {
            let h = md_extract_headings(&content);
            set_headings.set(h);
        }
    });

    // Set up wikilink click handler after first render
    let container_ref = NodeRef::<leptos::html::Div>::new();
    let handler_installed = RwSignal::new(false);

    Effect::new(move |_| {
        if !handler_installed.get() {
            if let Some(container) = container_ref.get() {
                setup_wikilink_handler(container.into());
                handler_installed.set(true);
            }
        }
    });

    view! {
        <div class="markdown-preview flex h-full">
            {move || {
                if render_toc && !headings.get().is_empty() {
                    view! {
                        <nav class="hidden lg:block w-48 flex-shrink-0 border-r border-gray-200 dark:border-gray-700 p-4 overflow-y-auto">
                            <h4 class="text-xs font-semibold text-gray-500 dark:text-gray-400 uppercase tracking-wider mb-3">
                                "Table of Contents"
                            </h4>
                            <For
                                each=move || headings.get()
                                key=|h| h.slug.clone()
                                let:heading
                            >
                                <a
                                    href={format!("#{}", heading.slug)}
                                    class=move || {
                                        let _indent = (heading.level.saturating_sub(1) as usize) * 12;
                                        format!(
                                            "block py-0.5 text-xs text-gray-600 dark:text-gray-400 hover:text-blue-600 dark:hover:text-blue-400 truncate transition-colors {}",
                                            if heading.level <= 2 { "font-medium" } else { "" }
                                        )
                                    }
                                    style=move || {
                                        let _indent = (heading.level.saturating_sub(1) as usize) * 12;
                                        format!("padding-left: {}px", (heading.level.saturating_sub(1) as usize) * 12)
                                    }
                                    title={heading.text.clone()}
                                >
                                    {heading.text.clone()}
                                </a>
                            </For>
                        </nav>
                    }.into_any()
                } else {
                    ().into_any()
                }
            }}
            <div class="flex-1 overflow-y-auto p-6" node_ref=container_ref>
                {move || {
                    let html = html_output.get();
                    if html.is_empty() {
                        view! {
                            <div class="flex items-center justify-center h-full text-gray-400 dark:text-gray-500">
                                <div class="text-center">
                                    <p class="text-lg mb-1">"No content to preview"</p>
                                    <p class="text-sm">"Start typing markdown to see a preview"</p>
                                </div>
                            </div>
                        }.into_any()
                    } else {
                        view! {
                            <div
                                class="prose prose-sm dark:prose-invert max-w-none wikilink-content"
                                inner_html={html}
                            ></div>
                        }.into_any()
                    }
                }}
            </div>
        </div>
    }
}
