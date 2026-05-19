# ADR-031: Performance Profiling Strategy

**Status:** Accepted
**Date:** 2026-02-11
**Context:** Phase 4 (Performance Engineering)
**Authors:** Performance Engineer Agent

---

## Context

The Tachyon system requires comprehensive performance profiling to:
1. Identify bottlenecks across all components
2. Validate performance requirements are met
3. Detect performance regressions
4. Guide optimization efforts based on data

This ADR documents the profiling methodology, tools, and processes for performance analysis.

---

## Decision

**We implement a multi-layered profiling strategy:**

1. **Development Profiling:** Continuous profiling during development
2. **Benchmark Profiling:** Controlled performance measurements
3. **Production Profiling:** Low-overhead monitoring in production
4. **Profiling Tools:** Toolchain for different profiling needs

### Profiling Layer Hierarchy:

```
┌─────────────────────────────────────────────────────────────┐
│               Production Profiling Layer                │
│  - Metrics collection (minimal overhead)              │
│  - Alerting on threshold violation                 │
├─────────────────────────────────────────────────────────────┤
│              Benchmark Profiling Layer                  │
│  - Controlled environment measurements                  │
│  - Regression detection across versions                 │
├─────────────────────────────────────────────────────────────┤
│              Development Profiling Layer                 │
│  - Deep profiling with detailed instrumentation         │
│  - Hotspot identification for optimization            │
└─────────────────────────────────────────────────────────────┘
```

---

## Alternatives Considered

### Alternative 1: Manual Performance Testing

**Description:** Use manual timing and observation for performance analysis.

**Pros:**
- No tool overhead
- Simple to implement
- No external dependencies

**Cons:**
- Subjective measurements
- No reproducible results
- Difficult to aggregate across team
- Cannot detect subtle regressions
- Time-consuming for comprehensive analysis

**Rejection:** Requires objective, automated measurements.

---

### Alternative 2: Production Profiling Only

**Description:** Only profile in production environment.

**Pros:**
- Real-world performance data
- No artificial workload bias

**Cons:**
- Cannot reproduce issues locally
- Hard to isolate variables
- Production risk during deep profiling
- Limited instrumentation flexibility
- Cannot test before deployment

**Rejection:** Requires pre-deployment profiling capability.

---

### Alternative 3: Single Profiling Tool

**Description:** Use one tool for all profiling needs.

**Pros:**
- Unified profiling data
- Single toolchain to maintain

**Cons:**
- No specialized tools for different use cases
- Compromised profiling quality
- May not support all profiling types
- Vendor lock-in risk

**Rejection:** Requires specialized toolchain for different profiling needs.

---

## Chosen Approach: Multi-Layered Profiling Strategy

**Rationale:**

1. **Tool Specialization:** Different tools for different profiling needs (CPU, memory, locks)
2. **Layer Separation:** Production, benchmark, and development profiling serve different purposes
3. **Reproducibility:** Benchmark layer provides controlled, reproducible measurements
4. **Realism:** Production layer captures real-world performance characteristics
5. **Flexibility:** Development layer allows deep profiling without production risk
6. **Continuous Monitoring:** Production metrics enable ongoing performance awareness

### Profiling Toolchain:

| Layer | Tool | Purpose | Overhead | Output |
|---------|------|-----------|----------|--------|
| Production | Prometheus + Grafana | Metrics collection | <1% | Time series |
| Production | tokio-console | Async task profiling | <2% | Task traces |
| Benchmark | Criterion.rs | Micro-benchmarks | 0% | Statistical |
| Benchmark | K6 | HTTP load testing | 0% | Throughput |
| Benchmark | wrk2 | WebSocket load testing | 0% | Latency |
| Development | flamegraph | CPU flame graphs | 0% | SVG visualization |
| Development | heaptrack | Memory allocation tracking | <5% | Allocation traces |
| Development | dhat | Heap profile visualization | 0% | Flame graph |
| Development | perf | Linux perf events | <1% | System call traces |

---

## Profiling Methodology

### 1. Production Profiling

#### 1.1 Metrics Collection

**Tool:** Prometheus + custom metrics exporter

