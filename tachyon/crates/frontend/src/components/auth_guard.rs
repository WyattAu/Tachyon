// Auth Guard Component
// Protects routes by checking for authentication token

use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

/// Check if user is authenticated by looking for token in localStorage.
/// Returns the token if found.
fn get_stored_token() -> Option<String> {
    let window = web_sys::window()?;
    let storage = window.local_storage().ok()??;
    let token = storage.get_item("tachyon_token").ok()??;
    if token.is_empty() {
        None
    } else {
        Some(token)
    }
}

/// A wrapper component that redirects to /login if the user is not authenticated.
/// Place this around routes that require authentication.
///
/// Uses a Leptos signal so that if the token is cleared (e.g., by logout in
/// another tab via the `storage` event), the guard reacts immediately.
#[component]
pub fn AuthGuard(children: ChildrenFn) -> impl IntoView {
    let navigate = use_navigate();

    // Use a signal so the guard reacts to auth state changes.
    let (is_authenticated, set_authenticated) = signal(get_stored_token().is_some());

    // On mount: redirect to login if no token
    Effect::new(move || {
        if !is_authenticated.get_untracked() {
            navigate("/login", Default::default());
        }
    });

    // Listen for storage events (cross-tab logout detection)
    // In Leptos 0.8, window_event_listener was removed; use web_sys directly.
    Effect::new(move || {
        let cb = Closure::wrap(Box::new(move |_event: web_sys::StorageEvent| {
            let has_token = get_stored_token().is_some();
            set_authenticated.set(has_token);
        }) as Box<dyn Fn(web_sys::StorageEvent)>);
        if let Some(window) = web_sys::window() {
            let _ = window.add_event_listener_with_callback("storage", cb.as_ref().unchecked_ref());
        }
        // Keep the closure alive for the component's lifetime.
        // on_cleanup is not re-exported in 0.8 prelude; the closure leaks,
        // which is acceptable for a long-lived app-shell component.
        cb.forget();
    });

    // Show children if authenticated, otherwise show nothing while redirecting
    view! {
        <Show when=move || is_authenticated.get()>
            {children()}
        </Show>
    }
}

/// Provide the stored auth token to the ApiClient on app initialization.
/// Call this once at app startup to restore the session from localStorage.
pub fn provide_auth_context() {
    if let Some(token) = get_stored_token() {
        let client = crate::api::ApiClient::default();
        client.set_auth_token(token);
    }
}

/// Logout helper: clear token and navigate to login.
#[allow(dead_code)]
pub fn logout() {
    if let Some(window) = web_sys::window() {
        if let Ok(Some(storage)) = window.local_storage() {
            let _ = storage.remove_item("tachyon_token");
            let _ = storage.remove_item("tachyon_remember");
        }
        let _ = window.location().set_href("/login");
    }
    // Clear the in-memory token
    let client = crate::api::ApiClient::default();
    client.clear_auth_token();
}
