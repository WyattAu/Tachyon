use leptos::prelude::*;

/// Represents a heading extracted from markdown content.
#[derive(Debug, Clone)]
pub struct Heading {
    pub level: u8,
    pub text: String,
    pub slug: String,
}

/// Extract headings from markdown content for table-of-contents display.
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

/// Scroll to a heading element by its slug ID in the current document.
fn scroll_to_heading(slug: &str) {
    if let Some(window) = web_sys::window() {
        let doc = match window.document() {
            Some(d) => d,
            None => return,
        };
        let el = match doc.get_element_by_id(slug) {
            Some(e) => e,
            None => return,
        };
        #[allow(unused_mut)]
        let mut opts = web_sys::ScrollIntoViewOptions::new();
        opts.set_behavior(web_sys::ScrollBehavior::Smooth);
        opts.set_block(web_sys::ScrollLogicalPosition::Start);
        el.scroll_into_view_with_scroll_into_view_options(&opts);
    }
}

/// Detect which heading is currently visible at the top of the viewport.
fn detect_active_heading(slugs: &[String]) -> Option<String> {
    let window = web_sys::window()?;
    let doc = window.document()?;
    let scroll_top = window.scroll_y().ok()? as f64;
    let view_height = doc.document_element()?.client_height() as f64;

    let mut active: Option<(String, f64)> = None;
    for slug in slugs {
        if let Some(el) = doc.get_element_by_id(slug) {
            let rect = el.get_bounding_client_rect();
            let top = rect.top() + scroll_top;
            if top <= scroll_top + view_height * 0.2 {
                match &active {
                    None => active = Some((slug.clone(), top)),
                    Some((_, prev_top)) if top >= *prev_top => {
                        active = Some((slug.clone(), top));
                    }
                    _ => {}
                }
            }
        }
    }
    active.map(|(slug, _)| slug)
}

#[component]
pub fn TableOfContents(
    markdown_content: String,
    #[prop(default = true)] show_toggle: bool,
    #[prop(default = false)] embedded: bool,
) -> impl IntoView {
    let headings = extract_headings(&markdown_content);
    let (is_visible, set_is_visible) = signal(true);
    let (active_slug, set_active_slug) = signal(String::new());

    // Scroll-spy: detect active heading on scroll
    {
        let slugs: Vec<String> = headings.iter().map(|h| h.slug.clone()).collect();
        let set_active = set_active_slug;
        let _ = leptos_use::use_event_listener(
            leptos_use::use_window(),
            leptos::ev::scroll,
            move |_: web_sys::Event| {
                if let Some(slug) = detect_active_heading(&slugs) {
                    set_active.set(slug);
                }
            },
        );
    }

    let toggle_label = move || {
        if is_visible.get() {
            "Hide Outline"
        } else {
            "Show Outline"
        }
    };

    view! {
        {if embedded {
            ().into_any()
        } else if show_toggle {
            view! {
                <button
                    class="toc-toggle text-xs text-gray-500 dark:text-gray-400 hover:text-blue-600 dark:hover:text-blue-400 mb-2 flex items-center gap-1"
                    on:click={move |_| set_is_visible.update(|v| *v = !*v)}
                >
                    {move || {
                        let arrow = if is_visible.get() { "\u{25BC}" } else { "\u{25B6}" };
                        format!("{} {}", arrow, toggle_label())
                    }}
                </button>
            }.into_any()
        } else {
            ().into_any()
        }}
        {move || {
            if !is_visible.get() || headings.is_empty() {
                return ().into_any();
            }

            let heading_items = headings.clone();
            view! {
                <nav class="toc-nav space-y-1 max-h-96 overflow-y-auto" aria-label="Table of contents">
                    <For
                        each=move || heading_items.clone()
                        key=|h| h.slug.clone()
                        let:heading
                    >
                        {
                            let slug_clone = heading.slug.clone();
                            let slug_for_scroll = heading.slug.clone();
                            let heading_level = heading.level;
                            let heading_text_for_id = heading.slug.clone();
                            let is_active = move || active_slug.get() == heading_text_for_id;

                            view! {
                                <a
                                    href={format!("#{}", slug_clone)}
                                    class=move || {
                                        let base = format!(
                                            "block py-0.5 text-xs transition-colors truncate {}",
                                            if heading_level <= 2 { "font-medium" } else { "" }
                                        );
                                        if is_active() {
                                            format!("{} {}", base, "text-blue-600 dark:text-blue-400 bg-blue-50 dark:bg-blue-900/20")
                                        } else {
                                            format!("{} {}", base, "text-gray-600 dark:text-gray-400 hover:text-blue-600 dark:hover:text-blue-400")
                                        }
                                    }
                                    style=move || format!("padding-left: {}px", (heading.level.saturating_sub(1) as usize) * 12)
                                    title={heading.text.clone()}
                                    on:click={move |ev: leptos::ev::MouseEvent| {
                                        ev.prevent_default();
                                        let s = slug_for_scroll.clone();
                                        scroll_to_heading(&s);
                                    }}
                                >
                                    {heading.text.clone()}
                                </a>
                            }
                        }
                    </For>
                </nav>
            }.into_any()
        }}
    }
}
