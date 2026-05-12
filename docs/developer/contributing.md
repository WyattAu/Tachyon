# Contributing Guide

Thank you for your interest in contributing to Tachyon!

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Setup](#development-setup)
- [How to Contribute](#how-to-contribute)
- [Pull Request Process](#pull-request-process)
- [Coding Standards](#coding-standards)
- [Testing](#testing)
- [Documentation](#documentation)

## Code of Conduct

This project follows the [Rust Code of Conduct](https://www.rust-lang.org/policies/code-of-conduct). Please read it before contributing.

## Getting Started

### Prerequisites

- Rust 1.75+ (2024 edition)
- PostgreSQL 12+
- Node.js 18+ (for frontend)
- Git

### Development Setup

```bash
# Clone the repository
git clone https://github.com/tachyon-org/tachyon.git
cd tachyon

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Build the project
cargo build

# Run tests
cargo test

# Setup database
createdb tachyon
cargo run --bin tachyon-db-setup
```

## How to Contribute

### Reporting Bugs

1. Check if the bug has already been reported in [Issues](https://github.com/tachyon-org/tachyon/issues)
2. If not, create a new issue with:
   - Clear title and description
   - Steps to reproduce
   - Expected vs actual behavior
   - Environment details (OS, Rust version, etc.)
   - Logs or screenshots if applicable

### Suggesting Features

1. Check existing issues for similar suggestions
2. Create a new issue with the `enhancement` label
3. Describe the feature and use case
4. Explain why it would be useful

### Contributing Code

1. Find an issue to work on (or create one)
2. Fork the repository
3. Create a feature branch
4. Make your changes
5. Submit a pull request

## Pull Request Process

### 1. Create a Branch

```bash
git checkout -b feature/your-feature-name
# or
git checkout -b fix/issue-number
```

### 2. Make Changes

- Follow [coding standards](#coding-standards)
- Write tests for new code
- Update documentation
- Keep changes focused

### 3. Run Checks

```bash
# Format code
cargo fmt

# Run linter
cargo clippy -- -D warnings

# Run tests
cargo test

# Run all checks
cargo make check
```

### 4. Commit Changes

Write clear commit messages:

```
type(scope): short description

Longer description if needed.

Fixes #issue-number
```

Types: `feat`, `fix`, `docs`, `test`, `refactor`, `chore`

Examples:
```
feat(api): add document search endpoint
fix(db): resolve connection pool leak
docs(readme): update installation instructions
```

### 5. Push and Create PR

```bash
git push origin feature/your-feature-name
```

Then create a pull request on GitHub with:
- Clear title and description
- Reference to related issues
- List of changes
- Screenshots (if UI changes)

### 6. Code Review

- Respond to review feedback
- Make requested changes
- Keep discussion constructive

## Coding Standards

### Rust Style

Follow standard Rust conventions:

```bash
# Format code
cargo fmt

# Check style
cargo clippy
```

Key guidelines:
- Use `rustfmt` defaults
- Follow Clippy recommendations
- Write documentation comments
- Use meaningful variable names

### Code Organization

```
crates/
├── crate-name/
│   ├── src/
│   │   ├── lib.rs       # Public API
│   │   ├── module.rs    # Module implementation
│   │   └── tests/       # Integration tests
│   └── Cargo.toml
```

### Documentation

Add rustdoc comments to public APIs:

```rust
/// Creates a new document.
///
/// # Arguments
///
/// * `title` - The document title
/// * `content` - The document content
///
/// # Returns
///
/// The created document
///
/// # Errors
///
/// Returns an error if the database operation fails
///
/// # Example
///
/// ```
/// let doc = create_document("Title", "Content")?;
/// ```
pub fn create_document(title: &str, content: &str) -> Result<Document> {
    // Implementation
}
```

### Error Handling

Use `thiserror` for custom errors:

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DocumentError {
    #[error("Document not found: {0}")]
    NotFound(String),
    
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
}
```

### Testing

Write unit and integration tests:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_create_document() {
        let doc = create_document("Test", "Content");
        assert!(doc.is_ok());
    }
    
    #[tokio::test]
    async fn test_async_operation() {
        let result = async_function().await;
        assert!(result.is_ok());
    }
}
```

## Testing

### Run Tests

```bash
# All tests
cargo test

# Specific crate
cargo test --package tachyon-server

# Specific test
cargo test test_name

# With output
cargo test -- --nocapture
```

### Test Coverage

```bash
# Install tarpaulin
cargo install cargo-tarpaulin

# Run with coverage
cargo tarpaulin --out Html
```

### Integration Tests

```bash
# Run integration tests
cargo test --test '*'
```

## Documentation

### Update Documentation

When adding features:
1. Update rustdoc comments
2. Update user guide if needed
3. Update API documentation
4. Add examples

### Build Documentation

```bash
# Build and open docs
cargo doc --open

# Build with private items
cargo doc --document-private-items
```

## Release Process

Maintainers handle releases:
1. Update version in `Cargo.toml`
2. Update `CHANGELOG.md`
3. Create git tag
4. Build release binaries
5. Publish to crates.io (if applicable)

## Getting Help

- **GitHub Discussions**: For questions and discussions
- **GitHub Issues**: For bug reports and features
- **Email**: maintainers@tachyon.example.com

## License

By contributing, you agree that your contributions will be licensed under the Apache License 2.0.

## Recognition

Contributors are recognized in:
- Git history
- Release notes
- Contributors file

Thank you for contributing to Tachyon! (new)
