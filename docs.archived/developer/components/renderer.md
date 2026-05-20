# Renderer Component

The `tachyon-renderer` crate handles Markdown to HTML conversion.

## Overview

Renderer provides:
- CommonMark and GFM parsing
- Syntax highlighting
- KaTeX math rendering
- Custom block processing
- Caching

## Architecture

```
┌────────────────────────────────────────┐
│             Renderer                    │
│  ┌──────────────────────────────────┐  │
│  │          Parser                  │  │
│  │  (pulldown-cmark with SIMD)      │  │
│  └──────────────────────────────────┘  │
│  ┌──────────────────────────────────┐  │
│  │       AST Processor              │  │
│  │  - Code highlighting             │  │
│  │  - Math rendering                │  │
│  │  - Custom blocks                 │  │
│  └──────────────────────────────────┘  │
│  ┌──────────────────────────────────┐  │
│  │          Cache                   │  │
│  │  (LRU with TTL)                  │  │
│  └──────────────────────────────────┘  │
└────────────────────────────────────────┘
```

## Key Types

### Renderer

```rust
pub struct Renderer {
    cache: LruCache<ContentHash, RenderedDocument>,
    options: RenderOptions,
    highlighter: SyntaxHighlighter,
}

pub struct RenderOptions {
    pub enable_gfm: bool,
    pub enable_math: bool,
    pub enable_diagrams: bool,
    pub syntax_theme: Theme,
}

pub struct RenderedDocument {
    pub html: String,
    pub metadata: DocumentMetadata,
    pub toc: Vec<TocEntry>,
}
```

### Output Formats

```rust
pub enum OutputFormat {
    Html,
    PlainText,
    Ast,
    Markdown,
}
```

## Usage

### Basic Rendering

```rust
let renderer = Renderer::new(RenderOptions::default());
let output = renderer.render("# Hello\n\nWorld")?;
println!("{}", output.html);
```

### With Options

```rust
let options = RenderOptions {
    enable_gfm: true,
    enable_math: true,
    enable_diagrams: true,
    syntax_theme: Theme::Dark,
};

let renderer = Renderer::new(options);
let output = renderer.render(markdown)?;
```

### Metadata Extraction

```rust
let output = renderer.render(markdown)?;
println!("Title: {:?}", output.metadata.title);
println!("Word count: {}", output.metadata.word_count);
println!("Headings: {}", output.metadata.heading_count);
```

## Markdown Extensions

### GitHub Flavored Markdown

- Tables
- Task lists
- Strikethrough
- Autolinks
- Footnotes

### Code Blocks

```rust
pub struct CodeBlock {
    pub language: String,
    pub code: String,
    pub highlighted: Option<String>,
}
```

Supported languages:
- Rust, Python, JavaScript, TypeScript
- JSON, TOML, YAML
- HTML, CSS, SQL
- Bash, Markdown

### Mathematics

Inline: `$E = mc^2$`
Block:
```
$$
\int_0^\infty e^{-x^2} dx = \frac{\sqrt{\pi}}{2}
$$
```

### Custom Blocks

```markdown
::: tip
This is a tip.
:::

::: warning
This is a warning.
:::

::: internal
Only visible to internal users.
:::
```

## Caching

### Cache Key

```rust
pub struct ContentHash([u8; 32]);

impl ContentHash {
    pub fn new(content: &str, options: &RenderOptions) -> Self {
        let mut hasher = Blake3Hasher::new();
        hasher.update(content.as_bytes());
        hasher.update(&options.to_bytes());
        Self(hasher.finalize())
    }
}
```

### Cache Behavior

```rust
impl Renderer {
    pub fn render(&mut self, content: &str) -> Result<RenderedDocument> {
        let hash = ContentHash::new(content, &self.options);
        
        if let Some(cached) = self.cache.get(&hash) {
            return Ok(cached.clone());
        }
        
        let rendered = self.render_uncached(content)?;
        self.cache.put(hash, rendered.clone());
        Ok(rendered)
    }
}
```

## Performance

### Targets

| Operation | Target |
|-----------|--------|
| Small document (<1KB) | < 1ms |
| Medium document (10KB) | < 5ms |
| Large document (100KB) | < 15ms |
| Cache hit | < 1ms |

### Benchmarks

```rust
#[bench]
fn bench_render_medium(b: &mut Bencher) {
    let renderer = Renderer::new(Default::default());
    let content = include_str!("../fixtures/medium.md");
    
    b.iter(|| renderer.render(content));
}
```

## Configuration

```toml
[rendering]
enable_gfm = true
enable_math = true
enable_diagrams = true
syntax_theme = "dark"
cache_size = 1000
cache_ttl_seconds = 3600
```

## Error Handling

```rust
#[derive(Debug, thiserror::Error)]
pub enum RenderError {
    #[error("Parse error: {0}")]
    Parse(String),
    
    #[error("Math rendering error: {0}")]
    Math(String),
    
    #[error("Invalid code block: {0}")]
    CodeBlock(String),
}
```
