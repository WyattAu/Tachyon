# ADR-002: BM25 Search Parameters Configuration

## Status

| Status | Accepted |
|---------|----------|
| Date | 2026-02-11 |
| Decision | Adopt BM25 algorithm with k1=1.5, b=0.75 for technical document search |
| Context | Full-Text Search Engine Architecture |

---

## Context and Problem Statement

### Current Situation

Tachyon requires full-text search capabilities for technical documentation repositories. The search engine must:
- Provide relevance-ranked results for queries
- Support 1-10 term queries (max 256 characters)
- Return top 100 results per query
- Handle technical terminology effectively

### Problem

Selecting optimal BM25 parameters is critical for search relevance in technical documentation. Incorrect parameters lead to:
- Poor ranking of technical terms
- Over-emphasis of common words
- Under-emphasis of domain-specific terminology
- Suboptimal user experience

### Constraints

| Constraint | Value | Source |
|------------|--------|---------|
| Max query terms | 1-10 terms | domain_constraints.toml:253-264 |
| Max query length | 256 characters | domain_constraints.toml:264-275 |
| Max results per query | 1000 results | test_vectors.toml:279-290 |
| Query latency | <100ms | test_vectors.toml:254-262 |
| Indexing time per doc | <500ms | domain_constraints.toml:238-248 |

### Research Findings

Multi-lingual research (78 sources across 15 languages) confirms:
- BM25 is universally recommended for technical document retrieval
- k1=1.5, b=0.75 identified as optimal for technical content
- TF-IDF serves as lightweight alternative for small repositories (<1000 documents)

Source: integrated_findings.md:129-151 (BM25 Algorithm Universality)

---

## Decision Drivers

| Factor | Impact | Weight |
|---------|--------|--------|
| Search Relevance | CRITICAL | 50% |
| Performance (SD-RQ-001) | HIGH | 25% |
| Domain Specificity | MEDIUM | 15% |
| Implementation Simplicity | MEDIUM | 10% |

---

## Considered Alternatives

### Alternative 1: TF-IDF (Term Frequency-Inverse Document Frequency)

**Description:** Use TF-IDF scoring without BM25 term saturation.

**Formula:**
```
TF(t, d) = f(t, d) / max_{t' in d} f(t', d)
IDF(t) = log(N / n(t))
TF-IDF(t, d) = TF(t, d) * IDF(t)
```

**Pros:**
- Simple implementation
- Lower computational overhead
- Suitable for small document sets (<1000 documents)

**Cons:**
- No term saturation parameter (k1)
- Poor handling of long documents
- Less effective for technical terminology

**Evaluation:** REJECTED - Insufficient for technical document search

### Alternative 2: BM25 with Default Parameters (k1=1.2, b=0.75)

**Description:** Use standard BM25 parameters from original research.

**Formula:**
```
score(D, Q) = sum(IDF(q_i) * (f(q_i, D) * (k1 + 1)) /
                      (f(q_i, D) + k1 * (1 - b + b * |D| / avgdl)))

k1 = 1.2, b = 0.75
```

**Pros:**
- Original research parameters
- Moderate term saturation
- Good balance for general content

**Cons:**
- k1=1.2 may under-saturate frequent terms
- Not optimized for technical documents

**Evaluation:** REJECTED - Not optimized for technical content

### Alternative 3: BM25 with k1=2.0, b=0.75 (Aggressive)

**Description:** Use aggressive term saturation for maximum term discrimination.

**Formula:**
```
k1 = 2.0, b = 0.75
```

**Pros:**
- Strong term frequency discrimination
- High precision for rare terms

**Cons:**
- Over-emphasizes rare terms
- May miss relevant but common technical terms
- Reduced recall for general queries

**Evaluation:** REJECTED - Too aggressive for technical documentation

### Alternative 4: BM25 with k1=1.5, b=0.75 (SELECTED)

**Description:** Use BM25 parameters optimized for technical document retrieval.

**Formula:**
```
score(D, Q) = sum(IDF(q_i) * (f(q_i, D) * (k1 + 1)) /
                      (f(q_i, D) + k1 * (1 - b + b * |D| / avgdl)))

k1 = 1.5, b = 0.75
```

