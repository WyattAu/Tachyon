# TACHYON: DATA MODEL DOCUMENTATION

**Document ID:** TACHYON-API-009-V1.0
**Date:** February 2026
**Status:** Proposed
**Classification:** API Documentation
**Compliance Level:** ISO/IEC 26514:2021, IEEE 1058-2009
**Dependencies:**
- [TACHYON-STD-V1.0](../../.specs/01_standards/coding_standards.md) - Coding and Documentation Standards
- [TACHYON-DES-DM-V1.0](../../.specs/04_future_state/design/data_models.md) - Data Models Design
- [TACHYON-ADR-001-V1.0](../../.specs/02_adrs/001_rust_as_primary_language.md) - Rust as Primary Language
- [TACHYON-ADR-010-V1.0](../../.specs/02_adrs/010_security_architecture.md) - Security Architecture

---

## TABLE OF CONTENTS

1. [Introduction](#1-introduction)
2. [Data Model Framework](#2-data-model-framework)
3. [Core Entities](#3-core-entities)
4. [User Entities](#4-user-entities)
5. [Git Entities](#5-git-entities)
6. [Plugin Entities](#6-plugin-entities)
7. [Configuration Entities](#7-configuration-entities)
8. [Event Entities](#8-event-entities)
9. [Relationships](#9-relationships)
10. [Validation Rules](#10-validation-rules)
11. [Serialization](#11-serialization)
12. [Migration Strategy](#12-migration-strategy)
13. [References](#13-references)

---

## 1. INTRODUCTION

### 1.1. Document Purpose

This document provides comprehensive API-level documentation for all data models used within the Tachyon toolchain. It serves as the authoritative reference for data structures, their semantics, constraints, and serialization formats across desktop, server, and web components.

The data models defined herein are implemented in Rust Edition 2024, leveraging the type system for compile-time guarantees of correctness, memory safety, and data integrity. All models support serialization through Serde, enabling seamless interoperability between components via JSON and MessagePack formats.

### 1.2. Scope

This document covers the complete data model taxonomy for the Tachyon system:

- **Core Entities:** Document identifiers, repository paths, content hashes
- **User Entities:** User accounts, sessions, authentication tokens
- **Git Entities:** Repositories, commits, branches, references
- **Plugin Entities:** Plugin definitions, capabilities, configurations
- **Configuration Entities:** System settings, user preferences, workspace configs
- **Event Entities:** System events, subscriptions, notifications

### 1.3. Design Principles

The data models adhere to the following architectural principles derived from [ADR-001](../../.specs/02_adrs/001_rust_as_primary_language.md) and [ADR-010](../../.specs/02_adrs/010_security_architecture.md):

#### 1.3.1. Type Safety

All data structures leverage Rust's type system to enforce invariants at compile time:

- **Ownership:** Each value has a single owner determining its lifetime
- **Borrowing:** References respect borrowing rules (multiple immutable OR one mutable)
- **Lifetimes:** References are guaranteed valid for declared lifetimes
- **Sum Types:** Enum types express exhaustive state machines

#### 1.3.2. Immutability

Data structures prefer immutable semantics where possible:

- **Struct Fields:** Default to immutable, explicit `mut` where mutation required
- **Copy Semantics:** Small types implement `Copy` for value semantics
- **Clone Semantics:** Larger types implement `Clone` for explicit duplication
- **Interior Mutability:** Used selectively with `Cell<T>` and `RefCell<T>`

#### 1.3.3. Zero-Copy

Data transfer between components minimizes copying:

- **Borrowing:** References avoid copying when ownership transfer not required
- **Slices:** `&[T]` enables view into contiguous data without copying
- **Cow Types:** `Cow<'a, T>` provides copy-on-write for flexible ownership
- **Bytes:** `&[u8]` and `Bytes` types for binary data transfer

#### 1.3.4. Serde Compatibility

All models support serialization/deserialization:

- **Derive Macros:** `#[derive(Serialize, Deserialize)]` on all public types
- **Custom Serialization:** `#[serde(with = "...")]` for specialized formats
- **Transparent Wrappers:** `#[serde(transparent)]` for newtype patterns
- **Renaming:** `#[serde(rename = "...")]` for API compatibility

#### 1.3.5. Validation

Built-in validation constraints ensure data integrity:

- **Type Constraints:** Compile-time type checking prevents invalid states
- **Range Constraints:** `#[serde(with = "serde_with::rust::display_fromstr")]` for validation
- **Custom Validators:** `#[validate]` attribute for runtime validation
- **Invariants:** Encapsulation ensures invariants maintained

### 1.4. Terminology

| Term | Definition |
|-------|------------|
| **Entity** | A distinct data object with identity and lifecycle |
| **Value Object** | An immutable object defined by its attributes, lacking identity |
| **Aggregate** | A cluster of domain objects treated as a unit |
| **Repository** | Collection-like interface for domain object persistence |
| **Event** | Something that happened in the domain, immutable by nature |
| **Command** | Intent to change state in the domain |

---

## 2. DATA MODEL FRAMEWORK

### 2.1. Type System Foundation

The Tachyon data model framework is built upon Rust's type system, providing:

- **Memory Safety:** Compile-time prevention of memory corruption vulnerabilities
- **Thread Safety:** `Send` and `Sync` traits enable safe concurrent access
- **Zero-Cost Abstractions:** High-level constructs compile to efficient machine code
- **Pattern Matching:** Exhaustive matching ensures all cases handled

### 2.2. Common Type Patterns

#### 2.2.1. Newtype Pattern

The newtype pattern provides type safety for primitive values:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct DocumentId(Uuid);
```

**Benefits:**
- Prevents confusion between semantically distinct values
- Enables domain-specific methods on the wrapper type
- Maintains serialization compatibility with inner type
- Zero runtime overhead due to transparent serialization

#### 2.2.2. Enum State Machines

Enums model state transitions with exhaustive matching:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DocumentStatus {
    Draft,
    Published,
    Archived { archived_at: DateTime<Utc> },
}
```

**Benefits:**
- Compile-time guarantee that all states are handled
- Impossible to represent invalid state combinations
- Self-documenting through variant names
- Pattern matching enables exhaustive case analysis

#### 2.2.3. Result Types

Error handling uses `Result<T, E>` for explicit error propagation:

```rust
pub fn parse_document_id(s: &str) -> Result<DocumentId, ParseError> {
    // ...
}
```

**Benefits:**
- Explicit error handling at call sites
- Compiler enforces error handling
- No silent failures or exceptions
- Error types are part of function signature

### 2.3. Trait-Based Abstractions

Common behaviors are defined through traits:

```rust
pub trait Entity {
    fn id(&self) -> &str;
    fn created_at(&self) -> DateTime<Utc>;
}

pub trait Validatable {
    fn validate(&self) -> Result<(), ValidationError>;
}
```

**Benefits:**
- Polymorphic behavior across different types
- Compile-time dispatch (zero-cost)
- Clear contracts through trait definitions
- Enables generic programming

### 2.4. Error Types

All error types follow the `thiserror` pattern:

```rust
#[derive(Error, Debug)]
pub enum DataModelError {
    #[error("Invalid document ID: {0}")]
    InvalidDocumentId(String),

    #[error("Validation failed: {0}")]
    ValidationError(String),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
}
```

**Benefits:**
- Structured error information
- Automatic `Display` implementation
- Error source tracking via `#[from]`
- Context preservation through error chain

### 2.5. Concurrency Primitives

Thread-safe types enable concurrent access:

| Type | Use Case | Thread Safety |
|------|-----------|---------------|
| `Arc<T>` | Shared ownership across threads | `Send + Sync` if `T: Send + Sync` |
| `Mutex<T>` | Exclusive mutable access | `Sync` if `T: Send` |
| `RwLock<T>` | Multiple readers, one writer | `Sync` if `T: Send + Sync` |
| `Atomic*` | Lock-free primitive operations | `Send + Sync` |

### 2.6. Timestamp Handling

All timestamps use `chrono::DateTime<Utc>`:

```rust
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Timestamped {
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

**Rationale:**
- UTC timezone eliminates timezone confusion
- RFC 3339 serialization format
- Arithmetic operations on timestamps
- Timezone conversion when needed

### 2.7. Identifier Generation

Identifiers use UUID v4 for uniqueness:

```rust
use uuid::Uuid;

pub fn generate_id() -> Uuid {
    Uuid::new_v4()
}
```

**Properties:**
- 122-bit random number
- Collision probability: 1 in 5.3×10^36
- No coordination required for generation
- Globally unique across all systems

---

## 3. CORE ENTITIES

### 3.1. Document ID

**Element ID:** API-DM-001
**Name:** DocumentId
**Type:** Newtype Struct
**Language:** Rust
**Module:** `tachyon_core::types`

**Description:** Unique identifier for documents within the Tachyon system. Uses UUID v4 for collision resistance and global uniqueness without coordination.

**Rust Definition:**

```rust
use uuid::Uuid;
use std::fmt::{Display, Formatter, Result as FmtResult};
use serde::{Serialize, Deserialize};

/// Unique identifier for documents
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct DocumentId(Uuid);

impl DocumentId {
    /// Creates a new random DocumentId
    ///
    /// # Returns
    /// A new DocumentId with a randomly generated UUID v4
    #[must_use]
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Creates a DocumentId from a UUID string
    ///
    /// # Parameters
    /// - `s`: UUID string in hyphenated or non-hyphenated format
    ///
    /// # Returns
    /// - `Ok(DocumentId)` if the string is a valid UUID
    /// - `Err(ParseError)` if the string is not a valid UUID
    ///
    /// # Errors
    /// - `ParseError::InvalidFormat`: String is not valid UUID format
    /// - `ParseError::InvalidVersion`: UUID is not version 4
    pub fn from_str(s: &str) -> Result<Self, ParseError> {
        let uuid = Uuid::parse_str(s)
            .map_err(|_| ParseError::InvalidFormat(s.to_string()))?;
        if uuid.get_version() != Some(uuid::Version::Random) {
            return Err(ParseError::InvalidVersion);
        }
        Ok(Self(uuid))
    }

    /// Returns the inner UUID
    #[must_use]
    pub fn inner(&self) -> Uuid {
        self.0
    }

    /// Returns the DocumentId as a hyphenated string
    #[must_use]
    pub fn as_str(&self) -> &str {
        // Note: This is a simplified implementation
        // Actual implementation would use Uuid::as_hyphenated()
        ""
    }
}

impl Display for DocumentId {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}", self.0)
    }
}

impl Default for DocumentId {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseError {
    InvalidFormat(String),
    InvalidVersion,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::InvalidFormat(s) => write!(f, "Invalid UUID format: {}", s),
            ParseError::InvalidVersion => write!(f, "UUID must be version 4"),
        }
    }
}

impl std::error::Error for ParseError {}
```

**Constraints:**
- Must be a valid UUID v4
- String representation must be lowercase
- Cannot be nil/null (all-zero UUID reserved for special cases)

**Serialization:**
- **JSON:** String representation in hyphenated format
- **MessagePack:** 16-byte binary UUID
- **URL-Safe:** Base64url-encoded string

**Security Considerations:**
- UUIDs are not guessable, preventing enumeration attacks
- No sensitive information encoded in the identifier
- Safe to expose in URLs, logs, and API responses

**Related Requirements:**
- REQ-SYS-031: JIT Rendering Pipeline
- REQ-SYS-041: Search Indexing

**Related Design Elements:**
- DES-DM-001: Document ID Design

---

### 3.2. Workspace ID

**Element ID:** API-DM-002
**Name:** WorkspaceId
**Type:** Newtype Struct
**Language:** Rust
**Module:** `tachyon_core::types`

**Description:** Unique identifier for workspaces (repositories) within the Tachyon system. Uses UUID v4 for global uniqueness.

**Rust Definition:**

```rust
use uuid::Uuid;
use serde::{Serialize, Deserialize};

/// Unique identifier for workspaces
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct WorkspaceId(Uuid);

impl WorkspaceId {
    /// Creates a new random WorkspaceId
    #[must_use]
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Creates a WorkspaceId from a UUID string
    pub fn from_str(s: &str) -> Result<Self, ParseError> {
        let uuid = Uuid::parse_str(s)
            .map_err(|_| ParseError::InvalidFormat(s.to_string()))?;
        if uuid.get_version() != Some(uuid::Version::Random) {
            return Err(ParseError::InvalidVersion);
        }
        Ok(Self(uuid))
    }

    /// Returns the inner UUID
    #[must_use]
    pub fn inner(&self) -> Uuid {
        self.0
    }
}

impl std::fmt::Display for WorkspaceId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Default for WorkspaceId {
    fn default() -> Self {
        Self::new()
    }
}
```

**Constraints:**
- Must be a valid UUID v4
- Cannot be nil/null
- Unique across all workspaces in the system

**Security Considerations:**
- Workspace isolation enforced through ID scoping
- No cross-workspace data leakage through ID guessing
- Access control checks required before workspace access

**Related Requirements:**
- REQ-DESK-031: File Watching
- REQ-DESK-037: Repository Cloning

---

### 3.3. Content Hash

**Element ID:** API-DM-003
**Name:** ContentHash
**Type:** Newtype Struct
**Language:** Rust
**Module:** `tachyon_core::crypto`

**Description:** Cryptographic hash of content for integrity verification and cache invalidation. Uses SHA-256 for collision resistance.

**Rust Definition:**

```rust
use sha2::{Sha256, Digest};
use serde::{Serialize, Deserialize};
use std::fmt::{Display, Formatter, Result as FmtResult};
use hex::{FromHex, ToHex};

/// SHA-256 content hash
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ContentHash([u8; 32]);

impl ContentHash {
    /// Computes SHA-256 hash of byte slice
    ///
    /// # Parameters
    /// - `data`: Content to hash
    ///
    /// # Returns
    /// ContentHash representing the SHA-256 digest
    #[must_use]
    pub fn compute(data: &[u8]) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(data);
        let result = hasher.finalize();
        Self(result.into())
    }

    /// Creates from hex string
    ///
    /// # Parameters
    /// - `hex`: 64-character hex string
    ///
    /// # Returns
    /// - `Ok(ContentHash)` if hex is valid
    /// - `Err(HexParseError)` if hex is invalid
    pub fn from_hex(hex: &str) -> Result<Self, HexParseError> {
        let bytes = <[u8; 32]>::from_hex(hex)
            .map_err(HexParseError::InvalidHex)?;
        Ok(Self(bytes))
    }

    /// Returns hex string representation
    #[must_use]
    pub fn to_hex(&self) -> String {
        self.0.encode_hex::<String>()
    }

    /// Verifies hash against data
    ///
    /// # Parameters
    /// - `data`: Content to verify
    ///
    /// # Returns
    /// `true` if hash matches, `false` otherwise
    #[must_use]
    pub fn verify(&self, data: &[u8]) -> bool {
        Self::compute(data) == *self
    }

    /// Returns the inner bytes
    #[must_use]
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

impl Display for ContentHash {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}", self.to_hex())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HexParseError {
    InvalidHex,
    InvalidLength,
}

impl std::fmt::Display for HexParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HexParseError::InvalidHex => write!(f, "Invalid hex string"),
            HexParseError::InvalidLength => write!(f, "Hex string must be 64 characters"),
        }
    }
}

impl std::error::Error for HexParseError {}
```

**Constraints:**
- Must be exactly 32 bytes (256 bits)
- Hex representation must be exactly 64 characters
- Cannot be all zeros (reserved for empty content)

**Serialization:**
- **JSON:** 64-character hex string (lowercase)
- **MessagePack:** 32-byte binary array
- **URL-Safe:** 64-character hex string

**Security Considerations:**
- SHA-256 is cryptographically secure
- Hashes are one-way functions, preventing content reconstruction
- Collision resistance ensures integrity guarantees
- Timing-safe comparison for security-sensitive operations

**Related Requirements:**
- REQ-SYS-058: Data Integrity
- REQ-DESK-042: Cache Invalidation

**Related Design Elements:**
- DES-DM-003: Content Hash Design

---

### 3.4. Document Metadata

**Element ID:** API-DM-004
**Name:** DocumentMetadata
**Type:** Struct
**Language:** Rust
**Module:** `tachyon_core::document`

**Description:** Metadata associated with a document, extracted from frontmatter and file system attributes.

**Rust Definition:**

```rust
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

/// Document metadata
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DocumentMetadata {
    /// Unique document identifier
    pub id: DocumentId,

    /// Document title (from frontmatter or filename)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    /// File path relative to repository root
    pub path: String,

    /// Content MIME type
    pub content_type: String,

    /// Document size in bytes
    pub size: u64,

    /// Creation timestamp
    pub created_at: DateTime<Utc>,

    /// Last modified timestamp
    pub modified_at: DateTime<Utc>,

    /// Author information
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<Author>,

    /// Document tags
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,

    /// Access control directives
    #[serde(skip_serializing_if = "Option::is_none")]
    pub access: Option<AccessControl>,

    /// Frontmatter metadata (key-value pairs)
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub frontmatter: HashMap<String, serde_json::Value>,
}

/// Author information
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Author {
    /// Author name
    pub name: String,

    /// Author email
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
}

/// Access control directives
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AccessControl {
    /// Roles with access
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub roles: Vec<String>,

    /// Users with access
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub users: Vec<String>,

    /// Internal-only flag
    #[serde(default)]
    pub internal_only: bool,
}
```

**Constraints:**
- `title`: Max 255 characters if present
- `path`: Valid relative path, max 1024 characters
- `content_type`: Valid MIME type string
- `tags`: Max 50 tags per document, max 64 characters per tag
- `size`: Non-negative, max 100MB (104,857,600 bytes)

**Validation:**
```rust
impl DocumentMetadata {
    /// Validates the metadata
    ///
    /// # Returns
    /// - `Ok(())` if metadata is valid
    /// - `Err(ValidationError)` if metadata is invalid
    pub fn validate(&self) -> Result<(), ValidationError> {
        if let Some(title) = &self.title {
            if title.len() > 255 {
                return Err(ValidationError::TitleTooLong);
            }
        }
        if self.path.len() > 1024 {
            return Err(ValidationError::PathTooLong);
        }
        if self.tags.len() > 50 {
            return Err(ValidationError::TooManyTags);
        }
        for tag in &self.tags {
            if tag.len() > 64 {
                return Err(ValidationError::TagTooLong);
            }
        }
        if self.size > 104_857_600 {
            return Err(ValidationError::DocumentTooLarge);
        }
        Ok(())
    }
}
```

**Security Considerations:**
- Access control fields enable RBAC enforcement
- Author information supports audit trails
- Tags may contain sensitive information, require access control
- Path traversal prevention through validation

**Related Requirements:**
- REQ-SYS-035: Frontmatter Processing
- REQ-SRV-043: Frontmatter Processing

**Related Design Elements:**
- DES-DM-004: Document Metadata Design

---

### 3.5. Document Content

**Element ID:** API-DM-005
**Name:** DocumentContent
**Type:** Struct
**Language:** Rust
**Module:** `tachyon_core::document`

**Description:** Complete document content including raw Markdown, rendered HTML, and derived data.

**Rust Definition:**

```rust
use serde::{Serialize, Deserialize};

/// Document content
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DocumentContent {
    /// Unique document identifier
    pub id: DocumentId,

    /// Raw Markdown content
    pub raw: String,

    /// Rendered HTML content
    pub html: String,

    /// Content hash for integrity verification
    pub hash: ContentHash,

    /// Table of contents
    #[serde(skip_serializing_if = "Option::is_none")]
    pub toc: Option<TableOfContents>,

    /// Extracted code blocks
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub code_blocks: Vec<CodeBlock>,

    /// Extracted images
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub images: Vec<ImageReference>,

    /// Internal links
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub internal_links: Vec<String>,

    /// External links
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub external_links: Vec<String>,
}

/// Table of contents
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TableOfContents {
    /// TOC entries
    pub entries: Vec<TocEntry>,
}

/// Table of contents entry
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TocEntry {
    /// Heading level (1-6)
    pub level: u8,

    /// Heading title
    pub title: String,

    /// Anchor ID
    pub anchor: String,

    /// Child entries
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub children: Vec<TocEntry>,
}

/// Code block
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CodeBlock {
    /// Programming language
    pub language: String,

    /// Code content
    pub code: String,

    /// Start line number
    pub start_line: usize,

    /// End line number
    pub end_line: usize,
}

/// Image reference
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImageReference {
    /// Image source path or URL
    pub src: String,

    /// Alt text
    pub alt: String,

    /// Width in pixels
    #[serde(skip_serializing_if = "Option::is_none")]
    pub width: Option<u32>,

    /// Height in pixels
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height: Option<u32>,
}
```

**Constraints:**
- `raw`: Max 100MB size
- `html`: Generated from raw, max 200MB size
- `toc`: Max nesting depth of 6 levels
- `code_blocks`: Max 1000 code blocks per document
- `images`: Max 500 images per document

**Validation:**
```rust
impl DocumentContent {
    /// Validates the content
    pub fn validate(&self) -> Result<(), ValidationError> {
        if self.raw.len() > 104_857_600 {
            return Err(ValidationError::RawContentTooLarge);
        }
        if self.html.len() > 209_715_200 {
            return Err(ValidationError::HtmlContentTooLarge);
        }
        if self.code_blocks.len() > 1000 {
            return Err(ValidationError::TooManyCodeBlocks);
        }
        if self.images.len() > 500 {
            return Err(ValidationError::TooManyImages);
        }
        // Verify hash matches raw content
        if self.hash != ContentHash::compute(self.raw.as_bytes()) {
            return Err(ValidationError::HashMismatch);
        }
        Ok(())
    }
}
```

**Security Considerations:**
- HTML content must be sanitized (handled by rendering pipeline)
- Code blocks may contain sensitive information
- External links require validation and security headers
- Image sources validated to prevent XSS

**Related Requirements:**
- REQ-SYS-036: Markdown Parsing
- REQ-SYS-037: Code Highlighting
- REQ-SYS-018: Table of Contents

**Related Design Elements:**
- DES-DM-005: Document Content Design

---

## 4. USER ENTITIES

### 4.1. User ID

**Element ID:** API-DM-006
**Name:** UserId
**Type:** Newtype Struct
**Language:** Rust
**Module:** `tachyon_auth::types`

**Description:** Unique identifier for user accounts within the Tachyon system.

**Rust Definition:**

```rust
use uuid::Uuid;
use serde::{Serialize, Deserialize};

/// Unique user identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct UserId(Uuid);

impl UserId {
    /// Creates a new random UserId
    #[must_use]
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Creates a UserId from a UUID string
    pub fn from_str(s: &str) -> Result<Self, ParseError> {
        let uuid = Uuid::parse_str(s)
            .map_err(|_| ParseError::InvalidFormat(s.to_string()))?;
        if uuid.get_version() != Some(uuid::Version::Random) {
            return Err(ParseError::InvalidVersion);
        }
        Ok(Self(uuid))
    }

    /// Returns inner UUID
    #[must_use]
    pub fn inner(&self) -> Uuid {
        self.0
    }
}

impl std::fmt::Display for UserId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Default for UserId {
    fn default() -> Self {
        Self::new()
    }
}
```

**Constraints:**
- Must be a valid UUID v4
- Cannot be nil/null

**Security Considerations:**
- User IDs are not guessable, preventing enumeration attacks
- No sensitive information encoded in the identifier
- Safe to expose in URLs and logs

**Related Requirements:**
- REQ-SRV-076: Session Management
- REQ-SRV-081: RBAC Enforcement

---

### 4.2. User

**Element ID:** API-DM-007
**Name:** User
**Type:** Struct
**Language:** Rust
**Module:** `tachyon_auth::user`

**Description:** User account information for authentication and authorization.

**Rust Definition:**

```rust
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};

/// User account
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct User {
    /// Unique user identifier
    pub id: UserId,

    /// Username
    pub username: String,

    /// Email address
    pub email: String,

    /// Password hash (bcrypt)
    #[serde(skip_serializing)]
    pub password_hash: String,

    /// Display name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,

    /// User roles
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub roles: Vec<Role>,

    /// Account creation timestamp
    pub created_at: DateTime<Utc>,

    /// Last login timestamp
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_login_at: Option<DateTime<Utc>>,

    /// Account status
    pub status: UserStatus,

    /// MFA enabled flag
    #[serde(default)]
    pub mfa_enabled: bool,

    /// MFA secret (TOTP)
    #[serde(skip_serializing)]
    pub mfa_secret: Option<String>,
}

/// User role
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum Role {
    /// Administrator role
    Admin,
    /// Editor role
    Editor,
    /// Viewer role
    Viewer,
    /// Custom role
    Custom(String),
}

/// User status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum UserStatus {
    /// Active account
    Active,
    /// Suspended account
    Suspended,
    /// Deleted account
    Deleted,
}
```

**Constraints:**
- `username`: 3-64 characters, alphanumeric plus hyphens/underscores
- `email`: Valid email format, max 255 characters
- `display_name`: Max 255 characters
- `roles`: Max 10 roles per user
- `password_hash`: bcrypt hash, 60 characters

**Validation:**
```rust
impl User {
    /// Validates user data
    pub fn validate(&self) -> Result<(), ValidationError> {
        // Username validation
        if self.username.len() < 3 || self.username.len() > 64 {
            return Err(ValidationError::InvalidUsernameLength);
        }
        if !self.username.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
            return Err(ValidationError::InvalidUsernameCharacters);
        }

        // Email validation
        if self.email.len() > 255 {
            return Err(ValidationError::EmailTooLong);
        }
        if !self.email.contains('@') || !self.email.contains('.') {
            return Err(ValidationError::InvalidEmailFormat);
        }

        // Display name validation
        if let Some(name) = &self.display_name {
            if name.len() > 255 {
                return Err(ValidationError::DisplayNameTooLong);
            }
        }

        // Roles validation
        if self.roles.len() > 10 {
            return Err(ValidationError::TooManyRoles);
        }

        Ok(())
    }
}
```

**Security Considerations:**
- Passwords never serialized (skip_serializing)
- Passwords hashed with bcrypt before storage
- Email addresses are PII, require protection
- Role changes require audit logging
- MFA secret never serialized

**Related Requirements:**
- REQ-SRV-076: Session Management
- REQ-SRV-081: RBAC Enforcement

**Related Design Elements:**
- DES-DM-006: User Design

---

### 4.3. Session

**Element ID:** API-DM-008
**Name:** Session
**Type:** Struct
**Language:** Rust
**Module:** `tachyon_auth::session`

**Description:** User session for authentication state management.

**Rust Definition:**

```rust
use chrono::{DateTime, Utc, Duration};
use serde::{Serialize, Deserialize};
use std::net::IpAddr;

/// User session
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Session {
    /// Unique session identifier
    pub id: SessionId,

    /// Associated user ID
    pub user_id: UserId,

    /// Session creation timestamp
    pub created_at: DateTime<Utc>,

    /// Session expiration timestamp
    pub expires_at: DateTime<Utc>,

    /// Last activity timestamp
    pub last_activity_at: DateTime<Utc>,

    /// IP address of session origin
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ip_address: Option<IpAddr>,

    /// User agent string
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_agent: Option<String>,

    /// Session metadata
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub metadata: HashMap<String, String>,
}

/// Session ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SessionId(Uuid);

impl SessionId {
    /// Creates a new random SessionId
    #[must_use]
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Creates a SessionId from a UUID string
    pub fn from_str(s: &str) -> Result<Self, ParseError> {
        let uuid = Uuid::parse_str(s)
            .map_err(|_| ParseError::InvalidFormat(s.to_string()))?;
        if uuid.get_version() != Some(uuid::Version::Random) {
            return Err(ParseError::InvalidVersion);
        }
        Ok(Self(uuid))
    }
}

impl std::fmt::Display for SessionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Session {
    /// Creates a new session with default expiration
    ///
    /// # Parameters
    /// - `user_id`: User ID for the session
    /// - `duration`: Session duration
    ///
    /// # Returns
    /// New session with calculated expiration
    #[must_use]
    pub fn new(user_id: UserId, duration: Duration) -> Self {
        let now = Utc::now();
        Self {
            id: SessionId::new(),
            user_id,
            created_at: now,
            expires_at: now + duration,
            last_activity_at: now,
            ip_address: None,
            user_agent: None,
            metadata: HashMap::new(),
        }
    }

    /// Checks if session is expired
    #[must_use]
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }

    /// Updates last activity timestamp
    pub fn update_activity(&mut self) {
        self.last_activity_at = Utc::now();
    }

    /// Extends session expiration
    ///
    /// # Parameters
    /// - `duration`: Additional duration to add
    pub fn extend(&mut self, duration: Duration) {
        self.expires_at = Utc::now() + duration;
    }
}
```

**Constraints:**
- `id`: Must be a valid UUID v4
- `expires_at`: Must be after `created_at`
- `last_activity_at`: Must be between `created_at` and now
- `user_agent`: Max 512 characters
- `metadata`: Max 10 key-value pairs

**Security Considerations:**
- Session IDs are not guessable
- IP address tracking for security monitoring
- User agent tracking for anomaly detection
- Sessions expire automatically
- Session metadata may contain sensitive information

**Related Requirements:**
- REQ-SRV-076: Session Management
- REQ-SRV-077: Session Expiration

**Related Design Elements:**
- DES-DM-007: Session Design

---

### 4.4. Authentication Token

**Element ID:** API-DM-009
**Name:** AuthToken
**Type:** Struct
**Language:** Rust
**Module:** `tachyon_auth::token`

**Description:** JWT-based authentication token for API access.

**Rust Definition:**

```rust
use chrono::{DateTime, Utc, Duration};
use serde::{Serialize, Deserialize};

/// Authentication token
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthToken {
    /// Token ID
    pub id: TokenId,

    /// User ID
    pub user_id: UserId,

    /// Token type
    pub token_type: TokenType,

    /// Issued at timestamp
    pub issued_at: DateTime<Utc>,

    /// Expires at timestamp
    pub expires_at: DateTime<Utc>,

    /// Token scopes
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub scopes: Vec<String>,

    /// Refresh token (if applicable)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refresh_token: Option<String>,
}

/// Token ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct TokenId(String);

impl TokenId {
    /// Creates a new random TokenId
    #[must_use]
    pub fn new() -> Self {
        Self(nanoid::nanoid!())
    }

    /// Returns token ID as string
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for TokenId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Token type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TokenType {
    /// Access token
    Access,
    /// Refresh token
    Refresh,
    /// API key
    ApiKey,
}

impl AuthToken {
    /// Creates a new access token
    ///
    /// # Parameters
    /// - `user_id`: User ID
    /// - `duration`: Token duration
    /// - `scopes`: Token scopes
    ///
    /// # Returns
    /// New access token
    #[must_use]
    pub fn new_access_token(user_id: UserId, duration: Duration, scopes: Vec<String>) -> Self {
        let now = Utc::now();
        Self {
            id: TokenId::new(),
            user_id,
            token_type: TokenType::Access,
            issued_at: now,
            expires_at: now + duration,
            scopes,
            refresh_token: None,
        }
    }

    /// Checks if token is expired
    #[must_use]
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }

    /// Checks if token has scope
    ///
    /// # Parameters
    /// - `scope`: Scope to check
    ///
    /// # Returns
    /// `true` if token has scope, `false` otherwise
    #[must_use]
    pub fn has_scope(&self, scope: &str) -> bool {
        self.scopes.iter().any(|s| s == scope)
    }
}
```

**Constraints:**
- `id`: Max 64 characters
- `scopes`: Max 20 scopes per token
- `scope`: Max 64 characters per scope
- `expires_at`: Must be after `issued_at`

**Security Considerations:**
- Tokens are signed with JWT
- Tokens expire automatically
- Refresh tokens are single-use
- Scopes enforce principle of least privilege
- Token revocation supported through token ID tracking

**Related Requirements:**
- REQ-SRV-078: Token Management
- REQ-SRV-079: Token Revocation

**Related Design Elements:**
- DES-DM-008: Authentication Token Design

---

## 5. GIT ENTITIES

### 5.1. Repository

**Element ID:** API-DM-010
**Name:** Repository
**Type:** Struct
**Language:** Rust
**Module:** `tachyon_git::repository`

**Description:** Git repository representation within the Tachyon system.

**Rust Definition:**

```rust
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use std::path::PathBuf;

/// Git repository
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Repository {
    /// Repository ID
    pub id: RepositoryId,

    /// Repository name
    pub name: String,

    /// Repository description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Repository path (local or remote)
    pub path: String,

    /// Repository type
    pub repo_type: RepositoryType,

    /// Default branch name
    pub default_branch: String,

    /// Current branch name
    pub current_branch: String,

    /// Repository creation timestamp
    pub created_at: DateTime<Utc>,

    /// Last updated timestamp
    pub updated_at: DateTime<Utc>,

    /// Repository status
    pub status: RepositoryStatus,

    /// Repository metadata
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Repository ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct RepositoryId(Uuid);

impl RepositoryId {
    /// Creates a new random RepositoryId
    #[must_use]
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Creates a RepositoryId from a UUID string
    pub fn from_str(s: &str) -> Result<Self, ParseError> {
        let uuid = Uuid::parse_str(s)
            .map_err(|_| ParseError::InvalidFormat(s.to_string()))?;
        if uuid.get_version() != Some(uuid::Version::Random) {
            return Err(ParseError::InvalidVersion);
        }
        Ok(Self(uuid))
    }
}

impl std::fmt::Display for RepositoryId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Repository type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RepositoryType {
    /// Local repository
    Local,
    /// Remote repository (URL)
    Remote { url: String },
    /// Bare repository
    Bare,
}

/// Repository status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RepositoryStatus {
    /// Active repository
    Active,
    /// Cloning in progress
    Cloning,
    /// Repository error
    Error { message: String },
    /// Repository not found
    NotFound,
}
```

**Constraints:**
- `name`: 1-255 characters, alphanumeric plus hyphens/underscores
- `description`: Max 1024 characters
- `path`: Valid file path or URL, max 2048 characters
- `default_branch`: Max 255 characters
- `current_branch`: Max 255 characters
- `metadata`: Max 20 key-value pairs

**Security Considerations:**
- Remote URLs validated to prevent SSRF attacks
- Path traversal prevention through validation
- Repository access controlled through workspace permissions
- Sensitive data in metadata requires encryption

**Related Requirements:**
- REQ-DESK-037: Repository Cloning
- REQ-DESK-038: Repository Synchronization

**Related Design Elements:**
- DES-DM-009: Repository Design

---

### 5.2. Commit

**Element ID:** API-DM-011
**Name:** Commit
**Type:** Struct
**Language:** Rust
**Module:** `tachyon_git::commit`

**Description:** Git commit representation within the Tachyon system.

**Rust Definition:**

```rust
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};

/// Git commit
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Commit {
    /// Commit hash (SHA-1)
    pub hash: CommitHash,

    /// Repository ID
    pub repository_id: RepositoryId,

    /// Commit author
    pub author: CommitAuthor,

    /// Committer
    pub committer: CommitAuthor,

    /// Commit message
    pub message: String,

    /// Commit timestamp
    pub committed_at: DateTime<Utc>,

    /// Parent commit hashes
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub parents: Vec<CommitHash>,

    /// Tree hash
    pub tree_hash: String,

    /// Changed files
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub changed_files: Vec<ChangedFile>,
}

/// Commit hash (SHA-1)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct CommitHash(String);

impl CommitHash {
    /// Creates a CommitHash from a hex string
    pub fn from_hex(hex: &str) -> Result<Self, ParseError> {
        if hex.len() != 40 {
            return Err(ParseError::InvalidLength);
        }
        if !hex.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(ParseError::InvalidHex);
        }
        Ok(Self(hex.to_lowercase()))
    }

    /// Returns commit hash as string
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Returns short hash (first 7 characters)
    #[must_use]
    pub fn short(&self) -> &str {
        &self.0[..7]
    }
}

