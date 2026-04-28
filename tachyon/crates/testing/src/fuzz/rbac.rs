//! Fuzzing tests for RBAC operations
//!
//! Property-based tests using random input to verify RBAC types
//! don't panic and handle edge cases gracefully.

#[allow(unused_imports)]
use tachyon_core::{generate_session_id, generate_user_id};
#[allow(unused_imports)]
use tachyon_rbac::{
    role::Role,
    Enforcer, Permission, PermissionChecker, Policy, PolicyEngine, PolicyRule, PolicyType,
    types::{Action, AuthContext, Effect, Resource, Subject},
};

#[test]
fn test_permission_random_patterns_no_panic() {
    let long_resource = "a".repeat(1000);
    let long_action = "a".repeat(1000);
    let patterns = vec![
        ("", ""),
        ("*", "*"),
        ("*", "read"),
        ("document:*", "*"),
        ("*", "document:*"),
        (long_resource.as_str(), "action"),
        ("resource", long_action.as_str()),
        ("resource", ""),
        ("", "action"),
        ("doc:123", "read"),
        ("doc:*", "r*"),
        ("**multiple**", "**wildcards**"),
        ("resource?name", "action?name"),
        ("resource\nname", "action\nname"),
    ];

    for (resource_pattern, action_pattern) in &patterns {
        let perm = Permission::new("test", resource_pattern, action_pattern, Effect::Allow);
        let resource = Resource::new("document", "doc1");
        let action = Action::new("read");
        let _ = perm.matches(&resource, &action);
    }
}

#[test]
fn test_permission_checker_random_rules_no_panic() {
    let mut checker = PermissionChecker::new();

    let effects = vec![Effect::Allow, Effect::Deny];
    let priorities = vec![-100, -1, 0, 1, 100, i32::MAX];

    for effect in &effects {
        for priority in &priorities {
            checker.add_permission(
                Permission::new("test", "*", "*", *effect).with_priority(*priority),
            );
        }
    }

    let resource = Resource::new("any", "thing");
    let subject = Subject::new("user", "1");

    for action_name in &["read", "write", "delete", "admin", ""] {
        let _ = checker.check_permission(&subject, &resource, &Action::new(action_name));
    }
}

#[test]
fn test_policy_engine_random_policies_no_panic() {
    let engine = PolicyEngine::new();

    for i in 0..20 {
        let policy = Policy::new(
            &format!("policy-{}", i),
            &format!("Policy {}", i),
            PolicyType::Rbac,
        )
        .add_rule(
            PolicyRule::new(
                &format!("r{}", i),
                "user:*",
                "document:*",
                "read",
                Effect::Allow,
            )
            .with_priority(i as i32),
        );

        engine.add_policy(policy);
    }

    let subject = Subject::new("user", "user1");
    let resource = Resource::new("document", "doc1");
    let action = Action::new("read");

    let _ = engine.evaluate(&subject, &resource, &action);
}

#[test]
fn test_subject_random_values_no_panic() {
    let long_500 = "a".repeat(500);
    let subject_types = vec!["", "user", "role", "service", long_500.as_str()];
    let subject_ids = vec!["", "1", "user-123", long_500.as_str()];

    for st in &subject_types {
        for si in &subject_ids {
            let subject = Subject::new(st, si);
            let _ = subject.validate();
            let _ = format!("{}", subject);
        }
    }
}

#[test]
fn test_resource_random_values_no_panic() {
    let long_500 = "a".repeat(500);
    let resource_types = vec!["", "document", "repository", "space", long_500.as_str()];
    let resource_ids = vec!["", "1", "doc-123", long_500.as_str()];

    for rt in &resource_types {
        for ri in &resource_ids {
            let resource = Resource::new(rt, ri);
            let _ = resource.validate();
            let _ = format!("{}", resource);
        }
    }
}

#[test]
fn test_action_random_values_no_panic() {
    let long_500 = "a".repeat(500);
    let action_names = vec![
        "", "read", "write", "delete", "admin", long_500.as_str(),
        "action with spaces", "action\nwith\nnewlines",
    ];

    for name in &action_names {
        let action = Action::new(name);
        let _ = action.validate();
    }
}

#[test]
fn test_role_all_variants_serde() {
    let roles = vec![Role::Guest, Role::User, Role::PowerUser, Role::Editor, Role::Moderator, Role::Admin, Role::Owner];

    for role in &roles {
        let json = serde_json::to_string(role).unwrap();
        let de: Role = serde_json::from_str(&json).unwrap();
        assert_eq!(*role, de);
        assert_eq!(role.privilege_level(), de.privilege_level());
        assert!(role.has_equal_or_higher_privilege(&de));
    }
}

#[test]
fn test_policy_rule_conditions() {
    let rule = PolicyRule::new("r1", "user:*", "document:*", "read", Effect::Allow)
        .with_condition("subject.type", "user")
        .with_condition("resource.type", "document");

    let subject = Subject::new("user", "1");
    let resource = Resource::new("document", "doc1");
    let action = Action::new("read");

    assert!(rule.matches(&subject, &resource, &action));
}

#[test]
fn test_auth_context_random_roles() {
    let long_100 = "a".repeat(100);
    let roles = vec!["", "admin", "user", "guest", long_100.as_str()];

    for role in &roles {
        let ctx = AuthContext::new(generate_user_id(), generate_session_id()).with_role(role);
        let _ = ctx.has_role(role);
    }
}

#[test]
fn test_effect_all_combinations() {
    let effects = vec![Effect::Allow, Effect::Deny];
    let mut results = Vec::new();

    for a in &effects {
        for b in &effects {
            results.push(a.combine(*b));
        }
    }

    assert_eq!(results.len(), 4);
    assert!(results.contains(&Effect::Allow));
    assert!(results.contains(&Effect::Deny));
}