**Metrics Collected:**
| Component | Metric | Type | Collection Method | Alert Threshold |
|-----------|--------|------|------------------|-----------------|
| Rendering | render_latency_seconds | Histogram | P99 > 0.02s | Alert |
| Rendering | cache_hit_rate | Ratio | < 0.7 | Alert |
| Search | search_latency_seconds | Histogram | P99 > 0.15s | Alert |
| Search | index_latency_seconds | Histogram | P99 > 0.75s | Alert |
| Git | commit_latency_seconds | Histogram | P99 > 2.0s | Alert |
| File Watcher | event_latency_seconds | Histogram | P99 > 0.2s | Alert |
| WebSocket | message_latency_seconds | Histogram | P99 > 0.1s | Alert |
| System | memory_usage_bytes | Gauge | > 80% budget | Alert |
| System | cpu_usage_ratio | Gauge | > 0.8 | Alert |
| Concurrency | lock_wait_time_seconds | Histogram | P99 > 0.01s | Alert |

**Implementation:**
```rust
use prometheus::{Histogram, Registry, IntCounter, IntGauge};
use prometheus_client::registry::Registry;

lazy_static! {
    static ref REGISTRY: Registry = Registry::new();
}

pub struct PerformanceMetrics {
    render_latency: Histogram,
    cache_hit_rate: IntCounter,
    search_latency: Histogram,
}

impl PerformanceMetrics {
    pub fn new() -> Self {
        PerformanceMetrics {
            render_latency: REGISTRY.register_histogram(
                "render_latency_seconds",
                "Document rendering latency"
            ).unwrap(),
            cache_hit_rate: REGISTRY.register_int_counter(
                "cache_hit_rate",
                "Cache hit rate"
            ).unwrap(),
            search_latency: REGISTRY.register_histogram(
                "search_latency_seconds",
                "Search query latency"
            ).unwrap(),
        }
    }

    pub fn record_render(&self, duration: Duration) {
        let seconds = duration.as_secs_f64();
        self.render_latency.observe(seconds);
    }

    pub fn record_cache_hit(&self) {
        self.cache_hit_rate.inc();
    }
}
```

**Traceability:** PF-RQ-004, performance_requirements.md

#### 1.2 tokio-console Profiling

**Purpose:** Profile async task execution and identify blocking operations.

**Configuration:**
```toml
# Tokio console configuration
[[filters]]
target = "tachyon::core::rendering"
[[filters]]
target = "tachyon::core::search"
[[filters]]
target = "tachyon::git::operations"
```

**Metrics Collected:**
| Metric | Description |
|--------|-------------|
| Task poll time | Time waiting in task queue |
| Task duration | Total task execution time |
| Task waker count | Number of wakeups |
| Async op duration | Individual async operation duration |

**Analysis Focus:**
- Identify tasks with excessive poll time
- Detect blocking I/O operations
- Find tasks with long durations

**Traceability:** IF-001, synchronization_design.md

---

### 2. Benchmark Profiling

#### 2.1 Criterion.rs Micro-Benchmarks

**Purpose:** Precise, statistical measurement of individual operations.

**Benchmark Categories:**

| Category | Benchmarks | Measurement | Target |
|-----------|-------------|-------------|---------|
| Rendering | markdown_parse, jit_render, template_render, cache_ops | Latency, throughput | PR-LAT-001, PR-LAT-002 |
| Search | query_bm25, index_document, cache_ops | Latency, throughput | PR-LAT-004, PR-LAT-005 |
| Git | commit, history_fetch, object_cache | Latency, throughput | PR-LAT-007, PR-LAT-008 |
| File Watcher | event_latency, event_throughput | Latency, throughput | PR-LAT-006 |
| WebSocket | message_delivery, broadcast | Latency, throughput | PR-LAT-009 |

**Regression Detection:**
```bash
# Save baseline
cargo bench -- --save-baseline main

# Compare against baseline
cargo bench -- --baseline main

# Generate comparison report
cargo bench -- --output-format html
```

**Acceptance Criteria:**
- No regression > 5% (minor)
- No regression > 10% (major) - investigate required
- Performance improvement > 10% qualifies as optimization success

**Traceability:** benchmark_suite.md, performance_requirements.md

#### 2.2 Load Testing

**K6 HTTP Load Testing:**
```javascript
// k6 script for HTTP endpoint load testing
import http from 'k6/http';
import { check, sleep } from 'k6';

export let options = {
  vus: 500,              // Virtual users
  duration: '5m',        // Test duration
  thresholds: {
    http_req_duration: ['p(95)<0.05'],  // 95th percentile < 50ms
    http_req_failed: ['rate<0.01'],        // Error rate < 1%
  },
};

export default function () {
  let res = http.get('http://localhost:8080/documents', options);

  check(res, {
    'Document list latency is under 50ms': (r) => r.timings.duration < 50,
    'Document list returned 200': (r) => r.status === 200,
  });

  sleep(1);
}
```

