# ADR-032: Benchmark Suite Design

**Status:** Accepted
**Date:** 2026-02-11
**Context:** Phase 4 (Performance Engineering)
**Authors:** Performance Engineer Agent

---

## Context

The Tachyon system requires a comprehensive benchmark suite to validate performance requirements, detect regressions, and guide optimization efforts.

This ADR documents the design decisions for the benchmark suite, including tool selection, workload design, and integration strategy.

---

## Decision

**We implement a multi-faceted benchmark suite with:**

1. **Criterion.rs for micro-benchmarks:** Precise, statistical measurements of individual operations
2. **K6 for HTTP load testing:** High-throughput HTTP endpoint testing
3. **wrk2 for WebSocket testing:** Concurrent WebSocket connection testing
4. **Representative workloads:** Realistic test data matching production usage
5. **Continuous integration:** Automated benchmarking in CI/CD pipeline

### Benchmark Architecture:

```
┌─────────────────────────────────────────────────────────────┐
│                 Benchmark Suite Architecture               │
├─────────────────────────────────────────────────────────────┤
│  Criterion.rs (Micro-benchmarks)                  │
│  - Latency measurements (P50, P95, P99)          │
│  - Throughput measurements                          │
│  - Memory allocation tracking                      │
│  - Statistical analysis (mean, std dev)             │
├─────────────────────────────────────────────────────────────┤
│  Load Testing Tools                                 │
│  - K6 (HTTP endpoints)                         │
│  - wrk2 (WebSocket)                            │
│  - Throughput focus, concurrent user simulation       │
├─────────────────────────────────────────────────────────────┤
│  Test Data & Workloads                            │
│  - Representative documents (small, medium, large)     │
│  - Realistic query patterns                       │
│  - Production-like scenarios                       │
└─────────────────────────────────────────────────────────────┘
```

---

## Alternatives Considered

### Alternative 1: Custom Benchmark Framework

**Description:** Build a custom benchmarking framework from scratch.

**Pros:**
- Full control over benchmark logic
- Tailored to specific Tachyon requirements

**Cons:**
- Significant development effort
- Maintenance burden
- Less statistical rigor than established tools
- Potential bugs in custom code
- No community support

**Rejection:** Criterion.rs provides better trade-off for our needs.

---

### Alternative 2: Production Benchmarking Only

**Description:** Only benchmark in production environment with real users.

**Pros:**
- Real-world performance data
- No artificial workload bias

**Cons:**
- Cannot reproduce issues locally
- Hard to isolate variables
- Production risk during benchmarking
- Cannot test before deployment
- Cannot measure specific components in isolation

**Rejection:** Requires controlled benchmark environment for reproducible results.

---

### Alternative 3: Manual Performance Testing

**Description:** Use manual timing and observation for performance validation.

**Pros:**
- No tool overhead
- Simple to implement

**Cons:**
- Subjective measurements
- No reproducible results
- Difficult to aggregate across team
- Cannot detect subtle regressions
- Time-consuming for comprehensive analysis

**Rejection:** Requires objective, automated measurements with statistical analysis.

---

## Chosen Approach: Criterion.rs + Load Testing Tools

**Rationale:**

1. **Criterion.rs Maturity:** Well-established, maintained benchmarking library for Rust
2. **Statistical Rigor:** Provides mean, median, std dev, confidence intervals
3. **Regression Detection:** Built-in baseline comparison and significance testing
4. **HTML Reports:** Rich visualizations and comparison charts
5. **Load Testing Integration:** K6 and wrk2 complement micro-benchmarks for system-level testing
6. **CI/CD Integration:** Automated benchmarking on every PR and main branch push

### Criterion.rs Advantages:

| Feature | Benefit | Relevance to Tachyon |
|-----------|--------|------------------------|
| Statistical analysis | P50, P95, P99 percentiles | Latency requirements |
| Baseline comparison | Automated regression detection | Continuous performance |
| Comparison mode | Multiple runs comparison | Optimization validation |
| Output formats | HTML, JSON, Console | Dashboard integration |
| Async support | tokio compatibility | Async task profiling |
| Memory profiling | heaptrack integration | Memory leak detection |

---

## Benchmark Design

### 1. Micro-Benchmark Categories

#### 1.1. Rendering Engine Benchmarks

**Benchmark IDs:** BM-REND-001 through BM-REND-004

| Benchmark | Component | Metric | Target | Test Data |
|-----------|-----------|---------|-----------|
| BM-REND-001 | Markdown parsing | Latency | small.md, large.md |
| BM-REND-002 | JIT rendering (cache miss) | Latency | typical.md |
| BM-REND-003 | JIT rendering (cache hit) | Latency | typical.md |
| BM-REND-004 | LRU cache operations | Latency, throughput | N/A |

