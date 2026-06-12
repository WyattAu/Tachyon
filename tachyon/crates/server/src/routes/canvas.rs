//! Canvas API routes
//! CRUD for canvases, nodes, and edges

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use serde::Serialize;
use tracing::info;

use crate::error::ServerError;
use tachyon_database::{
    Canvas, CanvasEdge, CanvasNode, CanvasRepository, CreateCanvasEdgeRequest,
    CreateCanvasNodeRequest, CreateCanvasRequest, DatabasePool, UpdateCanvasEdgeRequest,
    UpdateCanvasNodeRequest, UpdateCanvasRequest,
};

/// Canvas state
#[derive(Clone)]
pub struct CanvasState {
    pub pool: DatabasePool,
}

impl CanvasState {
    pub fn new(pool: DatabasePool) -> Self {
        Self { pool }
    }

    fn repo(&self) -> CanvasRepository {
        CanvasRepository::new(self.pool.clone())
    }
}

// ============================================================================
// Request/Response Types
// ============================================================================

#[derive(Debug, Serialize)]
pub struct CanvasResponse {
    pub canvas: Canvas,
}

#[derive(Debug, Serialize)]
pub struct CanvasNodeResponse {
    pub node: CanvasNode,
}

#[derive(Debug, Serialize)]
pub struct CanvasEdgeResponse {
    pub edge: CanvasEdge,
}

#[derive(Debug, Serialize)]
pub struct CanvasListResponse {
    pub canvases: Vec<Canvas>,
    pub total: usize,
}

#[derive(Debug, Serialize)]
pub struct CanvasFullResponse {
    pub canvas: Canvas,
    pub nodes: Vec<CanvasNode>,
    pub edges: Vec<CanvasEdge>,
}

// ============================================================================
// Handlers — Canvas CRUD
// ============================================================================

/// POST /api/v1/canvases — Create a new canvas
pub async fn create_canvas(
    State(state): State<CanvasState>,
    Json(req): Json<CreateCanvasRequest>,
) -> Result<Json<CanvasResponse>, ServerError> {
    let repo = state.repo();
    let canvas = repo.create_canvas(req).await?;
    info!("Canvas created: {}", canvas.id);
    Ok(Json(CanvasResponse { canvas }))
}

/// GET /api/v1/canvases — List canvases
pub async fn list_canvases(
    State(state): State<CanvasState>,
) -> Result<Json<CanvasListResponse>, ServerError> {
    let repo = state.repo();
    let canvases = repo.list_canvases(None).await?;
    let total = canvases.len();
    Ok(Json(CanvasListResponse { canvases, total }))
}

/// GET /api/v1/canvases/:id — Get a canvas with all nodes and edges
pub async fn get_canvas(
    State(state): State<CanvasState>,
    Path(id): Path<String>,
) -> Result<Json<CanvasFullResponse>, ServerError> {
    let repo = state.repo();
    let canvas = repo.get_canvas_by_id(&id).await?;
    let nodes = repo.list_canvas_nodes(&id).await?;
    let edges = repo.list_canvas_edges(&id).await?;
    Ok(Json(CanvasFullResponse {
        canvas,
        nodes,
        edges,
    }))
}

/// PUT /api/v1/canvases/:id — Update canvas
pub async fn update_canvas(
    State(state): State<CanvasState>,
    Path(id): Path<String>,
    Json(req): Json<UpdateCanvasRequest>,
) -> Result<Json<CanvasResponse>, ServerError> {
    let repo = state.repo();
    let canvas = repo.update_canvas(&id, req).await?;
    Ok(Json(CanvasResponse { canvas }))
}

/// DELETE /api/v1/canvases/:id — Delete canvas
pub async fn delete_canvas(
    State(state): State<CanvasState>,
    Path(id): Path<String>,
) -> Result<StatusCode, ServerError> {
    let repo = state.repo();
    repo.delete_canvas(&id).await?;
    Ok(StatusCode::NO_CONTENT)
}

// ============================================================================
// Handlers — Canvas Node CRUD
// ============================================================================

/// POST /api/v1/canvases/:canvas_id/nodes — Create a node
pub async fn create_node(
    State(state): State<CanvasState>,
    Path(canvas_id): Path<String>,
    Json(req): Json<CreateCanvasNodeRequest>,
) -> Result<Json<CanvasNodeResponse>, ServerError> {
    let repo = state.repo();
    let node = repo.create_canvas_node(&canvas_id, req).await?;
    Ok(Json(CanvasNodeResponse { node }))
}

