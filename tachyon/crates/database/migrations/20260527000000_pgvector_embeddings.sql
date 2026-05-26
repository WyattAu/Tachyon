CREATE EXTENSION IF NOT EXISTS vector;

ALTER TABLE documents ADD COLUMN IF NOT EXISTS embedding vector(1536);

CREATE INDEX IF NOT EXISTS idx_documents_embedding
    ON documents
    USING hnsw (embedding vector_cosine_ops)
    WHERE embedding IS NOT NULL;
