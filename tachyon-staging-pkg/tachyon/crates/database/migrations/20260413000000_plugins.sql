CREATE TABLE IF NOT EXISTS plugins (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    description TEXT,
    version VARCHAR(50) NOT NULL,
    author VARCHAR(255),
    homepage VARCHAR(500),
    license VARCHAR(100),
    -- Extension points this plugin hooks into (JSON array of strings)
    -- e.g. ["editor:command", "document:on-save", "sidebar:panel"]
    extension_points JSONB NOT NULL DEFAULT '[]'::jsonb,
    -- Plugin manifest JSON (full tachyon-plugin.toml serialized)
    manifest JSONB,
    -- Runtime type: "builtin", "wasm", "native"
    runtime_type VARCHAR(20) NOT NULL DEFAULT 'wasm',
    -- Path to plugin WASM binary or native library (relative to plugins dir)
    entry_point VARCHAR(500),
    enabled BOOLEAN NOT NULL DEFAULT FALSE,
    installed_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    -- Who installed this plugin
    installed_by VARCHAR(255)
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_plugins_name_version ON plugins(name, version);
CREATE INDEX IF NOT EXISTS idx_plugins_enabled ON plugins(enabled) WHERE enabled;
CREATE INDEX IF NOT EXISTS idx_plugins_runtime_type ON plugins(runtime_type);
