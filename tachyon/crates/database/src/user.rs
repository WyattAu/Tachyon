// User Persistence
// User storage and management for PostgreSQL

use crate::error::{DatabaseError, DatabaseResult};
use crate::schema::DatabasePool;
use chrono::{DateTime, Utc};
use sqlx::{query, query_as, Row};
use tachyon_core::id::UserId;
use tachyon_core::types::user::{User, UserRole, UserType};
use tracing::{debug, info, instrument, warn};

/// Database row model for the `users` table.
///
/// This maps 1:1 to the PostgreSQL `users` schema. Conversion to/from
/// `tachyon_core::types::user::User` is provided via `From` impls.
#[derive(Debug, Clone, sqlx::FromRow, utoipa::ToSchema)]
pub struct UserRecord {
    /// Primary key (UUID v7).
    pub id: uuid::Uuid,
    /// Unique login handle.
    pub username: String,
    /// Human-readable name shown in the UI.
    pub display_name: Option<String>,
    /// Verified email address (unique when present).
    pub email: Option<String>,
    /// Argon2id password hash.
    pub password_hash: String,
    /// Role stored as TEXT: `admin`, `editor`, `writer`, `reader`.
    pub role: String,
    /// Account classification stored as TEXT: `regular`, `service`, `system`.
    pub user_type: String,
    /// Whether the account is currently active.
    pub is_active: bool,
    /// Row-creation timestamp.
    pub created_at: DateTime<Utc>,
    /// Last-update timestamp.
    pub updated_at: DateTime<Utc>,
    /// Base32-encoded TOTP secret (present when 2FA is configured).
    pub totp_secret: Option<String>,
    /// Whether time-based one-time password 2FA is enabled.
    pub totp_enabled: bool,
    /// Single-use recovery codes for TOTP.
    pub totp_backup_codes: Option<Vec<String>>,
    /// When the user last verified their TOTP setup.
    pub totp_verified_at: Option<DateTime<Utc>>,
}

impl UserRecord {
    /// Parse the `role` TEXT column into a `UserRole`.
    fn parse_role(role: &str) -> UserRole {
        match role {
            "admin" => UserRole::Admin,
            "editor" => UserRole::Editor,
            "writer" => UserRole::Writer,
            _ => UserRole::Reader,
        }
    }

    /// Parse the `user_type` TEXT column into a `UserType`.
    fn parse_user_type(user_type: &str) -> UserType {
        match user_type {
            "service" => UserType::Service,
            "system" => UserType::System,
            _ => UserType::Regular,
        }
    }
}

impl From<UserRecord> for User {
    fn from(r: UserRecord) -> Self {
        let id = UserId::from_uuid(r.id);
        let mut user = User::new(
            id,
            r.username,
            r.display_name.unwrap_or_default(),
            UserRecord::parse_role(&r.role),
        );
        user.email = r.email;
        user.user_type = UserRecord::parse_user_type(&r.user_type);
        user.is_active = Some(r.is_active);
        user.password_hash = Some(r.password_hash);
        user.created_at = r.created_at;
        user.updated_at = r.updated_at;
        user
    }
}

/// User repository for persistence operations.
pub struct UserRepository {
    pool: DatabasePool,
}

impl UserRepository {
    /// Create a new user repository.
    pub fn new(pool: DatabasePool) -> Self {
        Self { pool }
    }

    // ── Create ──────────────────────────────────────────────────────

