BEGIN;

-- ============================================================================
-- Spaces & Document Hierarchy
-- Organizes documents into spaces (like Obsidian vaults or Notion workspaces)
-- ============================================================================

-- Spaces table: top-level containers for documents
CREATE TABLE IF NOT EXISTS spaces (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name TEXT NOT NULL,
    slug TEXT NOT NULL,
    description TEXT,
    icon TEXT DEFAULT 'folder',
    color TEXT DEFAULT '#3B82F6',
    owner_id UUID NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    parent_id UUID REFERENCES spaces(id) ON DELETE SET NULL,
    visibility TEXT NOT NULL DEFAULT 'private'
        CHECK (visibility IN ('private', 'team', 'public')),
    sort_order INT NOT NULL DEFAULT 0,
    is_default BOOLEAN NOT NULL DEFAULT false,
    settings JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT spaces_slug_owner_unique UNIQUE (slug, owner_id)
);

-- Index for owner's spaces listing
CREATE INDEX IF NOT EXISTS idx_spaces_owner_id ON spaces(owner_id);
-- Index for parent-child relationships (nested spaces)
CREATE INDEX IF NOT EXISTS idx_spaces_parent_id ON spaces(parent_id) WHERE parent_id IS NOT NULL;
-- Index for visibility filtering
CREATE INDEX IF NOT EXISTS idx_spaces_visibility ON spaces(visibility);

-- Space members: controls who can access a space
CREATE TABLE IF NOT EXISTS space_members (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    space_id UUID NOT NULL REFERENCES spaces(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    role TEXT NOT NULL DEFAULT 'viewer'
        CHECK (role IN ('owner', 'admin', 'editor', 'viewer')),
    joined_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    invited_by UUID REFERENCES users(id) ON DELETE SET NULL,
    CONSTRAINT space_member_unique UNIQUE (space_id, user_id)
);

CREATE INDEX IF NOT EXISTS idx_space_members_user_id ON space_members(user_id);
CREATE INDEX IF NOT EXISTS idx_space_members_space_id ON space_members(space_id);

-- Add space_id to documents table (nullable for backward compatibility)
ALTER TABLE documents ADD COLUMN IF NOT EXISTS space_id UUID REFERENCES spaces(id) ON DELETE SET NULL;

-- Index for querying documents within a space
CREATE INDEX IF NOT EXISTS idx_documents_space_id ON documents(space_id) WHERE space_id IS NOT NULL;

-- Add sort_order to documents for ordering within spaces
ALTER TABLE documents ADD COLUMN IF NOT EXISTS sort_order INT NOT NULL DEFAULT 0;

-- Function to auto-create a personal space for new users
CREATE OR REPLACE FUNCTION ensure_personal_space()
RETURNS TRIGGER AS $$
BEGIN
    INSERT INTO spaces (name, slug, description, icon, color, owner_id, visibility, is_default, settings)
    VALUES (
        'Personal',
        'personal',
        'Your personal space',
        'user',
        '#6366F1',
        NEW.id,
        'private',
        true,
        '{"auto_create": true}'
    )
    ON CONFLICT (slug, owner_id) DO NOTHING;

    -- Also add the user as owner of their personal space
    INSERT INTO space_members (space_id, user_id, role, invited_by)
    SELECT s.id, NEW.id, 'owner', NULL
    FROM spaces s
    WHERE s.slug = 'personal' AND s.owner_id = NEW.id
    ON CONFLICT (space_id, user_id) DO NOTHING;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Trigger to auto-create personal space when a new user registers
DROP TRIGGER IF EXISTS trg_ensure_personal_space ON users;
CREATE TRIGGER trg_ensure_personal_space
    AFTER INSERT ON users
    FOR EACH ROW
    EXECUTE FUNCTION ensure_personal_space();

COMMIT;
