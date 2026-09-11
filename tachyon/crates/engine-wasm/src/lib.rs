//! Tachyon Engine WASM
//!
//! WebAssembly bindings over the Tachyon Rust core, designed for consumption
//! by the SolidJS/Astro frontend (`apps/web`) and any other WyattAu project
//! that needs the engine.
//!
//! Scope (v1): markdown -> sanitized HTML, wikilink extraction, slugs.
//! LaTeX is passed through untouched so the host page can render it with
//! client-side KaTeX (the standard frontend pattern).

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn engine_version() -> String {
    engine_version_impl()
}

/// Render markdown to sanitized HTML.
///
/// Pipeline: wikilink pre-processing -> pulldown-cmark -> ammonia sanitize.
#[wasm_bindgen]
pub fn render_markdown(content: &str) -> String {
    render_markdown_impl(content)
}

/// Extract wiki-link targets from markdown content.
///
/// Matches `[[target]]` and `[[target|display]]`; returns unique targets
/// in order of first appearance.
#[wasm_bindgen]
pub fn extract_wikilinks(content: &str) -> Vec<JsValue> {
    extract_wikilinks_impl(content)
        .into_iter()
        .map(|s| JsValue::from_str(&s))
        .collect()
}

/// Convert arbitrary text into a URL-safe slug.
///
/// Mirrors `tachyon_core::util::slugify` so client and server produce
/// identical slugs — keep the two in sync.
#[wasm_bindgen]
pub fn slugify(input: &str) -> String {
    slugify_impl(input)
}

// ---------------------------------------------------------------------------
// Plain (testable) implementations — thin #[wasm_bindgen] wrappers above.
// ---------------------------------------------------------------------------

fn engine_version_impl() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

fn render_markdown_impl(content: &str) -> String {
    let preprocessed = preprocess_wikilinks(content);
    let parser = pulldown_cmark::Parser::new(&preprocessed);
    let mut html = String::with_capacity(preprocessed.len() * 3 / 2);
    pulldown_cmark::html::push_html(&mut html, parser);
    ammonia::clean(&html)
}

fn extract_wikilinks_impl(content: &str) -> Vec<String> {
    let mut seen = std::collections::HashSet::new();
    let mut out = Vec::new();
    for target in WIKILINK_RE
        .captures_iter(content)
        .filter_map(|c| c.get(1))
        .map(|m| m.as_str().trim().to_string())
        .filter(|t| !t.is_empty())
    {
        if seen.insert(target.clone()) {
            out.push(target);
        }
    }
    out
}

fn slugify_impl(input: &str) -> String {
    static HYPHEN_COLLAPSE: std::sync::LazyLock<regex::Regex> =
        std::sync::LazyLock::new(|| regex::Regex::new("-{2,}").expect("valid hyphen regex"));
    let slug = input
        .to_lowercase()
        .chars()
        .map(|c| {
            if c.is_alphanumeric() {
                c
            } else if c.is_whitespace() || c == '-' || c == '_' {
                '-'
            } else {
                '\0' // Mark for removal
            }
        })
        .filter(|c| *c != '\0')
        .collect::<String>();
    HYPHEN_COLLAPSE
        .replace_all(&slug, "-")
        .trim_matches('-')
        .to_string()
}

static WIKILINK_RE: std::sync::LazyLock<regex::Regex> = std::sync::LazyLock::new(|| {
    regex::Regex::new(r"\[\[([^\]|]+)(?:\|[^\]]+)?\]\]").expect("valid wikilink regex")
});

/// Convert `[[target]]` / `[[target|display]]` into internal links so
/// pulldown-cmark renders them as anchors.
fn preprocess_wikilinks(content: &str) -> String {
    // Protect fenced code blocks from wikilink substitution.
    let mut out = String::with_capacity(content.len());
    let mut in_fence = false;
    for line in content.split_inclusive('\n') {
        let trimmed = line.trim_start();
        if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
            in_fence = !in_fence;
            out.push_str(line);
            continue;
        }
        if in_fence {
            out.push_str(line);
        } else {
            let replaced = WIKILINK_RE.replace(line, |caps: &regex::Captures| {
                let target = caps.get(1).map(|m| m.as_str().trim()).unwrap_or("");
                let display = caps
                    .get(2)
                    .map(|m| m.as_str().trim())
                    .filter(|s| !s.is_empty())
                    .unwrap_or(target);
                format!("[{display}](/documents/{})", slugify(target))
            });
            out.push_str(&replaced);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_basic_markdown() {
        let html = render_markdown("# Hello\n\nWorld");
        assert!(html.contains("<h1>Hello</h1>"));
        assert!(html.contains("<p>World</p>"));
    }

    #[test]
    fn render_sanitizes_html() {
        let html = render_markdown("<script>alert(1)</script>ok");
        assert!(!html.contains("<script>"));
        assert!(html.contains("ok"));
    }

    #[test]
    fn wikilinks_converted_and_rendered() {
        let html = render_markdown("see [[My Doc]] and [[Other|the other]]");
        assert!(html.contains("/documents/my-doc"));
        assert!(html.contains("the other"));
        // code fences are protected
        let html2 = render_markdown("```\n[[Not A Link]]\n```");
        assert!(!html2.contains("/documents/not-a-link"));
    }

    #[test]
    fn extract_unique_targets() {
        let links = extract_wikilinks_impl("[[A]] [[B|bee]] [[A]]");
        assert_eq!(links.len(), 2);
        assert_eq!(links[0], "A");
        assert_eq!(links[1], "B");
    }

    #[test]
    fn slug_matches_core() {
        assert_eq!(slugify("Hello World"), "hello-world");
        assert_eq!(slugify("  multiple   spaces  "), "multiple-spaces");
        assert_eq!(slugify("Rust & WebAssembly"), "rust-webassembly");
    }
}
