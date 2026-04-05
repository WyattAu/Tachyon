# TACHYON: SEARCH API SPECIFICATION

**Document ID:** TACHYON-API-013-V1.0
**Date:** February 2026
**Status:** Proposed
**Classification:** API Specification
**Compliance Level:** ISO/IEC 26514:2021, IEEE 1063-2001
**Dependencies:** [TACHYON-STD-V1.0](../../.specs/01_standards/coding_standards.md), [TACHYON-REQ-SRV-V1.0](../../.specs/04_future_state/reqs/server_requirements.md), [TACHYON-DES-API-V1.0](../../.specs/04_future_state/design/api_interfaces.md)

---

## TABLE OF CONTENTS

1. [Introduction](#1-introduction)
2. [Search API Design Principles](#2-search-api-design-principles)
3. [Search Query API](#3-search-query-api)
4. [Search Filter API](#4-search-filter-api)
5. [Search Sort API](#5-search-sort-api)
6. [Search Autocomplete API](#6-search-autocomplete-api)
7. [Search Facets API](#7-search-facets-api)
8. [Search Performance](#8-search-performance)
9. [Search Security](#9-search-security)
10. [References](#10-references)

---

## 1. INTRODUCTION

### 1.1. Document Purpose

This document specifies the Search API for the Tachyon toolchain, providing comprehensive interfaces for full-text search, faceted filtering, autocomplete, and relevance ranking. The Search API enables efficient content discovery across documentation repositories with sub-100 millisecond response times for queries against 100,000 documents.

### 1.2. Scope

The Search API specification covers:
- Full-text search endpoints with relevance ranking
- Faceted search with tag, date, and author filtering
- Autocomplete suggestions for search queries
- Search result pagination and sorting
- Search highlighting and context extraction
- Performance requirements and caching strategies
- Security considerations for search operations

Out of scope:
- Search index implementation details (covered in design documents)
- Search query language syntax (covered in user documentation)
- Search analytics and reporting (covered in operations documentation)

### 1.3. Search Architecture Overview

The Tachyon Search API is built upon the Tantivy full-text search engine, providing inverted index-based search with BM25 relevance ranking. The search architecture supports:

- **Incremental Indexing:** Search index updates without full re-indexing
- **Real-time Synchronization:** Index synchronization with Git repository state
- **Access Control Filtering:** Search results filtered based on user permissions
- **Multi-field Search:** Search across title, content, tags, and metadata
- **Fuzzy Matching:** Configurable fuzzy search for typo tolerance

### 1.4. API Versioning

The Search API follows semantic versioning with URL-based versioning:
- Current version: `/api/v1/search`
- Version format: `/api/v{major}/search`
- Backward compatibility maintained within major versions
- Deprecated endpoints announced 90 days before removal

### 1.5. Related Requirements

| Requirement ID | Title | Priority |
|----------------|--------|-----------|
| REQ-SRV-026 | Full-Text Search | Critical |
| REQ-SRV-027 | Faceted Search | Medium |
| REQ-SRV-028 | Search Autocomplete | Medium |
| REQ-SRV-029 | Search Pagination | High |
| REQ-SRV-030 | Search Highlighting | Medium |
| REQ-SRV-056 | Tantivy Integration | Critical |
| REQ-SRV-057 | Incremental Indexing | High |
| REQ-SRV-058 | Index Synchronization | Critical |
| REQ-SRV-107 | Search Response Time | Critical |

---

## 2. SEARCH API DESIGN PRINCIPLES

### 2.1. Performance-First Design

The Search API prioritizes performance to meet sub-100 millisecond response time requirements:

- **Sub-100ms Latency:** Search queries return within 100ms for 100,000 documents
- **Efficient Indexing:** Incremental index updates minimize re-indexing overhead
- **Result Caching:** Frequently accessed search results cached for improved performance
- **Connection Pooling:** Database and index connections pooled for efficiency
- **Async Processing:** Non-blocking I/O using Tokio async runtime

### 2.2. Relevance Ranking

Search results are ranked using BM25 algorithm with configurable parameters:

- **BM25 Scoring:** Standard BM25 relevance ranking with term frequency and inverse document frequency
- **Field Boosting:** Configurable field weights (title, content, tags)
- **Phrase Proximity:** Proximity-based scoring for phrase queries
- **Freshness Boosting:** Optional time-based boosting for recent documents
- **Custom Scoring:** Extensible scoring functions for domain-specific relevance

### 2.3. Access Control Integration

Search API enforces access control at query time:

- **Permission Filtering:** Results filtered based on user's document access permissions
- **Frontmatter Enforcement:** Access control directives from document frontmatter respected
- **Internal Content Redaction:** `::: internal` blocks excluded from search index
- **RBAC Integration:** Role-based access control enforced for search operations
- **Audit Logging:** All search queries logged for security and analytics

### 2.4. RESTful Conventions

Search API follows RESTful conventions for consistency:

- **Resource-Oriented:** Search endpoints treat search as a resource
- **HTTP Methods:** GET for search queries, POST for complex queries
- **Status Codes:** Appropriate HTTP status codes for all responses
- **Content Negotiation:** JSON responses with proper content types
- **CORS Support:** Configurable CORS for cross-origin requests

### 2.5. Type Safety

Search API leverages Rust's type system for compile-time guarantees:

- **Strong Typing:** All request/response structures strongly typed
- **Validation:** Input validation at type level with serde deserialization
- **Error Handling:** Comprehensive error types with proper HTTP mapping
- **Null Safety:** Elimination of null pointer exceptions through Option<T> types
- **Immutable Data:** Default immutability for search results and configurations

### 2.6. Extensibility

Search API designed for extensibility:

- **Pluggable Scoring:** Custom scoring functions for domain-specific relevance
- **Field Addition:** Easy addition of new searchable fields
- **Filter Extension:** New filter types added without breaking changes
- **Sort Options:** Additional sort criteria supported via configuration
- **Plugin Architecture:** Optional plugin system for custom search features

---

## 3. SEARCH QUERY API

### 3.1. Full-Text Search Endpoint

#### API-SEARCH-001: Search Documents

**Element ID:** API-SEARCH-001
**Name:** GET /api/v1/search
**Type:** REST Endpoint
**Language:** Rust (Axum)
**Related Requirements:** REQ-SRV-026, REQ-SRV-056, REQ-SRV-107

**Description:** Performs full-text search across all indexed documents with relevance ranking, faceted filtering, and pagination. The endpoint supports complex query syntax including phrase search, boolean operators, and field-specific queries.

**Request Schema:**

```rust
use axum::extract::{Query, State};
use serde::Deserialize;
use std::collections::HashMap;

/// Search query parameters
#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    /// Search query string (required)
    /// Supports: phrase search, boolean operators, field-specific queries
    /// Maximum length: 1000 characters
    pub q: String,

    /// Facet filters (JSON-encoded)
    /// Format: {"tags": ["rust", "api"], "author": "john"}
    pub filters: Option<String>,

    /// Sort order (default: relevance)
    /// Options: relevance, date, title, size
    pub sort: Option<String>,

    /// Pagination offset (default: 0)
    /// Non-negative integer
    pub offset: Option<usize>,

    /// Page size (default: 20, max: 100)
    /// Range: 1-100 inclusive
    pub limit: Option<usize>,

    /// Fuzzy search enabled (default: false)
    /// Enables typo tolerance with edit distance 1
    pub fuzzy: Option<bool>,

    /// Highlight matches in results (default: true)
    /// Returns highlighted snippets with search terms
    pub highlight: Option<bool>,

    /// Fields to search (default: all fields)
    /// Format: comma-separated field names (title,content,tags)
    pub fields: Option<String>,

    /// Minimum score threshold (default: 0.0)
    /// Filters results below this relevance score
    pub min_score: Option<f32>,
}

pub async fn search_documents(
    Query(params): Query<SearchQuery>,
    State(user): State<AuthenticatedUser>,
) -> Result<Json<SearchResponse>, ApiError>;
```

**Response Schema:**

```rust
use serde::Serialize;

/// Search response containing results and metadata
#[derive(Debug, Serialize)]
pub struct SearchResponse {
    /// Search results ordered by relevance
    pub results: Vec<SearchResult>,

    /// Total matching documents
    pub total: usize,

    /// Query execution time in milliseconds
    pub query_time_ms: u64,

    /// Current pagination offset
    pub offset: usize,

    /// Current page size
    pub limit: usize,

    /// Facet counts for navigation
    pub facets: Option<FacetCounts>,

    /// Search query as executed
    pub query: String,
}

/// Individual search result
#[derive(Debug, Serialize)]
pub struct SearchResult {
    /// Document unique identifier
    pub id: DocumentId,

    /// Document title
    pub title: String,

    /// Document path
    pub path: String,

    /// Relevance score (BM25)
    pub score: f32,

    /// Highlighted snippets with search terms
    pub highlights: Vec<Highlight>,

    /// Document metadata
    pub metadata: DocumentMetadata,

    /// Matching fields
    pub matched_fields: Vec<String>,
}

/// Highlighted text snippet
#[derive(Debug, Serialize)]
pub struct Highlight {
    /// Field name containing the match
    pub field: String,

    /// Highlighted text fragment
    pub text: String,

    /// Match positions in the text
    pub positions: Vec<TextPosition>,
}

/// Text position for highlighting
#[derive(Debug, Serialize)]
pub struct TextPosition {
    /// Start position (0-indexed)
    pub start: usize,

    /// End position (exclusive)
    pub end: usize,
}

/// Facet counts for navigation
#[derive(Debug, Serialize)]
pub struct FacetCounts {
    /// Tag facet counts
    pub tags: HashMap<String, usize>,

    /// Author facet counts
    pub authors: HashMap<String, usize>,

    /// Date range facet counts
    pub date_ranges: HashMap<String, usize>,
}
```

**Query Syntax:**

The search endpoint supports the following query syntax:

| Syntax | Description | Example |
|---------|-------------|---------|
| **Phrase Search** | Exact phrase matching in quotes | `"API specification"` |
| **Boolean AND** | All terms must match | `rust AND async` |
| **Boolean OR** | Any term must match | `search OR query` |
| **Boolean NOT** | Term must not match | `rust NOT python` |
| **Field-Specific** | Search specific field | `title:API` |
| **Wildcard** | Prefix matching | `search*` |
| **Fuzzy** | Approximate matching | `search~` |
| **Proximity** | Terms within N words | `"API specification"~5` |
| **Boosting** | Term relevance boost | `rust^2 search` |

**Constraints:**

- `q`: Required, non-empty, 1-1000 characters
- `limit`: Optional, 1-100 inclusive, default 20
- `offset`: Optional, non-negative integer, default 0
- `sort`: Optional, must be one of `relevance`, `date`, `title`, `size`
- `fuzzy`: Optional, boolean, default false
- `highlight`: Optional, boolean, default true
- `min_score`: Optional, 0.0-1.0 inclusive, default 0.0

**Error Responses:**

| Status Code | Error Type | Description |
|-------------|-------------|-------------|
| 400 Bad Request | InvalidQuery | Query string is empty or exceeds maximum length |
| 400 Bad Request | InvalidLimit | Limit value outside valid range |
| 400 Bad Request | InvalidOffset | Negative offset value |
| 400 Bad Request | InvalidSort | Invalid sort option specified |
| 400 Bad Request | InvalidFilters | Malformed filter JSON |
| 401 Unauthorized | AuthenticationRequired | User not authenticated |
| 403 Forbidden | AccessDenied | User lacks search permissions |
| 429 Too Many Requests | RateLimitExceeded | Search rate limit exceeded |
| 500 Internal Server Error | SearchError | Internal search engine error |

**Performance Characteristics:**

- **Target Latency:** <100ms for queries against 100,000 documents
- **P99 Latency:** <200ms for 99th percentile of queries
- **Throughput:** 1000 queries/second on single server instance
- **Memory Usage:** <100MB per concurrent search operation

**Security Considerations:**

- Requires authentication for all search operations
- Filters results based on user's document access permissions
- Sanitizes query string to prevent injection attacks
- Logs all search queries for security audit
- Enforces rate limiting per user to prevent abuse
- Redacts internal content from search results

**Dependencies:**

- [ADR-001](../../.specs/02_adrs/001_rust_as_primary_language.md): Rust language for type-safe implementation
- [ADR-003](../../.specs/02_adrs/003_axum_for_http2_server.md): Axum framework for HTTP/2 server
- [ADR-007](../../.specs/02_adrs/007_tokio_for_async_runtime.md): Tokio async runtime for non-blocking I/O
- [TACHYON-REQ-SRV-V1.0](../../.specs/04_future_state/reqs/server_requirements.md): Server requirements for search functionality

**Example Request:**

```http
GET /api/v1/search?q=rust%20async&sort=relevance&limit=10&offset=0&fuzzy=true&highlight=true HTTP/1.1
Host: api.tachyon.example.com
Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...
Accept: application/json
```

**Example Response:**

```json
{
  "results": [
    {
      "id": "550e8400-e29b-41d4-a716-446655440000",
      "title": "Async Runtime Architecture",
      "path": "docs/architecture/async-runtime.md",
      "score": 2.847,
      "highlights": [
        {
          "field": "content",
          "text": "The <mark>async</mark> runtime provides efficient I/O handling",
          "positions": [
            { "start": 4, "end": 9 }
          ]
        }
      ],
      "metadata": {
        "author": "system-architect",
        "created_at": "2026-01-15T10:30:00Z",
        "updated_at": "2026-02-01T14:22:00Z",
        "tags": ["architecture", "rust", "async"]
      },
      "matched_fields": ["title", "content"]
    }
  ],
  "total": 42,
  "query_time_ms": 87,
  "offset": 0,
  "limit": 10,
  "facets": {
    "tags": {
      "architecture": 15,
      "rust": 28,
      "async": 12
    },
    "authors": {
      "system-architect": 8,
      "backend-developer": 18
    }
  },
  "query": "rust async"
}
```

---

### 3.2. Search Query Examples

#### Example 3.1: Simple Text Search

Search for documents containing "API specification":

```http
GET /api/v1/search?q=API%20specification HTTP/1.1
```

#### Example 3.2: Phrase Search

Search for exact phrase "full-text search":

```http
GET /api/v1/search?q="full-text%20search" HTTP/1.1
```

#### Example 3.3: Boolean Query

Search for documents containing "rust" AND "async" but NOT "python":

```http
GET /api/v1/search?q=rust%20AND%20async%20NOT%20python HTTP/1.1
```

#### Example 3.4: Field-Specific Search

Search for "API" in title field:

```http
GET /api/v1/search?q=title:API HTTP/1.1
```

#### Example 3.5: Fuzzy Search

Search with typo tolerance:

```http
GET /api/v1/search?q=serch&fuzzy=true HTTP/1.1
```

#### Example 3.6: Complex Query with Filters

Search for "rust" with tag filter and date sorting:

```http
GET /api/v1/search?q=rust&filters=%7B%22tags%22%3A%5B%22architecture%22%5D%7D&sort=date&limit=20 HTTP/1.1
```

---

### 3.3. Search Query Validation

The search endpoint performs comprehensive input validation:

**Query String Validation:**

1. **Length Check:** Query string must be 1-1000 characters
2. **Encoding Check:** Query string must be valid UTF-8
3. **Injection Check:** Query string sanitized for injection attacks
4. **Syntax Check:** Query syntax validated before execution

**Parameter Validation:**

1. **Type Validation:** All parameters validated against type definitions
2. **Range Validation:** Numeric parameters checked against valid ranges
3. **Enum Validation:** Enum parameters validated against allowed values
4. **JSON Validation:** Filter JSON validated for correct structure

**Response Validation:**

1. **Schema Validation:** Response validated against JSON schema
2. **Field Validation:** All required fields present and correctly typed
3. **Permission Validation:** Results filtered based on user permissions
4. **Sanitization Validation:** Output sanitized for XSS prevention

---

## 4. SEARCH FILTER API

### 4.1. Filter Types

The Search API supports multiple filter types for precise result refinement:

#### API-FILTER-001: Tag Filter

**Element ID:** API-FILTER-001
**Name:** Tag Facet Filter
**Type:** Query Parameter
**Related Requirements:** REQ-SRV-027

**Description:** Filters search results by document tags. Supports single tag, multiple tags with AND/OR logic, and tag exclusion.

**Filter Schema:**

```rust
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Tag filter configuration
#[derive(Debug, Deserialize, Serialize)]
pub struct TagFilter {
    /// Tags to include (AND logic by default)
    pub include: Option<Vec<String>>,

    /// Tags to include with OR logic
    pub include_any: Option<Vec<String>>,

    /// Tags to exclude
    pub exclude: Option<Vec<String>>,

    /// Minimum tag count required
    pub min_count: Option<usize>,
}

impl TagFilter {
    /// Validates tag filter configuration
    pub fn validate(&self) -> Result<(), FilterError> {
        // Validate tag count limits
        if let Some(ref tags) = self.include {
            if tags.len() > 50 {
                return Err(FilterError::TooManyTags(tags.len()));
            }
        }
        if let Some(ref tags) = self.include_any {
            if tags.len() > 50 {
                return Err(FilterError::TooManyTags(tags.len()));
            }
        }
        if let Some(ref tags) = self.exclude {
            if tags.len() > 50 {
                return Err(FilterError::TooManyTags(tags.len()));
            }
        }

        // Validate tag format
        let all_tags: Vec<&String> = self.include.iter()
            .chain(self.include_any.iter())
            .chain(self.exclude.iter())
            .flatten()
            .collect();

        for tag in all_tags {
            if tag.len() > 64 {
                return Err(FilterError::TagTooLong(tag.clone()));
            }
            if !is_valid_tag_name(tag) {
                return Err(FilterError::InvalidTag(tag.clone()));
            }
        }

        Ok(())
    }
}

/// Validates tag name format
fn is_valid_tag_name(tag: &str) -> bool {
    // Tags must be alphanumeric with hyphens and underscores
    tag.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_')
        && !tag.is_empty()
}
```

**Filter Examples:**

| Filter | Description | Example |
|---------|-------------|---------|
| **Single Tag** | Include documents with specific tag | `{"include": ["rust"]}` |
| **Multiple Tags (AND)** | Include documents with all specified tags | `{"include": ["rust", "api"]}` |
| **Multiple Tags (OR)** | Include documents with any specified tag | `{"include_any": ["rust", "python"]}` |
| **Tag Exclusion** | Exclude documents with specified tag | `{"exclude": ["draft"]}` |
| **Combined** | Complex filter with include and exclude | `{"include": ["rust"], "exclude": ["internal"]}` |

**Constraints:**

- `include`: Maximum 50 tags, each max 64 characters
- `include_any`: Maximum 50 tags, each max 64 characters
- `exclude`: Maximum 50 tags, each max 64 characters
- `min_count`: Non-negative integer, default 1

---

#### API-FILTER-002: Date Range Filter

**Element ID:** API-FILTER-002
**Name:** Date Range Filter
**Type:** Query Parameter
**Related Requirements:** REQ-SRV-027

**Description:** Filters search results by document creation or update date. Supports absolute dates, relative dates, and predefined ranges.

**Filter Schema:**

```rust
use chrono::{DateTime, Utc, Duration};
use serde::{Deserialize, Serialize};

/// Date range filter configuration
#[derive(Debug, Deserialize, Serialize)]
pub struct DateRangeFilter {
    /// Field to filter (created_at, updated_at)
    pub field: DateField,

    /// Start date (inclusive)
    pub start: Option<DateTime<Utc>>,

    /// End date (inclusive)
    pub end: Option<DateTime<Utc>>,

    /// Relative start date (e.g., "-7d", "-1m")
    pub start_relative: Option<String>,

    /// Relative end date (e.g., "now", "+1d")
    pub end_relative: Option<String>,

    /// Predefined range (today, week, month, year)
    pub range: Option<PredefinedRange>,
}

/// Date field to filter
#[derive(Debug, Deserialize, Serialize, Clone, Copy)]
pub enum DateField {
    #[serde(rename = "created_at")]
    CreatedAt,
    #[serde(rename = "updated_at")]
    UpdatedAt,
}

/// Predefined date ranges
#[derive(Debug, Deserialize, Serialize, Clone, Copy)]
pub enum PredefinedRange {
    #[serde(rename = "today")]
    Today,
    #[serde(rename = "week")]
    ThisWeek,
    #[serde(rename = "month")]
    ThisMonth,
    #[serde(rename = "year")]
    ThisYear,
}

impl DateRangeFilter {
    /// Resolves date range to absolute dates
    pub fn resolve(&self) -> Result<(Option<DateTime<Utc>>, Option<DateTime<Utc>>), FilterError> {
        let start = self.resolve_start()?;
        let end = self.resolve_end()?;
        Ok((start, end))
    }

    /// Resolves start date
    fn resolve_start(&self) -> Result<Option<DateTime<Utc>>, FilterError> {
        if let Some(start) = self.start {
            return Ok(Some(start));
        }
        if let Some(ref relative) = self.start_relative {
            return parse_relative_date(relative);
        }
        if let Some(range) = self.range {
            return Ok(Some(range.start_date()));
        }
        Ok(None)
    }

    /// Resolves end date
    fn resolve_end(&self) -> Result<Option<DateTime<Utc>>, FilterError> {
        if let Some(end) = self.end {
            return Ok(Some(end));
        }
        if let Some(ref relative) = self.end_relative {
            return parse_relative_date(relative);
        }
        if let Some(range) = self.range {
            return Ok(Some(range.end_date()));
        }
        Ok(None)
    }
}

/// Parses relative date string
fn parse_relative_date(s: &str) -> Result<Option<DateTime<Utc>>, FilterError> {
    let now = Utc::now();
    
    if s == "now" {
        return Ok(Some(now));
    }

    let (sign, num_str, unit) = parse_relative_string(s)?;
    let num: i64 = num_str.parse()
        .map_err(|_| FilterError::InvalidRelativeDate(s.to_string()))?;

    let duration = match unit {
        "s" => Duration::seconds(num),
        "m" => Duration::minutes(num),
        "h" => Duration::hours(num),
        "d" => Duration::days(num),
        "w" => Duration::weeks(num),
        "M" => Duration::days(num * 30),
        "y" => Duration::days(num * 365),
        _ => return Err(FilterError::InvalidRelativeDate(s.to_string())),
    };

    Ok(Some(if sign > 0 { now + duration } else { now - duration }))
}

impl PredefinedRange {
    /// Returns start date for predefined range
    fn start_date(&self) -> DateTime<Utc> {
        let now = Utc::now();
        match self {
            PredefinedRange::Today => {
                now.date_naive().and_hms_opt(0, 0, 0)
                    .unwrap()
                    .and_utc()
            }
            PredefinedRange::ThisWeek => {
                let weekday = now.weekday().num_days_from_monday();
                (now - Duration::days(weekday as i64))
                    .date_naive()
                    .and_hms_opt(0, 0, 0)
                    .unwrap()
                    .and_utc()
            }
            PredefinedRange::ThisMonth => {
                now.with_day(1).unwrap()
                    .with_hour(0).unwrap()
                    .with_minute(0).unwrap()
                    .with_second(0).unwrap()
                    .with_nanosecond(0).unwrap()
            }
            PredefinedRange::ThisYear => {
                now.with_month(1).unwrap()
                    .with_day(1).unwrap()
                    .with_hour(0).unwrap()
                    .with_minute(0).unwrap()
                    .with_second(0).unwrap()
                    .with_nanosecond(0).unwrap()
            }
        }
    }

    /// Returns end date for predefined range
    fn end_date(&self) -> DateTime<Utc> {
        let now = Utc::now();
        match self {
            PredefinedRange::Today => {
                now.date_naive().and_hms_opt(23, 59, 59)
                    .unwrap()
                    .and_utc()
            }
            PredefinedRange::ThisWeek => {
                let weekday = 6 - now.weekday().num_days_from_sunday();
                (now + Duration::days(weekday as i64))
                    .date_naive()
                    .and_hms_opt(23, 59, 59)
                    .unwrap()
                    .and_utc()
            }
            PredefinedRange::ThisMonth => {
                now.with_day(1).unwrap()
                    .with_month((now.month() % 12) + 1).unwrap()
                    .with_hour(23).unwrap()
                    .with_minute(59).unwrap()
                    .with_second(59).unwrap()
                    .with_nanosecond(0).unwrap()
            }
            PredefinedRange::ThisYear => {
                now.with_year(now.year() + 1).unwrap()
                    .with_month(1).unwrap()
                    .with_day(1).unwrap()
                    .with_hour(0).unwrap()
                    .with_minute(0).unwrap()
                    .with_second(0).unwrap()
                    .with_nanosecond(0).unwrap()
            }
        }
    }
}
```

**Filter Examples:**

| Filter | Description | Example |
|---------|-------------|---------|
| **Absolute Range** | Specific date range | `{"field": "created_at", "start": "2026-01-01T00:00:00Z", "end": "2026-01-31T23:59:59Z"}` |
| **Relative Range** | Documents from last 7 days | `{"field": "created_at", "start_relative": "-7d", "end_relative": "now"}` |
| **Predefined Range** | Documents from this month | `{"field": "updated_at", "range": "month"}` |
| **Updated Recently** | Documents updated in last 30 days | `{"field": "updated_at", "start_relative": "-30d"}` |

**Constraints:**

- `field`: Required, must be `created_at` or `updated_at`
- `start`: Optional, valid ISO 8601 datetime
- `end`: Optional, valid ISO 8601 datetime
- `start_relative`: Optional, format `(+|-)N[unit]` where unit is s, m, h, d, w, M, y
- `end_relative`: Optional, same format as `start_relative`
- `range`: Optional, must be one of `today`, `week`, `month`, `year`

---

#### API-FILTER-003: Author Filter

**Element ID:** API-FILTER-003
**Name:** Author Filter
**Type:** Query Parameter
**Related Requirements:** REQ-SRV-027

**Description:** Filters search results by document author. Supports single author, multiple authors, and author exclusion.

**Filter Schema:**

```rust
use serde::{Deserialize, Serialize};

/// Author filter configuration
#[derive(Debug, Deserialize, Serialize)]
pub struct AuthorFilter {
    /// Authors to include (AND logic by default)
    pub include: Option<Vec<String>>,

    /// Authors to include with OR logic
    pub include_any: Option<Vec<String>>,

    /// Authors to exclude
    pub exclude: Option<Vec<String>>,
}

impl AuthorFilter {
    /// Validates author filter configuration
    pub fn validate(&self) -> Result<(), FilterError> {
        let total_count = self.include.as_ref().map(|v| v.len()).unwrap_or(0)
            + self.include_any.as_ref().map(|v| v.len()).unwrap_or(0)
            + self.exclude.as_ref().map(|v| v.len()).unwrap_or(0);

        if total_count > 100 {
            return Err(FilterError::TooManyAuthors(total_count));
        }

        // Validate author format
        let all_authors: Vec<&String> = self.include.iter()
            .chain(self.include_any.iter())
            .chain(self.exclude.iter())
            .flatten()
            .collect();

        for author in all_authors {
            if author.len() > 128 {
                return Err(FilterError::AuthorTooLong(author.clone()));
            }
            if author.trim().is_empty() {
                return Err(FilterError::EmptyAuthor);
            }
        }

        Ok(())
    }
}
```

**Filter Examples:**

| Filter | Description | Example |
|---------|-------------|---------|
| **Single Author** | Documents by specific author | `{"include": ["john.doe"]}` |
| **Multiple Authors (AND)** | Documents by all specified authors | `{"include": ["john.doe", "jane.smith"]}` |
| **Multiple Authors (OR)** | Documents by any specified author | `{"include_any": ["john.doe", "jane.smith"]}` |
| **Author Exclusion** | Exclude documents by author | `{"exclude": ["bot"]}` |

**Constraints:**

- `include`: Maximum 100 authors, each max 128 characters
- `include_any`: Maximum 100 authors, each max 128 characters
- `exclude`: Maximum 100 authors, each max 128 characters

---

#### API-FILTER-004: Path Filter

**Element ID:** API-FILTER-004
**Name:** Path Filter
**Type:** Query Parameter
**Related Requirements:** REQ-SRV-027

**Description:** Filters search results by document path. Supports prefix matching, suffix matching, and glob patterns.

**Filter Schema:**

```rust
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Path filter configuration
#[derive(Debug, Deserialize, Serialize)]
pub struct PathFilter {
    /// Path prefix to match
    pub prefix: Option<String>,

    /// Path suffix to match
    pub suffix: Option<String>,

    /// Glob pattern to match
    pub glob: Option<String>,

    /// Exact path to match
    pub exact: Option<String>,

    /// Paths to exclude
    pub exclude: Option<Vec<String>>,
}

impl PathFilter {
    /// Validates path filter configuration
    pub fn validate(&self) -> Result<(), FilterError> {
        // Validate path length
        if let Some(ref prefix) = self.prefix {
            if prefix.len() > 1024 {
                return Err(FilterError::PathTooLong(prefix.clone()));
            }
        }
        if let Some(ref suffix) = self.suffix {
            if suffix.len() > 1024 {
                return Err(FilterError::PathTooLong(suffix.clone()));
            }
        }
        if let Some(ref glob) = self.glob {
            if glob.len() > 1024 {
                return Err(FilterError::PathTooLong(glob.clone()));
            }
            // Validate glob pattern
            if let Err(e) = glob::Pattern::new(glob) {
                return Err(FilterError::InvalidGlob(e.to_string()));
            }
        }
        if let Some(ref exact) = self.exact {
            if exact.len() > 1024 {
                return Err(FilterError::PathTooLong(exact.clone()));
            }
        }

        // Validate exclude paths
        if let Some(ref exclude) = self.exclude {
            if exclude.len() > 100 {
                return Err(FilterError::TooManyPaths(exclude.len()));
            }
            for path in exclude {
                if path.len() > 1024 {
                    return Err(FilterError::PathTooLong(path.clone()));
                }
            }
        }

        Ok(())
    }

    /// Checks if path matches filter
    pub fn matches(&self, path: &str) -> bool {
        // Check exclusions first
        if let Some(ref exclude) = self.exclude {
            for pattern in exclude {
                if self.path_matches_pattern(path, pattern) {
                    return false;
                }
            }
        }

        // Check inclusion criteria
        if let Some(ref exact) = self.exact {
            return path == exact;
        }
        if let Some(ref prefix) = self.prefix {
            if !path.starts_with(prefix) {
                return false;
            }
        }
        if let Some(ref suffix) = self.suffix {
            if !path.ends_with(suffix) {
                return false;
            }
        }
        if let Some(ref glob) = self.glob {
            if let Ok(pattern) = glob::Pattern::new(glob) {
                return pattern.matches(path);
            }
        }

        true
    }

    /// Checks if path matches pattern
    fn path_matches_pattern(&self, path: &str, pattern: &str) -> bool {
        // Simple prefix/suffix matching
        if pattern.ends_with('*') {
            let prefix = &pattern[..pattern.len()-1];
            return path.starts_with(prefix);
        }
        if pattern.starts_with('*') {
            let suffix = &pattern[1..];
            return path.ends_with(suffix);
        }
        
        // Exact match
        path == pattern
    }
}
```

**Filter Examples:**

| Filter | Description | Example |
|---------|-------------|---------|
| **Path Prefix** | Documents in specific directory | `{"prefix": "docs/api/"}` |
| **Path Suffix** | Documents with specific extension | `{"suffix": ".md"}` |
| **Glob Pattern** | Documents matching pattern | `{"glob": "docs/**/*.md"}` |
| **Exact Path** | Specific document | `{"exact": "docs/api/search.md"}` |
| **Exclude Paths** | Exclude specific paths | `{"prefix": "docs/", "exclude": ["docs/internal/", "docs/drafts/"]}` |

**Constraints:**

- `prefix`: Optional, max 1024 characters
- `suffix`: Optional, max 1024 characters
- `glob`: Optional, max 1024 characters, valid glob pattern
- `exact`: Optional, max 1024 characters
- `exclude`: Optional, max 100 paths, each max 1024 characters

---

### 4.2. Combined Filter Schema

All filter types can be combined in a single filter object:

```rust
use serde::{Deserialize, Serialize};

/// Combined search filters
#[derive(Debug, Deserialize, Serialize)]
pub struct SearchFilters {
    /// Tag filter
    pub tags: Option<TagFilter>,

    /// Date range filter
    pub date_range: Option<DateRangeFilter>,

    /// Author filter
    pub author: Option<AuthorFilter>,

    /// Path filter
    pub path: Option<PathFilter>,
}

impl SearchFilters {
    /// Validates all filters
    pub fn validate(&self) -> Result<(), FilterError> {
        if let Some(ref tags) = self.tags {
            tags.validate()?;
        }
        if let Some(ref author) = self.author {
            author.validate()?;
        }
        if let Some(ref path) = self.path {
            path.validate()?;
        }
        Ok(())
    }

    /// Checks if document matches all filters
    pub fn matches(&self, document: &Document) -> bool {
        if let Some(ref tags) = self.tags {
            if !tags.matches(document) {
                return false;
            }
        }
        if let Some(ref date_range) = self.date_range {
            if !date_range.matches(document) {
                return false;
            }
        }
        if let Some(ref author) = self.author {
            if !author.matches(document) {
                return false;
            }
        }
        if let Some(ref path) = self.path {
            if !path.matches(&document.path) {
                return false;
            }
        }
        true
    }
}
```

**Combined Filter Example:**

```json
{
  "tags": {
    "include": ["rust", "api"],
    "exclude": ["draft"]
  },
  "date_range": {
    "field": "updated_at",
    "start_relative": "-30d",
    "end_relative": "now"
  },
  "author": {
    "include": ["john.doe"]
  },
  "path": {
    "prefix": "docs/",
    "exclude": ["docs/internal/"]
  }
}
```

---

### 4.3. Filter Error Handling

**Error Types:**

```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum FilterError {
    #[error("Too many tags: {0} (maximum 50)")]
    TooManyTags(usize),

    #[error("Tag too long: {0} (maximum 64 characters)")]
    TagTooLong(String),

    #[error("Invalid tag name: {0}")]
    InvalidTag(String),

    #[error("Too many authors: {0} (maximum 100)")]
    TooManyAuthors(usize),

    #[error("Author too long: {0} (maximum 128 characters)")]
    AuthorTooLong(String),

    #[error("Empty author name")]
    EmptyAuthor,

    #[error("Path too long: {0} (maximum 1024 characters)")]
    PathTooLong(String),

    #[error("Too many paths: {0} (maximum 100)")]
    TooManyPaths(usize),

    #[error("Invalid glob pattern: {0}")]
    InvalidGlob(String),

    #[error("Invalid relative date: {0}")]
    InvalidRelativeDate(String),

    #[error("Invalid date format: {0}")]
    InvalidDateFormat(String),
}
```

**HTTP Error Mapping:**

| Error Type | HTTP Status | Error Code |
|------------|--------------|-------------|
| TooManyTags | 400 Bad Request | FILTER_TOO_MANY_TAGS |
| TagTooLong | 400 Bad Request | FILTER_TAG_TOO_LONG |
| InvalidTag | 400 Bad Request | FILTER_INVALID_TAG |
| TooManyAuthors | 400 Bad Request | FILTER_TOO_MANY_AUTHORS |
| AuthorTooLong | 400 Bad Request | FILTER_AUTHOR_TOO_LONG |
| EmptyAuthor | 400 Bad Request | FILTER_EMPTY_AUTHOR |
| PathTooLong | 400 Bad Request | FILTER_PATH_TOO_LONG |
| TooManyPaths | 400 Bad Request | FILTER_TOO_MANY_PATHS |
| InvalidGlob | 400 Bad Request | FILTER_INVALID_GLOB |
| InvalidRelativeDate | 400 Bad Request | FILTER_INVALID_DATE |
| InvalidDateFormat | 400 Bad Request | FILTER_DATE_FORMAT |

---

## 5. SEARCH SORT API

### 5.1. Sort Options

The Search API supports multiple sort options for result ordering:

#### API-SORT-001: Relevance Sort

**Element ID:** API-SORT-001
**Name:** Relevance Sort
**Type:** Query Parameter
**Related Requirements:** REQ-SRV-026

**Description:** Sorts search results by BM25 relevance score. This is the default sort option when no sort parameter is specified.

**Sort Schema:**

```rust
use serde::{Deserialize, Serialize};

/// Relevance sort configuration
#[derive(Debug, Deserialize, Serialize, Clone, Copy)]
pub struct RelevanceSort {
    /// Field weights for relevance scoring
    pub field_weights: Option<FieldWeights>,

    /// Freshness boost factor (0.0-1.0)
    pub freshness_boost: Option<f32>,

    /// Custom scoring function
    pub custom_score: Option<String>,
}

/// Field weights for relevance scoring
#[derive(Debug, Deserialize, Serialize)]
pub struct FieldWeights {
    /// Title field weight (default: 2.0)
    pub title: Option<f32>,

    /// Content field weight (default: 1.0)
    pub content: Option<f32>,

    /// Tags field weight (default: 1.5)
    pub tags: Option<f32>,

    /// Author field weight (default: 0.5)
    pub author: Option<f32>,
}

impl RelevanceSort {
    /// Validates relevance sort configuration
    pub fn validate(&self) -> Result<(), SortError> {
        if let Some(ref weights) = self.field_weights {
            weights.validate()?;
        }
        if let Some(boost) = self.freshness_boost {
            if boost < 0.0 || boost > 1.0 {
                return Err(SortError::InvalidFreshnessBoost(boost));
            }
        }
        Ok(())
    }
}

impl FieldWeights {
    /// Validates field weights
    pub fn validate(&self) -> Result<(), SortError> {
        let weights = [
            self.title.unwrap_or(2.0),
            self.content.unwrap_or(1.0),
            self.tags.unwrap_or(1.5),
            self.author.unwrap_or(0.5),
        ];

        for weight in weights {
            if weight < 0.0 {
                return Err(SortError::NegativeWeight(weight));
            }
            if weight > 10.0 {
                return Err(SortError::WeightTooLarge(weight));
            }
        }

        Ok(())
    }
}
```

**Sort Examples:**

| Sort | Description | Example |
|------|-------------|---------|
| **Default Relevance** | Default BM25 scoring | `sort=relevance` |
| **Custom Weights** | Custom field weights | `sort=relevance&field_weights={"title":3.0,"content":1.0}` |
| **Freshness Boost** | Boost recent documents | `sort=relevance&freshness_boost=0.5` |

**Constraints:**

- `field_weights.title`: Optional, 0.0-10.0 inclusive, default 2.0
- `field_weights.content`: Optional, 0.0-10.0 inclusive, default 1.0
- `field_weights.tags`: Optional, 0.0-10.0 inclusive, default 1.5
- `field_weights.author`: Optional, 0.0-10.0 inclusive, default 0.5
- `freshness_boost`: Optional, 0.0-1.0 inclusive, default 0.0

---

#### API-SORT-002: Date Sort

**Element ID:** API-SORT-002
**Name:** Date Sort
**Type:** Query Parameter
**Related Requirements:** REQ-SRV-027

**Description:** Sorts search results by document creation or update date. Supports ascending and descending order.

**Sort Schema:**

```rust
use serde::{Deserialize, Serialize};

/// Date sort configuration
#[derive(Debug, Deserialize, Serialize, Clone, Copy)]
pub struct DateSort {
    /// Date field to sort by (created_at, updated_at)
    pub field: DateField,

    /// Sort order (asc, desc)
    pub order: SortOrder,
}

/// Date field for sorting
#[derive(Debug, Deserialize, Serialize, Clone, Copy)]
pub enum DateField {
    #[serde(rename = "created_at")]
    CreatedAt,
    #[serde(rename = "updated_at")]
    UpdatedAt,
}

/// Sort order
#[derive(Debug, Deserialize, Serialize, Clone, Copy)]
pub enum SortOrder {
    #[serde(rename = "asc")]
    Ascending,
    #[serde(rename = "desc")]
    Descending,
}

impl DateSort {
    /// Validates date sort configuration
    pub fn validate(&self) -> Result<(), SortError> {
        // All enum variants are valid by construction
        Ok(())
    }
}
```

**Sort Examples:**

| Sort | Description | Example |
|------|-------------|---------|
| **Created Date (Newest)** | Newest documents first | `sort=date&field=created_at&order=desc` |
| **Created Date (Oldest)** | Oldest documents first | `sort=date&field=created_at&order=asc` |
| **Updated Date (Newest)** | Recently updated first | `sort=date&field=updated_at&order=desc` |
| **Updated Date (Oldest)** | Least recently updated first | `sort=date&field=updated_at&order=asc` |

**Constraints:**

- `field`: Required, must be `created_at` or `updated_at`
- `order`: Required, must be `asc` or `desc`

---

#### API-SORT-003: Title Sort

**Element ID:** API-SORT-003
**Name:** Title Sort
**Type:** Query Parameter
**Related Requirements:** REQ-SRV-027

**Description:** Sorts search results by document title alphabetically. Supports ascending and descending order with case-insensitive comparison.

**Sort Schema:**

```rust
use serde::{Deserialize, Serialize};

/// Title sort configuration
#[derive(Debug, Deserialize, Serialize, Clone, Copy)]
pub struct TitleSort {
    /// Sort order (asc, desc)
    pub order: SortOrder,

    /// Case sensitive comparison (default: false)
    pub case_sensitive: Option<bool>,
}

impl TitleSort {
    /// Validates title sort configuration
    pub fn validate(&self) -> Result<(), SortError> {
        // All enum variants are valid by construction
        Ok(())
    }
}
```

**Sort Examples:**

| Sort | Description | Example |
|------|-------------|---------|
| **Title (A-Z)** | Alphabetical ascending | `sort=title&order=asc` |
| **Title (Z-A)** | Alphabetical descending | `sort=title&order=desc` |
| **Title (Case-Sensitive)** | Case-sensitive sort | `sort=title&order=asc&case_sensitive=true` |

**Constraints:**

- `order`: Required, must be `asc` or `desc`
- `case_sensitive`: Optional, boolean, default false

---

#### API-SORT-004: Size Sort

**Element ID:** API-SORT-004
**Name:** Size Sort
**Type:** Query Parameter
**Related Requirements:** REQ-SRV-027

**Description:** Sorts search results by document size (character count or byte size). Supports ascending and descending order.

**Sort Schema:**

```rust
use serde::{Deserialize, Serialize};

/// Size sort configuration
#[derive(Debug, Deserialize, Serialize, Clone, Copy)]
pub struct SizeSort {
    /// Size metric (characters, bytes)
    pub metric: SizeMetric,

    /// Sort order (asc, desc)
    pub order: SortOrder,
}

/// Size metric for sorting
#[derive(Debug, Deserialize, Serialize, Clone, Copy)]
pub enum SizeMetric {
    #[serde(rename = "characters")]
    Characters,
    #[serde(rename = "bytes")]
    Bytes,
}

impl SizeSort {
    /// Validates size sort configuration
    pub fn validate(&self) -> Result<(), SortError> {
        // All enum variants are valid by construction
        Ok(())
    }
}
```

**Sort Examples:**

| Sort | Description | Example |
|------|-------------|---------|
| **Size (Smallest)** | Smallest documents first | `sort=size&metric=characters&order=asc` |
| **Size (Largest)** | Largest documents first | `sort=size&metric=characters&order=desc` |
| **Byte Size (Smallest)** | Smallest byte size first | `sort=size&metric=bytes&order=asc` |

**Constraints:**

- `metric`: Required, must be `characters` or `bytes`
- `order`: Required, must be `asc` or `desc`

---

### 5.2. Combined Sort Schema

Sort options can be combined with primary and secondary sort criteria:

```rust
use serde::{Deserialize, Serialize};

/// Combined sort configuration
#[derive(Debug, Deserialize, Serialize)]
pub struct SearchSort {
    /// Primary sort criterion
    pub primary: SortCriterion,

    /// Secondary sort criterion (optional)
    pub secondary: Option<SortCriterion>,

    /// Tertiary sort criterion (optional)
    pub tertiary: Option<SortCriterion>,
}

/// Sort criterion
#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type", content = "config")]
pub enum SortCriterion {
    #[serde(rename = "relevance")]
    Relevance(RelevanceSort),

    #[serde(rename = "date")]
    Date(DateSort),

    #[serde(rename = "title")]
    Title(TitleSort),

    #[serde(rename = "size")]
    Size(SizeSort),
}

impl SearchSort {
    /// Validates all sort criteria
    pub fn validate(&self) -> Result<(), SortError> {
        self.primary.validate()?;
        if let Some(ref secondary) = self.secondary {
            secondary.validate()?;
        }
        if let Some(ref tertiary) = self.tertiary {
            tertiary.validate()?;
        }
        Ok(())
    }
}
```

**Combined Sort Example:**

```json
{
  "primary": {
    "type": "relevance",
    "config": {
      "field_weights": {
        "title": 3.0,
        "content": 1.0
      }
    }
  },
  "secondary": {
    "type": "date",
    "config": {
      "field": "created_at",
      "order": "desc"
    }
  },
  "tertiary": {
    "type": "title",
    "config": {
      "order": "asc"
    }
  }
}
```

**Sort Behavior:**

1. **Primary Sort:** Results are sorted by primary criterion
2. **Secondary Sort:** Results with equal primary scores are sorted by secondary criterion
3. **Tertiary Sort:** Results with equal primary and secondary scores are sorted by tertiary criterion
4. **Stable Sort:** Sort is stable, preserving original order for equal values

---

### 5.3. Sort Error Handling

**Error Types:**

```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SortError {
    #[error("Invalid sort type: {0}")]
    InvalidSortType(String),

    #[error("Invalid sort order: {0}")]
    InvalidSortOrder(String),

    #[error("Negative weight: {0}")]
    NegativeWeight(f32),

    #[error("Weight too large: {0} (maximum 10.0)")]
    WeightTooLarge(f32),

    #[error("Invalid freshness boost: {0} (range 0.0-1.0)")]
    InvalidFreshnessBoost(f32),

    #[error("Invalid size metric: {0}")]
    InvalidSizeMetric(String),

    #[error("Invalid date field: {0}")]
    InvalidDateField(String),
}
```

**HTTP Error Mapping:**

| Error Type | HTTP Status | Error Code |
|------------|--------------|-------------|
| InvalidSortType | 400 Bad Request | SORT_INVALID_TYPE |
| InvalidSortOrder | 400 Bad Request | SORT_INVALID_ORDER |
| NegativeWeight | 400 Bad Request | SORT_NEGATIVE_WEIGHT |
| WeightTooLarge | 400 Bad Request | SORT_WEIGHT_TOO_LARGE |
| InvalidFreshnessBoost | 400 Bad Request | SORT_INVALID_BOOST |
| InvalidSizeMetric | 400 Bad Request | SORT_INVALID_METRIC |
| InvalidDateField | 400 Bad Request | SORT_INVALID_FIELD |

---

### 5.4. Sort Performance Considerations

**Performance Impact by Sort Type:**

| Sort Type | Performance Impact | Notes |
|------------|-------------------|-------|
| **Relevance** | Low | Pre-computed BM25 scores, fast sorting |
| **Date** | Low | Numeric comparison, very fast |
| **Title** | Medium | String comparison, moderate overhead |
| **Size** | Low | Numeric comparison, very fast |
| **Combined** | Medium | Multiple sort passes, moderate overhead |

**Optimization Strategies:**

1. **Index Pre-Sorting:** Maintain pre-sorted indexes for common sort criteria
2. **Score Caching:** Cache relevance scores for repeated queries
3. **Lazy Evaluation:** Sort only visible page of results
4. **Parallel Sorting:** Use parallel sorting for large result sets
5. **Sort Hinting:** Provide sort hints to search engine for optimization

---

## 6. SEARCH AUTOCOMPLETE API

### 6.1. Autocomplete Endpoint

#### API-AUTO-001: Search Autocomplete

**Element ID:** API-AUTO-001
**Name:** GET /api/v1/search/autocomplete
**Type:** REST Endpoint
**Language:** Rust (Axum)
**Related Requirements:** REQ-SRV-028

**Description:** Provides real-time search suggestions as user types. Autocomplete supports multiple suggestion types including titles, tags, paths, and phrases. Suggestions are ranked by relevance and filtered based on user permissions.

**Request Schema:**

```rust
use axum::extract::{Query, State};
use serde::Deserialize;

/// Autocomplete query parameters
#[derive(Debug, Deserialize)]
pub struct AutocompleteQuery {
    /// Partial query string (required)
    /// Minimum 1 character, maximum 100 characters
    pub q: String,

    /// Maximum suggestions (default: 10, max: 20)
    pub limit: Option<usize>,

    /// Suggestion types to include
    /// Options: title, tag, path, phrase, all
    pub types: Option<String>,

    /// Fuzzy matching enabled (default: false)
    pub fuzzy: Option<bool>,

    /// Minimum suggestion length
    pub min_length: Option<usize>,
}

pub async fn search_autocomplete(
    Query(params): Query<AutocompleteQuery>,
    State(user): State<AuthenticatedUser>,
) -> Result<Json<AutocompleteResponse>, ApiError>;
```

**Response Schema:**

```rust
use serde::Serialize;

/// Autocomplete response
#[derive(Debug, Serialize)]
pub struct AutocompleteResponse {
    /// Suggestion list ordered by relevance
    pub suggestions: Vec<Suggestion>,

    /// Query execution time in milliseconds
    pub query_time_ms: u64,
}

/// Individual suggestion
#[derive(Debug, Serialize)]
pub struct Suggestion {
    /// Suggestion text
    pub text: String,

    /// Suggestion type
    #[serde(rename = "type")]
    pub suggestion_type: SuggestionType,

    /// Document ID (if applicable)
    pub document_id: Option<DocumentId>,

    /// Relevance score
    pub score: f32,

    /// Match highlight positions
    pub highlights: Option<Vec<TextPosition>>,
}

/// Suggestion type
#[derive(Debug, Serialize, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum SuggestionType {
    /// Document title suggestion
    Title,

    /// Tag suggestion
    Tag,

    /// Document path suggestion
    Path,

    /// Phrase suggestion from content
    Phrase,

    /// Author suggestion
    Author,
}

/// Text position for highlighting
#[derive(Debug, Serialize)]
pub struct TextPosition {
    /// Start position (0-indexed)
    pub start: usize,

    /// End position (exclusive)
    pub end: usize,
}
```

**Suggestion Types:**

| Type | Description | Example |
|------|-------------|---------|
| **Title** | Document titles matching query | "API Specification" |
| **Tag** | Tags matching query | "rust" |
| **Path** | Document paths matching query | "docs/api/search.md" |
| **Phrase** | Common phrases from content | "full-text search" |
| **Author** | Author names matching query | "john.doe" |

**Constraints:**

- `q`: Required, 1-100 characters
- `limit`: Optional, 1-20 inclusive, default 10
- `types`: Optional, comma-separated list (title,tag,path,phrase,author,all), default all
- `fuzzy`: Optional, boolean, default false
- `min_length`: Optional, 1-50 inclusive, default 1

**Error Responses:**

| Status Code | Error Type | Description |
|-------------|-------------|-------------|
| 400 Bad Request | InvalidQuery | Query string is empty or exceeds maximum length |
| 400 Bad Request | InvalidLimit | Limit value outside valid range |
| 400 Bad Request | InvalidTypes | Invalid suggestion types specified |
| 401 Unauthorized | AuthenticationRequired | User not authenticated |
| 403 Forbidden | AccessDenied | User lacks search permissions |
| 429 Too Many Requests | RateLimitExceeded | Autocomplete rate limit exceeded |
| 500 Internal Server Error | AutocompleteError | Internal autocomplete error |

**Performance Characteristics:**

- **Target Latency:** <50ms for autocomplete queries
- **P99 Latency:** <100ms for 99th percentile of queries
- **Throughput:** 2000 queries/second on single server instance
- **Memory Usage:** <50MB per concurrent autocomplete operation

**Security Considerations:**

- Requires authentication for all autocomplete operations
- Filters suggestions based on user's document access permissions
- Sanitizes query string to prevent injection attacks
- Limits suggestion count to prevent information leakage
- Logs all autocomplete queries for security audit
- Enforces rate limiting per user to prevent abuse

---

### 6.2. Autocomplete Examples

#### Example 6.1: Title Autocomplete

Autocomplete document titles starting with "API":

```http
GET /api/v1/search/autocomplete?q=API&types=title&limit=10 HTTP/1.1
```

**Response:**

```json
{
  "suggestions": [
    {
      "text": "API Specification",
      "type": "title",
      "document_id": "550e8400-e29b-41d4-a716-446655440000",
      "score": 0.95,
      "highlights": [
        { "start": 0, "end": 3 }
      ]
    },
    {
      "text": "API Reference",
      "type": "title",
      "document_id": "660e8400-e29b-41d4-a716-4466554440001",
      "score": 0.87,
      "highlights": [
        { "start": 0, "end": 3 }
      ]
    }
  ],
  "query_time_ms": 23
}
```

#### Example 6.2: Tag Autocomplete

Autocomplete tags starting with "rust":

```http
GET /api/v1/search/autocomplete?q=rust&types=tag&limit=10 HTTP/1.1
```

**Response:**

```json
{
  "suggestions": [
    {
      "text": "rust",
      "type": "tag",
      "score": 1.0
    },
    {
      "text": "rust-async",
      "type": "tag",
      "score": 0.92
    }
  ],
  "query_time_ms": 18
}
```

#### Example 6.3: Fuzzy Autocomplete

Autocomplete with typo tolerance:

```http
GET /api/v1/search/autocomplete?q=serch&fuzzy=true&limit=10 HTTP/1.1
```

**Response:**

```json
{
  "suggestions": [
    {
      "text": "search",
      "type": "title",
      "document_id": "770e8400-e29b-41d4-a716-4466554550002",
      "score": 0.85,
      "highlights": [
        { "start": 0, "end": 6 }
      ]
    }
  ],
  "query_time_ms": 31
}
```

#### Example 6.4: Multi-Type Autocomplete

Autocomplete across all suggestion types:

```http
GET /api/v1/search/autocomplete?q=async&types=all&limit=15 HTTP/1.1
```

**Response:**

```json
{
  "suggestions": [
    {
      "text": "Async Runtime",
      "type": "title",
      "document_id": "880e8400-e29b-41d4-a716-4466554660003",
      "score": 0.94,
      "highlights": [
        { "start": 0, "end": 5 }
      ]
    },
    {
      "text": "async",
      "type": "tag",
      "score": 0.88
    },
    {
      "text": "docs/architecture/async-runtime.md",
      "type": "path",
      "score": 0.82
    }
  ],
  "query_time_ms": 27
}
```

---

### 6.3. Autocomplete Algorithm

The autocomplete API uses a prefix-based matching algorithm with fuzzy matching support:

**Matching Algorithm:**

1. **Prefix Matching:** Exact prefix match for high relevance
2. **Fuzzy Matching:** Edit distance calculation for typo tolerance
3. **Relevance Scoring:** Combination of match quality and document relevance
4. **Type Prioritization:** Title suggestions prioritized over other types
5. **Permission Filtering:** Results filtered based on user permissions

**Scoring Formula:**

$$
\text{Score} = w_1 \times \text{MatchQuality} + w_2 \times \text{DocumentRelevance} + w_3 \times \text{TypePriority}
$$

Where:
- $\text{MatchQuality}$: 1.0 for exact match, 0.5 for fuzzy match
- $\text{DocumentRelevance}$: BM25 score of document
- $\text{TypePriority}$: 1.0 for title, 0.8 for tag, 0.6 for path, 0.4 for phrase
- $w_1, w_2, w_3$: Weight coefficients (configurable)

**Performance Optimization:**

1. **Prefix Index:** Maintain prefix index for fast prefix lookups
2. **Result Caching:** Cache autocomplete results for common queries
3. **Limit Enforcement:** Enforce result limits at index level
4. **Parallel Matching:** Parallel matching across suggestion types
5. **Lazy Evaluation:** Evaluate suggestions only as needed for display

---

### 6.4. Autocomplete Error Handling

**Error Types:**

```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AutocompleteError {
    #[error("Query too short: {0} (minimum 1 character)")]
    QueryTooShort(usize),

    #[error("Query too long: {0} (maximum 100 characters)")]
    QueryTooLong(usize),

    #[error("Invalid limit: {0} (range 1-20)")]
    InvalidLimit(usize),

    #[error("Invalid suggestion types: {0}")]
    InvalidTypes(String),

    #[error("Autocomplete index not available")]
    IndexUnavailable,

    #[error("Autocomplete service overloaded")]
    ServiceOverloaded,
}
```

**HTTP Error Mapping:**

| Error Type | HTTP Status | Error Code |
|------------|--------------|-------------|
| QueryTooShort | 400 Bad Request | AUTO_QUERY_TOO_SHORT |
| QueryTooLong | 400 Bad Request | AUTO_QUERY_TOO_LONG |
| InvalidLimit | 400 Bad Request | AUTO_INVALID_LIMIT |
| InvalidTypes | 400 Bad Request | AUTO_INVALID_TYPES |
| IndexUnavailable | 503 Service Unavailable | AUTO_INDEX_UNAVAILABLE |
| ServiceOverloaded | 503 Service Unavailable | AUTO_SERVICE_OVERLOADED |

---

### 6.5. Autocomplete Rate Limiting

**Rate Limit Configuration:**

```rust
use std::time::Duration;

/// Autocomplete rate limit configuration
#[derive(Debug, Clone)]
pub struct AutocompleteRateLimit {
    /// Maximum requests per minute
    pub requests_per_minute: u32,

    /// Maximum requests per hour
    pub requests_per_hour: u32,

    /// Burst allowance
    pub burst_allowance: u32,
}

impl Default for AutocompleteRateLimit {
    fn default() -> Self {
        Self {
            requests_per_minute: 60,
            requests_per_hour: 1000,
            burst_allowance: 10,
        }
    }
}
```

**Rate Limit Enforcement:**

1. **Token Bucket Algorithm:** Use token bucket for rate limiting
2. **Per-User Limits:** Enforce limits per authenticated user
3. **Sliding Window:** Use sliding window for accurate rate tracking
4. **Headers Included:** Include rate limit headers in responses
5. **Graceful Degradation:** Return cached results when rate limited

**Rate Limit Headers:**

| Header | Description | Example |
|---------|-------------|---------|
| X-RateLimit-Limit | Requests per time window | `60` |
| X-RateLimit-Remaining | Remaining requests | `45` |
| X-RateLimit-Reset | Reset time (Unix timestamp) | `1707140800` |
| Retry-After | Seconds until retry allowed | `30` |

---

## 7. SEARCH FACETS API

### 7.1. Facet Overview

Faceted search enables users to refine search results by filtering on predefined categories (facets). The Search API supports faceted search for tags, authors, date ranges, and document types. Facets are computed dynamically based on search results and include counts for each facet value.

**Facet Types:**

| Facet Type | Description | Example Values |
|------------|-------------|---------------|
| **Tags** | Document tags | rust, api, architecture |
| **Authors** | Document authors | john.doe, jane.smith |
| **Date Ranges** | Creation/update date buckets | today, week, month, year |
| **Document Types** | Document type categories | markdown, html, pdf |
| **Paths** | Document path hierarchy | docs/api/, docs/guides/ |
| **Status** | Document status | published, draft, archived |

---

### 7.2. Facet Configuration

#### API-FACET-001: Facet Configuration

**Element ID:** API-FACET-001
**Name:** Facet Configuration
**Type:** Query Parameter
**Related Requirements:** REQ-SRV-027

**Description:** Configures which facets to include in search response and how to compute facet counts.

**Configuration Schema:**

```rust
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Facet configuration
#[derive(Debug, Deserialize, Serialize)]
pub struct FacetConfig {
    /// Facets to include (default: all)
    pub facets: Option<HashSet<FacetType>>,

    /// Maximum facet values per facet (default: 20)
    pub max_values: Option<usize>,

    /// Minimum count for facet inclusion (default: 1)
    pub min_count: Option<usize>,

    /// Sort facet values by count or name
    pub sort_by: Option<FacetSortOrder>,
}

/// Facet type
#[derive(Debug, Deserialize, Serialize, Clone, Copy, Hash, Eq)]
#[serde(rename_all = "lowercase")]
pub enum FacetType {
    /// Tag facets
    Tags,

    /// Author facets
    Authors,

    /// Date range facets
    DateRanges,

    /// Document type facets
    DocumentTypes,

    /// Path facets
    Paths,

    /// Status facets
    Status,
}

/// Facet sort order
#[derive(Debug, Deserialize, Serialize, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum FacetSortOrder {
    /// Sort by count (descending)
    Count,

    /// Sort by name (ascending)
    Name,
}

impl FacetConfig {
    /// Validates facet configuration
    pub fn validate(&self) -> Result<(), FacetError> {
        if let Some(max_values) = self.max_values {
            if max_values == 0 {
                return Err(FacetError::InvalidMaxValues(max_values));
            }
            if max_values > 100 {
                return Err(FacetError::MaxValuesTooLarge(max_values));
            }
        }
        if let Some(min_count) = self.min_count {
            if min_count == 0 {
                return Err(FacetError::InvalidMinCount(min_count));
            }
        }
        Ok(())
    }

    /// Returns default facet configuration
    pub fn default() -> Self {
        Self {
            facets: None, // Include all facets
            max_values: Some(20),
            min_count: Some(1),
            sort_by: Some(FacetSortOrder::Count),
        }
    }
}
```

**Configuration Examples:**

| Configuration | Description | Example |
|-------------|-------------|---------|
| **All Facets** | Include all facet types | `facets=all` |
| **Specific Facets** | Include only specified facets | `facets=tags,authors` |
| **Custom Limits** | Custom facet value limits | `max_values=50&min_count=2` |
| **Name Sort** | Sort facets by name | `sort_by=name` |

**Constraints:**

- `facets`: Optional, subset of facet types, default all
- `max_values`: Optional, 1-100 inclusive, default 20
- `min_count`: Optional, 1-1000 inclusive, default 1
- `sort_by`: Optional, must be `count` or `name`, default `count`

---

### 7.3. Facet Response Schema

**Facet Response Structure:**

```rust
use serde::Serialize;
use std::collections::HashMap;

/// Facet counts response
#[derive(Debug, Serialize)]
pub struct FacetCounts {
    /// Tag facet counts
    pub tags: HashMap<String, usize>,

    /// Author facet counts
    pub authors: HashMap<String, usize>,

    /// Date range facet counts
    pub date_ranges: HashMap<String, usize>,

    /// Document type facet counts
    pub document_types: HashMap<String, usize>,

    /// Path facet counts
    pub paths: HashMap<String, usize>,

    /// Status facet counts
    pub status: HashMap<String, usize>,
}

/// Individual facet value
#[derive(Debug, Serialize)]
pub struct FacetValue {
    /// Facet value
    pub value: String,

    /// Document count for this facet value
    pub count: usize,

    /// Selected state (if facet is active filter)
    pub selected: bool,
}
```

**Facet Response Example:**

```json
{
  "facets": {
    "tags": {
      "rust": 128,
      "api": 95,
      "architecture": 67,
      "async": 54
    },
    "authors": {
      "john.doe": 45,
      "jane.smith": 38,
      "system-architect": 22
    },
    "date_ranges": {
      "today": 12,
      "week": 34,
      "month": 89,
      "year": 241
    },
    "document_types": {
      "markdown": 287,
      "html": 15,
      "pdf": 8
    }
  }
}
```

---

### 7.4. Facet Computation

**Facet Counting Algorithm:**

1. **Result Aggregation:** Aggregate facet values from search results
2. **Count Calculation:** Count documents for each facet value
3. **Permission Filtering:** Exclude facet values for unauthorized documents
4. **Minimum Threshold:** Apply minimum count threshold
5. **Maximum Limitation:** Limit to top N facet values by count
6. **Sort Ordering:** Sort facet values by count or name

**Facet Computation Complexity:**

$$
O(n \times m)
$$

Where:
- $n$: Number of search results
- $m$: Number of facets

**Optimization Strategies:**

1. **Incremental Counting:** Update facet counts incrementally for pagination
2. **Parallel Aggregation:** Compute facets in parallel for large result sets
3. **Memoization:** Cache facet counts for repeated queries
4. **Lazy Evaluation:** Compute facets only when requested
5. **Index Pre-Computation:** Pre-compute facet counts for common queries

---

### 7.5. Facet Navigation

**Facet Selection Flow:**

```mermaid
graph LR
    A[Initial Search] --> B[Display Facets]
    B --> C[User Selects Facet]
    C --> D[Apply Facet Filter]
    D --> E[Refined Search]
    E --> F[Updated Facets]
    F --> C
```

**Facet Selection Behavior:**

1. **Facet Addition:** Adding a facet value narrows results
2. **Facet Removal:** Removing a facet value broadens results
3. **Multi-Select:** Multiple facet values can be selected per facet type
4. **Cross-Facet:** Facets of different types combine with AND logic
5. **Same-Facet:** Facet values of same type combine with OR logic

**Facet Selection Examples:**

| Selection | Behavior | Example |
|----------|-----------|---------|
| **Single Tag** | Documents with specific tag | `tags=rust` |
| **Multiple Tags** | Documents with any specified tag | `tags=rust|api` |
| **Tag + Author** | Documents with tag AND author | `tags=rust&authors=john.doe` |
| **Date Range + Type** | Documents in date range AND type | `date_ranges=month&document_types=markdown` |

---

### 7.6. Facet Error Handling

**Error Types:**

```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum FacetError {
    #[error("Invalid max values: {0} (range 1-100)")]
    InvalidMaxValues(usize),

    #[error("Max values too large: {0} (maximum 100)")]
    MaxValuesTooLarge(usize),

    #[error("Invalid min count: {0} (range 1-1000)")]
    InvalidMinCount(usize),

    #[error("Invalid facet type: {0}")]
    InvalidFacetType(String),

    #[error("Invalid sort order: {0}")]
    InvalidSortOrder(String),

    #[error("Facet computation failed")]
    ComputationFailed,

    #[error("Facet index not available")]
    IndexUnavailable,
}
```

**HTTP Error Mapping:**

| Error Type | HTTP Status | Error Code |
|------------|--------------|-------------|
| InvalidMaxValues | 400 Bad Request | FACET_INVALID_MAX_VALUES |
| MaxValuesTooLarge | 400 Bad Request | FACET_MAX_VALUES_TOO_LARGE |
| InvalidMinCount | 400 Bad Request | FACET_INVALID_MIN_COUNT |
| InvalidFacetType | 400 Bad Request | FACET_INVALID_TYPE |
| InvalidSortOrder | 400 Bad Request | FACET_INVALID_SORT |
| ComputationFailed | 500 Internal Server Error | FACET_COMPUTATION_FAILED |
| IndexUnavailable | 503 Service Unavailable | FACET_INDEX_UNAVAILABLE |

---

### 7.7. Facet Performance Considerations

**Performance Impact by Facet Count:**

| Facet Count | Performance Impact | Notes |
|-------------|-------------------|-------|
| **1-5 Facets** | Low | Minimal overhead |
| **6-10 Facets** | Medium | Moderate overhead |
| **11-20 Facets** | High | Significant overhead |
| **20+ Facets** | Very High | May impact query latency |

**Optimization Recommendations:**

1. **Facet Limiting:** Limit facets to top 10-20 most relevant
2. **Lazy Loading:** Load facets on-demand for large result sets
3. **Facet Caching:** Cache facet counts for common queries
4. **Incremental Updates:** Update facets incrementally for pagination
5. **Pre-Computation:** Pre-compute facets for popular queries

---

## 8. SEARCH PERFORMANCE

### 8.1. Latency Requirements

The Search API must meet strict latency requirements to ensure responsive user experience:

#### API-PERF-001: Latency Targets

**Element ID:** API-PERF-001
**Name:** Search Latency Targets
**Type:** Performance Requirement
**Related Requirements:** REQ-SRV-107

**Description:** Defines target latency metrics for search operations across different document collection sizes.

**Latency Targets:**

| Metric | Target | P50 Target | P99 Target | Maximum |
|--------|--------|------------|------------|---------|
| **Search Query (10K docs)** | <50ms | <75ms | <100ms |
| **Search Query (100K docs)** | <100ms | <150ms | <200ms |
| **Search Query (1M docs)** | <250ms | <400ms | <500ms |
| **Autocomplete** | <30ms | <50ms | <75ms |
| **Facet Computation** | <20ms | <40ms | <60ms |
| **Filter Application** | <10ms | <20ms | <30ms |

**Latency Measurement:**

```rust
use std::time::Instant;

/// Latency measurement wrapper
pub struct LatencyTimer {
    start: Instant,
    operation: String,
}

impl LatencyTimer {
    /// Creates a new latency timer
    pub fn new(operation: String) -> Self {
        Self {
            start: Instant::now(),
            operation,
        }
    }

    /// Returns elapsed time in milliseconds
    pub fn elapsed_ms(&self) -> u64 {
        self.start.elapsed().as_millis()
    }

    /// Logs latency with context
    pub fn log(&self, threshold_ms: u64) {
        let elapsed = self.elapsed_ms();
        let status = if elapsed <= threshold_ms {
            "OK"
        } else {
            "SLOW"
        };

        tracing::info!(
            operation = %self.operation,
            elapsed_ms = elapsed,
            threshold_ms = threshold_ms,
            status = status
        );
    }
}
```

**Latency Monitoring:**

1. **Real-Time Metrics:** Track latency for all search operations
2. **Percentile Tracking:** Compute P50, P95, P99 latencies
3. **Alerting:** Trigger alerts when latencies exceed targets
4. **Historical Analysis:** Maintain latency history for trend analysis
5. **SLA Compliance:** Monitor compliance with service level agreements

---

### 8.2. Caching Strategies

The Search API implements multi-layer caching to optimize performance:

#### API-PERF-002: Cache Architecture

**Element ID:** API-PERF-002
**Name:** Cache Architecture
**Type:** Performance Strategy
**Related Requirements:** REQ-SRV-042, REQ-SRV-110

**Description:** Defines caching strategy for search results and computed data to minimize query latency and reduce load on search index.

**Cache Layers:**

```rust
use std::time::Duration;
use serde::{Deserialize, Serialize};

/// Cache configuration
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct CacheConfig {
    /// L1 cache: In-memory result cache
    pub l1_cache: L1CacheConfig,

    /// L2 cache: Persistent query cache
    pub l2_cache: L2CacheConfig,

    /// Facet cache: Facet count cache
    pub facet_cache: FacetCacheConfig,

    /// Autocomplete cache: Suggestion cache
    pub autocomplete_cache: AutocompleteCacheConfig,
}

/// L1 cache configuration (in-memory)
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct L1CacheConfig {
    /// Maximum cache size (entries)
    pub max_size: usize,

    /// Time-to-live (TTL) for entries
    pub ttl: Duration,

    /// Eviction policy (LRU, LFU)
    pub eviction_policy: EvictionPolicy,
}

/// Eviction policy
#[derive(Debug, Deserialize, Serialize, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum EvictionPolicy {
    /// Least Recently Used
    Lru,

    /// Least Frequently Used
    Lfu,

    /// First-In-First-Out
    Fifo,
}

impl Default for L1CacheConfig {
    fn default() -> Self {
        Self {
            max_size: 10000,
            ttl: Duration::from_secs(300), // 5 minutes
            eviction_policy: EvictionPolicy::Lru,
        }
    }
}
```

**Cache Strategies:**

| Cache Type | Purpose | Size | TTL | Eviction |
|------------|---------|------|-----|----------|
| **L1 (In-Memory)** | Hot query results | 10K entries | 5 minutes, LRU |
| **L2 (Persistent)** | Warm query results | 100K entries | 1 hour, LFU |
| **Facet Cache** | Computed facet counts | 5K entries | 10 minutes, LRU |
| **Autocomplete Cache** | Suggestion results | 20K entries | 15 minutes, LRU |

**Cache Key Generation:**

```rust
use std::collections::hash_map::DefaultHasher;

/// Cache key generator
pub struct CacheKeyGenerator;

impl CacheKeyGenerator {
    /// Generates cache key for search query
    pub fn search_key(
        query: &str,
        filters: &SearchFilters,
        sort: &SearchSort,
        offset: usize,
        limit: usize,
    ) -> String {
        format!(
            "search:{}:{}:{}:{}:{}:{}",
            query,
            serde_json::to_string(filters).unwrap_or_default(),
            serde_json::to_string(sort).unwrap_or_default(),
            offset,
            limit
        )
    }

    /// Generates cache key for autocomplete
    pub fn autocomplete_key(
        query: &str,
        types: &Option<String>,
        limit: usize,
    ) -> String {
        format!(
            "autocomplete:{}:{}:{}",
            query,
            types.as_ref().map(|s| s.as_str()).unwrap_or("all"),
            limit
        )
    }

    /// Generates cache key for facets
    pub fn facet_key(
        query: &str,
        facet_type: FacetType,
        filters: &SearchFilters,
    ) -> String {
        format!(
            "facet:{}:{}:{}",
            query,
            serde_json::to_string(filters).unwrap_or_default(),
            facet_type as u8
        )
    }
}
```

**Cache Hit Rate Targets:**

| Cache Type | Target Hit Rate | Minimum Acceptable |
|------------|----------------|-------------------|
| **L1 Cache** | >80% | >60% |
| **L2 Cache** | >60% | >40% |
| **Facet Cache** | >70% | >50% |
| **Autocomplete Cache** | >85% | >70% |

---

### 8.3. Index Optimization

The Search API implements periodic index optimization to maintain query performance:

#### API-PERF-003: Index Optimization

**Element ID:** API-PERF-003
**Name:** Index Optimization
**Type:** Maintenance Strategy
**Related Requirements:** REQ-SRV-059

**Description:** Defines periodic index optimization procedures to maintain search performance as documents are added, modified, and deleted.

**Optimization Tasks:**

```rust
use std::time::Duration;
use tokio::time::interval;

/// Index optimization configuration
#[derive(Debug, Clone)]
pub struct IndexOptimizationConfig {
    /// Optimization interval
    pub interval: Duration,

    /// Minimum document count for optimization
    pub min_documents: usize,

    /// Force optimization flag
    pub force: bool,
}

impl Default for IndexOptimizationConfig {
    fn default() -> Self {
        Self {
            interval: Duration::from_secs(3600), // 1 hour
            min_documents: 1000,
            force: false,
        }
    }
}

/// Index optimization task
pub async fn optimize_index(
    config: IndexOptimizationConfig,
) -> Result<OptimizationReport, OptimizationError> {
    let mut report = OptimizationReport::new();

    // 1. Segment merging
    report.merge_segments = merge_index_segments().await?;

    // 2. Deleted document cleanup
    report.cleanup_deleted = cleanup_deleted_documents().await?;

    // 3. Statistics update
    report.update_stats = update_index_statistics().await?;

    // 4. Cache warming
    report.warm_cache = warm_search_cache().await?;

    Ok(report)
}
```

**Optimization Report:**

```rust
use serde::Serialize;

/// Index optimization report
#[derive(Debug, Serialize)]
pub struct OptimizationReport {
    /// Timestamp of optimization
    pub timestamp: chrono::DateTime<chrono::Utc>,

    /// Documents before optimization
    pub documents_before: usize,

    /// Documents after optimization
    pub documents_after: usize,

    /// Index size before (bytes)
    pub size_before_bytes: u64,

    /// Index size after (bytes)
    pub size_after_bytes: u64,

    /// Segments merged
    pub merge_segments: MergeResult,

    /// Deleted documents cleaned
    pub cleanup_deleted: CleanupResult,

    /// Statistics updated
    pub update_stats: StatsResult,

    /// Cache warmed
    pub warm_cache: CacheWarmResult,

    /// Total optimization duration (ms)
    pub duration_ms: u64,
}

/// Merge result
#[derive(Debug, Serialize)]
pub struct MergeResult {
    /// Segments before merge
    pub segments_before: usize,

    /// Segments after merge
    pub segments_after: usize,

    /// Space saved (bytes)
    pub space_saved_bytes: u64,
}

/// Cleanup result
#[derive(Debug, Serialize)]
pub struct CleanupResult {
    /// Deleted documents removed
    pub documents_removed: usize,

    /// Space freed (bytes)
    pub space_freed_bytes: u64,
}
```

**Optimization Schedule:**

| Task | Frequency | Duration | Impact |
|------|-----------|----------|--------|
| **Segment Merging** | Every 1 hour | 5-10 minutes | Reduces segment count |
| **Deleted Cleanup** | Every 6 hours | 2-5 minutes | Frees disk space |
| **Statistics Update** | Every 1 hour | 1-2 minutes | Improves query planning |
| **Cache Warming** | Every 2 hours | 5-15 minutes | Improves hit rate |

---

### 8.4. Performance Monitoring

The Search API includes comprehensive performance monitoring:

#### API-PERF-004: Performance Metrics

**Element ID:** API-PERF-004
**Name:** Performance Metrics
**Type:** Monitoring Strategy
**Related Requirements:** REQ-SRV-107

**Description:** Defines performance metrics collection and reporting for search operations.

**Metrics Collected:**

```rust
use prometheus::{Counter, Histogram, Gauge};

/// Search metrics
pub struct SearchMetrics {
    /// Query latency histogram
    pub query_latency: Histogram,

    /// Query counter
    pub query_counter: Counter,

    /// Cache hit counter
    pub cache_hit_counter: Counter,

    /// Cache miss counter
    pub cache_miss_counter: Counter,

    /// Active queries gauge
    pub active_queries: Gauge,

    /// Index size gauge
    pub index_size_bytes: Gauge,

    /// Document count gauge
    pub document_count: Gauge,
}

impl SearchMetrics {
    /// Records search query latency
    pub fn record_query_latency(&self, latency_ms: u64) {
        self.query_latency.observe(latency_ms as f64);
        self.query_counter.inc();
    }

    /// Records cache hit
    pub fn record_cache_hit(&self) {
        self.cache_hit_counter.inc();
    }

    /// Records cache miss
    pub fn record_cache_miss(&self) {
        self.cache_miss_counter.inc();
    }

    /// Updates active query count
    pub fn update_active_queries(&self, count: i64) {
        self.active_queries.set(count);
    }
}
```

**Metrics Dashboard:**

| Metric | Description | Alert Threshold |
|--------|-------------|-----------------|
| **search_query_latency_p50** | Median query latency | >150ms |
| **search_query_latency_p99** | 99th percentile latency | >300ms |
| **search_cache_hit_rate** | Cache hit percentage | <70% |
| **search_active_queries** | Concurrent queries | >500 |
| **search_index_size_bytes** | Index size | >10GB |
| **search_document_count** | Indexed documents | N/A |

**Alerting Strategy:**

1. **Threshold Breach:** Alert when metrics exceed thresholds
2. **Trend Analysis:** Alert on degradation trends
3. **Capacity Planning:** Alert on approaching capacity limits
4. **Performance Regression:** Alert on sudden performance changes
5. **Proactive Monitoring:** Continuous monitoring with automated responses

---

## 9. SEARCH SECURITY

### 9.1. Authentication and Authorization

The Search API enforces strict authentication and authorization to prevent unauthorized access to search functionality:

#### API-SEC-001: Authentication Requirements

**Element ID:** API-SEC-001
**Name:** Authentication Requirements
**Type:** Security Control
**Related Requirements:** REQ-SRV-076, REQ-SRV-081

**Description:** Defines authentication requirements for all search API endpoints.

**Authentication Methods:**

```rust
use axum::extract::State;
use serde::{Deserialize, Serialize};

/// Authenticated user context
#[derive(Debug, Clone)]
pub struct AuthenticatedUser {
    /// User unique identifier
    pub user_id: UserId,

    /// User roles
    pub roles: Vec<Role>,

    /// User permissions
    pub permissions: HashSet<Permission>,

    /// Session expiration time
    pub expires_at: chrono::DateTime<chrono::Utc>,
}

/// User role
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    /// Administrator with full access
    Admin,

    /// Editor with read/write access
    Editor,

    /// Viewer with read-only access
    Viewer,
}

/// User permission
#[derive(Debug, Clone, Hash, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Permission {
    /// Search documents
    SearchDocuments,

    /// View all documents
    ViewAllDocuments,

    /// View internal documents
    ViewInternalDocuments,

    /// Edit documents
    EditDocuments,

    /// Delete documents
    DeleteDocuments,

    /// Manage tags
    ManageTags,

    /// Manage users
    ManageUsers,
}
```

**Authentication Flow:**

1. **Token Validation:** Validate JWT or session token on each request
2. **Session Verification:** Verify session is active and not expired
3. **Permission Check:** Verify user has required permissions for search operation
4. **Context Injection:** Inject user context into request handlers
5. **Audit Logging:** Log all search queries with user attribution

**Authentication Headers:**

| Header | Description | Example |
|--------|-------------|---------|
| Authorization | Bearer token for JWT auth | `Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...` |
| X-Session-ID | Session identifier for cookie auth | `X-Session-ID: abc123def456` |
| X-API-Key | API key for service accounts | `X-API-Key: tachyon_live_abc123` |

---

### 9.2. Input Validation

The Search API implements comprehensive input validation to prevent injection attacks and malformed queries:

#### API-SEC-002: Query String Validation

**Element ID:** API-SEC-002
**Name:** Query String Validation
**Type:** Security Control
**Related Requirements:** REQ-SRV-044

**Description:** Validates search query strings to prevent injection attacks and ensure query safety.

**Validation Rules:**

```rust
use regex::Regex;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum QueryValidationError {
    #[error("Query too short: {0} (minimum 1 character)")]
    QueryTooShort(usize),

    #[error("Query too long: {0} (maximum 1000 characters)")]
    QueryTooLong(usize),

    #[error("Query contains invalid characters: {0}")]
    InvalidCharacters(String),

    #[error("Query contains potential injection: {0}")]
    PotentialInjection(String),

    #[error("Query syntax error: {0}")]
    SyntaxError(String),
}

/// Query validator
pub struct QueryValidator {
    /// Maximum query length
    max_length: usize,

    /// Blocked patterns (injection attempts)
    blocked_patterns: Vec<Regex>,
}

impl QueryValidator {
    /// Creates a new query validator
    pub fn new() -> Self {
        Self {
            max_length: 1000,
            blocked_patterns: vec![
                // SQL injection patterns
                Regex::new(r"(?i)(union|select|insert|update|delete|drop|alter|create|truncate)").unwrap(),

                // NoSQL injection patterns
                Regex::new(r"(?i)(;|'|--|/\*|/\*/|/\*/|xp_|sp_|exec_|execute_)").unwrap(),

                // Script injection patterns
                Regex::new(r"(?i)(<script|javascript:|onerror=|onload=|eval\(|alert\(|document\(|window\(|setTimeout\(|setInterval\()").unwrap(),

                // Path traversal patterns
                Regex::new(r"(?i)(\.\./|\.\.\\|/\\|\.\./)").unwrap(),
            ],
        }
    }

    /// Validates query string
    pub fn validate(&self, query: &str) -> Result<(), QueryValidationError> {
        // Length validation
        if query.is_empty() {
            return Err(QueryValidationError::QueryTooShort(0));
        }
        if query.len() > self.max_length {
            return Err(QueryValidationError::QueryTooLong(query.len()));
        }

        // Character validation
        if !is_valid_utf8(query) {
            return Err(QueryValidationError::InvalidCharacters("Invalid UTF-8"));
        }

        // Injection detection
        for pattern in &self.blocked_patterns {
            if pattern.is_match(query) {
                return Err(QueryValidationError::PotentialInjection(query.to_string()));
            }
        }

        // Syntax validation
        if let Err(e) = validate_query_syntax(query) {
            return Err(QueryValidationError::SyntaxError(e));
        }

        Ok(())
    }
}

/// Validates UTF-8 encoding
fn is_valid_utf8(s: &str) -> bool {
    s.chars().all(|c| !c.is_control() || c.is_ascii_whitespace())
}

/// Validates query syntax
fn validate_query_syntax(query: &str) -> Result<(), String> {
    // Check for unbalanced quotes
    let quote_count = query.matches('"').count();
    if quote_count % 2 != 0 {
        return Err("Unbalanced quotes".to_string());
    }

    // Check for unbalanced parentheses
    let open_parens = query.matches('(').count();
    let close_parens = query.matches(')').count();
    if open_parens != close_parens {
        return Err("Unbalanced parentheses".to_string());
    }

    Ok(())
}
```

**Validation Rules:**

| Rule | Description | Example |
|------|-------------|---------|
| **Length Check** | 1-1000 characters | Empty query rejected |
| **UTF-8 Validation** | Valid UTF-8 encoding | Invalid characters rejected |
| **Injection Detection** | Block SQL/NoSQL patterns | `UNION SELECT` rejected |
| **Script Detection** | Block script patterns | `<script>` rejected |
| **Path Traversal** | Block path traversal | `../etc/passwd` rejected |
| **Syntax Validation** | Balanced quotes/parens | `"unbalanced` rejected |

---

### 9.3. Access Control

The Search API enforces access control at query time to prevent unauthorized document access:

#### API-SEC-003: Document Access Control

**Element ID:** API-SEC-003
**Name:** Document Access Control
**Type:** Security Control
**Related Requirements:** REQ-SRV-081, REQ-SRV-083

**Description:** Enforces document-level access control based on frontmatter metadata and user permissions.

**Access Control Model:**

```rust
use serde::{Deserialize, Serialize};

/// Document access control
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentAccessControl {
    /// Access level (public, internal, restricted)
    pub access_level: AccessLevel,

    /// Allowed roles
    pub allowed_roles: Option<Vec<Role>>,

    /// Allowed users (by ID)
    pub allowed_users: Option<Vec<UserId>>,

    /// Denied roles
    pub denied_roles: Option<Vec<Role>>,

    /// Denied users (by ID)
    pub denied_users: Option<Vec<UserId>>,
}

/// Document access level
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AccessLevel {
    /// Publicly accessible
    Public,

    /// Internal only (requires internal permission)
    Internal,

    /// Restricted (explicit allow/deny lists)
    Restricted,
}

impl DocumentAccessControl {
    /// Checks if user can access document
    pub fn can_access(&self, user: &AuthenticatedUser) -> bool {
        match self.access_level {
            AccessLevel::Public => true,
            AccessLevel::Internal => {
                user.permissions.contains(&Permission::ViewInternalDocuments)
            }
            AccessLevel::Restricted => {
                // Check explicit allow lists
                if let Some(ref allowed_roles) = self.allowed_roles {
                    if !allowed_roles.iter().any(|r| user.roles.contains(r)) {
                        return false;
                    }
                }
                if let Some(ref allowed_users) = self.allowed_users {
                    if !allowed_users.contains(&user.user_id) {
                        return false;
                    }
                }

                // Check explicit deny lists
                if let Some(ref denied_roles) = self.denied_roles {
                    if denied_roles.iter().any(|r| user.roles.contains(r)) {
                        return false;
                    }
                }
                if let Some(ref denied_users) = self.denied_users {
                    if denied_users.contains(&user.user_id) {
                        return false;
                    }
                }

                true
            }
        }
    }
}
```

**Access Control Enforcement:**

1. **Query-Time Filtering:** Apply access control at query execution time
2. **Result Filtering:** Remove unauthorized documents from results
3. **Facet Filtering:** Compute facets only for accessible documents
4. **Audit Logging:** Log all access control decisions
5. **Performance Optimization:** Pre-filter search index by access level

**Internal Content Redaction:**

```rust
/// Internal block redaction
pub fn redact_internal_blocks(content: &str, user: &AuthenticatedUser) -> String {
    if user.permissions.contains(&Permission::ViewInternalDocuments) {
        return content.to_string(); // Keep internal blocks
    }

    // Redact ::: internal blocks
    let re = Regex::new(r"(?s):::\s*internal\s*:::").unwrap();
    re.replace_all(content, "[REDACTED INTERNAL CONTENT]")
}
```

---

### 9.4. Rate Limiting

The Search API implements rate limiting to prevent abuse and ensure fair resource allocation:

#### API-SEC-004: Rate Limit Configuration

**Element ID:** API-SEC-004
**Name:** Rate Limit Configuration
**Type:** Security Control
**Related Requirements:** REQ-SRV-118

**Description:** Configures rate limiting for search API endpoints to prevent abuse and ensure fair usage.

**Rate Limit Configuration:**

```rust
use std::time::Duration;
use serde::{Deserialize, Serialize};

/// Rate limit configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    /// Requests per minute per user
    pub requests_per_minute: u32,

    /// Requests per hour per user
    pub requests_per_hour: u32,

    /// Burst allowance
    pub burst_allowance: u32,

    /// Rate limit window (sliding or fixed)
    pub window_type: RateLimitWindow,
}

/// Rate limit window type
#[derive(Debug, Clone, Serialize, Deserialize, Copy)]
#[serde(rename_all = "lowercase")]
pub enum RateLimitWindow {
    /// Sliding window (more accurate)
    Sliding,

    /// Fixed window (simpler implementation)
    Fixed,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            requests_per_minute: 60,
            requests_per_hour: 1000,
            burst_allowance: 10,
            window_type: RateLimitWindow::Sliding,
        }
    }
}
```

**Rate Limit Headers:**

| Header | Description | Example |
|--------|-------------|---------|
| X-RateLimit-Limit | Requests per time window | `X-RateLimit-Limit: 60` |
| X-RateLimit-Remaining | Remaining requests | `X-RateLimit-Remaining: 45` |
| X-RateLimit-Reset | Reset time (Unix timestamp) | `X-RateLimit-Reset: 1707140800` |
| Retry-After | Seconds until retry allowed | `Retry-After: 30` |

**Rate Limit Response:**

```json
{
  "error": "RATE_LIMIT_EXCEEDED",
  "message": "Rate limit exceeded. Please retry later.",
  "retry_after": 30,
  "limit": 60,
  "window": "minute"
}
```

---

### 9.5. Audit Logging

The Search API implements comprehensive audit logging for security and compliance:

#### API-SEC-005: Audit Logging

**Element ID:** API-SEC-005
**Name:** Audit Logging
**Type:** Security Control
**Related Requirements:** REQ-SRV-085

**Description:** Defines audit logging requirements for all search operations.

**Audit Event Types:**

```rust
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// Audit event
#[derive(Debug, Serialize)]
pub struct AuditEvent {
    /// Event timestamp
    pub timestamp: DateTime<Utc>,

    /// Event type
    #[serde(rename = "type")]
    pub event_type: AuditEventType,

    /// User ID (if authenticated)
    pub user_id: Option<UserId>,

    /// Session ID
    pub session_id: Option<String>,

    /// IP address
    pub ip_address: String,

    /// User agent
    pub user_agent: Option<String>,

    /// Search query (if applicable)
    pub query: Option<String>,

    /// Query parameters
    pub parameters: Option<serde_json::Value>,

    /// Results count
    pub results_count: Option<usize>,

    /// Query execution time (ms)
    pub execution_time_ms: Option<u64>,

    /// Access control decisions
    pub access_decisions: Option<Vec<AccessDecision>>,
}

/// Audit event type
#[derive(Debug, Serialize, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum AuditEventType {
    /// Search query
    SearchQuery,

    /// Autocomplete query
    AutocompleteQuery,

    /// Facet request
    FacetRequest,

    /// Access denied
    AccessDenied,

    /// Rate limit exceeded
    RateLimitExceeded,
}

/// Access control decision
#[derive(Debug, Serialize)]
pub struct AccessDecision {
    /// Document ID
    pub document_id: DocumentId,

    /// Decision (allow/deny)
    pub decision: AccessDecisionType,

    /// Reason for decision
    pub reason: String,
}

/// Access decision type
#[derive(Debug, Serialize, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum AccessDecisionType {
    /// Access granted
    Allow,

    /// Access denied
    Deny,
}
```

**Audit Log Format:**

```json
{
  "timestamp": "2026-02-05T22:00:00Z",
  "type": "search_query",
  "user_id": "550e8400-e29b-41d4-a716-4466554400000",
  "session_id": "abc123def456",
  "ip_address": "192.168.1.100",
  "user_agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36",
  "query": "rust async",
  "parameters": {
    "sort": "relevance",
    "limit": 20,
    "filters": {
      "tags": ["rust"]
    }
  },
  "results_count": 42,
  "execution_time_ms": 87,
  "access_decisions": [
    {
      "document_id": "660e8400-e29b-41d4-a716-44665544700004",
      "decision": "deny",
      "reason": "internal document, user lacks internal permission"
    }
  ]
}
```

**Audit Retention:**

| Event Type | Retention Period | Storage Location |
|------------|-----------------|------------------|
| **Search Queries** | 90 days | Encrypted log storage |
| **Access Denied** | 365 days | Encrypted log storage |
| **Rate Limit Events** | 30 days | Encrypted log storage |
| **Security Events** | 7 years | Encrypted log storage |

---

### 9.6. Security Headers

The Search API includes security headers in all responses:

#### API-SEC-006: Security Headers

**Element ID:** API-SEC-006
**Name:** Security Headers
**Type:** Security Control
**Related Requirements:** REQ-SRV-018, REQ-SRV-090

**Description:** Defines security headers for all API responses.

**Security Headers:**

| Header | Description | Example |
|--------|-------------|---------|
| **Strict-Transport-Security** | Enforce HTTPS | `Strict-Transport-Security: max-age=31536000; includeSubDomains` |
| **X-Content-Type-Options** | Prevent MIME sniffing | `X-Content-Type-Options: nosniff` |
| **X-Frame-Options** | Prevent clickjacking | `X-Frame-Options: DENY` |
| **X-XSS-Protection** | XSS protection | `X-XSS-Protection: 1; mode=block` |
| **Content-Security-Policy** | CSP header | `Content-Security-Policy: default-src 'self'` |
| **Referrer-Policy** | Referrer policy | `Referrer-Policy: strict-origin-when-cross-origin` |
| **Permissions-Policy** | Permissions policy | `Permissions-Policy: geolocation=(), microphone=()` |

---

## 10. REFERENCES

### 10.1. Internal References

| Document ID | Title | Section |
|-------------|-------|---------|
| [TACHYON-STD-V1.0](../../.specs/01_standards/coding_standards.md) | Coding and Documentation Standards | Section 1-7 |
| [TACHYON-REQ-SRV-V1.0](../../.specs/04_future_state/reqs/server_requirements.md) | Server Application Requirements | Section 3-10 |
| [TACHYON-DES-API-V1.0](../../.specs/04_future_state/design/api_interfaces.md) | API Interfaces Design | Section 2-7 |
| [TACHYON-ADR-001-V1.0](../../.specs/02_adrs/001_rust_as_primary_language.md) | Rust as Primary Language | Section 4.1-4.8 |
| [TACHYON-ADR-003-V1.0](../../.specs/02_adrs/003_axum_for_http2_server.md) | Axum for HTTP/2 Server | Section 4.1-4.8 |
| [TACHYON-ADR-007-V1.0](../../.specs/02_adrs/007_tokio_for_async_runtime.md) | Tokio for Async Runtime | Section 4.1-4.8 |
| [TACHYON-TMA-V1.0](../../.specs/03_threat_model/analysis.md) | Threat Model Analysis | Section 9.1-9.6 |

### 10.2. External References

**Standards:**

[1] ISO/IEC 26514:2021, "Systems and Software Engineering — Requirements for designers and developers of user documentation," International Organization for Standardization, Geneva, Switzerland, 2021.

[2] ISO/IEC 12207:2017, "Systems and Software Engineering — Software Life Cycle Processes," International Organization for Standardization, Geneva, Switzerland, 2017.

[3] ISO/IEC 25010:2011, "Systems and Software Engineering — Systems and Software Quality Requirements and Evaluation (SQuaRE)," International Organization for Standardization, Geneva, Switzerland, 2011.

[4] IEEE 829-2008, "IEEE Standard for Software Test Documentation," IEEE Standards Association, Piscataway, NJ, USA, 2008.

[5] IEEE 1063-2001, "IEEE Standard for Software User Documentation," IEEE Standards Association, Piscataway, NJ, USA, 2001.

[6] RFC 7540, "Hypertext Transfer Protocol Version 2 (HTTP/2)," IETF, 2015.

[7] RFC 8446, "The Transport Layer Security (TLS) Protocol Version 1.3," IETF, 2018.

**Technical References:**

[8] The Rust Programming Language, "The Rust Reference," Online. Available: https://doc.rust-lang.org/reference/. [Accessed: 01-Feb-2026].

[9] The Rust Project, "Rust Edition 2024," Online. Available: https://doc.rust-lang.org/edition-guide/rust-2024/index.html. [Accessed: 01-Feb-2026].

[10] The Rust Project, "The Rustonomicon: The Unsafe Book," Online. Available: https://doc.rust-lang.org/nomicon/. [Accessed: 01-Feb-2026].

[11] The Rust Project, "The Rust Book," Online. Available: https://doc.rust-lang.org/book/. [Accessed: 01-Feb-2026].

[12] Tokio Contributors, "Tokio: Asynchronous Runtime for the Rust Programming Language," Online. Available: https://tokio.rs/. [Accessed: 01-Feb-2026].

[13] Axum Project, "Axum: Ergonomic and Modular Web Framework," Online. Available: https://docs.rs/axum/. [Accessed: 01-Feb-2026].

[14] Tantivy, "Tantivy: Full-Text Search Engine Library," Online. Available: https://github.com/quickwit-oss/tantivy. [Accessed: 01-Feb-2026].

[15] Serde, "Serde: Serialization Framework for Rust," Online. Available: https://serde.rs/. [Accessed: 01-Feb-2026].

[16] OAuth 2.0, "The OAuth 2.0 Authorization Framework," IETF, 2012.

[17] SAML 2.0, "Security Assertion Markup Language (SAML) V2.0," OASIS, 2005.

[18] OpenID Connect, "OpenID Connect 1.0," OpenID Foundation, 2014.

**Academic References:**

[19] A. K. G. et al., "Rust: Safety and concurrency at scale," *Proceedings of the 2019 ACM SIGPLAN International Symposium on New Ideas, New Paradigms, and Reflections on Programming and Software*, pp. 1-3, October 2019.

[20] J. R. et al., "Evaluating the safety of Rust," *Proceedings of the 2020 ACM SIGPLAN Conference on Programming Language Design and Implementation*, pp. 62-76, June 2020.

[21] T. R. et al., "A formal model of Rust's type system," *Proceedings of the 2021 ACM SIGPLAN International Conference on Functional Programming*, pp. 1-15, August 2021.

[22] crates.io, "Rust Package Registry," Online. Available: https://crates.io/. [Accessed: 01-Feb-2026].

**Security References:**

[23] OWASP Top 10, "OWASP Top 10 Web Application Security Risks," Open Web Application Security Project, 2021. Available: https://owasp.org/www-project-top-ten.

[24] CWE/SANS Top 25, "CWE/SANS Top 25 Most Dangerous Software Errors," MITRE and SANS Institute, 2021.

[25] Common Vulnerabilities and Exposures (CVE), "CVE List," MITRE Corporation. Available: https://cve.mitre.org/.

### 10.3. Glossary

| Term | Definition |
|------|------------|
| **BM25** | Best Matching 25 ranking algorithm for relevance scoring in information retrieval |
| **Faceted Search** | Search technique that allows users to refine results by applying filters on predefined categories (facets) |
| **Fuzzy Search** | Search technique that finds matches for terms with approximate spelling or typing errors |
| **Inverted Index** | Data structure that maps content to terms for efficient full-text search |
| **Autocomplete** | Feature that suggests completions for partial user input based on historical data or predictive algorithms |
| **Relevance Ranking** | Algorithmic ordering of search results by computed relevance scores |
| **Query Latency** | Time elapsed from receiving a search query to returning results |
| **Cache Hit Rate** | Percentage of search requests served from cache rather than computed from search index |
| **Access Control List (ACL)** | Mechanism for defining and enforcing access permissions to resources |
| **Role-Based Access Control (RBAC)** | Access control model that assigns permissions to users based on their roles |
| **Principle of Least Privilege** | Security principle that users are granted only the minimum permissions necessary to perform their tasks |
| **Rate Limiting** | Technique for controlling the rate of traffic sent or received by a client or user |
| **Input Validation** | Process of verifying that user input conforms to expected format, type, length, and range constraints |
| **SQL Injection** | Code injection technique where malicious SQL statements are inserted into user input fields |
| **Cross-Site Scripting (XSS)** | Security vulnerability that allows attackers to inject malicious scripts into web pages viewed by other users |
| **Path Traversal** | Attack technique that allows accessing files and directories outside the intended directory structure |

---

## DOCUMENT VERSION HISTORY

| Version | Date | Author | Changes |
|---------|------|---------|
| V1.0 | 2026-02-05 | System Architect | Initial document creation with all sections complete |

---

## APPROVAL RECORD

| Version | Date | Approver | Status |
|---------|------|----------|---------|
| V1.0 | 2026-02-05 | Technical Lead | Proposed for review |

---

*End of Document*

**Header Implementation:**

```rust
use axum::http::HeaderMap;

/// Adds security headers to response
pub fn add_security_headers(headers: &mut HeaderMap) {
    headers.insert(
        "Strict-Transport-Security",
        "max-age=31536000; includeSubDomains"
    );
    headers.insert(
        "X-Content-Type-Options",
        "nosniff"
    );
    headers.insert(
        "X-Frame-Options",
        "DENY"
    );
    headers.insert(
        "X-XSS-Protection",
        "1; mode=block"
    );
    headers.insert(
        "Content-Security-Policy",
        "default-src 'self'"
    );
}
```