**Workload Characteristics:**
- Document sizes: small (100 tokens), medium (1,000 tokens), large (10,000 tokens)
- Content types: plain text, code blocks, tables, frontmatter
- Cache states: warm, cold, invalidated

**Traceability:** performance_requirements.md, PR-LAT-001, PR-LAT-002

#### 1.2. Search Engine Benchmarks

**Benchmark IDs:** BM-SEARCH-001, BM-SEARCH-002

| Benchmark | Component | Metric | Target | Test Data |
|-----------|-----------|---------|-----------|
| BM-SEARCH-001 | BM25 query | Latency | single, multi-term, complex queries |
| BM-SEARCH-002 | Document indexing | Latency, throughput | small, medium, large docs |

**Workload Characteristics:**
- Index sizes: 100, 10,000, 100,000 documents
- Query types: single term, multi-term, complex
- Result limits: 10, 50, 100

**Traceability:** performance_requirements.md, PR-LAT-004, PR-LAT-005

#### 1.3. Git Operations Benchmarks

**Benchmark IDs:** BM-GIT-001, BM-GIT-002

| Benchmark | Component | Metric | Target | Test Data |
|-----------|-----------|---------|-----------|
| BM-GIT-001 | Git commit | Latency, throughput | single file, multiple files |
| BM-GIT-002 | History fetch | Latency | 100, 1000 commits |

**Traceability:** performance_requirements.md, PR-LAT-007, PR-LAT-008

#### 1.4. File Watcher Benchmarks

**Benchmark ID:** BM-WATCH-001

| Benchmark | Component | Metric | Target | Test Data |
|-----------|-----------|---------|-----------|
| BM-WATCH-001 | Event latency | Latency | file creation, modification, deletion |

**Traceability:** performance_requirements.md, PR-LAT-006

#### 1.5. WebSocket Benchmarks

**Benchmark ID:** BM-WS-001

| Benchmark | Component | Metric | Target | Test Data |
|-----------|-----------|---------|-----------|
| BM-WS-001 | Message delivery | Latency | various message types |

**Traceability:** performance_requirements.md, PR-LAT-009

---

### 2. Load Testing Benchmarks

#### 2.1. HTTP Endpoint Load Testing (K6)

**Benchmark Scenarios:**

| Scenario | Virtual Users | Duration | Target Metric | Success Criteria |
|-----------|---------------|----------|-------------|-----------------|
| Desktop load | 10 | 5 minutes | P99 latency < 50ms | UI-RQ-001 |
| Server load | 100 | 10 minutes | P99 latency < 50ms | IN-RQ-001 |
| Search load | 50 | 5 minutes | P99 latency < 150ms | SD-RQ-001 |
| Concurrent writes | 20 | 5 minutes | Error rate < 1% | CM-RQ-006 |
| Mixed workload | 100 | 10 minutes | All metrics met | System-level |

**K6 Script Structure:**
```javascript
// tachyon-k6.js
import http from 'k6/http';
import { check, sleep } from 'k6';

export let options = {
  stages: '1m30s',      // Warmup + test
  thresholds: {
    http_req_duration: ['p(95)<0.05'],  // 95th percentile < 50ms
    http_req_failed: ['rate<0.01'],        // Error rate < 1%
  },
};

export function benchmarkDocumentList() {
  let res = http.get('http://localhost:8080/documents', options);

  check(res, {
    'Document list latency is under 50ms': (r) => r.timings.duration < 50,
    'Document list returned 200': (r) => r.status === 200,
  });

  sleep(1);
}

export function benchmarkSearch() {
  let res = http.post('http://localhost:8080/search', {
    json: { query: 'rust async' },
  }, options);

  check(res, {
    'Search latency is under 100ms': (r) => r.timings.duration < 100,
    'Search returned 200': (r) => r.status === 200,
  });

  sleep(1);
}
```

**Traceability:** PR-THR-007, performance_requirements.md

#### 2.2. WebSocket Load Testing (wrk2)

**Benchmark Scenarios:**

| Scenario | Connections | Duration | Target Metric | Success Criteria |
|-----------|-------------|----------|-------------|-----------------|
| Desktop mode | 10 | 5 minutes | P99 latency < 100ms | IN-RQ-004 |
| Server mode | 1000 | 10 minutes | P99 latency < 100ms | IN-RQ-004 |
| Message flood | 5000 | 5 minutes | Throughput > 9000 msg/s | PR-THR-005 |

**wrk2 Command:**
```bash
# WebSocket load testing
wrk -t 4 -c 1000 -d 30s -s ws://localhost:8080/ws

# Parameters:
# -t 4: 4 threads
# -c 1000: 1000 concurrent connections
# -d 30s: 30 second duration
# -s: WebSocket protocol
```

**Traceability:** PR-THR-005, performance_requirements.md

