use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;

fn get_stored_token() -> Option<String> {
    let window = web_sys::window()?;
    let storage = window.local_storage().ok()??;
    let token = storage.get_item("tachyon_token").ok()??;
    if token.is_empty() { None } else { Some(token) }
}

#[component]
pub fn AuthGuard(children: ChildrenFn) -> impl IntoView {
    let navigate = use_navigate();

    let (is_authenticated, set_authenticated) = signal(get_stored_token().is_some());
    let (checked, set_checked) = signal(false);

    Effect::new(move || {
        if checked.get_untracked() {
            return;
        }

        let has_token = get_stored_token().is_some();
        set_authenticated.set(has_token);
        set_checked.set(true);

        if !has_token {
            let path = web_sys::window()
                .and_then(|w| w.location().pathname().ok())
                .unwrap_or_else(|| "/".to_string());
            let search = web_sys::window()
                .and_then(|w| w.location().search().ok())
                .unwrap_or_default();

            if path != "/login" && path != "/register" {
                let return_url = format!("{}{}", path, search);
                let encoded = return_url
                    .replace('%', "%25")
                    .replace('?', "%3F")
                    .replace('&', "%26")
                    .replace('=', "%3D")
                    .replace('#', "%23");
                navigate(&format!("/login?return={}", encoded), Default::default());
            } else {
                navigate("/login", Default::default());
            }
        }
    });

    Effect::new(move || {
        let cb = Closure::wrap(Box::new(move |_event: web_sys::StorageEvent| {
            let has_token = get_stored_token().is_some();
            set_authenticated.set(has_token);
        }) as Box<dyn Fn(web_sys::StorageEvent)>);
        if let Some(window) = web_sys::window() {
            let _ = window.add_event_listener_with_callback("storage", cb.as_ref().unchecked_ref());
        }
        cb.forget();
    });

    view! {
        <Show when=move || checked.get() && is_authenticated.get()>
            {children()}
        </Show>
        <Show when=move || !checked.get()>
            <div class="flex items-center justify-center min-h-[200px]">
                <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600"></div>
            </div>
        </Show>
    }
}

pub fn provide_auth_context() {
    let _ = get_stored_token();
}

/// Clear authentication state and redirect to login.
///
/// Callers should invoke this from a navigation component (e.g. the
/// top-bar user menu) to provide a logout action. The function itself
/// is intentionally decoupled from any specific UI element so that
/// multiple entry-points can trigger it.
pub fn logout() {
    if let Some(window) = web_sys::window() {
        if let Ok(Some(storage)) = window.local_storage() {
            let _ = storage.remove_item("tachyon_token");
            let _ = storage.remove_item("tachyon_remember");
        }
        let _ = window.location().set_href("/login");
    }
}

pub fn get_user_id() -> Option<String> {
    if let Some(window) = web_sys::window() {
        if let Ok(Some(storage)) = window.local_storage() {
            if let Ok(Some(token)) = storage.get_item("tachyon_token") {
                return Some(format!("user-{}", &token[..token.len().min(8)]));
            }
        }
    }
    None
}
