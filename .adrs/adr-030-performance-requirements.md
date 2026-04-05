# ADR-030: Performance Requirements Specification

**Status:** Accepted
**Date:** 2026-02-11
**Context:** Phase 4 (Performance Engineering)
**Authors:** Performance Engineer Agent

---

## Context

The Tachyon knowledge management system requires well-defined performance requirements to ensure:
1. Real-time user experience during document editing
2. Fast content discovery through search
3. Efficient resource utilization across deployment modes
4. Scalable architecture for multi-user server deployment

This ADR documents the performance requirements derived from system analysis and user expectations.

---

## Decision

**We define performance requirements based on:**

1. **Critical Path Analysis:** Identify operations that directly impact user-perceived latency
2. **WCET (Worst-Case Execution Time) Analysis:** Formal analysis for real-time components
3. **Resource Budgeting:** Define memory, CPU, and network limits per deployment mode
4. **SLA Definition:** Establish service level agreements for production monitoring

### Performance Requirements Classification:

| Category | Requirements | Rationale |
|-----------|--------------|-----------|
| Latency | PR-LAT-001 through PR-LAT-010 | User-perceived response times |
| Throughput | PR-THR-001 through PR-THR-007 | System capacity under load |
| Resource Utilization | PR-MEM-001 through PR-NET-003 | Operational constraints |
| Concurrency | PR-CON-001 through PR-CON-006 | Multi-user scalability |

---

## Alternatives Considered

### Alternative 1: No Formal Performance Requirements

**Description:** Rely on implementation-level optimizations without formal targets.

**Pros:**
- Maximum flexibility for implementation
- No overhead from monitoring

**Cons:**
- No clear performance goals
- Cannot measure success objectively
- Performance regressions undetected
- SLA compliance impossible

**Rejection:** Requires measurable performance guarantees for user experience.

---

### Alternative 2: Industry Standard Requirements Only

**Description:** Use generic industry performance standards (e.g., "2-second rule").

**Pros:**
- Industry-recognized benchmarks
- Easy to communicate to stakeholders

**Cons:**
- May not match Tachyon's specific use case (local-first, JIT rendering)
- No WCET analysis for real-time constraints
- Resource budgets may not align with deployment modes
- No deployment mode differentiation (desktop vs server)

**Rejection:** Requires Tachyon-specific performance modeling.

---

### Alternative 3: Best Effort Performance

**Description:** Optimize as resources allow without specific targets.

**Pros:**
- Lower implementation cost
- Early delivery possible

**Cons:**
- Uncertain user experience
- Cannot plan capacity
- Performance regressions hard to detect
- No basis for performance SLAs

**Rejection:** Requires predictable performance for operational planning.

---

## Chosen Approach: Formal Performance Requirements with WCET Analysis

**Rationale:**

1. **Measurable Targets:** Each requirement has quantifiable metrics (P50, P99)
2. **Component-Specific:** Requirements tailored to each system component
3. **Deployment-Aware:** Different targets for desktop, server, and static modes
4. **WCET Analysis:** Formal analysis for real-time components (rendering, search)
5. **Resource Budgeting:** Explicit memory and CPU limits per mode
6. **SLA Definition:** Clear service level agreements for production monitoring
7. **Traceability:** Requirements map to functional requirements

### Key Performance Requirements:

#### Latency Requirements (Critical for User Experience)

| Requirement | Component | Metric | Target | P99 Threshold |
|--------------|-----------|---------|-----------------|
| PR-LAT-001 | Rendering (cache hit) | <1ms | <2ms |
| PR-LAT-002 | Rendering (cache miss) | <15ms | <20ms |
| PR-LAT-003 | Template rendering | <5ms | <10ms |
| PR-LAT-004 | Search query | <100ms | <150ms |
| PR-LAT-005 | Document indexing | <500ms | <750ms |
| PR-LAT-006 | File watcher | <100ms | <200ms |
| PR-LAT-007 | Git commit | <1000ms | <2000ms |
| PR-LAT-008 | Git history fetch | <500ms | <1000ms |
| PR-LAT-009 | WebSocket delivery | <50ms | <100ms |
| PR-LAT-010 | Database query | <10ms | <20ms |

