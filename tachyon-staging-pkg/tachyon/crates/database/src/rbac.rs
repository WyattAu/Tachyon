// RBAC Mappings Storage
// User-role, role-permission, and policy storage (PostgreSQL)

use crate::error::{DatabaseError, DatabaseResult};
use crate::schema::DatabasePool;
use crate::types::*;
use chrono::{DateTime, Utc};
use sqlx::{query, query_as};
use tachyon_core::id::UserId;
use tracing::{debug, info, instrument};

/// User-role mapping repository
pub struct UserRoleRepository {
    pool: DatabasePool,
}

impl UserRoleRepository {
    /// Create a new user-role repository
    pub fn new(pool: DatabasePool) -> Self {
        Self { pool }
    }

    /// Assign a role to a user
    #[instrument(skip(self))]
    pub async fn assign_role(&self, mapping: UserRoleMapping) -> DatabaseResult<()> {
        let insert_sql = r#"
            INSERT INTO user_roles (user_id, role, assigned_by, assigned_at, expires_at)
            VALUES ($1, $2, $3, $4, $5)
        "#;

        let mut conn = self.pool.acquire().await?;
        query(insert_sql)
            .bind(&mapping.user_id)
            .bind(&mapping.role)
            .bind(&mapping.assigned_by)
            .bind(mapping.assigned_at)
            .bind(mapping.expires_at)
            .execute(&mut *conn)
            .await
            .map_err(|e| {
                if e.to_string().contains("unique") || e.to_string().contains("duplicate") {
                    DatabaseError::duplicate(
                        "user_role",
                        format!("User {} already has role {}", mapping.user_id, mapping.role),
                    )
                } else {
                    DatabaseError::QueryError(e.to_string())
                }
            })?;

        info!(
            "Role '{}' assigned to user: {}",
            mapping.role, mapping.user_id
        );
        Ok(())
    }

