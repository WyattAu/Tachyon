-- Indexes for N+1 query pattern fixes and missing query optimizations

-- Document listing ordered by updated_at (list_all, list_after_cursor)
CREATE INDEX IF NOT EXISTS idx_documents_updated_at
    ON documents(updated_at DESC);

-- Webhook event matching (GIN on events array for active webhooks)
CREATE INDEX IF NOT EXISTS idx_webhooks_events
    ON webhooks USING GIN(events)
    WHERE active = true;

-- Knowledge graph edge lookups by source+target (used by deactivate_edges_for_node, list_edges)
CREATE INDEX IF NOT EXISTS idx_kg_edges_source_target
    ON knowledge_graph_edges(source_id, target_id)
    WHERE is_active = true;
