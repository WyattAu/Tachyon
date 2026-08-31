
-- Knowledge graph nodes
CREATE TABLE IF NOT EXISTS knowledge_graph_nodes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    node_type TEXT NOT NULL DEFAULT 'concept',
    name TEXT NOT NULL,
    slug TEXT,
    description TEXT,
    content TEXT,
    visibility TEXT NOT NULL DEFAULT 'private',
    weight FLOAT NOT NULL DEFAULT 1.0,
    properties JSONB NOT NULL DEFAULT '{}',
    project_id UUID,
    document_id UUID,
    created_by UUID,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Knowledge graph edges
CREATE TABLE IF NOT EXISTS knowledge_graph_edges (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    source_id UUID NOT NULL REFERENCES knowledge_graph_nodes(id) ON DELETE CASCADE,
    target_id UUID NOT NULL REFERENCES knowledge_graph_nodes(id) ON DELETE CASCADE,
    edge_type TEXT NOT NULL DEFAULT 'related_to',
    label TEXT,
    description TEXT,
    weight FLOAT NOT NULL DEFAULT 1.0,
    confidence FLOAT,
    properties JSONB NOT NULL DEFAULT '{}',
    project_id UUID,
    created_by UUID,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes for traversal queries
CREATE INDEX IF NOT EXISTS idx_kg_nodes_type ON knowledge_graph_nodes(node_type);
CREATE INDEX IF NOT EXISTS idx_kg_nodes_project ON knowledge_graph_nodes(project_id) WHERE project_id IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_kg_nodes_document ON knowledge_graph_nodes(document_id) WHERE document_id IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_kg_nodes_active ON knowledge_graph_nodes(is_active) WHERE is_active = true;
CREATE INDEX IF NOT EXISTS idx_kg_nodes_slug ON knowledge_graph_nodes(slug) WHERE slug IS NOT NULL;

CREATE INDEX IF NOT EXISTS idx_kg_edges_source ON knowledge_graph_edges(source_id);
CREATE INDEX IF NOT EXISTS idx_kg_edges_target ON knowledge_graph_edges(target_id);
CREATE INDEX IF NOT EXISTS idx_kg_edges_type ON knowledge_graph_edges(edge_type);
CREATE INDEX IF NOT EXISTS idx_kg_edges_project ON knowledge_graph_edges(project_id) WHERE project_id IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_kg_edges_active ON knowledge_graph_edges(is_active) WHERE is_active = true;

-- Prevent duplicate edges (same source+target+type)
CREATE UNIQUE INDEX IF NOT EXISTS idx_kg_edges_unique ON knowledge_graph_edges(source_id, target_id, edge_type) WHERE is_active = true;
