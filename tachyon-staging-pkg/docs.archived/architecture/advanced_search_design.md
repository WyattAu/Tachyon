# Advanced Search Architecture Design (G.7)

## 1. Overview

Advanced search extends the existing Tantivy-based full-text search with hybrid retrieval, natural language query understanding, cross-document relationship discovery, personalized result ranking, and search analytics. The goal is to move from simple keyword matching to an intelligent retrieval system that understands user intent and surfaces relevant documents regardless of lexical overlap.

Core capabilities:

- Hybrid search combining keyword (BM25) and semantic (vector similarity) retrieval
- Natural language query understanding with intent classification and entity extraction
- Cross-document relationship graph for discovering related content
- Personalized ranking based on user behavior and collaborative signals
- Search analytics for monitoring quality and identifying gaps

## 2. Hybrid Search

### Architecture

Two-phase retrieval pipeline:

1. **Candidate generation** -- Run BM25 and vector search in parallel, merge results using Reciprocal Rank Fusion (RRF).
2. **Re-ranking** -- Apply a cross-encoder model to the top-K merged candidates for precision.

### BM25 (Keyword Retrieval)

Continue using the existing Tantivy index. Ensure the following fields are indexed with appropriate weighting:

| Field | Boost |
|-------|-------|
| Title | 3.0 |
| Headings (H1-H3) | 2.0 |
| Body content | 1.0 |
| Tags | 2.5 |
| Author | 1.5 |

### Vector Similarity (Semantic Retrieval)

- Generate embeddings for document chunks (512-token windows with 64-token overlap).
- Store embeddings in a dedicated vector store (qdrant or pgvector).
- Use the same embedding model specified in G.3 (AI Embeddings) if available, otherwise fall back to a lightweight local model (e.g., all-MiniLM-L6-v2).
- Index build: incremental updates on document save/delete via background worker.

### Reciprocal Rank Fusion

```
RRF_score(d) = sum over i of 1 / (k + rank_i(d))
```

where `k` is a smoothing constant (default 60), and `rank_i(d)` is the rank of document `d` in retrieval method `i`. Weights can be configured per retrieval source:

```toml
[search.rrf]
k = 60
weights = { bm25 = 1.0, vector = 1.0 }
```

### Cross-Encoder Re-ranking

- Model: cross-encoder/ms-marco-MiniLM-L-6-v2 or equivalent.
- Re-rank the top 50 fused candidates.
- Score threshold: drop results below configurable minimum to reduce noise.

## 3. Natural Language Query Understanding

### Intent Classification

Classify incoming queries into categories to select appropriate retrieval strategies:

| Intent | Description | Retrieval Strategy |
|--------|-------------|-------------------|
| `search` | Find documents on a topic | Full hybrid search |
| `navigate` | Go to a specific known document | Title/URL exact match first, then search |
| `compare` | Contrast multiple topics | Multi-query retrieval with deduplication |
| `summarize` | Get an overview of a topic | Retrieve top results, pass to LLM for synthesis |

Classification approach: lightweight classifier (logistic regression or small transformer fine-tuned on labeled query data). Must complete within 50ms to avoid adding latency.

### Entity Extraction

Extract structured entities from natural language queries:

- Document titles (fuzzy matching against document index)
- Tags (exact match against tag vocabulary)
- Authors (exact match against user index)
- Date ranges (parse natural date expressions: "last week", "January 2025")

Entities are used as structured filters combined with full-text retrieval.

### Query Rewriting

- Expand abbreviations and acronyms using a domain-specific dictionary.
- Decompose compound queries ("performance and security issues") into sub-queries.
- Generate synonym variants from the domain vocabulary (maintained in a configurable synonym map).

## 4. Cross-Document Relationships

### Document Link Graph

Maintain a directed graph of inter-document relationships:

- **Backlinks**: documents that link to the current document (parsed from markdown/wiki links).
- **References**: documents linked from the current document.
- **Implicit links**: detected via content similarity above a threshold (cosine > 0.85 on document embeddings).