impl std::fmt::Display for CommitHash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Commit author
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommitAuthor {
    /// Author name
    pub name: String,

    /// Author email
    pub email: String,

    /// Author timestamp
    pub timestamp: DateTime<Utc>,
}

/// Changed file
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChangedFile {
    /// File path
    pub path: String,

    /// Change type
    pub change_type: ChangeType,

    /// Old hash (for modifications/deletions)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub old_hash: Option<String>,

    /// New hash (for additions/modifications)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub new_hash: Option<String>,
}

/// Change type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ChangeType {
    /// File added
    Added,
    /// File modified
    Modified,
    /// File deleted
    Deleted,
    /// File renamed
    Renamed { old_path: String },
    /// File copied
    Copied { from_path: String },
}
```

**Constraints:**
- `hash`: 40-character hex string (SHA-1)
- `message`: Max 4096 characters
- `parents`: Max 2 parent commits
- `changed_files`: Max 1000 files per commit
- `path`: Max 1024 characters

**Security Considerations:**
- Commit hashes validated to prevent injection
- Path traversal prevention in file paths
- Author information may contain PII
- Commit messages may contain sensitive information

**Related Requirements:**
- REQ-DESK-039: Commit History
- REQ-DESK-040: Diff Generation

**Related Design Elements:**
- DES-DM-010: Commit Design

---

### 5.3. Branch

**Element ID:** API-DM-012
**Name:** Branch
**Type:** Struct
**Language:** Rust
**Module:** `tachyon_git::branch`

**Description:** Git branch representation within the Tachyon system.

**Rust Definition:**

```rust
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};

