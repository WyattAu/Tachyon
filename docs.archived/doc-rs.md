# Building Documentation (docs.rs)

This guide covers building and accessing Tachyon's rustdoc documentation.

## Building Documentation Locally

### Build and Open

```bash
cargo doc --open
```

This builds documentation for all crates and opens it in your browser.

### Build Specific Crate

```bash
cargo doc --package tachyon-server --open
```

### Build with Private Items

```bash
cargo doc --document-private-items --open
```

### Build without Dependencies

```bash
cargo doc --no-deps --open
```

## Documentation Structure

```
target/doc/
├── tachyon_server/
│   ├── index.html
│   ├── config/
│   ├── routes/
│   └── ...
├── tachyon_database/
├── tachyon_renderer/
└── ...
```

## Writing Documentation

### Module Documentation

```rust
//! Tachyon Server
//!
//! High-performance HTTP/2 server for knowledge management.
//!
//! # Features
//!
//! - REST API with OpenAPI documentation
//! - WebSocket support for real-time collaboration
//! - JWT and API key authentication
//!
//! # Example
//!
//! ```rust,no_run
//! use tachyon_server::config::ServerConfig;
//!
//! let config = ServerConfig::default();
//! ```

pub mod config;
pub mod routes;
pub mod websocket;
```

### Function Documentation

```rust
/// Creates a new document in the specified project.
///
/// # Arguments
///
/// * `project_id` - The UUID of the project
/// * `title` - The document title
/// * `content` - The document content in markdown
///
/// # Returns
///
/// Returns the created `Document` on success.
///
/// # Errors
///
/// Returns an error if:
/// - The project doesn't exist
/// - The user lacks permission
/// - The database operation fails
///
/// # Example
///
/// ```rust
/// use tachyon_server::documents;
///
/// let doc = documents::create(
///     "project-uuid",
///     "My Document",
///     "# Content"
/// )?;
/// ```
pub fn create(project_id: &str, title: &str, content: &str) -> Result<Document> {
    // Implementation
}
```

### Struct Documentation

```rust
/// Server configuration.
///
/// Contains all settings for the HTTP server, including:
/// - Network configuration (host, port)
/// - Database connection
/// - JWT settings
/// - CORS and security options
///
/// # Example
///
/// ```rust
/// use tachyon_server::config::ServerConfig;
///
/// let config = ServerConfig {
///     host: "0.0.0.0".to_string(),
///     port: 8080,
///     ..Default::default()
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    /// Server bind address
    pub host: String,
    /// Server port
    pub port: u16,
    // ...
}
```

## Documentation Tests

Code blocks in documentation are tested:

```rust
/// Adds two numbers.
///
/// # Example
///
/// ```
/// use mylib::add;
/// assert_eq!(add(2, 2), 4);
/// ```
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}
```

Run doc tests:

```bash
cargo test --doc
```

## Linking in Documentation

### Intra-doc Links

```rust
/// Creates a [`Document`] in the project.
///
/// See [`ServerConfig`] for configuration options.
pub fn create() -> Document { }
```

### External Crates

```rust
/// Uses [`serde_json::Value`] for JSON data.
pub fn parse(value: serde_json::Value) { }
```

## Documentation Linting

Check for documentation issues:

```bash
RUSTDOCFLAGS="-D warnings" cargo doc
```

## docs.rs

When published to crates.io, documentation is automatically built and hosted on docs.rs.

### Configuration

Add to `Cargo.toml`:

```toml
[package.metadata.docs.rs]
all-features = true
rustdoc-args = ["--cfg", "docsrs"]
```

### Accessing Published Docs

Once published, documentation is available at:

```
https://docs.rs/tachyon-server
```

## Best Practices

1. **Document all public APIs** with examples
2. **Use intra-doc links** for cross-references
3. **Include error conditions** in documentation
4. **Write runnable examples** in code blocks
5. **Keep docs up-to-date** with code changes
6. **Use markdown formatting** for readability

## Generating HTML

The generated HTML includes:
- Module structure
- Type signatures
- Function signatures
- Documentation comments
- Source code links
- Search functionality

## See Also

- [rustdoc Book](https://doc.rust-lang.org/rustdoc/)
- [API Guidelines](https://rust-lang.github.io/api-guidelines/)
- [Developer Guide](../developer/README.md)
