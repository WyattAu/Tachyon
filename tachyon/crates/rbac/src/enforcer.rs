// Enforcer Module
// Centralized enforcer for authorization decisions with caching and audit logging

use crate::cache::AuthorizationCache;
use crate::error::RbacResult;
use crate::permission::PermissionChecker;
use crate::policy::{Policy, PolicyEngine, PolicyRule, PolicyType};
use crate::session::SessionManager;
use crate::types::{AccessDecision, AccessRequest, Effect, Resource, Subject};
#[cfg(test)]
use crate::types::{Action, AuthContext};
use crate::SessionId;
#[cfg(test)]
use crate::UserId;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use tracing::{debug, error, info, instrument, warn};

// ============================================================================
// Enforcer Configuration
// ============================================================================

/// Configuration for the enforcer
#[derive(Debug, Clone)]
pub struct EnforcerConfig {
    /// Default cache TTL in seconds
    pub default_cache_ttl: u64,
    /// Maximum cache size
    pub max_cache_size: usize,
    /// Enable audit logging
    pub enable_audit_logging: bool,
    /// Enable caching
    pub enable_caching: bool,
}

impl Default for EnforcerConfig {
    fn default() -> Self {
        Self {
            default_cache_ttl: 300, // 5 minutes
            max_cache_size: 10000,
            enable_audit_logging: true,
            enable_caching: true,
        }
    }
}

// ============================================================================
// Enforcer
// ============================================================================

/// Centralized enforcer for authorization decisions
pub struct Enforcer {
    /// Permission checker
    permission_checker: PermissionChecker,
    /// Policy engine
    policy_engine: PolicyEngine,
    /// Session manager
    session_manager: Option<SessionManager>,
    /// Authorization cache
    cache: AuthorizationCache,
    /// Enforcer configuration
    config: EnforcerConfig,
    /// Audit log entries
    audit_log: Vec<AuditEntry>,
}

/// Audit log entry for authorization events
#[allow(dead_code, private_interfaces)]
#[derive(Debug, Clone)]
struct AuditEntry {
    /// Timestamp
    timestamp: DateTime<Utc>,
    /// Subject
    subject: String,
    /// Resource
    resource: String,
    /// Action
    action: String,
    /// Decision
    decision: String,
    /// Reason
    reason: String,
    /// Session ID
    session_id: Option<String>,
}

impl Enforcer {
    /// Create a new enforcer
    ///
    /// # Returns
    /// New Enforcer instance
    pub fn new() -> Self {
        let config = EnforcerConfig::default();
        let cache = AuthorizationCache::new(config.max_cache_size);

        let enforcer = Self {
            permission_checker: PermissionChecker::with_cache_size(1000),
            policy_engine: PolicyEngine::with_cache_size(1000),
            session_manager: None,
            cache,
            config,
            audit_log: Vec::new(),
        };
        enforcer.seed_default_policies();
        enforcer
    }

    /// Create a new enforcer with custom configuration
    ///
    /// # Arguments
    /// * `config` - Enforcer configuration
    ///
    /// # Returns
    /// New Enforcer instance
    pub fn with_config(config: EnforcerConfig) -> Self {
        let cache = AuthorizationCache::new(config.max_cache_size);

        let enforcer = Self {
            permission_checker: PermissionChecker::with_cache_size(1000),
            policy_engine: PolicyEngine::with_cache_size(1000),
            session_manager: None,
            cache,
            config,
            audit_log: Vec::new(),
        };
        enforcer.seed_default_policies();
        enforcer
    }

