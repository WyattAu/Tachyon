# Testing Guide

Comprehensive guide to testing in Tachyon.

## Overview

Tachyon uses a multi-layered testing strategy:

- **Unit Tests**: Test individual functions and types
- **Integration Tests**: Test component interactions
- **End-to-End Tests**: Test full user workflows

## Running Tests

### All Tests

```bash
cargo test
```

### Specific Crate

```bash
cargo test -p tachyon-server
```

### Specific Test

```bash
cargo test test_document_creation
```

### With Output

```bash
cargo test -- --nocapture
```

### Watch Mode

```bash
cargo watch -x test
```

## Unit Tests

Unit tests are located in the same file as the code they test:

```rust
// src/document.rs

pub fn parse_title(content: &str) -> Option<&str> {
    content.lines().next()?.strip_prefix('#')?.trim().into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_title() {
        let content = "# My Title\n\nContent";
        assert_eq!(parse_title(content), Some("My Title"));
    }

    #[test]
    fn test_no_title() {
        let content = "No title here";
        assert_eq!(parse_title(content), None);
    }
}
```

### Test Organization

```
crates/server/
├── src/
│   ├── lib.rs
│   ├── document.rs      # Includes #[cfg(test)] mod tests
│   └── search.rs
└── tests/               # Integration tests
    ├── api_test.rs
    └── common/
        └── mod.rs
```

## Integration Tests

Integration tests are in the `tests/` directory:

```rust
// tests/api_test.rs

use tachyon_server::test_utils::TestServer;
use reqwest::StatusCode;

#[tokio::test]
async fn test_create_document() {
    let server = TestServer::spawn().await;
    
    let response = server
        .post("/api/v1/documents")
        .json(&json!({
            "title": "Test Document",
            "content": "# Hello"
        }))
        .send()
        .await;
    
    assert_eq!(response.status(), StatusCode::CREATED);
    
    let doc: Document = response.json().await;
    assert_eq!(doc.title, "Test Document");
}
```

### Test Utilities

```rust
// tests/common/mod.rs

use tachyon_server::{Server, Config};
use once_cell::sync::Lazy;

pub struct TestServer {
    pub url: String,
    pub db: Database,
}

impl TestServer {
    pub async fn spawn() -> Self {
        let config = Config {
            port: 0, // Random port
            database: ":memory:".into(),
            ..Default::default()
        };
        
        let server = Server::new(config).await.unwrap();
        let url = format!("http://{}", server.addr());
        
        Self {
            url,
            db: server.database(),
        }
    }
}
```

## Test Fixtures

### Test Data

```rust
pub fn sample_document() -> Document {
    Document {
        id: DocumentId::new(),
        title: "Test Document".into(),
        content: "# Test\n\nContent".into(),
        status: DocumentStatus::Published,
        visibility: Visibility::Public,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    }
}

pub fn sample_markdown() -> &'static str {
    r#"
# Sample Document

This is a sample document for testing.

## Features

- Feature 1
- Feature 2

```rust
fn main() {
    println!("Hello");
}
```
"#
}
```

### Database Fixtures

```rust
pub async fn setup_test_db() -> Database {
    let db = Database::in_memory().await.unwrap();
    db.run_migrations().await.unwrap();
    db
}

pub async fn seed_documents(db: &Database) -> Vec<Document> {
    let docs = vec![
        Document::new("Doc 1", "Content 1"),
        Document::new("Doc 2", "Content 2"),
    ];
    
    for doc in &docs {
        db.save(doc).await.unwrap();
    }
    
    docs
}
```

## Async Testing

```rust
#[tokio::test]
async fn test_async_operation() {
    let result = async_function().await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_concurrent_operations() {
    let results = futures::future::join_all([
        operation_one(),
        operation_two(),
        operation_three(),
    ]).await;
    
    assert!(results.iter().all(|r| r.is_ok()));
}
```

## Mocking

### Using mockall

```rust
use mockall::automock;

#[automock]
pub trait Repository {
    async fn get(&self, id: &str) -> Result<Document>;
    async fn save(&self, doc: &Document) -> Result<()>;
}

#[tokio::test]
async fn test_with_mock() {
    let mut mock = MockRepository::new();
    
    mock.expect_get()
        .with(eq("doc-1"))
        .times(1)
        .returning(|_| Ok(Document::default()));
    
    let service = DocumentService::new(Box::new(mock));
    let result = service.get_document("doc-1").await;
    
    assert!(result.is_ok());
}
```

### Using wiremock

```rust
use wiremock::{MockServer, Mock, ResponseTemplate};
use wiremock::matchers::{method, path};

#[tokio::test]
async fn test_external_api() {
    let mock_server = MockServer::start().await;
    
    Mock::given(method("GET"))
        .and(path("/api/data"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&json!({
            "status": "ok"
        })))
        .mount(&mock_server)
        .await;
    
    let client = ApiClient::new(&mock_server.uri());
    let result = client.get_data().await;
    
    assert!(result.is_ok());
}
```

## Test Categories

### Fast Tests

Unit tests run quickly and frequently:

```bash
cargo test --lib
```

### Slow Tests

Integration tests marked with `#[ignore]`:

```rust
#[test]
#[ignore = "slow"]
fn test_large_document_indexing() {
    // Takes several seconds
}
```

