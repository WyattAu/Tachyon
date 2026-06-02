use crate::error::{DatabaseError, DatabaseResult};
use crate::schema::DatabasePool;
use crate::types::{GraphEdge, GraphNode};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::{Row, query, query_as};
use std::collections::{HashMap, HashSet, VecDeque};
use tachyon_core::types::edge::Edge;
use tachyon_core::types::node::Node;
use tracing::{info, instrument, warn};

// ============================================================================
// Type Conversions
// ============================================================================

impl From<&Node> for GraphNode {
    fn from(node: &Node) -> Self {
        let mut props = serde_json::Map::new();
        if !node.metadata.tags.is_empty() {
            props.insert("tags".to_string(), json!(node.metadata.tags));
        }
        for (k, v) in &node.metadata.custom_metadata {
            props.insert(k.clone(), v.clone());
        }
        if let Some(ref parent_id) = node.metadata.parent_id {
            props.insert("parent_id".to_string(), json!(parent_id.as_str()));
        }

        Self {
            id: node.id.as_str().to_string(),
            node_type: serde_json::to_value(node.node_type)
                .ok()
                .and_then(|v| v.as_str().map(|s| s.to_string()))
                .unwrap_or_else(|| format!("{:?}", node.node_type).to_lowercase()),
            name: node.metadata.title.clone(),
            slug: node.metadata.slug.clone(),
            description: node.metadata.description.clone(),
            content: None,
            visibility: serde_json::to_value(node.visibility)
                .ok()
                .and_then(|v| v.as_str().map(|s| s.to_string()))
                .unwrap_or_else(|| "private".to_string()),
            weight: 1.0,
            properties: serde_json::Value::Object(props),
            project_id: None,
            document_id: node.metadata.document_id.map(|id| id.as_str().to_string()),
            created_by: Some(node.metadata.created_by.as_str().to_string()),
            is_active: true,
            created_at: node.metadata.created_at,
            updated_at: node.metadata.updated_at,
            deactivated_at: None,
        }
    }
}

impl From<&Edge> for GraphEdge {
    fn from(edge: &Edge) -> Self {
        let (weight, confidence) = edge
            .weight
            .map(|w| (w.weight, Some(w.confidence)))
            .unwrap_or((1.0, None));

        Self {
            id: edge.id.as_str().to_string(),
            source_id: edge.source_id.as_str().to_string(),
            target_id: edge.target_id.as_str().to_string(),
            edge_type: serde_json::to_value(edge.edge_type)
                .ok()
                .and_then(|v| v.as_str().map(|s| s.to_string()))
                .unwrap_or_else(|| format!("{:?}", edge.edge_type).to_lowercase()),
            label: edge.metadata.label.clone(),
            description: edge.metadata.description.clone(),
            weight,
            confidence,
            properties: json!({}),
            project_id: None,
            created_by: Some(edge.metadata.created_by.as_str().to_string()),
            is_active: true,
            created_at: edge.metadata.created_at,
            updated_at: edge.metadata.updated_at,
            deactivated_at: edge.metadata.deactivated_at,
        }
    }
}

// ============================================================================
// Graph Repository
// ============================================================================

#[derive(sqlx::FromRow)]
struct BfsPathRow {
    path: Vec<uuid::Uuid>,
}

/// Repository for managing knowledge graph nodes and edges.
///
/// Provides CRUD operations for graph entities as well as traversal
/// queries including shortest-path, neighbor search, and connected-component
/// analysis.
pub struct GraphRepository {
    pool: DatabasePool,
}

impl GraphRepository {
    /// Create a new graph repository backed by the given connection pool.
    ///
    /// # Arguments
    /// * `pool` - Database connection pool
    pub fn new(pool: DatabasePool) -> Self {
        Self { pool }
    }

    // ========================================================================
    // Node CRUD
    // ========================================================================

