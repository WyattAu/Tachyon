// User type definitions
// Represents users, roles, and RBAC permissions in Tachyon system

use crate::id::UserId;
use crate::types::error::TachyonError;
use argon2::{
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Algorithm, Argon2, Params, Version,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;

// ============================================================================
// User Role Enum
// ============================================================================

/// User role for RBAC (Role-Based Access Control)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum UserRole {
    /// Reader - Can only read/view content
    #[serde(rename = "reader")]
    Reader,
    /// Writer - Can read and write content
    #[serde(rename = "writer")]
    Writer,
    /// Editor - Can edit and manage content structure
    #[serde(rename = "editor")]
    Editor,
    /// Admin - Full administrative access
    #[serde(rename = "admin")]
    Admin,
}

impl UserRole {
    /// Get the permission level for this role
    pub fn permission_level(&self) -> u8 {
        match self {
            Self::Reader => 1,
            Self::Writer => 2,
            Self::Editor => 3,
            Self::Admin => 4,
        }
    }

    /// Check if this role has at least the specified permission level
    pub fn has_permission(&self, min_level: u8) -> bool {
        self.permission_level() >= min_level
    }

    /// Check if this role can perform a specific action
    pub fn can_perform(&self, action: UserAction) -> bool {
        let required_level = action.required_level();
        self.has_permission(required_level)
    }
}

impl fmt::Display for UserRole {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let role = match self {
            Self::Reader => "reader",
            Self::Writer => "writer",
            Self::Editor => "editor",
            Self::Admin => "admin",
        };
        write!(f, "{}", role)
    }
}

// ============================================================================
// User Action Enum
// ============================================================================

/// User actions that require authorization
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum UserAction {
    /// Read/view content
    #[serde(rename = "read")]
    Read,
    /// Write/create content
    #[serde(rename = "write")]
    Write,
    /// Edit/modify content
    #[serde(rename = "edit")]
    Edit,
    /// Delete content
    #[serde(rename = "delete")]
    Delete,
    /// Manage users
    #[serde(rename = "manage_users")]
    ManageUsers,
    /// Configure system
    #[serde(rename = "configure")]
    Configure,
}

impl UserAction {
    /// Get the minimum permission level required for this action
    pub fn required_level(&self) -> u8 {
        match self {
            Self::Read => 1,
            Self::Write => 2,
            Self::Edit => 3,
            Self::Delete | Self::ManageUsers | Self::Configure => 4,
        }
    }
}

// ============================================================================
// User Permissions
// ============================================================================

/// User permissions structure for RBAC
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPermissions {
    /// User's primary role
    pub role: UserRole,
    /// Explicitly granted permissions (optional, for granular control)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub granted_permissions: Option<Vec<UserAction>>,
    /// Explicitly denied permissions
    #[serde(skip_serializing_if = "Option::is_none")]
    pub denied_permissions: Option<Vec<UserAction>>,
}

impl UserPermissions {
    /// Create new permissions with the specified role
    pub fn new(role: UserRole) -> Self {
        Self {
            role,
            granted_permissions: None,
            denied_permissions: None,
        }
    }

    /// Check if the user can perform a specific action
    pub fn can_perform(&self, action: UserAction) -> bool {
        // Check explicit denials first
        if let Some(ref denied) = self.denied_permissions {
            if denied.contains(&action) {
                return false;
            }
        }

        // Check explicit grants
        if let Some(ref granted) = self.granted_permissions {
            if granted.contains(&action) {
                return true;
            }
        }

        // Default to role-based permissions
        self.role.can_perform(action)
    }

    /// Grant a specific permission
    pub fn grant(&mut self, action: UserAction) {
        self.granted_permissions
            .get_or_insert_with(Vec::new)
            .push(action);
    }

    /// Deny a specific permission
    pub fn deny(&mut self, action: UserAction) {
        self.denied_permissions
            .get_or_insert_with(Vec::new)
            .push(action);
    }
}

impl Default for UserPermissions {
    fn default() -> Self {
        Self::new(UserRole::Reader)
    }
}

// ============================================================================
// User Type
// ============================================================================

/// User type for classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum UserType {
    /// Regular human user
    #[serde(rename = "regular")]
    Regular,
    /// Service account for automation
    #[serde(rename = "service")]
    Service,
    /// System account
    #[serde(rename = "system")]
    System,
}

impl fmt::Display for UserType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let user_type = match self {
            Self::Regular => "regular",
            Self::Service => "service",
            Self::System => "system",
        };
        write!(f, "{}", user_type)
    }
}

// ============================================================================
// User Struct
// ============================================================================

/// User entity in the Tachyon system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    /// Unique user identifier
    pub id: UserId,
    /// Username for login (unique)
    pub username: String,
    /// Display name (may not be unique)
    pub display_name: String,
    /// Email address (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    /// User type classification
    pub user_type: UserType,
    /// User permissions/role
    pub permissions: UserPermissions,
    /// When the user was created
    pub created_at: DateTime<Utc>,
    /// When the user was last updated
    pub updated_at: DateTime<Utc>,
    /// Whether the user is active
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_active: Option<bool>,
    /// Password hash (only for password-based auth)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password_hash: Option<String>,
}

