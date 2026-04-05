// Main entry point for the Leptos application

mod api;
mod components;
mod pages;
mod types;
mod styles;
pub mod websocket;

use leptos::prelude::*;
use leptos_router::components::*;
use leptos_router::path;
use components::AppShell;
use styles::GlobalStyles;

/// Not found page component
#[component]
fn NotFound() -> impl IntoView {
    view! {
        <div class="p-8 text-center">
            <h1 class="text-2xl font-bold text-gray-900 dark:text-white">404 - Not Found</h1>
            <p class="mt-2 text-gray-500">The requested page does not exist.</p>
            <a href="/" class="mt-4 inline-block text-blue-600 hover:underline">Go Home</a>
        </div>
    }
}

/// Main application component
#[component]
pub fn App() -> impl IntoView {
    // Theme signal - "light" or "dark"
    let (theme, set_theme) = signal("light".to_string());

    // Toggle theme function - also applies to document
    let toggle_theme = move || {
        let new_theme = if theme.get() == "light" { "dark" } else { "light" };
        set_theme.set(new_theme.to_string());
        
        // Apply theme to html element
        apply_theme_to_document(&new_theme);
        
        // Also save to localStorage
        if let Some(window) = web_sys::window() {
            if let Ok(Some(storage)) = window.local_storage() {
                let _ = storage.set_item("tachyon-theme", &new_theme);
            }
        }
    };

    view! {
        <GlobalStyles />
        <Router>
            <AppShell theme=theme toggle_theme=toggle_theme>
                <Routes fallback=NotFound>
                    <Route path=path!("/") view=pages::HomePage />
                    <Route path=path!("/dashboard") view=pages::DashboardPage />
                    <Route path=path!("/documents") view=pages::DocumentsPage />
                    <Route path=path!("/documents/:id/edit") view=pages::DocumentEditPage />
                    <Route path=path!("/teams") view=pages::TeamsPage />
                    <Route path=path!("/search") view=pages::SearchPage />
                    <Route path=path!("/catalog") view=pages::CatalogPage />
                    <Route path=path!("/settings") view=pages::SettingsPage />
                    <Route path=path!("/admin/roles") view=pages::RolesPage />
                    <Route path=path!("/login") view=pages::LoginPage />
                </Routes>
            </AppShell>
        </Router>
    }
}

/// Apply theme to the document's html element
fn apply_theme_to_document(theme: &str) {
    // This runs in the browser
    if let Some(window) = web_sys::window() {
        if let Some(document) = window.document() {
            if let Some(html) = document.document_element() {
                if theme == "dark" {
                    let _ = html.class_list().add_1("dark");
                } else {
                    let _ = html.class_list().remove_1("dark");
                }
            }
        }
    }
}
