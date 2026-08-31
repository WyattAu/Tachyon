-- CRDT document state persistence
-- Stores binary Yrs document state for real-time collaboration persistence

CREATE TABLE IF NOT EXISTS crdt_documents (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    document_id UUID NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
    -- Binary Yrs document state (encoded via Doc::encode_state_as_update)
    state_vector BYTEA NOT NULL DEFAULT '\x',
    -- Full document state (encoded via Doc::encode_state_as_update from empty)
    state BYTEA NOT NULL DEFAULT '\x',
    -- Version counter incremented on each flush
    version BIGINT NOT NULL DEFAULT 1,
    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    -- One CRDT state per document
    UNIQUE (document_id)
);

-- Index for fast lookups by document_id
CREATE INDEX IF NOT EXISTS idx_crdt_documents_document_id ON crdt_documents(document_id);

-- CRDT update log for incremental sync
-- Stores individual Yrs updates (deltas) for delta encoding and history
CREATE TABLE IF NOT EXISTS crdt_updates (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    document_id UUID NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
    -- Binary Yrs update (encoded via Update::encode)
    update BYTEA NOT NULL,
    -- Client that sent the update (nullable for server-generated)
    client_id UUID REFERENCES users(id) ON DELETE SET NULL,
    -- Sequence number for ordering (server-assigned, monotonically increasing)
    seq BIGINT NOT NULL,
    -- Timestamp
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Index for fetching updates in order for a document
CREATE INDEX IF NOT EXISTS idx_crdt_updates_doc_seq ON crdt_updates(document_id, seq);

-- Garbage collection: index for finding old updates to prune
CREATE INDEX IF NOT EXISTS idx_crdt_updates_created_at ON crdt_updates(document_id, created_at);