**wrk2 WebSocket Load Testing:**
```bash
# WebSocket load testing
wrk -t 4 -c 1000 -d 30s http://localhost:8080/ws

# Parameters:
# -t 4: 4 threads
# -c 1000: 1000 concurrent connections
# -d 30s: 30 second duration
```

**Traceability:** benchmark_suite.md, PR-THR-001 through PR-THR-007

---

### 3. Development Profiling

#### 3.1 CPU Profiling with flamegraph

**Purpose:** Identify CPU hotspots and optimize critical paths.

**Workflow:**
```bash
# 1. Run workload
cargo run --release --bench

# 2. Profile with perf
perf record -F 99 -g --call-graph dwarf -o perf.data ./tachyon

# 3. Generate flamegraph
perf script -i perf.data --no-inliner --no-inline | \
    flamegraph.pl > flamegraph.svg

# 4. Analyze flamegraph
# Open flamegraph.svg in browser to identify hot paths
```

**Analysis Focus:**
- Identify functions with >10% CPU time
- Find unexpected recursion or hot loops
- Analyze cache miss patterns
- Profile SIMD usage opportunities

**Traceability:** optimization_roadmap.md

#### 3.2 Memory Profiling with dhat

**Purpose:** Identify memory allocation patterns and leaks.

**Workflow:**
```bash
# 1. Run workload with dhat
cargo run --release --bench
dhat -t tachyon --event-filter='alloc*/dealloc*' --live -o dhat.json

# 2. Generate dhat report
dhat -i dhat.json --sort-by=self --show

# 3. Analyze allocation sites
# Look for:
# - High allocation counts in hot paths
# - Large allocations in tight loops
# - Memory leak patterns
```

**Analysis Focus:**
- Identify allocations > 1MB in hot paths
- Find repeated allocations (opportunity for pooling)
- Detect memory leaks (allocations without corresponding deallocations)
- Analyze string interning opportunities

**Traceability:** memory_management.md

#### 3.3 Lock Profiling

**Purpose:** Identify lock contention and synchronization bottlenecks.

**Tools:**
- tokio-console (async task waiting)
- Custom lock instrumentation
- Loom (model checking)

**Instrumentation:**
```rust
use std::time::Instant;
use std::sync::Mutex;

struct InstrumentedLock<T> {
    inner: Mutex<T>,
    name: &'static str,
    wait_time_histogram: Histogram,
}

impl<T> InstrumentedLock<T> {
    fn new(name: &'static str, inner: Mutex<T>, histogram: Histogram) -> Self {
        InstrumentedLock { inner, name, wait_time_histogram: histogram }
    }

    fn lock(&self) -> InstrumentedLockGuard<'_, T> {
        let start = Instant::now();
        let guard = self.inner.lock().unwrap();
        let wait_time = start.elapsed();
        self.wait_time_histogram.observe(wait_time.as_secs_f64());
        InstrumentedLockGuard { guard, lock: self }
    }
}

struct InstrumentedLockGuard<'a, 'b, T> {
    guard: std::sync::MutexGuard<'a, T>,
    lock: &'b InstrumentedLock<T>,
}

impl<'a, 'b, T> Drop for InstrumentedLockGuard<'a, 'b, T> {
    fn drop(&mut self) {
        // Guard dropped, lock released
    }
}
```

**Analysis Focus:**
- Identify locks with >10ms average wait time
- Find lock contention patterns
- Analyze lock ordering issues
- Detect potential deadlocks

**Traceability:** synchronization_design.md, thread_safety_analysis.md

---

## Profiling Workflow

### 1. Pre-Development Profiling

**Trigger:** Before optimization work

**Steps:**
1. Run full benchmark suite
2. Profile with flamegraph
3. Profile with dhat
4. Profile locks with custom instrumentation
5. Analyze results and identify top 3 bottlenecks
6. Create optimization plan

**Output:** Profiling report with:
- Identified hotspots
- Memory allocation analysis
- Lock contention analysis
- Optimization recommendations

### 2. Continuous Profiling (CI/CD)

**Trigger:** Every pull request and main branch push

**Steps:**
1. Run Criterion benchmarks
2. Compare against baseline
3. Fail if regression > 5%
4. Generate performance comparison report

**Configuration:**
```yaml
# .github/workflows/bench.yml
name: Benchmark CI

on:
  push:
    branches: [main]
  pull_request:

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
      - uses: actions/upload-artifact@v3
        with:
          name: benchmark-results
          path: target/criterion/
```