Run with:
```bash
cargo test -- --ignored
```

### Database Tests

Tests requiring database:

```rust
#[cfg(feature = "database-tests")]
#[tokio::test]
async fn test_database_operations() {
    // Requires DATABASE_URL
}
```

## Test Coverage

### Generating Coverage

```bash
# Install tarpaulin
cargo install cargo-tarpaulin

# Generate report
cargo tarpaulin --out Html --output-dir coverage/
```

### Coverage Targets

| Crate | Target |
|-------|--------|
| tachyon-core | 90% |
| tachyon-server | 80% |
| tachyon-renderer | 85% |
| tachyon-search | 80% |

## Performance Testing

### Benchmarks

```rust
// benches/render_bench.rs

use criterion::{criterion_group, criterion_main, Criterion};

fn render_benchmark(c: &mut Criterion) {
    let renderer = Renderer::new();
    let markdown = include_str!("../fixtures/large.md");
    
    c.bench_function("render_large", |b| {
        b.iter(|| renderer.render(markdown))
    });
}

criterion_group!(benches, render_benchmark);
criterion_main!(benches);
```

Run benchmarks:
```bash
cargo bench
```

### Stress Tests

```rust
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_concurrent_requests() {
    let server = TestServer::spawn().await;
    
    let results: Vec<_> = (0..100)
        .map(|i| {
            let url = server.url.clone();
            tokio::spawn(async move {
                reqwest::Client::new()
                    .get(&format!("{}/api/v1/documents", url))
                    .send()
                    .await
            })
        })
        .collect();
    
    let results = futures::future::join_all(results).await;
    assert!(results.iter().all(|r| r.is_ok()));
}
```

## End-to-End Tests

### Desktop E2E

Using WebDriver:

```rust
#[cfg(test)]
mod e2e {
    use thirtyfour::prelude::*;
    
    #[tokio::test]
    async fn test_desktop_app() -> WebDriverResult<()> {
        let caps = DesiredCapabilities::chrome();
        let driver = WebDriver::new("http://localhost:4444", caps).await?;
        
        driver.goto("http://localhost:8080").await?;
        
        let title = driver.find(By::Css("h1")).await?;
        assert_eq!(title.text().await?, "Welcome to Tachyon");
        
        driver.quit().await?;
        Ok(())
    }
}
```

### API E2E

```rust
#[tokio::test]
async fn test_document_lifecycle() {
    let client = TestClient::new();
    
    // Create
    let create_resp = client
        .post("/api/v1/documents")
        .json(&json!({
            "title": "E2E Test",
            "content": "# Test"
        }))
        .send()
        .await;
    assert_eq!(create_resp.status(), 201);
    let doc_id = create_resp.json::<Document>().await.id;
    
    // Read
    let read_resp = client
        .get(&format!("/api/v1/documents/{}", doc_id))
        .send()
        .await;
    assert_eq!(read_resp.status(), 200);
    
    // Update
    let update_resp = client
        .put(&format!("/api/v1/documents/{}", doc_id))
        .json(&json!({"content": "# Updated"}))
        .send()
        .await;
    assert_eq!(update_resp.status(), 200);
    
    // Delete
    let delete_resp = client
        .delete(&format!("/api/v1/documents/{}", doc_id))
        .send()
        .await;
    assert_eq!(delete_resp.status(), 204);
}
```

## CI/CD Integration

### GitHub Actions

```yaml
name: Tests

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
      
      - name: Cache cargo
        uses: actions/cache@v4
        with:
          path: |
            ~/.cargo/registry
            ~/.cargo/git
            target
          key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}
      
      - name: Run tests
        run: cargo test --all-features
      
      - name: Run clippy
        run: cargo clippy -- -D warnings
      
      - name: Check formatting
        run: cargo fmt --check
```

## Best Practices

### Test Naming

```rust
// Good: Descriptive names
#[test]
fn test_create_document_with_valid_data() { }

#[test]
fn test_create_document_with_empty_title_fails() { }

// Bad: Vague names
#[test]
fn test_create() { }
```

### Arrange-Act-Assert

```rust
#[test]
fn test_document_update() {
    // Arrange
    let mut doc = Document::new("Title", "Content");
    
    // Act
    doc.update_content("New Content");
    
    // Assert
    assert_eq!(doc.content, "New Content");
}
```

### Test Isolation

```rust
// Good: Each test is isolated
#[tokio::test]
async fn test_one() {
    let db = setup_test_db().await;
    // Test uses fresh database
}

#[tokio::test]
async fn test_two() {
    let db = setup_test_db().await;
    // Independent from test_one
}
```

### Error Messages

```rust
// Good: Clear assertion messages
assert_eq!(
    result.status,
    Status::Published,
    "Document should be published after approval"
);

// Bad: No context
assert_eq!(result.status, Status::Published);
```

## Debugging Tests

### Print Debugging

```bash
cargo test -- --nocapture
```

### LLDB Debugging

```bash
rust-lldb ./target/debug/deps/tachyon_core-*
(lldb) breakpoint set -n test_document_creation
(lldb) run
```

### VS Code

```json
{
    "type": "lldb",
    "request": "launch",
    "name": "Debug Test",
    "cargo": {
        "args": ["test", "--no-run", "--lib"],
        "filter": { "name": "tachyon-core" }
    }
}
```
