-- Add threading and text-range anchoring columns to document_comments
ALTER TABLE document_comments ADD COLUMN IF NOT EXISTS thread_id UUID REFERENCES document_comments(id) ON DELETE SET NULL;
ALTER TABLE document_comments ADD COLUMN IF NOT EXISTS start_offset INTEGER;
ALTER TABLE document_comments ADD COLUMN IF NOT EXISTS end_offset INTEGER;

CREATE INDEX IF NOT EXISTS idx_comments_thread_id ON document_comments(thread_id);
