// Permission Types and Checking
// Fine-grained permission system for RBAC

use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Fine-grained permission levels used for role-based access control.
///
/// Permissions form a hierarchy: `Owner > Admin > Delete > Write > Read`.
/// Higher-level permissions implicitly include all lower ones.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Permission {
    Read,
    Write,
    Delete,
    Admin,
    Owner,
}

impl Permission {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Read => "read",
            Self::Write => "write",
            Self::Delete => "delete",
            Self::Admin => "admin",
            Self::Owner => "owner",
        }
    }

    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "read" => Some(Self::Read),
            "write" => Some(Self::Write),
            "delete" => Some(Self::Delete),
            "admin" => Some(Self::Admin),
            "owner" => Some(Self::Owner),
            _ => None,
        }
    }

    /// Return the numeric hierarchy level of this permission (1–5).
    ///
    /// Higher values represent broader access.
    pub fn level(&self) -> u8 {
        match self {
            Self::Read => 1,
            Self::Write => 2,
            Self::Delete => 3,
            Self::Admin => 4,
            Self::Owner => 5,
        }
    }

    /// Check whether this permission level includes `other`.
    ///
    /// Returns `true` when `self.level() >= other.level()`, meaning
    /// e.g. `Admin` includes `Write` but not vice versa.
    pub fn includes(&self, other: &Permission) -> bool {
        self.level() >= other.level()
    }
}

impl std::fmt::Display for Permission {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// A named role with a set of permission strings.
///
/// Roles can be system-defined (immutable) or custom. The
/// [`has_permission`](Role::has_permission) method checks permission
/// via both direct name match and the hierarchical level system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Role {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
    pub permissions: HashSet<String>,
    pub is_system: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl Role {
    pub fn new(name: String, permissions: HashSet<String>) -> Self {
        let now = chrono::Utc::now();
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

    /// Check whether this role grants a specific permission.
    ///
    /// A role with the "owner" permission grants everything. Otherwise the
    /// check looks for a direct name match first, then falls back to the
    /// hierarchical [`Permission::includes`] logic.
    pub fn has_permission(&self, permission: &Permission) -> bool {
        if self.permissions.contains("owner") {
            return true;
        }
        if let Some(perm_str) = permission.as_str().into() {
            if self.permissions.contains(perm_str) {
                return true;
            }
        }
        for perm_name in &self.permissions {
            if let Some(role_perm) = Permission::from_str(perm_name) {
                if role_perm.includes(permission) {
                    return true;
                }
            }
        }
        false
    }

    /// Add a permission to this role and bump `updated_at`.
    pub fn add_permission(&mut self, permission: Permission) {
        self.permissions.insert(permission.as_str().to_string());
        self.updated_at = chrono::Utc::now();
    }

    /// Remove a permission from this role and bump `updated_at`.
    pub fn remove_permission(&mut self, permission: &Permission) {
        self.permissions.remove(permission.as_str());
        self.updated_at = chrono::Utc::now();
    }
}

pub struct DefaultRoles;

impl DefaultRoles {
    pub fn admin() -> Role {
        let mut perms = HashSet::new();
        perms.insert("owner".to_string());
        Role::new("admin".to_string(), perms)
            .with_description("Full administrative access".to_string())
    }

    pub fn editor() -> Role {
        let mut perms = HashSet::new();
        perms.insert("read".to_string());
        perms.insert("write".to_string());
        perms.insert("delete".to_string());
        Role::new("editor".to_string(), perms)
            .with_description("Can read, write, and delete content".to_string())
    }

    pub fn writer() -> Role {
        let mut perms = HashSet::new();
        perms.insert("read".to_string());
        perms.insert("write".to_string());
        Role::new("writer".to_string(), perms)
            .with_description("Can read and write content".to_string())
    }

    pub fn reader() -> Role {
        let mut perms = HashSet::new();
        perms.insert("read".to_string());
        Role::new("reader".to_string(), perms).with_description("Can only read content".to_string())
    }

    pub fn all() -> Vec<Role> {
        vec![
            Self::owner(),
            Self::admin(),
            Self::editor(),
            Self::writer(),
            Self::reader(),
        ]
    }

    pub fn owner() -> Role {
        let mut perms = HashSet::new();
        perms.insert("owner".to_string());
        Role::new("owner".to_string(), perms)
            .with_description("Full ownership with all permissions".to_string())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourcePermission {
    pub resource_type: String,
    pub resource_id: Option<String>,
    pub permission: Permission,
}

impl ResourcePermission {
    pub fn new(resource_type: String, permission: Permission) -> Self {
        Self {
            resource_type,
            resource_id: None,
            permission,
        }
    }

    pub fn for_resource(
        resource_type: String,
        resource_id: String,
        permission: Permission,
    ) -> Self {
        Self {
            resource_type,
            resource_id: Some(resource_id),
            permission,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_permission_level() {
        assert!(Permission::Owner.level() > Permission::Admin.level());
        assert!(Permission::Admin.level() > Permission::Delete.level());
        assert!(Permission::Delete.level() > Permission::Write.level());
        assert!(Permission::Write.level() > Permission::Read.level());
    }

    #[test]
    fn test_permission_includes() {
        assert!(Permission::Owner.includes(&Permission::Read));
        assert!(Permission::Admin.includes(&Permission::Write));
        assert!(!Permission::Read.includes(&Permission::Write));
    }

    #[test]
    fn test_role_permissions() {
        let admin = DefaultRoles::admin();
        assert!(admin.has_permission(&Permission::Read));
        assert!(admin.has_permission(&Permission::Write));
        assert!(admin.has_permission(&Permission::Delete));
        assert!(admin.has_permission(&Permission::Admin));

        let reader = DefaultRoles::reader();
        assert!(reader.has_permission(&Permission::Read));
        assert!(!reader.has_permission(&Permission::Write));
    }

    #[test]
    fn test_role_add_remove_permission() {
        let mut role = DefaultRoles::reader();
        assert!(!role.has_permission(&Permission::Write));

        role.add_permission(Permission::Write);
        assert!(role.has_permission(&Permission::Write));

        role.remove_permission(&Permission::Write);
        assert!(!role.has_permission(&Permission::Write));
    }
}