    /// Create a new enforcer with session management
    ///
    /// # Arguments
    /// * `session_manager` - Session manager
    /// * `config` - Enforcer configuration
    ///
    /// # Returns
    /// New Enforcer instance
    pub fn with_session_manager(session_manager: SessionManager, config: EnforcerConfig) -> Self {
        let cache = AuthorizationCache::new(config.max_cache_size);

        let enforcer = Self {
            permission_checker: PermissionChecker::with_cache_size(1000),
            policy_engine: PolicyEngine::with_cache_size(1000),
            session_manager: Some(session_manager),
            cache,
            config,
            audit_log: Vec::new(),
        };
        enforcer.seed_default_policies();
        enforcer
    }

    /// Check authorization for a request
    ///
    /// # Arguments
    /// * `request` - Access request
    ///
    /// # Returns
    /// Result containing the access decision or error
    #[instrument(skip_all)]
    pub fn authorize(&mut self, request: &AccessRequest) -> RbacResult<AccessDecision> {
        request.validate()?;

        // Check cache first
        let cache_key = self.generate_cache_key(request);
        if self.config.enable_caching {
            if let Some(cached) = self.cache.get(&cache_key) {
                debug!("Authorization cache hit for {}", cache_key);
                return Ok(cached);
            }
        }

        // Validate session if session manager is available
        if let Some(ref _session_manager) = self.session_manager {
            // Note: This is a synchronous wrapper around async session validation
            // In production, you'd use a different approach or make the enforcer async
        }

        // Evaluate policies
        let (policy_effect, matched_policies) = self.policy_engine.evaluate_cached(
            &request.subject,
            &request.resource,
            &request.action,
        )?;

        // Check permissions
        let permission_allowed = self.permission_checker.check_permission_uncached(
            &request.subject,
            &request.resource,
            &request.action,
        )?;

        // Combine decisions (deny takes precedence)
        let final_effect = if policy_effect == Effect::Deny || !permission_allowed {
            Effect::Deny
        } else {
            Effect::Allow
        };

        let reason = if final_effect == Effect::Allow {
            "Policy and permission check passed".to_string()
        } else {
            format!(
                "Access denied: policy_effect={:?}, permission_allowed={}",
                policy_effect, permission_allowed
            )
        };

        let decision = AccessDecision::new(final_effect, &reason)
            .with_matched_policies(matched_policies.as_slice())
            .with_cache_ttl(self.config.default_cache_ttl);

        // Cache the decision
        if self.config.enable_caching {
            self.cache.insert(cache_key, decision.clone());
        }

        // Log audit entry
        if self.config.enable_audit_logging {
            self.log_audit(request, &decision);
        }

        Ok(decision)
    }

    /// Check authorization for a request (async version)
    ///
    /// # Arguments
    /// * `request` - Access request
    ///
    /// # Returns
    /// Result containing the access decision or error
    pub async fn authorize_async(&mut self, request: &AccessRequest) -> RbacResult<AccessDecision> {
        request.validate()?;

        // Check cache first
        let cache_key = self.generate_cache_key(request);
        if self.config.enable_caching {
            if let Some(cached) = self.cache.get(&cache_key) {
                debug!("Authorization cache hit for {}", cache_key);
                return Ok(cached);
            }
        }

        // Validate session if session manager is available
        if let Some(ref session_manager) = self.session_manager {
            if let Some(session_id) = &request.context.attributes.get("session_id") {
                if let Ok(parsed_session_id) = SessionId::parse_str(session_id) {
                    session_manager
                        .validate_session(&parsed_session_id)
                        .await
                        .map_err(|e| {
                            error!("Session validation failed: {}", e);
                            e
                        })?;
                }
            }
        }

        // Evaluate policies
        let (policy_effect, matched_policies) = self.policy_engine.evaluate_cached(
            &request.subject,
            &request.resource,
            &request.action,
        )?;

        // Check permissions
        let permission_allowed = self.permission_checker.check_permission_uncached(
            &request.subject,
            &request.resource,
            &request.action,
        )?;

        // Combine decisions (deny takes precedence)
        let final_effect = if policy_effect == Effect::Deny || !permission_allowed {
            Effect::Deny
        } else {
            Effect::Allow
        };

        let reason = if final_effect == Effect::Allow {
            "Policy and permission check passed".to_string()
        } else {
            format!(
                "Access denied: policy_effect={:?}, permission_allowed={}",
                policy_effect, permission_allowed
            )
        };

        let decision = AccessDecision::new(final_effect, &reason)
            .with_matched_policies(matched_policies.as_slice())
            .with_cache_ttl(self.config.default_cache_ttl);

        // Cache the decision
        if self.config.enable_caching {
            self.cache.insert(cache_key, decision.clone());
        }

        // Log audit entry
        if self.config.enable_audit_logging {
            self.log_audit(request, &decision);
        }

        Ok(decision)
    }

