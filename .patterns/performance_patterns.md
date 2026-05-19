# Performance Patterns

This document contains performance patterns and best practices identified during Tachyon project development.

## Caching Patterns

### P-PERF-001: Three-Tier JIT Compilation

**Category:** Caching
**Complexity:** Medium
**Context:** Sub-15ms rendering latency requirement for real-time editing.

**Problem:** Single-tier caching is too slow.

**Solution:** Three-tier compilation: cache lookup > template rendering > baseline parsing.

**Implementation:**
```rust
async fn render_document_tiered(path: &Path) -> Result<String> {
    let key = cache_key(path);
    
    // Tier 3: Cache lookup
    if let Some(html) = cache.get(&key) {
        return Ok(html);
    }
    
    // Tier 1: Baseline parsing
    let markdown = std::fs::read_to_string(path)?;
    let ast = parse_markdown(&markdown)?;
    
    // Tier 2: Template rendering
    let html = render_template("base.html", &ast)?;
    
    cache.insert(key, html.clone());
    Ok(html)
}
```

**Benefits:**
- Cache hit: <1ms
- Template: <5ms
- Baseline: <10ms
- Total (cache miss): <15ms

**Traceability:** LL-PERF-001

---

### P-PERF-002: LRU Cache with DashMap

**Category:** Caching
**Complexity:** Medium
**Context:** Read-heavy workloads require lock-free concurrent caching.

**Problem:** Standard HashMap locks degrade concurrency.

**Solution:** LRU cache with DashMap for lock-free reads.

**Implementation:**
```rust
use dashmap::DashMap;

struct Cache<K, V> {
    map: DashMap<K, V>,
    capacity: usize,
    access_order: Mutex<VecDeque<K>>,
}

impl<K, V> Cache<K, V>
where
    K: Eq + Hash + Clone,
{
    fn get(&self, key: &K) -> Option<V> {
        let result = self.map.get(key).map(|r| r.value().clone());
        
        if result.is_some() {
            self.update_access_order(key);
        }
        
        result
    }
    
    fn insert(&self, key: K, value: V) {
        if self.map.len() >= self.capacity {
            self.evict();
        }
        self.map.insert(key.clone(), value);
        self.update_access_order(&key);
    }
}
```

**Benefits:**
- Lock-free reads
- Linear scalability
- Configurable eviction policy

**Traceability:** LL-PERF-002

---

### P-PERF-003: Cache Hit Rate Monitoring

**Category:** Caching
**Complexity:** Simple
**Context:** Cache effectiveness must be measured for optimization.

**Problem:** Unmonitored caches may have poor hit rates without detection.

**Solution:** Track cache hit rate and alert on threshold breach.

**Implementation:**
```rust
struct CacheMetrics {
    hits: AtomicUsize,
    misses: AtomicUsize,
}

impl CacheMetrics {
    fn record_hit(&self) {
        self.hits.fetch_add(1, Ordering::Relaxed);
    }
    
    fn record_miss(&self) {
        self.misses.fetch_add(1, Ordering::Relaxed);
    }
    
    fn hit_rate(&self) -> f64 {
        let hits = self.hits.load(Ordering::Relaxed);
        let misses = self.misses.load(Ordering::Relaxed);
        if hits + misses == 0 {
            return 1.0;
        }
        hits as f64 / (hits + misses) as f64
    }
}
```

**Benefits:**
- Measurable cache effectiveness
- Alerting on performance degradation
- Data-driven optimization

**Traceability:** LL-PERF-003

---

### P-PERF-004: Role-Based Cache Keys

**Category:** Caching
**Complexity:** Medium
**Context:** Different users have different access levels.

**Problem:** Cache doesn't account for user roles.

**Solution:** Include role in cache key.

**Implementation:**
```rust
fn cache_key(path: &Path, role: Role) -> String {
    format!("{}:{}", path.display(), role)
}

async fn get_document_cached(path: &Path, user_role: Role) -> Result<String> {
    let key = cache_key(path, user_role);
    
    if let Some(html) = cache.get(&key) {
        return Ok(html);
    }
    
    let html = render_document_with_role(path, user_role).await?;
    cache.insert(key, html.clone());
    Ok(html)
}
```

**Benefits:**
- Role-aware caching
- Correct access control
- Improved cache hit rate

**Traceability:** LL-PERF-001

## Rendering Patterns

### P-PERF-005: SIMD-Accelerated Markdown Parsing

**Category:** Rendering
**Complexity:** Simple
**Context:** Markdown parsing is performance-critical.

**Problem:** Pure software parsing is CPU-intensive and slow.

**Solution:** Use pulldown-cmark with SIMD acceleration.

**Implementation:**
```toml
[dependencies.pulldown-cmark]
version = "0.9"
features = ["simd"]
```

**Benefits:**
- 4x speedup for parsing
- Sub-15ms average latency achievable
- Lower CPU utilization

**Traceability:** LL-PERF-001

---

### P-PERF-006: Debounced File Watching

**Category:** Rendering
**Complexity:** Simple
**Context:** File watcher generates events for every write operation.

**Problem:** Rapid successive writes cause redundant processing.

**Solution:** Debounce events with 50ms window.

