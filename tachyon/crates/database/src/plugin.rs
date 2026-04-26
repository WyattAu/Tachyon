// Plugin Repository
// Database layer for managing installed plugins

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use tracing::instrument;

use crate::error::{DatabaseError, DatabaseResult};
use crate::schema::DatabasePool;

// ============================================================================
// Constants
// ============================================================================

const PLUGIN_SELECT_SQL: &str = r#"
    SELECT
        id::text as id,
        name,
        description,
        version,
        author,
        homepage,
        license,
        extension_points::text as extension_points,
        manifest::text as manifest,
        runtime_type,
        entry_point,
        enabled,
        installed_at,
        updated_at,
        installed_by::text as installed_by
    FROM plugins
"#;

// ============================================================================
// Domain Types
// ============================================================================

/// Installed plugin record
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Plugin {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub version: String,
    pub author: Option<String>,
    pub homepage: Option<String>,
    pub license: Option<String>,
    /// JSON string of extension points array
    pub extension_points: String,
    /// Full plugin manifest as JSON string
    pub manifest: Option<String>,
    /// Runtime type: "builtin", "wasm", "native"
    pub runtime_type: String,
    /// Path to plugin WASM binary or native library
    pub entry_point: Option<String>,
    pub enabled: bool,
    pub installed_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub installed_by: Option<String>,
}

impl Plugin {
    /// Parse extension_points JSON string into Vec<String>
    pub fn parse_extension_points(&self) -> DatabaseResult<Vec<String>> {
        serde_json::from_str(&self.extension_points).map_err(|e| {
            DatabaseError::SerializationError(format!("Failed to parse extension_points: {}", e))
        })
    }

    /// Serialize extension points to JSON string for storage
    pub fn serialize_extension_points(points: &[String]) -> DatabaseResult<String> {
        serde_json::to_string(points).map_err(|e| {
            DatabaseError::SerializationError(format!("Failed to serialize extension_points: {}", e))
        })
    }

    /// Parse manifest JSON string into a generic Value
    pub fn parse_manifest(&self) -> DatabaseResult<Option<serde_json::Value>> {
        match &self.manifest {
            Some(m) => {
                let val: serde_json::Value = serde_json::from_str(m).map_err(|e| {
                    DatabaseError::SerializationError(format!("Failed to parse manifest: {}", e))
                })?;
                Ok(Some(val))
            }
            None => Ok(None),
        }
    }
}

// ============================================================================
// Request DTOs
// ============================================================================

/// Create a new plugin record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePluginRequest {
    pub name: String,
    pub description: Option<String>,
    pub version: String,
    pub author: Option<String>,
    pub homepage: Option<String>,
    pub license: Option<String>,
    pub extension_points: Option<Vec<String>>,
    pub manifest: Option<serde_json::Value>,
    pub runtime_type: Option<String>,
    pub entry_point: Option<String>,
    pub enabled: Option<bool>,
    pub installed_by: Option<String>,
}

/// Update an existing plugin record
#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(Default)]
pub struct UpdatePluginRequest {
    pub description: Option<String>,
    pub version: Option<String>,
    pub author: Option<String>,
    pub homepage: Option<String>,
    pub license: Option<String>,
    pub extension_points: Option<Vec<String>>,
    pub manifest: Option<serde_json::Value>,
    pub entry_point: Option<String>,
    pub enabled: Option<bool>,
}

// ============================================================================
// Repository
// ============================================================================

#[derive(Clone)]
pub struct PluginRepository {
    pool: DatabasePool,
}

impl PluginRepository {
    pub fn new(pool: DatabasePool) -> Self {
        Self { pool }
    }

    /// Create a new plugin record
    #[instrument(skip(self, req))]
    pub async fn create(&self, req: CreatePluginRequest) -> DatabaseResult<Plugin> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = Utc::now();
        let extension_points = Plugin::serialize_extension_points(
            &req.extension_points.unwrap_or_default(),
        )?;
        let manifest = req.manifest
            .map(|m| serde_json::to_string(&m))
            .transpose()
            .map_err(|e| DatabaseError::SerializationError(format!("Failed to serialize manifest: {}", e)))?;
        let runtime_type = req.runtime_type.unwrap_or_else(|| "wasm".to_string());
        let enabled = req.enabled.unwrap_or(false);