---

### 3. Test Data Design

#### 3.1. Representative Documents

**Location:** `tachyon/benches/data/documents/`

| Document | Size | Tokens | Content | Purpose |
|----------|------|--------|----------|---------|
| small.md | 1KB | 100 | Basic Markdown | Parsing benchmark |
| typical.md | 10KB | 1,000 | Frontmatter + code | Rendering benchmark |
| large.md | 100KB | 10,000 | Nested structures | Stress test |
| internal.md | 10KB | 1,000 | With ::: internal blocks | Redaction benchmark |
| code_heavy.md | 20KB | 2,000 | Multiple code blocks | Template benchmark |

#### 3.2. Search Indexes

**Location:** `tachyon/benches/data/index/`

| Index | Documents | Size | Purpose |
|--------|-----------|--------|---------|
| small_index | 100 | 1MB | Query benchmark |
| medium_index | 10,000 | 100MB | Realistic load |
| large_index | 100,000 | 1GB | Stress test |

#### 3.3. Query Patterns

| Pattern | Terms | Filters | Purpose |
|---------|--------|----------|---------|
| single_term | 1 | None | Baseline query |
| multi_term | 3-5 | None | Realistic search |
| complex | 10 | Role-based | Complex query |
| fuzzy | 2 | Fuzzy match | Fuzzy search |
| phrase | 1 | Exact phrase | Phrase search |

---

### 4. Benchmark Execution Strategy

#### 4.1. Development Benchmarks

**Frequency:** Every pull request and main branch push

**Command:**
```bash
cargo bench -- --save-baseline main
```

**Integration with CI:**
```yaml
name: Benchmark CI

on:
  pull_request:
  branches: [main]
  push:
    branches: [main]

jobs:
  benchmark:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - run: cargo bench -- --save-baseline main
      - run: cargo bench -- --baseline main
      - name: Upload benchmark results
        uses: actions/upload-artifact@v3
        with:
          name: benchmark-results
          path: target/criterion/
```

#### 4.2. Regression Detection

**Criteria:**
- Minor regression: 5-10% degradation (investigate)
- Major regression: >10% degradation (block merge)

**Automated Action:**
```yaml
steps:
  - name: Check for regression
    run: |
      cargo bench -- --baseline main
      cargo install cargo-criterion-compare
      cargo-criterion-compare main target/criterion/
```

**Alerting:**
- Create GitHub issue on major regression
- Comment on PR with regression details
- Block merge if major regression detected

---

### 5. Benchmark Reporting

#### 5.1. Report Formats

**HTML Report:**
- Visual comparison charts
- Percentile distribution
- Throughput measurements
- Memory allocation summary

**JSON Report:**
- Machine-readable results
- CI/CD integration
- Historical comparison

#### 5.2. Dashboard Integration

**Grafana Dashboard:**
- Benchmark results panel
- Regression tracking
- Performance trends over time
- Comparison across branches

**Configuration:**
```json
{
  "dashboard": {
    "title": "Tachyon Benchmarks",
    "panels": [
      {
        "title": "Rendering Latency",
        "targets": ["tachyon_render_latency_p99"],
        "datasource": "prometheus"
      },
      {
        "title": "Search Latency",
        "targets": ["tachyon_search_latency_p99"],
        "datasource": "prometheus"
      },
      {
        "title": "Cache Hit Rate",
        "targets": ["tachyon_cache_hit_rate"],
        "datasource": "prometheus"
      }
    ]
  }
}
```

---

## Consequences

### Positive Consequences

1. **Rigorous Benchmarking:** Statistical analysis with confidence intervals
2. **Regression Detection:** Automated detection prevents performance degradation
3. **Load Testing:** Validates system capacity under realistic load
4. **CI/CD Integration:** Continuous performance validation
5. **Comprehensive Coverage:** Micro-benchmarks + load testing + production monitoring

### Negative Consequences

1. **Toolchain Complexity:** Multiple tools require maintenance and training
2. **CI/CD Overhead:** Benchmarking adds ~2-3 minutes to PR builds
3. **Data Management:** Representative test data requires storage and maintenance
4. **False Positives:** Automated regression detection may flag benign changes

---

## Related Documents

- [`performance_requirements.md`](.adrs/ - Performance targets
- [`benchmark_suite.md`](.adrs/ - Detailed benchmark design
- [`optimization_roadmap.md`](.adrs/ - Optimization plan
- [`profiling_strategy.md`](.adrs/adr-031-profiling-strategy.md) - Profiling methodology

---

## References

- Criterion.rs: https://bheisler.github.io/criterion.rs/book/
- K6: https://k6.io/
- wrk2: https://github.com/wg/wrk
- Continuous Benchmarking: https://github.com/bheisler/cargo-criterion-compare
