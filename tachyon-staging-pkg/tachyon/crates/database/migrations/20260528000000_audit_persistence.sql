-- Audit event persistence and permission audit log
-- Replaces the unused audit_log table with a schema matching AuditEvent

-- Drop the legacy unused audit_log table
DROP TABLE IF EXISTS audit_log;

-- New audit_events table matching the AuditEvent struct from server/src/audit.rs
CREATE TABLE IF NOT EXISTS audit_events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    event_type TEXT NOT NULL,
    severity TEXT NOT NULL DEFAULT 'Low',
    timestamp TIMESTAMPTZ NOT NULL DEFAULT now(),
    actor_id TEXT,
    actor_type TEXT,
    actor_username TEXT,
    target_id TEXT,
    target_type TEXT,
    action TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    ip_address TEXT,
    user_agent TEXT,
    request_id TEXT,
    session_id TEXT,
    device_id TEXT,
    geo_location TEXT,
    metadata JSONB NOT NULL DEFAULT '{}',
    outcome TEXT NOT NULL DEFAULT 'Success',
    correlation_id TEXT
);

-- Indexes for common query patterns
CREATE INDEX IF NOT EXISTS idx_audit_events_timestamp ON audit_events(timestamp DESC);
CREATE INDEX IF NOT EXISTS idx_audit_events_actor_id ON audit_events(actor_id) WHERE actor_id IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_audit_events_event_type ON audit_events(event_type);
CREATE INDEX IF NOT EXISTS idx_audit_events_severity ON audit_events(severity);
CREATE INDEX IF NOT EXISTS idx_audit_events_target ON audit_events(target_type, target_id) WHERE target_id IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_audit_events_outcome ON audit_events(outcome);
CREATE INDEX IF NOT EXISTS idx_audit_events_correlation ON audit_events(correlation_id) WHERE correlation_id IS NOT NULL;

-- Partition hint: old events can be archived via timestamp range
-- Recommended: CREATE TABLE audit_events_archive (LIKE audit_events) AS SELECT * FROM audit_events WHERE timestamp < now() - interval '90 days';

-- Permission audit log table (referenced by rbac.rs AuditLogRepository)
CREATE TABLE IF NOT EXISTS permission_audit_log (
    id BIGSERIAL PRIMARY KEY,
    user_id TEXT NOT NULL,
    session_id TEXT,
    subject_type TEXT,
    subject_id TEXT,
    role TEXT,
    permission TEXT,
    resource_type TEXT,
    resource_id TEXT,
    action TEXT NOT NULL,
    effect TEXT NOT NULL,
    policy_id BIGINT,
    reason TEXT,
    ip_address TEXT,
    timestamp TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_permission_audit_user ON permission_audit_log(user_id);
CREATE INDEX IF NOT EXISTS idx_permission_audit_timestamp ON permission_audit_log(timestamp DESC);
CREATE INDEX IF NOT EXISTS idx_permission_audit_effect ON permission_audit_log(effect);
CREATE INDEX IF NOT EXISTS idx_permission_audit_resource ON permission_audit_log(resource_type, resource_id);
