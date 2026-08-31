use leptos::*;

#[component]
pub fn SkipLinks() -> impl IntoView {
    view! {
        <a
            href="#main-content"
            class="sr-only focus:not-sr-only focus:fixed focus:top-4 focus:left-4 focus:z-50 focus:px-4 focus:py-2 focus:bg-blue-600 focus:text-white focus:rounded"
        >
            "Skip to main content"
        </a>
        <a
            href="#main-nav"
            class="sr-only focus:not-sr-only focus:fixed focus:top-4 focus:left-64 focus:z-50 focus:px-4 focus:py-2 focus:bg-blue-600 focus:text-white focus:rounded"
        >
            "Skip to navigation"
        </a>
    }
}