    /// Insert a new user into the database.
    ///
    /// Returns `DatabaseError::Duplicate` if the username or email already exists.
    #[instrument(skip(self, user), fields(username = %user.username))]
    pub async fn create(&self, user: &User) -> DatabaseResult<User> {
        let sql = r#"
            INSERT INTO users (id, username, display_name, email, password_hash, role, user_type, is_active, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            RETURNING *
        "#;

        let password_hash = user
            .password_hash
            .as_deref()
            .ok_or_else(|| DatabaseError::ValidationError("password_hash is required".into()))?;

        let mut conn = self.pool.acquire().await?;
        let record = query_as::<_, UserRecord>(sql)
            .bind(user.id.as_uuid())
            .bind(&user.username)
            .bind(&user.display_name)
            .bind(&user.email)
            .bind(password_hash)
            .bind(user.permissions.role.to_string())
            .bind(user.user_type.to_string())
            .bind(user.is_active.unwrap_or(true))
            .bind(user.created_at)
            .bind(user.updated_at)
            .fetch_one(&mut *conn)
            .await
            .map_err(|e| {
                let msg = e.to_string();
                if msg.contains("unique")
                    || msg.contains("duplicate")
                    || msg.contains("users_username_key")
                    || msg.contains("users_email_key")
                {
                    if msg.contains("email") {
                        DatabaseError::duplicate(
                            "user",
                            format!(
                                "Email already exists: {}",
                                user.email.as_deref().unwrap_or("")
                            ),
                        )
                    } else {
                        DatabaseError::duplicate(
                            "user",
                            format!("Username already exists: {}", user.username),
                        )
                    }
                } else {
                    DatabaseError::QueryError(msg)
                }
            })?;

        info!("User created: {} ({})", record.username, record.id);
        Ok(User::from(record))
    }

    // ── Read ────────────────────────────────────────────────────────

    /// Get a user by ID.
    #[instrument(skip(self))]
    pub async fn get_by_id(&self, id: &UserId) -> DatabaseResult<User> {
        let sql = "SELECT * FROM users WHERE id = $1";
        let mut conn = self.pool.acquire().await?;
        let record = query_as::<_, UserRecord>(sql)
            .bind(id.as_uuid())
            .fetch_optional(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?
            .ok_or_else(|| DatabaseError::not_found("user", id.as_str()))?;
        Ok(User::from(record))
    }

    /// Get a user by username.
    #[instrument(skip(self), fields(username = %username))]
    pub async fn get_by_username(&self, username: &str) -> DatabaseResult<User> {
        let sql = "SELECT * FROM users WHERE username = $1";
        let mut conn = self.pool.acquire().await?;
        let record = query_as::<_, UserRecord>(sql)
            .bind(username)
            .fetch_optional(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?
            .ok_or_else(|| DatabaseError::not_found("user", username))?;
        Ok(User::from(record))
    }

    /// Get a user by email.
    #[instrument(skip(self), fields(email = %email))]
    pub async fn get_by_email(&self, email: &str) -> DatabaseResult<User> {
        let sql = "SELECT * FROM users WHERE email = $1";
        let mut conn = self.pool.acquire().await?;
        let record = query_as::<_, UserRecord>(sql)
            .bind(email)
            .fetch_optional(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?
            .ok_or_else(|| DatabaseError::not_found("user", email))?;
        Ok(User::from(record))
    }

    /// Find a user by username or email (for login lookups).
    #[instrument(skip(self))]
    pub async fn find_by_username_or_email(&self, identifier: &str) -> DatabaseResult<User> {
        let sql = "SELECT * FROM users WHERE username = $1 OR email = $1";
        let mut conn = self.pool.acquire().await?;
        let record = query_as::<_, UserRecord>(sql)
            .bind(identifier)
            .fetch_optional(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?
            .ok_or_else(|| DatabaseError::not_found("user", identifier))?;
        Ok(User::from(record))
    }

    /// Get a user by phone number.
    #[instrument(skip(self), fields(phone = %phone))]
    pub async fn get_by_phone(&self, phone: &str) -> DatabaseResult<User> {
        let sql = "SELECT * FROM users WHERE phone = $1";
        let mut conn = self.pool.acquire().await?;
        let record = query_as::<_, UserRecord>(sql)
            .bind(phone)
            .fetch_optional(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?
            .ok_or_else(|| DatabaseError::not_found("user", phone))?;
        Ok(User::from(record))
    }

    // ── Update ──────────────────────────────────────────────────────

    /// Update a user's profile fields. Password is updated separately via `update_password`.
    #[instrument(skip(self), fields(user_id = %user_id))]
    pub async fn update(
        &self,
        user_id: &UserId,
        display_name: Option<&str>,
        email: Option<&str>,
        role: Option<UserRole>,
        is_active: Option<bool>,
    ) -> DatabaseResult<User> {
        let sql = r#"
            UPDATE users SET
                display_name = COALESCE($2, display_name),
                email = COALESCE($3, email),
                role = COALESCE($4, role),
                is_active = COALESCE($5, is_active),
                updated_at = NOW()
            WHERE id = $1
            RETURNING *
        "#;

        let mut conn = self.pool.acquire().await?;
        let record = query_as::<_, UserRecord>(sql)
            .bind(user_id.as_uuid())
            .bind(display_name)
            .bind(email)
            .bind(role.map(|r| r.to_string()))
            .bind(is_active)
            .fetch_optional(&mut *conn)
            .await
            .map_err(|e| {
                let msg = e.to_string();
                if msg.contains("unique") || msg.contains("duplicate") {
                    DatabaseError::duplicate("user", "Email already in use")
                } else {
                    DatabaseError::QueryError(msg)
                }
            })?
            .ok_or_else(|| DatabaseError::not_found("user", user_id.as_str()))?;

        info!("User updated: {}", user_id.as_str());
        Ok(User::from(record))
    }

    /// Update a user's password hash.
    #[instrument(skip(self, password_hash))]
    pub async fn update_password(
        &self,
        user_id: &UserId,
        password_hash: &str,
    ) -> DatabaseResult<()> {
        let sql = "UPDATE users SET password_hash = $2, updated_at = NOW() WHERE id = $1";
        let mut conn = self.pool.acquire().await?;
        let result = query(sql)
            .bind(user_id.as_uuid())
            .bind(password_hash)
            .execute(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(DatabaseError::not_found("user", user_id.as_str()));
        }
        info!("Password updated for user: {}", user_id.as_str());
        Ok(())
    }

    // ── Delete ──────────────────────────────────────────────────────

    /// Soft-delete a user by setting `is_active = false`.
    #[instrument(skip(self))]
    pub async fn deactivate(&self, user_id: &UserId) -> DatabaseResult<()> {
        let sql = "UPDATE users SET is_active = false, updated_at = NOW() WHERE id = $1";
        let mut conn = self.pool.acquire().await?;
        let result = query(sql)
            .bind(user_id.as_uuid())
            .execute(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(DatabaseError::not_found("user", user_id.as_str()));
        }
        info!("User deactivated: {}", user_id.as_str());
        Ok(())
    }

    /// Hard-delete a user from the database.
    ///
    /// **Warning:** This will cascade-delete all user-dependent records
    /// (sessions, saved searches, user_roles, etc.) per FK constraints.
    #[instrument(skip(self))]
    pub async fn delete(&self, user_id: &UserId) -> DatabaseResult<()> {
        let sql = "DELETE FROM users WHERE id = $1 CASCADE";
        let mut conn = self.pool.acquire().await?;
        let result = query(sql)
            .bind(user_id.as_uuid())
            .execute(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(DatabaseError::not_found("user", user_id.as_str()));
        }
        warn!("User hard-deleted: {}", user_id.as_str());
        Ok(())
    }

    // ── List ────────────────────────────────────────────────────────

    /// List users with pagination and optional role filter.
    pub async fn list(
        &self,
        page: usize,
        page_size: usize,
        role_filter: Option<&str>,
    ) -> DatabaseResult<(Vec<User>, i64)> {
        let offset = ((page.max(1) - 1) * page_size.min(100)) as i64;
        let limit = page_size.min(100) as i64;

        let (records, total): (Vec<UserRecord>, i64) = if let Some(role) = role_filter {
            let count_sql = "SELECT COUNT(*) as count FROM users WHERE role = $1";
            let data_sql = r#"
                SELECT * FROM users WHERE role = $1
                ORDER BY created_at DESC
                LIMIT $2 OFFSET $3
            "#;

            let mut conn = self.pool.acquire().await?;
            let count_row = sqlx::query(count_sql)
                .bind(role)
                .fetch_one(&mut *conn)
                .await
                .map_err(|e| DatabaseError::QueryError(e.to_string()))?;
            let total: i64 = count_row.get("count");

            let records = query_as::<_, UserRecord>(data_sql)
                .bind(role)
                .bind(limit)
                .bind(offset)
                .fetch_all(&mut *conn)
                .await
                .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

            (records, total)
        } else {
            let count_sql = "SELECT COUNT(*) as count FROM users";
            let data_sql = r#"
                SELECT * FROM users
                ORDER BY created_at DESC
                LIMIT $1 OFFSET $2
            "#;

            let mut conn = self.pool.acquire().await?;
            let count_row = sqlx::query(count_sql)
                .fetch_one(&mut *conn)
                .await
                .map_err(|e| DatabaseError::QueryError(e.to_string()))?;
            let total: i64 = count_row.get("count");

            let records = query_as::<_, UserRecord>(data_sql)
                .bind(limit)
                .bind(offset)
                .fetch_all(&mut *conn)
                .await
                .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

            (records, total)
        };

        let users = records.into_iter().map(User::from).collect();
        Ok((users, total))
    }

    // ── Existence checks ────────────────────────────────────────────

    /// Check if a user exists by ID.
    pub async fn exists(&self, id: &UserId) -> DatabaseResult<bool> {
        let sql = "SELECT EXISTS(SELECT 1 FROM users WHERE id = $1)";
        let mut conn = self.pool.acquire().await?;
        let row = sqlx::query(sql)
            .bind(id.as_uuid())
            .fetch_one(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;
        let exists: bool = row.get("exists");
        Ok(exists)
    }

    /// Check if a username is already taken.
    pub async fn username_exists(&self, username: &str) -> DatabaseResult<bool> {
        let sql = "SELECT EXISTS(SELECT 1 FROM users WHERE username = $1)";
        let mut conn = self.pool.acquire().await?;
        let row = sqlx::query(sql)
            .bind(username)
            .fetch_one(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;
        let exists: bool = row.get("exists");
        Ok(exists)
    }

    /// Check if an email is already taken.
    pub async fn email_exists(&self, email: &str) -> DatabaseResult<bool> {
        let sql = "SELECT EXISTS(SELECT 1 FROM users WHERE email = $1)";
        let mut conn = self.pool.acquire().await?;
        let row = sqlx::query(sql)
            .bind(email)
            .fetch_one(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;
        let exists: bool = row.get("exists");
        Ok(exists)
    }

    /// Count total users.
    pub async fn count(&self) -> DatabaseResult<i64> {
        let sql = "SELECT COUNT(*) as count FROM users";
        let mut conn = self.pool.acquire().await?;
        let row = sqlx::query(sql)
            .fetch_one(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;
        let count: i64 = row.get("count");
        Ok(count)
    }

    /// Seed the initial admin user if no users exist.
    ///
    /// Returns the admin user if seeded, or `None` if users already exist.
    #[instrument(skip(self, password))]
    pub async fn seed_admin(
        &self,
        username: &str,
        display_name: &str,
        email: &str,
        password: &str,
    ) -> DatabaseResult<Option<User>> {
        if self.count().await? > 0 {
            debug!("Users already exist, skipping admin seed");
            return Ok(None);
        }

        info!("Seeding initial admin user: {}", username);

        let user_id = tachyon_core::generate_user_id();
        let mut user = User::new(
            user_id,
            username.to_string(),
            display_name.to_string(),
            UserRole::Admin,
        );
        user.email = Some(email.to_string());
        user.set_password(password)
            .map_err(|e| DatabaseError::ValidationError(e.to_string()))?;

        let created = self.create(&user).await?;
        Ok(Some(created))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_record_parse_role() {
        assert!(matches!(UserRecord::parse_role("admin"), UserRole::Admin));
        assert!(matches!(UserRecord::parse_role("editor"), UserRole::Editor));
        assert!(matches!(UserRecord::parse_role("writer"), UserRole::Writer));
        assert!(matches!(UserRecord::parse_role("reader"), UserRole::Reader));
        assert!(matches!(
            UserRecord::parse_role("unknown"),
            UserRole::Reader
        ));
    }

    #[test]
    fn test_user_record_parse_user_type() {
        assert!(matches!(
            UserRecord::parse_user_type("regular"),
            UserType::Regular
        ));
        assert!(matches!(
            UserRecord::parse_user_type("service"),
            UserType::Service
        ));
        assert!(matches!(
            UserRecord::parse_user_type("system"),
            UserType::System
        ));
        assert!(matches!(
            UserRecord::parse_user_type("unknown"),
            UserType::Regular
        ));
    }
}