Storage: adjacency list in PostgreSQL, materialized as a JSONB column on the document record for fast reads. Graph rebuilds on document save.

### Related Documents

For each document, pre-compute a list of related documents:

1. Content similarity: cosine similarity of document embeddings, exclude self and explicit links.
2. Co-occurrence: documents frequently returned together in search sessions.
3. Tag overlap: Jaccard similarity on tag sets, weighted by tag specificity (inverse document frequency of tags).

Return up to 10 related documents, sorted by composite score.

### Knowledge Graph Traversal

If G.3 (AI Embeddings) is available, enable knowledge graph traversal:

- Nodes: documents, tags, authors, entities extracted from content.
- Edges: authored-by, tagged-with, references, contains-entity.
- Query-time traversal: given a result document, traverse the graph to surface connected entities and documents that may not have ranked highly in direct retrieval.

## 5. Personalized Ranking

### User Search History

- Store recent search queries and clicked results per user (TTL: 90 days).
- Boost documents the user has previously clicked for similar queries (query similarity via embedding cosine > 0.8).
- Penalize documents the user has explicitly dismissed.

### Collaborative Filtering

- Track query-result pairs across all users with click signals.
- For a given query, boost results that other users with similar query histories found useful.
- Implementation: item-to-item collaborative filtering using a sparse interaction matrix. No real-time training required -- use precomputed similarity scores updated nightly.

### Time Decay

Apply exponential decay to boost signals based on result age:

```
freshness_score = recency_weight * exp(-lambda * age_days)
```

where `lambda` controls decay rate (default: 0.01, configurable per content type). Time-decayed scores are combined with relevance scores in the final ranking.

## 6. Search Analytics

### Query Logging

Log every search query with:

- Query text (hashed for anonymization if PII detected)
- Intent classification result
- Number of results returned
- Response latency
- User ID (hashed)

Retention: 12 months rolling.

### Click-Through Rate Tracking

- Log which results are clicked from the search results page.
- Compute CTR per query-result pair.
- Surface low-CTR results for content quality review.

### Zero-Result Query Identification

- Flag queries that return zero results.
- Categorize: missing content, poor query formulation, index gap.
- Generate a weekly report for content creators to address coverage gaps.

### Popular Queries Dashboard

- Aggregate top queries by frequency over configurable time windows (daily, weekly, monthly).
- Track trending queries (queries with significant week-over-week increase).
- Expose via admin API endpoint.

## 7. Query Suggestions

### Autocomplete

- Index query history as a dedicated Tantivy collection.
- Prefix-match on query text, ranked by frequency and recency.
- Return top 5 suggestions for each keystroke (debounced at 150ms).

### Spelling Correction

- Build a character-level n-gram index from the document corpus.
- Detect likely misspellings using edit distance (Levenshtein <= 2) against the vocabulary.
- Apply the "Did you mean?" pattern: suggest correction when the corrected query yields significantly more results.

### Synonym Expansion

- Maintain a domain-specific synonym map (editable via admin UI or config file).
- At query time, expand each term with its synonyms using OR clauses.
- Synonym expansion applies only to the BM25 leg; vector search naturally captures semantic similarity.

## 8. Implementation Priority

| Phase | Feature | Duration |
|-------|---------|----------|
| 1 | Hybrid search (BM25 + vector + RRF) | 2 weeks |
| 2 | Query understanding (intent, entities, rewriting) | 2 weeks |
| 3 | Cross-document relationships (link graph, related docs) | 2 weeks |
| 4 | Personalized ranking (history, collaborative filtering) | 1 week |
| 5 | Search analytics and query suggestions | 1 week |

**Total estimated effort: 8 weeks.**

Phase 1 is the highest priority and delivers the largest single improvement to search quality. Subsequent phases build incrementally on the retrieval infrastructure established in Phase 1.
