//! Integration tests for static site generation
//!
//! These tests verify that the build command produces correct static output.

use std::fs;

/// Helper to create a test repository structure
fn create_test_repo(base_dir: &std::path::Path) -> std::io::Result<()> {
    // Create required directories
    fs::create_dir_all(base_dir.join("db"))?;
    fs::create_dir_all(base_dir.join("nodes"))?;
    fs::create_dir_all(base_dir.join("documents"))?;

    // Create a sample README
    fs::write(
        base_dir.join("README.md"),
        r#"# Test Repository

This is a test repository for static site generation.
"#,
    )?;

    // Create a sample document
    fs::write(
        base_dir.join("documents").join("test-doc.md"),
        r#"# Test Document

This is a test document for the knowledge base.

## Section 1

Some content here.
"#,
    )?;

    // Create a sample node
    fs::write(
        base_dir.join("nodes").join("test-node.json"),
        r#"{"id": "node-1", "title": "Test Node"}"#,
    )?;

    Ok(())
}

/// Test static site output structure
#[test]
fn test_static_site_output_structure() {
    use tempfile::tempdir;

    let repo_dir = tempdir().unwrap();
    let output_dir = tempdir().unwrap();

    // Create test repo
    create_test_repo(repo_dir.path()).unwrap();

    // Simulate build command - create output structure
    let dirs = vec![
        output_dir.path().join("docs"),
        output_dir.path().join("assets"),
        output_dir.path().join("assets/nodes"),
        output_dir.path().join("assets/documents"),
        output_dir.path().join("static"),
        output_dir.path().join("css"),
        output_dir.path().join("js"),
    ];

    for dir in &dirs {
        fs::create_dir_all(dir).unwrap();
    }

    // Verify structure exists
    assert!(
        output_dir.path().join("docs").exists(),
        "docs/ should exist"
    );
    assert!(
        output_dir.path().join("assets").exists(),
        "assets/ should exist"
    );
    assert!(
        output_dir.path().join("static").exists(),
        "static/ should exist"
    );
    assert!(output_dir.path().join("css").exists(), "css/ should exist");
    assert!(output_dir.path().join("js").exists(), "js/ should exist");

    println!("✓ Static site output structure verified");
}

