// Onboarding flow component
// Shows a multi-step wizard on first visit to guide new users
// through setting up their Tachyon workspace.

use leptos::prelude::*;

/// Onboarding wizard shown on first visit.
/// Steps: Welcome -> Create First Doc -> Invite Team -> Customize -> Done
///
/// Reserved for future use: first-visit onboarding flow.
#[component]
pub fn OnboardingWizard(#[prop(optional)] on_complete: Option<Callback<()>>) -> impl IntoView {
    let (step, set_step) = signal(0u32);
    let (display_name, set_display_name) = signal(String::new());
    let (workspace_name, set_workspace_name) = signal("My Workspace".to_string());

    let on_finish = move |_| {
        // Mark onboarding as complete
        if let Some(window) = web_sys::window() {
            if let Some(storage) = window.local_storage().ok().flatten() {
                let _ = storage.set_item("tachyon_onboarding_complete", "true");
            }
        }
        if let Some(cb) = on_complete {
            cb.run(());
        }
    };

    let on_skip = move |_| {
        if let Some(window) = web_sys::window() {
            if let Some(storage) = window.local_storage().ok().flatten() {
                let _ = storage.set_item("tachyon_onboarding_complete", "true");
            }
        }
        if let Some(cb) = on_complete {
            cb.run(());
        }
    };

    let next_step = move |_| {
        set_step.update(|s| *s += 1);
    };

    let prev_step = move |_| {
        set_step.update(|s| {
            if *s > 0 {
                *s -= 1;
            }
        });
    };

    let is_last_step = move || step.get() == 3;
    let is_first_step = move || step.get() == 0;

    view! {
        <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/50">
            <div class="bg-white dark:bg-gray-900 rounded-2xl shadow-2xl max-w-lg w-full mx-4 overflow-hidden">
                // Progress bar
                <div class="h-1 bg-gray-200 dark:bg-gray-700">
                    <div
                        class="h-1 bg-blue-600 transition-all duration-300"
                        style={move || format!("width: {}%", ((step.get() + 1) as f32 / 4.0) * 100.0)}
                    ></div>
                </div>

                // Content area
                <div class="p-8">
                    {move || match step.get() {
                        0 => view! {
                            <div class="text-center">
                                <div class="w-16 h-16 bg-blue-100 dark:bg-blue-900/30 rounded-full flex items-center justify-center mx-auto mb-4">
                                    <svg class="w-8 h-8 text-blue-600 dark:text-blue-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
                                    </svg>
                                </div>
                                <h2 class="text-2xl font-bold text-gray-900 dark:text-gray-100 mb-2">"Welcome to Tachyon"</h2>
                                <p class="text-gray-600 dark:text-gray-400 mb-6">
                                    "A fast, offline-first knowledge management system. Let's get you set up in a few quick steps."
                                </p>
                                <div class="text-left bg-gray-50 dark:bg-gray-800 rounded-lg p-4 space-y-2">
                                    <div class="flex items-center gap-2 text-sm text-gray-700 dark:text-gray-300">
                                        <span class="text-green-500">"- "</span>
                                        <span>"Markdown-first editing with rich preview"</span>
                                    </div>
                                    <div class="flex items-center gap-2 text-sm text-gray-700 dark:text-gray-300">
                                        <span class="text-green-500">"- "</span>
                                        <span>"Real-time collaboration with CRDT sync"</span>
                                    </div>
                                    <div class="flex items-center gap-2 text-sm text-gray-700 dark:text-gray-300">
                                        <span class="text-green-500">"- "</span>
                                        <span>"Works offline, syncs when connected"</span>
                                    </div>
                                    <div class="flex items-center gap-2 text-sm text-gray-700 dark:text-gray-300">
                                        <span class="text-green-500">"- "</span>
                                        <span>"Knowledge graph with bidirectional links"</span>
                                    </div>
                                </div>
                            </div>
                        }.into_any(),
                        1 => view! {
                            <div>
                                <h2 class="text-2xl font-bold text-gray-900 dark:text-gray-100 mb-2">"What should we call you?"</h2>
                                <p class="text-gray-600 dark:text-gray-400 mb-6">
                                    "Your display name will be shown to collaborators."
                                </p>
                                <div class="space-y-4">
                                    <div>
                                        <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">"Display Name"</label>
                                        <input
                                            type="text"
                                            placeholder="Your name"
                                            prop:value={move || display_name.get()}
                                            on:input={move |ev| {
                                                let val = event_target_value(&ev);
                                                set_display_name.set(val);
                                            }}
                                            class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-800 text-gray-900 dark:text-gray-100 focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
                                        />
                                    </div>
                                </div>
                            </div>
                        }.into_any(),
                        2 => view! {
                            <div>
                                <h2 class="text-2xl font-bold text-gray-900 dark:text-gray-100 mb-2">"Name Your Workspace"</h2>
                                <p class="text-gray-600 dark:text-gray-400 mb-6">
                                    "This is your personal space. You can create more workspaces later."
                                </p>
                                <div class="space-y-4">
                                    <div>
                                        <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">"Workspace Name"</label>
                                        <input
                                            type="text"
                                            prop:value={move || workspace_name.get()}
                                            on:input={move |ev| {
                                                let val = event_target_value(&ev);
                                                set_workspace_name.set(val);
                                            }}
                                            class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-800 text-gray-900 dark:text-gray-100 focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
                                        />
                                    </div>
                                </div>
                            </div>
                        }.into_any(),
                        3 => view! {
                            <div class="text-center">
                                <div class="w-16 h-16 bg-green-100 dark:bg-green-900/30 rounded-full flex items-center justify-center mx-auto mb-4">
                                    <svg class="w-8 h-8 text-green-600 dark:text-green-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
                                    </svg>
                                </div>
                                <h2 class="text-2xl font-bold text-gray-900 dark:text-gray-100 mb-2">"You're All Set!"</h2>
                                <p class="text-gray-600 dark:text-gray-400 mb-6">
                                    "Your workspace is ready. Start by creating your first document or exploring the sidebar."
                                </p>
                                <div class="flex flex-col sm:flex-row gap-3 justify-center">
                                    <a
                                        href="/documents/new"
                                        class="px-4 py-2 bg-blue-600 hover:bg-blue-700 text-white rounded-lg transition-colors text-sm font-medium"
                                    >
                                        "Create First Document"
                                    </a>
                                    <a
                                        href="/documents"
                                        class="px-4 py-2 border border-gray-300 dark:border-gray-600 hover:bg-gray-50 dark:hover:bg-gray-800 text-gray-700 dark:text-gray-300 rounded-lg transition-colors text-sm font-medium"
                                    >
                                        "Go to Documents"
                                    </a>
                                </div>
                            </div>
                        }.into_any(),
                        _ => ().into_any(),
                    }}
                </div>

                // Footer with navigation
                {move || {
                    if step.get() < 3 {
                        view! {
                            <div class="px-8 pb-6 flex items-center justify-between">
                                <div>
                                    {move || if !is_first_step() {
                                        view! {
                                            <button
                                                on:click=prev_step
                                                class="px-4 py-2 text-sm text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-gray-100 transition-colors"
                                            >
                                                "Back"
                                            </button>
                                        }.into_any()
                                    } else {
                                        view! {
                                            <button
                                                on:click=on_skip
                                                class="px-4 py-2 text-sm text-gray-400 hover:text-gray-600 dark:hover:text-gray-300 transition-colors"
                                            >
                                                "Skip"
                                            </button>
                                        }.into_any()
                                    }}
                                </div>
                                {move || if !is_last_step() {
                                    view! {
                                        <button
                                            on:click=next_step
                                            class="px-4 py-2 bg-blue-600 hover:bg-blue-700 text-white rounded-lg transition-colors text-sm font-medium"
                                        >
                                            "Continue"
                                        </button>
                                    }.into_any()
                                } else {
                                    view! {
                                        <button
                                            on:click=on_finish
                                            class="px-4 py-2 bg-green-600 hover:bg-green-700 text-white rounded-lg transition-colors text-sm font-medium"
                                        >
                                            "Get Started"
                                        </button>
                                    }.into_any()
                                }}
                            </div>
                        }.into_any()
                    } else {
                        ().into_any()
                    }
                }}
            </div>
        </div>
    }
}

/// Check if onboarding has been completed.
/// Returns true if the user should see the onboarding wizard.
///
/// Reserved for future use: conditional onboarding display.
pub fn should_show_onboarding() -> bool {
    if let Some(window) = web_sys::window() {
        if let Some(storage) = window.local_storage().ok().flatten() {
            return storage
                .get_item("tachyon_onboarding_complete")
                .ok()
                .flatten()
                .is_none();
        }
    }
    false
}
