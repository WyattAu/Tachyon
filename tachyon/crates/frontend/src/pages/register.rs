// Register Page
// Account creation page

use leptos::prelude::*;
use leptos::ev;
use leptos_router::hooks::use_navigate;
use leptos::task::spawn_local;
use crate::api::ApiClient;
use crate::components::ButtonSpinner;

/// Register page component
#[component]
pub fn RegisterPage() -> impl IntoView {
    let username = RwSignal::new(String::new());
    let email = RwSignal::new(String::new());
    let password = RwSignal::new(String::new());
    let confirm_password = RwSignal::new(String::new());
    let error = RwSignal::new(None::<String>);
    let loading = RwSignal::new(false);

    let nav = StoredValue::new(use_navigate());

    let on_submit = move |ev: ev::SubmitEvent| {
        ev.prevent_default();

        let username_val = username.get();
        let email_val = email.get();
        let password_val = password.get();
        let confirm_val = confirm_password.get();

        // Client-side validation
        if password_val != confirm_val {
            error.set(Some("Passwords do not match".to_string()));
            return;
        }
        if password_val.len() < 8 {
            error.set(Some("Password must be at least 8 characters".to_string()));
            return;
        }
        if username_val.len() < 3 {
            error.set(Some("Username must be at least 3 characters".to_string()));
            return;
        }

        loading.set(true);
        error.set(None);

        let username_clone = username_val.clone();
        let email_clone = email_val.clone();
        let password_clone = password_val.clone();

        spawn_local(async move {
            let client = ApiClient::default();

            match client.register(&username_clone, &email_clone, &password_clone).await {
                Ok(response) => {
                    if let Some(token) = response.access_token {
                        client.set_auth_token(token.clone());
                        if let Some(window) = web_sys::window() {
                            if let Ok(Some(storage)) = window.local_storage() {
                                let _ = storage.set_item("tachyon_token", &token);
                            }
                        }
                    }
                    nav.update_value(|n| n("/dashboard", Default::default()));
                }
                Err(e) => {
                    loading.set(false);
                    error.set(Some(format!("Registration failed: {}", e)));
                }
            }
        });
    };

    view! {
        <div class="min-h-screen flex items-center justify-center bg-gray-50 dark:bg-gray-900 py-12 px-4 sm:px-6 lg:px-8">
            <div class="max-w-md w-full space-y-8">
                <div class="text-center">
                    <h1 class="text-4xl font-bold text-gray-900 dark:text-white">"Tachyon"</h1>
                    <h2 class="mt-2 text-lg text-gray-600 dark:text-gray-400">"Create your account"</h2>
                </div>

                <div class="bg-white dark:bg-gray-800 rounded-lg shadow-md p-8">
                    <Show when=move || error.get().is_some()>
                        <div class="mb-4 p-3 bg-red-100 dark:bg-red-900 border border-red-400 dark:border-red-700 text-red-700 dark:text-red-200 rounded">
                            {move || error.get().unwrap_or_default()}
                        </div>
                    </Show>

                    <form on:submit=on_submit class="space-y-5">
                        <div>
                            <label for="reg-username" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">"Username"</label>
                            <input
                                id="reg-username"
                                type="text"
                                class="w-full px-4 py-2.5 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-white placeholder-gray-400 dark:placeholder-gray-500 focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                                placeholder="Choose a username"
                                on:input=move |ev| username.set(event_target_value(&ev))
                                prop:value=move || username.get()
                                minlength="3"
                                required
                            />
                        </div>

                        <div>
                            <label for="reg-email" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">"Email"</label>
                            <input
                                id="reg-email"
                                type="email"
                                class="w-full px-4 py-2.5 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-white placeholder-gray-400 dark:placeholder-gray-500 focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                                placeholder="you@example.com"
                                on:input=move |ev| email.set(event_target_value(&ev))
                                prop:value=move || email.get()
                                required
                            />
                        </div>

                        <div>
                            <label for="reg-password" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">"Password"</label>
                            <input
                                id="reg-password"
                                type="password"
                                class="w-full px-4 py-2.5 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-white placeholder-gray-400 dark:placeholder-gray-500 focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                                placeholder="At least 8 characters"
                                on:input=move |ev| password.set(event_target_value(&ev))
                                prop:value=move || password.get()
                                minlength="8"
                                required
                            />
                        </div>

                        <div>
                            <label for="reg-confirm" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">"Confirm Password"</label>
                            <input
                                id="reg-confirm"
                                type="password"
                                class="w-full px-4 py-2.5 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-white placeholder-gray-400 dark:placeholder-gray-500 focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                                placeholder="Confirm your password"
                                on:input=move |ev| confirm_password.set(event_target_value(&ev))
                                prop:value=move || confirm_password.get()
                                minlength="8"
                                required
                            />
                        </div>

                        <button
                            type="submit"
                            class="w-full py-3 px-4 bg-blue-600 hover:bg-blue-700 text-white font-medium rounded-lg transition-colors focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2 disabled:opacity-50 disabled:cursor-not-allowed flex items-center justify-center"
                            disabled=move || loading.get()
                        >
                            {move || if loading.get() {
                                view! { <span class="flex items-center justify-center"><ButtonSpinner />"Creating account..."</span> }.into_any()
                            } else {
                                view! { "Create Account" }.into_any()
                            }}
                        </button>
                    </form>
                </div>

                <div class="text-center">
                    <p class="text-sm text-gray-500 dark:text-gray-400">
                        "Already have an account? "
                        <a href="/login" class="text-blue-600 hover:underline dark:text-blue-400">"Sign in"</a>
                    </p>
                </div>

                <div class="text-center">
                    <a href="/" class="text-sm text-gray-500 dark:text-gray-400 hover:text-blue-600 dark:hover:text-blue-400">"← Back to Home"</a>
                </div>
            </div>
        </div>
    }
}