impl User {
    /// Create a new user
    ///
    /// # Arguments
    /// * `id` - User ID
    /// * `username` - Username
    /// * `display_name` - Display name
    /// * `role` - User role
    pub fn new(id: UserId, username: String, display_name: String, role: UserRole) -> Self {
        let now = Utc::now();
        Self {
            id,
            username,
            display_name,
            email: None,
            user_type: UserType::Regular,
            permissions: UserPermissions::new(role),
            created_at: now,
            updated_at: now,
            is_active: Some(true),
            password_hash: None,
        }
    }

    /// Set the user's email
    pub fn with_email(mut self, email: String) -> Self {
        self.email = Some(email);
        self
    }

    /// Set the user's type
    pub fn with_user_type(mut self, user_type: UserType) -> Self {
        self.user_type = user_type;
        self
    }

    /// Set whether the user is active
    pub fn with_active(mut self, active: bool) -> Self {
        self.is_active = Some(active);
        self
    }

    /// Set the user's role
    pub fn with_role(mut self, role: UserRole) -> Self {
        self.permissions.role = role;
        self
    }

    /// Update the timestamp
    pub fn touch(&mut self) {
        self.updated_at = Utc::now();
    }

    /// Check if the user can perform an action
    pub fn can_perform(&self, action: UserAction) -> bool {
        if !self.is_active.unwrap_or(true) {
            return false;
        }
        self.permissions.can_perform(action)
    }

    /// Hash a password using Argon2
    ///
    /// # Arguments
    /// * `password` - Plain text password
    ///
    /// # Returns
    /// Result containing the password hash or error
    pub fn hash_password(password: &str) -> Result<String, TachyonError> {
        let salt = SaltString::generate(&mut rand::rngs::OsRng);
        let argon2 = Argon2::new(
            Algorithm::Argon2id,
            Version::V0x13,
            Params::new(65536, 2, 1, None).map_err(|e| {
                TachyonError::internal("PASSWORD_HASH", format!("Argon2 params error: {}", e))
            })?,
        );

        let password_hash = argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| {
                TachyonError::internal("PASSWORD_HASH", format!("Password hashing error: {}", e))
            })?;

        Ok(password_hash.to_string())
    }

    /// Verify a password against the stored hash
    ///
    /// # Arguments
    /// * `password` - Plain text password to verify
    /// * `hash` - Stored password hash
    ///
    /// # Returns
    /// Result indicating if password is valid
    pub fn verify_password(password: &str, hash: &str) -> Result<bool, TachyonError> {
        let parsed_hash = PasswordHash::new(hash).map_err(|e| {
            TachyonError::authentication("INVALID_HASH", format!("Invalid password hash: {}", e))
        })?;

        let argon2 = Argon2::default();

        argon2
            .verify_password(password.as_bytes(), &parsed_hash)
            .map(|()| true)
            .map_err(|e| {
                TachyonError::authentication(
                    "PASSWORD_MISMATCH",
                    format!("Password verification failed: {}", e),
                )
            })
    }

    /// Set the password for this user
    ///
    /// # Arguments
    /// * `password` - Plain text password
    ///
    /// # Returns
    /// Result indicating success or error
    pub fn set_password(&mut self, password: &str) -> Result<(), TachyonError> {
        let hash = Self::hash_password(password)?;
        self.password_hash = Some(hash);
        self.touch();
        Ok(())
    }

    /// Verify the password for this user
    ///
    /// # Arguments
    /// * `password` - Plain text password to verify
    ///
    /// # Returns
    /// Result indicating if password is valid
    pub fn verify(&self, password: &str) -> Result<bool, TachyonError> {
        match &self.password_hash {
            Some(hash) => Self::verify_password(password, hash),
            None => Err(TachyonError::authentication(
                "NO_PASSWORD",
                "User has no password set",
            )),
        }
    }

    /// Validate the user data
    ///
    /// # Returns
    /// Result indicating valid data or validation error
    pub fn validate(&self) -> Result<(), TachyonError> {
        if self.username.is_empty() {
            return Err(TachyonError::field_validation(
                "username",
                "Username cannot be empty",
            ));
        }

        if self.username.len() < 3 {
            return Err(TachyonError::field_validation(
                "username",
                "Username must be at least 3 characters",
            ));
        }

        if self.username.len() > 50 {
            return Err(TachyonError::field_validation(
                "username",
                "Username cannot exceed 50 characters",
            ));
        }

        if self.display_name.is_empty() {
            return Err(TachyonError::field_validation(
                "display_name",
                "Display name cannot be empty",
            ));
        }

        if let Some(ref email) = self.email {
            if !email.contains('@') {
                return Err(TachyonError::field_validation(
                    "email",
                    "Invalid email format",
                ));
            }
        }

        Ok(())
    }
}

// ============================================================================
// UserBuilder for fluent construction
// ============================================================================

