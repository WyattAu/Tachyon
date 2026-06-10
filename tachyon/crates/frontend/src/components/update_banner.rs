// Update Banner Component
// Shows notification when a desktop app update is available

#![allow(dead_code)]

use leptos::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;

#[derive(Debug, Clone)]
pub struct UpdateInfo {
    pub available: bool,
    pub current_version: String,
    pub latest_version: Option<String>,
    pub release_notes: Option<String>,
}

async fn check_for_update_tauri() -> Option<UpdateInfo> {
    let window = web_sys::window()?;
    let tauri =
        js_sys::Reflect::get(&window, &js_sys::JsString::from("__TAURI_INTERNALS__")).ok()?;

    let invoke_fn = js_sys::Reflect::get(&tauri, &js_sys::JsString::from("invoke")).ok()?;
    let invoke_fn = invoke_fn.dyn_into::<js_sys::Function>().ok()?;

    let this = js_sys::Object::new();
    let command = js_sys::JsString::from("check_for_update");
    let args = js_sys::Array::new();

    let result = invoke_fn.call2(&this, &command, &args).ok()?;

    let obj = result.dyn_into::<js_sys::Object>().ok()?;

    let available = js_sys::Reflect::get(&obj, &js_sys::JsString::from("update_available"))
        .ok()
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    let current_version = js_sys::Reflect::get(&obj, &js_sys::JsString::from("current_version"))
        .ok()
        .and_then(|v| v.as_string())
        .unwrap_or_default();

    let latest_version = js_sys::Reflect::get(&obj, &js_sys::JsString::from("latest_version"))
        .ok()
        .and_then(|v| v.as_string());

    let release_notes = js_sys::Reflect::get(&obj, &js_sys::JsString::from("release_notes"))
        .ok()
        .and_then(|v| v.as_string());

    Some(UpdateInfo {
        available,
        current_version,
        latest_version,
        release_notes,
    })
}

fn download_update_tauri() {
    if let Some(window) = web_sys::window() {
        if let Ok(tauri) =
            js_sys::Reflect::get(&window, &js_sys::JsString::from("__TAURI_INTERNALS__"))
        {
            if !tauri.is_undefined() && !tauri.is_null() {
                if let Ok(invoke_fn) =
                    js_sys::Reflect::get(&tauri, &js_sys::JsString::from("invoke"))
                {
                    if let Ok(invoke_fn) = invoke_fn.dyn_into::<js_sys::Function>() {
                        let this = js_sys::Object::new();
                        let command = js_sys::JsString::from("download_and_install_update");
                        let args = js_sys::Array::new();
                        let _ = invoke_fn.call2(&this, &command, &args);
                    }
                }
            }
        }
    }
}

fn listen_download_progress(set_progress: WriteSignal<u32>) {
    if let Some(window) = web_sys::window() {
        if let Ok(tauri) =
            js_sys::Reflect::get(&window, &js_sys::JsString::from("__TAURI_INTERNALS__"))
        {
            if !tauri.is_undefined() && !tauri.is_null() {
                if let Ok(listen_fn) =
                    js_sys::Reflect::get(&tauri, &js_sys::JsString::from("listen"))
                {
                    if let Ok(listen_fn) = listen_fn.dyn_into::<js_sys::Function>() {
                        let this = js_sys::Object::new();
                        let event_name = js_sys::JsString::from("update-download-progress");

                        let closure =
                            wasm_bindgen::closure::Closure::<dyn Fn(wasm_bindgen::JsValue)>::new(
                                move |event: wasm_bindgen::JsValue| {
                                    let payload = js_sys::Reflect::get(
                                        &event,
                                        &js_sys::JsString::from("payload"),
                                    )
                                    .ok();
                                    if let Some(payload) = payload {
                                        if let Some(progress) = payload.as_f64() {
                                            set_progress.set(progress as u32);
                                        }
                                    }
                                },
                            );

                        let _ =
                            listen_fn.call2(&this, &event_name, closure.as_ref().unchecked_ref());
                        closure.forget();
                    }
                }
            }
        }
    }
}

#[component]
pub fn UpdateBanner() -> impl IntoView {
    let (update_info, set_update_info) = signal(None::<UpdateInfo>);
    let (dismissed, set_dismissed) = signal(false);
    let (downloading, set_downloading) = signal(false);
    let (download_progress, set_download_progress) = signal(0u32);

    Effect::new(move |_| {
        let set_info = set_update_info;
        spawn_local(async move {
            if web_sys::window()
                .and_then(|w| {
                    js_sys::Reflect::get(&w, &js_sys::JsString::from("__TAURI_INTERNALS__")).ok()
                })
                .is_some()
            {
                if let Some(info) = check_for_update_tauri().await {
                    set_info.set(Some(info));
                }
            }
        });
    });

    Effect::new(move |_| {
        listen_download_progress(set_download_progress);
    });

    let on_dismiss = Callback::new(move |_: leptos::ev::MouseEvent| {
        set_dismissed.set(true);
    });

    let on_download = Callback::new(move |_: leptos::ev::MouseEvent| {
        set_downloading.set(true);
        download_update_tauri();
    });

    move || {
        let info = update_info.get();
        let is_dismissed = dismissed.get();
        let is_downloading = downloading.get();
        let progress = download_progress.get();

        if is_dismissed {
            return ().into_any();
        }

        match info {
            Some(ref info) if info.available => {
                let latest = info
                    .latest_version
                    .clone()
                    .unwrap_or_else(|| "unknown".to_string());
                let notes = info.release_notes.clone();

                if is_downloading {
                    view! {
                        <div class="bg-blue-600 text-white px-4 py-2 text-center text-sm flex items-center justify-center gap-3">
                            <svg class="animate-spin h-4 w-4" fill="none" viewBox="0 0 24 24" aria-hidden="true">
                                <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                                <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z"></path>
                            </svg>
                            <span>"Downloading update... "</span>
                            <span class="font-medium">{progress}"%"</span>
                        </div>
                    }.into_any()
                } else {
                    let notes_clone = notes.clone();
                    view! {
                        <div class="bg-blue-600 text-white px-4 py-2 flex items-center justify-between text-sm">
                            <div class="flex items-center gap-2">
                                <svg class="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
                                </svg>
                                <span>"Update available: v"</span>
                                <span class="font-medium">{latest}</span>
                                {
                                    match notes_clone {
                                        Some(notes) if !notes.is_empty() => {
                                            let truncated = if notes.len() > 80 {
                                                format!("{}...", &notes.chars().take(80).collect::<String>())
                                            } else {
                                                notes
                                            };
                                            view! {
                                                <span class="hidden sm:inline text-blue-200 ml-2">"— "</span>
                                                <span class="hidden sm:inline text-blue-200">{truncated}</span>
                                            }.into_any()
                                        }
                                        _ => view! { <span></span> }.into_any()
                                    }
                                }
                            </div>
                            <div class="flex items-center gap-2 ml-4">
                                <button
                                    on:click=move |ev| on_download.run(ev)
                                    class="bg-white text-blue-600 px-3 py-0.5 rounded text-xs font-medium hover:bg-blue-50 transition-colors"
                                >
                                    "Update Now"
                                </button>
                                <button
                                    on:click=move |ev| on_dismiss.run(ev)
                                    class="text-blue-200 hover:text-white transition-colors"
                                    aria-label="Dismiss update notification"
                                >
                                    <svg class="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
                                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
                                    </svg>
                                </button>
                            </div>
                        </div>
                    }.into_any()
                }
            }
            _ => ().into_any(),
        }
    }
}
