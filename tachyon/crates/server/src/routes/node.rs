// Knowledge graph Node API routes
// Handles node CRUD operations and graph queries

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};
use uuid::Uuid;

/// Node data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeData {
    /// Node ID
    pub id: String,
    /// Node title
    pub title: String,
    /// Node content
    pub content: String,
    /// Node type
    pub node_type: String,
    /// Node metadata
    pub metadata: Option<HashMap<String, serde_json::Value>>,
    /// Created at
    pub created_at: DateTime<Utc>,
    /// Updated at
    pub updated_at: DateTime<Utc>,
    /// Creator ID
    pub creator_id: Option<String>,
}

/// Edge data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgeData {
    /// Edge ID
    pub id: String,
    /// Source node ID
    pub source_id: String,
    /// Target node ID
    pub target_id: String,
    /// Edge type
    pub edge_type: String,
    /// Edge metadata
    pub metadata: Option<HashMap<String, serde_json::Value>>,
    /// Created at
    pub created_at: DateTime<Utc>,
}

/// Application state for node routes
#[derive(Clone)]
pub struct NodeState {
    /// In-memory node store
    pub nodes: Arc<RwLock<HashMap<String, NodeData>>>,
    /// In-memory edge store
    pub edges: Arc<RwLock<HashMap<String, EdgeData>>>,
}

