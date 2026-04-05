// Team Management
// Team and team member data structures and repository

use crate::error::{DatabaseError, DatabaseResult};
use crate::schema::DatabasePool;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{query, query_as, FromRow};

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Team {
    pub id: String,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub owner_id: String,
    pub avatar_url: Option<String>,
    pub settings: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Team {
    pub fn new(name: String, slug: String, owner_id: String) -> Self {
        let now = Utc::now();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            slug,
            description: None,
            owner_id,
            avatar_url: None,
            settings: serde_json::json!({}),
            created_at: now,
            updated_at: now,
        }
    }

    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct TeamMember {
    pub id: i64,
    pub team_id: String,
    pub user_id: String,
    pub role_id: i64,
    pub role_name: String,
    pub joined_at: DateTime<Utc>,
    pub invited_by: Option<String>,
}

impl TeamMember {
    pub fn new(team_id: String, user_id: String, role_id: i64, role_name: String) -> Self {
        Self {
            id: 0,
            team_id,
            user_id,
            role_id,
            role_name,
            joined_at: Utc::now(),
            invited_by: None,
        }
    }

    pub fn with_inviter(mut self, invited_by: String) -> Self {
        self.invited_by = Some(invited_by);
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct RoleRecord {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
    pub permissions: serde_json::Value,
    pub is_system: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl RoleRecord {
    pub fn new(name: String, permissions: serde_json::Value) -> Self {
        let now = Utc::now();
        Self {
            id: 0,
            name,
            description: None,
            permissions,
            is_system: false,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }

    pub fn system(mut self) -> Self {
        self.is_system = true;
        self
    }

    pub fn parse_permissions(&self) -> DatabaseResult<Vec<String>> {
        serde_json::from_value(self.permissions.clone())
            .map_err(|e| DatabaseError::SerializationError(e.to_string()))
    }
}

#[derive(Clone)]
pub struct TeamRepository {
    pool: DatabasePool,
}

impl TeamRepository {
    pub fn new(pool: DatabasePool) -> Self {
        Self { pool }
    }

    pub async fn create(&self, team: &Team) -> DatabaseResult<Team> {
        let insert_sql = r#"
            INSERT INTO teams (id, name, slug, description, owner_id, avatar_url, settings, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING *
        "#;

        let mut conn = self.pool.acquire().await?;
        query_as::<_, Team>(insert_sql)
            .bind(&team.id)
            .bind(&team.name)
            .bind(&team.slug)
            .bind(&team.description)
            .bind(&team.owner_id)
            .bind(&team.avatar_url)
            .bind(&team.settings)
            .bind(&team.created_at)
            .bind(&team.updated_at)
            .fetch_one(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))
    }

    pub async fn get_by_id(&self, id: &str) -> DatabaseResult<Team> {
        let select_sql = "SELECT * FROM teams WHERE id = $1";

        let mut conn = self.pool.acquire().await?;
        query_as::<_, Team>(select_sql)
            .bind(id)
            .fetch_optional(&mut *conn)
            .await?
            .ok_or_else(|| DatabaseError::not_found("team", id))
    }

    pub async fn get_by_slug(&self, slug: &str) -> DatabaseResult<Team> {
        let select_sql = "SELECT * FROM teams WHERE slug = $1";

        let mut conn = self.pool.acquire().await?;
        query_as::<_, Team>(select_sql)
            .bind(slug)
            .fetch_optional(&mut *conn)
            .await?
            .ok_or_else(|| DatabaseError::not_found("team", slug))
    }

    pub async fn list_by_owner(&self, owner_id: &str) -> DatabaseResult<Vec<Team>> {
        let select_sql = "SELECT * FROM teams WHERE owner_id = $1 ORDER BY created_at DESC";

        let mut conn = self.pool.acquire().await?;
        query_as::<_, Team>(select_sql)
            .bind(owner_id)
            .fetch_all(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))
    }

    pub async fn list_for_user(&self, user_id: &str) -> DatabaseResult<Vec<Team>> {
        let select_sql = r#"
            SELECT t.* FROM teams t
            INNER JOIN team_members tm ON t.id = tm.team_id
            WHERE tm.user_id = $1
            ORDER BY t.created_at DESC
        "#;

        let mut conn = self.pool.acquire().await?;
        query_as::<_, Team>(select_sql)
            .bind(user_id)
            .fetch_all(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))
    }

    pub async fn update(&self, team: &Team) -> DatabaseResult<Team> {
        let update_sql = r#"
            UPDATE teams SET
                name = $2, slug = $3, description = $4, avatar_url = $5, settings = $6, updated_at = $7
            WHERE id = $1
            RETURNING *
        "#;

        let mut conn = self.pool.acquire().await?;
        query_as::<_, Team>(update_sql)
            .bind(&team.id)
            .bind(&team.name)
            .bind(&team.slug)
            .bind(&team.description)
            .bind(&team.avatar_url)
            .bind(&team.settings)
            .bind(&team.updated_at)
            .fetch_optional(&mut *conn)
            .await?
            .ok_or_else(|| DatabaseError::not_found("team", &team.id))
    }

    pub async fn delete(&self, id: &str) -> DatabaseResult<()> {
        let delete_sql = "DELETE FROM teams WHERE id = $1";

        let mut conn = self.pool.acquire().await?;
        let result = query(delete_sql)
            .bind(id)
            .execute(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(DatabaseError::not_found("team", id));
        }

        Ok(())
    }

    pub async fn add_member(&self, member: &TeamMember) -> DatabaseResult<TeamMember> {
        let insert_sql = r#"
            INSERT INTO team_members (team_id, user_id, role_id, role_name, joined_at, invited_by)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id, team_id, user_id, role_id, role_name, joined_at, invited_by
        "#;

        let mut conn = self.pool.acquire().await?;
        query_as::<_, TeamMember>(insert_sql)
            .bind(&member.team_id)
            .bind(&member.user_id)
            .bind(&member.role_id)
            .bind(&member.role_name)
            .bind(&member.joined_at)
            .bind(&member.invited_by)
            .fetch_one(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))
    }

    pub async fn get_member(&self, team_id: &str, user_id: &str) -> DatabaseResult<TeamMember> {
        let select_sql = "SELECT * FROM team_members WHERE team_id = $1 AND user_id = $2";

        let mut conn = self.pool.acquire().await?;
        query_as::<_, TeamMember>(select_sql)
            .bind(team_id)
            .bind(user_id)
            .fetch_optional(&mut *conn)
            .await?
            .ok_or_else(|| DatabaseError::not_found("team_member", format!("{}:{}", team_id, user_id)))
    }

    pub async fn list_members(&self, team_id: &str) -> DatabaseResult<Vec<TeamMember>> {
        let select_sql = "SELECT * FROM team_members WHERE team_id = $1 ORDER BY joined_at ASC";

        let mut conn = self.pool.acquire().await?;
        query_as::<_, TeamMember>(select_sql)
            .bind(team_id)
            .fetch_all(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))
    }

    pub async fn update_member_role(&self, team_id: &str, user_id: &str, role_id: i64, role_name: &str) -> DatabaseResult<()> {
        let update_sql = "UPDATE team_members SET role_id = $3, role_name = $4 WHERE team_id = $1 AND user_id = $2";

        let mut conn = self.pool.acquire().await?;
        let result = query(update_sql)
            .bind(team_id)
            .bind(user_id)
            .bind(role_id)
            .bind(role_name)
            .execute(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(DatabaseError::not_found("team_member", format!("{}:{}", team_id, user_id)));
        }

        Ok(())
    }

    pub async fn remove_member(&self, team_id: &str, user_id: &str) -> DatabaseResult<()> {
        let delete_sql = "DELETE FROM team_members WHERE team_id = $1 AND user_id = $2";

        let mut conn = self.pool.acquire().await?;
        let result = query(delete_sql)
            .bind(team_id)
            .bind(user_id)
            .execute(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(DatabaseError::not_found("team_member", format!("{}:{}", team_id, user_id)));
        }

        Ok(())
    }

    pub async fn is_member(&self, team_id: &str, user_id: &str) -> DatabaseResult<bool> {
        let select_sql = "SELECT 1 FROM team_members WHERE team_id = $1 AND user_id = $2";

        let mut conn = self.pool.acquire().await?;
        let result = query(select_sql)
            .bind(team_id)
            .bind(user_id)
            .fetch_optional(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        Ok(result.is_some())
    }
}

#[derive(Clone)]
pub struct RoleRepository {
    pool: DatabasePool,
}

impl RoleRepository {
    pub fn new(pool: DatabasePool) -> Self {
        Self { pool }
    }

    pub async fn create(&self, role: &RoleRecord) -> DatabaseResult<RoleRecord> {
        let insert_sql = r#"
            INSERT INTO roles (name, description, permissions, is_system, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING *
        "#;

        let mut conn = self.pool.acquire().await?;
        query_as::<_, RoleRecord>(insert_sql)
            .bind(&role.name)
            .bind(&role.description)
            .bind(&role.permissions)
            .bind(role.is_system)
            .bind(&role.created_at)
            .bind(&role.updated_at)
            .fetch_one(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))
    }

    pub async fn get_by_id(&self, id: i64) -> DatabaseResult<RoleRecord> {
        let select_sql = "SELECT * FROM roles WHERE id = $1";

        let mut conn = self.pool.acquire().await?;
        query_as::<_, RoleRecord>(select_sql)
            .bind(id)
            .fetch_optional(&mut *conn)
            .await?
            .ok_or_else(|| DatabaseError::not_found("role", id.to_string()))
    }

    pub async fn get_by_name(&self, name: &str) -> DatabaseResult<RoleRecord> {
        let select_sql = "SELECT * FROM roles WHERE name = $1";

        let mut conn = self.pool.acquire().await?;
        query_as::<_, RoleRecord>(select_sql)
            .bind(name)
            .fetch_optional(&mut *conn)
            .await?
            .ok_or_else(|| DatabaseError::not_found("role", name))
    }

    pub async fn list_all(&self) -> DatabaseResult<Vec<RoleRecord>> {
        let select_sql = "SELECT * FROM roles ORDER BY name ASC";

        let mut conn = self.pool.acquire().await?;
        query_as::<_, RoleRecord>(select_sql)
            .fetch_all(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))
    }

    pub async fn update(&self, role: &RoleRecord) -> DatabaseResult<RoleRecord> {
        let update_sql = r#"
            UPDATE roles SET
                name = $2, description = $3, permissions = $4, updated_at = $5
            WHERE id = $1 AND is_system = false
            RETURNING *
        "#;

        let mut conn = self.pool.acquire().await?;
        query_as::<_, RoleRecord>(update_sql)
            .bind(role.id)
            .bind(&role.name)
            .bind(&role.description)
            .bind(&role.permissions)
            .bind(&role.updated_at)
            .fetch_optional(&mut *conn)
            .await?
            .ok_or_else(|| DatabaseError::ValidationError("Cannot update system role".to_string()))
    }

    pub async fn delete(&self, id: i64) -> DatabaseResult<()> {
        let delete_sql = "DELETE FROM roles WHERE id = $1 AND is_system = false";

        let mut conn = self.pool.acquire().await?;
        let result = query(delete_sql)
            .bind(id)
            .execute(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(DatabaseError::ValidationError("Cannot delete system role".to_string()));
        }

        Ok(())
    }

    pub async fn seed_default_roles(&self) -> DatabaseResult<()> {
        let roles = vec![
            RoleRecord::new("owner".to_string(), serde_json::json!(["owner"]))
                .with_description("Full ownership with all permissions".to_string())
                .system(),
            RoleRecord::new("admin".to_string(), serde_json::json!(["admin", "delete", "write", "read"]))
                .with_description("Full administrative access".to_string())
                .system(),
            RoleRecord::new("editor".to_string(), serde_json::json!(["delete", "write", "read"]))
                .with_description("Can read, write, and delete content".to_string())
                .system(),
            RoleRecord::new("writer".to_string(), serde_json::json!(["write", "read"]))
                .with_description("Can read and write content".to_string())
                .system(),
            RoleRecord::new("reader".to_string(), serde_json::json!(["read"]))
                .with_description("Can only read content".to_string())
                .system(),
        ];

        for role in roles {
            if self.get_by_name(&role.name).await.is_err() {
                self.create(&role).await?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_team_creation() {
        let team = Team::new("Engineering".to_string(), "engineering".to_string(), "user-123".to_string());
        assert_eq!(team.name, "Engineering");
        assert_eq!(team.slug, "engineering");
        assert_eq!(team.owner_id, "user-123");
    }

    #[test]
    fn test_team_member_creation() {
        let member = TeamMember::new("team-1".to_string(), "user-1".to_string(), 1, "admin".to_string());
        assert_eq!(member.team_id, "team-1");
        assert_eq!(member.user_id, "user-1");
        assert_eq!(member.role_name, "admin");
    }

    #[test]
    fn test_role_record_creation() {
        let role = RoleRecord::new("custom".to_string(), serde_json::json!(["read", "write"]));
        assert_eq!(role.name, "custom");
        assert!(!role.is_system);
    }
}