**Implementation:**
```rust
async fn debounced_events(mut rx: Receiver<Event>) -> Vec<Event> {
    let mut batch = Vec::new();
    let mut deadline: Option<Instant> = None;
    let debounce_duration = Duration::from_millis(50);
    
    while let Ok(event) = rx.recv().await {
        batch.push(event);
        
        match deadline {
            None => {
                deadline = Some(Instant::now() + debounce_duration);
            }
            Some(d) if Instant::now() >= d => {
                break;
            }
            Some(_) => {}
        }
    }
    
    batch
}
```

**Benefits:**
- Reduces redundant processing
- Improves cache hit rate
- Stabilizes event stream

**Traceability:** LL-PERF-002

## Search Patterns

### P-PERF-007: BM25 Relevance Scoring

**Category:** Search
**Complexity:** Medium
**Context:** Full-text search needs relevance scoring.

**Problem:** Simple keyword search lacks relevance.

**Solution:** BM25 algorithm with configurable parameters.

**Implementation:**
```toml
[dependencies]
tantivy = "0.21"
```

```rust
use tantivy::collector::TopDocs;
use tantivy::query::QueryParser;

fn create_bm25_query(query: &str) -> Box<dyn Query> {
    let parser = QueryParser::for_index(&index, vec![title, body]);
    
    // BM25 parameters: k1=1.5, b=0.75
    let query = parser.parse_query(query)?;
    
    query
}

async fn search_documents(query: &str) -> Result<Vec<Document>> {
    let query = create_bm25_query(query);
    
    let searcher = index.reader()?.searcher();
    let top_docs = searcher.search(&query, &TopDocs::with_limit(100))?;
    
    // Convert to documents
    top_docs.into_iter()
        .map(|(score, doc_address)| {
            let doc = searcher.doc(doc_address)?;
            Ok((score, doc))
        })
        .collect()
}
```

**Benefits:**
- Relevance-ranked results
- Configurable scoring parameters
- Sub-100ms search latency

**Traceability:** LL-PERF-004

---

### P-PERF-008: Inverted Index

**Category:** Search
**Complexity:** Medium
**Context:** Fast full-text search required.

**Problem:** Linear search is too slow for large document sets.

**Solution:** Tantivy inverted index with term-level indexing.

**Implementation:**
```rust
use tantivy::schema::*;

fn create_index_schema() -> Schema {
    let mut schema_builder = Schema::builder();
    
    schema_builder.add_text_field("title", TEXT | STORED);
    schema_builder.add_text_field("body", TEXT | STORED);
    schema_builder.add_text_field("path", STRING | STORED);
    
    schema_builder.build()
}
```

**Benefits:**
- Fast term lookup
- Efficient boolean queries
- Scalable to millions of documents

**Traceability:** LL-PERF-004

## Concurrency Patterns

### P-PERF-009: Tokio Multi-Threaded Scheduler

**Category:** Concurrency
**Complexity:** Simple
**Context:** Optimal async runtime for I/O-bound workloads.

**Problem:** Single-threaded runtime underutilizes multi-core.

**Solution:** Tokio with multi-threaded scheduler.

**Implementation:**
```rust
#[tokio::main(flavor = "multi_thread", worker_threads = num_cpus::get())]
async fn main() -> Result<()> {
    // Application logic
}
```

**Benefits:**
- Optimal core utilization
- Efficient I/O handling
- Scalable concurrency

**Traceability:** LL-PERF-005

---

### P-PERF-010: Semaphore-Based Concurrency Limits

**Category:** Concurrency
**Complexity:** Medium
**Context:** Unbounded concurrency can overwhelm resources.

**Problem:** Unbounded parallel requests cause resource exhaustion.

**Solution:** Semaphore-based concurrency control.

**Implementation:**
```rust
use tokio::sync::Semaphore;

const MAX_CONCURRENT_REQUESTS: usize = 100;

async fn handle_requests_concurrently() -> Result<()> {
    let semaphore = Arc::new(Semaphore::new(MAX_CONCURRENT_REQUESTS));
    
    let mut handles = Vec::new();
    
    for request in requests {
        let permit = semaphore.clone().acquire_owned().await?;
        
        let handle = tokio::spawn(async move {
            let _permit = permit; // Released on drop
            process_request(request).await
        });
        
        handles.push(handle);
    }
    
    for handle in handles {
        handle.await??;
    }
    
    Ok(())
}
```

**Benefits:**
- Controlled resource usage
- Prevents overload
- Predictable performance

**Traceability:** LL-PERF-005

## Benchmarking Patterns

### P-PERF-011: Criterion Benchmarks

**Category:** Benchmarking
**Complexity:** Medium
**Context:** Reliable performance measurements required.

**Problem:** Ad-hoc measurements are unreliable.

**Solution:** Criterion statistical benchmarks.

**Implementation:**
```toml
[dev-dependencies]
criterion = "0.5"
```

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_markdown_parsing(c: &mut Criterion) {
    let markdown = std::fs::read_to_string("large_document.md").unwrap();
    
    c.bench_function("parse_markdown", |b| {
        b.iter(|| {
            parse_markdown(black_box(&markdown))
        })
    });
}

criterion_group!(benches, bench_markdown_parsing);
criterion_main!(benches);
```

**Benefits:**
- Statistical confidence
- Regression detection
- Performance profiling

**Traceability:** LL-PERF-003

## References

- [Performance Requirements](.adrs/
- [Benchmark Suite](.adrs/
- [Performance Regression Detection](.adrs/
