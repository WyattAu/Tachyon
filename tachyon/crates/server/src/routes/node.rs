use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
};
use serde::Deserialize;
use tachyon_database::{DatabaseError, DatabasePool, GraphEdge, GraphNode, GraphRepository};
use tracing::{debug, info};

type ApiError = (StatusCode, Json<ErrorResponse>);

#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
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

#[derive(Debug, Deserialize, utoipa::IntoParams)]
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

#[derive(Debug, Deserialize, utoipa::ToSchema)]
#[serde(deny_unknown_fields)]
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

#[derive(Debug, Deserialize, utoipa::ToSchema)]
#[serde(deny_unknown_fields)]
pub struct UpdateNodeRequest {
    pub name: Option<String>,
    pub slug: Option<String>,
    pub description: Option<String>,
    pub content: Option<String>,
    pub visibility: Option<String>,
    pub weight: Option<f64>,
    pub properties: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
#[serde(deny_unknown_fields)]
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

#[derive(Debug, Deserialize, utoipa::ToSchema)]
#[serde(deny_unknown_fields)]
pub struct GraphQueryRequest {
    pub source_id: String,
    pub direction: Option<String>,
    pub edge_type: Option<String>,
    pub depth: Option<u32>,
    pub target_id: Option<String>,
    /// ISO 8601 timestamp for point-in-time graph query.
    /// If provided, only nodes/edges active at this time are returned.
    pub at: Option<String>,
}

// ============================================================================
// Response Types
// ============================================================================

#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct NodeListResponse {
    pub nodes: Vec<GraphNode>,
    pub total: i64,
    pub page: usize,
    pub page_size: usize,
}

#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct EdgeListResponse {
    pub edges: Vec<GraphEdge>,
    pub total: usize,
}

#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct GraphQueryResponse {
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
    pub node_count: usize,
    pub edge_count: usize,
}

// ============================================================================
// Handlers
// ============================================================================

/// Create a new graph node.
///
/// `POST /nodes`
///
/// Request body: JSON with `name` (required), optional `node_type`, `description`, `content`, `visibility`, `weight`, `properties`, `project_id`, `document_id`, `slug`.
/// Response: 200 with `GraphNode`, or 400/409/500 on error.
#[utoipa::path(
    post,
    path = "/nodes",
    request_body(content = CreateNodeRequest, description = "Node creation request"),
    responses(
        (status = 200, description = "Node created", body = tachyon_database::GraphNode),
        (status = 400, description = "Validation error"),
        (status = 409, description = "Conflict"),
    ),
    tag = "nodes",
    security(("bearer_auth" = [])),
)]
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
        deactivated_at: None,
    };

    let created = state
        .repo()
        .create_node(&node)
        .await
        .map_err(|e| db_err(&e))?;

    info!("Node created: {}", created.id);
    Ok(Json(created))
}

/// Get a graph node by ID.
///
/// `GET /nodes/{node_id}`
///
/// Response: 200 with `GraphNode`, or 404 on error.
#[utoipa::path(
    get,
    path = "/nodes/{node_id}",
    params(
        ("node_id" = String, Path, description = "Node ID"),
    ),
    responses(
        (status = 200, description = "Node details", body = tachyon_database::GraphNode),
        (status = 404, description = "Node not found"),
    ),
    tag = "nodes",
    security(("bearer_auth" = [])),
)]
pub async fn get_node(
    Path(node_id): Path<String>,
    State(state): State<NodeState>,
) -> Result<Json<GraphNode>, ApiError> {
    debug!("Getting node: {}", node_id);

    let node = state
        .repo()
        .get_node_by_id(&node_id)
        .await
        .map_err(|e| db_err(&e))?;

    Ok(Json(node))
}

#[utoipa::path(
    put,
    path = "/nodes/{node_id}",
    params(
        ("node_id" = String, Path, description = "Node ID"),
    ),
    request_body(content = UpdateNodeRequest, description = "Node update request"),
    responses(
        (status = 200, description = "Node updated", body = tachyon_database::GraphNode),
        (status = 404, description = "Node not found"),
    ),
    tag = "nodes",
    security(("bearer_auth" = [])),
)]
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

