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
                let count = completions
                    .with(|c| c.as_ref().map(|v| v.len()).unwrap_or(0));
                if count > 0 {
                    let idx = highlighted_idx.get();
                    set_highlighted_idx.set((idx + 1) % count);
                }
            }
            "ArrowUp" => {
                ev.prevent_default();
                ev.stop_propagation();
                let count = completions
                    .with(|c| c.as_ref().map(|v| v.len()).unwrap_or(0));
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
                class="wikilink-autocomplete"
                style=move || format!(
                    "position: absolute; z-index: 50; left: {}px; top: {}px; \
                     background: white; border: 1px solid #e5e7eb; border-radius: 0.5rem; \
                     box-shadow: 0 4px 6px -1px rgba(0,0,0,0.1); max-height: 256px; \
                     overflow-y: auto; min-width: 240px;",
                    position.get().0,
                    position.get().1
                )
                on:keydown=on_keydown
                role="listbox"
                aria-label="Document suggestions"
            >
                <Suspense fallback=move || {
                    view! {
                        <div style="padding: 0.5rem; font-size: 0.875rem; color: #6b7280;">
                            "Searching..."
                        </div>
                    }
                }>
                    {move || completions.get().map(|items| {
                        if items.is_empty() {
                            view! {
                                <div style="padding: 0.5rem; font-size: 0.875rem; color: #6b7280;">
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
                                        style=move || {
                                            let bg = if is_highlighted.get() {
                                                "#eff6ff"
                                            } else {
                                                "transparent"
                                            };
                                            format!(
                                                "display: block; width: 100%; text-align: left; \
                                                 padding: 0.5rem 1rem; border: none; cursor: pointer; \
                                                 background: {}; font-size: 0.875rem; \
                                                 font-family: inherit;",
                                                bg
                                            )
                                        }
                                        on:click=move |_| on_select.run(item_for_click.clone())
                                        on:mouseenter=move |_| set_highlighted_idx.set(idx)
                                        role="option"
                                        aria-selected=move || is_highlighted.get()
                                    >
                                        <div style="font-weight: 500;">{title}</div>
                                        <div style="font-size: 0.75rem; color: #6b7280; overflow: hidden; text-overflow: ellipsis; white-space: nowrap;">
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
