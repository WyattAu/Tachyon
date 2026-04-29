// Organization Repository
// Manages organizations (teams/companies) for multi-tenant document scoping

use crate::error::{DatabaseError, DatabaseResult};
use crate::schema::DatabasePool;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{query, query_as, FromRow, Row};
use std::collections::HashMap;
use tracing::{debug, info, instrument};

const ORG_SELECT_SQL: &str = r#"
    SELECT
        id::text as id,
        name,
        slug,
        description,
        icon,
        logo_url,
        owner_id::text as owner_id,
        default_role,
        max_members,
        is_personal,
        settings::text as settings,
        created_at,
        updated_at
    FROM organizations
"#;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Organization {
    pub id: String,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub icon: String,
    pub logo_url: Option<String>,
    pub owner_id: String,
    pub default_role: String,
    pub max_members: i32,
    pub is_personal: bool,
    pub settings: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Organization {
    pub fn parse_settings(&self) -> DatabaseResult<serde_json::Value> {
        serde_json::from_str(&self.settings)
            .map_err(|e| DatabaseError::SerializationError(e.to_string()))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateOrganizationRequest {
    pub name: String,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub logo_url: Option<String>,
    pub default_role: Option<String>,
    pub max_members: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateOrganizationRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub logo_url: Option<String>,
    pub default_role: Option<String>,
    pub max_members: Option<i32>,
    pub settings: Option<serde_json::Value>,
}

// ============================================================================
// Organization Members
// ============================================================================

const ORG_MEMBER_SELECT_SQL: &str = r#"
    SELECT
        om.id::text as id,
        om.organization_id::text as organization_id,
        om.user_id::text as user_id,
        om.role as role,
        om.joined_at,
        om.invited_by::text as invited_by,
        u.username,
        u.display_name,
        u.avatar_url
    FROM organization_members om
    LEFT JOIN users u ON u.id = om.user_id
"#;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct OrganizationMember {
    pub id: String,
    pub organization_id: String,
    pub user_id: String,
    pub role: String,
    pub joined_at: DateTime<Utc>,
    pub invited_by: Option<String>,
    pub username: Option<String>,
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddOrganizationMemberRequest {
    pub user_id: String,
    pub role: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateOrganizationMemberRequest {
    pub role: String,
}

// ============================================================================
// Repository
// ============================================================================

#[derive(Clone)]
pub struct OrganizationRepository {
    pool: DatabasePool,
}

impl OrganizationRepository {
    pub fn new(pool: DatabasePool) -> Self {
        Self { pool }
    }

    // -- Organization CRUD --

    #[instrument(skip(self, req))]
    pub async fn create(
        &self,
        owner_id: &str,
        req: CreateOrganizationRequest,
    ) -> DatabaseResult<Organization> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = Utc::now();
        let slug = slug::slugify(&req.name);
        let icon = req.icon.unwrap_or_else(|| "building".to_string());
        let default_role = req.default_role.unwrap_or_else(|| "viewer".to_string());
        let max_members = req.max_members.unwrap_or(-1);

        let insert_sql = r#"
            INSERT INTO organizations (
                id, name, slug, description, icon, logo_url, owner_id,
                default_role, max_members, is_personal, settings, created_at, updated_at
            ) VALUES (
                $1::uuid, $2, $3, $4, $5, $6, $7::uuid,
                $8, $9, false, '{}', $10, $11
            )
            RETURNING id::text as id, name, slug, description, icon, logo_url,
                owner_id::text as owner_id, default_role, max_members, is_personal,
                settings::text as settings, created_at, updated_at
        "#;

        let mut conn = self.pool.acquire().await?;
        let org: Organization = query_as(insert_sql)
            .bind(&id)
            .bind(&req.name)
            .bind(&slug)
            .bind(&req.description)
            .bind(&icon)
            .bind(&req.logo_url)
            .bind(owner_id)
            .bind(&default_role)
            .bind(max_members)
            .bind(now)
            .bind(now)
            .fetch_one(&mut *conn)
            .await
            .map_err(|e| {
                if e.to_string().contains("duplicate key")
                    || e.to_string().contains("UNIQUE constraint")
                {
                    DatabaseError::duplicate(
                        "organization",
                        format!("Organization '{}' already exists", req.name),
                    )
                } else {
                    DatabaseError::QueryError(e.to_string())
                }
            })?;

        // Auto-add owner as member
        let add_member_sql = r#"
            INSERT INTO organization_members (organization_id, user_id, role, invited_by)
            VALUES ($1::uuid, $2::uuid, 'owner', NULL)
            ON CONFLICT (organization_id, user_id) DO NOTHING
        "#;
        let _ = query(add_member_sql)
            .bind(&id)
            .bind(owner_id)
            .execute(&mut *conn)
            .await;

        info!("Organization created: {} ({})", req.name, slug);
        Ok(org)
    }

    #[instrument(skip(self))]
    pub async fn get_by_id(&self, id: &str) -> DatabaseResult<Organization> {
        let select_sql = format!("{} WHERE id = $1::uuid", ORG_SELECT_SQL);

        let mut conn = self.pool.acquire().await?;
        let org: Option<Organization> = query_as(&select_sql)
            .bind(id)
            .fetch_optional(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        org.ok_or_else(|| DatabaseError::not_found("organization", id))
    }

    #[instrument(skip(self))]
    pub async fn get_by_slug(&self, slug: &str) -> DatabaseResult<Organization> {
        let select_sql = format!("{} WHERE slug = $1", ORG_SELECT_SQL);

        let mut conn = self.pool.acquire().await?;
        let org: Option<Organization> = query_as(&select_sql)
            .bind(slug)
            .fetch_optional(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        org.ok_or_else(|| DatabaseError::not_found("organization", slug))
    }

    /// Get the personal organization for a user
    #[instrument(skip(self))]
    pub async fn get_personal_org(&self, user_id: &str) -> DatabaseResult<Organization> {
        let select_sql = format!(
            "{} WHERE owner_id = $1::uuid AND is_personal = true LIMIT 1",
            ORG_SELECT_SQL
        );

        let mut conn = self.pool.acquire().await?;
        let org: Option<Organization> = query_as(&select_sql)
            .bind(user_id)
            .fetch_optional(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        org.ok_or_else(|| DatabaseError::not_found("personal_organization", user_id))
    }

    /// List organizations a user belongs to (either as owner or member)
    #[instrument(skip(self))]
    pub async fn list_for_user(
        &self,
        user_id: &str,
        include_personal: bool,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> DatabaseResult<Vec<Organization>> {
        let limit = limit.unwrap_or(100);
        let offset = offset.unwrap_or(0);

        let select_sql = if include_personal {
            format!(
                "{} WHERE (owner_id = $1::uuid OR id IN (SELECT organization_id FROM organization_members WHERE user_id = $1::uuid)) \
                 ORDER BY is_personal ASC, name ASC LIMIT $2 OFFSET $3",
                ORG_SELECT_SQL
            )
        } else {
            format!(
                "{} WHERE (owner_id = $1::uuid OR id IN (SELECT organization_id FROM organization_members WHERE user_id = $1::uuid)) \
                 AND is_personal = false \
                 ORDER BY name ASC LIMIT $2 OFFSET $3",
                ORG_SELECT_SQL
            )
        };

        let mut conn = self.pool.acquire().await?;
        let orgs: Vec<Organization> = query_as(&select_sql)
            .bind(user_id)
            .bind(limit)
            .bind(offset)
            .fetch_all(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        debug!("Found {} organizations for user {}", orgs.len(), user_id);
        Ok(orgs)
    }

    #[instrument(skip(self, req))]
    pub async fn update(
        &self,
        id: &str,
        req: UpdateOrganizationRequest,
    ) -> DatabaseResult<Organization> {
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
        let logo_url = req.logo_url.or(existing.logo_url);
        let default_role = req.default_role.unwrap_or(existing.default_role);
        let max_members = req.max_members.unwrap_or(existing.max_members);
        let settings = if let Some(s) = req.settings {
            serde_json::to_string(&s).unwrap_or_else(|_| "{}".to_string())
        } else {
            existing.settings.clone()
        };

        let update_sql = r#"
            UPDATE organizations SET
                name = $1, slug = $2, description = $3, icon = $4, logo_url = $5,
                default_role = $6, max_members = $7, settings = $8::jsonb, updated_at = $9
            WHERE id = $10::uuid
            RETURNING id::text as id, name, slug, description, icon, logo_url,
                owner_id::text as owner_id, default_role, max_members, is_personal,
                settings::text as settings, created_at, updated_at
        "#;

        let mut conn = self.pool.acquire().await?;
        let org: Organization = query_as(update_sql)
            .bind(&name)
            .bind(&slug)
            .bind(&description)
            .bind(&icon)
            .bind(&logo_url)
            .bind(&default_role)
            .bind(max_members)
            .bind(&settings)
            .bind(now)
            .bind(id)
            .fetch_one(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        info!("Organization updated: {}", id);
        Ok(org)
    }

    #[instrument(skip(self))]
    pub async fn delete(&self, id: &str) -> DatabaseResult<()> {
        let org = self.get_by_id(id).await?;
        if org.is_personal {
            return Err(DatabaseError::QueryError(
                "Cannot delete the personal organization".to_string(),
            ));
        }

        let delete_sql = "DELETE FROM organizations WHERE id = $1::uuid";

        let mut conn = self.pool.acquire().await?;
        let result = query(delete_sql)
            .bind(id)
            .execute(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(DatabaseError::not_found("organization", id));
        }

        info!("Organization deleted: {}", id);
        Ok(())
    }

    #[instrument(skip(self))]
    pub async fn count_for_user(
        &self,
        user_id: &str,
        include_personal: bool,
    ) -> DatabaseResult<i64> {
        let (count_sql, _) = if include_personal {
            (
                "SELECT COUNT(*) as count FROM organizations WHERE owner_id = $1::uuid OR id IN (SELECT organization_id FROM organization_members WHERE user_id = $1::uuid)",
                true,
            )
        } else {
            (
                "SELECT COUNT(*) as count FROM organizations WHERE (owner_id = $1::uuid OR id IN (SELECT organization_id FROM organization_members WHERE user_id = $1::uuid)) AND is_personal = false",
                false,
            )
        };

        let mut conn = self.pool.acquire().await?;
        let row = query(count_sql)
            .bind(user_id)
            .fetch_one(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        Ok(row.get("count"))
    }

    // -- Member management --

    #[instrument(skip(self, req))]
    pub async fn add_member(
        &self,
        org_id: &str,
        req: AddOrganizationMemberRequest,
    ) -> DatabaseResult<OrganizationMember> {
        let id = uuid::Uuid::new_v4().to_string();
        let role = req.role.unwrap_or_else(|| "viewer".to_string());

        let insert_sql = r#"
            INSERT INTO organization_members (id, organization_id, user_id, role)
            VALUES ($1::uuid, $2::uuid, $3::uuid, $4)
            RETURNING id::text as id, organization_id::text as organization_id, user_id::text as user_id,
                role, joined_at, invited_by::text as invited_by
        "#;

        let mut conn = self.pool.acquire().await?;
        query_as::<_, OrganizationMember>(insert_sql)
            .bind(&id)
            .bind(org_id)
            .bind(&req.user_id)
            .bind(&role)
            .fetch_optional(&mut *conn)
            .await
            .map_err(|e| {
                if e.to_string().contains("duplicate key")
                    || e.to_string().contains("UNIQUE constraint")
                {
                    DatabaseError::duplicate(
                        "organization_member",
                        "User is already a member of this organization",
                    )
                } else {
                    DatabaseError::QueryError(e.to_string())
                }
            })?;

        // Re-fetch with user info
        let select_sql = format!("{} WHERE om.id = $1::uuid", ORG_MEMBER_SELECT_SQL);
        let member: OrganizationMember = query_as(&select_sql)
            .bind(&id)
            .fetch_one(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        info!("Added member {} to organization {}", req.user_id, org_id);
        Ok(member)
    }

    #[instrument(skip(self))]
    pub async fn list_members(
        &self,
        org_id: &str,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> DatabaseResult<Vec<OrganizationMember>> {
        let limit = limit.unwrap_or(100);
        let offset = offset.unwrap_or(0);
        let select_sql = format!(
            "{} WHERE om.organization_id = $1::uuid ORDER BY om.role, om.joined_at LIMIT $2 OFFSET $3",
            ORG_MEMBER_SELECT_SQL
        );

        let mut conn = self.pool.acquire().await?;
        let members: Vec<OrganizationMember> = query_as(&select_sql)
            .bind(org_id)
            .bind(limit)
            .bind(offset)
            .fetch_all(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        Ok(members)
    }

    #[instrument(skip(self, req))]
    pub async fn update_member(
        &self,
        org_id: &str,
        user_id: &str,
        req: UpdateOrganizationMemberRequest,
    ) -> DatabaseResult<OrganizationMember> {
        let update_sql = r#"
            UPDATE organization_members SET role = $1
            WHERE organization_id = $2::uuid AND user_id = $3::uuid
        "#;

        let mut conn = self.pool.acquire().await?;
        let result = query(update_sql)
            .bind(&req.role)
            .bind(org_id)
            .bind(user_id)
            .execute(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(DatabaseError::not_found("organization_member", user_id));
        }

        // Re-fetch with user info
        let select_sql = format!(
            "{} WHERE om.organization_id = $1::uuid AND om.user_id = $2::uuid",
            ORG_MEMBER_SELECT_SQL
        );
        let member: OrganizationMember = query_as(&select_sql)
            .bind(org_id)
            .bind(user_id)
            .fetch_one(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        info!(
            "Updated member {} role to {} in org {}",
            user_id, req.role, org_id
        );
        Ok(member)
    }

    #[instrument(skip(self))]
    pub async fn remove_member(&self, org_id: &str, user_id: &str) -> DatabaseResult<()> {
        let delete_sql = "DELETE FROM organization_members WHERE organization_id = $1::uuid AND user_id = $2::uuid";

        let mut conn = self.pool.acquire().await?;
        let result = query(delete_sql)
            .bind(org_id)
            .bind(user_id)
            .execute(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(DatabaseError::not_found("organization_member", user_id));
        }

        info!("Removed member {} from organization {}", user_id, org_id);
        Ok(())
    }

    /// Check if a user is a member of an organization (or is the owner)
    #[instrument(skip(self))]
    pub async fn is_member(&self, org_id: &str, user_id: &str) -> DatabaseResult<bool> {
        let check_sql = r#"
            SELECT EXISTS(
                SELECT 1 FROM organizations WHERE id = $1::uuid AND owner_id = $2::uuid
                UNION
                SELECT 1 FROM organization_members WHERE organization_id = $1::uuid AND user_id = $2::uuid
            ) as is_member
        "#;

        let mut conn = self.pool.acquire().await?;
        let row = query(check_sql)
            .bind(org_id)
            .bind(user_id)
            .fetch_one(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        Ok(row.get("is_member"))
    }

    /// Get the role of a user in an organization
    #[instrument(skip(self))]
    pub async fn get_member_role(
        &self,
        org_id: &str,
        user_id: &str,
    ) -> DatabaseResult<Option<String>> {
        // Owner check first
        let org = self.get_by_id(org_id).await?;
        if org.owner_id == user_id {
            return Ok(Some("owner".to_string()));
        }

        let role_sql = r#"
            SELECT role FROM organization_members
            WHERE organization_id = $1::uuid AND user_id = $2::uuid
        "#;

        let mut conn = self.pool.acquire().await?;
        let row: Option<(String,)> = query_as(role_sql)
            .bind(org_id)
            .bind(user_id)
            .fetch_optional(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        Ok(row.map(|r| r.0))
    }

    /// Count members in an organization
    #[instrument(skip(self))]
    pub async fn count_members(&self, org_id: &str) -> DatabaseResult<i64> {
        let count_sql = r#"
            SELECT COUNT(*) as count FROM organization_members WHERE organization_id = $1::uuid
        "#;

        let mut conn = self.pool.acquire().await?;
        let row = query(count_sql)
            .bind(org_id)
            .fetch_one(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        Ok(row.get("count"))
    }

    /// Count members for multiple organizations in a single query
    #[instrument(skip(self))]
    pub async fn count_members_batch(
        &self,
        org_ids: &[String],
    ) -> DatabaseResult<HashMap<String, i64>> {
        if org_ids.is_empty() {
            return Ok(HashMap::new());
        }

        let count_sql = r#"
            SELECT organization_id::text as id, COUNT(*) as count
            FROM organization_members
            WHERE organization_id = ANY($1::uuid[])
            GROUP BY organization_id
        "#;

        let mut conn = self.pool.acquire().await?;
        let rows: Vec<(String, i64)> = query_as(count_sql)
            .bind(org_ids)
            .fetch_all(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        Ok(rows.into_iter().collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_organization_request_defaults() {
        let req = CreateOrganizationRequest {
            name: "Test Org".to_string(),
            description: None,
            icon: None,
            logo_url: None,
            default_role: None,
            max_members: None,
        };
        assert_eq!(req.name, "Test Org");
        assert!(req.description.is_none());
        assert!(req.icon.is_none());
        assert!(req.logo_url.is_none());
        assert!(req.default_role.is_none());
        assert!(req.max_members.is_none());
    }

    #[test]
    fn test_create_organization_request_with_all_fields() {
        let req = CreateOrganizationRequest {
            name: "Acme Corp".to_string(),
            description: Some("A company".to_string()),
            icon: Some("building".to_string()),
            logo_url: Some("https://example.com/logo.png".to_string()),
            default_role: Some("admin".to_string()),
            max_members: Some(100),
        };
        assert_eq!(req.name, "Acme Corp");
        assert_eq!(req.description.as_deref(), Some("A company"));
        assert_eq!(req.icon.as_deref(), Some("building"));
        assert_eq!(req.default_role.as_deref(), Some("admin"));
        assert_eq!(req.max_members, Some(100));
    }

    #[test]
    fn test_update_organization_request_all_none() {
        let req = UpdateOrganizationRequest {
            name: None,
            description: None,
            icon: None,
            logo_url: None,
            default_role: None,
            max_members: None,
            settings: None,
        };
        assert!(req.name.is_none());
        assert!(req.description.is_none());
        assert!(req.icon.is_none());
        assert!(req.logo_url.is_none());
        assert!(req.default_role.is_none());
        assert!(req.max_members.is_none());
        assert!(req.settings.is_none());
    }

    #[test]
    fn test_update_organization_request_with_settings() {
        let settings = serde_json::json!({"theme": "dark", "notifications": true});
        let req = UpdateOrganizationRequest {
            name: Some("Updated Org".to_string()),
            description: Some("Updated desc".to_string()),
            icon: None,
            logo_url: None,
            default_role: Some("editor".to_string()),
            max_members: Some(50),
            settings: Some(settings),
        };
        assert_eq!(req.name.as_deref(), Some("Updated Org"));
        assert_eq!(req.default_role.as_deref(), Some("editor"));
        assert!(req.settings.is_some());
    }

    #[test]
    fn test_add_member_request_defaults() {
        let req = AddOrganizationMemberRequest {
            user_id: "user-123".to_string(),
            role: None,
        };
        assert_eq!(req.user_id, "user-123");
        assert!(req.role.is_none());
    }

    #[test]
    fn test_add_member_request_with_role() {
        let req = AddOrganizationMemberRequest {
            user_id: "user-456".to_string(),
            role: Some("admin".to_string()),
        };
        assert_eq!(req.user_id, "user-456");
        assert_eq!(req.role.as_deref(), Some("admin"));
    }

    #[test]
    fn test_update_member_request() {
        let req = UpdateOrganizationMemberRequest {
            role: "viewer".to_string(),
        };
        assert_eq!(req.role, "viewer");
    }

    #[test]
    fn test_organization_parse_settings_valid() {
        let org = Organization {
            id: "1".to_string(),
            name: "Test".to_string(),
            slug: "test".to_string(),
            description: None,
            icon: "building".to_string(),
            logo_url: None,
            owner_id: "owner-1".to_string(),
            default_role: "viewer".to_string(),
            max_members: 100,
            is_personal: false,
            settings: r#"{"theme": "dark"}"#.to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        let settings = org.parse_settings().unwrap();
        assert_eq!(settings["theme"], "dark");
    }

    #[test]
    fn test_organization_parse_settings_invalid() {
        let org = Organization {
            id: "1".to_string(),
            name: "Test".to_string(),
            slug: "test".to_string(),
            description: None,
            icon: "building".to_string(),
            logo_url: None,
            owner_id: "owner-1".to_string(),
            default_role: "viewer".to_string(),
            max_members: 100,
            is_personal: false,
            settings: "not-json".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        assert!(org.parse_settings().is_err());
    }

    #[test]
    fn test_organization_parse_settings_empty() {
        let org = Organization {
            id: "1".to_string(),
            name: "Test".to_string(),
            slug: "test".to_string(),
            description: None,
            icon: "building".to_string(),
            logo_url: None,
            owner_id: "owner-1".to_string(),
            default_role: "viewer".to_string(),
            max_members: 100,
            is_personal: false,
            settings: "{}".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        let settings = org.parse_settings().unwrap();
        assert_eq!(settings.as_object().unwrap().len(), 0);
    }
}