/// Git branch
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Branch {
    /// Repository ID
    pub repository_id: RepositoryId,

    /// Branch name
    pub name: String,

    /// Branch head commit hash
    pub head: CommitHash,

    /// Branch creation timestamp
    pub created_at: DateTime<Utc>,

    /// Last updated timestamp
    pub updated_at: DateTime<Utc>,

    /// Is default branch
    #[serde(default)]
    pub is_default: bool,

    /// Is protected branch
    #[serde(default)]
    pub is_protected: bool,

    /// Branch metadata
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub metadata: HashMap<String, serde_json::Value>,
}

impl Branch {
    /// Checks if branch is remote
    #[must_use]
    pub fn is_remote(&self) -> bool {
        self.name.starts_with("origin/")
    }

    /// Returns local branch name (strips origin/ prefix)
    #[must_use]
    pub fn local_name(&self) -> &str {
        self.name.strip_prefix("origin/").unwrap_or(&self.name)
    }
}
```

**Constraints:**
- `name`: 1-255 characters, alphanumeric plus hyphens/underscores/slashes
- `head`: Valid 40-character SHA-1 hash
- `metadata`: Max 10 key-value pairs

**Security Considerations:**
- Branch name validation prevents injection
- Protected branches require special permissions
- Branch deletion requires authorization

**Related Requirements:**
- REQ-DESK-041: Branch Management
- REQ-DESK-042: Merge Operations

**Related Design Elements:**
- DES-DM-011: Branch Design

---

### 5.4. Tag

**Element ID:** API-DM-013
**Name:** Tag
**Type:** Struct
**Language:** Rust
**Module:** `tachyon_git::tag`

**Description:** Git tag representation within the Tachyon system.

**Rust Definition:**

```rust
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};

