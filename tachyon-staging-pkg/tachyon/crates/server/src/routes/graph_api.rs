use axum::{extract::State, response::Json};
use serde::Serialize;
use tachyon_database::{DatabasePool, GraphEdge, GraphNode, GraphRepository};
use tracing::debug;

use crate::error::ServerError;

#[derive(Clone)]
pub struct GraphApiState {
    pub pool: DatabasePool,
}

impl GraphApiState {
    pub fn new(pool: DatabasePool) -> Self {
        Self { pool }
    }

    fn repo(&self) -> GraphRepository {
        GraphRepository::new(self.pool.clone())
    }
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct GraphNodesResponse {
    pub nodes: Vec<GraphNode>,
    pub total: usize,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct GraphEdgesResponse {
    pub edges: Vec<GraphEdge>,
    pub total: usize,
}

/// Get all active graph nodes.
///
/// `GET /api/v1/graph/nodes`
///
/// Returns all active knowledge graph nodes with their properties.
#[utoipa::path(
    get,
    path = "/api/v1/graph/nodes",
    responses(
        (status = 200, description = "All graph nodes", body = GraphNodesResponse),
    ),
    tag = "graph",
    security(("bearer_auth" = [])),
)]
pub async fn get_graph_nodes(
    State(state): State<GraphApiState>,
) -> Result<Json<GraphNodesResponse>, ServerError> {
    debug!("Getting all graph nodes");

    let (nodes, _total) = state.repo().list_nodes(None, None, None, 1, 10000).await?;

    let total = nodes.len();

    Ok(Json(GraphNodesResponse { nodes, total }))
}

/// Get all active graph edges (wiki-link relationships).
///
/// `GET /api/v1/graph/edges`
///
/// Returns all active knowledge graph edges representing relationships
/// between nodes (e.g., wiki-links, references, dependencies).
#[utoipa::path(
    get,
    path = "/api/v1/graph/edges",
    responses(
        (status = 200, description = "All graph edges", body = GraphEdgesResponse),
    ),
    tag = "graph",
    security(("bearer_auth" = [])),
)]
pub async fn get_graph_edges(
    State(state): State<GraphApiState>,
) -> Result<Json<GraphEdgesResponse>, ServerError> {
    debug!("Getting all graph edges");

    let edges = state.repo().list_edges(None, None, None, None).await?;
    let total = edges.len();

    Ok(Json(GraphEdgesResponse { edges, total }))
}

/// Get orphan nodes (documents with no links).
///
/// `GET /api/v1/graph/orphans`
///
/// Returns all active knowledge graph nodes that have zero incoming
/// and zero outgoing edges — i.e., documents not connected to any
/// other node in the knowledge graph.
#[utoipa::path(
    get,
    path = "/api/v1/graph/orphans",
    responses(
        (status = 200, description = "Orphan nodes", body = GraphNodesResponse),
    ),
    tag = "graph",
    security(("bearer_auth" = [])),
)]
pub async fn get_graph_orphans(
    State(state): State<GraphApiState>,
) -> Result<Json<GraphNodesResponse>, ServerError> {
    debug!("Getting orphan graph nodes");

    let orphans = state.repo().get_orphan_nodes().await?;
    let total = orphans.len();

    Ok(Json(GraphNodesResponse {
        nodes: orphans,
        total,
    }))
}

pub fn create_graph_api_router() -> axum::Router<GraphApiState> {
    use axum::routing::get;

    axum::Router::new()
        .route("/graph/nodes", get(get_graph_nodes))
        .route("/graph/edges", get(get_graph_edges))
        .route("/graph/orphans", get(get_graph_orphans))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_graph_nodes_response_construction() {
        let resp = GraphNodesResponse {
            nodes: vec![],
            total: 0,
        };
        assert_eq!(resp.total, 0);
    }

    #[test]
    fn test_graph_edges_response_construction() {
        let resp = GraphEdgesResponse {
            edges: vec![],
            total: 0,
        };
        assert_eq!(resp.total, 0);
    }
}
