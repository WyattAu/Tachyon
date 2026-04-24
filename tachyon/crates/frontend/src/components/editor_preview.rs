use tachyon_editor::Editor;
use leptos::prelude::*;
use crate::api::ApiClient;
use crate::markdown::render_markdown_to_html;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;
use std::cell::RefCell;
use std::rc::Rc;

#[component]
#[allow(dead_code)]
pub fn EditorPreview(
    editor: RwSignal<Editor>,
    #[prop(default = String::new())]
    #[allow(unused)]
    document_id: String,
) -> impl IntoView {
    let (preview_html, set_preview_html) = signal(String::new());
    let (render_error, set_render_error) = signal::<Option<String>>(None);
    let (is_loading, set_is_loading) = signal(false);
    let (using_client_render, set_using_client_render) = signal(false);
    let editor_content = RwSignal::new(String::new());

    let debounce_handle: Rc<RefCell<Option<i32>>> = Rc::new(RefCell::new(None));

    Effect::new(move |_| {
        let content = editor.with(|e| e.content());
        if content != editor_content.get_untracked() {
            editor_content.set(content.clone());
            if content.trim().is_empty() {
                set_preview_html.set(String::new());
                set_render_error.set(None);
                set_using_client_render.set(false);
                return;
            }

            let handle = debounce_handle.borrow().clone();
            if let Some(h) = handle {
                let _ = web_sys::window().map(|w| {
                    let _ = w.clear_timeout_with_handle(h);
                });
            }

            let api = ApiClient::default();
            let set_html = set_preview_html.clone();
            let set_err = set_render_error.clone();
            let set_load = set_is_loading.clone();
            let set_client = set_using_client_render.clone();
            let content_for_closure = content.clone();
            let dh = debounce_handle.clone();

            let closure = wasm_bindgen::closure::Closure::<dyn Fn()>::new(move || {
                set_load.set(true);
                let api = api.clone();
                let content = content_for_closure.clone();
                let set_html = set_html.clone();
                let set_err = set_err.clone();
                let set_load = set_load.clone();
                let set_client = set_client.clone();

                spawn_local(async move {
                    match api.render_markdown(&content).await {
                        Ok(response) => {
                            set_html.set(response.html);
                            set_err.set(None);
                            set_load.set(false);
                            set_client.set(false);
                        }
                        Err(_) => {
                            let html = render_markdown_to_html(&content);
                            set_html.set(html);
                            set_err.set(None);
                            set_load.set(false);
                            set_client.set(true);
                        }
                    }
                });
            });

            let timeout = web_sys::window()
                .and_then(|w| {
                    w.set_timeout_with_callback_and_timeout_and_arguments_0(
                        closure.as_ref().unchecked_ref(),
                        300,
                    )
                    .ok()
                })
                .unwrap_or(0);

            *dh.borrow_mut() = Some(timeout);
            closure.forget();
        }
    });

    view! {
        <div class="editor-preview">
            {move || {
                if is_loading.get() && preview_html.get().is_empty() {
                    view! {
                        <div class="flex items-center justify-center h-full">
                            <div class="flex items-center gap-3 text-gray-500 dark:text-gray-400">
                                <div class="w-5 h-5 border-2 border-gray-400 border-t-transparent rounded-full animate-spin"></div>
                                <span>"Rendering..."</span>
                            </div>
                        </div>
                    }.into_any()
                } else if let Some(err) = render_error.get() {
                    view! {
                        <div class="p-3 bg-yellow-50 dark:bg-yellow-900/20 border border-yellow-200 dark:border-yellow-800 rounded text-sm text-yellow-700 dark:text-yellow-300">
                            {err}
                        </div>
                    }.into_any()
                } else if preview_html.get().is_empty() {
                    view! {
                        <div class="flex items-center justify-center h-full text-gray-400 dark:text-gray-500">
                            <div class="text-center">
                                <p class="text-lg mb-1">"No content yet"</p>
                                <p class="text-sm">"Start typing to see preview"</p>
                            </div>
                        </div>
                    }.into_any()
                } else {
                    view! {
                        <div>
                            {move || if using_client_render.get() {
                                view! {
                                    <div class="px-3 py-1.5 text-xs text-blue-600 dark:text-blue-400 bg-blue-50 dark:bg-blue-900/20 border-b border-blue-200 dark:border-blue-800">
                                        "Offline preview (client-side rendering)"
                                    </div>
                                }.into_any()
                            } else {
                                ().into_any()
                            }}
                            <div
                                class="prose prose-sm dark:prose-invert max-w-none"
                                inner_html={preview_html.get()}
                            ></div>
                        </div>
                    }.into_any()
                }
            }}
        </div>
    }
}
