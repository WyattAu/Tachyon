-- Temporal graph support: add deactivated_at timestamps for point-in-time queries
-- An edge/node is "active at time T" when: created_at <= T AND (deactivated_at IS NULL OR deactivated_at > T)

ALTER TABLE knowledge_graph_nodes
    ADD COLUMN IF NOT EXISTS deactivated_at TIMESTAMPTZ;

ALTER TABLE knowledge_graph_edges
    ADD COLUMN IF NOT EXISTS deactivated_at TIMESTAMPTZ;

-- Index for temporal range scans: find nodes/edges active at a given time
CREATE INDEX IF NOT EXISTS idx_kg_nodes_temporal
    ON knowledge_graph_nodes (created_at, deactivated_at)
    WHERE is_active = true;

CREATE INDEX IF NOT EXISTS idx_kg_edges_temporal
    ON knowledge_graph_edges (created_at, deactivated_at)
    WHERE is_active = true;

-- Update deactivate operations to set deactivated_at
-- (existing deactivate_node/deactivate_edge will be updated in application code)