    /// Generate cache key for a request
    ///
    /// # Arguments
    /// * `request` - Access request
    ///
    /// # Returns
    /// Cache key string
    fn generate_cache_key(&self, request: &AccessRequest) -> String {
        format!(
            "auth:{}:{}:{}:{}",
            request.subject,
            request.resource.resource_type,
            request.resource.resource_id,
            request.action.action_name
        )
    }

    /// Log audit entry
    ///
    /// # Arguments
    /// * `request` - Access request
    /// * `decision` - Access decision
    fn log_audit(&mut self, request: &AccessRequest, decision: &AccessDecision) {
        let entry = AuditEntry {
            timestamp: Utc::now(),
            subject: request.subject.to_string(),
            resource: request.resource.to_string(),
            action: request.action.action_name.clone(),
            decision: format!("{:?}", decision.effect),
            reason: decision.reason.clone(),
            session_id: request.context.attributes.get("session_id").cloned(),
        };

        self.audit_log.push(entry.clone());

        if decision.effect == Effect::Allow {
            info!(
                "Authorization allowed: {} -> {}:{}",
                request.subject, request.resource.resource_type, request.action.action_name
            );
        } else {
            warn!(
                "Authorization denied: {} -> {}:{}",
                request.subject, request.resource.resource_type, request.action.action_name
            );
        }
    }

    /// Get audit log entries
    ///
    /// # Arguments
    /// * `limit` - Maximum number of entries to return
    ///
    /// # Returns
    /// Vector of audit entries
    #[allow(private_interfaces)]
    pub fn get_audit_log(&self, limit: Option<usize>) -> Vec<AuditEntry> {
        let limit = limit.unwrap_or(self.audit_log.len());
        self.audit_log.iter().rev().take(limit).cloned().collect()
    }

    /// Clear audit log
    pub fn clear_audit_log(&mut self) {
        self.audit_log.clear();
    }

    /// Invalidate cache for a subject
    ///
    /// # Arguments
    /// * `subject` - Subject
    pub fn invalidate_subject_cache(&mut self, subject: &Subject) {
        self.permission_checker.invalidate_subject_cache(subject);
        self.policy_engine.invalidate_subject_cache(subject);
        self.cache.invalidate_subject(subject);
    }

    /// Invalidate cache for a resource
    ///
    /// # Arguments
    /// * `resource` - Resource
    pub fn invalidate_resource_cache(&mut self, resource: &Resource) {
        self.permission_checker.invalidate_resource_cache(resource);
        self.policy_engine.invalidate_resource_cache(resource);
        self.cache.invalidate_resource(resource);
    }

    /// Clear all caches
    pub fn clear_caches(&mut self) {
        self.permission_checker.clear_cache();
        self.policy_engine.invalidate_cache();
        self.cache.clear();
    }

    /// Get cache statistics
    ///
    /// # Returns
    /// HashMap of cache statistics
    pub fn get_cache_stats(&self) -> HashMap<String, usize> {
        let mut stats = HashMap::new();
        stats.insert("authorization_cache".to_string(), self.cache.size());
        stats.insert(
            "permission_cache".to_string(),
            self.permission_checker.cache_size(),
        );
        stats.insert("policy_cache".to_string(), self.policy_engine.cache_size());
        stats
    }