    /// Get all roles for a user
    pub async fn get_user_roles(
        &self,
        user_id: &UserId,
        include_expired: bool,
    ) -> DatabaseResult<Vec<UserRoleMapping>> {
        let select_sql = if include_expired {
            "SELECT * FROM user_roles WHERE user_id = $1 ORDER BY assigned_at DESC"
        } else {
            "SELECT * FROM user_roles WHERE user_id = $1 AND (expires_at IS NULL OR expires_at > NOW()) ORDER BY assigned_at DESC"
        };

        let mut conn = self.pool.acquire().await?;
        let roles = query_as::<_, UserRoleMapping>(select_sql)
            .bind(user_id.as_str())
            .fetch_all(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        Ok(roles)
    }

    /// Check if a user has a specific role
    pub async fn has_role(
        &self,
        user_id: &UserId,
        role: &str,
    ) -> DatabaseResult<Option<UserRoleMapping>> {
        let select_sql = r#"
            SELECT * FROM user_roles
            WHERE user_id = $1 AND role = $2
            AND (expires_at IS NULL OR expires_at > NOW())
        "#;

        let mut conn = self.pool.acquire().await?;
        let result = query_as::<_, UserRoleMapping>(select_sql)
            .bind(user_id.as_str())
            .bind(role)
            .fetch_optional(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        Ok(result)
    }

    /// Remove a role from a user
    #[instrument(skip(self))]
    pub async fn remove_role(&self, user_id: &UserId, role: &str) -> DatabaseResult<()> {
        let delete_sql = "DELETE FROM user_roles WHERE user_id = $1 AND role = $2";

        let mut conn = self.pool.acquire().await?;
        let result = query(delete_sql)
            .bind(user_id.as_str())
            .bind(role)
            .execute(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(DatabaseError::not_found(
                "user_role",
                format!("{}:{}", user_id.as_str(), role),
            ));
        }

        info!("Role '{}' removed from user: {}", role, user_id.as_str());
        Ok(())
    }

    /// Remove all roles from a user
    #[instrument(skip(self))]
    pub async fn remove_all_roles(&self, user_id: &UserId) -> DatabaseResult<u64> {
        let delete_sql = "DELETE FROM user_roles WHERE user_id = $1";

        let mut conn = self.pool.acquire().await?;
        let result = query(delete_sql)
            .bind(user_id.as_str())
            .execute(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        info!(
            "Removed {} roles from user: {}",
            result.rows_affected(),
            user_id.as_str()
        );
        Ok(result.rows_affected())
    }

    /// Get all users with a specific role
    pub async fn get_users_with_role(&self, role: &str) -> DatabaseResult<Vec<UserRoleMapping>> {
        let select_sql = r#"
            SELECT * FROM user_roles
            WHERE role = $1
            AND (expires_at IS NULL OR expires_at > NOW())
            ORDER BY assigned_at DESC
        "#;

        let mut conn = self.pool.acquire().await?;
        let mappings = query_as::<_, UserRoleMapping>(select_sql)
            .bind(role)
            .fetch_all(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        Ok(mappings)
    }

    /// Clean up expired role assignments
    #[instrument(skip(self))]
    pub async fn cleanup_expired(&self) -> DatabaseResult<u64> {
        let delete_sql = "DELETE FROM user_roles WHERE expires_at < NOW()";

        let mut conn = self.pool.acquire().await?;
        let result = query(delete_sql)
            .execute(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        info!(
            "Cleaned up {} expired role assignments",
            result.rows_affected()
        );
        Ok(result.rows_affected())
    }

    /// Update role expiration
    pub async fn update_expiration(
        &self,
        user_id: &UserId,
        role: &str,
        expires_at: DateTime<Utc>,
    ) -> DatabaseResult<()> {
        let update_sql = "UPDATE user_roles SET expires_at = $1 WHERE user_id = $2 AND role = $3";

        let mut conn = self.pool.acquire().await?;
        let result = query(update_sql)
            .bind(expires_at)
            .bind(user_id.as_str())
            .bind(role)
            .execute(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(DatabaseError::not_found(
                "user_role",
                format!("{}:{}", user_id.as_str(), role),
            ));
        }

        debug!("Role expiration updated: {}:{}", user_id.as_str(), role);
        Ok(())
    }
}

/// Role-permission mapping repository
pub struct RolePermissionRepository {
    pool: DatabasePool,
}

impl RolePermissionRepository {
    /// Create a new role-permission repository
    pub fn new(pool: DatabasePool) -> Self {
        Self { pool }
    }

    /// Grant a permission to a role
    #[instrument(skip(self))]
    pub async fn grant_permission(&self, mapping: RolePermissionMapping) -> DatabaseResult<()> {
        let conditions_json =
            RolePermissionMapping::serialize_conditions(&mapping.parse_conditions()?)?;

        let insert_sql = r#"
            INSERT INTO role_permissions (role, permission, resource_type, conditions, created_at)
            VALUES ($1, $2, $3, $4, $5)
        "#;

        let mut conn = self.pool.acquire().await?;
        query(insert_sql)
            .bind(&mapping.role)
            .bind(&mapping.permission)
            .bind(&mapping.resource_type)
            .bind(&conditions_json)
            .bind(mapping.created_at)
            .execute(&mut *conn)
            .await
            .map_err(|e| {
                if e.to_string().contains("unique") || e.to_string().contains("duplicate") {
                    DatabaseError::duplicate(
                        "role_permission",
                        format!(
                            "Role {} already has permission {} for resource type {:?}",
                            mapping.role, mapping.permission, mapping.resource_type
                        ),
                    )
                } else {
                    DatabaseError::QueryError(e.to_string())
                }
            })?;

        info!(
            "Permission '{}' granted to role '{}' for resource type {:?}",
            mapping.permission, mapping.role, mapping.resource_type
        );
        Ok(())
    }

    /// Get all permissions for a role
    pub async fn get_role_permissions(
        &self,
        role: &str,
        resource_type: Option<&str>,
    ) -> DatabaseResult<Vec<RolePermissionMapping>> {
        let select_sql = if resource_type.is_some() {
            "SELECT * FROM role_permissions WHERE role = $1 AND resource_type = $2 ORDER BY created_at DESC"
        } else {
            "SELECT * FROM role_permissions WHERE role = $1 ORDER BY created_at DESC"
        };

        let mut conn = self.pool.acquire().await?;
        let mut query_builder = query_as::<_, RolePermissionMapping>(select_sql).bind(role);
        if let Some(rt) = resource_type {
            query_builder = query_builder.bind(rt);
        }

        let permissions = query_builder
            .fetch_all(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        Ok(permissions)
    }

    /// Check if a role has a specific permission
    pub async fn has_permission(
        &self,
        role: &str,
        permission: &str,
        resource_type: Option<&str>,
    ) -> DatabaseResult<Option<RolePermissionMapping>> {
        let select_sql = if resource_type.is_some() {
            "SELECT * FROM role_permissions WHERE role = $1 AND permission = $2 AND resource_type = $3"
        } else {
            "SELECT * FROM role_permissions WHERE role = $1 AND permission = $2 AND resource_type IS NULL"
        };

        let mut conn = self.pool.acquire().await?;
        let mut query_builder = query_as::<_, RolePermissionMapping>(select_sql)
            .bind(role)
            .bind(permission);
        if let Some(rt) = resource_type {
            query_builder = query_builder.bind(rt);
        }

        let result = query_builder
            .fetch_optional(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        Ok(result)
    }

    /// Revoke a permission from a role
    #[instrument(skip(self))]
    pub async fn revoke_permission(
        &self,
        role: &str,
        permission: &str,
        resource_type: Option<&str>,
    ) -> DatabaseResult<()> {
        let delete_sql = if resource_type.is_some() {
            "DELETE FROM role_permissions WHERE role = $1 AND permission = $2 AND resource_type = $3"
        } else {
            "DELETE FROM role_permissions WHERE role = $1 AND permission = $2 AND resource_type IS NULL"
        };

        let mut conn = self.pool.acquire().await?;
        let mut query_builder = query(delete_sql).bind(role).bind(permission);
        if let Some(rt) = resource_type {
            query_builder = query_builder.bind(rt);
        }

        let result = query_builder
            .execute(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(DatabaseError::not_found(
                "role_permission",
                format!("{}:{}:{:?}", role, permission, resource_type),
            ));
        }

        info!(
            "Permission '{}' revoked from role '{}' for resource type {:?}",
            permission, role, resource_type
        );
        Ok(())
    }

    /// Remove all permissions from a role
    #[instrument(skip(self))]
    pub async fn remove_all_permissions(&self, role: &str) -> DatabaseResult<u64> {
        let delete_sql = "DELETE FROM role_permissions WHERE role = $1";

        let mut conn = self.pool.acquire().await?;
        let result = query(delete_sql)
            .bind(role)
            .execute(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        info!(
            "Removed {} permissions from role '{}'",
            result.rows_affected(),
            role
        );
        Ok(result.rows_affected())
    }

    /// Get all permissions for all roles
    pub async fn get_all_permissions(&self) -> DatabaseResult<Vec<RolePermissionMapping>> {
        let select_sql = "SELECT * FROM role_permissions ORDER BY role, permission";

        let mut conn = self.pool.acquire().await?;
        let permissions = query_as::<_, RolePermissionMapping>(select_sql)
            .fetch_all(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        Ok(permissions)
    }
}

/// Policy repository
pub struct PolicyRepository {
    pool: DatabasePool,
}

impl PolicyRepository {
    /// Create a new policy repository
    pub fn new(pool: DatabasePool) -> Self {
        Self { pool }
    }

    /// Create a new policy
    #[instrument(skip(self))]
    pub async fn create(&self, policy: PolicyRecord) -> DatabaseResult<()> {
        let rules_json = PolicyRecord::serialize_rules(&policy.parse_rules()?)?;

        let insert_sql = r#"
            INSERT INTO policies (name, policy_type, rules, priority, enabled, description, created_by, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        "#;

        let mut conn = self.pool.acquire().await?;
        query(insert_sql)
            .bind(&policy.name)
            .bind(&policy.policy_type)
            .bind(&rules_json)
            .bind(policy.priority)
            .bind(policy.enabled)
            .bind(&policy.description)
            .bind(&policy.created_by)
            .bind(policy.created_at)
            .bind(policy.updated_at)
            .execute(&mut *conn)
            .await
            .map_err(|e| {
                if e.to_string().contains("unique") || e.to_string().contains("duplicate") {
                    DatabaseError::duplicate(
                        "policy",
                        format!("Policy {} already exists", policy.name),
                    )
                } else {
                    DatabaseError::QueryError(e.to_string())
                }
            })?;

        info!("Policy created: {}", policy.name);
        Ok(())
    }

    /// Get a policy by ID
    #[instrument(skip(self))]
    pub async fn get_by_id(&self, id: i64) -> DatabaseResult<PolicyRecord> {
        let select_sql = "SELECT * FROM policies WHERE id = $1";

        let mut conn = self.pool.acquire().await?;
        let result = query_as::<_, PolicyRecord>(select_sql)
            .bind(id)
            .fetch_optional(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        result.ok_or_else(|| DatabaseError::not_found("policy", id.to_string()))
    }

    /// Get a policy by name
    #[instrument(skip(self))]
    pub async fn get_by_name(&self, name: &str) -> DatabaseResult<PolicyRecord> {
        let select_sql = "SELECT * FROM policies WHERE name = $1";

        let mut conn = self.pool.acquire().await?;
        let result = query_as::<_, PolicyRecord>(select_sql)
            .bind(name)
            .fetch_optional(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        result.ok_or_else(|| DatabaseError::not_found("policy", name))
    }

    /// List all policies
    pub async fn list(&self, enabled_only: bool) -> DatabaseResult<Vec<PolicyRecord>> {
        let select_sql = if enabled_only {
            "SELECT * FROM policies WHERE enabled = true ORDER BY priority DESC, created_at DESC"
        } else {
            "SELECT * FROM policies ORDER BY priority DESC, created_at DESC"
        };

        let mut conn = self.pool.acquire().await?;
        let policies = query_as::<_, PolicyRecord>(select_sql)
            .fetch_all(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        Ok(policies)
    }

    /// Update a policy
    #[instrument(skip(self))]
    pub async fn update(&self, policy: PolicyRecord) -> DatabaseResult<()> {
        let rules_json = PolicyRecord::serialize_rules(&policy.parse_rules()?)?;

        let update_sql = r#"
            UPDATE policies SET
                name = $1, policy_type = $2, rules = $3, priority = $4, enabled = $5, description = $6, updated_at = $7
            WHERE id = $8
        "#;

        let mut conn = self.pool.acquire().await?;
        let result = query(update_sql)
            .bind(&policy.name)
            .bind(&policy.policy_type)
            .bind(&rules_json)
            .bind(policy.priority)
            .bind(policy.enabled)
            .bind(&policy.description)
            .bind(policy.updated_at)
            .bind(policy.id)
            .execute(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(DatabaseError::not_found("policy", policy.id.to_string()));
        }

        info!("Policy updated: {}", policy.name);
        Ok(())
    }

    /// Delete a policy
    #[instrument(skip(self))]
    pub async fn delete(&self, id: i64) -> DatabaseResult<()> {
        let delete_sql = "DELETE FROM policies WHERE id = $1";

        let mut conn = self.pool.acquire().await?;
        let result = query(delete_sql)
            .bind(id)
            .execute(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(DatabaseError::not_found("policy", id.to_string()));
        }

        info!("Policy deleted: {}", id);
        Ok(())
    }

    /// Enable or disable a policy
    pub async fn set_enabled(&self, id: i64, enabled: bool) -> DatabaseResult<()> {
        let update_sql = "UPDATE policies SET enabled = $1, updated_at = NOW() WHERE id = $2";

        let mut conn = self.pool.acquire().await?;
        let result = query(update_sql)
            .bind(enabled)
            .bind(id)
            .execute(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(DatabaseError::not_found("policy", id.to_string()));
        }

        info!(
            "Policy {} {}",
            if enabled { "enabled" } else { "disabled" },
            id
        );
        Ok(())
    }
}

/// Permission audit log repository
pub struct AuditLogRepository {
    pool: DatabasePool,
}

impl AuditLogRepository {
    /// Create a new audit log repository
    pub fn new(pool: DatabasePool) -> Self {
        Self { pool }
    }

    /// Log a permission check
    #[instrument(skip(self))]
    pub async fn log(&self, log: PermissionAuditLog) -> DatabaseResult<()> {
        let insert_sql = r#"
            INSERT INTO permission_audit_log (
                user_id, session_id, subject_type, subject_id,
                resource_type, resource_id, action, effect,
                policy_id, reason, ip_address, timestamp
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
        "#;

        let mut conn = self.pool.acquire().await?;
        query(insert_sql)
            .bind(&log.user_id)
            .bind(&log.session_id)
            .bind(&log.subject_type)
            .bind(&log.subject_id)
            .bind(&log.resource_type)
            .bind(&log.resource_id)
            .bind(&log.action)
            .bind(&log.effect)
            .bind(log.policy_id)
            .bind(&log.reason)
            .bind(&log.ip_address)
            .bind(log.timestamp)
            .execute(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        debug!("Permission audit logged: {}", log.action);
        Ok(())
    }

    /// Get audit logs for a user
    pub async fn get_by_user(
        &self,
        user_id: &str,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> DatabaseResult<Vec<PermissionAuditLog>> {
        let (sql, limit, offset) = match (limit, offset) {
            (Some(l), Some(o)) => (
                "SELECT * FROM permission_audit_log WHERE user_id = $1 ORDER BY timestamp DESC LIMIT $2 OFFSET $3".to_string(),
                Some(l), Some(o),
            ),
            (Some(l), None) => (
                "SELECT * FROM permission_audit_log WHERE user_id = $1 ORDER BY timestamp DESC LIMIT $2".to_string(),
                Some(l), None,
            ),
            (None, Some(o)) => (
                "SELECT * FROM permission_audit_log WHERE user_id = $1 ORDER BY timestamp DESC OFFSET $2".to_string(),
                None, Some(o),
            ),
            (None, None) => (
                "SELECT * FROM permission_audit_log WHERE user_id = $1 ORDER BY timestamp DESC".to_string(),
                None, None,
            ),
        };

        let mut conn = self.pool.acquire().await?;
        let mut query_builder = query_as::<_, PermissionAuditLog>(&sql).bind(user_id);
        if let Some(limit) = limit {
            query_builder = query_builder.bind(limit);
        }
        if let Some(offset) = offset {
            query_builder = query_builder.bind(offset);
        }

        let logs = query_builder
            .fetch_all(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        Ok(logs)
    }

    /// Get audit logs for a resource
    pub async fn get_by_resource(
        &self,
        resource_type: &str,
        resource_id: &str,
        limit: Option<i64>,
    ) -> DatabaseResult<Vec<PermissionAuditLog>> {
        let limit = limit.unwrap_or(50);
        let select_sql = r#"
            SELECT * FROM permission_audit_log
            WHERE resource_type = $1 AND resource_id = $2
            ORDER BY timestamp DESC
            LIMIT $3
        "#;

        let mut conn = self.pool.acquire().await?;
        let logs = query_as::<_, PermissionAuditLog>(select_sql)
            .bind(resource_type)
            .bind(resource_id)
            .bind(limit)
            .fetch_all(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        Ok(logs)
    }

    /// Get denied access attempts
    pub async fn get_denied(
        &self,
        user_id: Option<&str>,
        limit: Option<i64>,
    ) -> DatabaseResult<Vec<PermissionAuditLog>> {
        let limit = limit.unwrap_or(50);

        let (select_sql, _has_user_param) = if user_id.is_some() {
            (
                "SELECT * FROM permission_audit_log WHERE user_id = $1 AND effect = 'deny' ORDER BY timestamp DESC LIMIT $2",
                true,
            )
        } else {
            (
                "SELECT * FROM permission_audit_log WHERE effect = 'deny' ORDER BY timestamp DESC LIMIT $1",
                false,
            )
        };

        let mut conn = self.pool.acquire().await?;
        let mut query_builder = query_as::<_, PermissionAuditLog>(select_sql);
        if let Some(uid) = user_id {
            query_builder = query_builder.bind(uid);
        }
        query_builder = query_builder.bind(limit);

        let logs = query_builder
            .fetch_all(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        Ok(logs)
    }

    /// Clean up old audit logs
    #[instrument(skip(self))]
    pub async fn cleanup_old_logs(&self, days_old: i64) -> DatabaseResult<u64> {
        let cutoff_date = Utc::now() - chrono::Duration::days(days_old);
        let delete_sql = "DELETE FROM permission_audit_log WHERE timestamp < $1";

        let mut conn = self.pool.acquire().await?;
        let result = query(delete_sql)
            .bind(cutoff_date)
            .execute(&mut *conn)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        info!("Deleted {} old audit logs", result.rows_affected());
        Ok(result.rows_affected())
    }
}
