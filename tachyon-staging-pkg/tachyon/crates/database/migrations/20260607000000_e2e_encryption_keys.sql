CREATE TABLE IF NOT EXISTS document_encryption_keys (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    document_id UUID NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
    owner_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    key_algorithm TEXT NOT NULL,
    public_key_fingerprint TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(document_id)
);

CREATE INDEX idx_encryption_keys_owner ON document_encryption_keys(owner_id);
