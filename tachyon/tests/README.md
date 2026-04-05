# Tachyon Testing Infrastructure

This document provides comprehensive documentation for the Tachyon testing infrastructure.

## Overview

Tachyon uses a multi-layered testing approach:

1. **Unit Tests** - Test individual functions and modules in isolation
2. **Integration Tests** - Test interactions between components
3. **E2E Tests** - Test complete user workflows in a browser
4. **Performance Tests** - Benchmark critical paths
5. **Security Tests** - Audit dependencies and code

## Test Structure

```
tachyon/
├── crates/
│   ├── database/tests/          # Database layer tests
│   │   ├── document_test.rs     # Document CRUD tests
│   │   ├── catalog_test.rs      # Catalog query tests
│   │   ├── user_test.rs         # User authentication tests
│   │   └── websocket_test.rs    # WebSocket connection tests
│   ├── server/tests/            # Server tests
│   │   ├── api_test.rs          # API endpoint tests
│   │   ├── auth_test.rs         # Authentication flow tests
│   │   └── search_test.rs       # Search functionality tests
│   ├── frontend/tests/          # Frontend tests
│   │   ├── component_tests.rs   # Component rendering tests
│   │   └── api_tests.rs         # API client tests
│   └── testing/                 # Testing utilities
│       ├── src/common/          # Shared test utilities
│       ├── src/unit/            # Unit test suites
│       ├── src/integration/     # Integration test suites
│       └── src/benchmarks/      # Performance benchmarks
└── tests/
    ├── e2e/                     # End-to-end tests
    │   ├── playwright.config.ts # Playwright configuration
    │   ├── auth.spec.ts         # Authentication E2E
    │   ├── documents.spec.ts    # Document CRUD E2E
    │   └── search.spec.ts       # Search E2E
    ├── coverage/                # Coverage configuration
    └── run_tests.sh             # Test runner script
```

## Running Tests

### Quick Start

```bash
# Run all backend tests
cargo test

# Run specific test file
cargo test --test document_test

# Run tests with specific pattern
cargo test document_crud
```

### Using the Test Runner

```bash
# Run all tests (unit + integration)
./tests/run_tests.sh

# Run only unit tests
./tests/run_tests.sh --unit-only

# Run only integration tests
./tests/run_tests.sh --integration-only

# Run E2E tests
./tests/run_tests.sh --e2e-only

# Run with coverage
./tests/run_tests.sh --coverage

# Run everything including E2E
./tests/run_tests.sh --all
```

### Frontend Tests

```bash
# Install wasm-pack
cargo install wasm-pack

# Run frontend tests
cd tachyon/crates/frontend
wasm-pack test --headless --chrome
wasm-pack test --headless --firefox
```

### E2E Tests

```bash
# Install dependencies
cd tachyon/tests/e2e
npm install

# Install Playwright browsers
npx playwright install

# Run tests
npm test

# Run in headed mode
npm run test:headed

# Run specific browser
npm run test:chromium
```

## Test Database

Tests require a PostgreSQL database. Setup:

```bash
# Create test database
createdb tachyon_test

# Or using Docker
docker run -d \
  --name tachyon-test-db \
  -e POSTGRES_USER=tachyon \
  -e POSTGRES_PASSWORD=tachyon \
  -e POSTGRES_DB=tachyon_test \
  -p 5432:5432 \
  postgres:16

# Set environment variable
export TEST_DATABASE_URL="postgres://tachyon:tachyon@localhost:5432/tachyon_test"
```

## Writing Tests

### Backend Unit Tests

```rust
#[tokio::test]
async fn test_create_document() {
    let pool = setup_test_db().await;
    let repo = DocumentRepository::new(pool);
    
    let doc = TestDataFactory::create_document();
    let result = repo.create_document(&doc).await;
    
    assert!(result.is_ok());
}
```

### Integration Tests

```rust
#[tokio::test]
async fn test_api_endpoint() {
    let app = create_test_app().await;
    
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/documents")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    
    assert_eq!(response.status(), StatusCode::OK);
}
```

### Frontend Tests

```rust
#[wasm_bindgen_test]
fn test_component_renders() {
    mount_to_body(|| view! { <MyComponent/> });
    
    let document = web_sys::window()
        .expect("no window")
        .document()
        .expect("no document");
    
    let element = document.query_selector(".my-component")
        .expect("Failed to query")
        .expect("Element not found");
    
    assert!(element.inner_html().len() > 0);
}
```

### E2E Tests

```typescript
test('should create document', async ({ page }) => {
    await page.goto('/documents');
    await page.click('button:has-text("New")');
    await page.fill('input[name="title"]', 'Test Document');
    await page.click('button[type="submit"]');
    
    await expect(page.locator('h1')).toContainText('Test Document');
});
```

## Test Utilities

### TestDataFactory

Creates test data with sensible defaults:

```rust
let doc = TestDataFactory::create_document();
let project = TestDataFactory::create_project();
let session = TestDataFactory::create_session();
```

### TestFixtures

Creates multiple test entities:

```rust
let docs = TestFixtures::create_test_documents(&repo, 10).await;
```

### TestDatabase

Manages test database lifecycle:

```rust
with_test_db(|db| async move {
    // Tests run here
}).await;
```

### Custom Assertions

```rust
assert_ok!(result);
assert_err!(result);
```

## Coverage

### Generate Coverage Report

```bash
# Install tarpaulin
cargo install cargo-tarpaulin

# Generate report
cargo tarpaulin --out Html --out Xml

# View report
open tarpaulin-report.html
```

### Coverage Targets

| Module | Target |
|--------|--------|
| tachyon-core | 70% |
| tachyon-database | 65% |
| tachyon-server | 60% |
| tachyon-rbac | 65% |
| tachyon-search | 60% |
| **Overall** | **60%** |

## CI/CD

Tests run automatically on:
- Push to main/develop branches
- Pull requests

GitHub Actions workflow: `.github/workflows/test.yml`

### CI Pipeline

1. **Lint & Format Check**
   ```bash
   cargo fmt --check
   cargo clippy
   ```

2. **Unit Tests**
   ```bash
   cargo test --lib
   ```

3. **Integration Tests**
   ```bash
   cargo test --test '*'
   ```

4. **Frontend Tests**
   ```bash
   wasm-pack test --headless
   ```

5. **E2E Tests**
   ```bash
   npm test
   ```

6. **Coverage Report**
   ```bash
   cargo tarpaulin --out Xml
   ```

## Best Practices

1. **Write Tests First**: Follow TDD when possible
2. **Test Edge Cases**: Don't just test happy paths
3. **Use Descriptive Names**: Test names should describe behavior
4. **Keep Tests Isolated**: Each test should be independent
5. **Clean Up**: Always clean up test data
6. **Use Mocks**: Mock external dependencies
7. **Document Tests**: Add comments for complex test logic

## Troubleshooting

### Database Connection Issues

```bash
# Check PostgreSQL is running
psql -U tachyon -d tachyon_test -c "SELECT 1"

# Check environment variable
echo $TEST_DATABASE_URL
```

### Frontend Test Issues

```bash
# Clear WASM cache
rm -rf target/wasm32-unknown-unknown

# Rebuild
cargo clean
cargo build --target wasm32-unknown-unknown
```

### E2E Test Issues

```bash
# Clear Playwright cache
npx playwright uninstall
npx playwright install

# Run in debug mode
npx playwright test --debug
```

## Resources

- [Rust Testing Book](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [Tokio Testing](https://docs.rs/tokio-test)
- [Playwright Docs](https://playwright.dev)
- [WASM Pack](https://rustwasm.github.io/wasm-pack/)
