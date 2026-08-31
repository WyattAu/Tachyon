// RBAC Types and Structures
// Core data structures for RBAC system

use crate::error::{RbacError, RbacResult};
use crate::{SessionId, UserId};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fmt;

// ============================================================================
// Subject
// ============================================================================

/// Subject represents an entity requesting access
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Subject {
    /// Subject type (user, role, service, etc.)
    pub subject_type: String,
    /// Unique subject identifier
    pub subject_id: String,
    /// Additional attributes for ABAC
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub attributes: BTreeMap<String, String>,
}

impl Subject {
    /// Create a new subject
    ///
    /// # Arguments
    /// * `subject_type` - Type of subject
    /// * `subject_id` - Unique subject identifier
    ///
    /// # Returns
    /// New Subject instance
    pub fn new(subject_type: &str, subject_id: &str) -> Self {
        Self {
            subject_type: subject_type.to_string(),
            subject_id: subject_id.to_string(),
            attributes: BTreeMap::new(),
        }
    }

    /// Create a subject from user ID
    ///
    /// # Arguments
    /// * `user_id` - User ID
    ///
    /// # Returns
    /// New Subject instance
    pub fn from_user(user_id: &UserId) -> Self {
        Self::new("user", &user_id.as_str())
    }

    /// Create a subject from session ID
    ///
    /// # Arguments
    /// * `session_id` - Session ID
    ///
    /// # Returns
    /// New Subject instance
    pub fn from_session(session_id: &SessionId) -> Self {
        Self::new("session", &session_id.as_str())
    }

    /// Create a subject from role
    ///
    /// # Arguments
    /// * `role` - Role name
    ///
    /// # Returns
    /// New Subject instance
    pub fn from_role(role: &str) -> Self {
        Self::new("role", role)
    }

    /// Add an attribute
    ///
    /// # Arguments
    /// * `key` - Attribute key
    /// * `value` - Attribute value
    ///
    /// # Returns
    /// Self for method chaining
    pub fn with_attribute(mut self, key: &str, value: &str) -> Self {
        self.attributes.insert(key.to_string(), value.to_string());
        self
    }

    /// Get an attribute value
    ///
    /// # Arguments
    /// * `key` - Attribute key
    ///
    /// # Returns
    /// Option containing the attribute value
    pub fn get_attribute(&self, key: &str) -> Option<&String> {
        self.attributes.get(key)
    }

    /// Validate the subject
    ///
    /// # Returns
    /// Result indicating if subject is valid or error
    pub fn validate(&self) -> RbacResult<()> {
        if self.subject_type.is_empty() {
            return Err(RbacError::invalid_subject("Subject type cannot be empty"));
        }

        if self.subject_id.is_empty() {
            return Err(RbacError::invalid_subject("Subject ID cannot be empty"));
        }

        Ok(())
    }
}

impl fmt::Display for Subject {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.subject_type, self.subject_id)
    }
}

// ============================================================================
// Resource
// ============================================================================

/// Resource represents an object being accessed
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Resource {
    /// Resource type (document, repository, etc.)
    pub resource_type: String,
    /// Unique resource identifier
    pub resource_id: String,
    /// Resource owner ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner_id: Option<String>,
    /// Additional attributes for ABAC
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub attributes: BTreeMap<String, String>,
}

impl Resource {
    /// Create a new resource
    ///
    /// # Arguments
    /// * `resource_type` - Type of resource
    /// * `resource_id` - Unique resource identifier
    ///
    /// # Returns
    /// New Resource instance
    pub fn new(resource_type: &str, resource_id: &str) -> Self {
        Self {
            resource_type: resource_type.to_string(),
            resource_id: resource_id.to_string(),
            owner_id: None,
            attributes: BTreeMap::new(),
        }
    }

    /// Set the resource owner
    ///
    /// # Arguments
    /// * `owner_id` - Owner ID
    ///
    /// # Returns
    /// Self for method chaining
    pub fn with_owner(mut self, owner_id: &str) -> Self {
        self.owner_id = Some(owner_id.to_string());
        self
    }

    /// Add an attribute
    ///
    /// # Arguments
    /// * `key` - Attribute key
    /// * `value` - Attribute value
    ///
    /// # Returns
    /// Self for method chaining
    pub fn with_attribute(mut self, key: &str, value: &str) -> Self {
        self.attributes.insert(key.to_string(), value.to_string());
        self
    }

    /// Get an attribute value
    ///
    /// # Arguments
    /// * `key` - Attribute key
    ///
    /// # Returns
    /// Option containing the attribute value
    pub fn get_attribute(&self, key: &str) -> Option<&String> {
        self.attributes.get(key)
    }

    /// Validate the resource
    ///
    /// # Returns
    /// Result indicating if resource is valid or error
    pub fn validate(&self) -> RbacResult<()> {
        if self.resource_type.is_empty() {
            return Err(RbacError::invalid_resource("Resource type cannot be empty"));
        }

        if self.resource_id.is_empty() {
            return Err(RbacError::invalid_resource("Resource ID cannot be empty"));
        }

        Ok(())
    }
}