/// Builder pattern for creating User instances
pub struct UserBuilder {
    id: UserId,
    username: String,
    display_name: String,
    email: Option<String>,
    user_type: UserType,
    role: UserRole,
    is_active: bool,
    password: Option<String>,
}

impl UserBuilder {
    /// Create a new UserBuilder
    ///
    /// # Arguments
    /// * `id` - User ID
    /// * `username` - Username
    /// * `display_name` - Display name
    pub fn new(id: UserId, username: String, display_name: String) -> Self {
        Self {
            id,
            username,
            display_name,
            email: None,
            user_type: UserType::Regular,
            role: UserRole::Reader,
            is_active: true,
            password: None,
        }
    }

    /// Set the user's email
    pub fn email(mut self, email: String) -> Self {
        self.email = Some(email);
        self
    }

    /// Set the user's type
    pub fn user_type(mut self, user_type: UserType) -> Self {
        self.user_type = user_type;
        self
    }

    /// Set the user's role
    pub fn role(mut self, role: UserRole) -> Self {
        self.role = role;
        self
    }

    /// Set whether the user is active
    pub fn active(mut self, active: bool) -> Self {
        self.is_active = active;
        self
    }

    /// Set the user's password
    pub fn password(mut self, password: String) -> Self {
        self.password = Some(password);
        self
    }

    /// Build the User instance
    ///
    /// # Returns
    /// Result containing the User or error
    pub fn build(self) -> Result<User, TachyonError> {
        let mut user = User::new(self.id, self.username, self.display_name, self.role);

        if let Some(email) = self.email {
            user = user.with_email(email);
        }

        user = user
            .with_user_type(self.user_type)
            .with_active(self.is_active);

        if let Some(password) = self.password {
            user.set_password(&password)?;
        }

        user.validate()?;

        Ok(user)
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_role_permission_level() {
        assert_eq!(UserRole::Reader.permission_level(), 1);
        assert_eq!(UserRole::Writer.permission_level(), 2);
        assert_eq!(UserRole::Editor.permission_level(), 3);
        assert_eq!(UserRole::Admin.permission_level(), 4);
    }

    #[test]
    fn test_user_role_can_perform() {
        assert!(UserRole::Reader.can_perform(UserAction::Read));
        assert!(!UserRole::Reader.can_perform(UserAction::Write));
        assert!(UserRole::Writer.can_perform(UserAction::Read));
        assert!(UserRole::Writer.can_perform(UserAction::Write));
        assert!(UserRole::Admin.can_perform(UserAction::Configure));
    }

    #[test]
    fn test_user_permissions() {
        let mut perms = UserPermissions::new(UserRole::Reader);
        assert!(perms.can_perform(UserAction::Read));
        assert!(!perms.can_perform(UserAction::Write));

        perms.grant(UserAction::Write);
        assert!(perms.can_perform(UserAction::Write));

        perms.deny(UserAction::Read);
        assert!(!perms.can_perform(UserAction::Read));
    }

    #[test]
    fn test_user_creation() {
        let user_id = crate::id::generate_user_id();
        let user = User::new(
            user_id,
            "testuser".to_string(),
            "Test User".to_string(),
            UserRole::Writer,
        );

        assert_eq!(user.username, "testuser");
        assert_eq!(user.display_name, "Test User");
        assert_eq!(user.permissions.role, UserRole::Writer);
        assert!(user.can_perform(UserAction::Read));
        assert!(user.can_perform(UserAction::Write));
    }

    #[test]
    fn test_user_validation() {
        let user_id = crate::id::generate_user_id();

        // Valid user
        let user = User::new(
            user_id,
            "validuser".to_string(),
            "Valid User".to_string(),
            UserRole::Reader,
        );
        assert!(user.validate().is_ok());

        // Empty username
        let invalid_user = User::new(
            crate::id::generate_user_id(),
            "".to_string(),
            "Test".to_string(),
            UserRole::Reader,
        );
        assert!(invalid_user.validate().is_err());

        // Invalid email
        let invalid_email = User::new(
            crate::id::generate_user_id(),
            "testuser".to_string(),
            "Test".to_string(),
            UserRole::Reader,
        )
        .with_email("not-an-email".to_string());
        assert!(invalid_email.validate().is_err());
    }

    #[test]
    fn test_user_builder() {
        let user_id = crate::id::generate_user_id();
        let user = UserBuilder::new(user_id, "testuser".to_string(), "Test User".to_string())
            .email("test@example.com".to_string())
            .role(UserRole::Editor)
            .active(true)
            .build()
            .expect("Should build user");

        assert_eq!(user.username, "testuser");
        assert_eq!(user.email, Some("test@example.com".to_string()));
        assert_eq!(user.permissions.role, UserRole::Editor);
    }

    #[test]
    fn test_user_inactive() {
        let user_id = crate::id::generate_user_id();
        let mut user = User::new(
            user_id,
            "testuser".to_string(),
            "Test User".to_string(),
            UserRole::Admin,
        );
        user.is_active = Some(false);

        // Inactive user cannot perform actions
        assert!(!user.can_perform(UserAction::Read));
    }
}
