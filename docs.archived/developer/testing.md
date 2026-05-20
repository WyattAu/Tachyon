# Testing Guide

This guide covers testing strategies and best practices for Tachyon.

## Overview

Tachyon has comprehensive testing at multiple levels:
- Unit tests
- Integration tests
- End-to-end tests
- Performance tests

## Test Structure

```
crates/
├── crate-name/
│   ├── src/
│   │   ├── lib.rs
│   │   └── module.rs       # Inline unit tests
│   └── tests/
│       ├── integration_test.rs
│       └── common/
│           └── mod.rs      # Test utilities
```

## Running Tests

### All Tests

```bash
cargo test
```

### Specific Crate

```bash
cargo test --package tachyon-server
```

### Specific Test

```bash
cargo test test_document_creation
```

### With Output

```bash
cargo test -- --nocapture
```

### Parallel Execution

```bash
# Run tests in parallel (default)
cargo test

# Run tests sequentially
cargo test -- --test-threads=1
```

## Unit Tests

Unit tests are written inline with the code:

```rust
// src/document.rs

pub fn create_document(title: &str, content: &str) -> Result<Document> {
    // Implementation
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_create_document_success() {
        let doc = create_document("Title", "Content");
        assert!(doc.is_ok());
        let doc = doc.unwrap();
        assert_eq!(doc.title, "Title");
        assert_eq!(doc.content, "Content");
    }
    
    #[test]
    fn test_create_document_empty_title() {
        let doc = create_document("", "Content");
        assert!(doc.is_err());
    }
}
```

### Async Tests

Use `tokio::test` for async functions:

```rust
#[tokio::test]
async fn test_async_document_fetch() {
    let db = setup_test_db().await;
    let doc = fetch_document(&db, "doc-id").await;
    assert!(doc.is_ok());
}
```

### Test Helpers

Create test helper functions:

```rust
#[cfg(test)]
mod test_utils {
    use super::*;
    
    pub fn create_test_document() -> Document {
        Document {
            id: Uuid::new_v4(),
            title: "Test Document".to_string(),
            content: "Test content".to_string(),
            ..Default::default()
        }
    }
    
    pub async fn setup_test_db() -> PgPool {
        PgPoolOptions::new()
            .connect("postgres://test:test@localhost/tachyon_test")
            .await
            .unwrap()
    }
}
```

## Integration Tests

Integration tests are in the `tests/` directory:

```rust
// tests/api_test.rs

use axum_test::TestServer;
use tachyon_server::build_router;

#[tokio::test]
async fn test_create_document_api() {
    let app = build_router().await;
    let server = TestServer::new(app).unwrap();
    
    let response = server
        .post("/api/v1/documents")
        .json(&json!({
            "title": "Test",
            "content": "Content"
        }))
        .await;
    
    assert_eq!(response.status_code(), StatusCode::CREATED);
}
```

### Database Tests

```rust
// tests/db_test.rs

use sqlx::postgres::PgPoolOptions;

#[sqlx::test]
async fn test_insert_document(pool: PgPool) {
    let result = sqlx::query!(
        "INSERT INTO documents (title, content) VALUES ($1, $2)",
        "Test",
        "Content"
    )
    .execute(&pool)
    .await;
    
    assert!(result.is_ok());
}
```

## Test Fixtures

### Creating Fixtures

```rust
// tests/common/fixtures.rs

pub struct TestFixtures;

impl TestFixtures {
    pub fn user() -> User {
        User {
            id: Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap(),
            email: "test@example.com".to_string(),
            name: "Test User".to_string(),
            role: "user".to_string(),
        }
    }
    
    pub fn document() -> Document {
        Document {
            id: Uuid::parse_str("00000000-0000-0000-0000-000000000002").unwrap(),
            title: "Test Document".to_string(),
            content: "# Test\n\nContent here".to_string(),
            ..Default::default()
        }
    }
}
```

### Using Fixtures

```rust
#[test]
fn test_with_fixture() {
    let user = TestFixtures::user();
    let doc = TestFixtures::document();
    
    // Test with consistent test data
}
```

## Mocking

### Mocking External Services

Use `mockall` for mocking:

```rust
use mockall::automock;

#[automock]
pub trait EmailService {
    fn send(&self, to: &str, subject: &str, body: &str) -> Result<()>;
}

#[test]
fn test_send_notification() {
    let mut mock_email = MockEmailService::new();
    mock_email
        .expect_send()
        .times(1)
        .returning(|_, _, _| Ok(()));
    
    let service = NotificationService::new(Box::new(mock_email));
    service.notify("user@example.com", "Test").unwrap();
}
```