**Pros:**
- Multi-lingual consensus (15 languages) confirms optimal for technical documents
- k1=1.5 provides good term saturation
- b=0.75 balances document length normalization
- Tantivy supports these parameters natively

**Cons:**
- Requires parameter tuning for different content types
- May not be optimal for non-technical content

**Evaluation:** ACCEPTED - Best balance for technical documentation search

---

## Decision

**Adopt BM25 algorithm with k1=1.5 and b=0.75 for full-text search.**

### Rationale

1. **Research Validation:**
   - Multi-lingual consensus (15 languages) confirms k1=1.5, b=0.75 optimal for technical documents
   - Confidence score: 0.99 (Very High) from integrated_findings.md:129-151
   - Original Robertson and Sparch Jones (1994) research supports this configuration

2. **Parameter Analysis:**
   - k1=1.5: Moderate term saturation
     - Allows term frequency to influence score without overwhelming
     - Balances precision and recall for technical terminology
   - b=0.75: Partial document length normalization
     - Reduces impact of document length on scoring
     - Prevents long documents from dominating results

3. **Technical Document Characteristics:**
   - Domain-specific terminology requires moderate term frequency weighting
   - Technical documentation often has consistent term usage patterns
   - k1=1.5 handles both frequent and rare technical terms appropriately

4. **Tantivy Integration:**
   - Tantivy 0.21.1 supports BM25 natively
   - Parameters are configurable at index time
   - No custom implementation required
   - Query performance: <100ms (test_vectors.toml:254-262)

5. **Fallback Strategy:**
   - TF-IDF available as alternative for small repositories (<1000 documents)
   - Configurable parameters allow tuning for different content types
   - ADR-003 documents fallback criteria

### Trade-offs

| Aspect | Benefit | Cost | Mitigation |
|---------|--------|---------|-------------|
| Relevance | Optimized for technical docs | Less optimal for general content | Configurable parameters |
| Performance | <100ms query time | Indexing overhead | Tantivy optimization |
| Recall | Good for technical terms | May miss general terms | Multi-modal search (future) |

---

## Implementation Plan

### Phase 1: Indexing Configuration

**Tasks:**
- Configure Tantivy index with BM25 tokenizer
- Set k1=1.5, b=0.75 as default parameters
- Implement parameter validation (k1 in [1.2, 2.0], b in [0.0, 1.0])
- Add configuration file support for per-repository tuning

**Traceability:** blue_paper.md:169-185 (SD-001)

**Dependencies:**
- tantivy 0.21.1 (dep_spec/tantivy/dep_spec.toml:1-89)

**Configuration Structure:**
```toml
[search.bm25]
k1 = 1.5  # Term saturation parameter
b = 0.75   # Length normalization parameter
```

### Phase 2: Query Processing

**Tasks:**
- Implement query parsing (1-10 terms, max 256 characters)
- Implement BM25 scoring with configured parameters
- Implement result ranking (descending by score)
- Implement pagination (max 100 results per query)

**Traceability:** blue_paper.md:186-202 (SD-002)

**Query Validation:**
```rust
// domain_constraints.toml:253-264
fn validate_query(query: &str) -> Result<&str, SearchError> {
    let terms: Vec<&str> = query.split_whitespace();
    
    if terms.is_empty() {
        return Err(SearchError::EmptyQuery);
    }
    
    if terms.len() > 10 {
        return Err(SearchError::TooManyTerms(terms.len()));
    }
    
    if query.len() > 256 {
        return Err(SearchError::QueryTooLong(query.len()));
    }
    
    Ok(query.trim())
}
```

### Phase 3: Indexing Performance

**Tasks:**
- Implement batch indexing (100 documents per batch from domain_constraints.toml:227-238)
- Measure indexing time per document (<500ms from domain_constraints.toml:238-248)
- Implement progress tracking for large repositories

**Traceability:** test_vectors.toml:244-254 (Tantivy indexing performance)

### Phase 4: Fallback Strategy

**Tasks:**
- Implement TF-IDF as alternative for repositories <1000 documents
- Add configuration option to select algorithm per repository
- Implement automatic algorithm selection based on document count

