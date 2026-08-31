#![allow(dead_code, clippy::redundant_locals)]

use crate::api::ApiClient;
use leptos::prelude::*;
use wasm_bindgen_futures::spawn_local;

const TOTAL_STEPS: u32 = 5;

#[component]
pub fn OnboardingPage() -> impl IntoView {
    let (step, set_step) = signal(0u32);
    let (display_name, set_display_name) = signal(String::new());
    let (doc_title, set_doc_title) = signal("My First Document".to_string());
    let (invite_email, set_invite_email) = signal(String::new());
    let (invites, set_invites) = signal(Vec::<String>::new());
    let (selected_template, set_selected_template) = signal(None::<String>);
    let (templates, set_templates) = signal(Vec::<(String, String)>::new());
    let (loading_templates, set_loading_templates) = signal(false);

    let load_templates = move || {
        let tpls: Vec<(String, String)> = vec![
            (
                "Meeting Notes".to_string(),
                "A template for capturing meeting notes with action items.".to_string(),
            ),
            (
                "Project Brief".to_string(),
                "Define project goals, scope, and deliverables.".to_string(),
            ),
            (
                "Technical Spec".to_string(),
                "Document technical decisions and architecture.".to_string(),
            ),
            (
                "Weekly Report".to_string(),
                "Track weekly progress and accomplishments.".to_string(),
            ),
            (
                "Knowledge Base".to_string(),
                "Create structured documentation articles.".to_string(),
            ),
        ];
        spawn_local(async move {
            set_loading_templates.set(true);
            set_templates.set(tpls);
            set_loading_templates.set(false);
        });
    };

    let next_step = move |_: leptos::ev::MouseEvent| {
        let email = invite_email.get();
        if !email.is_empty() && email.contains('@') && !invites.get().contains(&email) {
            set_invites.update(|v: &mut Vec<String>| {
                v.push(email);
            });
            set_invite_email.set(String::new());
        }
        set_step.update(|s| {
            if *s < TOTAL_STEPS - 1 {
                *s += 1;
            }
        });
    };

    let prev_step = move |_: leptos::ev::MouseEvent| {
        set_step.update(|s| {
            if *s > 0 {
                *s -= 1;
            }
        });
    };

    let add_invite = move |_: leptos::ev::MouseEvent| {
        let email = invite_email.get();
        if email.contains('@') && !invites.get().contains(&email) {
            set_invites.update(|v: &mut Vec<String>| {
                v.push(email);
            });
            set_invite_email.set(String::new());
        }
    };

    let remove_invite = move |email: String| {
        set_invites.update(|v: &mut Vec<String>| {
            v.retain(|e| e != &email);
        });
    };

    let on_finish = move |_: leptos::ev::MouseEvent| {
        if let Some(window) = web_sys::window() {
            if let Some(storage) = window.local_storage().ok().flatten() {
                let _ = storage.set_item("tachyon_onboarding_complete", "true");
            }
        }
        let name = display_name.get();
        if !name.is_empty() {
            let client = ApiClient::default();
            spawn_local(async move {
                let _ = client.update_profile(Some(&name), None).await;
            });
        }
        if let Some(window) = web_sys::window() {
            let _ = window.location().set_href("/dashboard");
        }
    };

    let on_skip = move |_: leptos::ev::MouseEvent| {
        if let Some(window) = web_sys::window() {
            if let Some(storage) = window.local_storage().ok().flatten() {
                let _ = storage.set_item("tachyon_onboarding_complete", "true");
            }
        }
        if let Some(window) = web_sys::window() {
            let _ = window.location().set_href("/dashboard");
        }
    };

    let current_step = step.get();
    Effect::new(move |_| {
        if current_step >= 3 && templates.get().is_empty() {
            load_templates();
        }
    });

    view! {
        <div class="min-h-screen bg-gray-50 dark:bg-gray-900 flex items-center justify-center p-4">
            <div class="bg-white dark:bg-gray-800 rounded-none shadow-2xl max-w-xl w-full overflow-hidden border border-gray-900 dark:border-gray-100">
                <div class="h-1 bg-gray-200 dark:bg-gray-700">
                    <div
                        class="h-1 bg-blue-600 transition-all duration-300"
                        style={move || format!("width: {}%", ((step.get() + 1) as f32 / TOTAL_STEPS as f32) * 100.0)}
                    ></div>
                </div>

                <div class="p-8">
                    <div class="flex items-center justify-between mb-6">
                        <h1 class="text-sm font-medium text-gray-500 dark:text-gray-400">
                            {move || format!("Step {} of {}", step.get() + 1, TOTAL_STEPS)}
                        </h1>
                        <button
                            on:click=on_skip
                            class="text-sm text-gray-400 hover:text-gray-600 dark:hover:text-gray-300 transition-colors"
                        >
                            "Skip setup"
                        </button>
                    </div>

                    {move || match step.get() {
                        0 => view! {
                            <div class="text-center">
                                <div class="w-16 h-16 bg-blue-100 dark:bg-blue-900/30 rounded-full flex items-center justify-center mx-auto mb-6">
                                    <svg class="w-8 h-8 text-blue-600 dark:text-blue-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 10V3L4 14h7v7l9-11h-7z" />
                                    </svg>
                                </div>
                                <h2 class="text-2xl font-bold text-gray-900 dark:text-gray-100 mb-3">"Welcome to Tachyon"</h2>
                                <p class="text-gray-600 dark:text-gray-400 mb-8 max-w-sm mx-auto">
                                    "A fast, offline-first knowledge management system. Let's get you set up in a few quick steps."
                                </p>
                                <div class="grid grid-cols-2 gap-4 text-left">
                                    <div class="p-3 bg-gray-50 dark:bg-gray-700/50 rounded-none">
                                        <div class="text-sm font-medium text-gray-900 dark:text-white">"Markdown-first"</div>
                                        <div class="text-xs text-gray-500 dark:text-gray-400">"Write in markdown with rich preview"</div>
                                    </div>
                                    <div class="p-3 bg-gray-50 dark:bg-gray-700/50 rounded-none">
                                        <div class="text-sm font-medium text-gray-900 dark:text-white">"Real-time collab"</div>
                                        <div class="text-xs text-gray-500 dark:text-gray-400">"CRDT-based sync with your team"</div>
                                    </div>
                                    <div class="p-3 bg-gray-50 dark:bg-gray-700/50 rounded-none">
                                        <div class="text-sm font-medium text-gray-900 dark:text-white">"Works offline"</div>
                                        <div class="text-xs text-gray-500 dark:text-gray-400">"Edit anywhere, sync when connected"</div>
                                    </div>
                                    <div class="p-3 bg-gray-50 dark:bg-gray-700/50 rounded-none">
                                        <div class="text-sm font-medium text-gray-900 dark:text-white">"Knowledge graph"</div>
                                        <div class="text-xs text-gray-500 dark:text-gray-400">"Bidirectional links between docs"</div>
                                    </div>
                                </div>
                            </div>
                        }.into_any(),
                        1 => view! {
                            <div>
                                <div class="flex items-center gap-3 mb-6">
                                    <div class="w-10 h-10 bg-blue-100 dark:bg-blue-900/30 rounded-full flex items-center justify-center">
                                        <svg class="w-5 h-5 text-blue-600 dark:text-blue-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M16 7a4 4 0 11-8 0 4 4 0 018 0zM12 14a7 7 0 00-7 7h14a7 7 0 00-7-7z" />
                                        </svg>
                                    </div>
                                    <div>
                                        <h2 class="text-xl font-bold text-gray-900 dark:text-gray-100">"What should we call you?"</h2>
                                        <p class="text-sm text-gray-500 dark:text-gray-400">"This will be shown to collaborators."</p>
                                    </div>
                                </div>
                                <div>
                                    <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">"Display Name"</label>
                                    <input
                                        type="text"
                                        placeholder="Your name"
                                        prop:value={move || display_name.get()}
                                        on:input=move |ev| set_display_name.set(event_target_value(&ev))
                                        class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-none bg-white dark:bg-gray-800 text-gray-900 dark:text-gray-100 focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
                                    />
                                </div>
                            </div>
                        }.into_any(),
                        2 => view! {
                            <div>
                                <div class="flex items-center gap-3 mb-6">
                                    <div class="w-10 h-10 bg-blue-100 dark:bg-blue-900/30 rounded-full flex items-center justify-center">
                                        <svg class="w-5 h-5 text-blue-600 dark:text-blue-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
                                        </svg>
                                    </div>
                                    <div>
                                        <h2 class="text-xl font-bold text-gray-900 dark:text-gray-100">"Create Your First Document"</h2>
                                        <p class="text-sm text-gray-500 dark:text-gray-400">"Give it a title to get started."</p>
                                    </div>
                                </div>
                                <div class="space-y-4">
                                    <div>
                                        <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">"Document Title"</label>
                                        <input
                                            type="text"
                                            prop:value={move || doc_title.get()}
                                            on:input=move |ev| set_doc_title.set(event_target_value(&ev))
                                            class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-none bg-white dark:bg-gray-800 text-gray-900 dark:text-gray-100 focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
                                        />
                                    </div>
                                    <div class="flex gap-3">
                                        <a
                                            href="/documents/new"
                                            class="flex-1 text-center px-4 py-2 bg-blue-600 hover:bg-blue-700 text-white rounded-none transition-colors text-sm font-medium"
                                        >
                                            "Create Document"
                                        </a>
                                        <a
                                            href="/documents"
                                            class="flex-1 text-center px-4 py-2 border border-gray-300 dark:border-gray-600 hover:bg-gray-50 dark:hover:bg-gray-700 text-gray-700 dark:text-gray-300 rounded-none transition-colors text-sm font-medium"
                                        >
                                            "Or import"
                                        </a>
                                    </div>
                                </div>
                            </div>
                        }.into_any(),
                        3 => view! {
                            <div>
                                <div class="flex items-center gap-3 mb-6">
                                    <div class="w-10 h-10 bg-blue-100 dark:bg-blue-900/30 rounded-full flex items-center justify-center">
                                        <svg class="w-5 h-5 text-blue-600 dark:text-blue-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M17 20h5v-2a3 3 0 00-5.356-1.857M17 20H7m10 0v-2c0-.656-.126-1.283-.356-1.857M7 20H2v-2a3 3 0 015.356-1.857M7 20v-2c0-.656.126-1.283.356-1.857m0 0a5.002 5.002 0 019.288 0M15 7a3 3 0 11-6 0 3 3 0 016 0zm6 3a2 2 0 11-4 0 2 2 0 014 0zM7 10a2 2 0 11-4 0 2 2 0 014 0z" />
                                        </svg>
                                    </div>
                                    <div>
                                        <h2 class="text-xl font-bold text-gray-900 dark:text-gray-100">"Invite Collaborators"</h2>
                                        <p class="text-sm text-gray-500 dark:text-gray-400">"Optional - you can always invite people later."</p>
                                    </div>
                                </div>
                                <div class="space-y-4">
                                    <div class="flex gap-2">
                                        <input
                                            type="email"
                                            placeholder="colleague@example.com"
                                            prop:value={move || invite_email.get()}
                                            on:input=move |ev| set_invite_email.set(event_target_value(&ev))
                                            on:keydown=move |ev: leptos::ev::KeyboardEvent| {
                                                if ev.key() == "Enter" {
                                                    ev.prevent_default();
                                                    let email = invite_email.get();
                                                    if email.contains('@') && !invites.get().contains(&email) {
                                                        set_invites.update(|v: &mut Vec<String>| { v.push(email); });
                                                        set_invite_email.set(String::new());
                                                    }
                                                }
                                            }
                                            class="flex-1 px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-none bg-white dark:bg-gray-800 text-gray-900 dark:text-gray-100 focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
                                        />
                                        <button
                                            on:click=add_invite
                                            class="px-4 py-2 bg-blue-600 hover:bg-blue-700 text-white rounded-none transition-colors text-sm font-medium"
                                        >
                                            "Add"
                                        </button>
                                    </div>
                                    {move || {
                                        let invs = invites.get();
                                        if invs.is_empty() {
                                            view! {
                                                <p class="text-sm text-gray-400 text-center py-4">"No collaborators added yet"</p>
                                            }.into_any()
                                        } else {
                                            view! {
                                                <div class="space-y-2">
                                                    {invs.into_iter().map(|email| {
                                                        let email_clone = email.clone();
                                                        view! {
                                                            <div class="flex items-center justify-between px-3 py-2 bg-gray-50 dark:bg-gray-700/50 rounded-none">
                                                                <span class="text-sm text-gray-700 dark:text-gray-300">{email}</span>
                                                                <button on:click=move |_| remove_invite(email_clone.clone()) class="text-gray-400 hover:text-red-500 transition-colors">
                                                                    <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                                                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
                                                                    </svg>
                                                                </button>
                                                            </div>
                                                        }
                                                    }).collect::<Vec<_>>()}
                                                </div>
                                            }.into_any()
                                        }
                                    }}
                                </div>
                            </div>
                        }.into_any(),
                        4 => view! {
                            <div>
                                <div class="flex items-center gap-3 mb-6">
                                    <div class="w-10 h-10 bg-blue-100 dark:bg-blue-900/30 rounded-full flex items-center justify-center">
                                        <svg class="w-5 h-5 text-blue-600 dark:text-blue-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 5a1 1 0 011-1h14a1 1 0 011 1v2a1 1 0 01-1 1H5a1 1 0 01-1-1V5zM4 13a1 1 0 011-1h6a1 1 0 011 1v6a1 1 0 01-1 1H5a1 1 0 01-1-1v-6zM16 13a1 1 0 011-1h2a1 1 0 011 1v6a1 1 0 01-1 1h-2a1 1 0 01-1-1v-6z" />
                                        </svg>
                                    </div>
                                    <div>
                                        <h2 class="text-xl font-bold text-gray-900 dark:text-gray-100">"Choose a Template"</h2>
                                        <p class="text-sm text-gray-500 dark:text-gray-400">"Optional - start with a pre-built template."</p>
                                    </div>
                                </div>
                                {move || if loading_templates.get() {
                                    view! {
                                        <div class="flex justify-center py-8">
                                            <div class="animate-spin rounded-full h-6 w-6 border-b-2 border-blue-600"></div>
                                        </div>
                                    }.into_any()
                                } else {
                                    view! {
                                        <div class="grid grid-cols-1 gap-3">
                                            <button
                                                on:click=move |_| set_selected_template.set(None)
                                                class={
                                                    move || {
                                                        let base = "text-left p-4 rounded-none border-2 transition-colors";
                                                        if selected_template.get().is_none() {
                                                            format!("{} border-blue-500 bg-blue-50 dark:bg-blue-900/20", base)
                                                        } else {
                                                            format!("{} border-gray-200 dark:border-gray-700 hover:border-gray-300 dark:hover:border-gray-600", base)
                                                        }
                                                    }
                                                }
                                            >
                                                <div class="text-sm font-medium text-gray-900 dark:text-white">"Blank Document"</div>
                                                <div class="text-xs text-gray-500 dark:text-gray-400">"Start from scratch"</div>
                                            </button>
                                            {templates.get().into_iter().map(|(name, desc)| {
                                                let name_clone = name.clone();
                                                let sel = selected_template.get();
                                                let is_selected = sel.as_ref() == Some(&name);
                                                view! {
                                                    <button
                                                        on:click=move |_| set_selected_template.set(Some(name_clone.clone()))
                                                        class={
                                                            if is_selected {
                                                                "text-left p-4 rounded-none border-2 border-blue-500 bg-blue-50 dark:bg-blue-900/20 transition-colors"
                                                            } else {
                                                                "text-left p-4 rounded-none border-2 border-gray-200 dark:border-gray-700 hover:border-gray-300 dark:hover:border-gray-600 transition-colors"
                                                            }
                                                        }
                                                    >
                                                        <div class="text-sm font-medium text-gray-900 dark:text-white">{name}</div>
                                                        <div class="text-xs text-gray-500 dark:text-gray-400">{desc}</div>
                                                    </button>
                                                }
                                            }).collect::<Vec<_>>()}
                                        </div>
                                    }.into_any()
                                }}
                            </div>
                        }.into_any(),
                        _ => ().into_any(),
                    }}
                </div>

                {move || {
                    let current = step.get();
                    if current < TOTAL_STEPS - 1 {
                        view! {
                            <div class="px-8 pb-6 flex items-center justify-between">
                                <div>
                                    {move || if current > 0 {
                                        view! {
                                            <button on:click=prev_step class="px-4 py-2 text-sm text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-gray-100 transition-colors">
                                                "Back"
                                            </button>
                                        }.into_any()
                                    } else {
                                        view! { <span></span> }.into_any()
                                    }}
                                </div>
                                <button
                                    on:click=next_step
                                    class="px-6 py-2 bg-blue-600 hover:bg-blue-700 text-white rounded-none transition-colors text-sm font-medium"
                                >
                                    "Continue"
                                </button>
                            </div>
                        }.into_any()
                    } else {
                        view! {
                            <div class="px-8 pb-6 flex items-center justify-between">
                                <button on:click=prev_step class="px-4 py-2 text-sm text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-gray-100 transition-colors">
                                    "Back"
                                </button>
                                <button
                                    on:click=on_finish
                                    class="px-6 py-2 bg-green-600 hover:bg-green-700 text-white rounded-none transition-colors text-sm font-medium"
                                >
                                    "Get Started"
                                </button>
                            </div>
                        }.into_any()
                    }
                }}
            </div>
        </div>
    }
}
