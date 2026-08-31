-- Canvas / Whiteboard tables

CREATE TABLE IF NOT EXISTS canvases (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    title TEXT NOT NULL,
    owner_id UUID NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_canvases_owner_id ON canvases(owner_id);

CREATE TABLE IF NOT EXISTS canvas_nodes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    canvas_id UUID NOT NULL REFERENCES canvases(id) ON DELETE CASCADE,
    node_type TEXT NOT NULL,
    data JSONB NOT NULL DEFAULT '{}',
    position_x DOUBLE PRECISION NOT NULL DEFAULT 0.0,
    position_y DOUBLE PRECISION NOT NULL DEFAULT 0.0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_canvas_nodes_canvas_id ON canvas_nodes(canvas_id);

CREATE TABLE IF NOT EXISTS canvas_edges (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    canvas_id UUID NOT NULL REFERENCES canvases(id) ON DELETE CASCADE,
    source_id UUID NOT NULL REFERENCES canvas_nodes(id) ON DELETE CASCADE,
    target_id UUID NOT NULL REFERENCES canvas_nodes(id) ON DELETE CASCADE,
    edge_type TEXT NOT NULL,
    style JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_canvas_edges_canvas_id ON canvas_edges(canvas_id);
CREATE INDEX IF NOT EXISTS idx_canvas_edges_source_id ON canvas_edges(source_id);
CREATE INDEX IF NOT EXISTS idx_canvas_edges_target_id ON canvas_edges(target_id);