    /// Set permission checker
    ///
    /// # Arguments
    /// * `checker` - Permission checker
    pub fn set_permission_checker(&mut self, checker: PermissionChecker) {
        self.permission_checker = checker;
    }

    /// Set policy engine
    ///
    /// # Arguments
    /// * `engine` - Policy engine
    pub fn set_policy_engine(&mut self, engine: PolicyEngine) {
        self.policy_engine = engine;
    }

    /// Set session manager
    ///
    /// # Arguments
    /// * `manager` - Session manager
    pub fn set_session_manager(&mut self, manager: SessionManager) {
        self.session_manager = Some(manager);
    }

    /// Get permission checker reference
    ///
    /// # Returns
    /// Reference to permission checker
    pub fn permission_checker(&self) -> &PermissionChecker {
        &self.permission_checker
    }

    /// Get policy engine reference
    ///
    /// # Returns
    /// Reference to policy engine
    pub fn policy_engine(&self) -> &PolicyEngine {
        &self.policy_engine
    }

    fn seed_default_policies(&self) {
        let admin_policy = Policy::new("default-admin", "Admin Full Access", PolicyType::Rbac)
            .add_rule(PolicyRule::new(
                "admin-all",
                "role:admin",
                "*",
                "*",
                Effect::Allow,
            ));

        let editor_policy = Policy::new("default-editor", "Editor Access", PolicyType::Rbac)
            .add_rule(PolicyRule::new("editor-doc-read", "role:editor", "document:*", "read", Effect::Allow))
            .add_rule(PolicyRule::new("editor-doc-write", "role:editor", "document:*", "write", Effect::Allow))
            .add_rule(PolicyRule::new("editor-doc-edit", "role:editor", "document:*", "edit", Effect::Allow))
            .add_rule(PolicyRule::new("editor-doc-delete", "role:editor", "document:*", "delete", Effect::Allow))
            .add_rule(PolicyRule::new("editor-space-read", "role:editor", "space:*", "read", Effect::Allow))
            .add_rule(PolicyRule::new("editor-space-write", "role:editor", "space:*", "write", Effect::Allow))
            .add_rule(PolicyRule::new("editor-node-read", "role:editor", "node:*", "read", Effect::Allow))
            .add_rule(PolicyRule::new("editor-node-write", "role:editor", "node:*", "write", Effect::Allow))
            .add_rule(PolicyRule::new("editor-node-edit", "role:editor", "node:*", "edit", Effect::Allow))
            .add_rule(PolicyRule::new("editor-node-delete", "role:editor", "node:*", "delete", Effect::Allow))
            .add_rule(PolicyRule::new("editor-search-read", "role:editor", "search:*", "read", Effect::Allow));

        let writer_policy = Policy::new("default-writer", "Writer Access", PolicyType::Rbac)
            .add_rule(PolicyRule::new("writer-doc-read", "role:writer", "document:*", "read", Effect::Allow))
            .add_rule(PolicyRule::new("writer-doc-write", "role:writer", "document:*", "write", Effect::Allow))
            .add_rule(PolicyRule::new("writer-space-read", "role:writer", "space:*", "read", Effect::Allow))
            .add_rule(PolicyRule::new("writer-node-read", "role:writer", "node:*", "read", Effect::Allow))
            .add_rule(PolicyRule::new("writer-node-write", "role:writer", "node:*", "write", Effect::Allow))
            .add_rule(PolicyRule::new("writer-search-read", "role:writer", "search:*", "read", Effect::Allow));

        let reader_policy = Policy::new("default-reader", "Reader Access", PolicyType::Rbac)
            .add_rule(PolicyRule::new("reader-doc-read", "role:reader", "document:*", "read", Effect::Allow))
            .add_rule(PolicyRule::new("reader-space-read", "role:reader", "space:*", "read", Effect::Allow))
            .add_rule(PolicyRule::new("reader-node-read", "role:reader", "node:*", "read", Effect::Allow))
            .add_rule(PolicyRule::new("reader-search-read", "role:reader", "search:*", "read", Effect::Allow));

        // Add policies directly to the engine's internal map to avoid
        // blocking_write() on the cache (which panics inside async runtime).
        // The cache is empty at this point so invalidation is unnecessary.
        let policies = [
            admin_policy,
            editor_policy,
            writer_policy,
            reader_policy,
        ];
        for policy in policies {
            self.policy_engine.add_policy_no_invalidate(policy);
        }
    }
}

