-- Document branches for collaborative editing workflow
CREATE TABLE IF NOT EXISTS document_branches (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    document_id UUID NOT NULL REFERENCES documents(id),
    branch_name TEXT NOT NULL,
    source_content TEXT NOT NULL,
    source_content_hash TEXT NOT NULL,
    source_version INTEGER NOT NULL DEFAULT 1,
    branched_by UUID REFERENCES users(id),
    status TEXT NOT NULL DEFAULT 'open' CHECK (status IN ('open', 'merged', 'closed', 'abandoned')),
    merged_at TIMESTAMPTZ,
    merged_by UUID REFERENCES users(id),
    merge_conflict TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(document_id, branch_name)
);

CREATE INDEX idx_document_branches_document_id ON document_branches(document_id);
CREATE INDEX idx_document_branches_status ON document_branches(status);