    /// Insert a new node into the knowledge graph.
    ///
    /// # Arguments
    /// * `node` - The node to create
    ///
    /// # Returns
    /// The persisted node with its database-assigned ID and timestamps.
    ///
    /// # Errors
    /// Returns a `DatabaseError::Duplicate` if the slug already exists,
    /// or a `DatabaseError::QueryError` on other SQL failures.
    #[instrument(skip(self, node), fields(node_name = %node.name))]
    pub async fn create_node(&self, node: &GraphNode) -> DatabaseResult<GraphNode> {
        let sql = r#"
            INSERT INTO knowledge_graph_nodes
                (node_type, name, slug, description, content, visibility, weight, properties, project_id, document_id, created_by, is_active, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
            RETURNING id, node_type, name, slug, description, content, visibility, weight, properties, project_id, document_id, created_by, is_active, created_at, updated_at
        "#;

        let mut conn = self.pool.acquire().await?;
        let record = query_as::<_, GraphNode>(sql)
            .bind(&node.node_type)
            .bind(&node.name)
            .bind(&node.slug)
            .bind(&node.description)
            .bind(&node.content)
            .bind(&node.visibility)
            .bind(node.weight)
            .bind(&node.properties)
            .bind(
                node.project_id
                    .as_deref()
                    .map(uuid::Uuid::parse_str)
                    .transpose()
                    .ok()
                    .flatten(),
            )
            .bind(
                node.document_id
                    .as_deref()
                    .map(uuid::Uuid::parse_str)
                    .transpose()
                    .ok()
                    .flatten(),
            )
            .bind(
                node.created_by
                    .as_deref()
                    .map(uuid::Uuid::parse_str)
                    .transpose()
                    .ok()
                    .flatten(),
            )
            .bind(node.is_active)
            .bind(node.created_at)
            .bind(node.updated_at)
            .fetch_one(&mut *conn)
            .await
            .map_err(|e| {
                let msg = e.to_string();
                if msg.contains("unique")
                    || msg.contains("duplicate")
                    || msg.contains("idx_kg_nodes_slug")
                {
                    DatabaseError::duplicate(
                        "node",
                        format!("Slug already exists: {:?}", node.slug),
                    )
                } else {
                    DatabaseError::QueryError(msg)
                }
            })?;

        info!("Node created: {} ({})", record.name, record.id);
        Ok(record)
    }

    /// Retrieve a single active node by its UUID.
    ///
    /// # Arguments
    /// * `id` - UUID string of the node
    ///
    /// # Returns
    /// The matching `GraphNode`.
    ///
    /// # Errors
    /// Returns `DatabaseError::NotFound` if no active node exists with the
    /// given ID, or `DatabaseError::ValidationError` if `id` is not a valid UUID.
    #[instrument(skip(self))]
    pub async fn get_node_by_id(&self, id: &str) -> DatabaseResult<GraphNode> {
        let sql = "SELECT * FROM knowledge_graph_nodes WHERE id = $1 AND is_active = true";
        let mut conn = self.pool.acquire().await?;
        let record = query_as::<_, GraphNode>(sql)
            .bind(
                uuid::Uuid::parse_str(id)
                    .map_err(|e| DatabaseError::ValidationError(format!("Invalid UUID: {}", e)))?,
            )
            .fetch_optional(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?
            .ok_or_else(|| DatabaseError::not_found("node", id))?;
        Ok(record)
    }

    #[instrument(skip(self), fields(slug = %slug))]
    pub async fn get_node_by_slug(&self, slug: &str) -> DatabaseResult<GraphNode> {
        let sql = "SELECT * FROM knowledge_graph_nodes WHERE slug = $1 AND is_active = true";
        let mut conn = self.pool.acquire().await?;
        let record = query_as::<_, GraphNode>(sql)
            .bind(slug)
            .fetch_optional(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?
            .ok_or_else(|| DatabaseError::not_found("node", slug))?;
        Ok(record)
    }

    pub async fn get_nodes_by_ids_batch(&self, ids: &[String]) -> DatabaseResult<Vec<GraphNode>> {
        if ids.is_empty() {
            return Ok(vec![]);
        }

        let uuids: Result<Vec<uuid::Uuid>, _> = ids
            .iter()
            .map(|id| {
                uuid::Uuid::parse_str(id)
                    .map_err(|e| DatabaseError::ValidationError(format!("Invalid UUID: {}", e)))
            })
            .collect();
        let uuids = uuids?;

        let sql =
            "SELECT * FROM knowledge_graph_nodes WHERE id = ANY($1::uuid[]) AND is_active = true";
        let mut conn = self.pool.acquire().await?;
        let records = query_as::<_, GraphNode>(sql)
            .bind(&uuids)
            .fetch_all(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        Ok(records)
    }

    pub async fn get_nodes_by_slugs_batch(
        &self,
        slugs: &[String],
    ) -> DatabaseResult<Vec<GraphNode>> {
        if slugs.is_empty() {
            return Ok(vec![]);
        }

        let sql =
            "SELECT * FROM knowledge_graph_nodes WHERE slug = ANY($1::text[]) AND is_active = true";
        let mut conn = self.pool.acquire().await?;
        let records = query_as::<_, GraphNode>(sql)
            .bind(slugs)
            .fetch_all(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        Ok(records)
    }

    pub async fn deactivate_edges_for_node(&self, node_id: &str) -> DatabaseResult<u64> {
        let sql = r#"
            UPDATE knowledge_graph_edges
            SET is_active = false, deactivated_at = NOW(), updated_at = NOW()
            WHERE (source_id = $1 OR target_id = $1) AND is_active = true
        "#;
        let uuid = uuid::Uuid::parse_str(node_id)
            .map_err(|e| DatabaseError::ValidationError(format!("Invalid UUID: {}", e)))?;
        let mut conn = self.pool.acquire().await?;
        let result = query(sql)
            .bind(uuid)
            .execute(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;
        Ok(result.rows_affected())
    }

    #[instrument(skip(self))]
    #[allow(clippy::too_many_arguments)]
    pub async fn update_node(
        &self,
        id: &str,
        name: Option<&str>,
        slug: Option<&str>,
        description: Option<&str>,
        content: Option<&str>,
        visibility: Option<&str>,
        weight: Option<f64>,
        properties: Option<&serde_json::Value>,
    ) -> DatabaseResult<GraphNode> {
        let sql = r#"
            UPDATE knowledge_graph_nodes SET
                name = COALESCE($2, name),
                slug = COALESCE($3, slug),
                description = COALESCE($4, description),
                content = COALESCE($5, content),
                visibility = COALESCE($6, visibility),
                weight = COALESCE($7, weight),
                properties = COALESCE($8, properties),
                updated_at = NOW()
            WHERE id = $1 AND is_active = true
            RETURNING id, node_type, name, slug, description, content, visibility, weight, properties, project_id, document_id, created_by, is_active, created_at, updated_at
        "#;

        let mut conn = self.pool.acquire().await?;
        let record = query_as::<_, GraphNode>(sql)
            .bind(
                uuid::Uuid::parse_str(id)
                    .map_err(|e| DatabaseError::ValidationError(format!("Invalid UUID: {}", e)))?,
            )
            .bind(name)
            .bind(slug)
            .bind(description)
            .bind(content)
            .bind(visibility)
            .bind(weight)
            .bind(properties)
            .fetch_optional(&mut *conn)
            .await
            .map_err(|e| {
                let msg = e.to_string();
                if msg.contains("unique")
                    || msg.contains("duplicate")
                    || msg.contains("idx_kg_nodes_slug")
                {
                    DatabaseError::duplicate("node", format!("Slug already exists: {:?}", slug))
                } else {
                    DatabaseError::QueryError(msg)
                }
            })?
            .ok_or_else(|| DatabaseError::not_found("node", id))?;

        info!("Node updated: {}", id);
        Ok(record)
    }

    #[instrument(skip(self))]
    pub async fn deactivate_node(&self, id: &str) -> DatabaseResult<()> {
        let sql = "UPDATE knowledge_graph_nodes SET is_active = false, deactivated_at = NOW(), updated_at = NOW() WHERE id = $1";
        let mut conn = self.pool.acquire().await?;
        let result = query(sql)
            .bind(
                uuid::Uuid::parse_str(id)
                    .map_err(|e| DatabaseError::ValidationError(format!("Invalid UUID: {}", e)))?,
            )
            .execute(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(DatabaseError::not_found("node", id));
        }
        info!("Node deactivated: {}", id);
        Ok(())
    }

    #[instrument(skip(self))]
    pub async fn delete_node(&self, id: &str) -> DatabaseResult<()> {
        let sql = "DELETE FROM knowledge_graph_nodes WHERE id = $1";
        let mut conn = self.pool.acquire().await?;
        let result = query(sql)
            .bind(
                uuid::Uuid::parse_str(id)
                    .map_err(|e| DatabaseError::ValidationError(format!("Invalid UUID: {}", e)))?,
            )
            .execute(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(DatabaseError::not_found("node", id));
        }
        warn!("Node hard-deleted: {}", id);
        Ok(())
    }

    pub async fn list_nodes(
        &self,
        node_type: Option<&str>,
        project_id: Option<&str>,
        search: Option<&str>,
        page: usize,
        page_size: usize,
    ) -> DatabaseResult<(Vec<GraphNode>, i64)> {
        let offset = ((page.max(1) - 1) * page_size.min(100)) as i64;
        let limit = page_size.min(100) as i64;

        let mut where_clauses = vec!["is_active = true".to_string()];
        let mut bind_idx = 0u32;

        if node_type.is_some() {
            bind_idx += 1;
            where_clauses.push(format!("node_type = ${}", bind_idx));
        }
        if project_id.is_some() {
            bind_idx += 1;
            where_clauses.push(format!("project_id = ${}", bind_idx));
        }
        if search.is_some() {
            bind_idx += 1;
            where_clauses.push(format!(
                "(name ILIKE ${} OR description ILIKE ${})",
                bind_idx, bind_idx
            ));
        }

        let where_sql = where_clauses.join(" AND ");
        let count_sql = format!(
            "SELECT COUNT(*) as count FROM knowledge_graph_nodes WHERE {}",
            where_sql
        );
        let data_sql = format!(
            "SELECT * FROM knowledge_graph_nodes WHERE {} ORDER BY updated_at DESC LIMIT ${} OFFSET ${}",
            where_sql,
            bind_idx + 1,
            bind_idx + 2
        );

        let mut conn = self.pool.acquire().await?;
        let mut count_query = sqlx::query(&count_sql);
        let mut data_query = query_as::<_, GraphNode>(&data_sql);

        if let Some(nt) = node_type {
            count_query = count_query.bind(nt);
            data_query = data_query.bind(nt);
        }
        if let Some(pid) = project_id {
            let uuid = uuid::Uuid::parse_str(pid)
                .map_err(|e| DatabaseError::ValidationError(format!("Invalid UUID: {}", e)))?;
            count_query = count_query.bind(uuid);
            data_query = data_query.bind(uuid);
        }
        if let Some(s) = search {
            let pattern = format!("%{}%", s);
            count_query = count_query.bind(pattern.clone());
            data_query = data_query.bind(pattern);
        }

        let count_row = count_query
            .fetch_one(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;
        let total: i64 = count_row.get("count");

        let records = data_query
            .bind(limit)
            .bind(offset)
            .fetch_all(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        Ok((records, total))
    }

    pub async fn count_nodes(&self) -> DatabaseResult<i64> {
        let sql = "SELECT COUNT(*) as count FROM knowledge_graph_nodes WHERE is_active = true";
        let mut conn = self.pool.acquire().await?;
        let row = sqlx::query(sql)
            .fetch_one(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;
        Ok(row.get("count"))
    }

    // ========================================================================
    // Edge CRUD
    // ========================================================================

    /// Insert a new edge into the knowledge graph.
    ///
    /// # Arguments
    /// * `edge` - The edge to create (must reference existing node UUIDs)
    ///
    /// # Returns
    /// The persisted edge with its database-assigned ID and timestamps.
    ///
    /// # Errors
    /// Returns `DatabaseError::Duplicate` if an identical edge already exists,
    /// `DatabaseError::ConstraintViolation` if source or target nodes are missing,
    /// or `DatabaseError::QueryError` on other SQL failures.
    #[instrument(skip(self, edge), fields(edge_type = %edge.edge_type))]
    pub async fn create_edge(&self, edge: &GraphEdge) -> DatabaseResult<GraphEdge> {
        let sql = r#"
            INSERT INTO knowledge_graph_edges
                (source_id, target_id, edge_type, label, description, weight, confidence, properties, project_id, created_by, is_active, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
            RETURNING id, source_id, target_id, edge_type, label, description, weight, confidence, properties, project_id, created_by, is_active, created_at, updated_at
        "#;

        let mut conn = self.pool.acquire().await?;
        let record = query_as::<_, GraphEdge>(sql)
            .bind(uuid::Uuid::parse_str(&edge.source_id).map_err(|e| {
                DatabaseError::ValidationError(format!("Invalid source UUID: {}", e))
            })?)
            .bind(uuid::Uuid::parse_str(&edge.target_id).map_err(|e| {
                DatabaseError::ValidationError(format!("Invalid target UUID: {}", e))
            })?)
            .bind(&edge.edge_type)
            .bind(&edge.label)
            .bind(&edge.description)
            .bind(edge.weight)
            .bind(edge.confidence)
            .bind(&edge.properties)
            .bind(
                edge.project_id
                    .as_deref()
                    .map(uuid::Uuid::parse_str)
                    .transpose()
                    .ok()
                    .flatten(),
            )
            .bind(
                edge.created_by
                    .as_deref()
                    .map(uuid::Uuid::parse_str)
                    .transpose()
                    .ok()
                    .flatten(),
            )
            .bind(edge.is_active)
            .bind(edge.created_at)
            .bind(edge.updated_at)
            .fetch_one(&mut *conn)
            .await
            .map_err(|e| {
                let msg = e.to_string();
                if msg.contains("unique")
                    || msg.contains("duplicate")
                    || msg.contains("idx_kg_edges_unique")
                {
                    DatabaseError::duplicate(
                        "edge",
                        format!(
                            "Edge already exists: {} -> {} ({})",
                            edge.source_id, edge.target_id, edge.edge_type
                        ),
                    )
                } else if msg.contains("foreign key") || msg.contains("references") {
                    DatabaseError::constraint_violation(
                        "Source or target node does not exist".to_string(),
                    )
                } else {
                    DatabaseError::QueryError(msg)
                }
            })?;

        info!("Edge created: {} ({})", record.id, record.edge_type);
        Ok(record)
    }

    #[instrument(skip(self))]
    pub async fn get_edge_by_id(&self, id: &str) -> DatabaseResult<GraphEdge> {
        let sql = "SELECT * FROM knowledge_graph_edges WHERE id = $1 AND is_active = true";
        let mut conn = self.pool.acquire().await?;
        let record = query_as::<_, GraphEdge>(sql)
            .bind(
                uuid::Uuid::parse_str(id)
                    .map_err(|e| DatabaseError::ValidationError(format!("Invalid UUID: {}", e)))?,
            )
            .fetch_optional(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?
            .ok_or_else(|| DatabaseError::not_found("edge", id))?;
        Ok(record)
    }

    #[instrument(skip(self))]
    #[allow(clippy::too_many_arguments)]
    pub async fn update_edge(
        &self,
        id: &str,
        edge_type: Option<&str>,
        label: Option<&str>,
        description: Option<&str>,
        weight: Option<f64>,
        confidence: Option<f64>,
        properties: Option<&serde_json::Value>,
    ) -> DatabaseResult<GraphEdge> {
        let sql = r#"
            UPDATE knowledge_graph_edges SET
                edge_type = COALESCE($2, edge_type),
                label = COALESCE($3, label),
                description = COALESCE($4, description),
                weight = COALESCE($5, weight),
                confidence = COALESCE($6, confidence),
                properties = COALESCE($7, properties),
                updated_at = NOW()
            WHERE id = $1 AND is_active = true
            RETURNING id, source_id, target_id, edge_type, label, description, weight, confidence, properties, project_id, created_by, is_active, created_at, updated_at
        "#;

        let mut conn = self.pool.acquire().await?;
        let record = query_as::<_, GraphEdge>(sql)
            .bind(
                uuid::Uuid::parse_str(id)
                    .map_err(|e| DatabaseError::ValidationError(format!("Invalid UUID: {}", e)))?,
            )
            .bind(edge_type)
            .bind(label)
            .bind(description)
            .bind(weight)
            .bind(confidence)
            .bind(properties)
            .fetch_optional(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?
            .ok_or_else(|| DatabaseError::not_found("edge", id))?;

        info!("Edge updated: {}", id);
        Ok(record)
    }

    #[instrument(skip(self))]
    pub async fn deactivate_edge(&self, id: &str) -> DatabaseResult<()> {
        let sql = "UPDATE knowledge_graph_edges SET is_active = false, deactivated_at = NOW(), updated_at = NOW() WHERE id = $1";
        let mut conn = self.pool.acquire().await?;
        let result = query(sql)
            .bind(
                uuid::Uuid::parse_str(id)
                    .map_err(|e| DatabaseError::ValidationError(format!("Invalid UUID: {}", e)))?,
            )
            .execute(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(DatabaseError::not_found("edge", id));
        }
        info!("Edge deactivated: {}", id);
        Ok(())
    }

    #[instrument(skip(self))]
    pub async fn delete_edge(&self, id: &str) -> DatabaseResult<()> {
        let sql = "DELETE FROM knowledge_graph_edges WHERE id = $1";
        let mut conn = self.pool.acquire().await?;
        let result = query(sql)
            .bind(
                uuid::Uuid::parse_str(id)
                    .map_err(|e| DatabaseError::ValidationError(format!("Invalid UUID: {}", e)))?,
            )
            .execute(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(DatabaseError::not_found("edge", id));
        }
        warn!("Edge hard-deleted: {}", id);
        Ok(())
    }

    pub async fn list_edges(
        &self,
        source_id: Option<&str>,
        target_id: Option<&str>,
        edge_type: Option<&str>,
        project_id: Option<&str>,
    ) -> DatabaseResult<Vec<GraphEdge>> {
        let mut where_clauses = vec!["is_active = true".to_string()];
        let mut bind_idx = 0u32;

        if source_id.is_some() {
            bind_idx += 1;
            where_clauses.push(format!("source_id = ${}", bind_idx));
        }
        if target_id.is_some() {
            bind_idx += 1;
            where_clauses.push(format!("target_id = ${}", bind_idx));
        }
        if edge_type.is_some() {
            bind_idx += 1;
            where_clauses.push(format!("edge_type = ${}", bind_idx));
        }
        if project_id.is_some() {
            bind_idx += 1;
            where_clauses.push(format!("project_id = ${}", bind_idx));
        }

        let where_sql = where_clauses.join(" AND ");
        let sql = format!(
            "SELECT * FROM knowledge_graph_edges WHERE {} ORDER BY created_at DESC",
            where_sql
        );

        let mut conn = self.pool.acquire().await?;
        let mut query = query_as::<_, GraphEdge>(&sql);

        if let Some(sid) = source_id {
            let uuid = uuid::Uuid::parse_str(sid)
                .map_err(|e| DatabaseError::ValidationError(format!("Invalid UUID: {}", e)))?;
            query = query.bind(uuid);
        }
        if let Some(tid) = target_id {
            let uuid = uuid::Uuid::parse_str(tid)
                .map_err(|e| DatabaseError::ValidationError(format!("Invalid UUID: {}", e)))?;
            query = query.bind(uuid);
        }
        if let Some(et) = edge_type {
            query = query.bind(et);
        }
        if let Some(pid) = project_id {
            let uuid = uuid::Uuid::parse_str(pid)
                .map_err(|e| DatabaseError::ValidationError(format!("Invalid UUID: {}", e)))?;
            query = query.bind(uuid);
        }

        let records = query
            .fetch_all(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        Ok(records)
    }

    // ========================================================================
    // Graph Queries
    // ========================================================================

    pub async fn get_neighbors(
        &self,
        node_id: &str,
        direction: &str,
        edge_type: Option<&str>,
        depth: u32,
    ) -> DatabaseResult<Vec<GraphEdge>> {
        let depth = depth.min(5);
        let uuid = uuid::Uuid::parse_str(node_id)
            .map_err(|e| DatabaseError::ValidationError(format!("Invalid UUID: {}", e)))?;

        let sql = match direction {
            "outgoing" => {
                if edge_type.is_some() {
                    r#"
                        WITH RECURSIVE neighbors AS (
                            SELECT *, 1 AS depth FROM knowledge_graph_edges WHERE source_id = $1 AND edge_type = $2 AND is_active = true
                            UNION ALL
                            SELECT e.*, n.depth + 1
                            FROM knowledge_graph_edges e
                            JOIN neighbors n ON e.source_id = n.target_id
                            WHERE e.is_active = true AND n.depth < $3
                        )
                        SELECT DISTINCT id, source_id, target_id, edge_type, label, description, weight, confidence, properties, project_id, created_by, is_active, created_at, updated_at
                        FROM neighbors ORDER BY depth, created_at
                    "#
                } else {
                    r#"
                        WITH RECURSIVE neighbors AS (
                            SELECT *, 1 AS depth FROM knowledge_graph_edges WHERE source_id = $1 AND is_active = true
                            UNION ALL
                            SELECT e.*, n.depth + 1
                            FROM knowledge_graph_edges e
                            JOIN neighbors n ON e.source_id = n.target_id
                            WHERE e.is_active = true AND n.depth < $2
                        )
                        SELECT DISTINCT id, source_id, target_id, edge_type, label, description, weight, confidence, properties, project_id, created_by, is_active, created_at, updated_at
                        FROM neighbors ORDER BY depth, created_at
                    "#
                }
            }
            "incoming" => {
                if edge_type.is_some() {
                    r#"
                        WITH RECURSIVE neighbors AS (
                            SELECT *, 1 AS depth FROM knowledge_graph_edges WHERE target_id = $1 AND edge_type = $2 AND is_active = true
                            UNION ALL
                            SELECT e.*, n.depth + 1
                            FROM knowledge_graph_edges e
                            JOIN neighbors n ON e.target_id = n.source_id
                            WHERE e.is_active = true AND n.depth < $3
                        )
                        SELECT DISTINCT id, source_id, target_id, edge_type, label, description, weight, confidence, properties, project_id, created_by, is_active, created_at, updated_at
                        FROM neighbors ORDER BY depth, created_at
                    "#
                } else {
                    r#"
                        WITH RECURSIVE neighbors AS (
                            SELECT *, 1 AS depth FROM knowledge_graph_edges WHERE target_id = $1 AND is_active = true
                            UNION ALL
                            SELECT e.*, n.depth + 1
                            FROM knowledge_graph_edges e
                            JOIN neighbors n ON e.target_id = n.source_id
                            WHERE e.is_active = true AND n.depth < $2
                        )
                        SELECT DISTINCT id, source_id, target_id, edge_type, label, description, weight, confidence, properties, project_id, created_by, is_active, created_at, updated_at
                        FROM neighbors ORDER BY depth, created_at
                    "#
                }
            }
            _ => {
                if edge_type.is_some() {
                    r#"
                        WITH RECURSIVE neighbors AS (
                            SELECT *, 1 AS depth FROM knowledge_graph_edges WHERE (source_id = $1 OR target_id = $1) AND edge_type = $2 AND is_active = true
                            UNION ALL
                            SELECT e.*, n.depth + 1
                            FROM knowledge_graph_edges e
                            JOIN neighbors n ON (e.source_id = n.target_id OR e.target_id = n.source_id)
                            WHERE e.is_active = true AND n.depth < $3
                        )
                        SELECT DISTINCT id, source_id, target_id, edge_type, label, description, weight, confidence, properties, project_id, created_by, is_active, created_at, updated_at
                        FROM neighbors ORDER BY depth, created_at
                    "#
                } else {
                    r#"
                        WITH RECURSIVE neighbors AS (
                            SELECT *, 1 AS depth FROM knowledge_graph_edges WHERE (source_id = $1 OR target_id = $1) AND is_active = true
                            UNION ALL
                            SELECT e.*, n.depth + 1
                            FROM knowledge_graph_edges e
                            JOIN neighbors n ON (e.source_id = n.target_id OR e.target_id = n.source_id)
                            WHERE e.is_active = true AND n.depth < $2
                        )
                        SELECT DISTINCT id, source_id, target_id, edge_type, label, description, weight, confidence, properties, project_id, created_by, is_active, created_at, updated_at
                        FROM neighbors ORDER BY depth, created_at
                    "#
                }
            }
        };

        let mut conn = self.pool.acquire().await?;

        let records = if let Some(et) = &edge_type {
            query_as::<_, GraphEdge>(sql)
                .bind(uuid)
                .bind(et)
                .bind(depth as i32)
                .fetch_all(&mut *conn)
                .await
        } else {
            query_as::<_, GraphEdge>(sql)
                .bind(uuid)
                .bind(depth as i32)
                .fetch_all(&mut *conn)
                .await
        }
        .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        Ok(records)
    }

    /// Find the shortest path between two nodes using a single recursive CTE.
    ///
    /// Replaces the previous iterative BFS (N+1 queries) with a single
    /// PostgreSQL recursive CTE that computes shortest paths entirely in the
    /// database, reducing round-trips from O(depth) to O(1).
    ///
    /// Traverses edges in both directions (undirected). The search is
    /// bounded by `max_depth` (capped at 5).
    ///
    /// # Arguments
    /// * `source_id` - UUID of the starting node
    /// * `target_id` - UUID of the destination node
    /// * `max_depth` - Maximum hop count to explore (clamped to 5)
    ///
    /// # Returns
    /// An ordered list of node UUID strings from source to target.
    /// Returns a single-element vector when source and target are the same.
    /// Returns an empty vector when no path exists within the depth limit.
    ///
    /// # Errors
    /// Returns `DatabaseError::ValidationError` if either UUID is invalid,
    /// or `DatabaseError::QueryError` on SQL failures.
    #[instrument(skip(self))]
    pub async fn get_shortest_path(
        &self,
        source_id: &str,
        target_id: &str,
        max_depth: u32,
    ) -> DatabaseResult<Vec<String>> {
        let max_depth = max_depth.min(5);
        let source = uuid::Uuid::parse_str(source_id)
            .map_err(|e| DatabaseError::ValidationError(format!("Invalid source UUID: {}", e)))?;
        let target = uuid::Uuid::parse_str(target_id)
            .map_err(|e| DatabaseError::ValidationError(format!("Invalid target UUID: {}", e)))?;

        if source == target {
            return Ok(vec![source_id.to_string()]);
        }

        let sql = r#"
            WITH RECURSIVE bfs AS (
                SELECT
                    CASE WHEN e.source_id = $1 THEN e.target_id ELSE e.source_id END AS node_id,
                    ARRAY[$1, CASE WHEN e.source_id = $1 THEN e.target_id ELSE e.source_id END] AS path,
                    1 AS depth
                FROM knowledge_graph_edges e
                WHERE (e.source_id = $1 OR e.target_id = $1) AND e.is_active = true

                UNION ALL

                SELECT
                    CASE WHEN e.source_id = b.node_id THEN e.target_id ELSE e.source_id END,
                    b.path || CASE WHEN e.source_id = b.node_id THEN e.target_id ELSE e.source_id END,
                    b.depth + 1
                FROM bfs b
                JOIN knowledge_graph_edges e ON (e.source_id = b.node_id OR e.target_id = b.node_id)
                WHERE b.depth < $3
                  AND e.is_active = true
                  AND NOT (CASE WHEN e.source_id = b.node_id THEN e.target_id ELSE e.source_id END = ANY(b.path))
            )
            SELECT path FROM bfs WHERE node_id = $2
            ORDER BY depth
            LIMIT 1
        "#;

        let mut conn = self.pool.acquire().await?;
        let row = query_as::<_, BfsPathRow>(sql)
            .bind(source)
            .bind(target)
            .bind(max_depth as i32)
            .fetch_optional(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        match row {
            Some(r) => Ok(r.path.iter().map(|u| u.to_string()).collect()),
            None => Ok(vec![]),
        }
    }

    pub async fn get_node_edges(&self, node_id: &str) -> DatabaseResult<Vec<GraphEdge>> {
        let sql = "SELECT * FROM knowledge_graph_edges WHERE (source_id = $1 OR target_id = $1) AND is_active = true ORDER BY created_at DESC";
        let mut conn = self.pool.acquire().await?;
        let records = query_as::<_, GraphEdge>(sql)
            .bind(
                uuid::Uuid::parse_str(node_id)
                    .map_err(|e| DatabaseError::ValidationError(format!("Invalid UUID: {}", e)))?,
            )
            .fetch_all(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;
        Ok(records)
    }

    /// Compute the connected components of the active knowledge graph.
    ///
    /// Loads all active nodes and edges into memory and runs BFS to
    /// identify disjoint sub-graphs.
    ///
    /// # Returns
    /// A vector of components, where each component is a vector of node
    /// UUID strings. Returns an empty vector when the graph has no active nodes.
    ///
    /// # Errors
    /// Returns `DatabaseError::QueryError` on SQL failures.
    pub async fn get_connected_components(&self) -> DatabaseResult<Vec<Vec<String>>> {
        let sql = "SELECT id FROM knowledge_graph_nodes WHERE is_active = true ORDER BY id";
        let mut conn = self.pool.acquire().await?;
        let node_rows = sqlx::query(sql)
            .fetch_all(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        let node_ids: Vec<String> = node_rows
            .iter()
            .map(|r| r.get::<uuid::Uuid, _>(0).to_string())
            .collect();

        if node_ids.is_empty() {
            return Ok(vec![]);
        }

        let edge_sql =
            "SELECT source_id, target_id FROM knowledge_graph_edges WHERE is_active = true";
        let edge_rows = sqlx::query(edge_sql)
            .fetch_all(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        let mut adj: HashMap<String, Vec<String>> = HashMap::new();
        for id in &node_ids {
            adj.insert(id.clone(), vec![]);
        }
        for row in edge_rows {
            let s: String = row.get::<uuid::Uuid, _>(0).to_string();
            let t: String = row.get::<uuid::Uuid, _>(1).to_string();
            adj.entry(s.clone()).or_default().push(t.clone());
            adj.entry(t.clone()).or_default().push(s.clone());
        }

        let mut visited: HashSet<String> = HashSet::new();
        let mut components: Vec<Vec<String>> = vec![];

        for node_id in &node_ids {
            if visited.contains(node_id) {
                continue;
            }
            let mut component = vec![];
            let mut queue = VecDeque::new();
            queue.push_back(node_id.clone());
            visited.insert(node_id.clone());

            while let Some(current) = queue.pop_front() {
                component.push(current.clone());
                if let Some(neighbors) = adj.get(&current) {
                    for neighbor in neighbors {
                        if !visited.contains(neighbor) {
                            visited.insert(neighbor.clone());
                            queue.push_back(neighbor.clone());
                        }
                    }
                }
            }
            components.push(component);
        }

        Ok(components)
    }

    // ========================================================================
    // Graph Stats
    // ========================================================================

    pub async fn get_graph_stats(&self) -> DatabaseResult<serde_json::Value> {
        let sql = r#"
            SELECT
                (SELECT COUNT(*) FROM knowledge_graph_nodes WHERE is_active = true) AS node_count,
                (SELECT COUNT(*) FROM knowledge_graph_edges WHERE is_active = true) AS edge_count,
                (SELECT COALESCE(json_object_agg(node_type, cnt), '{}'::json)
                 FROM (
                     SELECT node_type, COUNT(*)::text AS cnt
                     FROM knowledge_graph_nodes WHERE is_active = true
                     GROUP BY node_type
                 ) t
                ) AS nodes_by_type,
                (SELECT COALESCE(json_object_agg(edge_type, cnt), '{}'::json)
                 FROM (
                     SELECT edge_type, COUNT(*)::text AS cnt
                     FROM knowledge_graph_edges WHERE is_active = true
                     GROUP BY edge_type
                 ) t
                ) AS edges_by_type
        "#;

        let mut conn = self.pool.acquire().await?;
        let row = sqlx::query(sql)
            .fetch_one(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        let node_count: i64 = row.get("node_count");
        let edge_count: i64 = row.get("edge_count");
        let nodes_by_type_raw: serde_json::Value = row.get("nodes_by_type");
        let edges_by_type_raw: serde_json::Value = row.get("edges_by_type");

        let nodes_by_type: serde_json::Map<String, serde_json::Value> = nodes_by_type_raw
            .as_object()
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .map(|(k, v)| {
                let cnt: i64 = v.as_str().and_then(|s| s.parse().ok()).unwrap_or(0);
                (k, json!(cnt))
            })
            .collect();

        let edges_by_type: serde_json::Map<String, serde_json::Value> = edges_by_type_raw
            .as_object()
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .map(|(k, v)| {
                let cnt: i64 = v.as_str().and_then(|s| s.parse().ok()).unwrap_or(0);
                (k, json!(cnt))
            })
            .collect();

        let avg_degree = if node_count > 0 {
            (edge_count as f64 * 2.0) / node_count as f64
        } else {
            0.0
        };

        Ok(json!({
            "node_count": node_count,
            "edge_count": edge_count,
            "nodes_by_type": nodes_by_type,
            "edges_by_type": edges_by_type,
            "avg_degree": avg_degree,
        }))
    }

    // ========================================================================
    // Temporal Queries
    // ========================================================================

    /// Query the graph state at a specific point in time.
    ///
    /// Returns all nodes and edges that were active at the given timestamp.
    /// An entity is "active at time T" when: `created_at <= T AND (deactivated_at IS NULL OR deactivated_at > T)`
    #[instrument(skip(self))]
    pub async fn get_graph_at_time(
        &self,
        at: DateTime<Utc>,
    ) -> DatabaseResult<(Vec<GraphNode>, Vec<GraphEdge>)> {
        let mut conn = self.pool.acquire().await?;

        let nodes = query_as::<_, GraphNode>(
            "SELECT * FROM knowledge_graph_nodes \
             WHERE created_at <= $1 AND (deactivated_at IS NULL OR deactivated_at > $1) \
             ORDER BY created_at ASC",
        )
        .bind(at)
        .fetch_all(&mut *conn)
        .await
        .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        let edges = query_as::<_, GraphEdge>(
            "SELECT * FROM knowledge_graph_edges \
             WHERE created_at <= $1 AND (deactivated_at IS NULL OR deactivated_at > $1) \
             ORDER BY created_at ASC",
        )
        .bind(at)
        .fetch_all(&mut *conn)
        .await
        .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        info!(
            "Graph at {}: {} nodes, {} edges",
            at,
            nodes.len(),
            edges.len()
        );

        Ok((nodes, edges))
    }

    /// Compute the diff of the graph between two timestamps.
    ///
    /// Returns four sets:
    /// - `added_nodes`: nodes active at `to` but not at `from`
    /// - `removed_nodes`: nodes active at `from` but not at `to`
    /// - `added_edges`: edges active at `to` but not at `from`
    /// - `removed_edges`: edges active at `from` but not at `to`
    #[instrument(skip(self))]
    pub async fn get_graph_diff(
        &self,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> DatabaseResult<GraphDiff> {
        let (nodes_from, edges_from) = self.get_graph_at_time(from).await?;
        let (nodes_to, edges_to) = self.get_graph_at_time(to).await?;

        let from_node_ids: HashSet<String> = nodes_from.iter().map(|n| n.id.clone()).collect();
        let to_node_ids: HashSet<String> = nodes_to.iter().map(|n| n.id.clone()).collect();
        let from_edge_ids: HashSet<String> = edges_from.iter().map(|e| e.id.clone()).collect();
        let to_edge_ids: HashSet<String> = edges_to.iter().map(|e| e.id.clone()).collect();

        let added_nodes: Vec<GraphNode> = nodes_to
            .into_iter()
            .filter(|n| !from_node_ids.contains(&n.id))
            .collect();

        let removed_nodes: Vec<GraphNode> = nodes_from
            .into_iter()
            .filter(|n| !to_node_ids.contains(&n.id))
            .collect();

        let added_edges: Vec<GraphEdge> = edges_to
            .into_iter()
            .filter(|e| !from_edge_ids.contains(&e.id))
            .collect();

        let removed_edges: Vec<GraphEdge> = edges_from
            .into_iter()
            .filter(|e| !to_edge_ids.contains(&e.id))
            .collect();

        info!(
            "Graph diff ({} → {}): +{} nodes, -{} nodes, +{} edges, -{} edges",
            from,
            to,
            added_nodes.len(),
            removed_nodes.len(),
            added_edges.len(),
            removed_edges.len(),
        );

        Ok(GraphDiff {
            added_nodes,
            removed_nodes,
            added_edges,
            removed_edges,
            from_timestamp: from,
            to_timestamp: to,
        })
    }

    /// Get orphan nodes: active nodes with zero active edges (neither source nor target).
    pub async fn get_orphan_nodes(&self) -> DatabaseResult<Vec<GraphNode>> {
        let sql = r#"
            SELECT id, node_type, name, slug, description, content, visibility, weight, properties,
                   project_id, document_id, created_by, is_active, created_at, updated_at, deactivated_at
            FROM knowledge_graph_nodes
            WHERE is_active = true
              AND NOT EXISTS (
                  SELECT 1 FROM knowledge_graph_edges e
                  WHERE e.is_active = true AND (e.source_id = knowledge_graph_nodes.id OR e.target_id = knowledge_graph_nodes.id)
              )
            ORDER BY created_at DESC
        "#;

        let mut conn = self.pool.acquire().await?;
        let nodes = query_as::<_, GraphNode>(sql)
            .fetch_all(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        info!("Found {} orphan nodes", nodes.len());
        Ok(nodes)
    }
}

/// Result of a graph diff between two timestamps.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphDiff {
    /// Nodes that became active between `from` and `to`
    pub added_nodes: Vec<GraphNode>,
    /// Nodes that were deactivated between `from` and `to`
    pub removed_nodes: Vec<GraphNode>,
    /// Edges that became active between `from` and `to`
    pub added_edges: Vec<GraphEdge>,
    /// Edges that were deactivated between `from` and `to`
    pub removed_edges: Vec<GraphEdge>,
    /// Start of the diff window
    pub from_timestamp: DateTime<Utc>,
    /// End of the diff window
    pub to_timestamp: DateTime<Utc>,
}
