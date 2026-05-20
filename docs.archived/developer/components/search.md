# Search Component

The `tachyon-search` crate provides full-text search functionality.

## Overview

Search provides:
- Full-text indexing
- Fuzzy matching
- Field-specific queries
- Faceted search
- Real-time indexing

## Architecture

```
┌────────────────────────────────────────────┐
│            SearchEngine                     │
│  ┌──────────────────────────────────────┐  │
│  │           Index Writer               │  │
│  │  (Background indexing thread)        │  │
│  └──────────────────────────────────────┘  │
│  ┌──────────────────────────────────────┐  │
│  │           Index Reader               │  │
│  │  (Multiple readers for queries)      │  │
│  └──────────────────────────────────────┘  │
│  ┌──────────────────────────────────────┐  │
│  │         Query Parser                 │  │
│  │  (Parse user queries)                │  │
│  └──────────────────────────────────────┘  │
└────────────────────────────────────────────┘
```

## Key Types

### SearchEngine

```rust
pub struct SearchEngine {
    index: Index,
    reader: IndexReader,
    writer: Mutex<Option<IndexWriter>>,
    schema: Schema,
}

pub struct SearchConfig {
    pub index_path: PathBuf,
    pub commit_interval: Duration,
    pub max_results: usize,
}
```

### Schema

```rust
fn create_schema() -> Schema {
    let mut schema = Schema::builder();
    
    schema.add_text_field("title", TEXT | STORED);
    schema.add_text_field("content", TEXT);
    schema.add_text_field("author", TEXT | STORED);
    schema.add_text_field("tags", TEXT | STORED);
    schema.add_date_field("created", INDEXED | STORED);
    schema.add_date_field("modified", INDEXED | STORED);
    schema.add_text_field("path", STORED);
    schema.add_u64_field("status", INDEXED);
    
    schema.build()
}
```

### Query Types

```rust
pub struct Query {
    pub text: Option<String>,
    pub fields: HashMap<String, String>,
    pub filters: Vec<Filter>,
    pub sort: Option<Sort>,
    pub limit: usize,
    pub offset: usize,
}

pub enum Filter {
    Field { name: String, value: String },
    Range { name: String, from: Value, to: Value },
    DateRange { name: String, from: DateTime, to: DateTime },
}
```

## Usage

### Indexing

```rust
let engine = SearchEngine::open("./index")?;

// Add document
engine.index(Document {
    id: "doc-1".into(),
    title: "API Reference".into(),
    content: "Documentation for the API...".into(),
    tags: vec!["api", "reference"],
    // ...
}).await?;

// Commit changes
engine.commit().await?;
```

### Searching

```rust
// Simple search
let results = engine.search("api authentication").await?;

// Advanced query
let results = engine.search(Query {
    text: Some("api".into()),
    fields: HashMap::from([
        ("status".into(), "published".into()),
    ]),
    filters: vec![
        Filter::DateRange {
            name: "modified".into(),
            from: "2024-01-01".parse()?,
            to: "2024-12-31".parse()?,
        },
    ],
    limit: 20,
    offset: 0,
    ..Default::default()
}).await?;
```

### Results

```rust
pub struct SearchResult {
    pub hits: Vec<SearchHit>,
    pub total: usize,
    pub page: usize,
    pub pages: usize,
}

pub struct SearchHit {
    pub id: DocumentId,
    pub title: String,
    pub excerpt: String,
    pub score: f32,
    pub highlights: Vec<Highlight>,
}

pub struct Highlight {
    pub field: String,
    pub fragments: Vec<String>,
}
```

## Query Syntax

### Basic Queries

| Query | Description |
|-------|-------------|
| `hello` | Documents containing "hello" |
| `hello world` | Documents with both terms |
| `"exact phrase"` | Exact phrase match |
| `hello OR world` | Either term |

### Field Queries

| Query | Description |
|-------|-------------|
| `title:api` | "api" in title |
| `author:john` | Author is "john" |
| `tag:reference` | Has tag "reference" |

### Ranges

| Query | Description |
|-------|-------------|
| `created:>2024-01-01` | Created after date |
| `modified:<2024-06-01` | Modified before date |
| `count:[10 TO 20]` | Count in range |

### Modifiers

| Query | Description |
|-------|-------------|
| `hello~` | Fuzzy match |
| `hello~2` | Fuzzy with distance 2 |
| `hell*` | Prefix wildcard |
| `h?llo` | Single char wildcard |

## Real-Time Indexing

### File Watcher Integration

```rust
impl SearchEngine {
    pub fn watch(&self, path: &Path) -> Result<()> {
        let (tx, rx) = channel();
        let mut watcher = watcher(tx, Duration::from_millis(100))?;
        
        watcher.watch(path, RecursiveMode::Recursive)?;
        
        loop {
            match rx.recv() {
                Ok(event) => self.handle_file_event(event).await?,
                Err(e) => error!("Watch error: {}", e),
            }
        }
    }
    
    async fn handle_file_event(&self, event: Event) -> Result<()> {
        match event.kind {
            EventKind::Create(_) | EventKind::Modify(_) => {
                for path in &event.paths {
                    if path.extension() == Some("md") {
                        self.index_file(path).await?;
                    }
                }
            }
            EventKind::Remove(_) => {
                for path in &event.paths {
                    self.remove_file(path).await?;
                }
            }
            _ => {}
        }
        Ok(())
    }
}
```

## Performance

### Targets

| Operation | Target |
|-----------|--------|
| Index document | < 10ms |
| Search query | < 100ms |
| Index commit | < 500ms |
| Index size | ~1KB per document |

### Optimization

```rust
impl SearchEngine {
    pub fn optimize(&self) -> Result<()> {
        // Merge segments
        self.writer.lock()?.as_mut().unwrap().merge()?;
        Ok(())
    }
}
```

## Configuration

```toml
[search]
index_path = "./index"
commit_interval_seconds = 30
max_results = 20
enable_fuzzy = true
fuzzy_distance = 2
```

## Error Handling

```rust
#[derive(Debug, thiserror::Error)]
pub enum SearchError {
    #[error("Index error: {0}")]
    Index(String),
    
    #[error("Query parse error: {0}")]
    QueryParse(String),
    
    #[error("Document not found: {0}")]
    NotFound(String),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}
```