        let sql = r#"
            INSERT INTO plugins (id, name, description, version, author, homepage, license,
                                 extension_points, manifest, runtime_type, entry_point,
                                 enabled, installed_at, updated_at, installed_by)
            VALUES ($1::uuid, $2, $3, $4, $5, $6, $7,
                    $8::jsonb, $9::jsonb, $10, $11,
                    $12, $13, $14, $15::uuid)
            RETURNING
                id::text as id, name, description, version, author, homepage, license,
                extension_points::text as extension_points, manifest::text as manifest,
                runtime_type, entry_point, enabled, installed_at, updated_at,
                installed_by::text as installed_by
            "#.to_string();

        let mut conn = self.pool.acquire().await?;
        let plugin = sqlx::query_as::<_, Plugin>(&sql)
            .bind(&id)
            .bind(&req.name)
            .bind(&req.description)
            .bind(&req.version)
            .bind(&req.author)
            .bind(&req.homepage)
            .bind(&req.license)
            .bind(&extension_points)
            .bind(&manifest)
            .bind(&runtime_type)
            .bind(&req.entry_point)
            .bind(enabled)
            .bind(now)
            .bind(now)
            .bind(&req.installed_by)
            .fetch_one(&mut *conn)
            .await
            .map_err(|e| {
                let msg = e.to_string();
                if msg.contains("duplicate key") || msg.contains("UNIQUE constraint") {
                    DatabaseError::duplicate("plugin", format!("Plugin '{}' version '{}' already installed", req.name, req.version))
                } else {
                    DatabaseError::query_error(&msg)
                }
            })?;

