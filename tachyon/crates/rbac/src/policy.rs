// Policy Evaluation Module
// Policy engine with RBAC and ABAC support, precedence, and conflict resolution

use crate::error::{RbacError, RbacResult};
use crate::types::{Action, Effect, Resource, Subject};
use dashmap::DashMap;
use regex::Regex;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

// ============================================================================
// Policy Type
// ============================================================================

/// Type of policy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PolicyType {
    /// Role-Based Access Control
    Rbac,
    /// Attribute-Based Access Control
    Abac,
    /// Hybrid policy (RBAC + ABAC)
    Hybrid,
}

// ============================================================================
// Policy Rule
// ============================================================================

/// A single rule within a policy
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PolicyRule {
    /// Rule ID
    pub id: String,
    /// Subject pattern
    pub subject_pattern: String,
    /// Resource pattern
    pub resource_pattern: String,
    /// Action pattern
    pub action_pattern: String,
    /// Rule effect
    pub effect: Effect,
    /// Rule priority (higher = more important)
    pub priority: i32,
    /// Rule conditions (for ABAC)
    pub conditions: HashMap<String, String>,
    /// Rule description
    pub description: String,
}

impl PolicyRule {
    /// Create a new policy rule
    ///
    /// # Arguments
    /// * `id` - Rule ID
    /// * `subject_pattern` - Subject pattern
    /// * `resource_pattern` - Resource pattern
    /// * `action_pattern` - Action pattern
    /// * `effect` - Rule effect
    ///
    /// # Returns
    /// New PolicyRule instance
    pub fn new(
        id: &str,
        subject_pattern: &str,
        resource_pattern: &str,
        action_pattern: &str,
        effect: Effect,
    ) -> Self {
        Self {
            id: id.to_string(),
            subject_pattern: subject_pattern.to_string(),
            resource_pattern: resource_pattern.to_string(),
            action_pattern: action_pattern.to_string(),
            effect,
            priority: 0,
            conditions: HashMap::new(),
            description: String::new(),
        }
    }

    /// Set rule priority
    ///
    /// # Arguments
    /// * `priority` - Priority value
    ///
    /// # Returns
    /// Self for method chaining
    pub fn with_priority(mut self, priority: i32) -> Self {
        self.priority = priority;
        self
    }

    /// Add a condition
    ///
    /// # Arguments
    /// * `key` - Condition key
    /// * `value` - Condition value
    ///
    /// # Returns
    /// Self for method chaining
    pub fn with_condition(mut self, key: &str, value: &str) -> Self {
        self.conditions.insert(key.to_string(), value.to_string());
        self
    }

    /// Set rule description
    ///
    /// # Arguments
    /// * `description` - Description
    ///
    /// # Returns
    /// Self for method chaining
    pub fn with_description(mut self, description: &str) -> Self {
        self.description = description.to_string();
        self
    }

    /// Check if rule matches a request
    ///
    /// # Arguments
    /// * `subject` - Subject
    /// * `resource` - Resource
    /// * `action` - Action
    ///
    /// # Returns
    /// True if rule matches
    pub fn matches(&self, subject: &Subject, resource: &Resource, action: &Action) -> bool {
        self.matches_subject(subject)
            && self.matches_resource(resource)
            && self.matches_action(action)
            && self.check_conditions(subject, resource, action)
    }

    /// Check if rule matches subject
    ///
    /// # Arguments
    /// * `subject` - Subject
    ///
    /// # Returns
    /// True if subject matches
    fn matches_subject(&self, subject: &Subject) -> bool {
        // Match against full subject string "type:id"
        match_pattern(&self.subject_pattern, &subject.to_string())
    }

    /// Check if rule matches resource
    ///
    /// # Arguments
    /// * `resource` - Resource
    ///
    /// # Returns
    /// True if resource matches
    fn matches_resource(&self, resource: &Resource) -> bool {
        // Match against full resource string "type:id"
        match_pattern(&self.resource_pattern, &resource.to_string())
    }

    /// Check if rule matches action
    ///
    /// # Arguments
    /// * `action` - Action
    ///
    /// # Returns
    /// True if action matches
    fn matches_action(&self, action: &Action) -> bool {
        match_pattern(&self.action_pattern, &action.action_name)
    }