**Traceability:** integrated_findings.md:154-170 (TF-IDF as Complementary)

---

## Consequences

### Positive Consequences

1. **Search Relevance:**
   - Optimal ranking for technical document queries
   - k1=1.5 provides good term saturation
   - b=0.75 balances document length effects
   - Meets SD-RQ-001 (<100ms query time)

2. **Performance:**
   - Sub-100ms query time achievable
   - <500ms indexing per document
   - Efficient batch processing

3. **Flexibility:**
   - Configurable parameters for different content types
   - TF-IDF fallback for small repositories
   - Future-proof for alternative algorithms

4. **Research Validation:**
   - Multi-lingual consensus (15 languages)
   - Original 1994 research support
   - 0.99 confidence score

### Negative Consequences

1. **Content Specificity:**
   - k1=1.5, b=0.75 optimized for technical documents
   - May not be optimal for non-technical content
   - Requires parameter tuning for different domains

2. **Configuration Complexity:**
   - Per-repository parameter configuration
   - Users must understand BM25 parameters for tuning
   - Documentation required for parameter selection

3. **Indexing Overhead:**
   - BM25 indexing requires more computation than TF-IDF
   - Larger index size due to term frequency tracking

---

## Monitoring and Validation

### Success Criteria

| Metric | Target | Measurement Method |
|---------|--------|-------------------|
| Query latency | <100ms P95 | test_vectors.toml:254-262 |
| Indexing time per doc | <500ms | test_vectors.toml:244-254 |
| Search relevance | Human evaluation | User testing |
| Result ranking quality | NDCG@10 metric | Search evaluation |
| Precision/Recall | >70% precision, >80% recall | Test evaluation |

### Testing Strategy

1. **Unit Tests:**
   - Test BM25 scoring with known queries
   - Verify parameter validation
   - Test query parsing edge cases

2. **Integration Tests:**
   - Test full search pipeline (index + query + rank)
   - Test with technical document corpus
   - Test pagination and result limits

3. **Performance Tests:**
   - Benchmark query latency for 1-10 term queries
   - Measure indexing time for various document sizes
   - Profile memory usage at scale (100K documents)

4. **Relevance Tests:**
   - Human evaluation of search result quality
   - Compare against TF-IDF baseline
   - Test with domain-specific queries

### Rollback Plan

If BM25 parameters fail to provide adequate search relevance:

1. **Enable Per-Repository Configuration:** Allow parameter tuning per repository
2. **Implement A/B Testing:** Compare k1=1.5 vs. alternative values
3. **Add Relevance Feedback:** User feedback on result quality
4. **Consider Hybrid Approach:** Combine BM25 with semantic search

---

## Related Decisions

- blue_paper.md:186-202 (Search Engine - Indexer and Query Engine)
- integrated_findings.md:129-151 (BM25 Algorithm Universality)
- domain_constraints.toml:292-295 (BM25 ranking parameters)

---

## References

1. **Research Sources:**
   - yellow_paper.md:197-234 (BM25 algorithm)
   - integrated_findings.md:129-151 (BM25 Algorithm Universality)
   - yellow_paper.md:235-252 (TF-IDF as Complementary)

2. **Requirements:**
   - requirements.md:514-528 (SD-RQ-001)
   - requirements.md:529-543 (SD-RQ-002)
   - requirements.md:544-558 (SD-RQ-003)

3. **Domain Constraints:**
   - domain_constraints.toml:253-275 (Query constraints)
   - domain_constraints.toml:238-248 (Indexing constraints)
   - domain_constraints.toml:290-305 (BM25 ranking parameters)

4. **Test Vectors:**
   - test_vectors.toml:203-244 (BM25 single term query)
   - test_vectors.toml:244-254 (Tantivy indexing performance)

5. **Architecture:**
   - blue_paper.md:169-185 (Indexer)
   - blue_paper.md:186-202 (Query Engine)

---

**Document Revision History:**

| Version | Date | Author | Changes |
|---------|-------|--------|---------|
| 1.0 | 2026-02-11 | Initial ADR creation |
