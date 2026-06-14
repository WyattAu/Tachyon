use crate::push::{
    NotificationPreferences, get_notification_preferences, is_push_subscribed,
    request_notification_permission, save_notification_preferences, send_test_notification,
    subscribe_to_push, unsubscribe_from_push,
};
use leptos::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NotifTab {
    Push,
    Types,
}

#[component]
pub fn NotificationSettingsPage() -> impl IntoView {
    let (prefs, set_prefs) = signal(get_notification_preferences());
    let (push_enabled, set_push_enabled) = signal(is_push_subscribed());
    let (loading, set_loading) = signal(false);
    let (msg, set_msg) = signal(String::new());
    let (test_sending, set_test_sending) = signal(false);
    let (active_tab, set_active_tab) = signal(NotifTab::Push);

    let on_toggle_push = move |_| {
        let currently_enabled = push_enabled.get();
        set_loading.set(true);
        set_msg.set(String::new());
        let set_push = set_push_enabled;
        let set_load = set_loading;
        let set_message = set_msg;
        wasm_bindgen_futures::spawn_local(async move {
            if currently_enabled {
                match unsubscribe_from_push().await {
                    Ok(_) => {
                        set_push.set(false);
                        set_message.set("Push notifications disabled.".to_string());
                    }
                    Err(e) => set_message.set(format!("Failed to disable: {}", e)),
                }
            } else {
                match request_notification_permission().await {
                    Ok(true) => match subscribe_to_push().await {
                        Ok(_) => {
                            set_push.set(true);
                            set_message.set("Push notifications enabled.".to_string());
                        }
                        Err(e) => set_message.set(format!("Failed to subscribe: {}", e)),
                    },
                    Ok(false) => {
                        set_message.set("Notification permission denied.".to_string());
                    }
                    Err(e) => set_message.set(format!("Permission check failed: {}", e)),
                }
            }
            set_load.set(false);
        });
    };

    let on_toggle_type = move |field: fn(&mut NotificationPreferences) -> &mut bool| {
        set_prefs.update(|p| { let val = field(p); *val = !*val; });
        let current = prefs.get();
        save_notification_preferences(&current);
    };

    let on_test_notification = move |_| {
        set_test_sending.set(true);
        set_msg.set(String::new());
        let set_sending = set_test_sending;
        let set_message = set_msg;
        wasm_bindgen_futures::spawn_local(async move {
            match send_test_notification().await {
                Ok(_) => set_message.set("Test notification sent!".to_string()),
                Err(e) => set_message.set(format!("Failed to send test: {}", e)),
            }
            set_sending.set(false);
        });
    };

    view! {
        <div class="max-w-3xl">
            <h1 class="text-2xl font-bold mb-6 text-gray-900 dark:text-white">"Notification Settings"</h1>

            <div class="flex border-b border-gray-200 dark:border-gray-700 mb-6" role="tablist">
                <button
                    class=move || if active_tab.get() == NotifTab::Push {
                        "px-4 py-2.5 text-sm font-medium border-b-2 border-blue-500 text-blue-600 dark:text-blue-400"
                    } else {
                        "px-4 py-2.5 text-sm font-medium border-b-2 border-transparent text-gray-500 dark:text-gray-400 hover:text-gray-700 dark:hover:text-gray-300"
                    }
                    on:click=move |_| set_active_tab.set(NotifTab::Push)
                >"Push Notifications"</button>
                <button
                    class=move || if active_tab.get() == NotifTab::Types {
                        "px-4 py-2.5 text-sm font-medium border-b-2 border-blue-500 text-blue-600 dark:text-blue-400"
                    } else {
                        "px-4 py-2.5 text-sm font-medium border-b-2 border-transparent text-gray-500 dark:text-gray-400 hover:text-gray-700 dark:hover:text-gray-300"
                    }
                    on:click=move |_| set_active_tab.set(NotifTab::Types)
                >"Notification Types"</button>
            </div>

            <Show when={move || active_tab.get() == NotifTab::Push}>
                <div class="bg-white dark:bg-gray-800 rounded-none shadow border-2 border-gray-900 dark:border-gray-100 p-6">
                    <div class="mb-4">
                        <h2 class="text-lg font-semibold text-gray-900 dark:text-white">"Push Notifications"</h2>
                        <p class="text-sm text-gray-500 dark:text-gray-400">"Enable browser push notifications to stay updated"</p>
                    </div>
                    <div class="flex items-center justify-between">
                        <div>
                            <p class="text-sm font-medium text-gray-700 dark:text-gray-300">"Browser Push Notifications"</p>
                            <p class="text-sm text-gray-500 dark:text-gray-400">
                                {move || if push_enabled.get() {
                                    "Push notifications are enabled"
                                } else {
                                    "Push notifications are disabled"
                                }}
                            </p>
                        </div>
                        <button
                            type="button"
                            on:click=on_toggle_push
                            disabled=move || loading.get()
                            role="switch"
                            aria-checked=move || if push_enabled.get() { "true" } else { "false" }
                            aria-label="Toggle push notifications"
                            class=move || if push_enabled.get() {
                                "relative inline-flex h-6 w-11 flex-shrink-0 cursor-pointer rounded-full border-2 border-transparent bg-blue-600 transition-colors"
                            } else {
                                "relative inline-flex h-6 w-11 flex-shrink-0 cursor-pointer rounded-full border-2 border-transparent bg-gray-200 dark:bg-gray-600 transition-colors"
                            }>
                            <span class=move || if push_enabled.get() {
                                "pointer-events-none inline-block h-5 w-5 transform rounded-full bg-white shadow ring-0 transition translate-x-5"
                            } else {
                                "pointer-events-none inline-block h-5 w-5 transform rounded-full bg-white shadow ring-0 transition translate-x-0"
                            } />
                        </button>
                    </div>
                </div>
            </Show>

            <Show when={move || active_tab.get() == NotifTab::Types}>
                <div class="bg-white dark:bg-gray-800 rounded-none shadow border-2 border-gray-900 dark:border-gray-100 p-6">
                    <div class="mb-4">
                        <h2 class="text-lg font-semibold text-gray-900 dark:text-white">"Notification Types"</h2>
                        <p class="text-sm text-gray-500 dark:text-gray-400">"Choose which notifications you want to receive"</p>
                    </div>
                    <div class="space-y-4">
                        <NotifTypeToggle
                            label="Mentions"
                            description="When someone mentions you in a document"
                            enabled=move || prefs.get().mention
                            on_toggle=move |_| on_toggle_type(|p| &mut p.mention)
                        />
                        <NotifTypeToggle
                            label="Comments"
                            description="When someone comments on your documents"
                            enabled=move || prefs.get().comment
                            on_toggle=move |_| on_toggle_type(|p| &mut p.comment)
                        />
                        <NotifTypeToggle
                            label="Reviews"
                            description="When a review is requested or updated"
                            enabled=move || prefs.get().review
                            on_toggle=move |_| on_toggle_type(|p| &mut p.review)
                        />
                        <NotifTypeToggle
                            label="Assignments"
                            description="When you are assigned a task or document"
                            enabled=move || prefs.get().assignment
                            on_toggle=move |_| on_toggle_type(|p| &mut p.assignment)
                        />
                    </div>
                </div>
            </Show>

            <div class="mt-6 bg-white dark:bg-gray-800 rounded-none shadow border-2 border-gray-900 dark:border-gray-100 p-6">
                <div class="mb-4">
                    <h2 class="text-lg font-semibold text-gray-900 dark:text-white">"Test Notifications"</h2>
                    <p class="text-sm text-gray-500 dark:text-gray-400">"Send a test notification to verify your setup"</p>
                </div>
                <div class="flex items-center gap-3">
                    <button
                        on:click=on_test_notification
                        disabled=move || test_sending.get() || !push_enabled.get()
                        class="px-4 py-2 bg-blue-600 hover:bg-blue-700 disabled:bg-blue-400 text-white rounded-none transition-colors">
                        {move || if test_sending.get() { "Sending..." } else { "Send Test Notification" }}
                    </button>
                    <span class="text-sm text-gray-600 dark:text-gray-400">{move || msg.get()}</span>
                </div>
            </div>
        </div>
    }
}