    /// Check rule conditions
    ///
    /// # Arguments
    /// * `subject` - Subject
    /// * `resource` - Resource
    /// * `action` - Action
    ///
    /// # Returns
    /// True if all conditions are met
    fn check_conditions(&self, subject: &Subject, resource: &Resource, action: &Action) -> bool {
        for (key, expected_value) in &self.conditions {
            let actual_value = match key.as_str() {
                "subject.type" => Some(subject.subject_type.as_str()),
                "subject.id" => Some(subject.subject_id.as_str()),
                "resource.type" => Some(resource.resource_type.as_str()),
                "resource.id" => Some(resource.resource_id.as_str()),
                "action.name" => Some(action.action_name.as_str()),
                "action.scope" => action.scope.as_ref().map(|s| s.as_str()),
                _ => subject
                    .get_attribute(key)
                    .or_else(|| resource.get_attribute(key))
                    .map(|x| x.as_str()),
            };

            match actual_value {
                Some(value) if value == expected_value => continue,
                Some(_) => return false,
                None => return false,
            }
        }
        true
    }
}

/// Match a pattern against a string (supports wildcards)
///
/// # Arguments
/// * `pattern` - Pattern to match (supports * wildcard)
/// * `value` - Value to match against
///
/// # Returns
/// True if pattern matches value
fn match_pattern(pattern: &str, value: &str) -> bool {
    if pattern == "*" {
        return true;
    }

    if pattern == value {
        return true;
    }

    // Handle wildcard pattern
    if pattern.contains('*') {
        let regex_str = pattern.replace('*', ".*");
        if let Ok(re) = Regex::new(&regex_str) {
            return re.is_match(value);
        }
    }

    false
}

// ============================================================================
// Policy
// ============================================================================

/// Represents a policy with rules
#[derive(Debug, Clone)]
pub struct Policy {
    /// Policy ID
    pub id: String,
    /// Policy name
    pub name: String,
    /// Policy type
    pub policy_type: PolicyType,
    /// Policy rules
    pub rules: Vec<PolicyRule>,
    /// Policy description
    pub description: String,
    /// Whether policy is enabled
    pub enabled: bool,
}

impl Policy {
    /// Create a new policy
    ///
    /// # Arguments
    /// * `id` - Policy ID
    /// * `name` - Policy name
    /// * `policy_type` - Policy type
    ///
    /// # Returns
    /// New Policy instance
    pub fn new(id: &str, name: &str, policy_type: PolicyType) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            policy_type,
            rules: Vec::new(),
            description: String::new(),
            enabled: true,
        }
    }

    /// Add a rule to the policy
    ///
    /// # Arguments
    /// * `rule` - Rule to add
    ///
    /// # Returns
    /// Self for method chaining
    pub fn add_rule(mut self, rule: PolicyRule) -> Self {
        self.rules.push(rule);
        self
    }

    /// Set policy description
    ///
    /// # Arguments
    /// * `description` - Description
    ///
    /// # Returns
    /// Self for method chaining
    pub fn with_description(mut self, description: &str) -> Self {
        self.description = description.to_string();
        self
    }

    /// Enable the policy
    ///
    /// # Returns
    /// Self for method chaining
    pub fn enable(mut self) -> Self {
        self.enabled = true;
        self
    }

    /// Disable the policy
    ///
    /// # Returns
    /// Self for method chaining
    pub fn disable(mut self) -> Self {
        self.enabled = false;
        self
    }

    /// Get matching rules
    ///
    /// # Arguments
    /// * `subject` - Subject
    /// * `resource` - Resource
    /// * `action` - Action
    ///
    /// # Returns
    /// Vector of matching rules
    pub fn get_matching_rules(
        &self,
        subject: &Subject,
        resource: &Resource,
        action: &Action,
    ) -> Vec<&PolicyRule> {
        self.rules
            .iter()
            .filter(|rule| rule.matches(subject, resource, action))
            .collect()
    }
}

// ============================================================================
// Policy Engine
// ============================================================================

/// Policy engine for evaluating access control policies
#[derive(Debug)]
pub struct PolicyEngine {
    /// Available policies
    policies: DashMap<String, Policy>,
    /// Policy cache
    cache: DashMap<String, Vec<String>>,
    /// Maximum cache size
    #[allow(dead_code)]
    max_cache_size: usize,
    /// Current cache size
    current_cache_size: Arc<RwLock<usize>>,
}

impl PolicyEngine {
    /// Create a new policy engine
    ///
    /// # Returns
    /// New PolicyEngine instance
    pub fn new() -> Self {
        Self {
            policies: DashMap::new(),
            cache: DashMap::new(),
            max_cache_size: 1000,
            current_cache_size: Arc::new(RwLock::new(0)),
        }
    }

    /// Create a new policy engine with custom cache size
    ///
    /// # Arguments
    /// * `cache_size` - Cache size
    ///
    /// # Returns
    /// New PolicyEngine instance
    pub fn with_cache_size(cache_size: usize) -> Self {
        Self {
            policies: DashMap::new(),
            cache: DashMap::new(),
            max_cache_size: cache_size,
            current_cache_size: Arc::new(RwLock::new(0)),
        }
    }