impl fmt::Display for Resource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.resource_type, self.resource_id)
    }
}

// ============================================================================
// AuthContext
// ============================================================================

/// Authentication context containing user and session information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthContext {
    /// User ID
    pub user_id: UserId,
    /// Session ID
    pub session_id: SessionId,
    /// User roles
    pub roles: Vec<String>,
    /// Additional context attributes
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub attributes: BTreeMap<String, String>,
    /// Timestamp when context was created
    pub created_at: DateTime<Utc>,
}

impl AuthContext {
    /// Create a new authentication context
    ///
    /// # Arguments
    /// * `user_id` - User ID
    /// * `session_id` - Session ID
    ///
    /// # Returns
    /// New AuthContext instance
    pub fn new(user_id: UserId, session_id: SessionId) -> Self {
        Self {
            user_id,
            session_id,
            roles: Vec::new(),
            attributes: BTreeMap::new(),
            created_at: Utc::now(),
        }
    }

    /// Add a role to the context
    ///
    /// # Arguments
    /// * `role` - Role name
    ///
    /// # Returns
    /// Self for method chaining
    pub fn with_role(mut self, role: &str) -> Self {
        self.roles.push(role.to_string());
        self
    }

    /// Add multiple roles to the context
    ///
    /// # Arguments
    /// * `roles` - List of role names
    ///
    /// # Returns
    /// Self for method chaining
    pub fn with_roles(mut self, roles: &[&str]) -> Self {
        self.roles.extend(roles.iter().map(|r| r.to_string()));
        self
    }

    /// Add an attribute to the context
    ///
    /// # Arguments
    /// * `key` - Attribute key
    /// * `value` - Attribute value
    ///
    /// # Returns
    /// Self for method chaining
    pub fn with_attribute(mut self, key: &str, value: &str) -> Self {
        self.attributes.insert(key.to_string(), value.to_string());
        self
    }

    /// Check if context has a specific role
    ///
    /// # Arguments
    /// * `role` - Role name
    ///
    /// # Returns
    /// True if context has the role
    pub fn has_role(&self, role: &str) -> bool {
        self.roles.contains(&role.to_string())
    }

    /// Get an attribute value
    ///
    /// # Arguments
    /// * `key` - Attribute key
    ///
    /// # Returns
    /// Option containing the attribute value
    pub fn get_attribute(&self, key: &str) -> Option<&String> {
        self.attributes.get(key)
    }

    /// Create a subject from this context
    ///
    /// # Returns
    /// New Subject instance
    pub fn as_subject(&self) -> Subject {
        let mut subject = Subject::from_user(&self.user_id);
        for role in &self.roles {
            subject = subject.with_attribute("role", role);
        }
        subject
    }
}

// ============================================================================
// Action
// ============================================================================

/// Action represents an operation being performed
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Action {
    /// Action name (read, write, delete, etc.)
    pub action_name: String,
    /// Action scope (global, resource-specific)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
    /// Additional attributes
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub attributes: BTreeMap<String, String>,
}

impl Action {
    /// Create a new action
    ///
    /// # Arguments
    /// * `action_name` - Action name
    ///
    /// # Returns
    /// New Action instance
    pub fn new(action_name: &str) -> Self {
        Self {
            action_name: action_name.to_string(),
            scope: None,
            attributes: BTreeMap::new(),
        }
    }

    /// Set the action scope
    ///
    /// # Arguments
    /// * `scope` - Action scope
    ///
    /// # Returns
    /// Self for method chaining
    pub fn with_scope(mut self, scope: &str) -> Self {
        self.scope = Some(scope.to_string());
        self
    }

    /// Add an attribute
    ///
    /// # Arguments
    /// * `key` - Attribute key
    /// * `value` - Attribute value
    ///
    /// # Returns
    /// Self for method chaining
    pub fn with_attribute(mut self, key: &str, value: &str) -> Self {
        self.attributes.insert(key.to_string(), value.to_string());
        self
    }

    /// Validate the action
    ///
    /// # Returns
    /// Result indicating if action is valid or error
    pub fn validate(&self) -> RbacResult<()> {
        if self.action_name.is_empty() {
            return Err(RbacError::invalid_policy("Action name cannot be empty"));
        }

        Ok(())
    }
}

impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(ref scope) = self.scope {
            write!(f, "{}:{}", scope, self.action_name)
        } else {
            write!(f, "{}", self.action_name)
        }
    }
}

// ============================================================================
// Permission Effect
// ============================================================================

/// Effect of a permission evaluation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Effect {
    /// Allow the action
    #[serde(rename = "allow")]
    Allow,
    /// Deny the action
    #[serde(rename = "deny")]
    Deny,
}

impl Effect {
    /// Combine two effects (deny takes precedence)
    ///
    /// # Arguments
    /// * `other` - Other effect
    ///
    /// # Returns
    /// Combined effect
    pub fn combine(self, other: Self) -> Self {
        if self == Self::Deny || other == Self::Deny {
            Self::Deny
        } else {
            Self::Allow
        }
    }

    /// Convert effect to boolean
    ///
    /// # Returns
    /// True if effect is Allow
    pub fn is_allowed(&self) -> bool {
        matches!(self, Self::Allow)
    }
}

