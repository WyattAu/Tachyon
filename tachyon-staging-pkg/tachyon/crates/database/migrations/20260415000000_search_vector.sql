
-- ============================================================================
-- Search Vector Column
-- Adds a tsvector column for full-text search on the documents table.
-- Used by SearchRepository for ts_rank/ts_headline queries.
-- ============================================================================

-- Add the search_vector column (nullable, populated lazily by rebuild_search_index)
ALTER TABLE documents ADD COLUMN IF NOT EXISTS search_vector TSVECTOR;

-- Create a GIN index for fast tsvector queries
CREATE INDEX IF NOT EXISTS idx_documents_search_vector ON documents USING GIN(search_vector);

-- Create a trigger to auto-update search_vector on INSERT/UPDATE
CREATE OR REPLACE FUNCTION documents_search_vector_update() RETURNS trigger AS $$
BEGIN
    NEW.search_vector :=
        setweight(to_tsvector('english', COALESCE(NEW.title, '')), 'A') ||
        setweight(to_tsvector('english', COALESCE(NEW.description, '')), 'B') ||
        setweight(to_tsvector('english', COALESCE(NEW.content, '')), 'C') ||
        setweight(to_tsvector('english', COALESCE(
            (SELECT string_agg(elem, ' ')
             FROM jsonb_array_elements_text(NEW.tags) AS elem),
            ''
        )), 'B');
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS trg_documents_search_vector ON documents;
CREATE TRIGGER trg_documents_search_vector
    BEFORE INSERT OR UPDATE ON documents
    FOR EACH ROW
    EXECUTE FUNCTION documents_search_vector_update();
