-- Composite indexes for common multi-column query patterns

-- Sessions: WHERE user_id = $1 AND status = 'Active' ORDER BY last_activity DESC
CREATE INDEX IF NOT EXISTS idx_sessions_user_status_activity
    ON sessions(user_id, status, last_activity DESC);

-- Team members: WHERE team_id = $1 AND user_id = $2
CREATE INDEX IF NOT EXISTS idx_team_members_team_user
    ON team_members(team_id, user_id);

-- Space members: WHERE space_id = $1 AND user_id = $2
CREATE INDEX IF NOT EXISTS idx_space_members_space_user
    ON space_members(space_id, user_id);

-- Organization members: WHERE organization_id = $1 AND user_id = $2
CREATE INDEX IF NOT EXISTS idx_org_members_org_user
    ON organization_members(organization_id, user_id);

-- Project members: WHERE project_id = $1 AND user_id = $2
CREATE INDEX IF NOT EXISTS idx_project_members_project_user
    ON project_members(project_id, user_id);

-- Document reviews: WHERE document_id = $1 AND status IN (...)
CREATE INDEX IF NOT EXISTS idx_document_reviews_doc_status
    ON document_reviews(document_id, status);

-- Knowledge graph edges: WHERE source_id = $1 AND edge_type = $2
CREATE INDEX IF NOT EXISTS idx_kg_edges_source_type
    ON knowledge_graph_edges(source_id, edge_type) WHERE is_active = true;

-- Knowledge graph edges: WHERE target_id = $1 AND edge_type = $2
CREATE INDEX IF NOT EXISTS idx_kg_edges_target_type
    ON knowledge_graph_edges(target_id, edge_type) WHERE is_active = true;

-- User roles: WHERE user_id = $1 AND role_id = $2
CREATE INDEX IF NOT EXISTS idx_user_roles_user_role
    ON user_roles(user_id, role_id);

-- Document versions: WHERE document_id = $1 AND version_number = $2
CREATE INDEX IF NOT EXISTS idx_doc_versions_doc_number
    ON document_versions(document_id, version_number);

-- Document presence: WHERE user_id = $1 AND document_id = $2
CREATE INDEX IF NOT EXISTS idx_presence_user_document
    ON document_presence(user_id, document_id);

-- Subscriptions: WHERE organization_id = $1 ORDER BY created_at DESC
CREATE INDEX IF NOT EXISTS idx_subscriptions_org_created
    ON subscriptions(organization_id, created_at DESC);

-- Invoices: WHERE organization_id = $1 ORDER BY invoice_date DESC
CREATE INDEX IF NOT EXISTS idx_invoices_org_date
    ON invoices(organization_id, invoice_date DESC);
