// Main entry point for the Leptos application
//
// Allow dead code: many API client methods are defined here for future UI pages
// that are not yet implemented. Removing them would lose the API surface design.
#![allow(dead_code)]

mod api;
mod components;
mod i18n;
mod markdown;
mod offline;
mod pages;
mod storage;
mod styles;
pub mod sync_bridge;
mod types;
pub mod websocket;

use crate::api::ApiClient;
use components::{AppErrorBoundary, AppShell, AuthGuard, ThemeInitializer, provide_auth_context};
use leptos::prelude::*;
use leptos_router::components::*;
use leptos_router::path;
use storage::{BrowserStore, sync::SyncEngine};

// Mount the Leptos app to the browser DOM.
// wasm-bindgen calls this automatically when the WASM module loads.
#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn mount_app() {
    console_error_panic_hook::set_once();
    mount_to_body(App);
}

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

    let store = BrowserStore::new();
    provide_context(store.clone());

    let api = ApiClient::default();
    let sync_engine = SyncEngine::new(api, store);
    provide_context(sync_engine);

    // Theme signal - "light" or "dark"
    let (theme, set_theme) = signal("light".to_string());

    // Toggle theme function - also applies to document
    let toggle_theme = move || {
        let new_theme = if theme.get() == "light" {
            "dark"
        } else {
            "light"
        };
        set_theme.set(new_theme.to_string());

        // Apply theme to html element
        apply_theme_to_document(new_theme);

        // Also save to localStorage
        if let Some(window) = web_sys::window() {
            if let Ok(Some(storage)) = window.local_storage() {
                let _ = storage.set_item("tachyon-theme", new_theme);
            }
        }
    };

    view! {
        <AppErrorBoundary>
            <GlobalStyles />
            <ThemeInitializer />
            <Router>
                <AppShell theme=theme toggle_theme=toggle_theme>
                <Routes fallback=NotFound>
                    // Public routes (no auth required)
                    <Route path=path!("/") view=pages::HomePage />
                    <Route path=path!("/login") view=pages::LoginPage />
                    <Route path=path!("/register") view=pages::RegisterPage />

                    // Protected routes (require authentication)
                    <Route path=path!("/dashboard") view=move || {
                        view! {
                            <AuthGuard>
                                <AppErrorBoundary>
                                    <pages::DashboardPage />
                                </AppErrorBoundary>
                            </AuthGuard>
                        }
                    } />
                    <Route path=path!("/documents") view=move || {
                        view! {
                            <AuthGuard>
                                <AppErrorBoundary>
                                    <pages::DocumentsPage />
                                </AppErrorBoundary>
                            </AuthGuard>
                        }
                    } />
                    <Route path=path!("/documents/:id/edit") view=move || {
                        view! {
                            <AuthGuard>
                                <AppErrorBoundary>
                                    <pages::DocumentEditPage />
                                </AppErrorBoundary>
                            </AuthGuard>
                        }
                    } />
                    <Route path=path!("/documents/:id") view=move || {
                        view! {
                            <AuthGuard>
                                <AppErrorBoundary>
                                    <pages::DocumentPage />
                                </AppErrorBoundary>
                            </AuthGuard>
                        }
                    } />
                    <Route path=path!("/graph") view=move || {
                        view! {
                            <AuthGuard>
                                <AppErrorBoundary>
                                    <pages::GraphPage />
                                </AppErrorBoundary>
                            </AuthGuard>
                        }
                    } />
                    <Route path=path!("/teams") view=move || {
                        view! {
                            <AuthGuard>
                                <AppErrorBoundary>
                                    <pages::TeamsPage />
                                </AppErrorBoundary>
                            </AuthGuard>
                        }
                    } />
                    <Route path=path!("/teams/:team_id") view=move || {
                        view! {
                            <AuthGuard>
                                <AppErrorBoundary>
                                    <pages::TeamDetailPage />
                                </AppErrorBoundary>
                            </AuthGuard>
                        }
                    } />
                    <Route path=path!("/search") view=move || {
                        view! {
                            <AuthGuard>
                                <AppErrorBoundary>
                                    <pages::SearchPage />
                                </AppErrorBoundary>
                            </AuthGuard>
                        }
                    } />
                    <Route path=path!("/catalog") view=move || {
                        view! {
                            <AuthGuard>
                                <AppErrorBoundary>
                                    <pages::CatalogPage />
                                </AppErrorBoundary>
                            </AuthGuard>
                        }
                    } />
                    <Route path=path!("/tags") view=move || {
                        view! {
                            <AuthGuard>
                                <AppErrorBoundary>
                                    <pages::TagsPage />
                                </AppErrorBoundary>
                            </AuthGuard>
                        }
                    } />
                    <Route path=path!("/settings") view=move || {
                        view! {
                            <AuthGuard>
                                <AppErrorBoundary>
                                    <pages::SettingsPage />
                                </AppErrorBoundary>
                            </AuthGuard>
                        }
                    } />
                    <Route path=path!("/admin/roles") view=move || {
                        view! {
                            <AuthGuard>
                                <AppErrorBoundary>
                                    <pages::RolesPage />
                                </AppErrorBoundary>
                            </AuthGuard>
                        }
                    } />
                    <Route path=path!("/templates") view=move || {
                        view! {
                            <AuthGuard>
                                <AppErrorBoundary>
                                    <pages::TemplatesPage />
                                </AppErrorBoundary>
                            </AuthGuard>
                        }
                    } />
                    <Route path=path!("/plugins") view=move || {
                        view! {
                            <AuthGuard>
                                <AppErrorBoundary>
                                    <pages::PluginsPage />
                                </AppErrorBoundary>
                            </AuthGuard>
                        }
                    } />
                    <Route path=path!("/spaces") view=move || {
                        view! {
                            <AuthGuard>
                                <AppErrorBoundary>
                                    <pages::SpacesPage />
                                </AppErrorBoundary>
                            </AuthGuard>
                        }
                    } />
                    <Route path=path!("/ssg") view=move || {
                        view! {
                            <AuthGuard>
                                <AppErrorBoundary>
                                    <pages::SsgPage />
                                </AppErrorBoundary>
                            </AuthGuard>
                        }
                    } />
                    <Route path=path!("/billing") view=move || {
                        view! {
                            <AuthGuard>
                                <AppErrorBoundary>
                                    <pages::BillingPage />
                                </AppErrorBoundary>
                            </AuthGuard>
                        }
                    } />
                    <Route path=path!("/audit") view=move || {
                        view! {
                            <AuthGuard>
                                <AppErrorBoundary>
                                    <pages::AuditPage />
                                </AppErrorBoundary>
                            </AuthGuard>
                        }
                    } />
                    <Route path=path!("/profile") view=move || {
                        view! {
                            <AuthGuard>
                                <AppErrorBoundary>
                                    <pages::ProfilePage />
                                </AppErrorBoundary>
                            </AuthGuard>
                        }
                    } />
                    <Route path=path!("/onboarding") view=move || {
                        view! {
                            <AuthGuard>
                                <AppErrorBoundary>
                                    <pages::OnboardingPage />
                                </AppErrorBoundary>
                            </AuthGuard>
                        }
                    } />
                </Routes>
            </AppShell>
        </Router>
        </AppErrorBoundary>
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
