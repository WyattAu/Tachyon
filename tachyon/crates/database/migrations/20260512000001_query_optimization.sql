-- ============================================================================
-- Query optimization composite indexes
-- Targeted at common read patterns: document listing, search, filtering
-- by author/project/status/visibility, membership lookups, and notifications.
--
-- Uses IF NOT EXISTS for idempotent re-runs (safe in migrations).
-- CONCURRENTLY omitted because it cannot be combined with IF NOT EXISTS;
-- these indexes are on tables that receive low write volume during deploy.
-- ============================================================================

-- ---------------------------------------------------------------------------
-- documents: composite indexes for common filter combinations
-- ---------------------------------------------------------------------------

-- Listing published/public docs (WHERE status = $1 AND visibility = $2)
CREATE INDEX IF NOT EXISTS idx_documents_status_visibility
    ON documents(status, visibility);

-- User document list filtered by status (WHERE author_id = $1 AND status = $2)
CREATE INDEX IF NOT EXISTS idx_documents_author_status
    ON documents(author_id, status);

-- Project document list filtered by status (WHERE project_id = $1 AND status = $2)
CREATE INDEX IF NOT EXISTS idx_documents_project_status
    ON documents(project_id, status)
    WHERE project_id IS NOT NULL;

-- Slug unique lookup (WHERE slug = $1) — idempotent if already present
CREATE INDEX IF NOT EXISTS idx_documents_slug
    ON documents(slug);

-- ---------------------------------------------------------------------------
-- document_comments: comment thread ordering
-- ---------------------------------------------------------------------------

-- Comment thread lookup (WHERE document_id = $1 ORDER BY created_at DESC)
CREATE INDEX IF NOT EXISTS idx_document_comments_doc_created
    ON document_comments(document_id, created_at DESC);

-- ---------------------------------------------------------------------------
-- activity_events: user feeds and target-specific queries
-- ---------------------------------------------------------------------------

-- User activity feed (WHERE actor_id = $1 ORDER BY created_at DESC)
CREATE INDEX IF NOT EXISTS idx_activity_events_actor_created
    ON activity_events(actor_id, created_at DESC);

-- Target-specific activity (WHERE target_type = $1 AND target_id = $2)
CREATE INDEX IF NOT EXISTS idx_activity_events_target
    ON activity_events(target_type, target_id);

-- ---------------------------------------------------------------------------
-- notifications: unread notification queries
-- ---------------------------------------------------------------------------

-- Unread notifications sorted by recency (WHERE user_id = $1 AND read = $2 ORDER BY created_at DESC)
CREATE INDEX IF NOT EXISTS idx_notifications_user_read_created
    ON notifications(user_id, read, created_at DESC);

-- ---------------------------------------------------------------------------
-- sessions / refresh_tokens: cleanup by expiration
-- ---------------------------------------------------------------------------

-- Expired session cleanup (WHERE expires_at < NOW())
CREATE INDEX IF NOT EXISTS idx_sessions_expires
    ON sessions(expires_at);

-- Expired token cleanup (WHERE expires_at < NOW())
CREATE INDEX IF NOT EXISTS idx_refresh_tokens_expires_at
    ON refresh_tokens(expires_at);

-- ---------------------------------------------------------------------------
-- team_members / space_members: membership lookups
-- ---------------------------------------------------------------------------

-- Team membership lookup (WHERE team_id = $1 AND user_id = $2)
CREATE INDEX IF NOT EXISTS idx_team_members_team_user
    ON team_members(team_id, user_id);

-- Space membership lookup (WHERE space_id = $1 AND user_id = $2)
CREATE INDEX IF NOT EXISTS idx_space_members_space_user
    ON space_members(space_id, user_id);
