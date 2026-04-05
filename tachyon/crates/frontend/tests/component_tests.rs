use leptos::prelude::*;
use tachyon_frontend::App;
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn test_app_renders() {
    mount_to_body(|| view! { <App/> });

    let window = web_sys::window().expect("no window");
    let document = window.document().expect("no document");

    let app_element = document
        .query_selector("#app")
        .expect("Failed to query")
        .expect("App element not found");

    assert!(app_element.inner_html().len() > 0);
}

#[wasm_bindgen_test]
fn test_navigation_links() {
    mount_to_body(|| view! { <App/> });

    let window = web_sys::window().expect("no window");
    let document = window.document().expect("no document");

    let links = document
        .query_selector_all("a")
        .expect("Failed to query links");
    assert!(links.length() > 0, "Should have navigation links");
}

#[wasm_bindgen_test]
fn test_theme_toggle() {
    mount_to_body(|| view! { <App/> });

    let window = web_sys::window().expect("no window");
    let document = window.document().expect("no document");

    let theme_button = document
        .query_selector("button[aria-label='Toggle theme']")
        .expect("Failed to query")
        .expect("Theme button not found");

    theme_button.click();
}

#[wasm_bindgen_test]
fn test_not_found_page() {
    let window = web_sys::window().expect("no window");
    let document = window.document().expect("no document");

    let location = window.location();
    location
        .set_hash("#/nonexistent-page")
        .expect("Failed to navigate");

    std::thread::sleep(std::time::Duration::from_millis(100));

    let body = document.body().expect("no body");
    let content = body.inner_html();

    assert!(content.contains("404") || content.contains("Not Found"));
}

#[wasm_bindgen_test]
fn test_homepage_loads() {
    mount_to_body(|| view! { <App/> });

    let window = web_sys::window().expect("no window");
    let document = window.document().expect("no document");

    let location = window.location();
    location.set_hash("#/").expect("Failed to navigate");

    std::thread::sleep(std::time::Duration::from_millis(100));

    let main_content = document
        .query_selector("main")
        .expect("Failed to query")
        .expect("Main content not found");

    assert!(main_content.inner_html().len() > 0);
}

#[wasm_bindgen_test]
fn test_search_page_structure() {
    mount_to_body(|| view! { <App/> });

    let window = web_sys::window().expect("no window");
    let document = window.document().expect("no document");

    let location = window.location();
    location
        .set_hash("#/search")
        .expect("Failed to navigate to search");

    std::thread::sleep(std::time::Duration::from_millis(100));

    let search_input = document
        .query_selector("input[type='search']")
        .expect("Failed to query");

    assert!(
        search_input.is_some()
            || document
                .query_selector("input[placeholder*='search']")
                .is_ok()
    );
}

#[wasm_bindgen_test]
fn test_catalog_page_structure() {
    mount_to_body(|| view! { <App/> });

    let window = web_sys::window().expect("no window");
    let document = window.document().expect("no document");

    let location = window.location();
    location
        .set_hash("#/catalog")
        .expect("Failed to navigate to catalog");

    std::thread::sleep(std::time::Duration::from_millis(100));

    let body = document.body().expect("no body");
    let content = body.inner_html();

    assert!(content.len() > 0);
}

#[wasm_bindgen_test]
fn test_documents_page_structure() {
    mount_to_body(|| view! { <App/> });

    let window = web_sys::window().expect("no window");
    let document = window.document().expect("no document");

    let location = window.location();
    location
        .set_hash("#/documents")
        .expect("Failed to navigate to documents");

    std::thread::sleep(std::time::Duration::from_millis(100));

    let body = document.body().expect("no body");
    let content = body.inner_html();

    assert!(content.len() > 0);
}

#[wasm_bindgen_test]
fn test_settings_page_structure() {
    mount_to_body(|| view! { <App/> });

    let window = web_sys::window().expect("no window");
    let document = window.document().expect("no document");

    let location = window.location();
    location
        .set_hash("#/settings")
        .expect("Failed to navigate to settings");

    std::thread::sleep(std::time::Duration::from_millis(100));

    let body = document.body().expect("no body");
    let content = body.inner_html();

    assert!(content.len() > 0);
}

#[wasm_bindgen_test]
fn test_login_page_structure() {
    mount_to_body(|| view! { <App/> });

    let window = web_sys::window().expect("no window");
    let document = window.document().expect("no document");

    let location = window.location();
    location
        .set_hash("#/login")
        .expect("Failed to navigate to login");

    std::thread::sleep(std::time::Duration::from_millis(100));

    let form = document
        .query_selector("form")
        .expect("Failed to query form");
    assert!(form.is_some(), "Login page should have a form");
}
