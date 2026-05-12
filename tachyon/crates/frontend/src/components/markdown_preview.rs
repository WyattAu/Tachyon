#![allow(dead_code)]

use crate::markdown::{
    MarkdownHeading, extract_headings as md_extract_headings, render_markdown_to_html,
};
use leptos::prelude::*;

#[component]
pub fn MarkdownPreview(content: String, #[prop(default = true)] render_toc: bool) -> impl IntoView {
    let (html_output, set_html_output) = signal(String::new());
    let (headings, set_headings) = signal(Vec::<MarkdownHeading>::new());

    Effect::new(move |_| {
        let rendered = render_markdown_to_html(&content);
        set_html_output.set(rendered);

        if render_toc {
            let h = md_extract_headings(&content);
            set_headings.set(h);
        }
    });

    view! {
        <div class="markdown-preview flex h-full">
            {move || {
                if render_toc && !headings.get().is_empty() {
                    view! {
                        <nav class="hidden lg:block w-48 flex-shrink-0 border-r border-gray-200 dark:border-gray-700 p-4 overflow-y-auto">
                            <h4 class="text-xs font-semibold text-gray-500 dark:text-gray-400 uppercase tracking-wider mb-3">
                                "Table of Contents"
                            </h4>
                            <For
                                each=move || headings.get()
                                key=|h| h.slug.clone()
                                let:heading
                            >
                                <a
                                    href={format!("#{}", heading.slug)}
                                    class=move || {
                                        let _indent = (heading.level.saturating_sub(1) as usize) * 12;
                                        format!(
                                            "block py-0.5 text-xs text-gray-600 dark:text-gray-400 hover:text-blue-600 dark:hover:text-blue-400 truncate transition-colors {}",
                                            if heading.level <= 2 { "font-medium" } else { "" }
                                        )
                                    }
                                    style=move || {
                                        let _indent = (heading.level.saturating_sub(1) as usize) * 12;
                                        format!("padding-left: {}px", (heading.level.saturating_sub(1) as usize) * 12)
                                    }
                                    title={heading.text.clone()}
                                >
                                    {heading.text.clone()}
                                </a>
                            </For>
                        </nav>
                    }.into_any()
                } else {
                    ().into_any()
                }
            }}
            <div class="flex-1 overflow-y-auto p-6">
                {move || {
                    let html = html_output.get();
                    if html.is_empty() {
                        view! {
                            <div class="flex items-center justify-center h-full text-gray-400 dark:text-gray-500">
                                <div class="text-center">
                                    <p class="text-lg mb-1">"No content to preview"</p>
                                    <p class="text-sm">"Start typing markdown to see a preview"</p>
                                </div>
                            </div>
                        }.into_any()
                    } else {
                        view! {
                            <div
                                class="prose prose-sm dark:prose-invert max-w-none"
                                inner_html={html}
                            ></div>
                        }.into_any()
                    }
                }}
            </div>
        </div>
    }
}
