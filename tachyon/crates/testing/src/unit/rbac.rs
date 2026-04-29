//! Unit tests for RBAC types
//!
//! Tests for role hierarchy, permission creation, permission checking,
//! policy engine, enforcer, subjects, resources, and auth context.

#[allow(unused_imports)]
use tachyon_core::{generate_session_id, generate_user_id};
#[allow(unused_imports)]
use tachyon_rbac::{
    role::Role,
    types::{Action, AuthContext, Effect, Resource, Subject},
    Enforcer, Permission, PermissionChecker, Policy, PolicyEngine, PolicyRule, PolicyType,
    RbacResult,
};

#[test]
fn test_role_privilege_levels() {
    assert!(Role::Guest.privilege_level() < Role::User.privilege_level());
    assert!(Role::User.privilege_level() < Role::PowerUser.privilege_level());
    assert!(Role::PowerUser.privilege_level() < Role::Editor.privilege_level());
    assert!(Role::Editor.privilege_level() < Role::Moderator.privilege_level());
    assert!(Role::Moderator.privilege_level() < Role::Admin.privilege_level());
    assert!(Role::Admin.privilege_level() < Role::Owner.privilege_level());
}

#[test]
fn test_role_has_equal_or_higher() {
    assert!(Role::Admin.has_equal_or_higher_privilege(&Role::Guest));
    assert!(Role::Admin.has_equal_or_higher_privilege(&Role::Admin));
    assert!(!Role::Guest.has_equal_or_higher_privilege(&Role::Admin));
}

#[test]
fn test_role_permissions_guest() {
    let perms = Role::Guest.permissions();
    assert!(perms.contains("read"));
    assert!(!perms.contains("create_own"));
}

#[test]
fn test_role_permissions_owner() {
    let perms = Role::Owner.permissions();
    assert!(perms.contains("*"));
}

#[test]
fn test_role_permissions_editor() {
    let perms = Role::Editor.permissions();
    assert!(perms.contains("read"));
    assert!(perms.contains("create"));
    assert!(perms.contains("edit"));
    assert!(perms.contains("delete"));
}

#[test]
fn test_permission_creation() {
    let perm = Permission::new("read_docs", "document:*", "read", Effect::Allow)
        .with_description("Read all documents")
        .with_priority(10);

    assert_eq!(perm.id, "read_docs");
    assert_eq!(perm.resource_pattern, "document:*");
    assert_eq!(perm.action_pattern, "read");
    assert_eq!(perm.effect, Effect::Allow);
    assert_eq!(perm.priority, 10);
}

#[test]
fn test_permission_matches() {
    let perm = Permission::new("p1", "document:*", "read", Effect::Allow);
    let resource = Resource::new("document", "doc123");
    let action = Action::new("read");

    assert!(perm.matches(&resource, &action));

    let different_action = Action::new("write");
    assert!(!perm.matches(&resource, &different_action));
}

#[test]
fn test_permission_matches_wildcard() {
    let perm = Permission::new("p1", "*", "*", Effect::Allow);
    let resource = Resource::new("anything", "123");
    let action = Action::new("anything");

    assert!(perm.matches(&resource, &action));
}

#[test]
fn test_permission_checker_basic() {
    let mut checker = PermissionChecker::new();

    checker.add_permission(Permission::new(
        "allow",
        "document:*",
        "read",
        Effect::Allow,
    ));
    checker.add_permission(Permission::new("deny", "document:*", "write", Effect::Deny));

    let resource = Resource::new("document", "doc123");
    let subject = Subject::new("user", "user1");

    let allowed = checker
        .check_permission(&subject, &resource, &Action::new("read"))
        .unwrap();
    assert!(allowed);

    let denied = checker
        .check_permission(&subject, &resource, &Action::new("write"))
        .unwrap();
    assert!(!denied);
}

#[test]
fn test_permission_checker_default_allow() {
    let checker = PermissionChecker::new();
    let resource = Resource::new("space", "space1");
    let subject = Subject::new("user", "user1");
    let result = checker
        .check_permission(&subject, &resource, &Action::new("read"))
        .unwrap();
    assert!(result);
}

#[test]
fn test_permission_checker_priority() {
    let mut checker = PermissionChecker::new();

    checker.add_permission(
        Permission::new("allow", "document:*", "write", Effect::Allow).with_priority(1),
    );
    checker.add_permission(
        Permission::new("deny", "document:*", "write", Effect::Deny).with_priority(10),
    );

    let resource = Resource::new("document", "doc1");
    let subject = Subject::new("user", "user1");

    let result = checker
        .check_permission(&subject, &resource, &Action::new("write"))
        .unwrap();
    assert!(!result);
}

#[test]
fn test_permission_inheritance() {
    let mut inheritance = tachyon_rbac::permission::PermissionInheritance::new();
    inheritance.add_relationship("read", "read_own");
    inheritance.add_relationship("read_own", "read_any");

    let children = inheritance.get_all_children("read");
    assert_eq!(children.len(), 2);
    assert!(children.contains("read_own"));
    assert!(children.contains("read_any"));
}

