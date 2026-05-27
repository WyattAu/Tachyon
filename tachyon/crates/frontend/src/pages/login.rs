use crate::api::ApiClient;
use crate::components::ButtonSpinner;
use leptos::ev;
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::hooks::use_navigate;

fn is_valid_email(email: &str) -> bool {
    (|| {
        let at = email.find('@')?;
        let domain = email.get(at + 1..)?;
        let dot = domain.find('.')?;
        Some(dot > 0 && dot < domain.len() - 1 && at > 0)
    })()
    .unwrap_or(false)
}

fn get_return_url() -> String {
    web_sys::window()
        .and_then(|w| w.location().search().ok())
        .and_then(|search| {
            for pair in search.trim_start_matches('?').split('&') {
                if let Some(url) = pair.strip_prefix("return=") {
                    let decoded = url
                        .replace("%2F", "/")
                        .replace("%2f", "/")
                        .replace("%3F", "?")
                        .replace("%3f", "?")
                        .replace("%26", "&")
                        .replace("%3D", "=")
                        .replace("%3d", "=")
                        .replace("%23", "#")
                        .replace("%25", "%");
                    return Some(decoded);
                }
            }
            None
        })
        .unwrap_or_else(|| "/dashboard".to_string())
}

#[component]
pub fn LoginPage() -> impl IntoView {
    let username = RwSignal::new(String::new());
    let password = RwSignal::new(String::new());
    let error = RwSignal::new(None::<String>);
    let username_error = RwSignal::new(None::<String>);
    let password_error = RwSignal::new(None::<String>);
    let loading = RwSignal::new(false);
    let username_ref = NodeRef::<leptos::html::Input>::new();
    let password_ref = NodeRef::<leptos::html::Input>::new();
    let remember_me = RwSignal::new(false);
    let guest_login_enabled = true;
    let public_notes_enabled = true;
    let nav = StoredValue::new(use_navigate());

    let on_submit = move |ev: ev::SubmitEvent| {
        ev.prevent_default();

        let username_val = username.get();
        let password_val = password.get();
        let remember_val = remember_me.get();

        let mut u_err = None::<String>;
        if username_val.trim().is_empty() {
            u_err = Some("Username or email is required".to_string());
        } else if username_val.contains('@') && !is_valid_email(&username_val) {
            u_err = Some("Invalid email format".to_string());
        }
        let p_err = if password_val.len() < 8 {
            Some("Password must be at least 8 characters".to_string())
        } else {
            None
        };

        username_error.set(u_err.clone());
        password_error.set(p_err.clone());

        if u_err.is_some() || p_err.is_some() {
            if u_err.is_some() {
                if let Some(el) = username_ref.get() {
                    let _ = el.focus();
                }
            } else if p_err.is_some() {
                if let Some(el) = password_ref.get() {
                    let _ = el.focus();
                }
            }
            return;
        }

        loading.set(true);
        error.set(None);
        username_error.set(None);
        password_error.set(None);

        let return_url = get_return_url();

        spawn_local(async move {
            let client = ApiClient::default();
            match client.login(&username_val, &password_val).await {
                Ok(response) => {
                    if response.success {
                        if let Some(token) = &response.access_token {
                            if let Some(window) = web_sys::window() {
                                if let Ok(Some(storage)) = window.local_storage() {
                                    let _ = storage.set_item("tachyon_token", token);
                                }
                            }
                            client.set_auth_token(token.clone());

                            if remember_val {
                                if let Some(window) = web_sys::window() {
                                    if let Ok(Some(storage)) = window.local_storage() {
                                        let _ = storage.set_item("tachyon_remember", "true");
                                    }
                                }
                            }
                        }
                        nav.update_value(|n| n(&return_url, Default::default()));
                    } else {
                        loading.set(false);
                        error.set(response.error);
                    }
                }
                Err(e) => {
                    loading.set(false);
                    error.set(Some(format!("Login failed: {}", e)));
                }
            }
        });
    };

    let on_guest_login = move |_| {
        loading.set(true);
        error.set(None);
        spawn_local(async move {
            let client = ApiClient::default();
            match client.guest_login().await {
                Ok(response) => {
                    if response.success {
                        if let Some(token) = &response.access_token {
                            if let Some(window) = web_sys::window() {
                                if let Ok(Some(storage)) = window.local_storage() {
                                    let _ = storage.set_item("tachyon_token", token);
                                }
                            }
                            client.set_auth_token(token.clone());
                        }
                        nav.update_value(|n| n("/dashboard", Default::default()));
                    } else {
                        loading.set(false);
                        error.set(response.error);
                    }
                }
                Err(e) => {
                    loading.set(false);
                    error.set(Some(format!("Guest login failed: {}", e)));
                }
            }
        });
    };

    let on_google_login = move |_| {
        error.set(Some(
            "Google OAuth is not yet configured. Please contact your administrator.".to_string(),
        ));
    };

    let on_github_login = move |_| {
        error.set(Some(
            "GitHub OAuth is not yet configured. Please contact your administrator.".to_string(),
        ));
    };

    view! {
        <div class="min-h-screen flex items-center justify-center bg-gray-50 dark:bg-gray-900 py-12 px-4 sm:px-6 lg:px-8">
            <div class="max-w-md w-full space-y-8">
                <div class="text-center">
                    <h1 class="text-4xl font-bold text-gray-900 dark:text-white">"Tachyon"</h1>
                    <h2 class="mt-2 text-lg text-gray-600 dark:text-gray-400">"Knowledge Management Platform"</h2>
                </div>

                <div class="bg-white dark:bg-gray-800 rounded-none shadow-md p-8 border border-gray-900 dark:border-gray-100">
                    <h3 class="text-xl font-semibold text-gray-900 dark:text-white mb-6">"Sign in to your account"</h3>

                    <Show when=move || username_error.get().is_some()>
                        <div id="username-error" role="alert" class="mb-4 p-3 bg-yellow-50 dark:bg-yellow-900/30 border border-yellow-300 dark:border-yellow-700 text-yellow-700 dark:text-yellow-200 rounded text-sm">
                            {move || username_error.get().unwrap_or_default()}
                        </div>
                    </Show>

                    <Show when=move || password_error.get().is_some()>
                        <div id="password-error" role="alert" class="mb-4 p-3 bg-yellow-50 dark:bg-yellow-900/30 border border-yellow-300 dark:border-yellow-700 text-yellow-700 dark:text-yellow-200 rounded text-sm">
                            {move || password_error.get().unwrap_or_default()}
                        </div>
                    </Show>

                    <Show when=move || error.get().is_some()>
                        <div id="login-error" role="alert" aria-live="polite" class="mb-4 p-3 bg-red-100 dark:bg-red-900 border border-red-400 dark:border-red-700 text-red-700 dark:text-red-200 rounded">
                            {move || error.get().unwrap_or_default()}
                        </div>
                    </Show>

                    <div class="space-y-3 mb-6">
                        <button
                            type="button"
                            on:click=on_google_login
                            class="w-full py-2.5 px-4 bg-white dark:bg-gray-700 border border-gray-300 dark:border-gray-600 rounded-none text-gray-700 dark:text-gray-300 font-medium hover:bg-gray-50 dark:hover:bg-gray-600 transition-colors flex items-center justify-center gap-2"
                        >
                            <svg class="w-5 h-5" viewBox="0 0 24 24" aria-hidden="true">
                                <path fill="#4285F4" d="M22.56 12.25c0-.78-.07-1.53-.2-2.25H12v4.26h5.92a5.06 5.06 0 0 1-2.2 3.32v2.77h3.57c2.08-1.92 3.28-4.74 3.28-8.1z"/>
                                <path fill="#34A853" d="M12 23c2.97 0 5.46-.98 7.28-2.66l-3.57-2.77c-.98.66-2.23 1.06-3.71 1.06-2.86 0-5.29-1.93-6.16-4.53H2.18v2.84C3.99 20.53 7.7 23 12 23z"/>
                                <path fill="#FBBC05" d="M5.84 14.09c-.22-.66-.35-1.36-.35-2.09s.13-1.43.35-2.09V7.07H2.18C1.43 8.55 1 10.22 1 12s.43 3.45 1.18 4.93l2.85-2.22.81-.62z"/>
                                <path fill="#EA4335" d="M12 5.38c1.62 0 3.06.56 4.21 1.64l3.15-3.15C17.45 2.09 14.97 1 12 1 7.7 1 3.99 3.47 2.18 7.07l3.66 2.84c.87-2.6 3.3-4.53 6.16-4.53z"/>
                            </svg>
                            "Sign in with Google"
                        </button>
                        <button
                            type="button"
                            on:click=on_github_login
                            class="w-full py-2.5 px-4 bg-gray-800 dark:bg-gray-700 border border-gray-600 dark:border-gray-600 rounded-none text-white font-medium hover:bg-gray-700 dark:hover:bg-gray-600 transition-colors flex items-center justify-center gap-2"
                        >
                            <svg class="w-5 h-5" fill="currentColor" viewBox="0 0 24 24" aria-hidden="true">
                                <path d="M12 0c-6.626 0-12 5.373-12 12 0 5.302 3.438 9.8 8.207 11.387.599.111.793-.261.793-.577v-2.234c-3.338.726-4.033-1.416-4.033-1.416-.546-1.387-1.333-1.756-1.333-1.756-1.089-.745.083-.729.083-.729 1.205.084 1.839 1.237 1.839 1.237 1.07 1.834 2.807 1.304 3.492.997.107-.775.418-1.305.762-1.604-2.665-.305-5.467-1.334-5.467-5.931 0-1.311.469-2.381 1.236-3.221-.124-.303-.535-1.524.117-3.176 0 0 1.008-.322 3.301 1.23.957-.266 1.983-.399 3.003-.404 1.02.005 2.047.138 3.006.404 2.291-1.552 3.297-1.23 3.297-1.23.653 1.653.242 2.874.118 3.176.77.84 1.235 1.911 1.235 3.221 0 4.609-2.807 5.624-5.479 5.921.43.372.823 1.102.823 2.222v3.293c0 .319.192.694.801.576 4.765-1.589 8.199-6.086 8.199-11.386 0-6.627-5.373-12-12-12z"/>
                            </svg>
                            "Sign in with GitHub"
                        </button>
                    </div>

                    <div class="relative mb-6">
                        <div class="absolute inset-0 flex items-center">
                            <div class="w-full border-t border-gray-300 dark:border-gray-600"></div>
                        </div>
                        <div class="relative flex justify-center text-sm">
                            <span class="px-2 bg-white dark:bg-gray-800 text-gray-500 dark:text-gray-400">"Or continue with email"</span>
                        </div>
                    </div>

                    <form on:submit=on_submit class="space-y-6">
                        <div>
                            <label for="username" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">"Username or Email"</label>
                            <input
                                id="username"
                                name="username"
                                type="text"
                                class="w-full px-4 py-3 border border-gray-300 dark:border-gray-600 rounded-none bg-white dark:bg-gray-700 text-gray-900 dark:text-white placeholder-gray-400 dark:placeholder-gray-500 focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                                placeholder="Enter your username"
                                required="true"
                                aria-required="true"
                                aria-describedby="username-error"
                                aria-invalid=move || if username_error.get().is_some() { "true" } else { "false" }
                                node_ref=username_ref
                                on:input=move |ev| username.set(event_target_value(&ev))
                                prop:value=move || username.get()
                            />
                        </div>

                        <div>
                            <label for="password" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">"Password"</label>
                            <input
                                id="password"
                                name="password"
                                type="password"
                                class="w-full px-4 py-3 border border-gray-300 dark:border-gray-600 rounded-none bg-white dark:bg-gray-700 text-gray-900 dark:text-white placeholder-gray-400 dark:placeholder-gray-500 focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                                placeholder="Enter your password"
                                required="true"
                                aria-required="true"
                                aria-describedby="password-error"
                                aria-invalid=move || if password_error.get().is_some() { "true" } else { "false" }
                                node_ref=password_ref
                                on:input=move |ev| password.set(event_target_value(&ev))
                                prop:value=move || password.get()
                            />
                        </div>

                        <div class="flex items-center justify-between">
                            <div class="flex items-center">
                                <input
                                    id="remember"
                                    type="checkbox"
                                    class="h-4 w-4 text-blue-600 focus:ring-blue-500 border-gray-300 rounded"
                                    on:change=move |ev| remember_me.set(event_target_checked(&ev))
                                    prop:checked=move || remember_me.get()
                                />
                                <label for="remember" class="ml-2 block text-sm text-gray-700 dark:text-gray-300">"Remember me"</label>
                            </div>
                            <a href="/login/forgot" class="text-sm text-blue-600 hover:text-blue-500 dark:text-blue-400 dark:hover:text-blue-300">"Forgot password?"</a>
                        </div>

                        <button
                            type="submit"
                            class="w-full py-3 px-4 bg-blue-600 hover:bg-blue-700 text-white font-medium rounded-none transition-colors focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2 disabled:opacity-50 disabled:cursor-not-allowed flex items-center justify-center"
                            disabled=move || loading.get()
                        >
                            {move || if loading.get() {
                                view! { <span class="flex items-center justify-center"><ButtonSpinner />"Signing in..."</span> }.into_any()
                            } else {
                                view! { "Sign in" }.into_any()
                            }}
                        </button>
                    </form>

                    <Show when=move || guest_login_enabled>
                        <div class="mt-6">
                            <div class="relative">
                                <div class="absolute inset-0 flex items-center">
                                    <div class="w-full border-t border-gray-300 dark:border-gray-600"></div>
                                </div>
                                <div class="relative flex justify-center text-sm">
                                    <span class="px-2 bg-white dark:bg-gray-800 text-gray-500 dark:text-gray-400">"Or"</span>
                                </div>
                            </div>

                            <div class="mt-4 space-y-3">
                                <button
                                    type="button"
                                    on:click=on_guest_login
                                    class="w-full py-3 px-4 bg-gray-100 dark:bg-gray-700 hover:bg-gray-200 dark:hover:bg-gray-600 text-gray-700 dark:text-gray-300 font-medium rounded-none transition-colors focus:outline-none focus:ring-2 focus:ring-gray-500 focus:ring-offset-2 disabled:opacity-50 flex items-center justify-center"
                                    disabled=move || loading.get()
                                >
                                    <span class="flex items-center justify-center">
                                        {move || if loading.get() {
                                            view! { <ButtonSpinner /> }.into_any()
                                        } else {
                                            view! {
                                                <svg class="w-5 h-5 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M16 7a4 4 0 11-8 0 4 4 0 018 0zM12 14a7 7 0 00-7 7h14a7 7 0 00-7-7z"></path>
                                                </svg>
                                            }.into_any()
                                        }}
                                        "Continue as Guest"
                                    </span>
                                </button>

                                <Show when=move || public_notes_enabled>
                                    <p class="text-center text-xs text-gray-500 dark:text-gray-400">
                                        "Public notes are enabled. Guests can view public content without logging in."
                                    </p>
                                </Show>
                            </div>
                        </div>
                    </Show>
                </div>

                <div class="bg-blue-50 dark:bg-blue-900/30 rounded-none p-4">
                    <h4 class="text-sm font-medium text-blue-800 dark:text-blue-300 mb-2">"Getting Started"</h4>
                    <div class="text-xs text-blue-700 dark:text-blue-400 space-y-1">
                        <p>"Register a new account or ask an administrator to create one for you."</p>
                        <p>"The first user registered on a fresh instance will be the admin."</p>
                    </div>
                </div>

                <div class="text-center space-y-2">
                    <p class="text-sm text-gray-500 dark:text-gray-400">
                        "Don't have an account? "
                        <a href="/register" class="text-blue-600 hover:underline dark:text-blue-400">"Create one"</a>
                    </p>
                    <a href="/" class="text-sm text-gray-500 dark:text-gray-400 hover:text-blue-600 dark:hover:text-blue-400">"← Back to Home"</a>
                </div>
            </div>
        </div>
    }
}
