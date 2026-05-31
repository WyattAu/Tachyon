-- Registered OAuth2 applications (third-party integrations)
CREATE TABLE IF NOT EXISTS registered_applications (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT NOT NULL,
    description TEXT,
    owner_id UUID NOT NULL REFERENCES users(id),
    client_id TEXT NOT NULL UNIQUE,
    client_secret_hash TEXT NOT NULL,
    callback_urls TEXT[] NOT NULL DEFAULT '{}',
    scopes TEXT[] NOT NULL DEFAULT '{"read"}',
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    revoked_at TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS idx_registered_applications_client_id ON registered_applications(client_id);
CREATE INDEX IF NOT EXISTS idx_registered_applications_owner_id ON registered_applications(owner_id);

-- Add scopes and revoked_at to existing api_keys table
ALTER TABLE api_keys ADD COLUMN IF NOT EXISTS scopes TEXT[] NOT NULL DEFAULT '{"read"}';
ALTER TABLE api_keys ADD COLUMN IF NOT EXISTS revoked_at TIMESTAMPTZ;
