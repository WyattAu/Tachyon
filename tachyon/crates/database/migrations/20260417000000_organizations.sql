
-- ============================================================================
-- Organizations & Multi-tenancy
-- Top-level grouping for teams/companies; spaces and documents are scoped to orgs
-- ============================================================================

-- Organizations table
CREATE TABLE IF NOT EXISTS organizations (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name TEXT NOT NULL,
    slug TEXT NOT NULL UNIQUE,
    description TEXT,
    icon TEXT DEFAULT 'building',
    logo_url TEXT,
    owner_id UUID NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    default_role TEXT NOT NULL DEFAULT 'viewer'
        CHECK (default_role IN ('admin', 'editor', 'viewer')),
    max_members INT NOT NULL DEFAULT -1,
    is_personal BOOLEAN NOT NULL DEFAULT false,
    settings JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_organizations_slug ON organizations(slug);
CREATE INDEX IF NOT EXISTS idx_organizations_owner_id ON organizations(owner_id);

-- Organization members: controls who belongs to an org and their role
CREATE TABLE IF NOT EXISTS organization_members (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    role TEXT NOT NULL DEFAULT 'member'
        CHECK (role IN ('owner', 'admin', 'editor', 'viewer')),
    joined_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    invited_by UUID REFERENCES users(id) ON DELETE SET NULL,
    CONSTRAINT org_member_unique UNIQUE (organization_id, user_id)
);

CREATE INDEX IF NOT EXISTS idx_org_members_user_id ON organization_members(user_id);
CREATE INDEX IF NOT EXISTS idx_org_members_org_id ON organization_members(organization_id);

-- Add organization_id to spaces (nullable for backward compatibility)
ALTER TABLE spaces ADD COLUMN IF NOT EXISTS organization_id UUID REFERENCES organizations(id) ON DELETE SET NULL;

CREATE INDEX IF NOT EXISTS idx_spaces_organization_id ON spaces(organization_id) WHERE organization_id IS NOT NULL;

-- Add organization_id to documents (nullable for backward compatibility)
ALTER TABLE documents ADD COLUMN IF NOT EXISTS organization_id UUID REFERENCES organizations(id) ON DELETE SET NULL;

CREATE INDEX IF NOT EXISTS idx_documents_organization_id ON documents(organization_id) WHERE organization_id IS NOT NULL;

-- Function to auto-create a personal organization for new users
CREATE OR REPLACE FUNCTION ensure_personal_organization()
RETURNS TRIGGER AS $$
BEGIN
    INSERT INTO organizations (name, slug, description, icon, owner_id, default_role, is_personal, settings)
    VALUES (
        'Personal',
        'personal-' || NEW.id::text,
        'Your personal workspace',
        'user',
        NEW.id,
        'admin',
        true,
        '{"auto_create": true}'
    )
    ON CONFLICT (slug) DO NOTHING;

    -- Add the user as owner of their personal organization
    INSERT INTO organization_members (organization_id, user_id, role, invited_by)
    SELECT o.id, NEW.id, 'owner', NULL
    FROM organizations o
    WHERE o.slug = 'personal-' || NEW.id::text
    ON CONFLICT (organization_id, user_id) DO NOTHING;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Trigger to auto-create personal organization when a new user registers
DROP TRIGGER IF EXISTS trg_ensure_personal_organization ON users;
CREATE TRIGGER trg_ensure_personal_organization
    AFTER INSERT ON users
    FOR EACH ROW
    EXECUTE FUNCTION ensure_personal_organization();
