# Core Component

The `tachyon-core` crate provides fundamental types, traits, and error handling.

## Overview

Core is the foundation crate that all other crates depend on. It defines:

- Domain types (Document, User, etc.)
- Repository traits
- Error types
- Shared utilities

## Domain Types

### Document

```rust
pub struct Document {
    pub id: DocumentId,
    pub title: String,
    pub content: String,
    pub metadata: Metadata,
    pub status: DocumentStatus,
    pub visibility: Visibility,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub enum DocumentStatus {
    Draft,
    Published,
    Archived,
    Deleted,
}

pub enum Visibility {
    Public,
    Private,
    Restricted,
}
```

### Metadata

```rust
pub struct Metadata {
    pub author: Option<String>,
    pub tags: Vec<String>,
    pub custom: HashMap<String, Value>,
}
```

### User

```rust
pub struct User {
    pub id: UserId,
    pub username: String,
    pub email: String,
    pub role: Role,
    pub groups: Vec<GroupName>,
}
```

## Repository Trait

```rust
#[async_trait]
pub trait Repository: Send + Sync {
    async fn get(&self, id: &DocumentId) -> Result<Option<Document>>;
    async fn save(&self, document: &Document) -> Result<()>;
    async fn delete(&self, id: &DocumentId) -> Result<()>;
    async fn list(&self, filter: Filter) -> Result<Vec<Document>>;
    async fn search(&self, query: &Query) -> Result<Vec<SearchHit>>;
}
```

## Error Handling

```rust
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Document not found: {0}")]
    NotFound(String),
    
    #[error("Permission denied")]
    PermissionDenied,
    
    #[error("Validation error: {0}")]
    Validation(String),
    
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
```

## Usage Example

```rust
use tachyon_core::{Document, DocumentId, Repository};

async fn get_document<R: Repository>(
    repo: &R,
    id: &DocumentId,
) -> Result<Document> {
    repo.get(id)
        .await?
        .ok_or(Error::NotFound(id.to_string()))
}
```

## Dependencies

| Crate | Purpose |
|-------|---------|
| `serde` | Serialization |
| `chrono` | Date/time handling |
| `uuid` | Unique identifiers |
| `thiserror` | Error derive |
| `async-trait` | Async traits |
