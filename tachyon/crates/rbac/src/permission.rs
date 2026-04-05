// Permission Checking Module
// Fine-grained permission system with inheritance, composition, and caching

use crate::error::{RbacError, RbacResult};
use crate::types::{Action, Effect, Resource, Subject};
use dashmap::DashMap;
use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;

// ============================================================================
// Permission
// ============================================================================

/// Represents a permission with resource and action constraints
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Permission {
    /// Permission identifier
    pub id: String,
    /// Resource pattern (supports wildcards)
    pub resource_pattern: String,
    /// Action pattern (supports wildcards)
    pub action_pattern: String,
    /// Permission effect (allow/deny)
    pub effect: Effect,
    /// Additional conditions
    pub conditions: HashMap<String, String>,
    /// Priority (higher = more important)
    pub priority: i32,
    /// Permission description
    pub description: String,
}

impl Permission {
    /// Create a new permission
    ///
    /// # Arguments
    /// * `id` - Permission identifier
    /// * `resource_pattern` - Resource pattern
    /// * `action_pattern` - Action pattern
    /// * `effect` - Permission effect
    ///
    /// # Returns
    /// New Permission instance
    pub fn new(id: &str, resource_pattern: &str, action_pattern: &str, effect: Effect) -> Self {
        Self {
            id: id.to_string(),
            resource_pattern: resource_pattern.to_string(),
            action_pattern: action_pattern.to_string(),
            effect,
            conditions: HashMap::new(),
            priority: 0,
            description: String::new(),
        }
    }

    /// Set permission description
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

    /// Set permission priority
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

    /// Check if permission matches a resource and action
    ///
    /// # Arguments
    /// * `resource` - Resource
    /// * `action` - Action
    ///
    /// # Returns
    /// True if permission matches
    pub fn matches(&self, resource: &Resource, action: &Action) -> bool {
        self.matches_resource(resource) && self.matches_action(action)
    }

    /// Check if permission matches a resource
    ///
    /// # Arguments
    /// * `resource` - Resource
    ///
    /// # Returns
    /// True if permission matches resource
    pub fn matches_resource(&self, resource: &Resource) -> bool {
        // Match against full resource string "type:id"
        match_pattern(&self.resource_pattern, &resource.to_string())
    }