    /// Add a policy
    ///
    /// # Arguments
    /// * `policy` - Policy to add
    pub fn add_policy(&self, policy: Policy) {
        self.policies.insert(policy.id.clone(), policy);
        self.invalidate_cache();
    }

    /// Add a policy without invalidating the cache.
    ///
    /// Use this during initialization when the cache is known to be empty
    /// and calling `invalidate_cache()` would panic inside an async runtime.
    pub fn add_policy_no_invalidate(&self, policy: Policy) {
        self.policies.insert(policy.id.clone(), policy);
    }

    /// Get a policy by ID
    ///
    /// # Arguments
    /// * `policy_id` - Policy ID
    ///
    /// # Returns
    /// Option containing the policy
    pub fn get_policy(&self, policy_id: &str) -> Option<Policy> {
        self.policies.get(policy_id).map(|p| p.clone())
    }

    /// Remove a policy
    ///
    /// # Arguments
    /// * `policy_id` - Policy ID
    ///
    /// # Returns
    /// Result indicating success or error
    pub fn remove_policy(&self, policy_id: &str) -> RbacResult<()> {
        if self.policies.remove(policy_id).is_some() {
            self.invalidate_cache();
            Ok(())
        } else {
            Err(RbacError::not_found(format!(
                "Policy not found: {}",
                policy_id
            )))
        }
    }

    /// Evaluate policies for an access request
    ///
    /// # Arguments
    /// * `subject` - Subject
    /// * `resource` - Resource
    /// * `action` - Action
    ///
    /// # Returns
    /// Result containing the effect and matched policy IDs
    pub fn evaluate(
        &self,
        subject: &Subject,
        resource: &Resource,
        action: &Action,
    ) -> RbacResult<(Effect, Vec<String>)> {
        // Collect matching rules with (policy_id, priority, effect) - owned data to avoid lifetime issues
        let mut matching_rules: Vec<(String, i32, Effect)> = Vec::new();

        for policy_entry in self.policies.iter() {
            let policy = policy_entry.value();
            if !policy.enabled {
                continue;
            }

            for rule in &policy.rules {
                if rule.matches(subject, resource, action) {
                    matching_rules.push((policy.id.clone(), rule.priority, rule.effect));
                }
            }
        }

        // If no matching rules, default deny
        if matching_rules.is_empty() {
            return Ok((Effect::Deny, Vec::new()));
        }

        // Find the highest priority among all matching rules
        let highest_priority = matching_rules
            .iter()
            .map(|(_, priority, _)| *priority)
            .max()
            .unwrap_or(0);

        // Get rules at the highest priority level
        let highest_priority_rules: Vec<(String, Effect)> = matching_rules
            .into_iter()
            .filter(|(_, priority, _)| *priority == highest_priority)
            .map(|(policy_id, _, effect)| (policy_id, effect))
            .collect();

        // Apply conflict resolution: deny takes precedence among rules at same priority
        let mut final_effect = Effect::Allow;
        let mut matched_policies: Vec<String> = Vec::new();

        for (policy_id, effect) in &highest_priority_rules {
            if *effect == Effect::Deny {
                final_effect = Effect::Deny;
            }
            if !matched_policies.contains(policy_id) {
                matched_policies.push(policy_id.clone());
            }
        }

        Ok((final_effect, matched_policies))
    }

    /// Evaluate policies with caching
    ///
    /// # Arguments
    /// * `subject` - Subject
    /// * `resource` - Resource
    /// * `action` - Action
    ///
    /// # Returns
    /// Result containing the effect and matched policy IDs
    pub fn evaluate_cached(
        &self,
        subject: &Subject,
        resource: &Resource,
        action: &Action,
    ) -> RbacResult<(Effect, Vec<String>)> {
        let cache_key = self.generate_cache_key(subject, resource, action);

        if let Some(cached) = self.cache.get(&cache_key) {
            // Determine effect from cached policies
            let mut effect = Effect::Deny;
            for policy_id in cached.value() {
                if let Some(policy) = self.get_policy(policy_id) {
                    for rule in &policy.rules {
                        if rule.matches(subject, resource, action) && rule.effect == Effect::Allow {
                            effect = Effect::Allow;
                            break;
                        }
                    }
                }
            }
            return Ok((effect, cached.value().clone()));
        }

        let (effect, policy_ids) = self.evaluate(subject, resource, action)?;
        self.cache.insert(cache_key, policy_ids.clone());

        Ok((effect, policy_ids))
    }