### 3. Pre-Release Profiling

**Trigger:** Before v1.0 release

**Steps:**
1. Production profiling with Prometheus metrics
2. Load testing with K6 and wrk2
3. tokio-console profiling under realistic load
4. Analyze production metrics for 7 days
5. Generate final performance report

**Acceptance Criteria:**
- All latency requirements met (performance_requirements.md)
- No critical bottlenecks identified
- Resource utilization within budget
- Performance regression < 5% from baseline

### 4. Production Monitoring

**Trigger:** Continuous in production

**Components:**
- Prometheus metrics server
- Grafana dashboards
- Alert rules
- Performance SLA monitoring

**Dashboard Configuration:**
```json
// Grafana dashboard configuration
{
  "dashboard": {
    "title": "Tachyon Performance",
    "panels": [
      {
        "title": "Rendering Latency",
        "targets": [
          {
            "expr": "histogram_quantile(render_latency_seconds, 0.99)",
            "legendFormat": "P99: {{value}}s"
          }
        ]
      },
      {
        "title": "Search Latency",
        "targets": [
          {
            "expr": "histogram_quantile(search_latency_seconds, 0.99)",
            "legendFormat": "P99: {{value}}s"
          }
        ]
      },
      {
        "title": "Cache Hit Rate",
        "targets": [
          {
            "expr": "rate(cache_hit_total[5m]) / rate(cache_operations_total[5m])",
            "legendFormat": "{{value}}"
          }
        ]
      },
      {
        "title": "Memory Usage",
        "targets": [
          {
            "expr": "process_resident_memory_bytes / 1024 / 1024 / 1024",
            "legendFormat": "{{value}} MB"
          }
        ]
      }
    ]
  }
}
```

**Traceability:** performance_requirements.md

---

## Profiling Best Practices

### 1. Representative Workloads

**Principle:** Profile with realistic workloads, not synthetic benchmarks.

**Guidelines:**
- Use actual document sizes from production (small, medium, large)
- Use realistic query patterns (single term, multi-term, complex)
- Simulate realistic concurrency (10, 100, 1000 concurrent users)
- Use production-like hardware (same CPU architecture, memory configuration)

### 2. Statistical Significance

**Principle:** Use statistical methods to ensure measurements are significant.

**Guidelines:**
- Run benchmarks for at least 10 seconds
- Use P50, P95, P99 percentiles for latency
- Use confidence intervals for comparisons
- Detect regressions with >95% confidence

### 3. Environment Isolation

**Principle:** Profile in isolated environments to avoid noise.

**Guidelines:**
- Disable unnecessary services during profiling
- Use dedicated profiling machine (same specs as production)
- Warm up system before measurements
- Repeat measurements across multiple runs

### 4. Minimal Overhead

**Principle:** Use profiling tools with minimal performance impact.

**Guidelines:**
- Use sampling for production profiling (1-5% sampling rate)
- Use conditional compilation (feature flags) for debug profiling
- Separate profiling code from production code
- Use zero-copy techniques where possible

---

## Consequences

### Positive Consequences

1. **Comprehensive Profiling:** Multi-layered approach covers all profiling needs
2. **Tool Specialization:** Right tool for each profiling task
3. **Continuous Monitoring:** Production metrics enable ongoing performance awareness
4. **Regression Detection:** Automated detection prevents performance degradation
5. **Data-Driven Optimization:** Profiling data guides optimization priorities

### Negative Consequences

1. **Toolchain Complexity:** Multiple tools require maintenance and training
2. **Overhead:** Production profiling adds ~1-2% overhead
3. **False Positives:** Profiling may identify non-critical hotspots
4. **Learning Curve:** Team needs proficiency with multiple profiling tools

---

## Related Documents

- [`performance_requirements.md`](.adrs/ - Performance targets
- [`benchmark_suite.md`](.adrs/ - Benchmark design
- [`optimization_roadmap.md`](.adrs/ - Optimization plan
- [`thread_safety_analysis.md`](.adrs/ - Concurrency analysis
- [`deadlock_analysis.md`](.adrs/ - Lock analysis

---

## References

- Criterion.rs: https://bheisler.github.io/criterion.rs/book/
- Flamegraph: https://github.com/brendangreggert/Flamegraph
- dhat: https://github.com/psanford/dhat
- tokio-console: https://github.com/tokio-rs/console
- Prometheus: https://prometheus.io/
- Grafana: https://grafana.com/
- K6: https://k6.io/
- wrk2: https://github.com/wg/wrk
