-- pgvector embeddings: vector search support for documents.
--
-- This migration is fault-tolerant: if pgvector is not installed, the
-- extension creation and vector column/index creation are skipped gracefully.
-- The application will work without vector search; install pgvector and
-- re-run migrations to enable it.
--
-- Install pgvector on Debian/Ubuntu: apt install postgresql-16-pgvector
-- Install on Nix: use postgresqlPackages.postgresql_16_pgvector or build from source

DO $$ BEGIN
    CREATE EXTENSION IF NOT EXISTS vector;
EXCEPTION WHEN OTHERS THEN
    RAISE NOTICE 'pgvector extension not available, skipping vector embedding support (error: %)', SQLERRM;
    RETURN;
END $$;

-- Dimension 768 matches nomic-embed-text (Ollama default).
-- For OpenAI text-embedding-3-small (1536-dim), change to vector(1536).
DO $$ BEGIN
    ALTER TABLE documents ADD COLUMN IF NOT EXISTS embedding vector(768);
    CREATE INDEX IF NOT EXISTS idx_documents_embedding
        ON documents
        USING hnsw (embedding vector_cosine_ops)
        WHERE embedding IS NOT NULL;
EXCEPTION WHEN OTHERS THEN
    RAISE NOTICE 'vector type not available, skipping embedding columns (error: %)', SQLERRM;
END $$;
