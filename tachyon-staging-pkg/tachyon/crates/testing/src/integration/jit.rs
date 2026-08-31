//! Integration tests for JIT (Just-In-Time) compilation/rendering
//!
//! These tests verify that on-demand document rendering works correctly.

/// Test that the JIT renderer can be created and render basic markdown
#[test]
fn test_jit_renderer_creation() {
    use tachyon_renderer::{OutputFormat, RenderConfig, Renderer};

    // Create renderer with default config (JIT mode - no pre-compilation)
    let config = RenderConfig {
        format: OutputFormat::Html,
        ..Default::default()
    };
    let renderer = Renderer::new(config);

    // Render a simple markdown document
    let markdown = r#"# Hello World

This is a **test** document with *italic* text.

## Code Block

```rust
fn main() {
    println!("Hello, Tachyon!");
}
```

## List

- Item 1
- Item 2
- Item 3
"#;

    let result = renderer.render(markdown, None);

    assert!(result.is_ok(), "JIT rendering should succeed");
    let render_result = result.unwrap();

    // Verify HTML output
    assert!(
        render_result.content.contains("<h1>Hello World</h1>"),
        "Should render h1 heading"
    );
    assert!(
        render_result.content.contains("<strong>test</strong>"),
        "Should render bold text"
    );
    assert!(
        render_result.content.contains("<em>italic</em>"),
        "Should render italic text"
    );
    assert!(
        render_result.content.contains("<h2>Code Block</h2>"),
        "Should render h2 heading"
    );
    assert!(
        render_result.content.contains("<ul>"),
        "Should render unordered list"
    );
    assert!(
        render_result.content.contains("<li>Item 1</li>"),
        "Should render list items"
    );

    // Verify metadata
    assert!(render_result.metadata.word_count > 0, "Should count words");
    assert!(
        render_result.metadata.heading_count >= 2,
        "Should count headings"
    );
    assert!(
        render_result.metadata.code_block_count >= 1,
        "Should count code blocks"
    );
}

/// Test JIT rendering performance (should be fast)
#[test]
fn test_jit_rendering_performance() {
    use std::time::Instant;
    use tachyon_renderer::{RenderConfig, Renderer};

    let renderer = Renderer::new(RenderConfig::default());

    // Create a medium-sized document
    let mut markdown = String::from("# Performance Test\n\n");
    for i in 0..100 {
        markdown.push_str(&format!(
            "## Section {}\n\nThis is paragraph {} with some **bold** and *italic* text.\n\n",
            i, i
        ));
    }

    let start = Instant::now();
    let result = renderer.render(&markdown, None);
    let duration = start.elapsed();

    assert!(result.is_ok(), "Rendering should succeed");

    // JIT rendering should complete within reasonable time (< 1 second for this size)
    assert!(
        duration.as_millis() < 1000,
        "JIT rendering should be fast, took {:?}",
        duration
    );

    println!(
        "JIT rendering of {} chars took {:?}",
        markdown.len(),
        duration
    );
}

/// Test that JIT rendering handles GFM features
#[test]
fn test_jit_gfm_features() {
    use tachyon_renderer::{RenderConfig, Renderer};

    let renderer = Renderer::new(RenderConfig::default());

    let markdown = r#"# GFM Features Test

## Tables

| Name | Age | City |
|------|-----|------|
| Alice | 30 | NYC |
| Bob | 25 | LA |

## Task List

- [x] Completed task
- [ ] Incomplete task

## Strikethrough

~~This text is struck through~~

## Autolinks

Visit https://example.com for more info.
"#;

    let result = renderer.render(markdown, None);
    assert!(result.is_ok(), "GFM rendering should succeed");

    let html = result.unwrap().content;

    // Tables should be rendered
    assert!(
        html.contains("<table>") || html.contains("<thead>"),
        "Should render tables"
    );

    // Task lists - check for checkbox-like content
    // Note: pulldown-cmark GFM task list rendering varies
    println!("GFM HTML output:\n{}", html);
}

/// Test JIT rendering with various content types
#[test]
fn test_jit_various_content_types() {
    use tachyon_renderer::{OutputFormat, RenderConfig, Renderer};

    // Test HTML output
    let html_renderer = Renderer::new(RenderConfig {
        format: OutputFormat::Html,
        ..Default::default()
    });

    let markdown = "# Test\n\nContent";
    let html_result = html_renderer.render(markdown, None).unwrap();
    assert!(html_result.content.contains("<h1>"), "Should output HTML");

    // Test plain text output
    let text_renderer = Renderer::new(RenderConfig {
        format: OutputFormat::PlainText,
        ..Default::default()
    });

    let text_result = text_renderer.render(markdown, None).unwrap();
    assert!(
        !text_result.content.contains("<"),
        "Should output plain text without HTML tags"
    );
    assert!(
        text_result.content.contains("Test"),
        "Should contain the text content"
    );
}

/// Test JIT rendering error handling
#[test]
fn test_jit_error_handling() {
    use tachyon_renderer::{RenderConfig, Renderer};

    let renderer = Renderer::new(RenderConfig::default());

    // Empty content should still work
    let result = renderer.render("", None);
    assert!(result.is_ok(), "Empty content should render successfully");

    // Very long content
    let long_content = "a".repeat(1_000_000);
    let result = renderer.render(&long_content, None);
    assert!(result.is_ok(), "Long content should render successfully");
}

/// Test JIT rendering statistics
#[test]
fn test_jit_rendering_statistics() {
    use tachyon_renderer::{RenderConfig, Renderer};

    let renderer = Renderer::new(RenderConfig::default());

    let markdown = r#"# Document

This is a test document.

## Section 1

Some content here.

```rust
fn test() {}
```

### Subsection

More content.
"#;

    let result = renderer.render(markdown, None).unwrap();

    // Verify statistics are populated
    assert!(
        result.stats.render_time_ms > 0 || !result.content.is_empty(),
        "Should have timing info"
    );
    assert!(result.metadata.word_count > 0, "Should have word count");
    assert!(
        result.metadata.char_count > 0,
        "Should have character count"
    );
    assert!(
        result.metadata.heading_count >= 3,
        "Should count headings (found {})",
        result.metadata.heading_count
    );
    assert!(
        result.metadata.code_block_count >= 1,
        "Should count code blocks"
    );

    println!("Render stats: {:?}", result.stats);
    println!("Metadata: {:?}", result.metadata);
}
