// Project/Service Catalog Repository
// Backstage-like catalog functionality (PostgreSQL)

use crate::error::{DatabaseError, DatabaseResult};
use crate::schema::DatabasePool;
use crate::types::*;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::{query, Row};
use tracing::{debug, info, instrument};

/// Project catalog repository
pub struct CatalogRepository {
    pool: DatabasePool,
}

impl CatalogRepository {
    /// Create a new catalog repository
    pub fn new(pool: DatabasePool) -> Self {
        Self { pool }
    }

    // ========================================================================
    // Project Operations
    // ========================================================================

    /// Create a new project
    #[instrument(skip(self))]
    pub async fn create_project(&self, project: &Project) -> DatabaseResult<()> {
        let tags_json = serde_json::to_string(&project.tags)
            .map_err(|e| DatabaseError::SerializationError(e.to_string()))?;

        let insert_sql = r#"
            INSERT INTO projects (
                id, name, slug, description, project_type, owner_id, lifecycle,
                repository_url, docs_url, api_url, tags, metadata, language,
                framework, visibility, status, created_at, updated_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18)
        "#;

        let mut conn = self.pool.acquire().await?;
        query(insert_sql)
            .bind(&project.id)
            .bind(&project.name)
            .bind(&project.slug)
            .bind(&project.description)
            .bind(&project.project_type)
            .bind(&project.owner_id)
            .bind(&project.lifecycle)
            .bind(&project.repository_url)
            .bind(&project.docs_url)
            .bind(&project.api_url)
            .bind(&tags_json)
            .bind(&project.metadata)
            .bind(&project.language)
            .bind(&project.framework)
            .bind(&project.visibility)
            .bind(&project.status)
            .bind(project.created_at)
            .bind(project.updated_at)
            .execute(&mut *conn)
            .await
            .map_err(|e| {
                if e.to_string().contains("unique") || e.to_string().contains("duplicate") {
                    DatabaseError::duplicate(
                        "project",
                        format!("Project '{}' already exists", project.slug),
                    )
                } else {
                    DatabaseError::QueryError(e.to_string())
                }
            })?;

        info!("Project created: {} ({})", project.name, project.slug);
        Ok(())
    }

