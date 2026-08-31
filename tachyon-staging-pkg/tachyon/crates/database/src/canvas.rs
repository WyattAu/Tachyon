use crate::error::{DatabaseError, DatabaseResult};
use crate::schema::DatabasePool;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{query, query_as};
use tracing::{info, instrument};

// ============================================================================
// Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Canvas {
    pub id: uuid::Uuid,
    pub title: String,
    pub owner_id: uuid::Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct CanvasNode {
    pub id: uuid::Uuid,
    pub canvas_id: uuid::Uuid,
    pub node_type: String,
    pub data: serde_json::Value,
    pub position_x: f64,
    pub position_y: f64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct CanvasEdge {
    pub id: uuid::Uuid,
    pub canvas_id: uuid::Uuid,
    pub source_id: uuid::Uuid,
    pub target_id: uuid::Uuid,
    pub edge_type: String,
    pub style: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateCanvasRequest {
    pub title: String,
    pub owner_id: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateCanvasRequest {
    pub title: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateCanvasNodeRequest {
    pub node_type: String,
    pub data: Option<serde_json::Value>,
    pub position_x: Option<f64>,
    pub position_y: Option<f64>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateCanvasNodeRequest {
    pub node_type: Option<String>,
    pub data: Option<serde_json::Value>,
    pub position_x: Option<f64>,
    pub position_y: Option<f64>,
}

#[derive(Debug, Deserialize)]
pub struct CreateCanvasEdgeRequest {
    pub source_id: String,
    pub target_id: String,
    pub edge_type: String,
    pub style: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateCanvasEdgeRequest {
    pub edge_type: Option<String>,
    pub style: Option<serde_json::Value>,
}

// ============================================================================
// Repository
// ============================================================================

pub struct CanvasRepository {
    pool: DatabasePool,
}

impl CanvasRepository {
    pub fn new(pool: DatabasePool) -> Self {
        Self { pool }
    }

    // ========================================================================
    // Canvas CRUD
    // ========================================================================

    #[instrument(skip(self))]
    pub async fn create_canvas(&self, req: CreateCanvasRequest) -> DatabaseResult<Canvas> {
        let owner_uuid = uuid::Uuid::parse_str(&req.owner_id)
            .map_err(|e| DatabaseError::ValidationError(format!("Invalid owner UUID: {}", e)))?;

        let sql = r#"
            INSERT INTO canvases (title, owner_id)
            VALUES ($1, $2)
            RETURNING id, title, owner_id, created_at, updated_at
        "#;

        let mut conn = self.pool.acquire().await?;
        let record = query_as::<_, Canvas>(sql)
            .bind(&req.title)
            .bind(owner_uuid)
            .fetch_one(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        info!("Canvas created: {} ({})", record.title, record.id);
        Ok(record)
    }

    #[instrument(skip(self))]
    pub async fn get_canvas_by_id(&self, id: &str) -> DatabaseResult<Canvas> {
        let uuid = uuid::Uuid::parse_str(id)
            .map_err(|e| DatabaseError::ValidationError(format!("Invalid UUID: {}", e)))?;

        let sql = "SELECT * FROM canvases WHERE id = $1";
        let mut conn = self.pool.acquire().await?;
        let record = query_as::<_, Canvas>(sql)
            .bind(uuid)
            .fetch_optional(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?
            .ok_or_else(|| DatabaseError::not_found("canvas", id))?;

        Ok(record)
    }

    #[instrument(skip(self))]
    pub async fn list_canvases(&self, owner_id: Option<&str>) -> DatabaseResult<Vec<Canvas>> {
        let mut sql = "SELECT * FROM canvases".to_string();
        let mut conn = self.pool.acquire().await?;

        let records = if let Some(oid) = owner_id {
            let uuid = uuid::Uuid::parse_str(oid)
                .map_err(|e| DatabaseError::ValidationError(format!("Invalid UUID: {}", e)))?;
            sql.push_str(" WHERE owner_id = $1");
            query_as::<_, Canvas>(&sql)
                .bind(uuid)
                .fetch_all(&mut *conn)
                .await
        } else {
            query_as::<_, Canvas>(&sql).fetch_all(&mut *conn).await
        }
        .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        Ok(records)
    }

    #[instrument(skip(self))]
    pub async fn update_canvas(
        &self,
        id: &str,
        req: UpdateCanvasRequest,
    ) -> DatabaseResult<Canvas> {
        let uuid = uuid::Uuid::parse_str(id)
            .map_err(|e| DatabaseError::ValidationError(format!("Invalid UUID: {}", e)))?;

        let sql = r#"
            UPDATE canvases SET
                title = COALESCE($2, title),
                updated_at = NOW()
            WHERE id = $1
            RETURNING id, title, owner_id, created_at, updated_at
        "#;

        let mut conn = self.pool.acquire().await?;
        let record = query_as::<_, Canvas>(sql)
            .bind(uuid)
            .bind(req.title)
            .fetch_optional(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?
            .ok_or_else(|| DatabaseError::not_found("canvas", id))?;

        Ok(record)
    }

    #[instrument(skip(self))]
    pub async fn delete_canvas(&self, id: &str) -> DatabaseResult<()> {
        let uuid = uuid::Uuid::parse_str(id)
            .map_err(|e| DatabaseError::ValidationError(format!("Invalid UUID: {}", e)))?;

        let sql = "DELETE FROM canvases WHERE id = $1";
        let mut conn = self.pool.acquire().await?;
        let result = query(sql)
            .bind(uuid)
            .execute(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(DatabaseError::not_found("canvas", id));
        }
        info!("Canvas deleted: {}", id);
        Ok(())
    }

    // ========================================================================
    // Canvas Node CRUD
    // ========================================================================

    #[instrument(skip(self))]
    pub async fn create_canvas_node(
        &self,
        canvas_id: &str,
        req: CreateCanvasNodeRequest,
    ) -> DatabaseResult<CanvasNode> {
        let canvas_uuid = uuid::Uuid::parse_str(canvas_id)
            .map_err(|e| DatabaseError::ValidationError(format!("Invalid canvas UUID: {}", e)))?;

        let sql = r#"
            INSERT INTO canvas_nodes (canvas_id, node_type, data, position_x, position_y)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id, canvas_id, node_type, data, position_x, position_y, created_at, updated_at
        "#;

        let mut conn = self.pool.acquire().await?;
        let record = query_as::<_, CanvasNode>(sql)
            .bind(canvas_uuid)
            .bind(&req.node_type)
            .bind(req.data.unwrap_or(serde_json::json!({})))
            .bind(req.position_x.unwrap_or(0.0))
            .bind(req.position_y.unwrap_or(0.0))
            .fetch_one(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        info!("Canvas node created: {} ({})", record.node_type, record.id);
        Ok(record)
    }

    #[instrument(skip(self))]
    pub async fn get_canvas_node_by_id(&self, id: &str) -> DatabaseResult<CanvasNode> {
        let uuid = uuid::Uuid::parse_str(id)
            .map_err(|e| DatabaseError::ValidationError(format!("Invalid UUID: {}", e)))?;

        let sql = "SELECT * FROM canvas_nodes WHERE id = $1";
        let mut conn = self.pool.acquire().await?;
        let record = query_as::<_, CanvasNode>(sql)
            .bind(uuid)
            .fetch_optional(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?
            .ok_or_else(|| DatabaseError::not_found("canvas_node", id))?;

        Ok(record)
    }

    #[instrument(skip(self))]
    pub async fn list_canvas_nodes(&self, canvas_id: &str) -> DatabaseResult<Vec<CanvasNode>> {
        let uuid = uuid::Uuid::parse_str(canvas_id)
            .map_err(|e| DatabaseError::ValidationError(format!("Invalid UUID: {}", e)))?;

        let sql = "SELECT * FROM canvas_nodes WHERE canvas_id = $1 ORDER BY created_at";
        let mut conn = self.pool.acquire().await?;
        let records = query_as::<_, CanvasNode>(sql)
            .bind(uuid)
            .fetch_all(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        Ok(records)
    }

    #[instrument(skip(self))]
    pub async fn update_canvas_node(
        &self,
        id: &str,
        req: UpdateCanvasNodeRequest,
    ) -> DatabaseResult<CanvasNode> {
        let uuid = uuid::Uuid::parse_str(id)
            .map_err(|e| DatabaseError::ValidationError(format!("Invalid UUID: {}", e)))?;

        let sql = r#"
            UPDATE canvas_nodes SET
                node_type = COALESCE($2, node_type),
                data = COALESCE($3, data),
                position_x = COALESCE($4, position_x),
                position_y = COALESCE($5, position_y),
                updated_at = NOW()
            WHERE id = $1
            RETURNING id, canvas_id, node_type, data, position_x, position_y, created_at, updated_at
        "#;

        let mut conn = self.pool.acquire().await?;
        let record = query_as::<_, CanvasNode>(sql)
            .bind(uuid)
            .bind(req.node_type)
            .bind(req.data)
            .bind(req.position_x)
            .bind(req.position_y)
            .fetch_optional(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?
            .ok_or_else(|| DatabaseError::not_found("canvas_node", id))?;

        Ok(record)
    }

    #[instrument(skip(self))]
    pub async fn delete_canvas_node(&self, id: &str) -> DatabaseResult<()> {
        let uuid = uuid::Uuid::parse_str(id)
            .map_err(|e| DatabaseError::ValidationError(format!("Invalid UUID: {}", e)))?;

        let sql = "DELETE FROM canvas_nodes WHERE id = $1";
        let mut conn = self.pool.acquire().await?;
        let result = query(sql)
            .bind(uuid)
            .execute(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(DatabaseError::not_found("canvas_node", id));
        }
        info!("Canvas node deleted: {}", id);
        Ok(())
    }

    // ========================================================================
    // Canvas Edge CRUD
    // ========================================================================

    #[instrument(skip(self))]
    pub async fn create_canvas_edge(
        &self,
        canvas_id: &str,
        req: CreateCanvasEdgeRequest,
    ) -> DatabaseResult<CanvasEdge> {
        let canvas_uuid = uuid::Uuid::parse_str(canvas_id)
            .map_err(|e| DatabaseError::ValidationError(format!("Invalid canvas UUID: {}", e)))?;
        let source_uuid = uuid::Uuid::parse_str(&req.source_id)
            .map_err(|e| DatabaseError::ValidationError(format!("Invalid source UUID: {}", e)))?;
        let target_uuid = uuid::Uuid::parse_str(&req.target_id)
            .map_err(|e| DatabaseError::ValidationError(format!("Invalid target UUID: {}", e)))?;

        let sql = r#"
            INSERT INTO canvas_edges (canvas_id, source_id, target_id, edge_type, style)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id, canvas_id, source_id, target_id, edge_type, style, created_at, updated_at
        "#;

        let mut conn = self.pool.acquire().await?;
        let record = query_as::<_, CanvasEdge>(sql)
            .bind(canvas_uuid)
            .bind(source_uuid)
            .bind(target_uuid)
            .bind(&req.edge_type)
            .bind(req.style.unwrap_or(serde_json::json!({})))
            .fetch_one(&mut *conn)
            .await
            .map_err(|e| {
                let msg = e.to_string();
                if msg.contains("foreign key") || msg.contains("references") {
                    DatabaseError::constraint_violation("Source or target node does not exist")
                } else {
                    DatabaseError::QueryError(msg)
                }
            })?;

        info!("Canvas edge created: {} ({})", record.edge_type, record.id);
        Ok(record)
    }

    #[instrument(skip(self))]
    pub async fn get_canvas_edge_by_id(&self, id: &str) -> DatabaseResult<CanvasEdge> {
        let uuid = uuid::Uuid::parse_str(id)
            .map_err(|e| DatabaseError::ValidationError(format!("Invalid UUID: {}", e)))?;

        let sql = "SELECT * FROM canvas_edges WHERE id = $1";
        let mut conn = self.pool.acquire().await?;
        let record = query_as::<_, CanvasEdge>(sql)
            .bind(uuid)
            .fetch_optional(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?
            .ok_or_else(|| DatabaseError::not_found("canvas_edge", id))?;

        Ok(record)
    }

    #[instrument(skip(self))]
    pub async fn list_canvas_edges(&self, canvas_id: &str) -> DatabaseResult<Vec<CanvasEdge>> {
        let uuid = uuid::Uuid::parse_str(canvas_id)
            .map_err(|e| DatabaseError::ValidationError(format!("Invalid UUID: {}", e)))?;

        let sql = "SELECT * FROM canvas_edges WHERE canvas_id = $1 ORDER BY created_at";
        let mut conn = self.pool.acquire().await?;
        let records = query_as::<_, CanvasEdge>(sql)
            .bind(uuid)
            .fetch_all(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        Ok(records)
    }

    #[instrument(skip(self))]
    pub async fn update_canvas_edge(
        &self,
        id: &str,
        req: UpdateCanvasEdgeRequest,
    ) -> DatabaseResult<CanvasEdge> {
        let uuid = uuid::Uuid::parse_str(id)
            .map_err(|e| DatabaseError::ValidationError(format!("Invalid UUID: {}", e)))?;

        let sql = r#"
            UPDATE canvas_edges SET
                edge_type = COALESCE($2, edge_type),
                style = COALESCE($3, style),
                updated_at = NOW()
            WHERE id = $1
            RETURNING id, canvas_id, source_id, target_id, edge_type, style, created_at, updated_at
        "#;

        let mut conn = self.pool.acquire().await?;
        let record = query_as::<_, CanvasEdge>(sql)
            .bind(uuid)
            .bind(req.edge_type)
            .bind(req.style)
            .fetch_optional(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?
            .ok_or_else(|| DatabaseError::not_found("canvas_edge", id))?;

        Ok(record)
    }

    #[instrument(skip(self))]
    pub async fn delete_canvas_edge(&self, id: &str) -> DatabaseResult<()> {
        let uuid = uuid::Uuid::parse_str(id)
            .map_err(|e| DatabaseError::ValidationError(format!("Invalid UUID: {}", e)))?;

        let sql = "DELETE FROM canvas_edges WHERE id = $1";
        let mut conn = self.pool.acquire().await?;
        let result = query(sql)
            .bind(uuid)
            .execute(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(DatabaseError::not_found("canvas_edge", id));
        }
        info!("Canvas edge deleted: {}", id);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_canvas_request_deserialization() {
        let req = CreateCanvasRequest {
            title: "Test Canvas".to_string(),
            owner_id: "550e8400-e29b-41d4-a716-446655440000".to_string(),
        };
        assert_eq!(req.title, "Test Canvas");
    }

    #[test]
    fn test_canvas_node_request_defaults() {
        let req = CreateCanvasNodeRequest {
            node_type: "text".to_string(),
            data: None,
            position_x: None,
            position_y: None,
        };
        assert_eq!(req.node_type, "text");
    }

    #[test]
    fn test_canvas_edge_request() {
        let req = CreateCanvasEdgeRequest {
            source_id: "550e8400-e29b-41d4-a716-446655440000".to_string(),
            target_id: "550e8400-e29b-41d4-a716-446655440001".to_string(),
            edge_type: "arrow".to_string(),
            style: None,
        };
        assert_eq!(req.edge_type, "arrow");
    }
}