#[test]
fn test_policy_engine_basic() {
    let engine = PolicyEngine::new();

    let policy = Policy::new("p1", "Document Access", PolicyType::Rbac)
        .add_rule(PolicyRule::new(
            "r1",
            "user:*",
            "document:*",
            "read",
            Effect::Allow,
        ))
        .add_rule(PolicyRule::new(
            "r2",
            "user:*",
            "document:*",
            "write",
            Effect::Deny,
        ));

    engine.add_policy(policy);

    let subject = Subject::new("user", "user1");
    let resource = Resource::new("document", "doc1");

    let (effect, _) = engine
        .evaluate(&subject, &resource, &Action::new("read"))
        .unwrap();
    assert_eq!(effect, Effect::Allow);

    let (effect, _) = engine
        .evaluate(&subject, &resource, &Action::new("write"))
        .unwrap();
    assert_eq!(effect, Effect::Deny);
}

#[test]
fn test_policy_engine_default_deny() {
    let engine = PolicyEngine::new();
    let subject = Subject::new("user", "user1");
    let resource = Resource::new("unknown", "1");

    let (effect, _) = engine
        .evaluate(&subject, &resource, &Action::new("read"))
        .unwrap();
    assert_eq!(effect, Effect::Deny);
}

#[test]
fn test_policy_rule_creation() {
    let rule = PolicyRule::new("r1", "user:*", "document:*", "read", Effect::Allow)
        .with_priority(5)
        .with_description("Allow read");

    assert_eq!(rule.id, "r1");
    assert_eq!(rule.effect, Effect::Allow);
    assert_eq!(rule.priority, 5);
}

#[test]
fn test_subject_creation() {
    let subject = Subject::new("user", "user123");
    assert_eq!(subject.subject_type, "user");
    assert_eq!(subject.subject_id, "user123");
    assert!(subject.attributes.is_empty());
}

#[test]
fn test_subject_validation() {
    let valid = Subject::new("user", "123");
    assert!(valid.validate().is_ok());

    let invalid_type = Subject::new("", "123");
    assert!(invalid_type.validate().is_err());

    let invalid_id = Subject::new("user", "");
    assert!(invalid_id.validate().is_err());
}

#[test]
fn test_subject_from_user() {
    let user_id = generate_user_id();
    let subject = Subject::from_user(&user_id);
    assert_eq!(subject.subject_type, "user");
    assert_eq!(subject.subject_id, user_id.as_str());
}

#[test]
fn test_subject_with_attribute() {
    let subject = Subject::new("user", "1").with_attribute("role", "admin");
    assert_eq!(subject.get_attribute("role").unwrap(), "admin");
}

#[test]
fn test_resource_creation() {
    let resource = Resource::new("document", "doc456");
    assert_eq!(resource.resource_type, "document");
    assert_eq!(resource.resource_id, "doc456");
    assert!(resource.owner_id.is_none());
}

#[test]
fn test_resource_validation() {
    let valid = Resource::new("document", "doc1");
    assert!(valid.validate().is_ok());

    let invalid = Resource::new("", "doc1");
    assert!(invalid.validate().is_err());
}

#[test]
fn test_auth_context() {
    let user_id = generate_user_id();
    let session_id = generate_session_id();
    let ctx = AuthContext::new(user_id, session_id)
        .with_role("admin")
        .with_roles(&["user", "moderator"])
        .with_attribute("ip", "127.0.0.1");

    assert!(ctx.has_role("admin"));
    assert!(ctx.has_role("user"));
    assert!(!ctx.has_role("guest"));
    assert_eq!(ctx.get_attribute("ip").unwrap(), "127.0.0.1");
}

#[test]
fn test_effect_combine() {
    assert_eq!(Effect::Allow.combine(Effect::Allow), Effect::Allow);
    assert_eq!(Effect::Allow.combine(Effect::Deny), Effect::Deny);
    assert_eq!(Effect::Deny.combine(Effect::Allow), Effect::Deny);
    assert_eq!(Effect::Deny.combine(Effect::Deny), Effect::Deny);
}

#[test]
fn test_enforcer_creation() {
    let enforcer = Enforcer::new();
    assert_eq!(enforcer.permission_checker().cache_size(), 0);
}

#[test]
fn test_enforcer_authorize_allow() {
    let mut enforcer = Enforcer::new();

    let policy = Policy::new("p1", "Test", PolicyType::Rbac).add_rule(PolicyRule::new(
        "r1",
        "user:*",
        "document:*",
        "read",
        Effect::Allow,
    ));
    enforcer.policy_engine().add_policy(policy);

    let subject = Subject::new("user", "user1");
    let resource = Resource::new("document", "doc1");
    let action = Action::new("read");
    let context = AuthContext::new(generate_user_id(), generate_session_id());
    let request = tachyon_rbac::types::AccessRequest::new(subject, resource, action, context);

    let decision = enforcer.authorize(&request).unwrap();
    assert!(decision.is_allowed());
}

#[test]
fn test_role_serde() {
    let role = Role::Admin;
    let json = serde_json::to_string(&role).unwrap();
    let de: Role = serde_json::from_str(&json).unwrap();
    assert_eq!(role, de);
}

#[test]
fn test_subject_display() {
    let subject = Subject::new("user", "123");
    assert_eq!(format!("{}", subject), "user:123");
}

#[test]
fn test_resource_display() {
    let resource = Resource::new("document", "doc1");
    assert_eq!(format!("{}", resource), "document:doc1");
}