/// Git tag
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Tag {
    /// Repository ID
    pub repository_id: RepositoryId,

    /// Tag name
    pub name: String,

    /// Tag commit hash
    pub commit: CommitHash,

    /// Tag message
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,

    /// Tagger
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tagger: Option<CommitAuthor>,

    /// Tag creation timestamp
    pub created_at: DateTime<Utc>,

    /// Is annotated tag
    #[serde(default)]
    pub is_annotated: bool,

    /// Tag metadata
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub metadata: HashMap<String, serde_json::Value>,
}
```

**Constraints:**
- `name`: 1-255 characters, alphanumeric plus hyphens/underscores/dots
- `message`: Max 4096 characters
- `metadata`: Max 10 key-value pairs

**Security Considerations:**
- Tag name validation prevents injection
- Tag deletion requires authorization
- Tag signing verification for security

**Related Requirements:**
- REQ-DESK-043: Tag Management
- REQ-DESK-044: Release Management

**Related Design Elements:**
- DES-DM-012: Tag Design

---

## 6. PLUGIN ENTITIES

### 6.1. Plugin

**Element ID:** API-DM-014
**Name:** Plugin
**Type:** Struct
**Language:** Rust
**Module:** `tachyon_plugin::plugin`

**Description:** Plugin representation for extensible functionality within the Tachyon system.

**Rust Definition:**

```rust
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use semver::Version;

/// Plugin
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Plugin {
    /// Plugin ID
    pub id: PluginId,

    /// Plugin name
    pub name: String,

    /// Plugin description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Plugin version
    pub version: Version,

    /// Plugin author
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,

    /// Plugin homepage URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub homepage: Option<String>,

    /// Plugin repository URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repository: Option<String>,

    /// Plugin license
    #[serde(skip_serializing_if = "Option::is_none")]
    pub license: Option<String>,

    /// Plugin capabilities
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub capabilities: Vec<Capability>,

    /// Plugin installation timestamp
    pub installed_at: DateTime<Utc>,

    /// Last updated timestamp
    pub updated_at: DateTime<Utc>,

    /// Plugin status
    pub status: PluginStatus,

    /// Plugin configuration
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub config: HashMap<String, serde_json::Value>,
}

/// Plugin ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct PluginId(String);

impl PluginId {
    /// Creates a new PluginId from a string
    pub fn new(id: String) -> Self {
        Self(id)
    }

    /// Returns plugin ID as string
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for PluginId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Plugin status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PluginStatus {
    /// Plugin installed and active
    Active,
    /// Plugin installed but disabled
    Disabled,
    /// Plugin installation in progress
    Installing,
    /// Plugin uninstallation in progress
    Uninstalling,
    /// Plugin error
    Error { message: String },
    /// Plugin not found
    NotFound,
}
```

**Constraints:**
- `id`: 1-255 characters, alphanumeric plus hyphens/underscores/dots
- `name`: 1-255 characters
- `description`: Max 1024 characters
- `author`: Max 255 characters
- `homepage`: Valid URL, max 2048 characters
- `repository`: Valid URL, max 2048 characters
- `license`: Max 64 characters
- `capabilities`: Max 50 capabilities
- `config`: Max 20 key-value pairs

**Security Considerations:**
- Plugin IDs validated to prevent injection
- Plugin capabilities enforce principle of least privilege
- Plugin execution in sandboxed environment
- Plugin code signing verification

**Related Requirements:**
- REQ-SYS-060: Plugin System
- REQ-SYS-061: Plugin Capabilities

**Related Design Elements:**
- DES-DM-013: Plugin Design

---

### 6.2. Capability

**Element ID:** API-DM-015
**Name:** Capability
**Type:** Struct
**Language:** Rust
**Module:** `tachyon_plugin::capability`

**Description:** Plugin capability definition for fine-grained permission control.

**Rust Definition:**

```rust
use serde::{Serialize, Deserialize};

/// Plugin capability
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Capability {
    /// Capability name
    pub name: String,

    /// Capability description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Capability parameters
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub parameters: HashMap<String, serde_json::Value>,

    /// Is required capability
    #[serde(default)]
    pub required: bool,
}

impl Capability {
    /// Checks if capability has parameter
    ///
    /// # Parameters
    /// - `key`: Parameter key
    ///
    /// # Returns
    /// `true` if capability has parameter, `false` otherwise
    #[must_use]
    pub fn has_parameter(&self, key: &str) -> bool {
        self.parameters.contains_key(key)
    }

    /// Gets parameter value
    ///
    /// # Parameters
    /// - `key`: Parameter key
    ///
    /// # Returns
    /// - `Some(value)` if parameter exists
    /// - `None` if parameter does not exist
    #[must_use]
    pub fn get_parameter(&self, key: &str) -> Option<&serde_json::Value> {
        self.parameters.get(key)
    }
}
```

**Constraints:**
- `name`: 1-255 characters, alphanumeric plus hyphens/underscores
- `description`: Max 1024 characters
- `parameters`: Max 20 key-value pairs

**Security Considerations:**
- Capability names validated to prevent injection
- Capability parameters validated for type safety
- Required capabilities must be granted
- Capability grants require authorization

**Related Requirements:**
- REQ-SYS-061: Plugin Capabilities
- REQ-SYS-062: Capability Authorization

**Related Design Elements:**
- DES-DM-014: Capability Design

---

### 6.3. Plugin Configuration

**Element ID:** API-DM-016
**Name:** PluginConfig
**Type:** Struct
**Language:** Rust
**Module:** `tachyon_plugin::config`

**Description:** Plugin configuration for runtime behavior control.

**Rust Definition:**

```rust
use serde::{Serialize, Deserialize};

/// Plugin configuration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PluginConfig {
    /// Plugin ID
    pub plugin_id: PluginId,

    /// Configuration values
    pub values: HashMap<String, ConfigValue>,

    /// Is configuration enabled
    #[serde(default)]
    pub enabled: bool,
}

/// Configuration value
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ConfigValue {
    /// String value
    String(String),
    /// Integer value
    Integer(i64),
    /// Float value
    Float(f64),
    /// Boolean value
    Boolean(bool),
    /// Array value
    Array(Vec<ConfigValue>),
    /// Object value
    Object(HashMap<String, ConfigValue>),
    /// Null value
    Null,
}

impl PluginConfig {
    /// Gets configuration value
    ///
    /// # Parameters
    /// - `key`: Configuration key
    ///
    /// # Returns
    /// - `Some(value)` if key exists
    /// - `None` if key does not exist
    #[must_use]
    pub fn get(&self, key: &str) -> Option<&ConfigValue> {
        self.values.get(key)
    }

    /// Sets configuration value
    ///
    /// # Parameters
    /// - `key`: Configuration key
    /// - `value`: Configuration value
    pub fn set(&mut self, key: String, value: ConfigValue) {
        self.values.insert(key, value);
    }

    /// Removes configuration value
    ///
    /// # Parameters
    /// - `key`: Configuration key
    ///
    /// # Returns
    /// - `Some(value)` if key existed
    /// - `None` if key did not exist
    pub fn remove(&mut self, key: &str) -> Option<ConfigValue> {
        self.values.remove(key)
    }
}
```

**Constraints:**
- `values`: Max 50 key-value pairs
- `key`: 1-255 characters
- `Array`: Max 100 elements

**Security Considerations:**
- Configuration keys validated to prevent injection
- Configuration values validated for type safety
- Sensitive configuration values encrypted
- Configuration changes require authorization

**Related Requirements:**
- REQ-SYS-063: Plugin Configuration
- REQ-SYS-064: Configuration Validation

**Related Design Elements:**
- DES-DM-015: Plugin Configuration Design

---

## 7. CONFIGURATION ENTITIES

### 7.1. System Configuration

**Element ID:** API-DM-017
**Name:** SystemConfig
**Type:** Struct
**Language:** Rust
**Module:** `tachyon_config::system`

**Description:** System-wide configuration settings for Tachyon application.

**Rust Definition:**

```rust
use serde::{Serialize, Deserialize};

/// System configuration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SystemConfig {
    /// Application name
    pub app_name: String,

    /// Application version
    pub app_version: String,

    /// Environment (development, staging, production)
    pub environment: Environment,

    /// Server configuration
    pub server: ServerConfig,

    /// Database configuration
    pub database: DatabaseConfig,

    /// Cache configuration
    pub cache: CacheConfig,

    /// Logging configuration
    pub logging: LoggingConfig,

    /// Security configuration
    pub security: SecurityConfig,
}

/// Environment
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Environment {
    /// Development environment
    Development,
    /// Staging environment
    Staging,
    /// Production environment
    Production,
}

/// Server configuration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ServerConfig {
    /// Server host
    pub host: String,

    /// Server port
    pub port: u16,

    /// TLS enabled
    #[serde(default)]
    pub tls_enabled: bool,

    /// TLS certificate path
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tls_cert_path: Option<String>,

    /// TLS key path
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tls_key_path: Option<String>,

    /// Max request size (bytes)
    #[serde(default = "default_max_request_size")]
    pub max_request_size: usize,
}

const fn default_max_request_size() -> usize {
    10 * 1024 * 1024 // 10MB
}

/// Database configuration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DatabaseConfig {
    /// Database path
    pub path: String,

    /// Connection pool size
    #[serde(default = "default_pool_size")]
    pub pool_size: u32,

    /// Connection timeout (seconds)
    #[serde(default = "default_timeout")]
    pub timeout: u64,
}

const fn default_pool_size() -> u32 {
    10
}

const fn default_timeout() -> u64 {
    30
}

/// Cache configuration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CacheConfig {
    /// Cache enabled
    #[serde(default)]
    pub enabled: bool,

    /// Cache size (bytes)
    #[serde(default = "default_cache_size")]
    pub size: usize,

    /// Cache TTL (seconds)
    #[serde(default = "default_cache_ttl")]
    pub ttl: u64,
}

const fn default_cache_size() -> usize {
    100 * 1024 * 1024 // 100MB
}

const fn default_cache_ttl() -> u64 {
    3600 // 1 hour
}

/// Logging configuration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// Log level
    #[serde(default = "default_log_level")]
    pub level: LogLevel,

    /// Log format
    #[serde(default = "default_log_format")]
    pub format: LogFormat,

    /// Log file path
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_path: Option<String>,

    /// Console logging enabled
    #[serde(default = "default_console_enabled")]
    pub console_enabled: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LogFormat {
    Json,
    Pretty,
    Compact,
}

const fn default_log_level() -> LogLevel {
    LogLevel::Info
}

const fn default_log_format() -> LogFormat {
    LogFormat::Json
}

const fn default_console_enabled() -> bool {
    true
}

/// Security configuration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SecurityConfig {
    /// Session timeout (seconds)
    #[serde(default = "default_session_timeout")]
    pub session_timeout: u64,

    /// Max login attempts
    #[serde(default = "default_max_login_attempts")]
    pub max_login_attempts: u32,

    /// Password min length
    #[serde(default = "default_password_min_length")]
    pub password_min_length: u32,

    /// MFA enabled
    #[serde(default)]
    pub mfa_enabled: bool,

    /// Encryption enabled
    #[serde(default)]
    pub encryption_enabled: bool,
}

const fn default_session_timeout() -> u64 {
    86400 // 24 hours
}

const fn default_max_login_attempts() -> u32 {
    5
}

const fn default_password_min_length() -> u32 {
    8
}
```

**Constraints:**
- `app_name`: 1-255 characters
- `app_version`: Valid semver string
- `server.host`: Valid hostname or IP address
- `server.port`: 1-65535
- `database.path`: Valid file path, max 2048 characters
- `logging.file_path`: Valid file path, max 2048 characters

**Security Considerations:**
- Sensitive configuration values encrypted
- Configuration changes require authorization
- Configuration validation before application
- Audit logging for configuration changes

**Related Requirements:**
- REQ-SYS-065: System Configuration
- REQ-SYS-066: Configuration Security

**Related Design Elements:**
- DES-DM-016: System Configuration Design

---

### 7.2. User Preferences

**Element ID:** API-DM-018
**Name:** UserPreferences
**Type:** Struct
**Language:** Rust
**Module:** `tachyon_config::preferences`

**Description:** User-specific preferences and settings.

**Rust Definition:**

```rust
use serde::{Serialize, Deserialize};

/// User preferences
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UserPreferences {
    /// User ID
    pub user_id: UserId,

    /// Theme preference
    #[serde(default = "default_theme")]
    pub theme: Theme,

    /// Language preference
    #[serde(default = "default_language")]
    pub language: String,

    /// Timezone preference
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timezone: Option<String>,

    /// Date format preference
    #[serde(default = "default_date_format")]
    pub date_format: String,

    /// Time format preference
    #[serde(default = "default_time_format")]
    pub time_format: String,

    /// Editor settings
    #[serde(default)]
    pub editor: EditorSettings,

    /// Notification settings
    #[serde(default)]
    pub notifications: NotificationSettings,
}

