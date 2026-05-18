-- Performance indexes for common query patterns
-- Identified during Phase 4.3 query optimization analysis

-- Document listing by author (common in user profile pages)
CREATE INDEX IF NOT EXISTS idx_documents_author_updated
    ON documents(author_id, updated_at DESC);

-- Document listing by project (project-scoped views)
CREATE INDEX IF NOT EXISTS idx_documents_project_updated
    ON documents(project_id, updated_at DESC)
    WHERE project_id IS NOT NULL;

-- Document listing by status (admin dashboards, published content)
CREATE INDEX IF NOT EXISTS idx_documents_status_updated
    ON documents(status, updated_at DESC);

-- Document listing by visibility (public content listing)
CREATE INDEX IF NOT EXISTS idx_documents_visibility_updated
    ON documents(visibility, updated_at DESC)
    WHERE visibility = 'public';

-- Activity feed queries (target_id + target_type pattern)
CREATE INDEX IF NOT EXISTS idx_activity_events_target_type_created
    ON activity_events(target_id, target_type, created_at DESC);

-- Comments by document (document detail page)
CREATE INDEX IF NOT EXISTS idx_document_comments_document_created
    ON document_comments(document_id, created_at DESC);

-- Document versions by document (version history)
CREATE INDEX IF NOT EXISTS idx_document_versions_document_created
    ON document_versions(document_id, created_at DESC);

-- User sessions cleanup (expired session removal)
CREATE INDEX IF NOT EXISTS idx_sessions_expires_at
    ON sessions(expires_at)
    WHERE status = 'active';

-- Refresh tokens by user (token management)
CREATE INDEX IF NOT EXISTS idx_refresh_tokens_user_expires
    ON refresh_tokens(user_id, expires_at DESC);