#[component]
fn NotifTypeToggle(
    label: &'static str,
    description: &'static str,
    enabled: impl Fn() -> bool + Clone + 'static + Send,
    on_toggle: impl Fn(leptos::ev::MouseEvent) + 'static,
) -> impl IntoView {
    let enabled2 = enabled.clone();
    let enabled3 = enabled.clone();
    view! {
        <div class="flex items-center justify-between py-2">
            <div>
                <p class="text-sm font-medium text-gray-700 dark:text-gray-300">{label}</p>
                <p class="text-sm text-gray-500 dark:text-gray-400">{description}</p>
            </div>
            <button
                type="button"
                on:click=on_toggle
                role="switch"
                aria-checked=move || if enabled() { "true" } else { "false" }
                aria-label=label
                class=move || if enabled2() {
                    "relative inline-flex h-6 w-11 flex-shrink-0 cursor-pointer rounded-full border-2 border-transparent bg-blue-600 transition-colors"
                } else {
                    "relative inline-flex h-6 w-11 flex-shrink-0 cursor-pointer rounded-full border-2 border-transparent bg-gray-200 dark:bg-gray-600 transition-colors"
                }>
                <span class=move || if enabled3() {
                    "pointer-events-none inline-block h-5 w-5 transform rounded-full bg-white shadow ring-0 transition translate-x-5"
                } else {
                    "pointer-events-none inline-block h-5 w-5 transform rounded-full bg-white shadow ring-0 transition translate-x-0"
                } />
            </button>
        </div>
    }
}
