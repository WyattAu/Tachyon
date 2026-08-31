# AI Integration Architecture Design (G.3)

## 1. Overview

The AI integration layer provides a plugin-based interface to multiple AI providers, enabling semantic search, auto-tagging, content classification, and AI-assisted writing across the Tachyon platform. The design prioritizes privacy, extensibility, and provider-agnostic operation.

Core capabilities:

- Pluggable AI provider interface supporting OpenAI, Anthropic, and local LLMs (Ollama)
- Semantic search via embeddings with hybrid keyword + vector retrieval
- Automatic tagging and classification of documents on write
- AI-assisted writing, summarization, translation, and grammar/style suggestions

## 2. Provider Interface

All AI operations are abstracted behind a single trait, allowing provider swapping without downstream changes.

```rust
#[async_trait]
pub trait AiProvider: Send + Sync {
    async fn complete(
        &self,
        prompt: &str,
        context: &CompletionContext,
    ) -> Result<String>;

    async fn embed(&self, text: &str) -> Result<Vec<f32>>;

    async fn classify(
        &self,
        text: &str,
        labels: &[String],
    ) -> Result<HashMap<String, f32>>;

    fn name(&self) -> &str;
    fn capabilities(&self) -> ProviderCapabilities;
}

pub struct CompletionContext {
    pub system_prompt: Option<String>,
    pub documents: Vec<DocumentSnippet>,
    pub max_tokens: u32,
    pub temperature: f32,
}

pub struct ProviderCapabilities {
    pub completion: bool,
    pub embedding: bool,
    pub classification: bool,
    pub max_context_tokens: u32,
}
```

### Provider Implementations

| Provider | Completion | Embedding | Classification | Notes |
|----------|-----------|-----------|----------------|-------|
| OpenAI   | GPT-4o, GPT-4o-mini | text-embedding-3-small/large | Via prompt | Cloud, pay-per-token |
| Anthropic | Claude 4, Claude 3.5 | Not native (proxy to OpenAI) | Via prompt | Cloud, pay-per-token |
| Ollama   | llama3, mistral, codellama | nomic-embed-text, all-minilm | Via prompt | Local, self-hosted |

Providers are instantiated via a factory pattern:

```rust
pub struct AiProviderFactory;

impl AiProviderFactory {
    pub fn create(config: &AiProviderConfig) -> Result<Box<dyn AiProvider>> {
        match config.provider_type {
            ProviderType::OpenAI => Ok(Box::new(OpenAiProvider::new(config)?)),
            ProviderType::Anthropic => Ok(Box::new(AnthropicProvider::new(config)?)),
            ProviderType::Ollama => Ok(Box::new(OllamaProvider::new(config)?)),
        }
    }
}
```

## 3. Semantic Search

### Embedding Pipeline

On every document write, embeddings are generated asynchronously to avoid blocking the write path.

1. Document content is chunked (512 tokens, 64 token overlap) using a sentence-boundary-aware splitter.
2. Each chunk is embedded via the configured provider's `embed` method.
3. Embeddings are stored alongside chunk metadata (document ID, position, heading context).

### Storage

Two options, selected at deployment time:

- **pgvector** (PostgreSQL extension): Stores embeddings as `vector(1536)` columns. Preferred when avoiding additional infrastructure. Query via SQL `<=>` cosine distance operator.
- **Qdrant**: Dedicated vector database. Preferred at scale (>1M documents). HTTP/gRPC API with filtering, payload indexing, and batch upsert.

```sql
-- pgvector schema
CREATE TABLE document_embeddings (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    document_id UUID NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
    chunk_index INT NOT NULL,
    content     TEXT NOT NULL,
    embedding   vector(1536) NOT NULL,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX ON document_embeddings USING hnsw (embedding vector_cosine_ops);
```

### Hybrid Search

Retrieval combines two signals:

1. **Keyword search** (Tantivy): BM25-ranked results for exact term matching.
2. **Semantic search** (pgvector/Qdrant): Cosine similarity-ranked results for conceptual matching.

Results are merged using **Reciprocal Rank Fusion (RRF)**:

```
RRF_score(d) = SUM(1 / (k + rank_i(d)))
```

where `k = 60` (standard constant) and `rank_i(d)` is the rank of document `d` in result set `i`.

