use leptos::prelude::*;
use wasm_bindgen::JsCast;

#[derive(Debug, Clone, PartialEq)]
pub enum ConnectionStatus {
    Online,
    Offline,
    Syncing,
}

#[component]
pub fn OnlineStatusIndicator() -> impl IntoView {
    let (status, set_status) = signal(ConnectionStatus::Online);
    let (pending_count, set_pending_count) = signal(0usize);

    Effect::new(move |_| {
        let set_status = set_status.clone();

        if let Some(window) = web_sys::window() {
            let navigator = window.navigator();
            let online = navigator.on_line();
            set_status.set(if online {
                ConnectionStatus::Online
            } else {
                ConnectionStatus::Offline
            });

            let set_online = set_status.clone();
            let online_closure = wasm_bindgen::closure::Closure::<dyn Fn(_)>::new(
                move |_: wasm_bindgen::JsValue| {
                    set_online.set(ConnectionStatus::Online);
                },
            );
            let _ = window.add_event_listener_with_callback(
                "online",
                online_closure.as_ref().unchecked_ref(),
            );
            online_closure.forget();

            let set_offline = set_status.clone();
            let offline_closure = wasm_bindgen::closure::Closure::<dyn Fn(_)>::new(
                move |_: wasm_bindgen::JsValue| {
                    set_offline.set(ConnectionStatus::Offline);
                },
            );
            let _ = window.add_event_listener_with_callback(
                "offline",
                offline_closure.as_ref().unchecked_ref(),
            );
            offline_closure.forget();
        }
    });

    Effect::new(move |_| {
        let set_status = set_status.clone();
        let set_pending = set_pending_count.clone();

        if let Some(window) = web_sys::window() {
            let set_status_msg = set_status.clone();
            let set_pending_msg = set_pending.clone();
            let msg_closure = wasm_bindgen::closure::Closure::<dyn Fn(_)>::new(
                move |event: wasm_bindgen::JsValue| {
                    if let Some(msg_event) = event.dyn_ref::<web_sys::MessageEvent>() {
                        if let Ok(data) = js_sys::JSON::stringify(&msg_event.data()) {
                            if let Some(data_str) = data.as_string() {
                                if data_str.contains("SYNC_START") {
                                    set_status_msg.set(ConnectionStatus::Syncing);
                                } else if data_str.contains("SYNC_COMPLETE") {
                                    set_status_msg.set(ConnectionStatus::Online);
                                    set_pending_msg.set(0);
                                }
                            }
                        }
                    }
                },
            );
            let worker_container = window.navigator().service_worker();
            let _ = worker_container
                .add_event_listener_with_callback("message", msg_closure.as_ref().unchecked_ref());
            msg_closure.forget();
        }
    });

    let trigger_sync = move |_| {
        if let Some(window) = web_sys::window() {
            let worker_container = window.navigator().service_worker();
            if let Ok(Some(worker)) = worker_container.active() {
                let _ = worker.post_message(
                    &serde_wasm_bindgen::to_value(&serde_json::json!({
                        "type": "TRIGGER_SYNC"
                    }))
                    .unwrap_or_default(),
                );
            }
        }
        set_status.set(ConnectionStatus::Syncing);
    };

    let status_class = move || match status.get() {
        ConnectionStatus::Online => "bg-green-500",
        ConnectionStatus::Offline => "bg-red-500",
        ConnectionStatus::Syncing => "bg-yellow-500 animate-pulse",
    };

    let status_text = move || match status.get() {
        ConnectionStatus::Online => "Online",
        ConnectionStatus::Offline => "Offline",
        ConnectionStatus::Syncing => "Syncing...",
    };

    view! {
        <div class="flex items-center gap-2">
            <div class=move || format!("w-2 h-2 rounded-full {}", status_class()) />
            <span class="text-xs text-gray-500 dark:text-gray-400">
                {status_text}
            </span>
            {move || {
                let count = pending_count.get();
                if count > 0 {
                    view! {
                        <button
                            on:click=trigger_sync
                            class="text-xs text-blue-600 dark:text-blue-400 hover:underline"
                            title="Sync pending changes"
                        >
                            {format!("{} pending", count)}
                        </button>
                    }.into_any()
                } else if status.get() == ConnectionStatus::Offline {
                    view! {
                        <span class="text-xs text-red-500 dark:text-red-400">
                            "Changes will sync when online"
                        </span>
                    }.into_any()
                } else {
                    ().into_any()
                }
            }}
        </div>
    }
}
