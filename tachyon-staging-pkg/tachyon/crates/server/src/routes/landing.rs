pub async fn landing_page() -> axum::response::Html<String> {
    // If the static directory contains an index.html (e.g. trunk-built frontend),
    // serve that instead of the hardcoded landing page. This enables the full
    // web GUI when TACHYON_STATIC_DIR points to a trunk output directory.
    let static_dir = crate::config::static_dir();
    let frontend_index = std::path::Path::new(&static_dir).join("index.html");
    if frontend_index.is_file() {
        match std::fs::read_to_string(&frontend_index) {
            Ok(html) => return axum::response::Html(html),
            Err(e) => {
                tracing::warn!("Failed to read frontend index.html: {}", e);
            }
        }
    }
    axum::response::Html(HTML.to_string())
}

const HTML: &str = r##"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>Tachyon</title>
<style>
  *, *::before, *::after { margin: 0; padding: 0; box-sizing: border-box; }

  body {
    font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Helvetica, Arial, sans-serif, "Apple Color Emoji", "Segoe UI Emoji";
    background: #fff;
    color: #111;
    line-height: 1.6;
    padding: 0;
  }

  @media (prefers-color-scheme: dark) {
    body { background: #111; color: #e5e5e5; }
    a { color: #60a5fa; }
    a:hover { color: #93c5fd; }
    code { background: #1a1a1a; border-color: #333; }
    hr { border-color: #333; }
    footer { border-color: #333; }
  }

  a { color: #2563eb; text-decoration: none; }
  a:hover { text-decoration: underline; }

  header {
    max-width: 720px;
    margin: 0 auto;
    padding: 96px 24px 48px;
  }

  h1 {
    font-size: 3rem;
    font-weight: 800;
    letter-spacing: -0.03em;
    margin-bottom: 12px;
  }

  .tagline {
    font-size: 1.125rem;
    color: #555;
    max-width: 520px;
  }

  @media (prefers-color-scheme: dark) {
    .tagline { color: #999; }
  }

  main {
    max-width: 720px;
    margin: 0 auto;
    padding: 0 24px 48px;
  }

  section { margin-bottom: 48px; }

  h2 {
    font-size: 1.5rem;
    font-weight: 700;
    letter-spacing: -0.02em;
    margin-bottom: 16px;
    padding-bottom: 8px;
    border-bottom: 2px solid #111;
  }

  @media (prefers-color-scheme: dark) {
    h2 { border-color: #e5e5e5; }
  }

  ul {
    list-style: none;
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 8px;
  }

  @media (max-width: 480px) {
    ul { grid-template-columns: 1fr; }
  }

  li {
    padding: 8px 0;
    border-bottom: 1px solid #ddd;
    font-size: 0.9375rem;
  }

  @media (prefers-color-scheme: dark) {
    li { border-color: #2a2a2a; }
  }

  li strong {
    display: block;
    margin-bottom: 2px;
  }

  li span {
    color: #666;
    font-size: 0.8125rem;
  }

  @media (prefers-color-scheme: dark) {
    li span { color: #888; }
  }

  code {
    display: block;
    font-family: "SF Mono", "Fira Code", "Fira Mono", "Roboto Mono", "Cascadia Code", Menlo, Consolas, monospace;
    font-size: 0.875rem;
    background: #f5f5f5;
    border: 1px solid #ddd;
    padding: 16px;
    overflow-x: auto;
    line-height: 1.5;
  }

  .links {
    display: flex;
    gap: 24px;
    flex-wrap: wrap;
  }

  .links a {
    font-size: 0.9375rem;
    font-weight: 500;
  }

  hr {
    border: none;
    border-top: 1px solid #ddd;
    margin: 48px 0 0;
  }

  @media (prefers-color-scheme: dark) {
    hr { border-color: #333; }
  }

  footer {
    max-width: 720px;
    margin: 0 auto;
    padding: 16px 24px 48px;
    font-size: 0.8125rem;
    color: #888;
    border-bottom: 2px solid #ddd;
  }

  @media (prefers-color-scheme: dark) {
    footer { color: #666; border-color: #333; }
  }
</style>
</head>
<body>

<header>
  <h1>Tachyon</h1>
  <p class="tagline">Self-hosted knowledge management. Rust-native. No vendor lock-in.</p>
</header>

<main>

  <section>
    <h2>Features</h2>
    <ul>
      <li><strong>Real-time collaboration</strong><span>Conflict-free replicated data types (CRDT)</span></li>
      <li><strong>Semantic search</strong><span>Full-text and vector search via pgvector</span></li>
      <li><strong>Static site generation</strong><span>Publish knowledge bases as static sites</span></li>
      <li><strong>WASM plugin system</strong><span>Extend functionality with sandboxed WebAssembly plugins</span></li>
      <li><strong>REST API + GraphQL</strong><span>Dual API surface for integration and queries</span></li>
      <li><strong>Import / Export</strong><span>Markdown, DOCX, CSV and more</span></li>
    </ul>
  </section>

  <section>
    <h2>Quick start</h2>
    <code>docker compose up</code>
  </section>

  <section>
    <h2>Links</h2>
    <div class="links">
      <a href="/swagger-ui">API documentation</a>
      <a href="/health">Health status</a>
      <a href="/graphql/playground">GraphQL playground</a>
    </div>
  </section>

</main>

<hr>

<footer>
  Open source under Apache-2.0
</footer>

</body>
</html>"##;

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_landing_page_returns_html() {
        let response = landing_page().await;
        let body = response.0;
        assert!(body.contains("Tachyon"), "response must contain 'Tachyon'");
        assert!(
            body.contains("docker compose"),
            "response must contain 'docker compose'"
        );
        assert!(
            body.contains("swagger-ui"),
            "response must contain 'swagger-ui'"
        );
    }
}
