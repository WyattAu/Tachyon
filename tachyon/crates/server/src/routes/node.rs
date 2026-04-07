use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
};
use serde::Deserialize;
use tachyon_database::{DatabaseError, DatabasePool, GraphEdge, GraphNode, GraphRepository};
use tracing::{debug, info};

type ApiError = (StatusCode, Json<ErrorResponse>);

#[derive(Debug, serde::Serialize)]
pub struct ErrorResponse {
    pub code: String,
    pub message: String,
}

#[derive(Clone)]
pub struct NodeState {
    pub pool: DatabasePool,
}

impl NodeState {
    pub fn new(pool: DatabasePool) -> Self {
        Self { pool }
    }

    fn repo(&self) -> GraphRepository {
        GraphRepository::new(self.pool.clone())
    }
}

fn db_err(e: &DatabaseError) -> ApiError {
    match e {
        DatabaseError::NotFound { .. } => (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                code: "NOT_FOUND".into(),
                message: e.to_string(),
            }),
        ),
        DatabaseError::ValidationError(msg) => (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                code: "VALIDATION_ERROR".into(),
                message: msg.clone(),
            }),
        ),
        DatabaseError::Duplicate { .. } => (
            StatusCode::CONFLICT,
            Json(ErrorResponse {
                code: "CONFLICT".into(),
                message: e.to_string(),
            }),
        ),
        DatabaseError::ConstraintViolation(msg) => (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                code: "CONSTRAINT_VIOLATION".into(),
                message: msg.clone(),
            }),
        ),
        _ => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                code: "INTERNAL_ERROR".into(),
                message: "Internal server error".into(),
            }),
        ),
    }
}

// ============================================================================
// Query Parameters
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct NodeQuery {
    pub page: Option<usize>,
    pub page_size: Option<usize>,
    pub node_type: Option<String>,
    pub search: Option<String>,
    pub project_id: Option<String>,
}

// ============================================================================
// Request Types
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CreateNodeRequest {
    pub name: String,
    pub node_type: Option<String>,
    pub description: Option<String>,
    pub content: Option<String>,
    pub visibility: Option<String>,
    pub weight: Option<f64>,
    pub properties: Option<serde_json::Value>,
    pub project_id: Option<String>,
    pub document_id: Option<String>,
    pub slug: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateNodeRequest {
    pub name: Option<String>,
    pub slug: Option<String>,
    pub description: Option<String>,
    pub content: Option<String>,
    pub visibility: Option<String>,
    pub weight: Option<f64>,
    pub properties: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct CreateEdgeRequest {
    pub source_id: String,
    pub target_id: String,
    pub edge_type: Option<String>,
    pub label: Option<String>,
    pub description: Option<String>,
    pub weight: Option<f64>,
    pub confidence: Option<f64>,
    pub properties: Option<serde_json::Value>,
    pub project_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct GraphQueryRequest {
    pub source_id: String,
    pub direction: Option<String>,
    pub edge_type: Option<String>,
    pub depth: Option<u32>,
    pub target_id: Option<String>,
}

// ============================================================================
// Response Types
// ============================================================================

#[derive(Debug, serde::Serialize)]
pub struct NodeListResponse {
    pub nodes: Vec<GraphNode>,
    pub total: i64,
    pub page: usize,
    pub page_size: usize,
}

#[derive(Debug, serde::Serialize)]
pub struct EdgeListResponse {
    pub edges: Vec<GraphEdge>,
    pub total: usize,
}

#[derive(Debug, serde::Serialize)]
pub struct GraphQueryResponse {
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
    pub node_count: usize,
    pub edge_count: usize,
}

// ============================================================================
// Handlers
// ============================================================================

pub async fn create_node(
    State(state): State<NodeState>,
    Json(req): Json<CreateNodeRequest>,
) -> Result<Json<GraphNode>, ApiError> {
    info!("Creating new node: {}", req.name);

    if req.name.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                code: "VALIDATION_ERROR".into(),
                message: "Name cannot be empty".into(),
            }),
        ));
    }

    let node = GraphNode {
        id: uuid::Uuid::new_v4().to_string(),
        node_type: req.node_type.unwrap_or_else(|| "document".into()),
        name: req.name,
        slug: req.slug,
        description: req.description,
        content: req.content,
        visibility: req.visibility.unwrap_or_else(|| "public".into()),
        weight: req.weight.unwrap_or(1.0),
        properties: req.properties.unwrap_or(serde_json::json!({})),
        project_id: req.project_id,
        document_id: req.document_id,
        created_by: None,
        is_active: true,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    let created = state.repo().create_node(&node).await.map_err(|e| db_err(&e))?;

    info!("Node created: {}", created.id);
    Ok(Json(created))
}

