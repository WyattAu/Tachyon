CREATE EXTENSION IF NOT EXISTS vector;

-- Dimension 768 matches nomic-embed-text (Ollama default).
-- For OpenAI text-embedding-3-small (1536-dim), change to vector(1536).
ALTER TABLE documents ADD COLUMN IF NOT EXISTS embedding vector(768);

CREATE INDEX IF NOT EXISTS idx_documents_embedding
    ON documents
    USING hnsw (embedding vector_cosine_ops)
    WHERE embedding IS NOT NULL;
