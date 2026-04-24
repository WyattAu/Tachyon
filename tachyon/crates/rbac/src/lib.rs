// Tachyon RBAC - Role-Based Access Control
// Provides permission checking, policy evaluation, session management, and enforcer logic

pub mod cache;
pub mod enforcer;
pub mod error;
pub mod permission;
pub mod policy;
pub mod session;
pub mod types;

// Re-export common types for convenience
pub use cache::AuthorizationCache;
pub use enforcer::{Enforcer, EnforcerConfig};
pub use error::{RbacError, RbacResult};
pub use permission::{Permission, PermissionChecker};
pub use policy::{Policy, PolicyEngine, PolicyRule, PolicyType};
pub use session::{SessionManager, SessionStore};
pub use types::{AuthContext, Resource, Subject};

#[doc(hidden)]
pub use tachyon_core::id::{SessionId, UserId};
#[doc(hidden)]
pub use tachyon_core::types::session::{
    Session, SessionStatus, SessionToken, SessionType, TokenType,
};
#[doc(hidden)]
pub use tachyon_core::types::user::{User, UserAction, UserPermissions, UserRole, UserType};

/// RBAC library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Initialize the RBAC system with default configuration
///
/// # Returns
/// Result containing the initialized Enforcer or error
///
/// # Errors
/// Returns error if initialization fails
pub fn init() -> RbacResult<Enforcer> {
    Ok(Enforcer::new())
}

/// Initialize the RBAC system with a custom policy configuration
///
/// # Arguments
/// * `policy_config` - Path to policy configuration file
///
/// # Returns
/// Result containing the initialized Enforcer or error
///
/// # Errors
/// Returns error if initialization fails or policy config is invalid
pub fn init_with_config(_policy_config: &str) -> RbacResult<Enforcer> {
    Ok(Enforcer::with_config(EnforcerConfig::default()))
}

/// Initialize the RBAC system with a SQLite database backend
///
/// # Arguments
/// * `database_url` - SQLite database connection URL
///
/// # Returns
/// Result containing the initialized Enforcer or error
///
/// # Errors
/// Returns error if initialization fails or database connection fails
pub async fn init_with_db(database_url: &str) -> RbacResult<Enforcer> {
    let session_manager = SessionManager::new(database_url).await?;
    Ok(Enforcer::with_session_manager(
        session_manager,
        EnforcerConfig::default(),
    ))
}

/// Create a new authorization context
///
/// # Arguments
/// * `user_id` - User ID
/// * `session_id` - Session ID
///
/// # Returns
/// New AuthContext instance
pub fn auth_context(user_id: UserId, session_id: SessionId) -> AuthContext {
    AuthContext::new(user_id, session_id)
}

/// Create a new resource reference
///
/// # Arguments
/// * `resource_type` - Type of resource (e.g., "document", "repository")
/// * `resource_id` - Unique resource identifier
///
/// # Returns
/// New Resource instance
pub fn resource(resource_type: &str, resource_id: &str) -> Resource {
    Resource::new(resource_type, resource_id)
}

/// Create a new subject reference
///
/// # Arguments
/// * `subject_type` - Type of subject (e.g., "user", "role")
/// * `subject_id` - Unique subject identifier
///
/// # Returns
/// New Subject instance
pub fn subject(subject_type: &str, subject_id: &str) -> Subject {
    Subject::new(subject_type, subject_id)
}
