// Document Template Repository
// Template management for reusable document structures

use crate::error::{DatabaseError, DatabaseResult};
use crate::schema::DatabasePool;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{query, query_as, FromRow, Row};
use tracing::{debug, info, instrument};

const TEMPLATE_SELECT_SQL: &str = r#"
    SELECT 
        id::text as id,
        name,
        description,
        content,
        category,
        tags::text as tags,
        created_at,
        updated_at,
        created_by::text as created_by
    FROM document_templates
"#;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DocumentTemplate {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub content: String,
    pub category: Option<String>,
    pub tags: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: String,
}

impl DocumentTemplate {
    pub fn parse_tags(&self) -> DatabaseResult<Vec<String>> {
        serde_json::from_str(&self.tags)
            .map_err(|e| DatabaseError::SerializationError(e.to_string()))
    }

    pub fn serialize_tags(tags: &[String]) -> DatabaseResult<String> {
        serde_json::to_string(tags)
            .map_err(|e| DatabaseError::SerializationError(e.to_string()))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTemplateRequest {
    pub name: String,
    pub description: Option<String>,
    pub content: String,
    pub category: Option<String>,
    pub tags: Option<Vec<String>>,
    pub created_by: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateTemplateRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub content: Option<String>,
    pub category: Option<String>,
    pub tags: Option<Vec<String>>,
}

#[derive(Clone)]
pub struct TemplateRepository {
    pool: DatabasePool,
}

impl TemplateRepository {
    pub fn new(pool: DatabasePool) -> Self {
        Self { pool }
    }

    #[instrument(skip(self, req))]
    pub async fn create(&self, req: CreateTemplateRequest) -> DatabaseResult<DocumentTemplate> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = Utc::now();
        let tags = DocumentTemplate::serialize_tags(&req.tags.unwrap_or_default())?;

        let insert_sql = r#"
            INSERT INTO document_templates (
                id, name, description, content, category, tags, created_at, updated_at, created_by
            ) VALUES ($1::uuid, $2, $3, $4, $5, $6::jsonb, $7, $8, $9::uuid)
            RETURNING id::text as id, name, description, content, category, tags::text as tags, created_at, updated_at, created_by::text as created_by
        "#;

        let mut conn = self.pool.acquire().await?;
        let template: DocumentTemplate = query_as(insert_sql)
            .bind(&id)
            .bind(&req.name)
            .bind(&req.description)
            .bind(&req.content)
            .bind(&req.category)
            .bind(&tags)
            .bind(&now)
            .bind(&now)
            .bind(&req.created_by)
            .fetch_one(&mut *conn)
            .await
            .map_err(|e| {
                if e.to_string().contains("duplicate key") || e.to_string().contains("UNIQUE constraint") {
                    DatabaseError::duplicate("template", format!("Template '{}' already exists", req.name))
                } else {
                    DatabaseError::QueryError(e.to_string())
                }
            })?;

        info!("Template created: {}", req.name);
        Ok(template)
    }

    #[instrument(skip(self))]
    pub async fn get_by_id(&self, id: &str) -> DatabaseResult<DocumentTemplate> {
        let select_sql = format!("{} WHERE id = $1::uuid", TEMPLATE_SELECT_SQL);
        
        let mut conn = self.pool.acquire().await?;
        let template: Option<DocumentTemplate> = query_as(&select_sql)
            .bind(id)
            .fetch_optional(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        template.ok_or_else(|| DatabaseError::not_found("template", id))
    }

    #[instrument(skip(self))]
    pub async fn get_by_name(&self, name: &str) -> DatabaseResult<DocumentTemplate> {
        let select_sql = format!("{} WHERE name = $1", TEMPLATE_SELECT_SQL);
        
        let mut conn = self.pool.acquire().await?;
        let template: Option<DocumentTemplate> = query_as(&select_sql)
            .bind(name)
            .fetch_optional(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        template.ok_or_else(|| DatabaseError::not_found("template", name))
    }

    #[instrument(skip(self))]
    pub async fn list(&self, category: Option<&str>, limit: Option<i64>, offset: Option<i64>) -> DatabaseResult<Vec<DocumentTemplate>> {
        let limit = limit.unwrap_or(50);
        let offset = offset.unwrap_or(0);

        let (select_sql, has_category) = match category {
            Some(_cat) => (format!("{} WHERE category = $1 ORDER BY name ASC LIMIT $2 OFFSET $3", TEMPLATE_SELECT_SQL), true),
            None => (format!("{} ORDER BY name ASC LIMIT $1 OFFSET $2", TEMPLATE_SELECT_SQL), false),
        };

        let mut conn = self.pool.acquire().await?;
        
        let templates: Vec<DocumentTemplate> = if has_category {
            query_as(&select_sql)
                .bind(category.unwrap())
                .bind(limit)
                .bind(offset)
                .fetch_all(&mut *conn)
                .await
                .map_err(|e| DatabaseError::QueryError(e.to_string()))?
        } else {
            query_as(&select_sql)
                .bind(limit)
                .bind(offset)
                .fetch_all(&mut *conn)
                .await
                .map_err(|e| DatabaseError::QueryError(e.to_string()))?
        };

        debug!("Found {} templates", templates.len());
        Ok(templates)
    }

    #[instrument(skip(self, req))]
    pub async fn update(&self, id: &str, req: UpdateTemplateRequest) -> DatabaseResult<DocumentTemplate> {
        let existing = self.get_by_id(id).await?;
        let now = Utc::now();

        let name = req.name.unwrap_or(existing.name);
        let description = req.description.or(existing.description);
        let content = req.content.unwrap_or(existing.content);
        let category = req.category.or(existing.category);
        let tags = match req.tags {
            Some(t) => DocumentTemplate::serialize_tags(&t)?,
            None => existing.tags,
        };

        let update_sql = r#"
            UPDATE document_templates SET
                name = $1, description = $2, content = $3, category = $4, tags = $5::jsonb, updated_at = $6
            WHERE id = $7::uuid
            RETURNING id::text as id, name, description, content, category, tags::text as tags, created_at, updated_at, created_by::text as created_by
        "#;

        let mut conn = self.pool.acquire().await?;
        let template: DocumentTemplate = query_as(update_sql)
            .bind(&name)
            .bind(&description)
            .bind(&content)
            .bind(&category)
            .bind(&tags)
            .bind(&now)
            .bind(id)
            .fetch_one(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        info!("Template updated: {}", id);
        Ok(template)
    }

    #[instrument(skip(self))]
    pub async fn delete(&self, id: &str) -> DatabaseResult<()> {
        let delete_sql = "DELETE FROM document_templates WHERE id = $1::uuid";

        let mut conn = self.pool.acquire().await?;
        let result = query(delete_sql)
            .bind(id)
            .execute(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(DatabaseError::not_found("template", id));
        }

        info!("Template deleted: {}", id);
        Ok(())
    }

    #[instrument(skip(self))]
    pub async fn list_categories(&self) -> DatabaseResult<Vec<String>> {
        let select_sql = "SELECT DISTINCT category FROM document_templates WHERE category IS NOT NULL ORDER BY category";

        let mut conn = self.pool.acquire().await?;
        let rows = query(select_sql)
            .fetch_all(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        Ok(rows.iter().filter_map(|r| r.get("category")).collect())
    }

    #[instrument(skip(self))]
    pub async fn count(&self, category: Option<&str>) -> DatabaseResult<i64> {
        let (count_sql, has_category) = match category {
            Some(_) => ("SELECT COUNT(*) as count FROM document_templates WHERE category = $1", true),
            None => ("SELECT COUNT(*) as count FROM document_templates", false),
        };

        let mut conn = self.pool.acquire().await?;
        let row = if has_category {
            query(count_sql)
                .bind(category.unwrap())
                .fetch_one(&mut *conn)
                .await
                .map_err(|e| DatabaseError::QueryError(e.to_string()))?
        } else {
            query(count_sql)
                .fetch_one(&mut *conn)
                .await
                .map_err(|e| DatabaseError::QueryError(e.to_string()))?
        };

        Ok(row.get("count"))
    }
}
