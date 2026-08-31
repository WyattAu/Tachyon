CREATE TABLE IF NOT EXISTS usage_records (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID REFERENCES organizations(id) ON DELETE CASCADE,
    metric_type VARCHAR(64) NOT NULL,
    value BIGINT NOT NULL DEFAULT 1,
    recorded_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_usage_records_org_time ON usage_records(org_id, recorded_at DESC);
CREATE INDEX IF NOT EXISTS idx_usage_records_metric ON usage_records(org_id, metric_type, recorded_at DESC);