        Ok(plugin)
    }

    /// Get a plugin by ID
    #[instrument(skip(self))]
    pub async fn get_by_id(&self, id: &str) -> DatabaseResult<Plugin> {
        let sql = format!("{} WHERE id = $1::uuid", PLUGIN_SELECT_SQL);
        let mut conn = self.pool.acquire().await?;
        sqlx::query_as::<_, Plugin>(&sql)
            .bind(id)
            .fetch_optional(&mut *conn)
            .await?
            .ok_or_else(|| DatabaseError::not_found("plugin", id))
    }

    /// List plugins with optional filters
    #[instrument(skip(self))]
    pub async fn list(
        &self,
        enabled_only: Option<bool>,
        runtime_type: Option<&str>,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> DatabaseResult<Vec<Plugin>> {
        let mut conditions = vec![];
        let mut bind_idx = 0u32;

        if let Some(true) = enabled_only {
            bind_idx += 1;
            conditions.push(format!("enabled = ${}", bind_idx));
        }
        if let Some(_rt) = runtime_type {
            bind_idx += 1;
            conditions.push(format!("runtime_type = ${}", bind_idx));
        }

        let where_clause = if conditions.is_empty() {
            String::new()
        } else {
            format!(" WHERE {}", conditions.join(" AND "))
        };

        let order_clause = " ORDER BY name ASC, version DESC";
        let limit_clause = match limit {
            Some(l) => format!(" LIMIT {}", l),
            None => String::new(),
        };
        let offset_clause = match offset {
            Some(o) => format!(" OFFSET {}", o),
            None => String::new(),
        };

        let sql = format!(
            "{}{}{}{}{}",
            PLUGIN_SELECT_SQL, where_clause, order_clause, limit_clause, offset_clause
        );

        let mut conn = self.pool.acquire().await?;
        let mut query = sqlx::query_as::<_, Plugin>(&sql);

        if let Some(true) = enabled_only {
            query = query.bind(true);
        }
        if let Some(rt) = runtime_type {
            query = query.bind(rt);
        }

        let plugins = query.fetch_all(&mut *conn).await?;
        Ok(plugins)
    }

    /// Update a plugin
    #[instrument(skip(self, req))]
    pub async fn update(&self, id: &str, req: UpdatePluginRequest) -> DatabaseResult<Plugin> {
        // Fetch existing
        let existing = self.get_by_id(id).await?;

        let description = req.description.or(existing.description);
        let version = req.version.unwrap_or(existing.version);
        let author = req.author.or(existing.author);
        let homepage = req.homepage.or(existing.homepage);
        let license = req.license.or(existing.license);

        let extension_points = if let Some(ref points) = req.extension_points {
            Plugin::serialize_extension_points(points)?
        } else {
            existing.extension_points
        };

        let manifest = if let Some(ref m) = req.manifest {
            Some(serde_json::to_string(m).map_err(|e| {
                DatabaseError::SerializationError(format!("Failed to serialize manifest: {}", e))
            })?)
        } else {
            existing.manifest
        };

        let entry_point = req.entry_point.or(existing.entry_point);
        let enabled = req.enabled.unwrap_or(existing.enabled);
        let now = Utc::now();

        let sql = r#"
            UPDATE plugins SET
                description = $2,
                version = $3,
                author = $4,
                homepage = $5,
                license = $6,
                extension_points = $7::jsonb,
                manifest = $8::jsonb,
                entry_point = $9,
                enabled = $10,
                updated_at = $11
            WHERE id = $1::uuid
            RETURNING
                id::text as id, name, description, version, author, homepage, license,
                extension_points::text as extension_points, manifest::text as manifest,
                runtime_type, entry_point, enabled, installed_at, updated_at,
                installed_by::text as installed_by
        "#;

        let mut conn = self.pool.acquire().await?;
        let plugin = sqlx::query_as::<_, Plugin>(sql)
            .bind(id)
            .bind(&description)
            .bind(&version)
            .bind(&author)
            .bind(&homepage)
            .bind(&license)
            .bind(&extension_points)
            .bind(&manifest)
            .bind(&entry_point)
            .bind(enabled)
            .bind(now)
            .fetch_one(&mut *conn)
            .await
            .map_err(|e| DatabaseError::query_error(e.to_string()))?;

        Ok(plugin)
    }

    /// Toggle plugin enabled state
    #[instrument(skip(self))]
    pub async fn set_enabled(&self, id: &str, enabled: bool) -> DatabaseResult<Plugin> {
        self.update(id, UpdatePluginRequest {
            enabled: Some(enabled),
            ..Default::default()
        }).await
    }

    /// Delete a plugin
    #[instrument(skip(self))]
    pub async fn delete(&self, id: &str) -> DatabaseResult<()> {
        let sql = "DELETE FROM plugins WHERE id = $1::uuid";
        let mut conn = self.pool.acquire().await?;
        let result = sqlx::query(sql)
            .bind(id)
            .execute(&mut *conn)
            .await?;

        if result.rows_affected() == 0 {
            return Err(DatabaseError::not_found("plugin", id));
        }

        Ok(())
    }

    /// Count plugins with optional filter
    #[instrument(skip(self))]
    pub async fn count(&self, enabled_only: Option<bool>) -> DatabaseResult<i64> {
        let sql = if enabled_only == Some(true) {
            "SELECT COUNT(*) as count FROM plugins WHERE enabled = $1".to_string()
        } else {
            "SELECT COUNT(*) as count FROM plugins".to_string()
        };

        let mut q = sqlx::query_scalar::<_, i64>(&sql);

        if enabled_only == Some(true) {
            q = q.bind(true);
        }

        let mut conn = self.pool.acquire().await?;
        let count = q.fetch_one(&mut *conn).await?;
        Ok(count)
    }

    /// Get distinct runtime types
    #[instrument(skip(self))]
    pub async fn list_runtime_types(&self) -> DatabaseResult<Vec<String>> {
        let sql = "SELECT DISTINCT runtime_type FROM plugins ORDER BY runtime_type";
        let mut conn = self.pool.acquire().await?;
        let types = sqlx::query_scalar::<_, String>(sql)
            .fetch_all(&mut *conn)
            .await?;
        Ok(types)
    }
}