pub async fn get_node(
    Path(node_id): Path<String>,
    State(state): State<NodeState>,
) -> Result<Json<GraphNode>, ApiError> {
    debug!("Getting node: {}", node_id);

    let node = state.repo().get_node_by_id(&node_id).await.map_err(|e| db_err(&e))?;

    Ok(Json(node))
}

pub async fn update_node(
    Path(node_id): Path<String>,
    State(state): State<NodeState>,
    Json(req): Json<UpdateNodeRequest>,
) -> Result<Json<GraphNode>, ApiError> {
    info!("Updating node: {}", node_id);

    let updated = state
        .repo()
        .update_node(
            &node_id,
            req.name.as_deref(),
            req.slug.as_deref(),
            req.description.as_deref(),
            req.content.as_deref(),
            req.visibility.as_deref(),
            req.weight,
            req.properties.as_ref(),
        )
        .await
        .map_err(|e| db_err(&e))?;

    info!("Node updated: {}", node_id);
    Ok(Json(updated))
}

pub async fn delete_node(
    Path(node_id): Path<String>,
    State(state): State<NodeState>,
) -> Result<StatusCode, ApiError> {
    info!("Deleting node: {}", node_id);

    state
        .repo()
        .deactivate_node(&node_id)
        .await
        .map_err(|e| db_err(&e))?;

    info!("Node deactivated: {}", node_id);
    Ok(StatusCode::NO_CONTENT)
}

pub async fn list_nodes(
    Query(query): Query<NodeQuery>,
    State(state): State<NodeState>,
) -> Result<Json<NodeListResponse>, ApiError> {
    debug!("Listing nodes with filters: {:?}", query);

    let page = query.page.unwrap_or(1).max(1);
    let page_size = query.page_size.unwrap_or(20).clamp(1, 100);

    let (nodes, total) = state
        .repo()
        .list_nodes(
            query.node_type.as_deref(),
            query.project_id.as_deref(),
            query.search.as_deref(),
            page,
            page_size,
        )
        .await
        .map_err(|e| db_err(&e))?;

    Ok(Json(NodeListResponse {
        nodes,
        total,
        page,
        page_size,
    }))
}

pub async fn create_edge(
    State(state): State<NodeState>,
    Json(req): Json<CreateEdgeRequest>,
) -> Result<Json<GraphEdge>, ApiError> {
    info!("Creating edge from {} to {}", req.source_id, req.target_id);

    if req.source_id == req.target_id {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                code: "VALIDATION_ERROR".into(),
                message: "Source and target cannot be the same".into(),
            }),
        ));
    }

    state
        .repo()
        .get_node_by_id(&req.source_id)
        .await
        .map_err(|e| db_err(&e))?;

    state
        .repo()
        .get_node_by_id(&req.target_id)
        .await
        .map_err(|e| db_err(&e))?;

    let edge = GraphEdge {
        id: uuid::Uuid::new_v4().to_string(),
        source_id: req.source_id,
        target_id: req.target_id,
        edge_type: req.edge_type.unwrap_or_else(|| "related_to".into()),
        label: req.label,
        description: req.description,
        weight: req.weight.unwrap_or(1.0),
        confidence: req.confidence,
        properties: req.properties.unwrap_or(serde_json::json!({})),
        project_id: req.project_id,
        created_by: None,
        is_active: true,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    let created = state.repo().create_edge(&edge).await.map_err(|e| db_err(&e))?;

    info!("Edge created: {}", created.id);
    Ok(Json(created))
}

pub async fn get_node_edges(
    Path(node_id): Path<String>,
    State(state): State<NodeState>,
) -> Result<Json<EdgeListResponse>, ApiError> {
    debug!("Getting edges for node: {}", node_id);

    let edges = state
        .repo()
        .get_node_edges(&node_id)
        .await
        .map_err(|e| db_err(&e))?;

    let total = edges.len();

    Ok(Json(EdgeListResponse { edges, total }))
}

pub async fn delete_edge(
    Path(edge_id): Path<String>,
    State(state): State<NodeState>,
) -> Result<StatusCode, ApiError> {
    info!("Deleting edge: {}", edge_id);

    state
        .repo()
        .delete_edge(&edge_id)
        .await
        .map_err(|e| db_err(&e))?;

    info!("Edge deleted: {}", edge_id);
    Ok(StatusCode::NO_CONTENT)
}

