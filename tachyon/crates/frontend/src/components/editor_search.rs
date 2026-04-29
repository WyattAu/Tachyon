use leptos::prelude::*;
use tachyon_editor::Editor;

#[component]
pub fn EditorSearch(editor: RwSignal<Editor>, show: RwSignal<bool>) -> impl IntoView {
    let search_query = RwSignal::new(String::new());
    let replace_query = RwSignal::new(String::new());
    let case_sensitive = RwSignal::new(false);
    let whole_word = RwSignal::new(false);
    let use_regex = RwSignal::new(false);
    let match_count = RwSignal::new(0usize);
    let current_match = RwSignal::new(0usize);

    let do_search = move || {
        let query = search_query.get();
        if query.is_empty() {
            match_count.set(0);
            current_match.set(0);
            return;
        }
        let mc = match_count;
        let cm = current_match;
        editor.update(|e| {
            let results = e.find(&query);
            mc.set(results.len());
            cm.set(if results.is_empty() { 0 } else { 1 });
        });
    };

    let go_next = move |_: leptos::ev::MouseEvent| {
        editor.update(|e| {
            e.find_next();
        });
        let total = match_count.get();
        if total > 0 {
            let idx = current_match.get();
            current_match.set(if idx >= total { 1 } else { idx + 1 });
        }
    };

    let go_prev = move |_: leptos::ev::MouseEvent| {
        editor.update(|e| {
            e.find_previous();
        });
        let total = match_count.get();
        if total > 0 {
            let idx = current_match.get();
            current_match.set(if idx <= 1 { total } else { idx - 1 });
        }
    };

    let do_replace = move |_: leptos::ev::MouseEvent| {
        let replacement = replace_query.get();
        editor.update(|e| {
            e.replace_next(&replacement);
        });
        do_search();
    };

    let do_replace_all = move |_: leptos::ev::MouseEvent| {
        let query = search_query.get();
        let replacement = replace_query.get();
        editor.update(|e| {
            e.replace_all(&query, &replacement);
        });
        do_search();
    };

    let close = move |_: leptos::ev::MouseEvent| {
        show.set(false);
    };

    let do_search_for_enter = move || {
        do_search();
    };

    let on_keydown = move |ev: web_sys::KeyboardEvent| {
        if ev.key() == "Escape" {
            show.set(false);
        }
        if ev.key() == "Enter" && !ev.shift_key() {
            do_search_for_enter();
        }
    };

    view! {
        {move || if show.get() {
            view! {
                <div class="editor-search" on:keydown={on_keydown}>
                    <div class="flex items-center gap-2 mb-2">
                        <input
                            type="text"
                            class="editor-search-input"
                            placeholder="Search..."
                            prop:value={move || search_query.get()}
                            on:input=move |ev| {
                                search_query.set(event_target_value(&ev));
                            }
                        />
                        <span class="editor-search-count">
                            {move || {
                                let total = match_count.get();
                                let cur = current_match.get();
                                if total == 0 {
                                    "No results".to_string()
                                } else {
                                    format!("{} of {}", cur, total)
                                }
                            }}
                        </span>
                        <button class="editor-search-btn" on:click={go_prev}>{"\u{25B2}"}</button>
                        <button class="editor-search-btn" on:click={go_next}>{"\u{25BC}"}</button>
                        <button class="editor-search-btn" on:click={close}>{"\u{2715}"}</button>
                    </div>

                    <div class="flex items-center gap-2 mb-2">
                        <label class="editor-search-toggle">
                            <input type="checkbox" prop:checked={move || case_sensitive.get()} on:change=move |ev| { case_sensitive.set(event_target_checked(&ev)); } />
                            {"Aa"}
                        </label>
                        <label class="editor-search-toggle">
                            <input type="checkbox" prop:checked={move || whole_word.get()} on:change=move |ev| { whole_word.set(event_target_checked(&ev)); } />
                            {"W"}
                        </label>
                        <label class="editor-search-toggle">
                            <input type="checkbox" prop:checked={move || use_regex.get()} on:change=move |ev| { use_regex.set(event_target_checked(&ev)); } />
                            {".*"}
                        </label>
                    </div>

                    <div class="flex items-center gap-2">
                        <input
                            type="text"
                            class="editor-search-input"
                            placeholder="Replace..."
                            prop:value={move || replace_query.get()}
                            on:input=move |ev| {
                                replace_query.set(event_target_value(&ev));
                            }
                        />
                        <button class="editor-search-action-btn" on:click={do_replace}>{"Replace"}</button>
                        <button class="editor-search-action-btn" on:click={do_replace_all}>{"All"}</button>
                    </div>
                </div>
            }.into_any()
        } else {
            ().into_any()
        }}
    }
}
