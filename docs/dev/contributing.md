# Contributing Guide

Guidelines for contributing to Tachyon.

## Code of Conduct

- Be respectful and inclusive
- Welcome newcomers
- Accept constructive criticism
- Focus on what's best for the project

## Getting Started

1. Fork the repository
2. Clone your fork
3. Set up development environment ([Setup Guide](setup.md))
4. Create a feature branch

```bash
git checkout -b feature/my-feature
```

## Development Process

### 1. Create an Issue

Before starting work:
- Check existing issues
- Open a new issue describing the change
- Wait for maintainer feedback on large changes

### 2. Make Changes

```bash
# Create branch
git checkout -b feature/my-feature

# Make changes
# ...

# Run tests
cargo test

# Check formatting
cargo fmt --check

# Run linter
cargo clippy
```

### 3. Commit Changes

Write clear commit messages:

```
type(scope): brief description

Detailed explanation if needed.

- Bullet points for multiple changes
- Reference issues: Fixes #123
```

Types:
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation
- `refactor`: Code refactoring
- `test`: Tests
- `chore`: Maintenance

### 4. Push and Create PR

```bash
git push origin feature/my-feature
```

Create a Pull Request with:
- Description of changes
- Link to related issues
- Test coverage notes
- Breaking changes (if any)

## Code Standards

### Rust Style

Follow standard Rust conventions:

```rust
// Good: Descriptive names
fn calculate_document_hash(content: &str) -> String {
    // ...
}

// Bad: Cryptic names
fn calc(c: &str) -> String {
    // ...
}
```

```rust
// Good: Error handling with thiserror
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Document not found: {0}")]
    NotFound(String),
    #[error("Permission denied")]
    PermissionDenied,
}
```

```rust
// Good: Use Result instead of unwrap
pub fn load_document(path: &Path) -> Result<Document> {
    let content = fs::read_to_string(path)?;
    Document::parse(&content)
}
```

### Documentation

All public items must be documented:

```rust
/// Parses markdown content into a document.
///
/// # Arguments
///
/// * `content` - The markdown content to parse
///
/// # Returns
///
/// A parsed document with extracted metadata
///
/// # Errors
///
/// Returns an error if the content is invalid UTF-8
///
/// # Example
///
/// ```
/// let doc = parse_markdown("# Hello")?;
/// assert_eq!(doc.title, "Hello");
/// ```
pub fn parse_markdown(content: &str) -> Result<Document> {
    // ...
}
```

### Testing

Write tests for new functionality:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_document_creation() {
        let doc = Document::new("Test", "Content");
        assert_eq!(doc.title, "Test");
        assert_eq!(doc.content, "Content");
    }

    #[test]
    fn test_invalid_input() {
        let result = Document::new("", "");
        assert!(result.is_err());
    }
}
```

## Pull Request Guidelines

### PR Checklist

- [ ] Code compiles without warnings
- [ ] All tests pass
- [ ] New code is tested
- [ ] Documentation updated
- [ ] CHANGELOG.md updated
- [ ] Commit messages are clear

### PR Template

```markdown
## Description

Brief description of changes.

## Type of Change

- [ ] Bug fix
- [ ] New feature
- [ ] Breaking change
- [ ] Documentation update

## Testing

Describe testing performed.

## Checklist

- [ ] Tests pass
- [ ] Code formatted
- [ ] Linter passes
- [ ] Documentation updated
```

### Review Process

1. Automated checks run (CI)
2. At least one maintainer review required
3. Address review feedback
4. Maintainer approves and merges

## Project Structure

When adding new features:

### New Feature

1. Add types to `tachyon-core`
2. Implement in appropriate crate
3. Add tests in crate's `tests/`
4. Update documentation

### New API Endpoint

1. Add route in `tachyon-server/src/routes/`
2. Add handler with error handling
3. Add request/response types
4. Add API tests
5. Update API documentation

### New Frontend Component

1. Create component in `tachyon-frontend/src/components/`
2. Add styles (Tailwind)
3. Add tests if complex
4. Update storybook (if applicable)

## Commit Guidelines

### Atomic Commits

Each commit should be a single, focused change:

```bash
# Good: One logical change per commit
git commit -m "feat(search): add fuzzy matching support"
git commit -m "docs(search): document fuzzy matching options"

# Bad: Multiple unrelated changes
git commit -m "add fuzzy search and fix typo and update deps"
```

### Commit Message Format

```
<type>(<scope>): <description>

[optional body]

[optional footer]
```

Scopes:
- `core`: Core types and traits
- `server`: Server functionality
- `desktop`: Desktop application
- `frontend`: Web frontend
- `search`: Search functionality
- `docs`: Documentation

## Testing Requirements

### Unit Tests

Required for:
- Business logic
- Data transformations
- Error handling

### Integration Tests

Required for:
- API endpoints
- Database operations
- File operations

### Test Coverage

Aim for >80% coverage on new code.

```bash
# Generate coverage report
cargo tarpaulin --out Html
```

## Documentation Requirements

### Code Documentation

- All public items documented
- Examples for complex functions
- Error conditions documented

### User Documentation

Update for:
- New features
- Changed behavior
- Breaking changes

### API Documentation

Update OpenAPI specs for:
- New endpoints
- Changed parameters
- Changed responses

## Release Process

Maintainers handle releases:

1. Update VERSION.md
2. Update CHANGELOG.md
3. Create release PR
4. Merge and tag release
5. CI publishes artifacts

## Getting Help

- GitHub Discussions: Questions and ideas
- GitHub Issues: Bug reports
- Code comments: Implementation questions

## Recognition

Contributors are recognized in:
- CHANGELOG.md
- GitHub contributors page
- Release notes

Thank you for contributing!
