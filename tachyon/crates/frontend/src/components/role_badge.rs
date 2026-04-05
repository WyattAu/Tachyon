// Role Badge Component
// Displays user roles with badges and controls visibility based on permissions

use leptos::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserRole {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
    pub permissions: Vec<String>,
    pub is_system: bool,
}

impl UserRole {
    pub fn has_permission(&self, permission: &Permission) -> bool {
        if self.permissions.contains(&"owner".to_string()) {
            return true;
        }
        for perm_str in &self.permissions {
            if let Some(p) = Self::parse_permission(perm_str) {
                if p.includes(permission) {
                    return true;
                }
            }
        }
        false
    }

    fn parse_permission(s: &str) -> Option<Permission> {
        match s.to_lowercase().as_str() {
            "read" => Some(Permission::Read),
            "write" => Some(Permission::Write),
            "delete" => Some(Permission::Delete),
            "admin" => Some(Permission::Admin),
            "owner" => Some(Permission::Owner),
            _ => None,
        }
    }

    pub fn badge_color(&self) -> &'static str {
        match self.name.to_lowercase().as_str() {
            "owner" => "bg-purple-100 text-purple-800 dark:bg-purple-900 dark:text-purple-200",
            "admin" => "bg-red-100 text-red-800 dark:bg-red-900 dark:text-red-200",
            "editor" => "bg-orange-100 text-orange-800 dark:bg-orange-900 dark:text-orange-200",
            "writer" => "bg-blue-100 text-blue-800 dark:bg-blue-900 dark:text-blue-200",
            "reader" => "bg-gray-100 text-gray-800 dark:bg-gray-800 dark:text-gray-200",
            _ => "bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200",
        }
    }
}

#[component]
pub fn RoleBadge(role: UserRole, #[prop(optional)] size: Option<String>) -> impl IntoView {
    let size_class = match size.as_deref() {
        Some("sm") => "text-xs px-2 py-0.5",
        Some("lg") => "text-sm px-3 py-1.5",
        _ => "text-xs px-2.5 py-1",
    };

    let color_class = role.badge_color();

    view! {
        <span class=format!("inline-flex items-center font-medium rounded-full {} {}", size_class, color_class)>
            {role.name.clone()}
        </span>
    }
}

#[component]
pub fn PermissionBadge(permission: Permission) -> impl IntoView {
    let (color, icon) = match permission {
        Permission::Owner => (
            "bg-purple-100 text-purple-800 dark:bg-purple-900 dark:text-purple-200",
            "👑",
        ),
        Permission::Admin => (
            "bg-red-100 text-red-800 dark:bg-red-900 dark:text-red-200",
            "🛡️",
        ),
        Permission::Delete => (
            "bg-orange-100 text-orange-800 dark:bg-orange-900 dark:text-orange-200",
            "🗑️",
        ),
        Permission::Write => (
            "bg-blue-100 text-blue-800 dark:bg-blue-900 dark:text-blue-200",
            "✏️",
        ),
        Permission::Read => (
            "bg-gray-100 text-gray-800 dark:bg-gray-800 dark:text-gray-200",
            "👁️",
        ),
    };

    view! {
        <span class=format!("inline-flex items-center gap-1 text-xs font-medium rounded-full px-2 py-0.5 {}", color)>
            <span>{icon}</span>
            {permission.as_str().to_string()}
        </span>
    }
}

#[component]
pub fn RoleBasedVisibility(
    current_role: UserRole,
    required_permission: Permission,
    children: Children,
) -> impl IntoView {
    let visible = current_role.has_permission(&required_permission);

    view! {
        {if visible {
            Some(children())
        } else {
            None
        }}
    }
}

#[component]
pub fn AdminOnly(current_role: UserRole, children: Children) -> impl IntoView {
    let is_admin = current_role.has_permission(&Permission::Admin);

    view! {
        {if is_admin {
            Some(children())
        } else {
            None
        }}
    }
}

#[component]
pub fn OwnerOnly(current_role: UserRole, children: Children) -> impl IntoView {
    let is_owner = current_role.has_permission(&Permission::Owner);

    view! {
        {if is_owner {
            Some(children())
        } else {
            None
        }}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_permission_level() {
        assert!(Permission::Owner.level() > Permission::Admin.level());
        assert!(Permission::Admin.level() > Permission::Delete.level());
    }

    #[test]
    fn test_role_has_permission() {
        let admin_role = UserRole {
            id: 1,
            name: "admin".to_string(),
            description: None,
            permissions: vec![
                "admin".to_string(),
                "delete".to_string(),
                "write".to_string(),
                "read".to_string(),
            ],
            is_system: true,
        };

        assert!(admin_role.has_permission(&Permission::Read));
        assert!(admin_role.has_permission(&Permission::Write));
        assert!(admin_role.has_permission(&Permission::Delete));
        assert!(admin_role.has_permission(&Permission::Admin));
    }
}