    /// Check if permission matches an action
    ///
    /// # Arguments
    /// * `action` - Action
    ///
    /// # Returns
    /// True if permission matches action
    pub fn matches_action(&self, action: &Action) -> bool {
        match_pattern(&self.action_pattern, &action.action_name)
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
// Permission Inheritance
// ============================================================================

/// Permission inheritance graph
#[derive(Debug, Clone)]
pub struct PermissionInheritance {
    /// Parent-child relationships
    relationships: HashMap<String, Vec<String>>,
}

impl PermissionInheritance {
    /// Create a new permission inheritance graph
    ///
    /// # Returns
    /// New PermissionInheritance instance
    pub fn new() -> Self {
        Self {
            relationships: HashMap::new(),
        }
    }

    /// Add a parent-child relationship
    ///
    /// # Arguments
    /// * `parent` - Parent permission ID
    /// * `child` - Child permission ID
    pub fn add_relationship(&mut self, parent: &str, child: &str) {
        self.relationships
            .entry(parent.to_string())
            .or_insert_with(Vec::new)
            .push(child.to_string());
    }

    /// Get all transitive children of a permission
    ///
    /// # Arguments
    /// * `permission_id` - Permission ID
    ///
    /// # Returns
    /// Set of all descendant permission IDs
    pub fn get_all_children(&self, permission_id: &str) -> HashSet<String> {
        let mut children = HashSet::new();
        self.collect_children(permission_id, &mut children);
        children
    }

    /// Collect children recursively
    fn collect_children(&self, permission_id: &str, children: &mut HashSet<String>) {
        if let Some(direct_children) = self.relationships.get(permission_id) {
            for child in direct_children {
                if children.insert(child.clone()) {
                    self.collect_children(child, children);
                }
            }
        }
    }

    /// Get all transitive parents of a permission
    ///
    /// # Arguments
    /// * `permission_id` - Permission ID
    ///
    /// # Returns
    /// Set of all ancestor permission IDs
    pub fn get_all_parents(&self, permission_id: &str) -> HashSet<String> {
        let mut parents = HashSet::new();
        self.collect_parents(permission_id, &mut parents);
        parents
    }

    /// Collect parents recursively
    fn collect_parents(&self, permission_id: &str, parents: &mut HashSet<String>) {
        for (parent, children) in &self.relationships {
            if children.contains(&permission_id.to_string()) && parents.insert(parent.clone()) {
                self.collect_parents(parent, parents);
            }
        }
    }
}

impl Default for PermissionInheritance {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Permission Cache
// ============================================================================

/// Permission cache for performance optimization
#[derive(Debug)]
pub struct PermissionCache {
    /// Cache entries
    cache: DashMap<String, bool>,
    /// Maximum cache size
    max_size: usize,
    /// Current cache size
    current_size: Arc<RwLock<usize>>,
}

impl PermissionCache {
    /// Create a new permission cache
    ///
    /// # Arguments
    /// * `max_size` - Maximum cache size
    ///
    /// # Returns
    /// New PermissionCache instance
    pub fn new(max_size: usize) -> Self {
        Self {
            cache: DashMap::new(),
            max_size,
            current_size: Arc::new(RwLock::new(0)),
        }
    }

    /// Check if a permission is cached
    ///
    /// # Arguments
    /// * `key` - Cache key
    ///
    /// # Returns
    /// Option containing cached result
    pub fn get(&self, key: &str) -> Option<bool> {
        self.cache.get(key).map(|entry| *entry.value())
    }

    /// Cache a permission result
    ///
    /// # Arguments
    /// * `key` - Cache key
    /// * `value` - Permission result
    pub fn put(&self, key: String, value: bool) {
        let mut size_guard = self.current_size.blocking_write();
        if *size_guard >= self.max_size {
            // Clear half the cache when full
            let entries_to_remove = self.max_size / 2;
            let mut removed = 0;
            self.cache.retain(|_, _| {
                removed += 1;
                removed <= entries_to_remove
            });
            *size_guard -= entries_to_remove;
        }
        self.cache.insert(key, value);
        *size_guard += 1;
    }

    /// Invalidate a cache entry
    ///
    /// # Arguments
    /// * `key` - Cache key
    pub fn invalidate(&self, key: &str) {
        self.cache.remove(key);
        let mut size_guard = self.current_size.blocking_write();
        if *size_guard > 0 {
            *size_guard -= 1;
        }
    }

    /// Clear all cache entries
    pub fn clear(&self) {
        self.cache.clear();
        let mut size_guard = self.current_size.blocking_write();
        *size_guard = 0;
    }

    /// Get current cache size
    ///
    /// # Returns
    /// Current cache size
    pub fn size(&self) -> usize {
        *self.current_size.blocking_read()
    }
}

impl Default for PermissionCache {
    fn default() -> Self {
        Self::new(1000)
    }
}

// ============================================================================
// Permission Checker
// ============================================================================

/// Permission checker for evaluating access control
#[derive(Debug)]
pub struct PermissionChecker {
    /// Available permissions
    permissions: Vec<Permission>,
    /// Permission inheritance graph
    inheritance: PermissionInheritance,
    /// Permission cache
    cache: PermissionCache,
}

impl PermissionChecker {
    /// Create a new permission checker
    ///
    /// # Returns
    /// New PermissionChecker instance
    pub fn new() -> Self {
        Self {
            permissions: Vec::new(),
            inheritance: PermissionInheritance::new(),
            cache: PermissionCache::new(1000),
        }
    }

    /// Create a new permission checker with custom cache size
    ///
    /// # Arguments
    /// * `cache_size` - Cache size
    ///
    /// # Returns
    /// New PermissionChecker instance
    pub fn with_cache_size(cache_size: usize) -> Self {
        Self {
            permissions: Vec::new(),
            inheritance: PermissionInheritance::new(),
            cache: PermissionCache::new(cache_size),
        }
    }

    /// Add a permission
    ///
    /// # Arguments
    /// * `permission` - Permission to add
    pub fn add_permission(&mut self, permission: Permission) {
        self.permissions.push(permission);
    }

    /// Add multiple permissions
    ///
    /// # Arguments
    /// * `permissions` - Permissions to add
    pub fn add_permissions(&mut self, permissions: Vec<Permission>) {
        self.permissions.extend(permissions);
    }

    /// Add an inheritance relationship
    ///
    /// # Arguments
    /// * `parent` - Parent permission ID
    /// * `child` - Child permission ID
    pub fn add_inheritance(&mut self, parent: &str, child: &str) {
        self.inheritance.add_relationship(parent, child);
    }

    /// Check if a subject has permission to perform an action on a resource
    ///
    /// # Arguments
    /// * `subject` - Subject
    /// * `resource` - Resource
    /// * `action` - Action
    ///
    /// # Returns
    /// Result indicating if permission is granted or error
    pub fn check_permission(
        &self,
        subject: &Subject,
        resource: &Resource,
        action: &Action,
    ) -> RbacResult<bool> {
        let cache_key = self.generate_cache_key(subject, resource, action);

        if let Some(cached) = self.cache.get(&cache_key) {
            return Ok(cached);
        }

        let result = self.evaluate_permission(subject, resource, action)?;
        self.cache.put(cache_key, result);

        Ok(result)
    }

    /// Check permission without cache
    ///
    /// # Arguments
    /// * `subject` - Subject
    /// * `resource` - Resource
    /// * `action` - Action
    ///
    /// # Returns
    /// Result indicating if permission is granted or error
    pub fn check_permission_uncached(
        &self,
        subject: &Subject,
        resource: &Resource,
        action: &Action,
    ) -> RbacResult<bool> {
        self.evaluate_permission(subject, resource, action)
    }

    /// Evaluate permission against all rules
    ///
    /// # Arguments
    /// * `subject` - Subject
    /// * `resource` - Resource
    /// * `action` - Action
    ///
    /// # Returns
    /// Result indicating if permission is granted or error
    fn evaluate_permission(
        &self,
        _subject: &Subject,
        resource: &Resource,
        action: &Action,
    ) -> RbacResult<bool> {
        let mut matching_permissions: Vec<&Permission> = self
            .permissions
            .iter()
            .filter(|p| p.matches(resource, action))
            .collect();

        // Sort by priority (higher priority first)
        matching_permissions.sort_by(|a, b| b.priority.cmp(&a.priority));

        if matching_permissions.is_empty() {
            // No permissions configured for this resource/action - allow by default
            // Permission checker is an optional fine-grained control layer
            return Ok(true);
        }

        // Find highest priority allow or deny
        for permission in matching_permissions {
            if permission.effect == Effect::Deny {
                return Ok(false);
            }
            if permission.effect == Effect::Allow {
                return Ok(true);
            }
        }

        Ok(false)
    }

    /// Generate cache key for permission check
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

    /// Clear all cache entries
    pub fn clear_cache(&self) {
        self.cache.clear();
    }

    /// Get cache statistics
    ///
    /// # Returns
    /// Current cache size
    pub fn cache_size(&self) -> usize {
        self.cache.size()
    }
}

impl PermissionCache {
    /// Retain only entries that match a predicate
    ///
    /// # Arguments
    /// * `predicate` - Predicate function
    fn retain<F>(&self, predicate: F)
    where
        F: Fn(&str, &bool) -> bool,
    {
        let mut removed = 0;
        self.cache.retain(|k, v| {
            let keep = predicate(k, v);
            if !keep {
                removed += 1;
            }
            keep
        });
        let mut size_guard = self.current_size.blocking_write();
        *size_guard -= removed;
    }
}

impl Default for PermissionChecker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_permission_creation() {
        let permission = Permission::new("read_docs", "document:*", "read", Effect::Allow)
            .with_description("Read all documents")
            .with_priority(10);

        assert_eq!(permission.id, "read_docs");
        assert_eq!(permission.resource_pattern, "document:*");
        assert_eq!(permission.action_pattern, "read");
        assert_eq!(permission.effect, Effect::Allow);
    }

    #[test]
    fn test_match_pattern() {
        assert!(match_pattern("*", "anything"));
        assert!(match_pattern("exact", "exact"));
        assert!(!match_pattern("exact", "different"));

        assert!(match_pattern("doc:*", "doc:123"));
        assert!(!match_pattern("doc:*", "file:123"));
    }

    #[test]
    fn test_permission_matches() {
        let permission = Permission::new("read_docs", "document:*", "read", Effect::Allow);
        let resource = Resource::new("document", "doc123");
        let action = Action::new("read");

        assert!(permission.matches(&resource, &action));

        let different_action = Action::new("write");
        assert!(!permission.matches(&resource, &different_action));
    }

    #[test]
    fn test_permission_inheritance() {
        let mut inheritance = PermissionInheritance::new();
        inheritance.add_relationship("parent", "child1");
        inheritance.add_relationship("parent", "child2");
        inheritance.add_relationship("child1", "grandchild");

        let children = inheritance.get_all_children("parent");
        assert_eq!(children.len(), 3);
        assert!(children.contains("child1"));
        assert!(children.contains("child2"));
        assert!(children.contains("grandchild"));

        let parents = inheritance.get_all_parents("grandchild");
        assert_eq!(parents.len(), 2);
        assert!(parents.contains("child1"));
        assert!(parents.contains("parent"));
    }

    #[test]
    fn test_permission_checker() {
        let mut checker = PermissionChecker::new();

        let allow = Permission::new("allow", "document:*", "read", Effect::Allow);
        let deny = Permission::new("deny", "document:*", "write", Effect::Deny);

        checker.add_permission(allow);
        checker.add_permission(deny);

        let resource = Resource::new("document", "doc123");
        let action_read = Action::new("read");
        let action_write = Action::new("write");
        let subject = Subject::new("user", "user123");

        assert!(
            checker
                .check_permission(&subject, &resource, &action_read)
                .unwrap()
        );
        assert!(
            !checker
                .check_permission(&subject, &resource, &action_write)
                .unwrap()
        );
    }

    #[test]
    fn test_permission_cache() {
        let cache = PermissionCache::new(10);

        cache.put("key1".to_string(), true);
        cache.put("key2".to_string(), false);

        assert_eq!(cache.get("key1"), Some(true));
        assert_eq!(cache.get("key2"), Some(false));
        assert_eq!(cache.get("key3"), None);
        assert_eq!(cache.size(), 2);

        cache.invalidate("key1");
        assert_eq!(cache.get("key1"), None);
        assert_eq!(cache.size(), 1);

        cache.clear();
        assert_eq!(cache.size(), 0);
    }
}
