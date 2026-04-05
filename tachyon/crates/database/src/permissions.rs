// Permission Types and Checking
// Fine-grained permission system for RBAC

use serde::{Deserialize, Serialize};
use std::collections::HashSet;

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

    pub fn level(&self) -> u8 {
        match self {
            Self::Read => 1,
            Self::Write => 2,
            Self::Delete => 3,
            Self::Admin => 4,
            Self::Owner => 5,
        }
    }

    pub fn includes(&self, other: &Permission) -> bool {
        self.level() >= other.level()
    }
}

impl std::fmt::Display for Permission {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

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

    pub fn add_permission(&mut self, permission: Permission) {
        self.permissions.insert(permission.as_str().to_string());
        self.updated_at = chrono::Utc::now();
    }

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

pub fn check_permission(role: &Role, permission: Permission) -> bool {
    role.has_permission(&permission)
}

pub fn check_any_permission(role: &Role, permissions: &[Permission]) -> bool {
    permissions.iter().any(|p| role.has_permission(p))
}

pub fn check_all_permissions(role: &Role, permissions: &[Permission]) -> bool {
    permissions.iter().all(|p| role.has_permission(p))
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
