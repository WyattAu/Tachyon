-- Presence tracking for real-time collaboration
-- Stores ephemeral user presence per document with TTL-based expiry

CREATE TABLE IF NOT EXISTS document_presence (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id         UUID NOT NULL,
    user_name       TEXT NOT NULL DEFAULT '',
    document_id     UUID NOT NULL,
    status          TEXT NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'idle', 'away')),
    cursor_section  TEXT,
    cursor_line     INTEGER,
    cursor_selection TEXT,
    connected_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_seen_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- A user can only have one presence record per document
    UNIQUE (user_id, document_id),

    -- FK to documents table
    CONSTRAINT fk_presence_document FOREIGN KEY (document_id)
        REFERENCES documents(id) ON DELETE CASCADE
);

-- Index for fast lookups by document (most common query)
CREATE INDEX IF NOT EXISTS idx_presence_document
    ON document_presence (document_id, last_seen_at DESC);

-- Index for TTL cleanup
CREATE INDEX IF NOT EXISTS idx_presence_last_seen
    ON document_presence (last_seen_at);

-- Index for user-level queries
CREATE INDEX IF NOT EXISTS idx_presence_user
    ON document_presence (user_id);