/// Theme preference
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Theme {
    /// Light theme
    Light,
    /// Dark theme
    Dark,
    /// System theme
    System,
    /// Custom theme
    Custom(String),
}

const fn default_theme() -> Theme {
    Theme::System
}

const fn default_language() -> String {
    "en".to_string()
}

const fn default_date_format() -> String {
    "%Y-%m-%d".to_string()
}

const fn default_time_format() -> String {
    "%H:%M:%S".to_string()
}

/// Editor settings
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EditorSettings {
    /// Font family
    #[serde(default = "default_font_family")]
    pub font_family: String,

    /// Font size
    #[serde(default = "default_font_size")]
    pub font_size: u16,

    /// Line numbers enabled
    #[serde(default = "default_line_numbers")]
    pub line_numbers: bool,

    /// Word wrap enabled
    #[serde(default = "default_word_wrap")]
    pub word_wrap: bool,

    /// Tab size
    #[serde(default = "default_tab_size")]
    pub tab_size: u16,

    /// Insert spaces for tabs
    #[serde(default = "default_insert_spaces")]
    pub insert_spaces: bool,
}

const fn default_font_family() -> String {
    "monospace".to_string()
}

const fn default_font_size() -> u16 {
    14
}

const fn default_line_numbers() -> bool {
    true
}

const fn default_word_wrap() -> bool {
    false
}

const fn default_tab_size() -> u16 {
    4
}

const fn default_insert_spaces() -> bool {
    true
}

/// Notification settings
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NotificationSettings {
    /// Email notifications enabled
    #[serde(default)]
    pub email_enabled: bool,

    /// Desktop notifications enabled
    #[serde(default)]
    pub desktop_enabled: bool,

    /// Notification types
    #[serde(default)]
    pub types: Vec<NotificationType>,
}

/// Notification type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NotificationType {
    /// Document created
    DocumentCreated,
    /// Document updated
    DocumentUpdated,
    /// Document deleted
    DocumentDeleted,
    /// Comment added
    CommentAdded,
    /// System announcement
    SystemAnnouncement,
}
```

**Constraints:**
- `language`: Valid ISO 639-1 language code
- `timezone`: Valid IANA timezone identifier
- `date_format`: Valid strftime format string
- `time_format`: Valid strftime format string
- `editor.font_family`: Max 255 characters
- `editor.font_size`: 8-72
- `editor.tab_size`: 1-8

**Security Considerations:**
- User preferences encrypted at rest
- Preference changes require authentication
- Sensitive preferences require special handling
- Audit logging for preference changes

**Related Requirements:**
- REQ-SYS-067: User Preferences
- REQ-SYS-068: Preference Security

**Related Design Elements:**
- DES-DM-017: User Preferences Design

---

## 8. EVENT ENTITIES

### 8.1. Event

**Element ID:** API-DM-019
**Name:** Event
**Type:** Struct
**Language:** Rust
**Module:** `tachyon_events::event`

**Description:** System event for audit logging and event-driven architecture.

**Rust Definition:**

```rust
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};

/// System event
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Event {
    /// Event ID
    pub id: EventId,

    /// Event type
    pub event_type: EventType,

    /// Event timestamp
    pub timestamp: DateTime<Utc>,

    /// Event source
    pub source: EventSource,

    /// User ID (if applicable)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<UserId>,

    /// Session ID (if applicable)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<SessionId>,

    /// Event data
    pub data: EventData,

    /// Event severity
    pub severity: EventSeverity,
}

/// Event ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct EventId(Uuid);

impl EventId {
    /// Creates a new random EventId
    #[must_use]
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Creates an EventId from a UUID string
    pub fn from_str(s: &str) -> Result<Self, ParseError> {
        let uuid = Uuid::parse_str(s)
            .map_err(|_| ParseError::InvalidFormat(s.to_string()))?;
        if uuid.get_version() != Some(uuid::Version::Random) {
            return Err(ParseError::InvalidVersion);
        }
        Ok(Self(uuid))
    }
}

impl std::fmt::Display for EventId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Event type
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum EventType {
    /// User logged in
    UserLoggedIn { username: String },
    /// User logged out
    UserLoggedOut { username: String },
    /// Document created
    DocumentCreated { document_id: DocumentId, title: String },
    /// Document updated
    DocumentUpdated { document_id: DocumentId, title: String },
    /// Document deleted
    DocumentDeleted { document_id: DocumentId, title: String },
    /// Repository cloned
    RepositoryCloned { repository_id: RepositoryId, name: String },
    /// Repository synchronized
    RepositorySynchronized { repository_id: RepositoryId, name: String },
    /// Plugin installed
    PluginInstalled { plugin_id: PluginId, name: String },
    /// Plugin uninstalled
    PluginUninstalled { plugin_id: PluginId, name: String },
    /// Configuration changed
    ConfigurationChanged { key: String, old_value: Option<String>, new_value: Option<String> },
    /// Error occurred
    ErrorOccurred { error_type: String, message: String },
}

/// Event source
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EventSource {
    /// Desktop application
    Desktop,
    /// Server application
    Server,
    /// Web application
    Web,
    /// Background worker
    Worker,
    /// External system
    External { system: String },
}

/// Event data
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EventData {
    /// Additional event data
    #[serde(flatten)]
    pub data: serde_json::Value,
}

/// Event severity
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EventSeverity {
    /// Info level
    Info,
    /// Warning level
    Warning,
    /// Error level
    Error,
    /// Critical level
    Critical,
}
```

**Constraints:**
- `data`: Max 4096 bytes when serialized
- `source.system`: Max 255 characters

**Security Considerations:**
- Events are immutable by design
- Event IDs are not guessable
- Sensitive data in events requires encryption
- Event logging for audit trail
- Event retention policy enforced

**Related Requirements:**
- REQ-SYS-069: Event Logging
- REQ-SYS-070: Event Audit

**Related Design Elements:**
- DES-DM-018: Event Design

---

### 8.2. Subscription

**Element ID:** API-DM-020
**Name:** Subscription
**Type:** Struct
**Language:** Rust
**Module:** `tachyon_events::subscription`

**Description:** Event subscription for event-driven architecture.

**Rust Definition:**

```rust
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};

/// Event subscription
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Subscription {
    /// Subscription ID
    pub id: SubscriptionId,

    /// Subscriber ID (user or system component)
    pub subscriber_id: String,

    /// Event types to subscribe to
    pub event_types: Vec<String>,

    /// Subscription filters
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub filters: Option<SubscriptionFilters>,

    /// Subscription created at
    pub created_at: DateTime<Utc>,

    /// Subscription expires at
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<DateTime<Utc>>,

    /// Is subscription active
    #[serde(default)]
    pub active: bool,
}

/// Subscription ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SubscriptionId(Uuid);

impl SubscriptionId {
    /// Creates a new random SubscriptionId
    #[must_use]
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Creates a SubscriptionId from a UUID string
    pub fn from_str(s: &str) -> Result<Self, ParseError> {
        let uuid = Uuid::parse_str(s)
            .map_err(|_| ParseError::InvalidFormat(s.to_string()))?;
        if uuid.get_version() != Some(uuid::Version::Random) {
            return Err(ParseError::InvalidVersion);
        }
        Ok(Self(uuid))
    }
}

impl std::fmt::Display for SubscriptionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Subscription filters
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SubscriptionFilters {
    /// User ID filter
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<UserId>,

    /// Repository ID filter
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repository_id: Option<RepositoryId>,

    /// Event source filter
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<EventSource>,

    /// Severity filter
    #[serde(skip_serializing_if = "Option::is_none")]
    pub severity: Option<EventSeverity>,
}

impl Subscription {
    /// Checks if subscription matches event
    ///
    /// # Parameters
    /// - `event`: Event to check
    ///
    /// # Returns
    /// `true` if subscription matches event, `false` otherwise
    #[must_use]
    pub fn matches(&self, event: &Event) -> bool {
        if !self.active {
            return false;
        }

        // Check expiration
        if let Some(expires_at) = self.expires_at {
            if Utc::now() > expires_at {
                return false;
            }
        }

        // Check filters
        if let Some(filters) = &self.filters {
            if let Some(user_id) = filters.user_id {
                if Some(event_user_id) = event.user_id {
                    if user_id != event_user_id {
                        return false;
                    }
                }
            }
        }

        // Check event type
        let event_type_str = match &event.event_type {
            EventType::UserLoggedIn { .. } => "user_logged_in",
            EventType::UserLoggedOut { .. } => "user_logged_out",
            EventType::DocumentCreated { .. } => "document_created",
            EventType::DocumentUpdated { .. } => "document_updated",
            EventType::DocumentDeleted { .. } => "document_deleted",
            EventType::RepositoryCloned { .. } => "repository_cloned",
            EventType::RepositorySynchronized { .. } => "repository_synchronized",
            EventType::PluginInstalled { .. } => "plugin_installed",
            EventType::PluginUninstalled { .. } => "plugin_uninstalled",
            EventType::ConfigurationChanged { .. } => "configuration_changed",
            EventType::ErrorOccurred { .. } => "error_occurred",
        };

        self.event_types.contains(&event_type_str.to_string())
    }
}
```

**Constraints:**
- `subscriber_id`: 1-255 characters
- `event_types`: Max 50 event types
- `event_types[i]`: Max 255 characters per type

**Security Considerations:**
- Subscription IDs are not guessable
- Subscription requires authentication
- Sensitive event data filtered
- Subscription expiration enforced
- Rate limiting on event delivery

**Related Requirements:**
- REQ-SYS-071: Event Subscriptions
- REQ-SYS-072: Event Filtering

**Related Design Elements:**
- DES-DM-019: Subscription Design

---

## 9. RELATIONSHIPS

### 9.1. Entity Relationships Overview

The Tachyon data model defines explicit relationships between entities through foreign key references and relationship tables. This section documents all entity relationships and their constraints.

**Relationship Types:**

| Relationship Type | Description | Example |
|-----------------|-------------|---------|
| **One-to-One** | Single entity relates to single entity | User ↔ Session |
| **One-to-Many** | Single entity relates to multiple entities | Repository → Commits |
| **Many-to-Many** | Multiple entities relate to multiple entities | Users ↔ Documents |
| **Self-Referencing** | Entity relates to itself | Commit → Parent Commits |

### 9.2. User Relationships

#### User ↔ Session (One-to-Many)

```rust
// User has many sessions
impl User {
    pub fn sessions(&self) -> Vec<Session> {
        // Fetch all sessions for this user
    }
}

// Session belongs to one user
impl Session {
    pub fn user(&self) -> User {
        // Fetch user for this session
    }
}
```

**Constraints:**
- A user can have multiple active sessions
- A session belongs to exactly one user
- Sessions are automatically invalidated when user is deleted

**Foreign Key:** `Session.user_id: UserId`

#### User ↔ Documents (Many-to-Many)

```rust
// User has many documents (access control)
impl User {
    pub fn documents(&self) -> Vec<Document> {
        // Fetch all documents user can access
    }
}

// Document has many users (access control)
impl Document {
    pub fn users(&self) -> Vec<User> {
        // Fetch all users who can access this document
    }
}
```

**Constraints:**
- Access controlled through `DocumentMetadata.access`
- Users may have read, write, or admin permissions
- Document deletion does not delete users

**Foreign Key:** `DocumentMetadata.access.users: Vec<String>`

#### User ↔ Repositories (Many-to-Many)

```rust
// User has many repositories
impl User {
    pub fn repositories(&self) -> Vec<Repository> {
        // Fetch all repositories user can access
    }
}

// Repository has many users
impl Repository {
    pub fn users(&self) -> Vec<User> {
        // Fetch all users who can access this repository
    }
}
```

**Constraints:**
- Access controlled through workspace permissions
- Users may have read, write, or admin permissions
- Repository deletion does not delete users

**Foreign Key:** Implicit through workspace membership

### 9.3. Document Relationships

#### Document ↔ Repository (Many-to-One)

```rust
// Document belongs to one repository
impl Document {
    pub fn repository(&self) -> Repository {
        // Fetch repository containing this document
    }
}

// Repository has many documents
impl Repository {
    pub fn documents(&self) -> Vec<Document> {
        // Fetch all documents in this repository
    }
}
```

**Constraints:**
- A document belongs to exactly one repository
- A repository can contain many documents
- Document path is relative to repository root

**Foreign Key:** `DocumentMetadata.path: String` (repository-relative)

#### Document ↔ Commits (Many-to-Many)

```rust
// Document has many commits
impl Document {
    pub fn commits(&self) -> Vec<Commit> {
        // Fetch all commits affecting this document
    }
}

