// Login Page
// Authentication page with guest login and public notes support

use leptos::prelude::*;
use leptos::ev;
use leptos_router::hooks::use_navigate;
use leptos::task::spawn_local;
use crate::api::ApiClient;

/// Login page component
#[component]
pub fn LoginPage() -> impl IntoView {
    // Form state using RwSignal for interior mutability
    let username = RwSignal::new(String::new());
    let password = RwSignal::new(String::new());
    let error = RwSignal::new(None::<String>);
    let loading = RwSignal::new(false);
    let remember_me = RwSignal::new(false);
    
    // Configuration
    let guest_login_enabled = true;
    let public_notes_enabled = true;
    
    // Navigation - wrap in StoredValue for use in spawn_local
    let nav = StoredValue::new(use_navigate());
    
    // Handle form submission
    let on_submit = move |ev: ev::SubmitEvent| {
        ev.prevent_default();
        
        // Get current values
        let username_val = username.get();
        let password_val = password.get();
        let remember_val = remember_me.get();
        
        // Set loading state
        loading.set(true);
        error.set(None);
        
        // Spawn async task
        spawn_local(async move {
            let client = ApiClient::default();
            
            match client.login(&username_val, &password_val).await {
                Ok(response) => {
                    if response.success {
                        if let Some(token) = &response.access_token {
                            client.set_auth_token(token.clone());
                            
                            if remember_val {
                                if let Some(window) = web_sys::window() {
                                    if let Ok(Some(storage)) = window.local_storage() {
                                        let _ = storage.set_item("tachyon_token", token);
                                        let _ = storage.set_item("tachyon_remember", "true");
                                    }
                                }
                            }
                        }
                        
                        // Navigate using stored value
                        nav.update_value(|n| n("/catalog", Default::default()));
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
    
    // Handle guest login
    let on_guest_login = move |_| {
        loading.set(true);
        error.set(None);
        
        spawn_local(async move {
            let client = ApiClient::default();
            
            match client.guest_login().await {
                Ok(response) => {
                    if response.success {
                        if let Some(token) = &response.access_token {
                            client.set_auth_token(token.clone());
                        }
                        nav.update_value(|n| n("/catalog", Default::default()));
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

    view! {
        <div class="min-h-screen flex items-center justify-center bg-gray-50 dark:bg-gray-900 py-12 px-4 sm:px-6 lg:px-8">
            <div class="max-w-md w-full space-y-8">
                <div class="text-center">
                    <h1 class="text-4xl font-bold text-gray-900 dark:text-white">"Tachyon"</h1>
                    <h2 class="mt-2 text-lg text-gray-600 dark:text-gray-400">"Knowledge Management Platform"</h2>
                </div>
                
                <div class="bg-white dark:bg-gray-800 rounded-lg shadow-md p-8">
                    <h3 class="text-xl font-semibold text-gray-900 dark:text-white mb-6">"Sign in to your account"</h3>
                    
                    <Show when=move || error.get().is_some()>
                        <div class="mb-4 p-3 bg-red-100 dark:bg-red-900 border border-red-400 dark:border-red-700 text-red-700 dark:text-red-200 rounded">
                            {move || error.get().unwrap_or_default()}
                        </div>
                    </Show>
                    
                    <form on:submit=on_submit class="space-y-6">
                        <div>
                            <label for="username" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">"Username or Email"</label>
                            <input
                                id="username"
                                type="text"
                                class="w-full px-4 py-3 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-white placeholder-gray-400 dark:placeholder-gray-500 focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                                placeholder="Enter your username"
                                on:input=move |ev| username.set(event_target_value(&ev))
                                prop:value=move || username.get()
                            />
                        </div>
                        
                        <div>
                            <label for="password" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">"Password"</label>
                            <input
                                id="password"
                                type="password"
                                class="w-full px-4 py-3 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-white placeholder-gray-400 dark:placeholder-gray-500 focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                                placeholder="Enter your password"
                                on:input=move |ev| password.set(event_target_value(&ev))
                                prop:value=move || password.get()
                            />
                        </div>
                        
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
                        
                        <button
                            type="submit"
                            class="w-full py-3 px-4 bg-blue-600 hover:bg-blue-700 text-white font-medium rounded-lg transition-colors focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2 disabled:opacity-50 disabled:cursor-not-allowed"
                            disabled=move || loading.get()
                        >
                            {move || if loading.get() { "Signing in..." } else { "Sign in" }}
                        </button>
                    </form>
                    
                    <Show when=move || guest_login_enabled>
                        <div class="mt-6">
                            <div class="relative">
                                <div class="absolute inset-0 flex items-center">
                                    <div class="w-full border-t border-gray-300 dark:border-gray-600"></div>
                                </div>
                                <div class="relative flex justify-center text-sm">
                                    <span class="px-2 bg-white dark:bg-gray-800 text-gray-500 dark:text-gray-400">"Or continue with"</span>
                                </div>
                            </div>
                            
                            <div class="mt-4 space-y-3">
                                <button
                                    type="button"
                                    on:click=on_guest_login
                                    class="w-full py-3 px-4 bg-gray-100 dark:bg-gray-700 hover:bg-gray-200 dark:hover:bg-gray-600 text-gray-700 dark:text-gray-300 font-medium rounded-lg transition-colors focus:outline-none focus:ring-2 focus:ring-gray-500 focus:ring-offset-2 disabled:opacity-50"
                                    disabled=move || loading.get()
                                >
                                    <span class="flex items-center justify-center">
                                        <svg class="w-5 h-5 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M16 7a4 4 0 11-8 0 4 4 0 018 0zM12 14a7 7 0 00-7 7h14a7 7 0 00-7-7z"></path>
                                        </svg>
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
                
                <div class="bg-blue-50 dark:bg-blue-900/30 rounded-lg p-4">
                    <h4 class="text-sm font-medium text-blue-800 dark:text-blue-300 mb-2">"Getting Started"</h4>
                    <div class="text-xs text-blue-700 dark:text-blue-400 space-y-1">
                        <p>"Register a new account or ask an administrator to create one for you."</p>
                        <p>"The first user registered on a fresh instance will be the admin."</p>
                    </div>
                </div>
                
                <div class="text-center">
                    <a href="/" class="text-sm text-gray-500 dark:text-gray-400 hover:text-blue-600 dark:hover:text-blue-400">"← Back to Home"</a>
                </div>
            </div>
        </div>
    }
}
