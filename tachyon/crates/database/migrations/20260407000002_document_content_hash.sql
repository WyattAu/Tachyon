BEGIN;

-- Content hash for integrity checking and deduplication.
-- SHA-256 of the raw markdown content (trimmed, normalized line endings).
-- Invariant: hash(content) == hash(content) always (deterministic).
-- Used to detect: silent corruption, concurrent edits, duplicate documents.
ALTER TABLE documents ADD COLUMN IF NOT EXISTS content_hash TEXT;

-- Index for fast hash lookups (deduplication, sync)
CREATE INDEX IF NOT EXISTS idx_documents_content_hash ON documents(content_hash) WHERE content_hash IS NOT NULL;

-- conflict_detected flag: set to true when content_hash mismatch is detected
-- during sync (e.g., file changed while DB was also updated). Must be manually resolved.
ALTER TABLE documents ADD COLUMN IF NOT EXISTS conflict_detected BOOLEAN NOT NULL DEFAULT false;

COMMIT;