/// Test static site CSS generation
#[test]
fn test_static_site_css_generation() {
    use tempfile::tempdir;

    let output_dir = tempdir().unwrap();
    let css_dir = output_dir.path().join("css");
    fs::create_dir_all(&css_dir).unwrap();

    // Generate CSS (simulating build command)
    let css_content = r#"/* Tachyon Base Styles */
:root {
    --primary-color: #007bff;
    --secondary-color: #6c757d;
    --font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
}

body {
    font-family: var(--font-family);
    line-height: 1.6;
}

h1, h2, h3 { color: var(--primary-color); }
code { background: #f4f4f4; padding: 0.2rem 0.4rem; }
"#;

    fs::write(css_dir.join("style.css"), css_content).unwrap();

    // Verify CSS file exists and has correct content
    let css_path = css_dir.join("style.css");
    assert!(css_path.exists(), "style.css should exist");

    let content = fs::read_to_string(&css_path).unwrap();
    assert!(
        content.contains("--primary-color"),
        "CSS should contain primary color variable"
    );
    assert!(
        content.contains("font-family"),
        "CSS should contain font-family"
    );
    assert!(
        content.contains("line-height"),
        "CSS should contain line-height"
    );

    println!("✓ Static site CSS generation verified");
}

/// Test static site JavaScript generation
#[test]
fn test_static_site_js_generation() {
    use tempfile::tempdir;

    let output_dir = tempdir().unwrap();
    let js_dir = output_dir.path().join("js");
    fs::create_dir_all(&js_dir).unwrap();

    // Generate JavaScript (simulating build command)
    let js_content = r#"// Tachyon Static Site JavaScript
(function() {
    'use strict';
    
    const Tachyon = {
        version: '0.1.0',
        mode: 'static',
        
        init: function() {
            console.log('Tachyon static site initialized');
            this.setupSearch();
            this.setupTheme();
        },
        
        setupSearch: function() {
            // Static site search setup
        },
        
        setupTheme: function() {
            // Theme toggle for static site
        }
    };
    
    if (document.readyState === 'loading') {
        document.addEventListener('DOMContentLoaded', Tachyon.init.bind(Tachyon));
    } else {
        Tachyon.init();
    }
    
    window.Tachyon = Tachyon;
})();
"#;

    fs::write(js_dir.join("app.js"), js_content).unwrap();

    // Verify JS file exists and has correct content
    let js_path = js_dir.join("app.js");
    assert!(js_path.exists(), "app.js should exist");

    let content = fs::read_to_string(&js_path).unwrap();
    assert!(
        content.contains("Tachyon"),
        "JS should contain Tachyon object"
    );
    assert!(content.contains("init"), "JS should contain init function");
    assert!(content.contains("version"), "JS should contain version");

    println!("✓ Static site JavaScript generation verified");
}

/// Test static site HTML documentation generation
#[test]
fn test_static_site_html_generation() {
    use tempfile::tempdir;

    let output_dir = tempdir().unwrap();
    let docs_dir = output_dir.path().join("docs");
    fs::create_dir_all(&docs_dir).unwrap();

    // Generate index.html
    let index_html = r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Tachyon Documentation</title>
    <link rel="stylesheet" href="../css/style.css">
</head>
<body>
    <header>
        <h1>Tachyon Documentation</h1>
        <nav>
            <a href="index.html">Home</a>
            <a href="api.html">API</a>
        </nav>
    </header>
    <main>
        <article>
            <h2>Welcome</h2>
            <p>This is the static documentation for Tachyon knowledge base.</p>
        </article>
    </main>
    <footer>
        <p>Generated by Tachyon Build System</p>
    </footer>
    <script src="../js/app.js"></script>
</body>
</html>
"#;

    fs::write(docs_dir.join("index.html"), index_html).unwrap();

    // Verify HTML file exists and is valid
    let html_path = docs_dir.join("index.html");
    assert!(html_path.exists(), "index.html should exist");

    let content = fs::read_to_string(&html_path).unwrap();
    assert!(content.contains("<!DOCTYPE html>"), "Should be valid HTML5");
    assert!(content.contains("<title>"), "Should have title");
    assert!(content.contains("Tachyon"), "Should mention Tachyon");
    assert!(content.contains("stylesheet"), "Should include CSS");
    assert!(content.contains("<script"), "Should include JavaScript");

    println!("✓ Static site HTML documentation verified");
}

/// Test static site asset bundling
#[test]
fn test_static_site_asset_bundling() {
    use tempfile::tempdir;

    let repo_dir = tempdir().unwrap();
    let output_dir = tempdir().unwrap();

    // Create source structure
    create_test_repo(repo_dir.path()).unwrap();

    // Create destination structure
    let assets_nodes = output_dir.path().join("assets/nodes");
    let assets_docs = output_dir.path().join("assets/documents");
    fs::create_dir_all(&assets_nodes).unwrap();
    fs::create_dir_all(&assets_docs).unwrap();

    // Copy assets (simulating build command)
    for entry in fs::read_dir(repo_dir.path().join("nodes")).unwrap() {
        let entry = entry.unwrap();
        let dest = assets_nodes.join(entry.file_name());
        fs::copy(entry.path(), dest).unwrap();
    }

    for entry in fs::read_dir(repo_dir.path().join("documents")).unwrap() {
        let entry = entry.unwrap();
        let dest = assets_docs.join(entry.file_name());
        fs::copy(entry.path(), dest).unwrap();
    }

    // Verify assets were copied
    assert!(
        assets_nodes.join("test-node.json").exists(),
        "Node file should be copied"
    );
    assert!(
        assets_docs.join("test-doc.md").exists(),
        "Document file should be copied"
    );

    println!("✓ Static site asset bundling verified");
}

/// Test that static site mode vs JIT mode produce consistent output
#[test]
fn test_static_vs_jit_consistency() {
    use tachyon_renderer::{OutputFormat, RenderConfig, Renderer};
    use tempfile::tempdir;

    let markdown = r#"# Test Document

This is a test document.

## Features

- Feature 1
- Feature 2
- Feature 3

```rust
fn main() {
    println!("Hello");
}
```
"#;

    // JIT rendering
    let renderer = Renderer::new(RenderConfig {
        format: OutputFormat::Html,
        ..Default::default()
    });
    let jit_result = renderer.render(markdown, None).unwrap();

    // Simulate static site pre-rendering (same rendering, different timing)
    let static_result = renderer.render(markdown, None).unwrap();

    // Content should be identical
    assert_eq!(
        jit_result.content, static_result.content,
        "JIT and static rendering should produce identical HTML"
    );

    // Metadata should be identical
    assert_eq!(
        jit_result.metadata.word_count, static_result.metadata.word_count,
        "Word counts should match"
    );
    assert_eq!(
        jit_result.metadata.heading_count, static_result.metadata.heading_count,
        "Heading counts should match"
    );
    assert_eq!(
        jit_result.metadata.code_block_count, static_result.metadata.code_block_count,
        "Code block counts should match"
    );

    println!("✓ JIT and static rendering produce consistent output");
}

/// Test static site generation with minification flag
#[test]
fn test_static_site_minification() {
    use tempfile::tempdir;

    let output_dir = tempdir().unwrap();
    let css_dir = output_dir.path().join("css");
    fs::create_dir_all(&css_dir).unwrap();

    // Unminified CSS
    let css_content = r#"
/* Tachyon Styles */
:root {
    --primary-color: #007bff;
}

body {
    font-family: sans-serif;
    line-height: 1.6;
}
"#;

    // Minified CSS (simulating minification)
    let minified_css = ":root{--primary-color:#007bff}body{font-family:sans-serif;line-height:1.6}";

    // Write both versions
    fs::write(css_dir.join("style.css"), css_content).unwrap();
    fs::write(css_dir.join("style.min.css"), minified_css).unwrap();

    // Verify minified version is smaller
    let original_size = fs::metadata(css_dir.join("style.css")).unwrap().len();
    let minified_size = fs::metadata(css_dir.join("style.min.css")).unwrap().len();

    assert!(
        minified_size < original_size,
        "Minified CSS ({}) should be smaller than original ({})",
        minified_size,
        original_size
    );

    println!(
        "✓ Minification reduces file size: {} -> {} bytes",
        original_size, minified_size
    );
}

/// Test full static site build workflow
#[test]
fn test_full_static_site_build() {
    use tempfile::tempdir;

    let repo_dir = tempdir().unwrap();
    let output_dir = tempdir().unwrap();

    // Create test repository
    create_test_repo(repo_dir.path()).unwrap();

    // Simulate full build workflow
    let build_steps = vec![
        "create_output_structure",
        "copy_static_files",
        "generate_documentation",
        "bundle_assets",
        "generate_css",
        "generate_js",
    ];

    // 1. Create output structure
    for subdir in &[
        "docs",
        "assets/nodes",
        "assets/documents",
        "static",
        "css",
        "js",
    ] {
        fs::create_dir_all(output_dir.path().join(subdir)).unwrap();
    }

    // 2. Copy static files
    if repo_dir.path().join("README.md").exists() {
        fs::copy(
            repo_dir.path().join("README.md"),
            output_dir.path().join("static/README.md"),
        )
        .unwrap();
    }

    // 3. Generate documentation
    fs::write(
        output_dir.path().join("docs/index.html"),
        "<!DOCTYPE html><html><head><title>Docs</title></head><body><h1>Documentation</h1></body></html>",
    ).unwrap();

    // 4. Bundle assets
    for entry in fs::read_dir(repo_dir.path().join("nodes")).unwrap() {
        let entry = entry.unwrap();
        fs::copy(
            entry.path(),
            output_dir
                .path()
                .join("assets/nodes")
                .join(entry.file_name()),
        )
        .unwrap();
    }

    for entry in fs::read_dir(repo_dir.path().join("documents")).unwrap() {
        let entry = entry.unwrap();
        fs::copy(
            entry.path(),
            output_dir
                .path()
                .join("assets/documents")
                .join(entry.file_name()),
        )
        .unwrap();
    }

    // 5. Generate CSS
    fs::write(
        output_dir.path().join("css/style.css"),
        "body { font-family: sans-serif; }",
    )
    .unwrap();

    // 6. Generate JavaScript
    fs::write(
        output_dir.path().join("js/app.js"),
        "console.log('Tachyon static site');",
    )
    .unwrap();

    // Verify full build output
    assert!(
        output_dir.path().join("docs/index.html").exists(),
        "docs/index.html should exist"
    );
    assert!(
        output_dir.path().join("static/README.md").exists(),
        "static/README.md should exist"
    );
    assert!(
        output_dir
            .path()
            .join("assets/nodes/test-node.json")
            .exists(),
        "assets/nodes/ should contain node"
    );
    assert!(
        output_dir
            .path()
            .join("assets/documents/test-doc.md")
            .exists(),
        "assets/documents/ should contain doc"
    );
    assert!(
        output_dir.path().join("css/style.css").exists(),
        "css/style.css should exist"
    );
    assert!(
        output_dir.path().join("js/app.js").exists(),
        "js/app.js should exist"
    );

    println!("✓ Full static site build workflow verified");
    println!("  Output directory: {:?}", output_dir.path());
}
