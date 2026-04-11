-- Add outgoing links column to documents table
ALTER TABLE documents ADD COLUMN IF NOT EXISTS outgoing_links JSONB DEFAULT '[]';

-- Create index for backlink lookups
CREATE INDEX IF NOT EXISTS idx_documents_outgoing_links ON documents USING GIN (outgoing_links);
