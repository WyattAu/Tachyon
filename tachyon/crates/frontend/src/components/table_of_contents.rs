use leptos::prelude::*;

/// Represents a heading extracted from markdown content.
///
/// Reserved for future use: table-of-contents generation.
#[derive(Debug, Clone)]
pub struct Heading {
    pub level: u8,
    pub text: String,
    pub slug: String,
}

/// Extract headings from markdown content for table-of-contents display.
///
/// Reserved for future use: TOC sidebar component.
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_single_heading() {
        let md = "# Title";
        let headings = extract_headings(md);
        assert_eq!(headings.len(), 1);
        assert_eq!(headings[0].level, 1);
        assert_eq!(headings[0].text, "Title");
        assert_eq!(headings[0].slug, "title");
    }

    #[test]
    fn test_extract_multiple_headings() {
        let md = "# Title\n## Subtitle\n### Section\n#### Deep";
        let headings = extract_headings(md);
        assert_eq!(headings.len(), 4);
        assert_eq!(headings[0].level, 1);
        assert_eq!(headings[1].level, 2);
        assert_eq!(headings[2].level, 3);
        assert_eq!(headings[3].level, 4);
    }

    #[test]
    fn test_extract_headings_skips_code_block() {
        let md = "```\n# Not A Heading\n```\n# Real Heading";
        let headings = extract_headings(md);
        assert_eq!(headings.len(), 1);
        assert_eq!(headings[0].text, "Real Heading");
    }

    #[test]
    fn test_extract_headings_skips_non_heading_lines() {
        let md = "Some text\n# Heading\nMore text\n## Sub";
        let headings = extract_headings(md);
        assert_eq!(headings.len(), 2);
    }

    #[test]
    fn test_extract_headings_slug_generation() {
        let md = "# Hello World! How Are You?";
        let headings = extract_headings(md);
        assert_eq!(headings[0].slug, "hello-world--how-are-you-");
    }

    #[test]
    fn test_extract_headings_empty_content() {
        let headings = extract_headings("");
        assert!(headings.is_empty());
    }

    #[test]
    fn test_extract_headings_no_headings() {
        let headings = extract_headings("Just some text\nwith no headings");
        assert!(headings.is_empty());
    }

    #[test]
    fn test_extract_headings_skips_empty_heading_text() {
        let md = "#\n## ";
        let headings = extract_headings(md);
        assert!(headings.is_empty());
    }

    #[test]
    fn test_extract_headings_max_level_6() {
        let md = "####### Seven Hashes";
        let headings = extract_headings(md);
        assert_eq!(headings.len(), 1);
        assert_eq!(headings[0].level, 6);
    }

    #[test]
    fn test_extract_headings_code_block_toggle() {
        let md = "# Before\n```\n# Inside\n```\n# After\n```\n# Second inside\n```\n# Final";
        let headings = extract_headings(md);
        assert_eq!(headings.len(), 3);
        assert_eq!(headings[0].text, "Before");
        assert_eq!(headings[1].text, "After");
        assert_eq!(headings[2].text, "Final");
    }
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