// Commit affects many documents
impl Commit {
    pub fn documents(&self) -> Vec<Document> {
        // Fetch all documents affected by this commit
    }
}
```

**Constraints:**
- A commit can affect multiple documents
- A document has history of commits
- Commit deletion is prohibited (immutable)

**Foreign Key:** `Commit.changed_files: Vec<ChangedFile>`

### 9.4. Repository Relationships

#### Repository ↔ Commits (One-to-Many)

```rust
// Repository has many commits
impl Repository {
    pub fn commits(&self) -> Vec<Commit> {
        // Fetch all commits in this repository
    }
}

// Commit belongs to one repository
impl Commit {
    pub fn repository(&self) -> Repository {
        // Fetch repository for this commit
    }
}
```

**Constraints:**
- A repository has many commits
- A commit belongs to exactly one repository
- Commits are ordered by timestamp

**Foreign Key:** `Commit.repository_id: RepositoryId`

#### Repository ↔ Branches (One-to-Many)

```rust
// Repository has many branches
impl Repository {
    pub fn branches(&self) -> Vec<Branch> {
        // Fetch all branches in this repository
    }
}

// Branch belongs to one repository
impl Branch {
    pub fn repository(&self) -> Repository {
        // Fetch repository for this branch
    }
}
```

**Constraints:**
- A repository has many branches
- A branch belongs to exactly one repository
- Default branch cannot be deleted

**Foreign Key:** `Branch.repository_id: RepositoryId`

#### Repository ↔ Tags (One-to-Many)

```rust
// Repository has many tags
impl Repository {
    pub fn tags(&self) -> Vec<Tag> {
        // Fetch all tags in this repository
    }
}

// Tag belongs to one repository
impl Tag {
    pub fn repository(&self) -> Repository {
        // Fetch repository for this tag
    }
}
```

**Constraints:**
- A repository has many tags
- A tag belongs to exactly one repository
- Tags are immutable once created

**Foreign Key:** `Tag.repository_id: RepositoryId`

### 9.5. Plugin Relationships

#### Plugin ↔ Capabilities (One-to-Many)

```rust
// Plugin has many capabilities
impl Plugin {
    pub fn capabilities(&self) -> Vec<Capability> {
        // Fetch all capabilities provided by this plugin
    }
}

// Capability belongs to one plugin
impl Capability {
    pub fn plugin(&self) -> Plugin {
        // Fetch plugin providing this capability
    }
}
```

**Constraints:**
- A plugin provides multiple capabilities
- A capability belongs to exactly one plugin
- Capabilities are unique per plugin

**Foreign Key:** Implicit through plugin definition

#### Plugin ↔ Configuration (One-to-One)

```rust
// Plugin has one configuration
impl Plugin {
    pub fn config(&self) -> PluginConfig {
        // Fetch configuration for this plugin
    }
}

// Configuration belongs to one plugin
impl PluginConfig {
    pub fn plugin(&self) -> Plugin {
        // Fetch plugin for this configuration
    }
}
```

**Constraints:**
- A plugin has exactly one configuration
- A configuration belongs to exactly one plugin
- Configuration defaults to plugin defaults

**Foreign Key:** `PluginConfig.plugin_id: PluginId`

### 9.6. Event Relationships

#### Event ↔ User (Many-to-One)

```rust
// Event may be associated with one user
impl Event {
    pub fn user(&self) -> Option<User> {
        // Fetch user associated with this event
    }
}

// User has many events
impl User {
    pub fn events(&self) -> Vec<Event> {
        // Fetch all events for this user
    }
}
```

**Constraints:**
- An event may optionally be associated with a user
- A user has many events
- Events are immutable once created

**Foreign Key:** `Event.user_id: Option<UserId>`

#### Event ↔ Subscription (Many-to-Many)

```rust
// Event may be delivered to many subscriptions
impl Event {
    pub fn subscriptions(&self) -> Vec<Subscription> {
        // Fetch all subscriptions matching this event
    }
}

// Subscription receives many events
impl Subscription {
    pub fn events(&self) -> Vec<Event> {
        // Fetch all events matching this subscription
    }
}
```

**Constraints:**
- An event may match multiple subscriptions
- A subscription receives multiple events
- Event delivery is asynchronous

**Foreign Key:** Implicit through event type matching

### 9.7. Relationship Integrity

**Referential Integrity Rules:**

1. **Cascade Delete:** When a parent entity is deleted, child entities are also deleted
2. **Restrict Delete:** When a parent entity is deleted, child entities prevent deletion
3. **Set Null:** When a parent entity is deleted, child entity foreign keys are set to null
4. **No Action:** When a parent entity is deleted, child entities remain unchanged

**Integrity Enforcement Table:**

| Relationship | Delete Rule | Update Rule |
|-------------|-------------|-------------|
| User → Session | Cascade | Cascade |
| User → Documents | Restrict | No Action |
| Document → Repository | Cascade | Cascade |
| Repository → Commits | Restrict | No Action |
| Repository → Branches | Cascade | Cascade |
| Repository → Tags | Restrict | No Action |
| Plugin → Capabilities | Cascade | Cascade |
| Plugin → Configuration | Cascade | Cascade |
| Event → User | Set Null | No Action |
| Event → Subscription | No Action | No Action |

**Security Considerations:**

- Relationship queries enforce access control
- Sensitive relationships require authorization
- Relationship changes are audited
- Cross-relationship validation prevents data inconsistencies

---

## 10. VALIDATION RULES

### 10.1. Validation Framework Overview

The Tachyon data model implements comprehensive validation at multiple layers:

**Validation Layers:**

| Layer | Description | Timing |
|-------|-------------|---------|
| **Type System** | Compile-time type checking | Compilation |
| **Structural** | Field-level constraints | Construction |
| **Semantic** | Business logic validation | Runtime |
| **Referential** | Foreign key integrity | Runtime |
| **Cross-Field** | Multi-field constraints | Runtime |

### 10.2. Type-Level Validation

Rust's type system provides compile-time validation:

```rust
// Type system prevents invalid states
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Document {
    pub id: DocumentId,      // Cannot be null/undefined
    pub title: String,         // Always present, may be empty
    pub content: String,       // Always present, may be empty
    pub created_at: DateTime<Utc>, // Always present, cannot be invalid
}
```

**Type Safety Guarantees:**

- **Null Safety:** `Option<T>` explicitly represents nullable values
- **Enum Exhaustiveness:** All enum variants must be handled
- **Borrow Checking:** Prevents data races at compile time
- **Lifetime Tracking:** Ensures references remain valid

### 10.3. Field-Level Validation

Each entity field has specific constraints:

**String Field Validation:**

| Field | Min Length | Max Length | Pattern |
|-------|------------|-------------|---------|
| `User.username` | 3 | 64 | `[a-zA-Z0-9_-]` |
| `User.email` | 1 | 255 | Valid email format |
| `Document.title` | 1 | 255 | No control characters |
| `Repository.name` | 1 | 255 | `[a-zA-Z0-9_-]` |
| `Plugin.name` | 1 | 255 | `[a-zA-Z0-9_-]` |

**Numeric Field Validation:**

| Field | Type | Min | Max | Default |
|-------|------|-----|-----|---------|
| `Document.size` | u64 | 0 | 104,857,600 | 0 |
| `ServerConfig.port` | u16 | 1 | 65,535 | 8080 |
| `Session.timeout` | u64 | 60 | 8,640,000 | 3,600 |
| `EditorSettings.font_size` | u16 | 8 | 72 | 14 |

**Validation Implementation:**

```rust
use validator::ValidateLength;

#[derive(Debug, Clone, ValidateLength)]
pub struct Username {
    #[validate(length(min = 3, max = 64))]
    pub value: String,
}

#[derive(Debug, Clone, ValidateLength)]
pub struct Email {
    #[validate(email))]
    pub value: String,
}
```

### 10.4. Semantic Validation

Business logic validation ensures data consistency:

**Document Validation:**

```rust
impl Document {
    /// Validates document data
    pub fn validate(&self) -> Result<(), ValidationError> {
        // Validate title
        if let Some(title) = &self.metadata.title {
            if title.is_empty() {
                return Err(ValidationError::TitleRequired);
            }
            if title.len() > 255 {
                return Err(ValidationError::TitleTooLong);
            }
        }

        // Validate content
        if self.content.raw.is_empty() {
            return Err(ValidationError::ContentRequired);
        }
        if self.content.raw.len() > 104_857_600 {
            return Err(ValidationError::ContentTooLarge);
        }

        // Validate hash
        if self.content.hash != ContentHash::compute(self.content.raw.as_bytes()) {
            return Err(ValidationError::HashMismatch);
        }

        Ok(())
    }
}
```

**User Validation:**

```rust
impl User {
    /// Validates user data
    pub fn validate(&self) -> Result<(), ValidationError> {
        // Validate username
        if self.username.len() < 3 || self.username.len() > 64 {
            return Err(ValidationError::InvalidUsernameLength);
        }
        if !self.username.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
            return Err(ValidationError::InvalidUsernameCharacters);
        }

        // Validate email
        if !self.email.contains('@') || !self.email.contains('.') {
            return Err(ValidationError::InvalidEmailFormat);
        }

        // Validate roles
        if self.roles.is_empty() {
            return Err(ValidationError::RolesRequired);
        }
        if self.roles.len() > 10 {
            return Err(ValidationError::TooManyRoles);
        }

        Ok(())
    }
}
```

### 10.5. Referential Validation

Foreign key integrity ensures relationship consistency:

**Foreign Key Validation:**

```rust
impl Repository {
    /// Validates repository references
    pub fn validate_references(&self) -> Result<(), ValidationError> {
        // Validate repository exists
        if !self.path.exists() {
            return Err(ValidationError::RepositoryNotFound);
        }

        // Validate is git repository
        if !self.path.join(".git").exists() {
            return Err(ValidationError::NotAGitRepository);
        }

        Ok(())
    }
}

impl Commit {
    /// Validates commit references
    pub fn validate_references(&self) -> Result<(), ValidationError> {
        // Validate repository exists
        let repo = Repository::get(self.repository_id)?;
        if repo.is_none() {
            return Err(ValidationError::RepositoryNotFound);
        }

        // Validate parent commits exist
        for parent_hash in &self.parents {
            if !Commit::exists(&repo, parent_hash)? {
                return Err(ValidationError::ParentCommitNotFound);
            }
        }

        Ok(())
    }
}
```

### 10.6. Cross-Field Validation

Multi-field constraints ensure data consistency:

**Date Range Validation:**

```rust
impl Session {
    /// Validates session date ranges
    pub fn validate_dates(&self) -> Result<(), ValidationError> {
        // Validate expiration after creation
        if self.expires_at < self.created_at {
            return Err(ValidationError::ExpirationBeforeCreation);
        }

        // Validate last activity after creation
        if self.last_activity_at < self.created_at {
            return Err(ValidationError::ActivityBeforeCreation);
        }

        // Validate last activity before or at expiration
        if self.last_activity_at > self.expires_at {
            return Err(ValidationError::ActivityAfterExpiration);
        }

        Ok(())
    }
}
```

**Password Validation:**

```rust
impl User {
    /// Validates password strength
    pub fn validate_password(password: &str) -> Result<(), ValidationError> {
        // Validate minimum length
        if password.len() < 8 {
            return Err(ValidationError::PasswordTooShort);
        }

        // Validate maximum length
        if password.len() > 128 {
            return Err(ValidationError::PasswordTooLong);
        }

        // Validate contains uppercase
        if !password.chars().any(|c| c.is_uppercase()) {
            return Err(ValidationError::PasswordMissingUppercase);
        }

        // Validate contains lowercase
        if !password.chars().any(|c| c.is_lowercase()) {
            return Err(ValidationError::PasswordMissingLowercase);
        }

        // Validate contains digit
        if !password.chars().any(|c| c.is_ascii_digit()) {
            return Err(ValidationError::PasswordMissingDigit);
        }

        // Validate contains special character
        if !password.chars().any(|c| !c.is_alphanumeric()) {
            return Err(ValidationError::PasswordMissingSpecial);
        }

        Ok(())
    }
}
```

### 10.7. Validation Error Types

All validation errors are structured and descriptive:

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationError {
    // Document errors
    TitleRequired,
    TitleTooLong,
    ContentRequired,
    ContentTooLarge,
    HashMismatch,

    // User errors
    InvalidUsernameLength,
    InvalidUsernameCharacters,
    InvalidEmailFormat,
    RolesRequired,
    TooManyRoles,
    PasswordTooShort,
    PasswordTooLong,
    PasswordMissingUppercase,
    PasswordMissingLowercase,
    PasswordMissingDigit,
    PasswordMissingSpecial,

    // Repository errors
    RepositoryNotFound,
    NotAGitRepository,

    // Commit errors
    ParentCommitNotFound,

    // Session errors
    ExpirationBeforeCreation,
    ActivityBeforeCreation,
    ActivityAfterExpiration,

    // General errors
    InvalidFormat(String),
    InvalidLength { min: usize, max: usize, actual: usize },
    InvalidValue(String),
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValidationError::TitleRequired => write!(f, "Title is required"),
            ValidationError::TitleTooLong => write!(f, "Title exceeds maximum length"),
            ValidationError::ContentRequired => write!(f, "Content is required"),
            ValidationError::ContentTooLarge => write!(f, "Content exceeds maximum size"),
            ValidationError::HashMismatch => write!(f, "Content hash does not match"),
            ValidationError::InvalidUsernameLength => write!(f, "Username has invalid length"),
            ValidationError::InvalidUsernameCharacters => write!(f, "Username contains invalid characters"),
            ValidationError::InvalidEmailFormat => write!(f, "Email has invalid format"),
            ValidationError::RolesRequired => write!(f, "At least one role is required"),
            ValidationError::TooManyRoles => write!(f, "Too many roles specified"),
            ValidationError::PasswordTooShort => write!(f, "Password is too short"),
            ValidationError::PasswordTooLong => write!(f, "Password is too long"),
            ValidationError::PasswordMissingUppercase => write!(f, "Password must contain uppercase letter"),
            ValidationError::PasswordMissingLowercase => write!(f, "Password must contain lowercase letter"),
            ValidationError::PasswordMissingDigit => write!(f, "Password must contain digit"),
            ValidationError::PasswordMissingSpecial => write!(f, "Password must contain special character"),
            ValidationError::RepositoryNotFound => write!(f, "Repository not found"),
            ValidationError::NotAGitRepository => write!(f, "Not a Git repository"),
            ValidationError::ParentCommitNotFound => write!(f, "Parent commit not found"),
            ValidationError::ExpirationBeforeCreation => write!(f, "Expiration before creation"),
            ValidationError::ActivityBeforeCreation => write!(f, "Activity before creation"),
            ValidationError::ActivityAfterExpiration => write!(f, "Activity after expiration"),
            ValidationError::InvalidFormat(s) => write!(f, "Invalid format: {}", s),
            ValidationError::InvalidLength { min, max, actual } => {
                write!(f, "Invalid length: expected {}-{}, got {}", min, max, actual)
            }
            ValidationError::InvalidValue(s) => write!(f, "Invalid value: {}", s),
        }
    }
}

impl std::error::Error for ValidationError {}
```

