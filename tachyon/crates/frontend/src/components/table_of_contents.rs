use leptos::prelude::*;
use std::collections::HashMap;
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;

/// A flat item for rendering, carrying index and depth.
#[derive(Debug, Clone)]
pub struct FlatItem {
    idx: usize,
    depth: usize,
    heading: Heading,
    child_count: usize,
}

/// Represents a heading extracted from markdown content.
#[derive(Debug, Clone, PartialEq)]
pub struct Heading {
    pub level: u8,
    pub text: String,
    pub slug: String,
}

/// A node in the nested heading tree, used for collapse/expand.
#[derive(Debug, Clone)]
pub struct TocNode {
    pub heading: Heading,
    pub children: Vec<TocNode>,
}

impl TocNode {
    fn new(heading: Heading) -> Self {
        Self {
            heading,
            children: Vec::new(),
        }
    }
}

/// Build a nested tree from a flat list of headings.
pub fn build_heading_tree(headings: &[Heading]) -> Vec<TocNode> {
    let mut root: Vec<TocNode> = Vec::new();
    let mut stack: Vec<(u8, Vec<usize>)> = Vec::new();

    for heading in headings {
        let level = heading.level;
        let node = TocNode::new(heading.clone());

        while let Some(&(top_level, _)) = stack.last() {
            if top_level < level {
                break;
            }
            stack.pop();
        }

        if let Some((_, path)) = stack.last() {
            if let Some(parent) = find_node_at_path(&mut root, path) {
                let child_idx = parent.children.len();
                parent.children.push(node);
                let mut new_path = path.clone();
                new_path.push(child_idx);
                stack.push((level, new_path));
            } else {
                let idx = root.len();
                root.push(node);
                stack.push((level, vec![idx]));
            }
        } else {
            let idx = root.len();
            root.push(node);
            stack.push((level, vec![idx]));
        }
    }

    root
}

fn find_node_at_path<'a>(nodes: &'a mut [TocNode], path: &[usize]) -> Option<&'a mut TocNode> {
    if path.is_empty() {
        return None;
    }
    let mut current = nodes.get_mut(path[0])?;
    for &idx in &path[1..] {
        current = current.children.get_mut(idx)?;
    }
    Some(current)
}

/// Flatten the tree back to a list for rendering, tracking depth.
pub fn flatten_tree(nodes: &[TocNode], depth: usize) -> Vec<(usize, Heading)> {
    let mut result = Vec::new();
    for node in nodes {
        result.push((depth, node.heading.clone()));
        result.extend(flatten_tree(&node.children, depth + 1));
    }
    result
}

/// Get all slugs in document order.
pub fn collect_slugs(nodes: &[TocNode]) -> Vec<String> {
    let mut slugs = Vec::new();
    for node in nodes {
        slugs.push(node.heading.slug.clone());
        slugs.extend(collect_slugs(&node.children));
    }
    slugs
}

/// Count children for a heading at a given index in a flat list.
pub fn count_children_for_heading(flat: &[FlatItem], idx: usize) -> usize {
    let parent_depth = flat[idx].depth;
    let mut count = 0;
    for item in flat.iter().skip(idx + 1) {
        if item.depth <= parent_depth {
            break;
        }
        if item.depth == parent_depth + 1 {
            count += 1;
        }
    }
    count
}

