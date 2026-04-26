use leptos::prelude::*;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Heading {
    pub level: u8,
    pub text: String,
    pub slug: String,
}

#[allow(dead_code)]
pub fn extract_headings(markdown: &str) -> Vec<Heading> {
    let mut headings = Vec::new();
    let mut in_code_block = false;

    for line in markdown.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("```") {
            in_code_block = !in_code_block;
            continue;
        }
        if in_code_block {
            continue;
        }
        if !trimmed.starts_with('#') {
            continue;
        }
        let level = trimmed.chars().take_while(|&c| c == '#').count().min(6);
        let text = trimmed[level..].trim().to_string();
        if text.is_empty() {
            continue;
        }
        let slug: String = text
            .chars()
            .map(|c| {
                if c.is_alphanumeric() {
                    c.to_lowercase().next().unwrap()
                } else {
                    '-'
                }
            })
            .collect();
        headings.push(Heading {
            level: level as u8,
            text,
            slug,
        });
    }
    headings
}

#[component]
pub fn TableOfContents(markdown_content: String) -> impl IntoView {
    let headings = extract_headings(&markdown_content);

    view! {
        <div class="text-sm">
            <h3 class="font-semibold text-gray-900 dark:text-white mb-3 text-xs uppercase tracking-wider">
                "Outline"
            </h3>
            {if headings.is_empty() {
                view! {
                    <p class="text-gray-400 text-xs italic">"No headings found"</p>
                }.into_any()
            } else {
                view! {
                    <nav class="space-y-1 max-h-96 overflow-y-auto">
                        <For
                            each=move || headings.clone()
                            key=|h| h.slug.clone()
                            let:heading
                        >
                            <a
                                href={format!("#{}", heading.slug)}
                                class=move || format!(
                                    "block py-0.5 text-gray-600 dark:text-gray-400 hover:text-blue-600 dark:hover:text-blue-400 transition-colors truncate {}",
                                    if heading.level <= 2 { "font-medium" } else { "" }
                                )
                                style=move || format!("padding-left: {}px", (heading.level.saturating_sub(1) as usize) * 12)
                                title={heading.text.clone()}
                            >
                                {heading.text.clone()}
                            </a>
                        </For>
                    </nav>
                }.into_any()
            }}
        </div>
    }
}