impl NodeState {
    /// Create a new node state
    pub fn new() -> Self {
        Self {
            nodes: Arc::new(RwLock::new(HashMap::new())),
            edges: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Create a node state with existing data
    pub fn with_data(nodes: HashMap<String, NodeData>, edges: HashMap<String, EdgeData>) -> Self {
        Self {
            nodes: Arc::new(RwLock::new(nodes)),
            edges: Arc::new(RwLock::new(edges)),
        }
    }
}

impl Default for NodeState {
    fn default() -> Self {
        Self::new()
    }
}

/// Query parameters for node listing
#[derive(Debug, Deserialize)]
pub struct NodeQuery {
    /// Page number
    pub page: Option<usize>,
    /// Page size
    pub page_size: Option<usize>,
    /// Node type filter
    pub node_type: Option<String>,
    /// Search term
    pub search: Option<String>,
}

/// Request to create a node
#[derive(Debug, Deserialize)]
pub struct CreateNodeRequest {
    /// Node title
    pub title: String,
    /// Node content
    pub content: String,
    /// Node type
    pub node_type: String,
    /// Node metadata
    pub metadata: Option<HashMap<String, serde_json::Value>>,
    /// Creator ID (optional)
    pub creator_id: Option<String>,
}

/// Request to update a node
#[derive(Debug, Deserialize)]
pub struct UpdateNodeRequest {
    /// Node title
    pub title: Option<String>,
    /// Node content
    pub content: Option<String>,
    /// Node type
    pub node_type: Option<String>,
    /// Node metadata
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

/// Request to create an edge
#[derive(Debug, Deserialize)]
pub struct CreateEdgeRequest {
    /// Source node ID
    pub source_id: String,
    /// Target node ID
    pub target_id: String,
    /// Edge type
    pub edge_type: String,
    /// Edge metadata
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

/// Node response
#[derive(Debug, Serialize)]
pub struct NodeResponse {
    /// Node ID
    pub id: String,
    /// Node title
    pub title: String,
    /// Node content
    pub content: String,
    /// Node type
    pub node_type: String,
    /// Node metadata
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, serde_json::Value>>,
    /// Created at
    pub created_at: String,
    /// Updated at
    pub updated_at: String,
    /// Creator ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub creator_id: Option<String>,
}

impl From<NodeData> for NodeResponse {
    fn from(node: NodeData) -> Self {
        Self {
            id: node.id,
            title: node.title,
            content: node.content,
            node_type: node.node_type,
            metadata: node.metadata,
            created_at: node.created_at.to_rfc3339(),
            updated_at: node.updated_at.to_rfc3339(),
            creator_id: node.creator_id,
        }
    }
}

/// Edge response
#[derive(Debug, Serialize)]
pub struct EdgeResponse {
    /// Edge ID
    pub id: String,
    /// Source node ID
    pub source_id: String,
    /// Target node ID
    pub target_id: String,
    /// Edge type
    pub edge_type: String,
    /// Edge metadata
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, serde_json::Value>>,
    /// Created at
    pub created_at: String,
}

impl From<EdgeData> for EdgeResponse {
    fn from(edge: EdgeData) -> Self {
        Self {
            id: edge.id,
            source_id: edge.source_id,
            target_id: edge.target_id,
            edge_type: edge.edge_type,
            metadata: edge.metadata,
            created_at: edge.created_at.to_rfc3339(),
        }
    }
}

/// Node list response
#[derive(Debug, Serialize)]
pub struct NodeListResponse {
    /// List of nodes
    pub nodes: Vec<NodeResponse>,
    /// Total count
    pub total: usize,
    /// Page number
    pub page: usize,
    /// Page size
    pub page_size: usize,
}

/// Edge list response
#[derive(Debug, Serialize)]
pub struct EdgeListResponse {
    /// List of edges
    pub edges: Vec<EdgeResponse>,
    /// Total count
    pub total: usize,
}

/// Graph query request
#[derive(Debug, Deserialize)]
pub struct GraphQueryRequest {
    /// Start node ID
    pub start_node: Option<String>,
    /// End node ID
    pub end_node: Option<String>,
    /// Edge type filter
    pub edge_type: Option<String>,
    /// Node type filter
    pub node_type: Option<String>,
    /// Max depth
    pub max_depth: Option<usize>,
}

/// Graph query response
#[derive(Debug, Serialize)]
pub struct GraphQueryResponse {
    /// Nodes in the graph
    pub nodes: Vec<NodeResponse>,
    /// Edges in the graph
    pub edges: Vec<EdgeResponse>,
    /// Total node count
    pub node_count: usize,
    /// Total edge count
    pub edge_count: usize,
}

/// Error response
#[derive(Debug, Serialize)]
pub struct NodeErrorResponse {
    /// Error code
    pub code: String,
    /// Error message
    pub message: String,
}

/// Create a new node
pub async fn create_node(
    State(state): State<NodeState>,
    Json(req): Json<CreateNodeRequest>,
) -> Result<Json<NodeResponse>, (StatusCode, Json<NodeErrorResponse>)> {
    info!("Creating new node: {}", req.title);

    let now = Utc::now();

    let node = NodeData {
        id: format!("node_{}", Uuid::new_v4()),
        title: req.title,
        content: req.content,
        node_type: req.node_type,
        metadata: req.metadata,
        created_at: now,
        updated_at: now,
        creator_id: req.creator_id,
    };

    let response = NodeResponse::from(node.clone());

    // Store node
    let mut nodes = state.nodes.write().await;
    nodes.insert(node.id.clone(), node);

    info!("Node created: {}", response.id);

    Ok(Json(response))
}

/// Get a node by ID
pub async fn get_node(
    Path(node_id): Path<String>,
    State(state): State<NodeState>,
) -> Result<Json<NodeResponse>, (StatusCode, Json<NodeErrorResponse>)> {
    debug!("Getting node: {}", node_id);

    let nodes = state.nodes.read().await;
    
    match nodes.get(&node_id) {
        Some(node) => Ok(Json(NodeResponse::from(node.clone()))),
        None => {
            debug!("Node not found: {}", node_id);
            Err((
                StatusCode::NOT_FOUND,
                Json(NodeErrorResponse {
                    code: "NOT_FOUND".to_string(),
                    message: format!("Node {} not found", node_id),
                }),
            ))
        }
    }
}

/// Update a node
pub async fn update_node(
    Path(node_id): Path<String>,
    State(state): State<NodeState>,
    Json(req): Json<UpdateNodeRequest>,
) -> Result<Json<NodeResponse>, (StatusCode, Json<NodeErrorResponse>)> {
    info!("Updating node: {}", node_id);

    let mut nodes = state.nodes.write().await;
    
    match nodes.get_mut(&node_id) {
        Some(node) => {
            if let Some(title) = req.title {
                node.title = title;
            }
            if let Some(content) = req.content {
                node.content = content;
            }
            if let Some(node_type) = req.node_type {
                node.node_type = node_type;
            }
            if let Some(metadata) = req.metadata {
                node.metadata = Some(metadata);
            }
            node.updated_at = Utc::now();

            let response = NodeResponse::from(node.clone());
            info!("Node updated: {}", node_id);
            Ok(Json(response))
        }
        None => {
            debug!("Node not found for update: {}", node_id);
            Err((
                StatusCode::NOT_FOUND,
                Json(NodeErrorResponse {
                    code: "NOT_FOUND".to_string(),
                    message: format!("Node {} not found", node_id),
                }),
            ))
        }
    }
}

/// Delete a node
pub async fn delete_node(
    Path(node_id): Path<String>,
    State(state): State<NodeState>,
) -> Result<StatusCode, (StatusCode, Json<NodeErrorResponse>)> {
    info!("Deleting node: {}", node_id);

    // First remove the node
    let mut nodes = state.nodes.write().await;
    
    match nodes.remove(&node_id) {
        Some(_) => {
            // Also remove all edges connected to this node
            let mut edges = state.edges.write().await;
            edges.retain(|_, edge| edge.source_id != node_id && edge.target_id != node_id);
            
            info!("Node deleted: {}", node_id);
            Ok(StatusCode::NO_CONTENT)
        }
        None => {
            debug!("Node not found for deletion: {}", node_id);
            Err((
                StatusCode::NOT_FOUND,
                Json(NodeErrorResponse {
                    code: "NOT_FOUND".to_string(),
                    message: format!("Node {} not found", node_id),
                }),
            ))
        }
    }
}

/// List nodes
pub async fn list_nodes(
    Query(query): Query<NodeQuery>,
    State(state): State<NodeState>,
) -> Result<Json<NodeListResponse>, (StatusCode, Json<NodeErrorResponse>)> {
    debug!("Listing nodes");

    let page = query.page.unwrap_or(1).max(1);
    let page_size = query.page_size.unwrap_or(20).min(100);

    let nodes = state.nodes.read().await;
    
    // Filter nodes
    let filtered: Vec<NodeResponse> = nodes
        .values()
        .filter(|n| {
            // Filter by node type
            if let Some(ref node_type) = query.node_type {
                if &n.node_type != node_type {
                    return false;
                }
            }
            // Filter by search term
            if let Some(ref search) = query.search {
                let search_lower = search.to_lowercase();
                if !n.title.to_lowercase().contains(&search_lower) 
                    && !n.content.to_lowercase().contains(&search_lower) {
                    return false;
                }
            }
            true
        })
        .map(|n| NodeResponse::from(n.clone()))
        .collect();

    let total = filtered.len();
    
    // Paginate
    let start = (page - 1) * page_size;
    let end = start + page_size;
    let paginated: Vec<NodeResponse> = filtered
        .into_iter()
        .skip(start)
        .take(page_size)
        .collect();

    Ok(Json(NodeListResponse {
        nodes: paginated,
        total,
        page,
        page_size,
    }))
}

/// Create an edge between nodes
pub async fn create_edge(
    State(state): State<NodeState>,
    Json(req): Json<CreateEdgeRequest>,
) -> Result<Json<EdgeResponse>, (StatusCode, Json<NodeErrorResponse>)> {
    info!("Creating edge from {} to {}", req.source_id, req.target_id);

    // Verify both nodes exist
    let nodes = state.nodes.read().await;
    if !nodes.contains_key(&req.source_id) {
        return Err((
            StatusCode::NOT_FOUND,
            Json(NodeErrorResponse {
                code: "SOURCE_NOT_FOUND".to_string(),
                message: format!("Source node {} not found", req.source_id),
            }),
        ));
    }
    if !nodes.contains_key(&req.target_id) {
        return Err((
            StatusCode::NOT_FOUND,
            Json(NodeErrorResponse {
                code: "TARGET_NOT_FOUND".to_string(),
                message: format!("Target node {} not found", req.target_id),
            }),
        ));
    }
    drop(nodes);

    let now = Utc::now();
    let edge = EdgeData {
        id: format!("edge_{}", Uuid::new_v4()),
        source_id: req.source_id,
        target_id: req.target_id,
        edge_type: req.edge_type,
        metadata: req.metadata,
        created_at: now,
    };

    let response = EdgeResponse::from(edge.clone());

    // Store edge
    let mut edges = state.edges.write().await;
    edges.insert(edge.id.clone(), edge);

    info!("Edge created: {}", response.id);

    Ok(Json(response))
}

/// Get edges for a node
pub async fn get_node_edges(
    Path(node_id): Path<String>,
    State(state): State<NodeState>,
) -> Result<Json<EdgeListResponse>, (StatusCode, Json<NodeErrorResponse>)> {
    debug!("Getting edges for node: {}", node_id);

    let edges = state.edges.read().await;
    
    let node_edges: Vec<EdgeResponse> = edges
        .values()
        .filter(|e| e.source_id == node_id || e.target_id == node_id)
        .map(|e| EdgeResponse::from(e.clone()))
        .collect();

    let total = node_edges.len();

    Ok(Json(EdgeListResponse {
        edges: node_edges,
        total,
    }))
}

/// Delete an edge
pub async fn delete_edge(
    Path(edge_id): Path<String>,
    State(state): State<NodeState>,
) -> Result<StatusCode, (StatusCode, Json<NodeErrorResponse>)> {
    info!("Deleting edge: {}", edge_id);

    let mut edges = state.edges.write().await;
    
    match edges.remove(&edge_id) {
        Some(_) => {
            info!("Edge deleted: {}", edge_id);
            Ok(StatusCode::NO_CONTENT)
        }
        None => {
            debug!("Edge not found for deletion: {}", edge_id);
            Err((
                StatusCode::NOT_FOUND,
                Json(NodeErrorResponse {
                    code: "NOT_FOUND".to_string(),
                    message: format!("Edge {} not found", edge_id),
                }),
            ))
        }
    }
}

/// Query knowledge graph
pub async fn query_graph(
    State(state): State<NodeState>,
    Json(req): Json<GraphQueryRequest>,
) -> Result<Json<GraphQueryResponse>, (StatusCode, Json<NodeErrorResponse>)> {
    info!("Querying knowledge graph");

    let nodes = state.nodes.read().await;
    let edges = state.edges.read().await;

    let max_depth = req.max_depth.unwrap_or(3);

    // Start with empty result
    let mut result_nodes: HashMap<String, NodeData> = HashMap::new();
    let mut result_edges: HashMap<String, EdgeData> = HashMap::new();

    // If start_node provided, do BFS traversal
    if let Some(start_node_id) = &req.start_node {
        if let Some(start_node) = nodes.get(start_node_id) {
            result_nodes.insert(start_node.id.clone(), start_node.clone());
            
            // BFS
            let mut visited: HashMap<String, bool> = HashMap::new();
            visited.insert(start_node_id.clone(), true);
            let mut queue: Vec<String> = vec![start_node_id.clone()];
            
            while let Some(current) = queue.pop() {
                if let Some(current_depth) = visited.get(&current) {
                    if *current_depth as usize >= max_depth {
                        continue;
                    }
                }
                
                // Find all edges from current node
                for edge in edges.values() {
                    if edge.source_id == current {
                        if !visited.contains_key(&edge.target_id) {
                            visited.insert(edge.target_id.clone(), true);
                            queue.push(edge.target_id.clone());
                            
                            if let Some(target_node) = nodes.get(&edge.target_id) {
                                // Apply node type filter
                                if let Some(ref node_type) = req.node_type {
                                    if &target_node.node_type != node_type {
                                        continue;
                                    }
                                }
                                result_nodes.insert(target_node.id.clone(), target_node.clone());
                            }
                        }
                        // Apply edge type filter
                        if let Some(ref edge_type) = req.edge_type {
                            if &edge.edge_type != edge_type {
                                continue;
                            }
                        }
                        result_edges.insert(edge.id.clone(), edge.clone());
                    }
                }
            }
        }
    } else {
        // No start node - return all nodes (filtered)
        for node in nodes.values() {
            // Apply node type filter
            if let Some(ref node_type) = req.node_type {
                if &node.node_type != node_type {
                    continue;
                }
            }
            result_nodes.insert(node.id.clone(), node.clone());
        }
        
        // Apply edge type filter
        for edge in edges.values() {
            if let Some(ref edge_type) = req.edge_type {
                if &edge.edge_type != edge_type {
                    continue;
                }
            }
            result_edges.insert(edge.id.clone(), edge.clone());
        }
    }

    let node_responses: Vec<NodeResponse> = result_nodes
        .values()
        .map(|n| NodeResponse::from(n.clone()))
        .collect();

    let edge_responses: Vec<EdgeResponse> = result_edges
        .values()
        .map(|e| EdgeResponse::from(e.clone()))
        .collect();

    Ok(Json(GraphQueryResponse {
        nodes: node_responses,
        edges: edge_responses,
        node_count: result_nodes.len(),
        edge_count: result_edges.len(),
    }))
}

/// Create the node router (without state - caller must use .with_state())
pub fn create_node_router() -> axum::Router<NodeState> {
    use axum::routing::{delete, get, post};

    axum::Router::new()
        .route("/nodes", post(create_node))
        .route("/nodes", get(list_nodes))
        .route("/nodes/{node_id}", get(get_node))
        .route("/nodes/{node_id}", post(update_node))
        .route("/nodes/{node_id}", delete(delete_node))
        .route("/edges", post(create_edge))
        .route("/nodes/{node_id}/edges", get(get_node_edges))
        .route("/edges/{edge_id}", delete(delete_edge))
        .route("/graph/query", post(query_graph))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_node_request_construction() {
        let req = CreateNodeRequest {
            title: "Test Node".to_string(),
            content: "Test content".to_string(),
            node_type: "concept".to_string(),
            metadata: None,
            creator_id: None,
        };

        assert_eq!(req.title, "Test Node");
        assert_eq!(req.node_type, "concept");
    }

    #[test]
    fn test_create_edge_request_construction() {
        let req = CreateEdgeRequest {
            source_id: "node-1".to_string(),
            target_id: "node-2".to_string(),
            edge_type: "references".to_string(),
            metadata: None,
        };

        assert_eq!(req.source_id, "node-1");
        assert_eq!(req.target_id, "node-2");
    }

    #[test]
    fn test_node_data_creation() {
        let now = Utc::now();
        let node = NodeData {
            id: "node-123".to_string(),
            title: "Test Node".to_string(),
            content: "Content".to_string(),
            node_type: "concept".to_string(),
            metadata: None,
            created_at: now,
            updated_at: now,
            creator_id: Some("user-1".to_string()),
        };

        assert_eq!(node.id, "node-123");
        assert!(node.creator_id.is_some());
    }

    #[test]
    fn test_node_response_from_data() {
        let now = Utc::now();
        let node = NodeData {
            id: "node-123".to_string(),
            title: "Test Node".to_string(),
            content: "Content".to_string(),
            node_type: "concept".to_string(),
            metadata: Some(HashMap::new()),
            created_at: now,
            updated_at: now,
            creator_id: Some("user-1".to_string()),
        };

        let response = NodeResponse::from(node);
        assert_eq!(response.id, "node-123");
        assert_eq!(response.title, "Test Node");
    }
}
