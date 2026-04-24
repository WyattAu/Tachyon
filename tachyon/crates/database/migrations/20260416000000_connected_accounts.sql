-- Connected accounts (OAuth2 provider links)
-- Tracks which OAuth2 providers a user has connected to their account.
-- A user can have multiple providers linked (e.g., Google + GitHub).

CREATE TABLE IF NOT EXISTS connected_accounts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    provider VARCHAR(50) NOT NULL,         -- 'google', 'github', etc.
    provider_user_id VARCHAR(255) NOT NULL, -- User ID at the provider
    provider_email VARCHAR(255),            -- Email from provider
    provider_username VARCHAR(255),         -- Username from provider
    avatar_url TEXT,                        -- Avatar URL from provider
    access_token TEXT,                      -- Encrypted provider access token (optional)
    refresh_token TEXT,                     -- Encrypted provider refresh token (optional)
    token_expires_at TIMESTAMPTZ,           -- When the access token expires
    connected_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_used_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT uq_connected_accounts_provider UNIQUE (provider, provider_user_id)
);

-- Index for looking up a user's connected accounts
CREATE INDEX IF NOT EXISTS idx_connected_accounts_user_id ON connected_accounts(user_id);

-- Index for looking up by provider + provider_user_id (OAuth callback lookup)
CREATE INDEX IF NOT EXISTS idx_connected_accounts_provider ON connected_accounts(provider, provider_user_id);