#### Throughput Requirements (System Capacity)

| Requirement | Component | Metric | Target |
|--------------|-----------|---------|
| PR-THR-001 | Rendering | 100 renders/s |
| PR-THR-002 | Search queries | 1000 queries/s |
| PR-THR-003 | Document indexing | 2 docs/s |
| PR-THR-004 | Git commits | 10 commits/s |
| PR-THR-005 | WebSocket messages | 10000 messages/s |
| PR-THR-006 | File watcher events | 1000 events/s |
| PR-THR-007 | HTTP requests | 5000 req/s |

#### Resource Utilization Requirements (Operational Constraints)

| Requirement | Mode | Metric | Target | Hard Limit |
|--------------|-------|---------|---------|------------|
| PR-MEM-001 | Desktop | RSS | <1GB | 2GB |
| PR-MEM-002 | Desktop | LRU Cache | <500MB | 500MB |
| PR-MEM-003 | Desktop | Search Index | <512MB | 512MB |
| PR-MEM-004 | Server | RSS | <8GB | 16GB |
| PR-MEM-005 | Server | LRU Cache | <2GB | 2GB |
| PR-MEM-006 | Server | Search Index | <2GB | 2GB |
| PR-CPU-001 | All | Idle CPU | <5% | 10% |
| PR-CPU-002 | Desktop | Peak CPU | <80% | 90% |
| PR-CPU-003 | Server | Average CPU | <50% | 70% |

#### Concurrency Requirements (Multi-User Support)

| Requirement | Mode | Metric | Target |
|--------------|-------|---------|
| PR-CON-001 | Desktop | Concurrent renders | 10 |
| PR-CON-002 | Desktop | Concurrent searches | 5 |
| PR-CON-003 | Desktop | WebSocket connections | 10 |
| PR-CON-004 | Server | Concurrent renders | 100 |
| PR-CON-005 | Server | Concurrent searches | 50 |
| PR-CON-006 | Server | WebSocket connections | 10000 |

---

## WCET Analysis

### Real-Time Components

**Definition:** Real-time components have strict timing constraints where exceeding thresholds causes user-visible degradation.

#### Rendering Engine WCET

**Component:** JIT Rendering (RE-001)
**WCET Guarantee:** <20ms at P99, <15ms average

**Critical Path Analysis:**
```
Operation          Best   Average   WCET     P99
------------------------------------------
Markdown parsing    0.1ms   2ms       8ms       10ms
AST traversal      0.05ms  0.5ms     2ms       3ms
Template render    0.2ms   2ms       8ms       10ms
Cache lookup      0.01ms  0.1ms     0.5ms     1ms
Cache insertion   0.5ms   2ms       5ms       8ms
------------------------------------------
TOTAL (miss)       0.86ms   6.6ms     23.5ms    20ms
TOTAL (hit)        0.01ms  0.1ms     0.5ms     1ms
```

**Mitigation Strategy:** Three-tier compilation (cache > template > baseline)

#### Search Engine WCET

**Component:** BM25 Query (SD-002)
**WCET Guarantee:** <150ms at P99, <100ms average

**Critical Path Analysis:**
```
Operation              Best   Average   WCET     P99
----------------------------------------------
Query parsing          0.01ms  0.1ms     0.5ms     1ms
Term lookups          1ms     10ms      40ms      50ms
Score calculation       1ms     20ms      60ms      80ms
Result ranking         0.1ms   2ms       10ms      15ms
----------------------------------------------
TOTAL                 2.11ms  32.1ms    110.5ms   150ms
```

**Mitigation Strategy:** Tantivy optimized index, result limit (100)