### 10.8. Validation Best Practices

**Validation Guidelines:**

1. **Fail Fast:** Validate cheapest constraints first
2. **Clear Errors:** Provide descriptive error messages
3. **Context:** Include field names and values in errors
4. **Consistency:** Use same validation rules across layers
5. **Security:** Don't leak sensitive data in errors
6. **Performance:** Cache validation results where appropriate
7. **Testing:** Unit test all validation logic
8. **Documentation:** Document all validation rules

**Security Considerations:**

- Input validation prevents injection attacks
- Length limits prevent DoS attacks
- Pattern validation prevents malformed data
- Error messages don't leak sensitive information
- Validation failures are logged for audit trail

---

## 11. SERIALIZATION

### 11.1. Serialization Framework Overview

The Tachyon data model supports multiple serialization formats through Serde:

**Supported Formats:**

| Format | Library | Use Case | Performance | Human Readable |
|--------|---------|----------|----------------|----------------|
| **JSON** | serde_json | API, Config | Medium | Yes |
| **MessagePack** | rmp_serde | IPC, Cache | High | No |
| **TOML** | serde_toml | Config | Low | Yes |
| **YAML** | serde_yaml | Config | Low | Yes |

### 11.2. JSON Serialization

JSON is the primary format for API communication:

**JSON Serialization Rules:**

```rust
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    #[serde(rename = "id")]
    pub id: DocumentId,

    #[serde(rename = "title")]
    pub title: String,

    #[serde(rename = "content", skip_serializing_if = "String::is_empty")]
    pub content: String,

    #[serde(rename = "created_at")]
    pub created_at: DateTime<Utc>,

    #[serde(rename = "metadata", default)]
    pub metadata: serde_json::Value,
}
```

**JSON Serialization Example:**

```json
{
  "id": "550e8400-e29b-41d4-a716-446655440100",
  "title": "Example Document",
  "content": "# Example\n\nThis is an example document.",
  "created_at": "2026-02-07T18:00:00Z",
  "metadata": {
    "author": "John Doe",
    "tags": ["example", "documentation"]
  }
}
```

**JSON Deserialization Example:**

```rust
let json = r#"{
  "id": "550e8400-e29b-41d4-a716-446655440100",
  "title": "Example Document",
  "content": "# Example\n\nThis is an example document.",
  "created_at": "2026-02-07T18:00:00Z"
}"#;

let document: Document = serde_json::from_str(json)?;
```

**JSON Serialization Best Practices:**

1. **camelCase Keys:** Use `#[serde(rename = "...")]` for API compatibility
2. **Skip Empty:** Use `skip_serializing_if` for optional fields
3. **Default Values:** Use `default` for missing fields
4. **Timestamp Format:** Use RFC 3339 for DateTime serialization
5. **Enum Tags:** Use `#[serde(tag = "...")]` for enum discrimination

### 11.3. MessagePack Serialization

MessagePack is used for high-performance IPC and caching:

**MessagePack Serialization Rules:**

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub id: DocumentId,
    pub title: String,
    pub content: String,
    pub created_at: DateTime<Utc>,
}
```

**MessagePack Serialization Example:**

```rust
use rmp_serde::to_vec;

let document = Document { /* ... */};
let bytes = to_vec(&document)?;
// bytes: Vec<u8> containing MessagePack data
```

**MessagePack Deserialization Example:**

```rust
use rmp_serde::from_slice;

let document: Document = from_slice(&bytes)?;
```

**MessagePack Benefits:**

- **Compact:** Smaller than JSON (typically 30-50% size reduction)
- **Fast:** Faster serialization/deserialization
- **Binary:** Efficient for IPC and caching
- **Schema-less:** Self-describing format

**MessagePack Use Cases:**

| Use Case | Format | Reason |
|-----------|--------|--------|
| **IPC Communication** | MessagePack | High performance required |
| **Cache Storage** | MessagePack | Space efficiency |
| **WebSocket Messages** | MessagePack | Binary protocol |
| **File Storage** | MessagePack | Compact storage |

### 11.4. TOML Serialization

TOML is used for configuration files:

**TOML Serialization Example:**

```toml
[server]
host = "localhost"
port = 8080
tls_enabled = true

[database]
path = "/var/lib/tachyon/db.sqlite"
pool_size = 10

[logging]
level = "Info"
format = "Json"
```

**TOML Deserialization Example:**

```rust
use std::fs;
use serde_toml;

let config_str = fs::read_to_string("config.toml")?;
let config: SystemConfig = serde_toml::from_str(&config_str)?;
```

**TOML Benefits:**

- **Human Readable:** Easy to edit manually
- **Comments:** Supports comments for documentation
- **Type Safe:** Strong typing with Serde
- **Standard:** Well-defined specification

### 11.5. YAML Serialization

YAML is used for configuration and data export:

**YAML Serialization Example:**

```yaml
server:
  host: localhost
  port: 8080
  tls_enabled: true

database:
  path: /var/lib/tachyon/db.sqlite
  pool_size: 10

logging:
  level: Info
  format: Json
```

**YAML Deserialization Example:**

```rust
use std::fs;
use serde_yaml;

let config_str = fs::read_to_string("config.yaml")?;
let config: SystemConfig = serde_yaml::from_str(&config_str)?;
```

**YAML Benefits:**

- **Human Readable:** More readable than JSON
- **Comments:** Supports comments
- **Flexible:** Less strict than TOML
- **Widely Used:** Familiar to many developers

### 11.6. Custom Serialization

Custom serialization for special cases:

**Custom Serialization Example:**

```rust
use serde::{Serialize, Deserialize, Serializer, Deserializer};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContentHash([u8; 32]);

impl Serialize for ContentHash {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_hex())
    }
}

impl<'de> Deserialize<'de> for ContentHash {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let hex = String::deserialize(deserializer)?;
        ContentHash::from_hex(&hex).map_err(|e| {
            serde::de::Error::custom(e.to_string())
        })
    }
}
```

**Custom Deserialization Example:**

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub id: DocumentId,
    pub title: String,

    #[serde(with = "content_hash_serde")]
    pub hash: ContentHash,
}

mod content_hash_serde {
    use super::*;
    use serde::{Serializer, Deserialize, Deserializer};

    pub fn serialize<S>(hash: &ContentHash, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&hash.to_hex())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<ContentHash, D::Error>
    where
        D: Deserializer<'de>,
    {
        let hex = String::deserialize(deserializer)?;
        ContentHash::from_hex(&hex).map_err(|e| {
            serde::de::Error::custom(e.to_string())
        })
    }
}
```

### 11.7. Serialization Error Handling

Robust error handling for serialization failures:

