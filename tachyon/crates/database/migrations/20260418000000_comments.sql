-- Document comments with inline anchors
CREATE TABLE IF NOT EXISTS document_comments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    document_id UUID NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
    author_id UUID NOT NULL,
    author_name VARCHAR(255) NOT NULL DEFAULT '',
    content TEXT NOT NULL,
    anchor_section VARCHAR(255),
    anchor_line_start INTEGER,
    anchor_line_end INTEGER,
    anchor_selection TEXT,
    status VARCHAR(20) NOT NULL DEFAULT 'open',
    parent_id UUID REFERENCES document_comments(id) ON DELETE CASCADE,
    mentions JSONB NOT NULL DEFAULT '[]'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    resolved_at TIMESTAMPTZ,
    resolved_by UUID
);

CREATE INDEX IF NOT EXISTS idx_comments_document_id ON document_comments(document_id);
CREATE INDEX IF NOT EXISTS idx_comments_author_id ON document_comments(author_id);
CREATE INDEX IF NOT EXISTS idx_comments_parent_id ON document_comments(parent_id);
CREATE INDEX IF NOT EXISTS idx_comments_status ON document_comments(status);
CREATE INDEX IF NOT EXISTS idx_comments_created_at ON document_comments(created_at DESC);

ALTER TABLE documents ADD COLUMN IF NOT EXISTS comment_count INTEGER NOT NULL DEFAULT 0;
