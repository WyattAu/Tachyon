-- Fix column types to match code expectations
-- Tags/frontmatter: TEXT[] -> JSONB (code inserts as ::jsonb)
-- Search index: add content_type/weight columns, remove title/tags
-- Author FK: drop constraint (auth system is demo-only, users not persisted to DB)

BEGIN;

-- Drop foreign key constraint on documents.author_id
-- The auth system is demo-only and does not persist users to the database.
-- Re-add this constraint once user persistence is implemented.
ALTER TABLE documents DROP CONSTRAINT IF EXISTS documents_author_id_fkey;

-- Fix documents table: tags and frontmatter should be JSONB
ALTER TABLE documents ALTER COLUMN tags SET DATA TYPE JSONB USING
    CASE WHEN tags IS NULL THEN '[]'::jsonb
         WHEN tags::text = '' THEN '[]'::jsonb
         ELSE ('[' || replace(replace(tags::text, '{', '"'), '}', '"') || ']')::jsonb
    END,
ALTER COLUMN tags SET DEFAULT '[]'::jsonb;

ALTER TABLE documents ALTER COLUMN frontmatter SET DATA TYPE JSONB USING
    CASE WHEN frontmatter IS NULL THEN '{}'::jsonb
         WHEN frontmatter::text = '' THEN '{}'::jsonb
         ELSE frontmatter::jsonb
    END,
ALTER COLUMN frontmatter SET DEFAULT '{}'::jsonb;

-- Fix templates table: tags should be JSONB
ALTER TABLE templates ALTER COLUMN tags SET DATA TYPE JSONB USING
    CASE WHEN tags IS NULL THEN '[]'::jsonb
         WHEN tags::text = '' THEN '[]'::jsonb
         ELSE ('[' || replace(replace(tags::text, '{', '"'), '}', '"') || ']')::jsonb
    END,
ALTER COLUMN tags SET DEFAULT '[]'::jsonb;

-- Fix projects table: tags should be JSONB
ALTER TABLE projects ALTER COLUMN tags SET DATA TYPE JSONB USING
    CASE WHEN tags IS NULL THEN '[]'::jsonb
         WHEN tags::text = '' THEN '[]'::jsonb
         ELSE ('[' || replace(replace(tags::text, '{', '"'), '}', '"') || ']')::jsonb
    END,
ALTER COLUMN tags SET DEFAULT '[]'::jsonb;

-- Fix components table: tags should be JSONB
ALTER TABLE components ALTER COLUMN tags SET DATA TYPE JSONB USING
    CASE WHEN tags IS NULL THEN '[]'::jsonb
         WHEN tags::text = '' THEN '[]'::jsonb
         ELSE ('[' || replace(replace(tags::text, '{', '"'), '}', '"') || ']')::jsonb
    END,
ALTER COLUMN tags SET DEFAULT '[]'::jsonb;

-- Fix search_index table: restructure to EAV pattern used by code
-- Code expects: id, document_id, content_type, content, weight, created_at, updated_at
-- Old schema had: id, document_id, content, title, tags, created_at, updated_at

-- Drop old indexes
DROP INDEX IF EXISTS idx_search_index_tags;
DROP INDEX IF EXISTS idx_search_index_title_fts;

-- Add missing columns
ALTER TABLE search_index ADD COLUMN IF NOT EXISTS content_type TEXT NOT NULL DEFAULT '';
ALTER TABLE search_index ADD COLUMN IF NOT EXISTS weight FLOAT NOT NULL DEFAULT 1.0;

-- Remove old columns
ALTER TABLE search_index DROP COLUMN IF EXISTS title;
ALTER TABLE search_index DROP COLUMN IF EXISTS tags;

-- Add NOT NULL constraint to content_type
ALTER TABLE search_index ALTER COLUMN content_type SET NOT NULL;
ALTER TABLE search_index ALTER COLUMN content_type SET DEFAULT '';

-- Add new indexes
CREATE INDEX IF NOT EXISTS idx_search_index_content_type ON search_index(content_type);

COMMIT;
