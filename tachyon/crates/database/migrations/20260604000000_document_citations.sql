CREATE TABLE IF NOT EXISTS document_citations (
    source_id UUID NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
    target_id UUID NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
    context TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (source_id, target_id)
);

CREATE INDEX idx_document_citations_target ON document_citations(target_id);
CREATE INDEX idx_document_citations_source ON document_citations(source_id);
