-- Document comments system - add threading columns to existing table
-- The table was created in 20260418000000_comments.sql
-- This migration adds: depth, anchor_type, anchor_value, is_resolved, deleted_at

-- Add new columns that don't exist in the original schema
ALTER TABLE document_comments ADD COLUMN IF NOT EXISTS anchor_type TEXT NOT NULL DEFAULT 'document';
ALTER TABLE document_comments ADD COLUMN IF NOT EXISTS anchor_value TEXT;
ALTER TABLE document_comments ADD COLUMN IF NOT EXISTS depth INTEGER NOT NULL DEFAULT 0;
ALTER TABLE document_comments ADD COLUMN IF NOT EXISTS is_resolved BOOLEAN NOT NULL DEFAULT FALSE;
ALTER TABLE document_comments ADD COLUMN IF NOT EXISTS deleted_at TIMESTAMPTZ;

-- Rename status column to match is_resolved if needed (old table has 'status', new uses 'is_resolved')
-- The old 'status' VARCHAR column and 'resolved_by/ resolved_at' already exist

-- Indexes (IF NOT EXISTS since earlier migration may have created them)
CREATE INDEX IF NOT EXISTS idx_comments_document_id ON document_comments(document_id);
CREATE INDEX IF NOT EXISTS idx_comments_parent_id ON document_comments(parent_id);
CREATE INDEX IF NOT EXISTS idx_comments_author_id ON document_comments(author_id);
CREATE INDEX IF NOT EXISTS idx_comments_created_at ON document_comments(created_at DESC);

-- Partial index: unresolved comments per document
CREATE INDEX IF NOT EXISTS idx_comments_unresolved ON document_comments(document_id)
    WHERE is_resolved = FALSE AND deleted_at IS NULL;
