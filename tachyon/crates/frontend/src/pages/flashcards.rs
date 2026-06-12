use leptos::prelude::*;
use leptos::task::spawn_local;
use serde::{Deserialize, Serialize};

// ============================================================================
// API Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Flashcard {
    pub id: String,
    pub document_id: String,
    pub front: String,
    pub back: String,
    pub tags: serde_json::Value,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlashcardSrsState {
    pub flashcard_id: String,
    pub state: i16,
    pub step: i16,
    pub stability: f64,
    pub difficulty: f64,
    pub due: String,
    pub reps: i32,
    pub lapses: i32,
    pub last_review: Option<String>,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewQueueResponse {
    pub cards: Vec<Flashcard>,
    pub total_due: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewResponse {
    pub flashcard: Flashcard,
    pub srs_state: FlashcardSrsState,
    pub message: String,
}

// ============================================================================
// Page Component
// ============================================================================

#[component]
pub fn FlashcardsPage() -> impl IntoView {
    let (cards, set_cards) = signal(Vec::<Flashcard>::new());
    let (current_index, set_current_index) = signal(0usize);
    let (show_back, set_show_back) = signal(false);
    let (loading, set_loading) = signal(true);
    let (reviewing, set_reviewing) = signal(false);
    let (session_stats, set_session_stats) = signal(SessionStats::default());
    let (error, set_error) = signal(None::<String>);

    let total_due = move || cards.get().len();
    let current_card = move || cards.get().get(current_index.get()).cloned();
    let is_done = move || current_index.get() >= cards.get().len();
    let progress = move || {
        let total = total_due();
        if total == 0 {
            0.0
        } else {
            (current_index.get() as f64 / total as f64) * 100.0
        }
    };

    // Load due cards
    let load_cards = {
        let set_loading = set_loading.clone();
        let set_error = set_error.clone();
        let set_cards = set_cards.clone();
        move || {
            set_loading.set(true);
            set_error.set(None);
            spawn_local(async move {
                let window = web_sys::window().expect("no window");
                let location = window.location();
                let base = format!(
                    "{}://{}",
                    location.protocol().expect("no protocol"),
                    location.host().expect("no host")
                );
                let url = format!("{}/api/v1/flashcards/review", base);
                match gloo_net::http::Request::get(&url).send().await {
                    Ok(resp) => {
                        if let Ok(data) = resp.json::<ReviewQueueResponse>().await {
                            set_cards.set(data.cards);
                            set_current_index.set(0);
                            set_show_back.set(false);
                            set_loading.set(false);
                        } else {
                            set_error.set(Some("Failed to parse response".to_string()));
                            set_loading.set(false);
                        }
                    }
                    Err(e) => {
                        set_error.set(Some(format!("Failed to load cards: {}", e)));
                        set_loading.set(false);
                    }
                }
            });
        }
    };

    // Initial load
    Effect::new(move |_| {
        load_cards();
    });

    let submit_review = move |rating: i16| {
        let card = match current_card() {
            Some(c) => c,
            None => return,
        };
        set_reviewing.set(true);
        let card_id = card.id.clone();
        spawn_local(async move {
            let window = web_sys::window().expect("no window");
            let location = window.location();
            let base = format!(
                "{}://{}",
                location.protocol().expect("no protocol"),
                location.host().expect("no host")
            );
            let url = format!("{}/api/v1/flashcards/{}/review", base, card_id);
            let body = serde_json::json!({ "rating": rating });
            let resp = gloo_net::http::Request::post(&url)
                .header("Content-Type", "application/json")
                .json(&body)
                .expect("failed to serialize")
                .send()
                .await;
            match resp {
                Ok(r) => {
                    if let Ok(_review_resp) = r.json::<ReviewResponse>().await {
                        set_session_stats.update(|s| {
                            s.reviews += 1;
                            match rating {
                                0 => s.again += 1,
                                1 => s.hard += 1,
                                2 => s.good += 1,
                                3 => s.easy += 1,
                                _ => {}
                            }
                        });
                        set_current_index.update(|i| *i += 1);
                        set_show_back.set(false);
                    }
                }
                Err(_) => {}
            }
            set_reviewing.set(false);
        });
    };

    view! {
        <div class="max-w-3xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
            <div class="mb-8">
                <h1 class="text-3xl font-bold text-gray-900 dark:text-white">"Flashcards"</h1>
                <p class="mt-2 text-gray-600 dark:text-gray-400">
                    "Review due flashcards using spaced repetition."
                </p>
            </div>

            <Show when=move || error.get().is_some()>
                <div class="mb-6 p-4 bg-red-100 dark:bg-red-900 border-2 border-red-400 dark:border-red-700 text-red-700 dark:text-red-200 rounded-none">
                    {move || error.get().unwrap_or_default()}
                </div>
            </Show>

            <Show when=move || loading.get()>
                <div class="flex items-center justify-center py-16">
                    <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600"></div>
                    <span class="ml-3 text-gray-500">"Loading cards..."</span>
                </div>
            </Show>

            <Show when=move || !loading.get() && !is_done()>
                // Progress bar
                <div class="mb-6">
                    <div class="flex justify-between text-sm text-gray-500 dark:text-gray-400 mb-1">
                        <span>"Card "{current_index.get() + 1}" of "{total_due()}</span>
                        <span>{format!("{:.0}%", progress())}</span>
                    </div>
                    <div class="w-full bg-gray-200 dark:bg-gray-700 h-2 rounded-full">
                        <div
                            class="bg-blue-600 h-2 rounded-full transition-all duration-300"
                            style:width={format!("{}%", progress())}
                        ></div>
                    </div>
                </div>

                // Flashcard
                {move || {
                    current_card().map(|card| {
                        let front = card.front.clone();
                        let back = card.back.clone();
                        let card_class = if show_back.get() {
                            "bg-white dark:bg-gray-800 border-2 border-blue-500 dark:border-blue-400 p-8 min-h-[300px] flex items-center justify-center cursor-pointer rounded-none shadow-lg"
                        } else {
                            "bg-white dark:bg-gray-800 border-2 border-gray-900 dark:border-gray-100 p-8 min-h-[300px] flex items-center justify-center cursor-pointer rounded-none shadow-lg hover:border-blue-400 dark:hover:border-blue-500 transition-colors"
                        };
                        view! {
                            <div class=card_class on:click=move |_| set_show_back.update(|b| *b = !*b)>
                                <div class="text-center w-full">
                                    <div class="text-xs uppercase tracking-wider text-gray-400 dark:text-gray-500 mb-4">
                                        {if show_back.get() { "Back" } else { "Front" }}
                                    </div>
                                    <div class="text-lg text-gray-900 dark:text-white whitespace-pre-wrap">
                                        {if show_back.get() { back.clone() } else { front.clone() }}
                                    </div>
                                    <div class="mt-4 text-xs text-gray-400 dark:text-gray-500">
                                        "Click to flip"
                                    </div>
                                </div>
                            </div>
                        }
                    })
                }}

                // Rating buttons
                <Show when=move || show_back.get()>
                    <div class="mt-6 grid grid-cols-4 gap-3">
                        <button
                            class="px-4 py-3 bg-red-100 dark:bg-red-900 text-red-700 dark:text-red-200 rounded-none font-medium hover:bg-red-200 dark:hover:bg-red-800 transition-colors disabled:opacity-50"
                            disabled=reviewing
                            on:click=move |_| submit_review(0)
                        >
                            "Again"
                        </button>
                        <button
                            class="px-4 py-3 bg-orange-100 dark:bg-orange-900 text-orange-700 dark:text-orange-200 rounded-none font-medium hover:bg-orange-200 dark:hover:bg-orange-800 transition-colors disabled:opacity-50"
                            disabled=reviewing
                            on:click=move |_| submit_review(1)
                        >
                            "Hard"
                        </button>
                        <button
                            class="px-4 py-3 bg-green-100 dark:bg-green-900 text-green-700 dark:text-green-200 rounded-none font-medium hover:bg-green-200 dark:hover:bg-green-800 transition-colors disabled:opacity-50"
                            disabled=reviewing
                            on:click=move |_| submit_review(2)
                        >
                            "Good"
                        </button>
                        <button
                            class="px-4 py-3 bg-blue-100 dark:bg-blue-900 text-blue-700 dark:text-blue-200 rounded-none font-medium hover:bg-blue-200 dark:hover:bg-blue-800 transition-colors disabled:opacity-50"
                            disabled=reviewing
                            on:click=move |_| submit_review(3)
                        >
                            "Easy"
                        </button>
                    </div>
                </Show>
            </Show>

            // Session complete
            <Show when=move || !loading.get() && is_done()>
                <div class="bg-white dark:bg-gray-800 border-2 border-gray-900 dark:border-gray-100 rounded-none p-8 text-center">
                    <div class="text-4xl mb-4">"🎉"</div>
                    <h2 class="text-2xl font-bold text-gray-900 dark:text-white mb-2">"Session Complete!"</h2>
                    <p class="text-gray-600 dark:text-gray-400 mb-6">
                        "You've reviewed all due cards."
                    </p>
                    <div class="grid grid-cols-4 gap-4 mb-6">
                        <div class="bg-red-50 dark:bg-red-900/30 p-3 rounded-none">
                            <div class="text-2xl font-bold text-red-600 dark:text-red-400">{move || session_stats.get().again}</div>
                            <div class="text-xs text-red-500 dark:text-red-400">"Again"</div>
                        </div>
                        <div class="bg-orange-50 dark:bg-orange-900/30 p-3 rounded-none">
                            <div class="text-2xl font-bold text-orange-600 dark:text-orange-400">{move || session_stats.get().hard}</div>
                            <div class="text-xs text-orange-500 dark:text-orange-400">"Hard"</div>
                        </div>
                        <div class="bg-green-50 dark:bg-green-900/30 p-3 rounded-none">
                            <div class="text-2xl font-bold text-green-600 dark:text-green-400">{move || session_stats.get().good}</div>
                            <div class="text-xs text-green-500 dark:text-green-400">"Good"</div>
                        </div>
                        <div class="bg-blue-50 dark:bg-blue-900/30 p-3 rounded-none">
                            <div class="text-2xl font-bold text-blue-600 dark:text-blue-400">{move || session_stats.get().easy}</div>
                            <div class="text-xs text-blue-500 dark:text-blue-400">"Easy"</div>
                        </div>
                    </div>
                    <button
                        class="px-6 py-3 bg-blue-600 text-white rounded-none font-medium hover:bg-blue-700 transition-colors"
                        on:click=move |_| {
                            set_session_stats.set(SessionStats::default());
                            load_cards();
                        }
                    >
                        "Review Again"
                    </button>
                </div>
            </Show>

            // No cards due
            <Show when=move || !loading.get() && total_due() == 0 && !is_done()>
                <div class="bg-white dark:bg-gray-800 border-2 border-gray-900 dark:border-gray-100 rounded-none p-8 text-center">
                    <div class="text-4xl mb-4">"📚"</div>
                    <h2 class="text-2xl font-bold text-gray-900 dark:text-white mb-2">"All caught up!"</h2>
                    <p class="text-gray-600 dark:text-gray-400">
                        "No cards are due for review right now."
                    </p>
                </div>
            </Show>
        </div>
    }
}

#[derive(Debug, Clone, Default)]
struct SessionStats {
    reviews: usize,
    again: usize,
    hard: usize,
    good: usize,
    easy: usize,
}