/// Count children for a heading at a given index (pre-computation helper).
fn count_children_for_heading_idx(flat: &[FlatItem], idx: usize) -> usize {
    count_children_for_heading(flat, idx)
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
    let tree = build_heading_tree(&headings);
    let flat = flatten_tree(&tree, 0);
    let all_slugs: Vec<String> = flat.iter().map(|(_, h)| h.slug.clone()).collect();

    let (is_visible, set_is_visible) = signal(true);
    let (active_slug, set_active_slug) = signal(String::new());
    let (collapsed_slugs, set_collapsed_slugs) = signal({
        let mut m = HashMap::new();
        for (depth, heading) in &flat {
            let has_child_at_next_depth = flat.iter().any(|(d, h)| {
                h.slug != heading.slug
                    && *d == depth + 1
                    && flat
                        .iter()
                        .position(|(_, hh)| hh.slug == heading.slug)
                        .and_then(|parent_i| {
                            flat.iter()
                                .position(|(_, hh)| hh.slug == h.slug)
                                .map(|child_i| parent_i < child_i)
                        })
                        .unwrap_or(false)
            });
            if has_child_at_next_depth {
                m.insert(heading.slug.clone(), true);
            }
        }
        m
    });

    // IntersectionObserver-based scroll spy
    {
        let slugs = all_slugs.clone();
        let set_active = set_active_slug;

        let callback = Closure::<dyn Fn(web_sys::js_sys::Array)>::new(
            move |entries: web_sys::js_sys::Array| {
                let window = match web_sys::window() {
                    Some(w) => w,
                    None => return,
                };
                let view_height = window
                    .document()
                    .and_then(|d| d.document_element())
                    .map(|e| e.client_height() as f64)
                    .unwrap_or(0.0);

                let mut best_slug: Option<String> = None;
                let mut best_top = f64::NEG_INFINITY;

                for i in 0..entries.length() {
                    let entry: web_sys::IntersectionObserverEntry = entries.get(i).unchecked_into();
                    let target = entry.target();
                    let slug = target.get_attribute("id").unwrap_or_default();
                    if slug.is_empty() {
                        continue;
                    }

                    let rect = entry.bounding_client_rect();
                    let top = rect.top();

                    if top <= view_height * 0.3 && top > best_top {
                        best_top = top;
                        best_slug = Some(slug);
                    }
                }

                if let Some(slug) = best_slug {
                    set_active.set(slug);
                }
            },
        );

        let observer_opts = {
            let opts = web_sys::IntersectionObserverInit::new();
            opts.set_root_margin("0px 0px -70% 0px");
            opts.set_threshold(&JsValue::from_f64(0.0));
            opts
        };

        let observer = web_sys::IntersectionObserver::new_with_options(
            callback.as_ref().unchecked_ref(),
            &observer_opts,
        )
        .ok();

        let observer_cell = std::cell::RefCell::new(observer);

        // Fallback scroll listener
        {
            let slugs_clone = all_slugs.clone();
            let _ = leptos_use::use_event_listener(
                leptos_use::use_window(),
                leptos::ev::scroll,
                move |_: web_sys::Event| {
                    if let Some(slug) = detect_active_heading(&slugs_clone) {
                        set_active.set(slug);
                    }
                },
            );
        }

        // Observe heading elements after mount
        Effect::new(move |_| {
            let _ = observer_cell.borrow();
            let window = match web_sys::window() {
                Some(w) => w,
                None => return,
            };
            let doc = match window.document() {
                Some(d) => d,
                None => return,
            };
            if let Some(ref obs) = *observer_cell.borrow() {
                for slug in &slugs {
                    if let Some(el) = doc.get_element_by_id(slug) {
                        obs.observe(&el);
                    }
                }
            }
        });

        callback.forget();
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

            let flat_headings = flatten_tree(&build_heading_tree(&headings), 0);
            let flat_items: Vec<FlatItem> = flat_headings
                .iter()
                .enumerate()
                .map(|(i, (d, h))| FlatItem {
                    idx: i,
                    depth: *d,
                    heading: h.clone(),
                    child_count: 0,
                })
                .collect();
            // Pre-compute child counts
            let flat_items: Vec<FlatItem> = flat_items
                .iter()
                .enumerate()
                .map(|(i, item)| FlatItem {
                    child_count: count_children_for_heading_idx(&flat_items, i),
                    ..item.clone()
                })
                .collect();

            // Pre-compute the view data to avoid ownership issues in nested closures
            let items_data: Vec<(FlatItem, bool, bool)> = flat_items
                .iter()
                .map(|item| {
                    let collapsed = collapsed_slugs.get();
                    let active = active_slug.get();
                    let is_active = active == item.heading.slug;
                    let is_collapsed = collapsed.get(&item.heading.slug).copied().unwrap_or(true);
                    (item.clone(), is_active, is_collapsed)
                })
                .collect();
            let items_data_clone = items_data.clone();

            view! {
                <nav class="toc-nav space-y-0.5 max-h-96 overflow-y-auto" aria-label="Table of contents">
                    <For
                        each=move || items_data_clone.clone()
                        key=|item| item.0.heading.slug.clone()
                        let:item
                    >
                        {
                            let (flat_item, is_active, is_collapsed) = item;
                            let depth = flat_item.depth;
                            let heading = flat_item.heading.clone();
                            let slug_for_click = heading.slug.clone();
                            let slug_for_toggle = heading.slug.clone();
                            let heading_level = heading.level;
                            let heading_text = heading.text.clone();
                            let indent_px = depth * 14;
                            let has_children = flat_item.child_count > 0;
                            let child_count = flat_item.child_count;

                            let active_class = if is_active {
                                "text-blue-600 dark:text-blue-400 bg-blue-50 dark:bg-blue-900/20 font-medium"
                            } else {
                                "text-gray-600 dark:text-gray-400 hover:text-blue-600 dark:hover:text-blue-400"
                            };

                            let level_indicator = match heading_level {
                                1 => "H1",
                                2 => "H2",
                                3 => "H3",
                                4 => "H4",
                                5 => "H5",
                                _ => "H6",
                            };

                            view! {
                                <div
                                    class="flex items-center group"
                                    style=move || format!("margin-left: {}px", indent_px)
                                >
                                    {if has_children {
                                        view! {
                                            <button
                                                class="toc-collapse-btn w-4 h-4 flex items-center justify-center text-gray-400 hover:text-gray-600 dark:hover:text-gray-300 flex-shrink-0"
                                                title=if is_collapsed {
                                                    format!("Expand {} sub-headings", child_count)
                                                } else {
                                                    format!("Collapse {} sub-headings", child_count)
                                                }
                                                on:click={move |ev: leptos::ev::MouseEvent| {
                                                    ev.prevent_default();
                                                    ev.stop_propagation();
                                                    set_collapsed_slugs.update(|m| {
                                                        let c = m
                                                            .get(&slug_for_toggle)
                                                            .copied()
                                                            .unwrap_or(true);
                                                        m.insert(slug_for_toggle.clone(), !c);
                                                    });
                                                }}
                                            >
                                                <span class="text-[10px]">
                                                    {if is_collapsed { "\u{25B6}" } else { "\u{25BC}" }}
                                                </span>
                                            </button>
                                        }
                                        .into_any()
                                    } else {
                                        view! { <span class="w-4 flex-shrink-0"></span> }.into_any()
                                    }}
                                    <a
                                        href={format!("#{}", slug_for_click)}
                                        class=move || {
                                            format!(
                                                "block py-0.5 px-1 text-xs transition-colors truncate rounded {}",
                                                active_class
                                            )
                                        }
                                        title={heading_text.clone()}
                                        on:click={move |ev: leptos::ev::MouseEvent| {
                                            ev.prevent_default();
                                            scroll_to_heading(&slug_for_click);
                                        }}
                                    >
                                        <span class="text-[10px] text-gray-400 dark:text-gray-500 mr-1 font-mono">
                                            {level_indicator}
                                        </span>
                                        {heading.text.clone()}
                                    </a>
                                </div>
                            }
                        }
                    </For>
                </nav>
            }
            .into_any()
        }}
    }
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

    // --- Tree tests ---

    #[test]
    fn test_build_heading_tree_flat() {
        let headings = vec![
            Heading {
                level: 1,
                text: "A".into(),
                slug: "a".into(),
            },
            Heading {
                level: 1,
                text: "B".into(),
                slug: "b".into(),
            },
        ];
        let tree = build_heading_tree(&headings);
        assert_eq!(tree.len(), 2);
        assert!(tree[0].children.is_empty());
        assert!(tree[1].children.is_empty());
    }

    #[test]
    fn test_build_heading_tree_nested() {
        let headings = vec![
            Heading {
                level: 1,
                text: "H1".into(),
                slug: "h1".into(),
            },
            Heading {
                level: 2,
                text: "H2a".into(),
                slug: "h2a".into(),
            },
            Heading {
                level: 2,
                text: "H2b".into(),
                slug: "h2b".into(),
            },
            Heading {
                level: 3,
                text: "H3".into(),
                slug: "h3".into(),
            },
        ];
        let tree = build_heading_tree(&headings);
        assert_eq!(tree.len(), 1);
        assert_eq!(tree[0].children.len(), 2);
        assert_eq!(tree[0].children[0].heading.text, "H2a");
        assert_eq!(tree[0].children[1].heading.text, "H2b");
        assert_eq!(tree[0].children[1].children.len(), 1);
        assert_eq!(tree[0].children[1].children[0].heading.text, "H3");
    }

    #[test]
    fn test_collect_slugs() {
        let headings = vec![
            Heading {
                level: 1,
                text: "A".into(),
                slug: "a".into(),
            },
            Heading {
                level: 2,
                text: "B".into(),
                slug: "b".into(),
            },
        ];
        let tree = build_heading_tree(&headings);
        let slugs = collect_slugs(&tree);
        assert_eq!(slugs, vec!["a", "b"]);
    }

    #[test]
    fn test_count_children_for_heading() {
        let headings = vec![
            Heading {
                level: 1,
                text: "H1".into(),
                slug: "h1".into(),
            },
            Heading {
                level: 2,
                text: "H2a".into(),
                slug: "h2a".into(),
            },
            Heading {
                level: 2,
                text: "H2b".into(),
                slug: "h2b".into(),
            },
            Heading {
                level: 1,
                text: "H1b".into(),
                slug: "h1b".into(),
            },
        ];
        let tree = build_heading_tree(&headings);
        let flat_tuples = flatten_tree(&tree, 0);
        let flat: Vec<FlatItem> = flat_tuples
            .iter()
            .enumerate()
            .map(|(i, (d, h))| FlatItem {
                idx: i,
                depth: *d,
                heading: h.clone(),
                child_count: 0,
            })
            .collect();
        assert_eq!(flat.len(), 4);
        assert_eq!(count_children_for_heading(&flat, 0), 2);
        assert_eq!(count_children_for_heading(&flat, 3), 0);
    }

    #[test]
    fn test_flatten_tree_preserves_order() {
        let headings = vec![
            Heading {
                level: 1,
                text: "A".into(),
                slug: "a".into(),
            },
            Heading {
                level: 2,
                text: "B".into(),
                slug: "b".into(),
            },
            Heading {
                level: 2,
                text: "C".into(),
                slug: "c".into(),
            },
            Heading {
                level: 3,
                text: "D".into(),
                slug: "d".into(),
            },
            Heading {
                level: 1,
                text: "E".into(),
                slug: "e".into(),
            },
        ];
        let tree = build_heading_tree(&headings);
        let flat = flatten_tree(&tree, 0);
        assert_eq!(flat.len(), 5);
        assert_eq!(flat[0].1, headings[0]);
        assert_eq!(flat[1].1, headings[1]);
        assert_eq!(flat[2].1, headings[2]);
        assert_eq!(flat[3].1, headings[3]);
        assert_eq!(flat[4].1, headings[4]);
    }

    #[test]
    fn test_heading_equality() {
        let h1 = Heading {
            level: 1,
            text: "Test".into(),
            slug: "test".into(),
        };
        let h2 = Heading {
            level: 1,
            text: "Test".into(),
            slug: "test".into(),
        };
        assert_eq!(h1, h2);
    }

    #[test]
    fn test_build_heading_tree_deep_nesting() {
        let headings = vec![
            Heading {
                level: 1,
                text: "L1".into(),
                slug: "l1".into(),
            },
            Heading {
                level: 2,
                text: "L2".into(),
                slug: "l2".into(),
            },
            Heading {
                level: 3,
                text: "L3".into(),
                slug: "l3".into(),
            },
            Heading {
                level: 4,
                text: "L4".into(),
                slug: "l4".into(),
            },
            Heading {
                level: 5,
                text: "L5".into(),
                slug: "l5".into(),
            },
            Heading {
                level: 6,
                text: "L6".into(),
                slug: "l6".into(),
            },
        ];
        let tree = build_heading_tree(&headings);
        assert_eq!(tree.len(), 1);
        assert_eq!(tree[0].children.len(), 1);
        assert_eq!(tree[0].children[0].children.len(), 1);
        assert_eq!(tree[0].children[0].children[0].children.len(), 1);
        assert_eq!(
            tree[0].children[0].children[0].children[0].children.len(),
            1
        );
        assert_eq!(
            tree[0].children[0].children[0].children[0].children[0]
                .children
                .len(),
            1
        );
    }

    #[test]
    fn test_build_heading_tree_level_regression() {
        let headings = vec![
            Heading {
                level: 1,
                text: "A".into(),
                slug: "a".into(),
            },
            Heading {
                level: 3,
                text: "B".into(),
                slug: "b".into(),
            },
            Heading {
                level: 2,
                text: "C".into(),
                slug: "c".into(),
            },
        ];
        let tree = build_heading_tree(&headings);
        assert_eq!(tree.len(), 2);
        assert_eq!(tree[0].heading.text, "A");
        assert_eq!(tree[0].children.len(), 1);
        assert_eq!(tree[0].children[0].heading.text, "B");
        assert_eq!(tree[1].heading.text, "C");
    }

    #[test]
    fn test_flatten_tree_depth() {
        let headings = vec![
            Heading {
                level: 1,
                text: "A".into(),
                slug: "a".into(),
            },
            Heading {
                level: 2,
                text: "B".into(),
                slug: "b".into(),
            },
            Heading {
                level: 3,
                text: "C".into(),
                slug: "c".into(),
            },
        ];
        let tree = build_heading_tree(&headings);
        let flat = flatten_tree(&tree, 0);
        assert_eq!(flat[0].0, 0);
        assert_eq!(flat[1].0, 1);
        assert_eq!(flat[2].0, 2);
    }
}
