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
use components::{AppShell, AuthGuard, provide_auth_context};

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
    // Restore auth token from localStorage on app startup
    provide_auth_context();

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
                    // Public routes (no auth required)
                    <Route path=path!("/") view=pages::HomePage />
                    <Route path=path!("/login") view=pages::LoginPage />

                    // Protected routes (require authentication)
                    <Route path=path!("/dashboard") view=move || {
                        view! {
                            <AuthGuard>
                                <pages::DashboardPage />
                            </AuthGuard>
                        }
                    } />
                    <Route path=path!("/documents") view=move || {
                        view! {
                            <AuthGuard>
                                <pages::DocumentsPage />
                            </AuthGuard>
                        }
                    } />
                    <Route path=path!("/documents/:id/edit") view=move || {
                        view! {
                            <AuthGuard>
                                <pages::DocumentEditPage />
                            </AuthGuard>
                        }
                    } />
                    <Route path=path!("/teams") view=move || {
                        view! {
                            <AuthGuard>
                                <pages::TeamsPage />
                            </AuthGuard>
                        }
                    } />
                    <Route path=path!("/search") view=move || {
                        view! {
                            <AuthGuard>
                                <pages::SearchPage />
                            </AuthGuard>
                        }
                    } />
                    <Route path=path!("/catalog") view=move || {
                        view! {
                            <AuthGuard>
                                <pages::CatalogPage />
                            </AuthGuard>
                        }
                    } />
                    <Route path=path!("/settings") view=move || {
                        view! {
                            <AuthGuard>
                                <pages::SettingsPage />
                            </AuthGuard>
                        }
                    } />
                    <Route path=path!("/admin/roles") view=move || {
                        view! {
                            <AuthGuard>
                                <pages::RolesPage />
                            </AuthGuard>
                        }
                    } />
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

use styles::GlobalStyles;
