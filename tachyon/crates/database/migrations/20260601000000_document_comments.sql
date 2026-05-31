-- Document comments system
-- Supports inline comments on document sections (paragraph, heading, code block)
-- with threading (replies) and resolution tracking.

CREATE TABLE IF NOT EXISTS document_comments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    document_id TEXT NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
    parent_id UUID REFERENCES document_comments(id) ON DELETE CASCADE,

    -- Who wrote the comment
    author_id TEXT NOT NULL,

    -- Comment content
    content TEXT NOT NULL,

    -- Anchor in the document (CSS selector, heading ID, line number)
    anchor_type TEXT NOT NULL DEFAULT 'document',  -- 'document', 'heading', 'line', 'selection'
    anchor_value TEXT,                              -- e.g. 'introduction', 'L42', '#my-heading'

    -- Thread depth (root = 0, replies increment)
    depth INTEGER NOT NULL DEFAULT 0,

    -- Status
    is_resolved BOOLEAN NOT NULL DEFAULT FALSE,
    resolved_by TEXT,
    resolved_at TIMESTAMPTZ,

    -- Metadata
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

-- Indexes for common queries
CREATE INDEX idx_comments_document_id ON document_comments(document_id);
CREATE INDEX idx_comments_parent_id ON document_comments(parent_id);
CREATE INDEX idx_comments_author_id ON document_comments(author_id);
CREATE INDEX idx_comments_created_at ON document_comments(created_at DESC);

-- Partial index: unresolved comments per document
CREATE INDEX idx_comments_unresolved ON document_comments(document_id)
    WHERE is_resolved = FALSE AND deleted_at IS NULL;