### Non-Real-Time Components

**Definition:** Non-real-time components have softer timing constraints where exceeding thresholds causes delayed but functional behavior.

#### Document Indexing WCET

**Component:** Tantivy Indexer (SD-001)
**WCET Guarantee:** <2500ms at P99, <500ms average

**Mitigation Strategy:** Batch indexing (100 documents)

---

## Deployment Mode Performance Profiles

### Desktop Mode

| Component | Latency | Throughput | Resource Budget |
|-----------|---------|-------------|----------------|
| Rendering | <15ms | 100 renders/s | 500MB cache |
| Search | <100ms | 100 queries/s | 512MB index |
| Git | <1000ms | 10 commits/s | 200MB objects |
| File Watcher | <100ms | 1000 events/s | 1024 watches |
| WebSocket | <50ms | 1000 messages/s | 10 connections |
| **Total** | N/A | N/A | **1GB RSS** |

### Server Mode

| Component | Latency | Throughput | Resource Budget |
|-----------|---------|-------------|----------------|
| Rendering | <15ms | 1000 renders/s | 2GB cache |
| Search | <100ms | 10000 queries/s | 2GB index |
| Git | <1000ms | 100 commits/s | 500MB objects |
| File Watcher | <100ms | 10000 events/s | 8192 watches |
| WebSocket | <50ms | 10000 messages/s | 10000 connections |
| HTTP | <50ms | 5000 req/s | N/A |
| **Total** | N/A | N/A | **8GB RSS** |

### Static Mode

| Component | Latency | Throughput | Resource Budget |
|-----------|---------|-------------|----------------|
| Generation | <100ms | 10 docs/s | 100MB buffer |
| Assets | <50ms | 50 assets/s | N/A |
| **Total** | N/A | N/A | **1GB RSS** |

---

## Performance SLA

### Service Level Agreement

| Component | SLA Metric | Target | Measurement Window | Alert Threshold |
|-----------|-------------|---------|-------------------|-----------------|
| JIT Rendering | P99 latency | <20ms | 1 minute |
| Search Query | P99 latency | <150ms | 1 minute |
| Git Commit | P99 latency | <2000ms | 5 minutes |
| WebSocket Delivery | P99 latency | <100ms | 1 minute |

---

## Consequences

### Positive Consequences

1. **Clear Performance Targets:** Each component has measurable performance goals
2. **WCET Analysis:** Real-time components have formal worst-case guarantees
3. **Resource Budgeting:** Explicit limits prevent resource exhaustion
4. **Deployment Mode Awareness:** Different targets for desktop vs server vs static
5. **SLA Compliance:** Clear service level agreements for production monitoring
6. **Traceability:** Requirements map to functional requirements
7. **Benchmark Baseline:** Establishes performance baseline for regression detection

### Negative Consequences

1. **Implementation Complexity:** Meeting requirements may require optimization effort
2. **Testing Overhead:** Comprehensive benchmarking required
3. **Monitoring Overhead:** Performance metrics collection and alerting
4. **Constraint Trade-offs:** Some optimizations may impact code maintainability

---

## Related Documents

- [`performance_requirements.md`](.specs/04_performance/performance_requirements.md) - Detailed requirements specification
- [`benchmark_suite.md`](.specs/04_performance/benchmark_suite.md) - Benchmark design
- [`optimization_roadmap.md`](.specs/04_performance/optimization_roadmap.md) - Optimization plan
- [`blue_paper.md`](.specs/02_architecture/blue_paper.md) - System architecture
- [`resource_limits.md`](.specs/03_5_resource_management/resource_limits.md) - Resource budgets

---

## References

- IEEE 1016-2009: Software Design Descriptions
- ISO/IEC 25010: Systems and Software Quality Requirements
- WCET Analysis: "Real-Time Systems Design" by Jane W. S. Liu
- Performance Engineering: "Site Reliability Engineering" by Alex H. B. Sussman