impl Default for Enforcer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_enforcer_creation() {
        let enforcer = Enforcer::new();
        assert!(enforcer.session_manager.is_none());
    }

    #[test]
    fn test_enforcer_authorize() {
        let mut enforcer = Enforcer::new();

        // Add a policy
        let policy =
            crate::policy::Policy::new("policy1", "Test Policy", crate::policy::PolicyType::Rbac)
                .add_rule(crate::policy::PolicyRule::new(
                    "rule1",
                    "user:*",
                    "document:*",
                    "read",
                    Effect::Allow,
                ));

        enforcer.policy_engine.add_policy(policy);

        // Create a request
        let subject = Subject::new("user", "user123");
        let resource = Resource::new("document", "doc123");
        let action = Action::new("read");
        let context = AuthContext::new(UserId::new(), SessionId::new());
        let request =
            AccessRequest::new(subject.clone(), resource.clone(), action.clone(), context);

        // Authorize
        let decision = enforcer.authorize(&request).unwrap();
        assert!(decision.is_allowed());
    }

    #[test]
    fn test_enforcer_cache() {
        let mut enforcer = Enforcer::new();

        // Add a policy
        let policy =
            crate::policy::Policy::new("policy1", "Test Policy", crate::policy::PolicyType::Rbac)
                .add_rule(crate::policy::PolicyRule::new(
                    "rule1",
                    "user:*",
                    "document:*",
                    "read",
                    Effect::Allow,
                ));

        enforcer.policy_engine.add_policy(policy);

        // Create a request
        let subject = Subject::new("user", "user123");
        let resource = Resource::new("document", "doc123");
        let action = Action::new("read");
        let context = AuthContext::new(UserId::new(), SessionId::new());
        let request =
            AccessRequest::new(subject.clone(), resource.clone(), action.clone(), context);

        // Authorize twice - second should be cached
        let decision1 = enforcer.authorize(&request).unwrap();
        let decision2 = enforcer.authorize(&request).unwrap();

        assert!(decision1.is_allowed());
        assert!(decision2.is_allowed());

        let stats = enforcer.get_cache_stats();
        assert_eq!(stats.get("authorization_cache"), Some(&1));
    }

    #[test]
    fn test_audit_logging() {
        let mut enforcer = Enforcer::new();

        // Add a policy
        let policy =
            crate::policy::Policy::new("policy1", "Test Policy", crate::policy::PolicyType::Rbac)
                .add_rule(crate::policy::PolicyRule::new(
                    "rule1",
                    "user:*",
                    "document:*",
                    "read",
                    Effect::Allow,
                ));

        enforcer.policy_engine.add_policy(policy);

        // Create a request
        let subject = Subject::new("user", "user123");
        let resource = Resource::new("document", "doc123");
        let action = Action::new("read");
        let context = AuthContext::new(UserId::new(), SessionId::new());
        let request =
            AccessRequest::new(subject.clone(), resource.clone(), action.clone(), context);

        // Authorize
        enforcer.authorize(&request).unwrap();

        // Check audit log
        let audit_log = enforcer.get_audit_log(Some(10));
        assert_eq!(audit_log.len(), 1);
        assert_eq!(audit_log[0].decision, "Allow");
    }
}