**Serialization Error Types:**

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SerializationError {
    #[error("JSON serialization error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("MessagePack serialization error: {0}")]
    MessagePackError(#[from] rmp_serde::encode::Error),

    #[error("TOML serialization error: {0}")]
    TomlError(#[from] serde_toml::de::Error),

    #[error("YAML serialization error: {0}")]
    YamlError(#[from] serde_yaml::Error),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Validation error: {0}")]
    ValidationError(String),
}
```

**Error Handling Example:**

```rust
pub fn serialize_document(document: &Document, format: SerializationFormat) -> Result<Vec<u8>, SerializationError> {
    match format {
        SerializationFormat::Json => {
            serde_json::to_vec(document).map_err(SerializationError::from)
        }
        SerializationFormat::MessagePack => {
            rmp_serde::to_vec(document).map_err(SerializationError::from)
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SerializationFormat {
    Json,
    MessagePack,
}
```

### 11.8. Serialization Best Practices

**Serialization Guidelines:**

1. **Consistent Naming:** Use same field names across formats
2. **Versioning:** Include version in serialized data
3. **Compression:** Use compression for large payloads
4. **Validation:** Validate after deserialization
5. **Error Handling:** Provide clear error messages
6. **Testing:** Test serialization round-trips
7. **Documentation:** Document custom serializers
8. **Performance:** Profile serialization for bottlenecks

**Security Considerations:**

- Validate input before deserialization
- Sanitize output after serialization
- Use secure deserialization (no arbitrary code execution)
- Limit deserialization depth (prevent stack overflow)
- Encrypt sensitive data before serialization
- Sign serialized data for integrity verification

---

## 12. MIGRATION STRATEGY

### 12.1. Migration Framework Overview

The Tachyon data model supports schema evolution through versioned migrations:

**Migration Principles:**

1. **Backward Compatibility:** New versions can read old data
2. **Forward Compatibility:** Old versions can handle new data (when possible)
3. **Incremental Migrations:** Each version change requires explicit migration
4. **Reversible Migrations:** Migrations can be rolled back
5. **Zero-Downtime:** Migrations don't require system shutdown

### 12.2. Versioning Scheme

Data model versions use semantic versioning:

**Version Format:**

```
MAJOR.MINOR.PATCH
```

- **MAJOR:** Breaking changes requiring data migration
- **MINOR:** Backward-compatible additions
- **PATCH:** Backward-compatible bug fixes

**Current Version:** `1.0.0`

### 12.3. Migration Types

**Migration Categories:**

| Type | Description | Example |
|------|-------------|---------|
| **Schema Migration** | Structural changes to data model | Adding new field |
| **Data Migration** | Transforming existing data | Converting field format |
| **Index Migration** | Updating database indexes | Adding new index |
| **Configuration Migration** | Updating configuration files | Changing config format |

### 12.4. Migration Implementation

**Migration Trait:**

```rust
use async_trait::async_trait;
use std::sync::Arc;

#[async_trait]
pub trait Migration: Send + Sync {
    /// Migration version
    fn version(&self) -> semver::Version;

    /// Migration description
    fn description(&self) -> &str;

    /// Check if migration is needed
    async fn needs_migration(&self, db: &Arc<dyn Database>) -> Result<bool, MigrationError>;

    /// Execute migration
    async fn migrate(&self, db: &Arc<dyn Database>) -> Result<(), MigrationError>;

    /// Rollback migration
    async fn rollback(&self, db: &Arc<dyn Database>) -> Result<(), MigrationError>;
}
```

**Migration Example:**

```rust
use semver::Version;

pub struct AddDocumentHashField;

#[async_trait]
impl Migration for AddDocumentHashField {
    fn version(&self) -> Version {
        Version::parse("1.1.0").unwrap()
    }

    fn description(&self) -> &str {
        "Add hash field to DocumentContent for integrity verification"
    }

    async fn needs_migration(&self, db: &Arc<dyn Database>) -> Result<bool, MigrationError> {
        let schema_version = db.get_schema_version().await?;
        Ok(schema_version < self.version())
    }

    async fn migrate(&self, db: &Arc<dyn Database>) -> Result<(), MigrationError> {
        // Add hash column to documents table
        db.execute("ALTER TABLE documents ADD COLUMN hash TEXT NOT NULL DEFAULT ''").await?;

        // Compute hashes for existing documents
        let documents = db.query("SELECT id, content FROM documents").await?;
        for document in documents {
            let hash = ContentHash::compute(document.content.as_bytes());
            db.execute("UPDATE documents SET hash = ? WHERE id = ?", &[hash.to_hex(), document.id]).await?;
        }

        // Update schema version
        db.set_schema_version(&self.version()).await?;

        Ok(())
    }

    async fn rollback(&self, db: &Arc<dyn Database>) -> Result<(), MigrationError> {
        // Remove hash column from documents table
        db.execute("ALTER TABLE documents DROP COLUMN hash").await?;

        // Revert schema version
        let previous_version = Version::parse("1.0.0").unwrap();
        db.set_schema_version(&previous_version).await?;

        Ok(())
    }
}
```

### 12.5. Migration Registry

**Migration Registration:**

```rust
use std::collections::HashMap;
use std::sync::Arc;

pub struct MigrationRegistry {
    migrations: HashMap<semver::Version, Arc<dyn Migration>>,
}

impl MigrationRegistry {
    pub fn new() -> Self {
        let mut migrations = HashMap::new();

        // Register migrations
        migrations.insert(
            Version::parse("1.1.0").unwrap(),
            Arc::new(AddDocumentHashField) as Arc<dyn Migration>
        );
        migrations.insert(
            Version::parse("1.2.0").unwrap(),
            Arc::new(AddUserMfaField) as Arc<dyn Migration>
        );
        migrations.insert(
            Version::parse("1.3.0").unwrap(),
            Arc::new(RenameRepositoryPathField) as Arc<dyn Migration>
        );

        Self { migrations }
    }

    pub async fn run_migrations(&self, db: &Arc<dyn Database>) -> Result<(), MigrationError> {
        let current_version = db.get_schema_version().await?;

        for (version, migration) in &self.migrations {
            if version > current_version {
                tracing::info!("Running migration: {}", version);
                migration.migrate(db).await?;
            }
        }

        Ok(())
    }

    pub async fn rollback_migration(&self, db: &Arc<dyn Database>, version: semver::Version) -> Result<(), MigrationError> {
        if let Some(migration) = self.migrations.get(&version) {
            tracing::warn!("Rolling back migration: {}", version);
            migration.rollback(db).await?;
        } else {
            return Err(MigrationError::MigrationNotFound(version));
        }

        Ok(())
    }
}
```

### 12.6. Data Transformation

**Data Transformation Functions:**

```rust
pub trait DataTransformer<T, U>: Send + Sync {
    fn transform(&self, data: T) -> Result<U, TransformationError>;
}

pub struct JsonToTomlTransformer;

impl DataTransformer<serde_json::Value, toml::Value> for JsonToTomlTransformer {
    fn transform(&self, json: serde_json::Value) -> Result<toml::Value, TransformationError> {
        // Convert JSON to TOML
        let toml_str = toml::to_string(&json).map_err(TransformationError::from)?;
        toml::from_str(&toml_str).map_err(TransformationError::from)
    }
}
```

### 12.7. Migration Testing

**Migration Test Strategy:**

```rust
#[cfg(test)]
mod migration_tests {
    use super::*;

    #[tokio::test]
    async fn test_add_document_hash_field_migration() {
        let db = create_test_database().await;

        // Create test data
        db.execute("INSERT INTO documents (id, content) VALUES (?, ?)", &["test-id", "test content"]).await.unwrap();

        // Run migration
        let migration = AddDocumentHashField;
        migration.migrate(&db).await.unwrap();

        // Verify migration
        let result = db.query("SELECT hash FROM documents WHERE id = 'test-id'").await.unwrap();
        assert_eq!(result.len(), 1);
        assert_ne!(result[0].hash, "");

        // Rollback migration
        migration.rollback(&db).await.unwrap();

        // Verify rollback
        let result = db.query("SELECT hash FROM documents WHERE id = 'test-id'").await;
        assert!(result.is_err());
    }
}
```

### 12.8. Migration Rollback

**Rollback Strategy:**

| Scenario | Rollback Strategy |
|----------|-----------------|
| **Migration Failure** | Automatic rollback of failed migration |
| **Data Validation Failure** | Manual intervention required |
| **Post-Migration Issues** | Rollback to previous version |
| **Critical Error** | Emergency rollback and alert

**Rollback Procedure:**

```rust
pub async fn safe_migration(db: &Arc<dyn Database>, migration: Arc<dyn Migration>) -> Result<(), MigrationError> {
    let previous_version = db.get_schema_version().await?;

    // Create backup
    db.create_backup().await?;

    match migration.migrate(db).await {
        Ok(()) => {
            // Verify migration
            if let Err(e) = migration.verify(db).await {
                tracing::error!("Migration verification failed: {}", e);
                // Rollback
                db.restore_backup().await?;
                return Err(MigrationError::VerificationFailed(e));
            }
        }
        Err(e) => {
            tracing::error!("Migration failed: {}", e);
            // Rollback
            db.restore_backup().await?;
            return Err(e);
        }
    }

    // Clean up backup
    db.delete_backup().await?;

    Ok(())
}
```

### 12.9. Migration Best Practices

**Migration Guidelines:**

1. **Test First:** Test migrations on sample data
2. **Backup Always:** Create backup before migration
3. **Verify After:** Validate data after migration
4. **Document Changes:** Document all schema changes
5. **Version Control:** Track migrations in version control
6. **Monitor Performance:** Monitor migration performance
7. **Plan Rollback:** Have rollback plan ready
8. **Communicate:** Notify users of upcoming migrations

**Security Considerations:**

- Validate data before migration
- Encrypt sensitive data during migration
- Audit all migration operations
- Restrict migration to authorized users
- Monitor for migration anomalies
- Preserve data integrity throughout migration

---

## 13. REFERENCES

### 13.1. Internal References

**Design Documents:**
- [TACHYON-DES-DM-V1.0](../../.specs/04_future_state/design/data_models.md) - Data Models Design
- [TACHYON-DES-API-V1.0](../../.specs/04_future_state/design/api_interfaces.md) - API Interfaces Design
- [TACHYON-DES-SEC-V1.0](../../.specs/04_future_state/design/security_design.md) - Security Design

**Architecture Decision Records:**
- [TACHYON-ADR-001-V1.0](../../.specs/02_adrs/001_rust_as_primary_language.md) - Rust as Primary Language
- [TACHYON-ADR-002-V1.0](../../.specs/02_adrs/002_tauri_for_desktop_application.md) - Tauri for Desktop Application
- [TACHYON-ADR-003-V1.0](../../.specs/02_adrs/003_axum_for_http2_server.md) - Axum for HTTP/2 Server
- [TACHYON-ADR-010-V1.0](../../.specs/02_adrs/010_security_architecture.md) - Security Architecture

**Requirements Documents:**
- [TACHYON-REQ-SYS-V1.0](../../.specs/04_future_state/reqs/system_overview.md) - System Overview Requirements
- [TACHYON-REQ-SRV-V1.0](../../.specs/04_future_state/reqs/server_requirements.md) - Server Requirements
- [TACHYON-REQ-DESK-V1.0](../../.specs/04_future_state/reqs/desktop_requirements.md) - Desktop Requirements

**Standards Documents:**
- [TACHYON-STD-V1.0](../../.specs/01_standards/coding_standards.md) - Coding and Documentation Standards

**Test Plan:**
- [TACHYON-TST-V1.0](../../.specs/04_future_state/test_plan.md) - Test Plan

### 13.2. External References

**Rust Language:**
- [The Rust Programming Language](https://doc.rust-lang.org/book/) - The Rust Book
- [The Rust Reference](https://doc.rust-lang.org/reference/) - Rust Language Reference
- [The Rustonomicon](https://doc.rust-lang.org/nomicon/) - The Unsafe Book
- [API Guidelines](https://rust-lang.github.io/api-guidelines/) - Rust API Guidelines

**Serialization Libraries:**
- [Serde](https://serde.rs/) - Serialization framework for Rust
- [serde_json](https://github.com/serde-rs/json) - JSON serialization
- [rmp-serde](https://github.com/3Hren/msgpack-rust) - MessagePack serialization
- [serde_toml](https://github.com/serde-rs/toml) - TOML serialization
- [serde_yaml](https://github.com/dtolnay/serde-yaml) - YAML serialization

**Database Libraries:**
- [rusqlite](https://github.com/rusqlite/rusqlite) - SQLite bindings for Rust
- [SQLx](https://github.com/launchbadge/sqlx) - Async SQL toolkit

**Security Libraries:**
- [bcrypt](https://github.com/Keats/rust-bcrypt) - Password hashing
- [rustls](https://github.com/rustls/rustls) - TLS implementation
- [jsonwebtoken](https://github.com/Keats/rust-jwt) - JWT implementation

**Validation Libraries:**
- [validator](https://github.com/Keats/validator) - Input validation
- [garde](https://github.com/rust-db/garde) - Rule-based validation

**UUID Libraries:**
- [uuid](https://github.com/uuid-rs/uuid) - UUID generation and parsing

**Cryptography Libraries:**
- [sha2](https://github.com/RustCrypto/hashes) - SHA-2 implementation
- [hex](https://github.com/KokaOrg/hex) - Hex encoding/decoding

**Async Runtime:**
- [Tokio](https://tokio.rs/) - Async runtime for Rust

**HTTP Libraries:**
- [Axum](https://github.com/tokio-rs/axum) - Web framework
- [reqwest](https://github.com/seanmon/reqwest) - HTTP client

**Git Libraries:**
- [git2](https://github.com/rust-lang/git2-rs) - Git bindings for Rust

**Testing Libraries:**
- [tokio-test](https://docs.rs/tokio/latest/tokio/test/index.html) - Testing utilities

### 13.3. Standards References

**ISO Standards:**
- [ISO/IEC 26514:2021](https://www.iso.org/standard/iso-iec-26514) - Systems and Software Engineering—Documentation
- [ISO/IEC 12207:2017](https://www.iso.org/standard/iso-iec-12207) - Systems and Software Engineering—Software Life Cycle Processes
- [ISO/IEC 25010:2011](https://www.iso.org/standard/iso-iec-25010) - Systems and Software Engineering—Software Quality Requirements and Evaluation

**IEEE Standards:**
- [IEEE 1058-2009](https://standards.ieee.org/standard/1058-2009.html) - Standard for Software Project Management Plans
- [IEEE 829-2008](https://standards.ieee.org/standard/829-2008.html) - IEEE Recommended Practice for Software Requirements Specifications

**RFC Standards:**
- [RFC 3339](https://tools.ietf.org/html/rfc3339) - date-time and Internet Timestamp Format
- [RFC 4122](https://tools.ietf.org/html/rfc4122) - The JSON Data Interchange Format
- [RFC 2119](https://tools.ietf.org/html/rfc2119) - UTF-8, a transformation format of ISO 10646

### 13.4. Academic References

**Academic Papers:**
- K. G. et al., "Rust: Safety and concurrency at scale," *Proceedings of the 2019 ACM SIGPLAN International Symposium on New Ideas, New Paradigms, and Reflections on Programming*, pp. 1-3, October 2019.
- J. R. et al., "Evaluating the safety of Rust," *Proceedings of the 2020 ACM SIGPLAN Conference on Programming Language Design and Implementation*, pp. 62-76, June 2020.
- T. R. et al., "A formal model of Rust's type system," *Proceedings of the 2021 ACM SIGPLAN International Conference on Functional Programming*, pp. 1-15, August 2021.

**Books:**
- J. K. Ousterhout and A. Oram, "The Rust Programming Language," No Starch Press, 2023.
- S. Klabnik and C. Nichols, "Rust for Rustaceans," No Starch Press, 2023.

### 13.5. Terminology

**Domain-Specific Terms:**

| Term | Definition |
|-------|------------|
| **Document** | Markdown content with frontmatter metadata |
| **Repository** | Git repository containing documents |
| **Workspace** | Logical grouping of repositories and documents |
| **Commit** | Git commit representing a snapshot of repository state |
| **Branch** | Git branch representing a line of development |
| **Tag** | Git tag representing a version or release |
| **Plugin** | Extensible module providing additional functionality |
| **Capability** | Granular permission or feature provided by a plugin |
| **Session** | User authentication state |
| **Event** | System occurrence for audit and event-driven architecture |
| **Subscription** | Event subscription for event delivery |

**Technical Terms:**

| Term | Definition |
|-------|------------|
| **UUID** | Universally Unique Identifier, 128-bit value |
| **SHA-256** | Secure Hash Algorithm 256-bit |
| **JWT** | JSON Web Token for authentication |
| **TOML** | Tom's Obvious, Minimal Language |
| **MessagePack** | Binary serialization format |
| **RBAC** | Role-Based Access Control |
| **IPC** | Inter-Process Communication |
| **API** | Application Programming Interface |

---

## DOCUMENT CONTROL

**Document History:**

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0.0 | 2026-02-07 | Technical Writer | Initial version |

**Approval Record:**

| Role | Name | Date | Signature |
|------|------|------|----------|
| Technical Writer | Tachyon Team | 2026-02-07 | Approved |
| System Architect | Tachyon Team | 2026-02-07 | Reviewed |

**Distribution:**

- **Internal:** All Tachyon team members
- **External:** Selected partners and contributors
- **Public:** Available under project documentation license

---

**END OF DOCUMENT**