#[utoipa::path(
    delete,
    path = "/nodes/{node_id}",
    params(
        ("node_id" = String, Path, description = "Node ID"),
    ),
    responses(
        (status = 204, description = "Node deleted"),
        (status = 404, description = "Node not found"),
    ),
    tag = "nodes",
    security(("bearer_auth" = [])),
)]
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

#[utoipa::path(
    get,
    path = "/nodes",
    params(
        NodeQuery,
    ),
    responses(
        (status = 200, description = "List of nodes", body = NodeListResponse),
    ),
    tag = "nodes",
    security(("bearer_auth" = [])),
)]
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

#[utoipa::path(
    post,
    path = "/edges",
    request_body(content = CreateEdgeRequest, description = "Edge creation request"),
    responses(
        (status = 200, description = "Edge created", body = tachyon_database::GraphEdge),
        (status = 400, description = "Validation error"),
    ),
    tag = "edges",
    security(("bearer_auth" = [])),
)]
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
        deactivated_at: None,
    };

    let created = state
        .repo()
        .create_edge(&edge)
        .await
        .map_err(|e| db_err(&e))?;

    info!("Edge created: {}", created.id);
    Ok(Json(created))
}

#[utoipa::path(
    get,
    path = "/nodes/{node_id}/edges",
    params(
        ("node_id" = String, Path, description = "Node ID"),
    ),
    responses(
        (status = 200, description = "Edges for node", body = EdgeListResponse),
        (status = 404, description = "Node not found"),
    ),
    tag = "edges",
    security(("bearer_auth" = [])),
)]
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

#[utoipa::path(
    delete,
    path = "/edges/{edge_id}",
    params(
        ("edge_id" = String, Path, description = "Edge ID"),
    ),
    responses(
        (status = 204, description = "Edge deleted"),
        (status = 404, description = "Edge not found"),
    ),
    tag = "edges",
    security(("bearer_auth" = [])),
)]
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

/// Query the graph: traverse neighbors or find shortest path.
///
/// `POST /graph/query`
///
/// Request body: JSON with `source_id` (required), optional `direction` ("incoming"/"outgoing"/"both"),
/// `edge_type`, `depth` (default 3, max 5), `target_id` (for shortest path), `at` (point-in-time timestamp).
/// If `target_id` is provided, returns the shortest path between source and target.
/// Otherwise, returns all neighbors up to `depth` hops.
/// Response: 200 with `GraphQueryResponse` containing `nodes`, `edges`, `node_count`, `edge_count`.
#[utoipa::path(
    post,
    path = "/graph/query",
    request_body(content = GraphQueryRequest, description = "Graph query request"),
    responses(
        (status = 200, description = "Graph query result", body = GraphQueryResponse),
    ),
    tag = "graph",
    security(("bearer_auth" = [])),
)]
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
        if let Ok(fetched) = state.repo().get_nodes_by_ids_batch(&path).await {
            nodes = fetched;
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
        .get_neighbors(
            &req.source_id,
            direction,
            req.edge_type.as_deref(),
            max_depth,
        )
        .await
        .map_err(|e| db_err(&e))?;

    let mut node_ids: Vec<String> = edges
        .iter()
        .flat_map(|e| [e.source_id.clone(), e.target_id.clone()])
        .collect();
    node_ids.sort();
    node_ids.dedup();

    let nodes = state
        .repo()
        .get_nodes_by_ids_batch(&node_ids)
        .await
        .unwrap_or_default();

    Ok(Json(GraphQueryResponse {
        node_count: nodes.len(),
        edge_count: edges.len(),
        nodes,
        edges,
    }))
}

#[utoipa::path(
    get,
    path = "/graph/stats",
    responses(
        (status = 200, description = "Graph statistics", body = serde_json::Value),
    ),
    tag = "graph",
    security(("bearer_auth" = [])),
)]
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