    /// Generate cache key
    ///
    /// # Arguments
    /// * `subject` - Subject
    /// * `resource` - Resource
    /// * `action` - Action
    ///
    /// # Returns
    /// Cache key string
    fn generate_cache_key(
        &self,
        subject: &Subject,
        resource: &Resource,
        action: &Action,
    ) -> String {
        format!(
            "{}:{}:{}:{}",
            subject, resource.resource_type, resource.resource_id, action.action_name
        )
    }

    /// Invalidate cache
    pub fn invalidate_cache(&self) {
        self.cache.clear();
        let mut size_guard = self.current_cache_size.blocking_write();
        *size_guard = 0;
    }

    /// Invalidate cache for a specific subject
    ///
    /// # Arguments
    /// * `subject` - Subject
    pub fn invalidate_subject_cache(&self, subject: &Subject) {
        let prefix = format!("{}:", subject);
        self.cache.retain(|k, _| !k.starts_with(&prefix));
    }

    /// Invalidate cache for a specific resource
    ///
    /// # Arguments
    /// * `resource` - Resource
    pub fn invalidate_resource_cache(&self, resource: &Resource) {
        let prefix = format!("{}:{}:", resource.resource_type, resource.resource_id);
        self.cache.retain(|k, _| !k.contains(&prefix));
    }

    /// Get all policies
    ///
    /// # Returns
    /// Vector of all policies
    pub fn get_all_policies(&self) -> Vec<Policy> {
        self.policies.iter().map(|p| p.value().clone()).collect()
    }

    /// Get policies by type
    ///
    /// # Arguments
    /// * `policy_type` - Policy type
    ///
    /// # Returns
    /// Vector of policies of the specified type
    pub fn get_policies_by_type(&self, policy_type: PolicyType) -> Vec<Policy> {
        self.policies
            .iter()
            .filter(|p| p.value().policy_type == policy_type)
            .map(|p| p.value().clone())
            .collect()
    }

    /// Get cache statistics
    ///
    /// # Returns
    /// Current cache size
    pub fn cache_size(&self) -> usize {
        *self.current_cache_size.blocking_read()
    }
}

impl Default for PolicyEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_policy_rule_creation() {
        let rule = PolicyRule::new("rule1", "user:*", "document:*", "read", Effect::Allow)
            .with_priority(10)
            .with_description("Allow users to read documents");

        assert_eq!(rule.id, "rule1");
        assert_eq!(rule.effect, Effect::Allow);
        assert_eq!(rule.priority, 10);
    }

    #[test]
    fn test_policy_creation() {
        let policy = Policy::new("policy1", "Document Access", PolicyType::Rbac)
            .add_rule(PolicyRule::new(
                "rule1",
                "user:*",
                "document:*",
                "read",
                Effect::Allow,
            ))
            .with_description("Document access policy");

        assert_eq!(policy.id, "policy1");
        assert_eq!(policy.policy_type, PolicyType::Rbac);
        assert_eq!(policy.rules.len(), 1);
    }

    #[test]
    fn test_policy_engine() {
        let engine = PolicyEngine::new();

        let policy = Policy::new("policy1", "Document Access", PolicyType::Rbac)
            .add_rule(PolicyRule::new(
                "rule1",
                "user:*",
                "document:*",
                "read",
                Effect::Allow,
            ))
            .add_rule(PolicyRule::new(
                "rule2",
                "user:*",
                "document:*",
                "write",
                Effect::Deny,
            ));

        engine.add_policy(policy);

        let subject = Subject::new("user", "user123");
        let resource = Resource::new("document", "doc123");
        let action_read = Action::new("read");
        let action_write = Action::new("write");

        let (effect_read, _) = engine.evaluate(&subject, &resource, &action_read).unwrap();
        assert_eq!(effect_read, Effect::Allow);

        let (effect_write, _) = engine.evaluate(&subject, &resource, &action_write).unwrap();
        assert_eq!(effect_write, Effect::Deny);
    }

    #[test]
    #[test]
    fn test_policy_precedence() {
        let engine = PolicyEngine::new();

        let policy = Policy::new("policy1", "Precedence Test", PolicyType::Rbac)
            .add_rule(
                PolicyRule::new("rule1", "user:*", "document:*", "read", Effect::Deny)
                    .with_priority(5),
            )
            .add_rule(
                PolicyRule::new("rule2", "user:*", "document:*", "read", Effect::Allow)
                    .with_priority(10),
            );

        engine.add_policy(policy);

        let subject = Subject::new("user", "user123");
        let resource = Resource::new("document", "doc123");
        let action = Action::new("read");

        let (effect, _) = engine.evaluate(&subject, &resource, &action).unwrap();
        assert_eq!(effect, Effect::Allow);
    }
}