    /// Get a project by ID
    pub async fn get_project(&self, id: &str) -> DatabaseResult<Project> {
        // Cast UUID columns to TEXT and extract JSONB as text for manual deserialization
        let select_sql = "SELECT id::text, name, slug, description, project_type, owner_id::text, \
             organization_id::text, lifecycle, repository_url, docs_url, api_url, \
             tags::text as tags_text, metadata::text as metadata_text, language, framework, visibility, status, created_at, updated_at \
             FROM projects WHERE id = $1::uuid";

        let mut conn = self.pool.acquire().await?;
        let row = query(select_sql)
            .bind(id)
            .fetch_optional(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        if let Some(row) = row {
            let tags_text: String = row.get("tags_text");
            let metadata_text: String = row.get("metadata_text");
            
            let tags: Vec<String> = serde_json::from_str(&tags_text)
                .map_err(|e| DatabaseError::SerializationError(format!("tags: {}", e)))?;
            let metadata: serde_json::Value = serde_json::from_str(&metadata_text)
                .map_err(|e| DatabaseError::SerializationError(format!("metadata: {}", e)))?;

            Ok(Project {
                id: row.get("id"),
                name: row.get("name"),
                slug: row.get("slug"),
                description: row.get("description"),
                project_type: row.get("project_type"),
                owner_id: row.get("owner_id"),
                organization_id: row.get("organization_id"),
                lifecycle: row.get("lifecycle"),
                repository_url: row.get("repository_url"),
                docs_url: row.get("docs_url"),
                api_url: row.get("api_url"),
                tags,
                metadata,
                language: row.get("language"),
                framework: row.get("framework"),
                visibility: row.get("visibility"),
                status: row.get("status"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            })
        } else {
            Err(DatabaseError::not_found("project", id))
        }
    }

    /// Get a project by slug
    pub async fn get_project_by_slug(&self, slug: &str) -> DatabaseResult<Project> {
        // Cast UUID columns to TEXT and extract JSONB as text for manual deserialization
        let select_sql = "SELECT id::text, name, slug, description, project_type, owner_id::text, \
             organization_id::text, lifecycle, repository_url, docs_url, api_url, \
             tags::text as tags_text, metadata::text as metadata_text, language, framework, visibility, status, created_at, updated_at \
             FROM projects WHERE slug = $1";

        let mut conn = self.pool.acquire().await?;
        let row = query(select_sql)
            .bind(slug)
            .fetch_optional(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        if let Some(row) = row {
            let tags_text: String = row.get("tags_text");
            let metadata_text: String = row.get("metadata_text");
            
            let tags: Vec<String> = serde_json::from_str(&tags_text)
                .map_err(|e| DatabaseError::SerializationError(format!("tags: {}", e)))?;
            let metadata: serde_json::Value = serde_json::from_str(&metadata_text)
                .map_err(|e| DatabaseError::SerializationError(format!("metadata: {}", e)))?;

            Ok(Project {
                id: row.get("id"),
                name: row.get("name"),
                slug: row.get("slug"),
                description: row.get("description"),
                project_type: row.get("project_type"),
                owner_id: row.get("owner_id"),
                organization_id: row.get("organization_id"),
                lifecycle: row.get("lifecycle"),
                repository_url: row.get("repository_url"),
                docs_url: row.get("docs_url"),
                api_url: row.get("api_url"),
                tags,
                metadata,
                language: row.get("language"),
                framework: row.get("framework"),
                visibility: row.get("visibility"),
                status: row.get("status"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            })
        } else {
            Err(DatabaseError::not_found("project", slug))
        }
    }

    /// List all projects with optional filters
    pub async fn list_projects(
        &self,
        project_type: Option<&str>,
        owner_id: Option<&str>,
        status: Option<&str>,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> DatabaseResult<Vec<Project>> {
        // Cast UUID columns to TEXT and extract JSONB as text for manual deserialization
        let mut sql = String::from(
            "SELECT id::text, name, slug, description, project_type, owner_id::text, \
             organization_id::text, lifecycle, repository_url, docs_url, api_url, \
             tags::text as tags_text, metadata::text as metadata_text, language, framework, visibility, status, created_at, updated_at \
             FROM projects WHERE 1=1"
        );
        let mut param_count = 0;

        if project_type.is_some() {
            param_count += 1;
            sql.push_str(&format!(" AND project_type = ${}", param_count));
        }
        if owner_id.is_some() {
            param_count += 1;
            sql.push_str(&format!(" AND owner_id = ${}", param_count));
        }
        if status.is_some() {
            param_count += 1;
            sql.push_str(&format!(" AND status = ${}", param_count));
        }

        sql.push_str(" ORDER BY updated_at DESC");

        if let Some(_lim) = limit {
            param_count += 1;
            sql.push_str(&format!(" LIMIT ${}", param_count));
        }
        if let Some(_off) = offset {
            param_count += 1;
            sql.push_str(&format!(" OFFSET ${}", param_count));
        }

        let mut conn = self.pool.acquire().await?;
        let mut query_builder = sqlx::query(&sql);

        if let Some(pt) = project_type {
            query_builder = query_builder.bind(pt);
        }
        if let Some(oid) = owner_id {
            query_builder = query_builder.bind(oid);
        }
        if let Some(s) = status {
            query_builder = query_builder.bind(s);
        }
        if let Some(lim) = limit {
            query_builder = query_builder.bind(lim);
        }
        if let Some(off) = offset {
            query_builder = query_builder.bind(off);
        }

        let rows = query_builder
            .fetch_all(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        // Manually deserialize each row
        let mut projects = Vec::new();
        for row in rows {
            let tags_text: String = row.get("tags_text");
            let metadata_text: String = row.get("metadata_text");
            
            let tags: Vec<String> = serde_json::from_str(&tags_text)
                .map_err(|e| DatabaseError::SerializationError(format!("tags: {}", e)))?;
            let metadata: serde_json::Value = serde_json::from_str(&metadata_text)
                .map_err(|e| DatabaseError::SerializationError(format!("metadata: {}", e)))?;

            let project = Project {
                id: row.get("id"),
                name: row.get("name"),
                slug: row.get("slug"),
                description: row.get("description"),
                project_type: row.get("project_type"),
                owner_id: row.get("owner_id"),
                organization_id: row.get("organization_id"),
                lifecycle: row.get("lifecycle"),
                repository_url: row.get("repository_url"),
                docs_url: row.get("docs_url"),
                api_url: row.get("api_url"),
                tags,
                metadata,
                language: row.get("language"),
                framework: row.get("framework"),
                visibility: row.get("visibility"),
                status: row.get("status"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            };
            projects.push(project);
        }

        Ok(projects)
    }

    /// Update a project
    #[instrument(skip(self))]
    pub async fn update_project(&self, project: &Project) -> DatabaseResult<()> {
        let tags_json = serde_json::to_string(&project.tags)
            .map_err(|e| DatabaseError::SerializationError(e.to_string()))?;

        let update_sql = r#"
            UPDATE projects SET
                name = $1, description = $2, project_type = $3, lifecycle = $4,
                repository_url = $5, docs_url = $6, api_url = $7, tags = $8,
                metadata = $9, language = $10, framework = $11, visibility = $12,
                status = $13, updated_at = $14
            WHERE id = $15
        "#;

        let mut conn = self.pool.acquire().await?;
        let result = query(update_sql)
            .bind(&project.name)
            .bind(&project.description)
            .bind(&project.project_type)
            .bind(&project.lifecycle)
            .bind(&project.repository_url)
            .bind(&project.docs_url)
            .bind(&project.api_url)
            .bind(&tags_json)
            .bind(&project.metadata)
            .bind(&project.language)
            .bind(&project.framework)
            .bind(&project.visibility)
            .bind(&project.status)
            .bind(project.updated_at)
            .bind(&project.id)
            .execute(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(DatabaseError::not_found("project", &project.id));
        }

        info!("Project updated: {}", project.id);
        Ok(())
    }

    /// Delete a project
    #[instrument(skip(self))]
    pub async fn delete_project(&self, id: &str) -> DatabaseResult<()> {
        let delete_sql = "DELETE FROM projects WHERE id = $1";

        let mut conn = self.pool.acquire().await?;
        let result = query(delete_sql)
            .bind(id)
            .execute(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(DatabaseError::not_found("project", id));
        }

        info!("Project deleted: {}", id);
        Ok(())
    }

    /// Search projects by name, description, or tags
    pub async fn search_projects(&self, query_text: &str, limit: Option<i64>) -> DatabaseResult<Vec<Project>> {
        let limit = limit.unwrap_or(50);
        let search_pattern = format!("%{}%", query_text.to_lowercase());

        // Cast UUID columns to TEXT and extract JSONB as text for manual deserialization
        let select_sql = r#"
            SELECT id::text, name, slug, description, project_type, owner_id::text,
             organization_id::text, lifecycle, repository_url, docs_url, api_url,
             tags::text as tags_text, metadata::text as metadata_text, language, framework, visibility, status, created_at, updated_at
            FROM projects
            WHERE LOWER(name) LIKE $1
               OR LOWER(description) LIKE $1
               OR EXISTS (
                   SELECT 1 FROM jsonb_array_elements_text(tags::jsonb) tag
                   WHERE LOWER(tag) LIKE $1
               )
            ORDER BY updated_at DESC
            LIMIT $2
        "#;

        let mut conn = self.pool.acquire().await?;
        let rows = query(select_sql)
            .bind(&search_pattern)
            .bind(limit)
            .fetch_all(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        // Manually deserialize each row
        let mut projects = Vec::new();
        for row in rows {
            let tags_text: String = row.get("tags_text");
            let metadata_text: String = row.get("metadata_text");
            
            let tags: Vec<String> = serde_json::from_str(&tags_text)
                .map_err(|e| DatabaseError::SerializationError(format!("tags: {}", e)))?;
            let metadata: serde_json::Value = serde_json::from_str(&metadata_text)
                .map_err(|e| DatabaseError::SerializationError(format!("metadata: {}", e)))?;

            let project = Project {
                id: row.get("id"),
                name: row.get("name"),
                slug: row.get("slug"),
                description: row.get("description"),
                project_type: row.get("project_type"),
                owner_id: row.get("owner_id"),
                organization_id: row.get("organization_id"),
                lifecycle: row.get("lifecycle"),
                repository_url: row.get("repository_url"),
                docs_url: row.get("docs_url"),
                api_url: row.get("api_url"),
                tags,
                metadata,
                language: row.get("language"),
                framework: row.get("framework"),
                visibility: row.get("visibility"),
                status: row.get("status"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            };
            projects.push(project);
        }

        Ok(projects)
    }

    // ========================================================================
    // Component Operations
    // ========================================================================

    /// Create a new component
    #[instrument(skip(self))]
    pub async fn create_component(&self, component: &Component) -> DatabaseResult<()> {
        let tags_json = serde_json::to_string(&component.tags)
            .map_err(|e| DatabaseError::SerializationError(e.to_string()))?;

        let insert_sql = r#"
            INSERT INTO components (
                id, name, component_type, project_id, owner_id, system_id,
                repository_url, docs_url, api_spec_url, tags, lifecycle,
                created_at, updated_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
        "#;

        let mut conn = self.pool.acquire().await?;
        query(insert_sql)
            .bind(&component.id)
            .bind(&component.name)
            .bind(&component.component_type)
            .bind(&component.project_id)
            .bind(&component.owner_id)
            .bind(&component.system_id)
            .bind(&component.repository_url)
            .bind(&component.docs_url)
            .bind(&component.api_spec_url)
            .bind(&tags_json)
            .bind(&component.lifecycle)
            .bind(component.created_at)
            .bind(component.updated_at)
            .execute(&mut *conn)
            .await
            .map_err(|e| {
                if e.to_string().contains("unique") || e.to_string().contains("duplicate") {
                    DatabaseError::duplicate(
                        "component",
                        format!("Component '{}' already exists", component.name),
                    )
                } else {
                    DatabaseError::QueryError(e.to_string())
                }
            })?;

        info!("Component created: {} ({})", component.name, component.id);
        Ok(())
    }

    /// Get a component by ID
    pub async fn get_component(&self, id: &str) -> DatabaseResult<Component> {
        // Cast UUID columns to TEXT and extract JSONB as text for manual deserialization
        let select_sql = "SELECT id::text, name, component_type, project_id::text, owner_id::text, \
             system_id::text, repository_url, docs_url, api_spec_url, tags::text as tags_text, lifecycle, created_at, updated_at \
             FROM components WHERE id = $1::uuid";

        let mut conn = self.pool.acquire().await?;
        let row = query(select_sql)
            .bind(id)
            .fetch_optional(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        if let Some(row) = row {
            let tags_text: String = row.get("tags_text");
            let tags: Vec<String> = serde_json::from_str(&tags_text)
                .map_err(|e| DatabaseError::SerializationError(format!("tags: {}", e)))?;

            Ok(Component {
                id: row.get("id"),
                name: row.get("name"),
                component_type: row.get("component_type"),
                project_id: row.get("project_id"),
                owner_id: row.get("owner_id"),
                system_id: row.get("system_id"),
                repository_url: row.get("repository_url"),
                docs_url: row.get("docs_url"),
                api_spec_url: row.get("api_spec_url"),
                tags,
                lifecycle: row.get("lifecycle"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            })
        } else {
            Err(DatabaseError::not_found("component", id))
        }
    }

    /// List components by project
    pub async fn list_components_by_project(&self, project_id: &str) -> DatabaseResult<Vec<Component>> {
        // Cast UUID columns to TEXT and extract JSONB as text for manual deserialization
        let select_sql = "SELECT id::text, name, component_type, project_id::text, owner_id::text, \
             system_id::text, repository_url, docs_url, api_spec_url, tags::text as tags_text, lifecycle, created_at, updated_at \
             FROM components WHERE project_id = $1::uuid ORDER BY name";

        let mut conn = self.pool.acquire().await?;
        let rows = query(select_sql)
            .bind(project_id)
            .fetch_all(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        // Manually deserialize each row
        let mut components = Vec::new();
        for row in rows {
            let tags_text: String = row.get("tags_text");
            let tags: Vec<String> = serde_json::from_str(&tags_text)
                .map_err(|e| DatabaseError::SerializationError(format!("tags: {}", e)))?;

            let component = Component {
                id: row.get("id"),
                name: row.get("name"),
                component_type: row.get("component_type"),
                project_id: row.get("project_id"),
                owner_id: row.get("owner_id"),
                system_id: row.get("system_id"),
                repository_url: row.get("repository_url"),
                docs_url: row.get("docs_url"),
                api_spec_url: row.get("api_spec_url"),
                tags,
                lifecycle: row.get("lifecycle"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            };
            components.push(component);
        }

        Ok(components)
    }

    /// Delete a component
    #[instrument(skip(self))]
    pub async fn delete_component(&self, id: &str) -> DatabaseResult<()> {
        let delete_sql = "DELETE FROM components WHERE id = $1";

        let mut conn = self.pool.acquire().await?;
        let result = query(delete_sql)
            .bind(id)
            .execute(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(DatabaseError::not_found("component", id));
        }

        info!("Component deleted: {}", id);
        Ok(())
    }

    // ========================================================================
    // Project Member Operations
    // ========================================================================

    /// Add a member to a project
    #[instrument(skip(self))]
    pub async fn add_project_member(&self, member: &ProjectMember) -> DatabaseResult<()> {
        let insert_sql = r#"
            INSERT INTO project_members (project_id, user_id, role, added_by, added_at)
            VALUES ($1, $2, $3, $4, $5)
        "#;

        let mut conn = self.pool.acquire().await?;
        query(insert_sql)
            .bind(&member.project_id)
            .bind(&member.user_id)
            .bind(&member.role)
            .bind(&member.added_by)
            .bind(member.added_at)
            .execute(&mut *conn)
            .await
            .map_err(|e| {
                if e.to_string().contains("unique") || e.to_string().contains("duplicate") {
                    DatabaseError::duplicate(
                        "project_member",
                        format!("User {} is already a member of project {}", member.user_id, member.project_id),
                    )
                } else {
                    DatabaseError::QueryError(e.to_string())
                }
            })?;

        debug!("Member added to project: {} -> {}", member.user_id, member.project_id);
        Ok(())
    }

    /// List project members
    pub async fn list_project_members(&self, project_id: &str) -> DatabaseResult<Vec<ProjectMember>> {
        // Cast UUID columns to TEXT for compatibility
        let select_sql = "SELECT id, project_id::text, user_id::text, role, added_by::text, added_at \
             FROM project_members WHERE project_id = $1::uuid ORDER BY added_at";

        let mut conn = self.pool.acquire().await?;
        let rows = query(select_sql)
            .bind(project_id)
            .fetch_all(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        // Manually deserialize each row
        let mut members = Vec::new();
        for row in rows {
            let member = ProjectMember {
                id: row.get("id"),
                project_id: row.get("project_id"),
                user_id: row.get("user_id"),
                role: row.get("role"),
                added_by: row.get("added_by"),
                added_at: row.get("added_at"),
            };
            members.push(member);
        }

        Ok(members)
    }

    /// Remove a member from a project
    #[instrument(skip(self))]
    pub async fn remove_project_member(&self, project_id: &str, user_id: &str) -> DatabaseResult<()> {
        let delete_sql = "DELETE FROM project_members WHERE project_id = $1 AND user_id = $2";

        let mut conn = self.pool.acquire().await?;
        let result = query(delete_sql)
            .bind(project_id)
            .bind(user_id)
            .execute(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(DatabaseError::not_found(
                "project_member",
                format!("{}:{}", project_id, user_id),
            ));
        }

        debug!("Member removed from project: {} -> {}", user_id, project_id);
        Ok(())
    }

    // ========================================================================
    // Statistics
    // ========================================================================

    /// Get catalog statistics
    pub async fn get_stats(&self) -> DatabaseResult<CatalogStats> {
        let mut conn = self.pool.acquire().await?;

        let project_count: i64 = query("SELECT COUNT(*) as count FROM projects")
            .fetch_one(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?
            .get("count");

        let component_count: i64 = query("SELECT COUNT(*) as count FROM components")
            .fetch_one(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?
            .get("count");

        let member_count: i64 = query("SELECT COUNT(*) as count FROM project_members")
            .fetch_one(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?
            .get("count");

        Ok(CatalogStats {
            project_count,
            component_count,
            member_count,
        })
    }
}

/// Catalog statistics
#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct CatalogStats {
    /// Total number of projects
    pub project_count: i64,
    /// Total number of components
    pub component_count: i64,
    /// Total number of project members
    pub member_count: i64,
}

/// Request to create a new project
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateProjectRequest {
    /// Project name
    pub name: String,
    /// Project slug
    pub slug: String,
    /// Description
    pub description: Option<String>,
    /// Project type
    #[serde(default = "default_project_type")]
    pub project_type: String,
    /// Owner ID
    pub owner_id: String,
    /// Repository URL
    pub repository_url: Option<String>,
    /// Tags
    #[serde(default)]
    pub tags: Vec<String>,
    /// Language
    pub language: Option<String>,
    /// Framework
    pub framework: Option<String>,
    /// Visibility
    #[serde(default = "default_visibility")]
    pub visibility: String,
}

fn default_project_type() -> String {
    "service".to_string()
}

fn default_visibility() -> String {
    "internal".to_string()
}

impl CreateProjectRequest {
    /// Convert to Project model
    pub fn to_project(&self, id: String) -> Project {
        let now = Utc::now();
        Project {
            id,
            name: self.name.clone(),
            slug: self.slug.clone(),
            description: self.description.clone(),
            project_type: self.project_type.clone(),
            owner_id: self.owner_id.clone(),
            organization_id: None,
            lifecycle: "experimental".to_string(),
            repository_url: self.repository_url.clone(),
            docs_url: None,
            api_url: None,
            tags: self.tags.clone(),
            metadata: serde_json::json!({}),
            language: self.language.clone(),
            framework: self.framework.clone(),
            visibility: self.visibility.clone(),
            status: "active".to_string(),
            created_at: now,
            updated_at: now,
        }
    }
}

/// Request to create a new component
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateComponentRequest {
    /// Component name
    pub name: String,
    /// Component type
    pub component_type: String,
    /// Project ID
    pub project_id: String,
    /// Owner ID
    pub owner_id: String,
    /// System ID
    pub system_id: Option<String>,
    /// Repository URL
    pub repository_url: Option<String>,
    /// Tags
    #[serde(default)]
    pub tags: Vec<String>,
}

impl CreateComponentRequest {
    /// Convert to Component model
    pub fn to_component(&self, id: String) -> Component {
        let now = Utc::now();
        Component {
            id,
            name: self.name.clone(),
            component_type: self.component_type.clone(),
            project_id: self.project_id.clone(),
            owner_id: self.owner_id.clone(),
            system_id: self.system_id.clone(),
            repository_url: self.repository_url.clone(),
            docs_url: None,
            api_spec_url: None,
            tags: self.tags.clone(),
            lifecycle: "experimental".to_string(),
            created_at: now,
            updated_at: now,
        }
    }
}
