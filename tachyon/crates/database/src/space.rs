// Space Repository
// Manages spaces (workspaces/vaults) for organizing documents into hierarchies

use crate::error::{DatabaseError, DatabaseResult};
use crate::schema::DatabasePool;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{query, query_as, FromRow, Row};
use std::collections::HashMap;
use tracing::{debug, info, instrument};

const SPACE_SELECT_SQL: &str = r#"
    SELECT
        id::text as id,
        name,
        slug,
        description,
        icon,
        color,
        owner_id::text as owner_id,
        parent_id::text as parent_id,
        visibility,
        sort_order,
        is_default,
        settings::text as settings,
        created_at,
        updated_at
    FROM spaces
"#;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Space {
    pub id: String,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub icon: String,
    pub color: String,
    pub owner_id: String,
    pub parent_id: Option<String>,
    pub visibility: String,
    pub sort_order: i32,
    pub is_default: bool,
    pub settings: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Space {
    pub fn parse_settings(&self) -> DatabaseResult<serde_json::Value> {
        serde_json::from_str(&self.settings)
            .map_err(|e| DatabaseError::SerializationError(e.to_string()))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSpaceRequest {
    pub name: String,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub color: Option<String>,
    pub parent_id: Option<String>,
    pub visibility: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateSpaceRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub color: Option<String>,
    pub parent_id: Option<Option<String>>,
    pub visibility: Option<String>,
    pub sort_order: Option<i32>,
}

// ============================================================================
// Space Members
// ============================================================================

const SPACE_MEMBER_SELECT_SQL: &str = r#"
    SELECT
        sm.id::text as id,
        sm.space_id::text as space_id,
        sm.user_id::text as user_id,
        sm.role as role,
        sm.joined_at,
        sm.invited_by::text as invited_by,
        u.username,
        u.display_name,
        u.avatar_url
    FROM space_members sm
    LEFT JOIN users u ON u.id = sm.user_id
"#;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SpaceMember {
    pub id: String,
    pub space_id: String,
    pub user_id: String,
    pub role: String,
    pub joined_at: DateTime<Utc>,
    pub invited_by: Option<String>,
    pub username: Option<String>,
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddSpaceMemberRequest {
    pub user_id: String,
    pub role: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateSpaceMemberRequest {
    pub role: String,
}

// ============================================================================
// Repository
// ============================================================================

#[derive(Clone)]
pub struct SpaceRepository {
    pool: DatabasePool,
}

impl SpaceRepository {
    pub fn new(pool: DatabasePool) -> Self {
        Self { pool }
    }

    // -- Space CRUD --

    #[instrument(skip(self, req))]
    pub async fn create(&self, owner_id: &str, req: CreateSpaceRequest) -> DatabaseResult<Space> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = Utc::now();
        let slug = slug::slugify(&req.name);
        let icon = req.icon.unwrap_or_else(|| "folder".to_string());
        let color = req.color.unwrap_or_else(|| "#3B82F6".to_string());
        let visibility = req.visibility.unwrap_or_else(|| "private".to_string());

        let insert_sql = r#"
            INSERT INTO spaces (
                id, name, slug, description, icon, color, owner_id, parent_id,
                visibility, sort_order, is_default, settings, created_at, updated_at
            ) VALUES (
                $1::uuid, $2, $3, $4, $5, $6, $7::uuid, $8::uuid,
                $9, 0, false, '{}', $10, $11
            )
            RETURNING id::text as id, name, slug, description, icon, color,
                owner_id::text as owner_id, parent_id::text as parent_id,
                visibility, sort_order, is_default, settings::text as settings,
                created_at, updated_at
        "#;

        let mut conn = self.pool.acquire().await?;
        let space: Space = query_as(insert_sql)
            .bind(&id)
            .bind(&req.name)
            .bind(&slug)
            .bind(&req.description)
            .bind(&icon)
            .bind(&color)
            .bind(owner_id)
            .bind(&req.parent_id)
            .bind(&visibility)
            .bind(now)
            .bind(now)
            .fetch_one(&mut *conn)
            .await
            .map_err(|e| {
                if e.to_string().contains("duplicate key")
                    || e.to_string().contains("UNIQUE constraint")
                {
                    DatabaseError::duplicate(
                        "space",
                        format!("Space '{}' already exists", req.name),
                    )
                } else {
                    DatabaseError::QueryError(e.to_string())
                }
            })?;

        // Auto-add owner as member
        let add_member_sql = r#"
            INSERT INTO space_members (space_id, user_id, role, invited_by)
            VALUES ($1::uuid, $2::uuid, 'owner', NULL)
            ON CONFLICT (space_id, user_id) DO NOTHING
        "#;
        let _ = query(add_member_sql)
            .bind(&id)
            .bind(owner_id)
            .execute(&mut *conn)
            .await;

        info!("Space created: {} ({})", req.name, slug);
        Ok(space)
    }

    #[instrument(skip(self))]
    pub async fn get_by_id(&self, id: &str) -> DatabaseResult<Space> {
        let select_sql = format!("{} WHERE id = $1::uuid", SPACE_SELECT_SQL);

        let mut conn = self.pool.acquire().await?;
        let space: Option<Space> = query_as(&select_sql)
            .bind(id)
            .fetch_optional(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        space.ok_or_else(|| DatabaseError::not_found("space", id))
    }

    #[instrument(skip(self))]
    pub async fn get_by_slug(&self, slug: &str, owner_id: &str) -> DatabaseResult<Space> {
        let select_sql = format!(
            "{} WHERE slug = $1 AND owner_id = $2::uuid",
            SPACE_SELECT_SQL
        );

        let mut conn = self.pool.acquire().await?;
        let space: Option<Space> = query_as(&select_sql)
            .bind(slug)
            .bind(owner_id)
            .fetch_optional(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        space.ok_or_else(|| DatabaseError::not_found("space", slug))
    }

    #[instrument(skip(self))]
    pub async fn list(
        &self,
        owner_id: Option<&str>,
        parent_id: Option<&str>,
        visibility: Option<&str>,
        include_member_spaces: Option<&str>,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> DatabaseResult<Vec<Space>> {
        let limit = limit.unwrap_or(100);
        let offset = offset.unwrap_or(0);
        let _ = include_member_spaces; // handled via owner_id filter below

        let (select_sql, has_owner, has_parent, has_visibility) = match (owner_id, parent_id, visibility) {
            (Some(_), Some(_), Some(_)) => (
                format!("{} WHERE (owner_id = $1::uuid OR id IN (SELECT space_id FROM space_members WHERE user_id = $2::uuid)) \
                         AND parent_id $3::uuid AND visibility = $4 \
                         ORDER BY sort_order ASC, name ASC LIMIT $5 OFFSET $6", SPACE_SELECT_SQL),
                true, true, true,
            ),
            (Some(_), Some(_), None) => (
                format!("{} WHERE (owner_id = $1::uuid OR id IN (SELECT space_id FROM space_members WHERE user_id = $2::uuid)) \
                         AND parent_id $3::uuid \
                         ORDER BY sort_order ASC, name ASC LIMIT $4 OFFSET $5", SPACE_SELECT_SQL),
                true, true, false,
            ),
            (Some(_), None, Some(_)) => (
                format!("{} WHERE (owner_id = $1::uuid OR id IN (SELECT space_id FROM space_members WHERE user_id = $2::uuid)) \
                         AND visibility = $3 \
                         ORDER BY sort_order ASC, name ASC LIMIT $4 OFFSET $5", SPACE_SELECT_SQL),
                true, false, true,
            ),
            (Some(_), None, None) => (
                format!("{} WHERE (owner_id = $1::uuid OR id IN (SELECT space_id FROM space_members WHERE user_id = $2::uuid)) \
                         ORDER BY sort_order ASC, name ASC LIMIT $3 OFFSET $4", SPACE_SELECT_SQL),
                true, false, false,
            ),
            (None, Some(_), Some(_)) => (
                format!("{} WHERE parent_id $1::uuid AND visibility = $2 \
                         ORDER BY sort_order ASC, name ASC LIMIT $3 OFFSET $4", SPACE_SELECT_SQL),
                false, true, true,
            ),
            (None, Some(_), None) => (
                format!("{} WHERE parent_id $1::uuid \
                         ORDER BY sort_order ASC, name ASC LIMIT $2 OFFSET $3", SPACE_SELECT_SQL),
                false, true, false,
            ),
            (None, None, Some(_)) => (
                format!("{} WHERE visibility = $1 \
                         ORDER BY sort_order ASC, name ASC LIMIT $2 OFFSET $3", SPACE_SELECT_SQL),
                false, false, true,
            ),
            (None, None, None) => (
                format!("{} ORDER BY sort_order ASC, name ASC LIMIT $1 OFFSET $2", SPACE_SELECT_SQL),
                false, false, false,
            ),
        };

        let mut conn = self.pool.acquire().await?;

        let spaces: Vec<Space> = match (has_owner, has_parent, has_visibility) {
            (true, true, true) => {
                let pid_bind = if parent_id.unwrap().is_empty() {
                    None::<String>
                } else {
                    parent_id.map(|s| s.to_string())
                };
                query_as(&select_sql)
                    .bind(owner_id.unwrap())
                    .bind(owner_id.unwrap())
                    .bind(pid_bind)
                    .bind(visibility.unwrap())
                    .bind(limit)
                    .bind(offset)
                    .fetch_all(&mut *conn)
                    .await
                    .map_err(|e| DatabaseError::QueryError(e.to_string()))?
            }
            (true, true, false) => {
                let pid_bind = if parent_id.unwrap().is_empty() {
                    None::<String>
                } else {
                    parent_id.map(|s| s.to_string())
                };
                query_as(&select_sql)
                    .bind(owner_id.unwrap())
                    .bind(owner_id.unwrap())
                    .bind(pid_bind)
                    .bind(limit)
                    .bind(offset)
                    .fetch_all(&mut *conn)
                    .await
                    .map_err(|e| DatabaseError::QueryError(e.to_string()))?
            }
            (true, false, true) => query_as(&select_sql)
                .bind(owner_id.unwrap())
                .bind(owner_id.unwrap())
                .bind(visibility.unwrap())
                .bind(limit)
                .bind(offset)
                .fetch_all(&mut *conn)
                .await
                .map_err(|e| DatabaseError::QueryError(e.to_string()))?,
            (true, false, false) => query_as(&select_sql)
                .bind(owner_id.unwrap())
                .bind(owner_id.unwrap())
                .bind(limit)
                .bind(offset)
                .fetch_all(&mut *conn)
                .await
                .map_err(|e| DatabaseError::QueryError(e.to_string()))?,
            (false, true, true) => {
                let pid_bind = if parent_id.unwrap().is_empty() {
                    None::<String>
                } else {
                    parent_id.map(|s| s.to_string())
                };
                query_as(&select_sql)
                    .bind(pid_bind)
                    .bind(visibility.unwrap())
                    .bind(limit)
                    .bind(offset)
                    .fetch_all(&mut *conn)
                    .await
                    .map_err(|e| DatabaseError::QueryError(e.to_string()))?
            }
            (false, true, false) => {
                let pid_bind = if parent_id.unwrap().is_empty() {
                    None::<String>
                } else {
                    parent_id.map(|s| s.to_string())
                };
                query_as(&select_sql)
                    .bind(pid_bind)
                    .bind(limit)
                    .bind(offset)
                    .fetch_all(&mut *conn)
                    .await
                    .map_err(|e| DatabaseError::QueryError(e.to_string()))?
            }
            (false, false, true) => query_as(&select_sql)
                .bind(visibility.unwrap())
                .bind(limit)
                .bind(offset)
                .fetch_all(&mut *conn)
                .await
                .map_err(|e| DatabaseError::QueryError(e.to_string()))?,
            (false, false, false) => query_as(&select_sql)
                .bind(limit)
                .bind(offset)
                .fetch_all(&mut *conn)
                .await
                .map_err(|e| DatabaseError::QueryError(e.to_string()))?,
        };

        debug!("Found {} spaces", spaces.len());
        Ok(spaces)
    }

    /// List top-level spaces (no parent) for a user
    #[instrument(skip(self))]
    pub async fn list_root_spaces(
        &self,
        user_id: &str,
        limit: Option<i64>,
    ) -> DatabaseResult<Vec<Space>> {
        let limit = limit.unwrap_or(100);
        let select_sql = format!(
            "{} WHERE (owner_id = $1::uuid OR id IN (SELECT space_id FROM space_members WHERE user_id = $1::uuid)) \
             AND parent_id IS NULL ORDER BY sort_order ASC, name ASC LIMIT $2",
            SPACE_SELECT_SQL
        );

        let mut conn = self.pool.acquire().await?;
        let spaces: Vec<Space> = query_as(&select_sql)
            .bind(user_id)
            .bind(limit)
            .fetch_all(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        debug!("Found {} root spaces for user {}", spaces.len(), user_id);
        Ok(spaces)
    }

    /// List child spaces of a given parent
    #[instrument(skip(self))]
    pub async fn list_child_spaces(
        &self,
        parent_id: &str,
        user_id: &str,
    ) -> DatabaseResult<Vec<Space>> {
        let select_sql = format!(
            "{} WHERE parent_id = $1::uuid AND \
             (owner_id = $2::uuid OR id IN (SELECT space_id FROM space_members WHERE user_id = $2::uuid)) \
             ORDER BY sort_order ASC, name ASC",
            SPACE_SELECT_SQL
        );

        let mut conn = self.pool.acquire().await?;
        let spaces: Vec<Space> = query_as(&select_sql)
            .bind(parent_id)
            .bind(user_id)
            .fetch_all(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        Ok(spaces)
    }

    /// Get the default (personal) space for a user
    #[instrument(skip(self))]
    pub async fn get_default_space(&self, user_id: &str) -> DatabaseResult<Space> {
        let select_sql = format!(
            "{} WHERE owner_id = $1::uuid AND is_default = true LIMIT 1",
            SPACE_SELECT_SQL
        );

        let mut conn = self.pool.acquire().await?;
        let space: Option<Space> = query_as(&select_sql)
            .bind(user_id)
            .fetch_optional(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        space.ok_or_else(|| DatabaseError::not_found("default_space", user_id))
    }

    #[instrument(skip(self, req))]
    pub async fn update(&self, id: &str, req: UpdateSpaceRequest) -> DatabaseResult<Space> {
        let existing = self.get_by_id(id).await?;
        let now = Utc::now();

        let name = req.name.clone().unwrap_or(existing.name);
        let slug = if req.name.is_some() {
            slug::slugify(&name)
        } else {
            existing.slug
        };
        let description = req.description.or(existing.description);
        let icon = req.icon.unwrap_or(existing.icon);
        let color = req.color.unwrap_or(existing.color);
        let parent_id = req.parent_id.unwrap_or(existing.parent_id);
        let visibility = req.visibility.unwrap_or(existing.visibility);
        let sort_order = req.sort_order.unwrap_or(existing.sort_order);

        let update_sql = r#"
            UPDATE spaces SET
                name = $1, slug = $2, description = $3, icon = $4, color = $5,
                parent_id = $6::uuid, visibility = $7, sort_order = $8, updated_at = $9
            WHERE id = $10::uuid
            RETURNING id::text as id, name, slug, description, icon, color,
                owner_id::text as owner_id, parent_id::text as parent_id,
                visibility, sort_order, is_default, settings::text as settings,
                created_at, updated_at
        "#;

        let mut conn = self.pool.acquire().await?;
        let space: Space = query_as(update_sql)
            .bind(&name)
            .bind(&slug)
            .bind(&description)
            .bind(&icon)
            .bind(&color)
            .bind(&parent_id)
            .bind(&visibility)
            .bind(sort_order)
            .bind(now)
            .bind(id)
            .fetch_one(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        info!("Space updated: {}", id);
        Ok(space)
    }

    #[instrument(skip(self))]
    pub async fn delete(&self, id: &str) -> DatabaseResult<()> {
        // Check if it's a default space -- prevent deletion
        let space = self.get_by_id(id).await?;
        if space.is_default {
            return Err(DatabaseError::QueryError(
                "Cannot delete the default personal space".to_string(),
            ));
        }

        let delete_sql = "DELETE FROM spaces WHERE id = $1::uuid";

        let mut conn = self.pool.acquire().await?;
        let result = query(delete_sql)
            .bind(id)
            .execute(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(DatabaseError::not_found("space", id));
        }

        info!("Space deleted: {}", id);
        Ok(())
    }

    #[instrument(skip(self))]
    pub async fn count(&self, owner_id: Option<&str>) -> DatabaseResult<i64> {
        let (count_sql, has_owner) = match owner_id {
            Some(_) => (
                "SELECT COUNT(*) as count FROM spaces WHERE owner_id = $1::uuid OR id IN (SELECT space_id FROM space_members WHERE user_id = $1::uuid)",
                true,
            ),
            None => ("SELECT COUNT(*) as count FROM spaces", false),
        };

        let mut conn = self.pool.acquire().await?;
        let row = if has_owner {
            query(count_sql)
                .bind(owner_id.unwrap())
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

    /// Count documents in a space
    #[instrument(skip(self))]
    pub async fn count_documents(&self, space_id: &str) -> DatabaseResult<i64> {
        let count_sql = "SELECT COUNT(*) as count FROM documents WHERE space_id = $1::uuid";

        let mut conn = self.pool.acquire().await?;
        let row = query(count_sql)
            .bind(space_id)
            .fetch_one(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        Ok(row.get("count"))
    }

    /// Count documents for multiple spaces in a single query
    #[instrument(skip(self))]
    pub async fn count_documents_batch(
        &self,
        space_ids: &[String],
    ) -> DatabaseResult<HashMap<String, i64>> {
        if space_ids.is_empty() {
            return Ok(HashMap::new());
        }

        let count_sql = r#"
            SELECT space_id::text as id, COUNT(*) as count
            FROM documents
            WHERE space_id = ANY($1::uuid[])
            GROUP BY space_id
        "#;

        let mut conn = self.pool.acquire().await?;
        let rows: Vec<(String, i64)> = query_as(count_sql)
            .bind(space_ids)
            .fetch_all(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        Ok(rows.into_iter().collect())
    }

    /// Move a document to a space
    #[instrument(skip(self))]
    pub async fn move_document(
        &self,
        document_id: &str,
        space_id: Option<&str>,
    ) -> DatabaseResult<()> {
        let update_sql =
            "UPDATE documents SET space_id = $1::uuid, updated_at = now() WHERE id = $2::uuid";

        let mut conn = self.pool.acquire().await?;
        query(update_sql)
            .bind(space_id)
            .bind(document_id)
            .execute(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        info!("Document {} moved to space {:?}", document_id, space_id);
        Ok(())
    }

    // -- Member management --

    #[instrument(skip(self, req))]
    pub async fn add_member(
        &self,
        space_id: &str,
        req: AddSpaceMemberRequest,
    ) -> DatabaseResult<SpaceMember> {
        let id = uuid::Uuid::new_v4().to_string();
        let role = req.role.unwrap_or_else(|| "viewer".to_string());

        let insert_sql = r#"
            INSERT INTO space_members (id, space_id, user_id, role)
            VALUES ($1::uuid, $2::uuid, $3::uuid, $4)
            RETURNING id::text as id, space_id::text as space_id, user_id::text as user_id,
                role, joined_at, invited_by::text as invited_by
        "#;

        let mut conn = self.pool.acquire().await?;
        query_as::<_, SpaceMember>(insert_sql)
            .bind(&id)
            .bind(space_id)
            .bind(&req.user_id)
            .bind(&role)
            .fetch_optional(&mut *conn)
            .await
            .map_err(|e| {
                if e.to_string().contains("duplicate key")
                    || e.to_string().contains("UNIQUE constraint")
                {
                    DatabaseError::duplicate(
                        "space_member",
                        "User is already a member of this space",
                    )
                } else {
                    DatabaseError::QueryError(e.to_string())
                }
            })?;

        // Re-fetch with user info (username, display_name, avatar_url)
        let select_sql = format!("{} WHERE sm.id = $1::uuid", SPACE_MEMBER_SELECT_SQL);
        let member: SpaceMember = query_as(&select_sql)
            .bind(&id)
            .fetch_one(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        info!("Added member {} to space {}", req.user_id, space_id);
        Ok(member)
    }

    #[instrument(skip(self))]
    pub async fn list_members(&self, space_id: &str) -> DatabaseResult<Vec<SpaceMember>> {
        let select_sql = format!(
            "{} WHERE sm.space_id = $1::uuid ORDER BY sm.role, sm.joined_at",
            SPACE_MEMBER_SELECT_SQL
        );

        let mut conn = self.pool.acquire().await?;
        let members: Vec<SpaceMember> = query_as(&select_sql)
            .bind(space_id)
            .fetch_all(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        Ok(members)
    }

    #[instrument(skip(self, req))]
    pub async fn update_member(
        &self,
        space_id: &str,
        user_id: &str,
        req: UpdateSpaceMemberRequest,
    ) -> DatabaseResult<SpaceMember> {
        let update_sql = r#"
            UPDATE space_members SET role = $1
            WHERE space_id = $2::uuid AND user_id = $3::uuid
        "#;

        let mut conn = self.pool.acquire().await?;
        let result = query(update_sql)
            .bind(&req.role)
            .bind(space_id)
            .bind(user_id)
            .execute(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(DatabaseError::not_found("space_member", user_id));
        }

        // Re-fetch with user info
        let select_sql = format!(
            "{} WHERE sm.space_id = $1::uuid AND sm.user_id = $2::uuid",
            SPACE_MEMBER_SELECT_SQL
        );
        let member: SpaceMember = query_as(&select_sql)
            .bind(space_id)
            .bind(user_id)
            .fetch_one(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        info!(
            "Updated member {} role to {} in space {}",
            user_id, req.role, space_id
        );
        Ok(member)
    }

    #[instrument(skip(self))]
    pub async fn remove_member(&self, space_id: &str, user_id: &str) -> DatabaseResult<()> {
        let delete_sql =
            "DELETE FROM space_members WHERE space_id = $1::uuid AND user_id = $2::uuid";

        let mut conn = self.pool.acquire().await?;
        let result = query(delete_sql)
            .bind(space_id)
            .bind(user_id)
            .execute(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(DatabaseError::not_found("space_member", user_id));
        }

        info!("Removed member {} from space {}", user_id, space_id);
        Ok(())
    }

    /// Check if a user is a member of a space (or is the owner)
    #[instrument(skip(self))]
    pub async fn is_member(&self, space_id: &str, user_id: &str) -> DatabaseResult<bool> {
        let check_sql = r#"
            SELECT EXISTS(
                SELECT 1 FROM spaces WHERE id = $1::uuid AND owner_id = $2::uuid
                UNION
                SELECT 1 FROM space_members WHERE space_id = $1::uuid AND user_id = $2::uuid
            ) as is_member
        "#;

        let mut conn = self.pool.acquire().await?;
        let row = query(check_sql)
            .bind(space_id)
            .bind(user_id)
            .fetch_one(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        Ok(row.get("is_member"))
    }
}
