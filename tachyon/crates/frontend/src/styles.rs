// Global styles component — placeholder.
// All custom CSS (design tokens, animations, editor, toolbar, search, split view)
// is defined in index.html <style> blocks to avoid Leptos view! macro {{ }} escaping
// issues that produce invalid CSS (nested { { ... } } blocks).

use leptos::prelude::*;

/// Global styles component — all CSS now in index.html.
#[component]
pub fn GlobalStyles() -> impl IntoView {
    view! {
        <style></style>
    }
}