pub async fn query_graph(
    State(state): State<NodeState>,
    Json(req): Json<GraphQueryRequest>,
) -> Result<Json<GraphQueryResponse>, ApiError> {
    info!("Querying graph from source: {}", req.source_id);

    let max_depth = req.depth.unwrap_or(3).min(5);
    let direction = req.direction.as_deref().unwrap_or("both");

    if let Some(ref target_id) = req.target_id {
        let path = state
            .repo()
            .get_shortest_path(&req.source_id, target_id, max_depth)
            .await
            .map_err(|e| db_err(&e))?;

        let mut nodes = Vec::new();
        for nid in &path {
            match state.repo().get_node_by_id(nid).await {
                Ok(node) => nodes.push(node),
                Err(_) => continue,
            }
        }

        let mut edges = Vec::new();
        for i in 0..path.len().saturating_sub(1) {
            let between = state
                .repo()
                .list_edges(Some(&path[i]), Some(&path[i + 1]), None, None)
                .await
                .map_err(|e| db_err(&e))?;
            edges.extend(between);
        }

        nodes.sort_by(|a, b| a.id.cmp(&b.id));
        edges.sort_by(|a, b| a.id.cmp(&b.id));

        return Ok(Json(GraphQueryResponse {
            node_count: nodes.len(),
            edge_count: edges.len(),
            nodes,
            edges,
        }));
    }

    let edges = state
        .repo()
        .get_neighbors(&req.source_id, direction, req.edge_type.as_deref(), max_depth)
        .await
        .map_err(|e| db_err(&e))?;

    let mut node_ids: Vec<String> = edges
        .iter()
        .flat_map(|e| [e.source_id.clone(), e.target_id.clone()])
        .collect();
    node_ids.sort();
    node_ids.dedup();

    let mut nodes = Vec::new();
    for nid in node_ids {
        match state.repo().get_node_by_id(&nid).await {
            Ok(node) => nodes.push(node),
            Err(_) => continue,
        }
    }

    Ok(Json(GraphQueryResponse {
        node_count: nodes.len(),
        edge_count: edges.len(),
        nodes,
        edges,
    }))
}

pub async fn get_graph_stats(
    State(state): State<NodeState>,
) -> Result<Json<serde_json::Value>, ApiError> {
    debug!("Getting graph stats");

    let stats = state
        .repo()
        .get_graph_stats()
        .await
        .map_err(|e| db_err(&e))?;

    Ok(Json(stats))
}

// ============================================================================
// Router
// ============================================================================

pub fn create_node_router() -> axum::Router<NodeState> {
    use axum::routing::{delete, get, post, put};

    axum::Router::new()
        .route("/nodes", post(create_node))
        .route("/nodes", get(list_nodes))
        .route("/nodes/{node_id}", get(get_node))
        .route("/nodes/{node_id}", put(update_node))
        .route("/nodes/{node_id}", delete(delete_node))
        .route("/edges", post(create_edge))
        .route("/nodes/{node_id}/edges", get(get_node_edges))
        .route("/edges/{edge_id}", delete(delete_edge))
        .route("/graph/query", post(query_graph))
        .route("/graph/stats", get(get_graph_stats))
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_node_request_construction() {
        let req = CreateNodeRequest {
            name: "Test Node".into(),
            node_type: Some("concept".into()),
            description: Some("A test concept".into()),
            content: None,
            visibility: None,
            weight: None,
            properties: None,
            project_id: None,
            document_id: None,
            slug: None,
        };
        assert_eq!(req.name, "Test Node");
        assert_eq!(req.node_type.as_deref(), Some("concept"));
    }

    #[test]
    fn test_create_edge_request_construction() {
        let req = CreateEdgeRequest {
            source_id: "node-1".into(),
            target_id: "node-2".into(),
            edge_type: Some("references".into()),
            label: None,
            description: None,
            weight: None,
            confidence: None,
            properties: None,
            project_id: None,
        };
        assert_eq!(req.source_id, "node-1");
        assert_eq!(req.target_id, "node-2");
    }

    #[test]
    fn test_update_node_request_construction() {
        let req = UpdateNodeRequest {
            name: Some("Updated Name".into()),
            slug: None,
            description: None,
            content: None,
            visibility: None,
            weight: Some(2.5),
            properties: None,
        };
        assert_eq!(req.name.as_deref(), Some("Updated Name"));
        assert_eq!(req.weight, Some(2.5));
    }

    #[test]
    fn test_graph_query_request_construction() {
        let req = GraphQueryRequest {
            source_id: "node-1".into(),
            direction: Some("outgoing".into()),
            edge_type: None,
            depth: Some(2),
            target_id: Some("node-2".into()),
        };
        assert_eq!(req.source_id, "node-1");
        assert_eq!(req.depth, Some(2));
        assert!(req.target_id.is_some());
    }

    #[test]
    fn test_node_list_response_construction() {
        let resp = NodeListResponse {
            nodes: vec![],
            total: 0,
            page: 1,
            page_size: 20,
        };
        assert_eq!(resp.total, 0);
        assert_eq!(resp.page, 1);
    }

    #[test]
    fn test_edge_list_response_construction() {
        let resp = EdgeListResponse {
            edges: vec![],
            total: 0,
        };
        assert_eq!(resp.total, 0);
    }
}
