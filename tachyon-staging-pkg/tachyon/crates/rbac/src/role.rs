// Role definitions module
// Defines user roles and their hierarchy

use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// User role in the system
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Role {
    /// Guest user (read-only)
    Guest,
    /// Regular user
    User,
    /// Power user (extended permissions)
    PowerUser,
    /// Editor (can edit most content)
    Editor,
    /// Moderator (can moderate content)
    Moderator,
    /// Administrator (full access)
    Admin,
    /// System owner (root access)
    Owner,
}

impl Role {
    /// Check if this role has equal or higher privilege than another
    ///
    /// # Arguments
    /// * `other` - The role to compare against
    ///
    /// # Returns
    /// true if this role has equal or higher privilege
    pub fn has_equal_or_higher_privilege(&self, other: &Role) -> bool {
        self.privilege_level() >= other.privilege_level()
    }

    /// Get the privilege level of this role (higher = more privilege)
    pub fn privilege_level(&self) -> u8 {
        match self {
            Role::Guest => 0,
            Role::User => 1,
            Role::PowerUser => 2,
            Role::Editor => 3,
            Role::Moderator => 4,
            Role::Admin => 5,
            Role::Owner => 6,
        }
    }

    /// Get all permissions for this role
    pub fn permissions(&self) -> HashSet<String> {
        let mut perms = HashSet::new();

        match self {
            Role::Guest => {
                perms.insert("read".to_string());
            }
            Role::User => {
                perms.insert("read".to_string());
                perms.insert("create_own".to_string());
                perms.insert("edit_own".to_string());
                perms.insert("delete_own".to_string());
            }
            Role::PowerUser => {
                perms.insert("read".to_string());
                perms.insert("create_own".to_string());
                perms.insert("edit_own".to_string());
                perms.insert("delete_own".to_string());
                perms.insert("share_own".to_string());
            }
            Role::Editor => {
                perms.insert("read".to_string());
                perms.insert("create".to_string());
                perms.insert("edit".to_string());
                perms.insert("delete".to_string());
            }
            Role::Moderator => {
                perms.insert("read".to_string());
                perms.insert("create".to_string());
                perms.insert("edit".to_string());
                perms.insert("delete".to_string());
                perms.insert("moderate".to_string());
                perms.insert("review".to_string());
            }
            Role::Admin => {
                perms.insert("read".to_string());
                perms.insert("create".to_string());
                perms.insert("edit".to_string());
                perms.insert("delete".to_string());
                perms.insert("share".to_string());
                perms.insert("admin".to_string());
                perms.insert("review".to_string());
                perms.insert("approve".to_string());
            }
            Role::Owner => {
                perms.insert("*".to_string());
            }
        }

        perms
    }
}

#[allow(clippy::derivable_impls)]
impl Default for Role {
    fn default() -> Self {
        Role::Guest
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_privilege_levels() {
        assert!(Role::Admin.privilege_level() > Role::User.privilege_level());
        assert!(Role::Owner.privilege_level() > Role::Admin.privilege_level());
    }

    #[test]
    fn test_has_equal_or_higher_privilege() {
        assert!(Role::Admin.has_equal_or_higher_privilege(&Role::User));
        assert!(!Role::User.has_equal_or_higher_privilege(&Role::Admin));
    }
}
