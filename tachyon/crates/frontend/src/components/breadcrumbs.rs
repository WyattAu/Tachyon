use leptos::prelude::*;

#[derive(Debug, Clone)]
pub struct BreadcrumbItem {
    pub label: String,
    pub href: Option<String>,
}

#[component]
pub fn Breadcrumbs(items: Vec<BreadcrumbItem>) -> impl IntoView {
    view! {
        <nav class="flex items-center text-sm text-gray-500 dark:text-gray-400 mb-4">
            <For
                each=move || items.clone()
                key=|item| item.label.clone()
                let:item
            >
                {move || {
                    let is_last = item.href.is_none();
                    if is_last {
                        view! {
                            <span class="text-gray-900 dark:text-white font-medium">
                                {item.label.clone()}
                            </span>
                        }.into_any()
                    } else {
                        let href = item.href.clone().unwrap_or_default();
                        view! {
                            <>
                                <a
                                    href={href}
                                    class="hover:text-blue-600 dark:hover:text-blue-400 transition-colors"
                                >
                                    {item.label.clone()}
                                </a>
                                <span class="mx-2 text-gray-300 dark:text-gray-600">"/"</span>
                            </>
                        }.into_any()
                    }
                }}
            </For>
        </nav>
    }
}
