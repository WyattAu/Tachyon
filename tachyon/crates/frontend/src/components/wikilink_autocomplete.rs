use crate::api::ApiClient;
use crate::types::SearchResultItem;
use leptos::prelude::*;

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct WikilinkCompletion {
    pub title: String,
    pub slug: String,
    pub snippet: String,
}

impl WikilinkCompletion {
    pub fn from_search_result(item: &SearchResultItem) -> Self {
        Self {
            title: item.title.clone(),
            slug: item.slug.clone().unwrap_or_default(),
            snippet: item
                .headline
                .clone()
                .unwrap_or_default()
                .chars()
                .take(100)
                .collect(),
        }
    }
}

#[component]
pub fn WikilinkAutocomplete(
    query: ReadSignal<String>,
    visible: ReadSignal<bool>,
    position: ReadSignal<(f64, f64)>,
    on_select: Callback<WikilinkCompletion>,
) -> impl IntoView {
    let api_client = ApiClient::default();

    let completions = LocalResource::new(move || {
        let q = query.get();
        let client = api_client.clone();
        async move {
            if q.is_empty() || q.len() < 2 {
                return vec![];
            }
            match client.search(&q, None, Some(1), Some(5)).await {
                Ok(results) => results
                    .results
                    .iter()
                    .map(WikilinkCompletion::from_search_result)
                    .collect(),
                Err(_) => vec![],
            }
        }
    });

    let (highlighted_idx, set_highlighted_idx) = signal(0usize);

    Effect::new(move |_| {
        if visible.get() {
            set_highlighted_idx.set(0);
        }
    });

    let on_keydown = move |ev: web_sys::KeyboardEvent| {
        if !visible.get() {
            return;
        }

        match ev.key().as_str() {
            "ArrowDown" => {
                ev.prevent_default();
                ev.stop_propagation();
                let count = completions.with(|c| c.as_ref().map(|v| v.len()).unwrap_or(0));
                if count > 0 {
                    let idx = highlighted_idx.get();
                    set_highlighted_idx.set((idx + 1) % count);
                }
            }
            "ArrowUp" => {
                ev.prevent_default();
                ev.stop_propagation();
                let count = completions.with(|c| c.as_ref().map(|v| v.len()).unwrap_or(0));
                if count > 0 {
                    let idx = highlighted_idx.get();
                    set_highlighted_idx.set(idx.saturating_sub(1));
                }
            }
            "Enter" | "Tab" => {
                ev.prevent_default();
                ev.stop_propagation();
                let idx = highlighted_idx.get();
                completions.with(|c| {
                    if let Some(items) = c {
                        if let Some(item) = items.get(idx) {
                            on_select.run(item.clone());
                        }
                    }
                });
            }
            "Escape" => {
                ev.prevent_default();
                ev.stop_propagation();
            }
            _ => {}
        }
    };

    view! {
        <Show when=move || visible.get()>
            <div
                class="wikilink-autocomplete absolute z-50 bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 shadow-lg max-h-64 overflow-y-auto min-w-60"
                style=move || format!(
                    "left: {}px; top: {}px;",
                    position.get().0,
                    position.get().1
                )
                on:keydown=on_keydown
                role="listbox"
                aria-label="Document suggestions"
            >
                <Suspense fallback=move || {
                    view! {
                        <div class="p-2 text-sm text-gray-500 dark:text-gray-400">
                            "Searching..."
                        </div>
                    }
                }>
                    {move || completions.get().map(|items| {
                        if items.is_empty() {
                            view! {
                                <div class="p-2 text-sm text-gray-500 dark:text-gray-400">
                                    "No documents found"
                                </div>
                            }.into_any()
                        } else {
                            items.into_iter().enumerate().map(|(idx, item)| {
                                let is_highlighted = Memo::new(move |_| highlighted_idx.get() == idx);
                                let item_for_click = item.clone();
                                let title = item.title.clone();
                                let snippet = item.snippet.clone();
                                view! {
                                    <button
                                        class=move || {
                                            let bg = if is_highlighted.get() {
                                                "bg-blue-50 dark:bg-blue-900/30"
                                            } else {
                                                "bg-transparent"
                                            };
                                            format!(
                                                "block w-full text-left px-4 py-2 border-none cursor-pointer text-sm font-sans hover:bg-gray-50 dark:hover:bg-gray-700 {}",
                                                bg
                                            )
                                        }
                                        on:click=move |_| on_select.run(item_for_click.clone())
                                        on:mouseenter=move |_| set_highlighted_idx.set(idx)
                                        role="option"
                                        aria-selected=move || is_highlighted.get()
                                    >
                                        <div class="font-medium">{title}</div>
                                        <div class="text-xs text-gray-500 dark:text-gray-400 truncate">
                                            {snippet}
                                        </div>
                                    </button>
                                }
                            }).collect::<Vec<_>>().into_any()
                        }
                    })}
                </Suspense>
            </div>
        </Show>
    }
}