// ============================================================================
// Access Request
// ============================================================================

/// Access request containing all information for authorization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessRequest {
    /// Subject requesting access
    pub subject: Subject,
    /// Resource being accessed
    pub resource: Resource,
    /// Action being performed
    pub action: Action,
    /// Context information
    pub context: AuthContext,
    /// Request timestamp
    pub requested_at: DateTime<Utc>,
}

impl AccessRequest {
    /// Create a new access request
    ///
    /// # Arguments
    /// * `subject` - Subject
    /// * `resource` - Resource
    /// * `action` - Action
    /// * `context` - Auth context
    ///
    /// # Returns
    /// New AccessRequest instance
    pub fn new(subject: Subject, resource: Resource, action: Action, context: AuthContext) -> Self {
        Self {
            subject,
            resource,
            action,
            context,
            requested_at: Utc::now(),
        }
    }

    /// Validate the access request
    ///
    /// # Returns
    /// Result indicating if request is valid or error
    pub fn validate(&self) -> RbacResult<()> {
        self.subject.validate()?;
        self.resource.validate()?;
        self.action.validate()?;
        Ok(())
    }
}

// ============================================================================
// Access Decision
// ============================================================================

/// Access decision resulting from policy evaluation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AccessDecision {
    /// Effect of the decision
    pub effect: Effect,
    /// Reason for the decision
    pub reason: String,
    /// Policies that matched
    pub matched_policies: Vec<String>,
    /// Decision timestamp
    pub decided_at: DateTime<Utc>,
    /// Caching duration (None for no cache)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_ttl: Option<u64>,
}

impl AccessDecision {
    /// Create a new access decision
    ///
    /// # Arguments
    /// * `effect` - Effect
    /// * `reason` - Reason for decision
    ///
    /// # Returns
    /// New AccessDecision instance
    pub fn new(effect: Effect, reason: &str) -> Self {
        Self {
            effect,
            reason: reason.to_string(),
            matched_policies: Vec::new(),
            decided_at: Utc::now(),
            cache_ttl: None,
        }
    }

    /// Add a matched policy
    ///
    /// # Arguments
    /// * `policy_id` - Policy ID
    ///
    /// # Returns
    /// Self for method chaining
    pub fn with_matched_policy(mut self, policy_id: &str) -> Self {
        self.matched_policies.push(policy_id.to_string());
        self
    }

    /// Add multiple matched policies
    ///
    /// # Arguments
    /// * `policy_ids` - Slice of policy IDs
    ///
    /// # Returns
    /// Self for method chaining
    pub fn with_matched_policies(mut self, policy_ids: &[String]) -> Self {
        for policy_id in policy_ids {
            self.matched_policies.push(policy_id.clone());
        }
        self
    }

    /// Set cache TTL
    ///
    /// # Arguments
    /// * `ttl` - Cache TTL in seconds
    ///
    /// # Returns
    /// Self for method chaining
    pub fn with_cache_ttl(mut self, ttl: u64) -> Self {
        self.cache_ttl = Some(ttl);
        self
    }

    /// Check if access is allowed
    ///
    /// # Returns
    /// True if access is allowed
    pub fn is_allowed(&self) -> bool {
        self.effect.is_allowed()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_subject_creation() {
        let subject = Subject::new("user", "user123");
        assert_eq!(subject.subject_type, "user");
        assert_eq!(subject.subject_id, "user123");
    }

    #[test]
    fn test_subject_validation() {
        let valid_subject = Subject::new("user", "user123");
        assert!(valid_subject.validate().is_ok());

        let invalid_subject = Subject::new("", "user123");
        assert!(invalid_subject.validate().is_err());
    }

    #[test]
    fn test_resource_creation() {
        let resource = Resource::new("document", "doc456");
        assert_eq!(resource.resource_type, "document");
        assert_eq!(resource.resource_id, "doc456");
    }

    #[test]
    fn test_auth_context() {
        let user_id = UserId::new();
        let session_id = SessionId::new();
        let context = AuthContext::new(user_id, session_id)
            .with_role("admin")
            .with_attribute("ip", "127.0.0.1");

        assert_eq!(context.user_id, user_id);
        assert_eq!(context.session_id, session_id);
        assert!(context.has_role("admin"));
    }

    #[test]
    fn test_effect_combine() {
        assert_eq!(Effect::Allow.combine(Effect::Allow), Effect::Allow);
        assert_eq!(Effect::Allow.combine(Effect::Deny), Effect::Deny);
        assert_eq!(Effect::Deny.combine(Effect::Allow), Effect::Deny);
        assert_eq!(Effect::Deny.combine(Effect::Deny), Effect::Deny);
    }

    #[test]
    fn test_access_decision() {
        let decision = AccessDecision::new(Effect::Allow, "Policy matched")
            .with_matched_policy("policy1")
            .with_cache_ttl(60);

        assert!(decision.is_allowed());
        assert_eq!(decision.matched_policies.len(), 1);
        assert_eq!(decision.cache_ttl, Some(60));
    }
}
