CREATE TABLE IF NOT EXISTS activity_events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    actor_id UUID NOT NULL,
    event_type VARCHAR(50) NOT NULL,
    -- event types: document_created, document_updated, document_deleted,
    --             review_created, review_approved, review_rejected, review_commented,
    --             user_joined, team_created
    target_type VARCHAR(50) NOT NULL,  -- 'document', 'review', 'user', 'team'
    target_id UUID NOT NULL,
    description TEXT NOT NULL,
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_activity_events_actor ON activity_events(actor_id);
CREATE INDEX IF NOT EXISTS idx_activity_events_target ON activity_events(target_type, target_id);
CREATE INDEX IF NOT EXISTS idx_activity_events_created ON activity_events(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_activity_events_type ON activity_events(event_type);