```rust
pub struct HybridSearch {
    tantivy_index: tantivy::Index,
    vector_store: Box<dyn VectorStore>,
    rrf_k: usize,
}

impl HybridSearch {
    pub async fn search(
        &self,
        query: &str,
        limit: usize,
    ) -> Result<Vec<SearchResult>> {
        let keyword_results = self.tantivy_search(query, limit * 3).await?;
        let semantic_results = self.vector_search(query, limit * 3).await?;
        Ok(merge_rrf(keyword_results, semantic_results, self.rrf_k, limit))
    }
}
```

## 4. Auto-Tagging

Triggered on every document save event.

### Pipeline

1. Extract document text (full content or first 2048 tokens for long documents).
2. Call `AiProvider::classify(text, organization_tag_space)` to get label-confidence scores.
3. Filter labels above configurable confidence threshold (default: 0.7).
4. Present suggested tags to the user (inline UI or notification).
5. User accepts, rejects, or modifies suggestions.
6. Accepted tags are written to the document's tag set.

### Learning from Corrections

When a user rejects or modifies an AI-suggested tag:

- The correction is logged as a training signal: `(document_id, suggested_tags, actual_tags)`.
- Corrections accumulate per-organization and are used to fine-tune classification prompts with few-shot examples.
- For local deployments, corrections can optionally fine-tune a local model via LoRA.

## 5. Writing Assistance

All writing features are accessed through a unified assistant API that operates on the current document context.

| Feature | Method | Trigger |
|---------|--------|---------|
| Completion | `AiProvider::complete` with document context as prefix | Keystroke debounce (500ms idle) |
| Summarization | `AiProvider::complete` with summarization prompt | User action (Ctrl+Shift+S) |
| Translation | `AiProvider::complete` with translation prompt + target language | User action |
| Grammar/Style | `AiProvider::complete` with proofreading prompt on selection | User action (Ctrl+Shift+G) |

Context window management: for long documents, only the surrounding 2048 tokens (1024 before, 1024 after cursor) plus the document outline (heading structure) are sent to the provider.

## 6. Configuration

### Per-Organization Settings

```yaml
ai:
  default_provider: ollama          # openai | anthropic | ollama
  providers:
    openai:
      api_key_encrypted: "<vault reference>"
      completion_model: gpt-4o-mini
      embedding_model: text-embedding-3-small
    anthropic:
      api_key_encrypted: "<vault reference>"
      completion_model: claude-sonnet-4-20250514
    ollama:
      endpoint: http://localhost:11434
      completion_model: llama3
      embedding_model: nomic-embed-text
  features:
    semantic_search:
      enabled: true
      provider: ollama              # can differ from default_provider
      storage: pgvector              # pgvector | qdrant
    auto_tagging:
      enabled: true
      confidence_threshold: 0.7
    writing_assistance:
      enabled: true
      completion: true
      summarization: true
      translation: true
      grammar: false
  rate_limits:
    requests_per_minute: 60
    tokens_per_day: 1000000
```

### API Key Management

- API keys are encrypted at rest using AES-256-GCM with a per-organization key derived from the platform master key (via HKDF).
- Keys are never logged or exposed in API responses.
- Key rotation is supported without service interruption (old key remains valid for 24 hours during rotation).

## 7. Privacy

- **No external data transmission by default.** The Ollama (local) provider is the default configuration. All data remains on-premises.
- External providers (OpenAI, Anthropic) must be explicitly enabled per-organization, with an acknowledgment prompt during configuration.
- When external providers are enabled, only the minimum necessary text is transmitted (no metadata, no user PII).
- **Data retention:** Embeddings stored in the database are deleted when the source document is deleted (ON DELETE CASCADE). No AI provider stores Tachyon data beyond the API call lifetime.
- **Audit logging:** All AI provider calls are logged (provider, model, token count, timestamp) for compliance. Content is not logged.

## 8. Implementation Priority

| Phase | Feature | Duration | Dependencies |
|-------|---------|----------|--------------|
| 1 | AiProvider trait + Ollama implementation | 1 week | None |
| 1b | OpenAI implementation | 3 days | Phase 1 |
| 2 | Embedding pipeline + pgvector storage | 2 weeks | Phase 1 |
| 3 | Hybrid search (Tantivy + vector + RRF) | 1.5 weeks | Phase 2 |
| 4 | Auto-tagging pipeline | 1.5 weeks | Phase 1 |
| 5 | Writing assistance (completion, summarization) | 1.5 weeks | Phase 1 |
| 6 | Anthropic implementation | 3 days | Phase 1 |
| 7 | Configuration UI + rate limiting | 1 week | All phases |

**Total estimated effort: 8 weeks** for a single developer. Phases 1-3 (semantic search) are the critical path. Phases 4-7 can proceed in parallel after Phase 1.