/// GET /api/v1/canvases/:canvas_id/nodes — List nodes
pub async fn list_nodes(
    State(state): State<CanvasState>,
    Path(canvas_id): Path<String>,
) -> Result<Json<Vec<CanvasNode>>, ServerError> {
    let repo = state.repo();
    let nodes = repo.list_canvas_nodes(&canvas_id).await?;
    Ok(Json(nodes))
}

/// PUT /api/v1/canvases/nodes/:id — Update a node
pub async fn update_node(
    State(state): State<CanvasState>,
    Path(id): Path<String>,
    Json(req): Json<UpdateCanvasNodeRequest>,
) -> Result<Json<CanvasNodeResponse>, ServerError> {
    let repo = state.repo();
    let node = repo.update_canvas_node(&id, req).await?;
    Ok(Json(CanvasNodeResponse { node }))
}

/// DELETE /api/v1/canvases/nodes/:id — Delete a node
pub async fn delete_node(
    State(state): State<CanvasState>,
    Path(id): Path<String>,
) -> Result<StatusCode, ServerError> {
    let repo = state.repo();
    repo.delete_canvas_node(&id).await?;
    Ok(StatusCode::NO_CONTENT)
}

// ============================================================================
// Handlers — Canvas Edge CRUD
// ============================================================================

/// POST /api/v1/canvases/:canvas_id/edges — Create an edge
pub async fn create_edge(
    State(state): State<CanvasState>,
    Path(canvas_id): Path<String>,
    Json(req): Json<CreateCanvasEdgeRequest>,
) -> Result<Json<CanvasEdgeResponse>, ServerError> {
    let repo = state.repo();
    let edge = repo.create_canvas_edge(&canvas_id, req).await?;
    Ok(Json(CanvasEdgeResponse { edge }))
}

/// GET /api/v1/canvases/:canvas_id/edges — List edges
pub async fn list_edges(
    State(state): State<CanvasState>,
    Path(canvas_id): Path<String>,
) -> Result<Json<Vec<CanvasEdge>>, ServerError> {
    let repo = state.repo();
    let edges = repo.list_canvas_edges(&canvas_id).await?;
    Ok(Json(edges))
}

/// PUT /api/v1/canvases/edges/:id — Update an edge
pub async fn update_edge(
    State(state): State<CanvasState>,
    Path(id): Path<String>,
    Json(req): Json<UpdateCanvasEdgeRequest>,
) -> Result<Json<CanvasEdgeResponse>, ServerError> {
    let repo = state.repo();
    let edge = repo.update_canvas_edge(&id, req).await?;
    Ok(Json(CanvasEdgeResponse { edge }))
}

/// DELETE /api/v1/canvases/edges/:id — Delete an edge
pub async fn delete_edge(
    State(state): State<CanvasState>,
    Path(id): Path<String>,
) -> Result<StatusCode, ServerError> {
    let repo = state.repo();
    repo.delete_canvas_edge(&id).await?;
    Ok(StatusCode::NO_CONTENT)
}

// ============================================================================
// Router
// ============================================================================

pub fn create_canvas_router() -> axum::Router<CanvasState> {
    use axum::routing::{delete, get, post, put};

    axum::Router::new()
        // Canvas CRUD
        .route("/canvases", post(create_canvas))
        .route("/canvases", get(list_canvases))
        .route("/canvases/{canvas_id}", get(get_canvas))
        .route("/canvases/{canvas_id}", put(update_canvas))
        .route("/canvases/{canvas_id}", delete(delete_canvas))
        // Node CRUD (nested under canvas)
        .route("/canvases/{canvas_id}/nodes", post(create_node))
        .route("/canvases/{canvas_id}/nodes", get(list_nodes))
        // Node CRUD (standalone)
        .route("/canvases/nodes/{node_id}", put(update_node))
        .route("/canvases/nodes/{node_id}", delete(delete_node))
        // Edge CRUD (nested under canvas)
        .route("/canvases/{canvas_id}/edges", post(create_edge))
        .route("/canvases/{canvas_id}/edges", get(list_edges))
        // Edge CRUD (standalone)
        .route("/canvases/edges/{edge_id}", put(update_edge))
        .route("/canvases/edges/{edge_id}", delete(delete_edge))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_canvas_response_serialization() {
        let resp = CanvasResponse {
            canvas: Canvas {
                id: uuid::Uuid::new_v4(),
                title: "Test".to_string(),
                owner_id: uuid::Uuid::new_v4(),
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            },
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("Test"));
    }
}