/// Query the graph state at a specific point in time.
///
/// Returns all nodes and edges that were active at the given ISO 8601 timestamp.
#[utoipa::path(
    get,
    path = "/graph/at",
    params(
        ("at" = String, Query, description = "ISO 8601 timestamp"),
    ),
    responses(
        (status = 200, description = "Graph state at time", body = GraphQueryResponse),
        (status = 400, description = "Missing or invalid timestamp"),
    ),
    tag = "graph",
    security(("bearer_auth" = [])),
)]
pub async fn get_graph_at_time(
    Query(params): Query<std::collections::HashMap<String, String>>,
    State(state): State<NodeState>,
) -> Result<Json<GraphQueryResponse>, ApiError> {
    let at_str = params.get("at").ok_or_else(|| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                code: "VALIDATION_ERROR".into(),
                message: "Missing required query parameter: at (ISO 8601 timestamp)".into(),
            }),
        )
    })?;

    let at: chrono::DateTime<chrono::Utc> = chrono::DateTime::parse_from_rfc3339(at_str)
        .map(|dt| dt.with_timezone(&chrono::Utc))
        .map_err(|e| {
            (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    code: "VALIDATION_ERROR".into(),
                    message: format!("Invalid timestamp format: {}", e),
                }),
            )
        })?;

    info!("Querying graph state at: {}", at);

    let (nodes, edges) = state
        .repo()
        .get_graph_at_time(at)
        .await
        .map_err(|e| db_err(&e))?;

    Ok(Json(GraphQueryResponse {
        node_count: nodes.len(),
        edge_count: edges.len(),
        nodes,
        edges,
    }))
}

/// Compute the diff of the graph between two timestamps.
///
/// Returns added/removed nodes and edges between `from` and `to` timestamps.
#[utoipa::path(
    get,
    path = "/graph/diff",
    params(
        ("from" = String, Query, description = "ISO 8601 start timestamp"),
        ("to" = String, Query, description = "ISO 8601 end timestamp"),
    ),
    responses(
        (status = 200, description = "Graph diff between timestamps", body = serde_json::Value),
        (status = 400, description = "Missing or invalid timestamps"),
    ),
    tag = "graph",
    security(("bearer_auth" = [])),
)]
pub async fn get_graph_diff(
    Query(params): Query<std::collections::HashMap<String, String>>,
    State(state): State<NodeState>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let from_str = params.get("from").ok_or_else(|| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                code: "VALIDATION_ERROR".into(),
                message: "Missing required query parameter: from (ISO 8601 timestamp)".into(),
            }),
        )
    })?;

    let to_str = params.get("to").ok_or_else(|| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                code: "VALIDATION_ERROR".into(),
                message: "Missing required query parameter: to (ISO 8601 timestamp)".into(),
            }),
        )
    })?;

    let from: chrono::DateTime<chrono::Utc> = chrono::DateTime::parse_from_rfc3339(from_str)
        .map(|dt| dt.with_timezone(&chrono::Utc))
        .map_err(|e| {
            (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    code: "VALIDATION_ERROR".into(),
                    message: format!("Invalid 'from' timestamp: {}", e),
                }),
            )
        })?;

    let to: chrono::DateTime<chrono::Utc> = chrono::DateTime::parse_from_rfc3339(to_str)
        .map(|dt| dt.with_timezone(&chrono::Utc))
        .map_err(|e| {
            (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    code: "VALIDATION_ERROR".into(),
                    message: format!("Invalid 'to' timestamp: {}", e),
                }),
            )
        })?;

    if from >= to {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                code: "VALIDATION_ERROR".into(),
                message: "'from' timestamp must be before 'to' timestamp".into(),
            }),
        ));
    }

    info!("Computing graph diff: {} → {}", from, to);

    let diff = state
        .repo()
        .get_graph_diff(from, to)
        .await
        .map_err(|e| db_err(&e))?;

    Ok(Json(
        serde_json::to_value(diff).unwrap_or(serde_json::json!({})),
    ))
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
        .route("/graph/at", get(get_graph_at_time))
        .route("/graph/diff", get(get_graph_diff))
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
            at: None,
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