### Mocking Time

```rust
use tokio::time::{freeze, advance};

#[tokio::test]
async fn test_time_dependent() {
    freeze(|| {
        // Time is frozen
        let now = chrono::Utc::now();
        
        // Test time-dependent code
    });
}
```

## Test Coverage

### Generate Coverage Report

```bash
# Install tarpaulin
cargo install cargo-tarpaulin

# Generate coverage
cargo tarpaulin --out Html --output-dir coverage/

# View report
open coverage/tarpaulin-report.html
```

### Coverage Goals

- **Unit tests**: > 80% coverage
- **Critical paths**: 100% coverage
- **Error handling**: All error paths covered

## Performance Testing

### Benchmark Tests

```rust
#[bench]
fn bench_parse_markdown(b: &mut test::Bencher) {
    let markdown = load_test_markdown();
    b.iter(|| {
        parse_markdown(&markdown)
    });
}
```

### Load Testing

Use tools like `wrk` or `hey`:

```bash
# Install hey
go install github.com/rakyll/hey@latest

# Load test API
hey -n 1000 -c 100 http://localhost:8080/api/v1/documents
```

## Test Database

### Setup Test Database

```bash
# Create test database
createdb tachyon_test

# Run migrations
DATABASE_URL=postgres://localhost/tachyon_test sqlx migrate run
```

### Clean Test Database

```rust
#[cfg(test)]
mod test_db {
    use sqlx::PgPool;
    
    pub async fn clean_db(pool: &PgPool) {
        sqlx::query("TRUNCATE documents, users CASCADE")
            .execute(pool)
            .await
            .unwrap();
    }
}
```

## Continuous Integration

Tests run automatically on CI:

```yaml
# .github/workflows/test.yml
name: Tests

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    
    services:
      postgres:
        image: postgres:14
        env:
          POSTGRES_DB: tachyon_test
          POSTGRES_USER: test
          POSTGRES_PASSWORD: test
    
    steps:
      - uses: actions/checkout@v3
      
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      
      - name: Run tests
        run: cargo test --all-features
        env:
          DATABASE_URL: postgres://test:test@localhost/tachyon_test
```

## Test Best Practices

### 1. Write Clear Test Names

```rust
// Good
#[test]
fn test_create_document_with_valid_data_returns_document() {}

// Bad
#[test]
fn test_create() {}
```

### 2. Test One Thing

```rust
// Good - one assertion per test concept
#[test]
fn test_document_title() {
    let doc = create_document("Title", "");
    assert_eq!(doc.title, "Title");
}

#[test]
fn test_document_content() {
    let doc = create_document("", "Content");
    assert_eq!(doc.content, "Content");
}
```

### 3. Use Assertions Effectively

```rust
// Specific assertion
assert_eq!(result, expected);

// Boolean assertion
assert!(result.is_ok());

// With custom message
assert!(result.is_ok(), "Failed to create document");

// Assert panic
#[test]
#[should_panic(expected = "empty title")]
fn test_empty_title_panics() {
    create_document("", "content").unwrap();
}
```

### 4. Test Edge Cases

```rust
#[test]
fn test_empty_input() {}

#[test]
fn test_max_length_input() {}

#[test]
fn test_unicode_input() {}

#[test]
fn test_special_characters() {}
```

### 5. Test Error Paths

```rust
#[test]
fn test_database_connection_error() {}

#[test]
fn test_invalid_input_error() {}

#[test]
fn test_permission_denied_error() {}
```

## Debugging Tests

### Print Debug Output

```bash
cargo test -- --nocapture
```

### Run Single Test with Debug

```bash
RUST_LOG=debug cargo test test_name -- --nocapture
```

### Use Debugger

```bash
# With rust-gdb
rust-gdb --args target/debug/deps/module-test test_name

# With VS Code
# Add launch configuration for cargo test
```

## Test Organization

```
tests/
├── integration/
│   ├── api/
│   │   ├── documents_test.rs
│   │   └── users_test.rs
│   └── db/
│       └── migrations_test.rs
├── e2e/
│   └── user_journey_test.rs
└── common/
    ├── fixtures.rs
    ├── helpers.rs
    └── mod.rs
```

## Next Steps

- [Contributing Guide](contributing.md) - Contribution process
- [Architecture](architecture.md) - System design
- [API Guide](api.md) - API documentation
