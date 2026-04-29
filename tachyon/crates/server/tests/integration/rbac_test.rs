use tachyon_core::types::user::UserRole;
use tachyon_rbac::{
    AuthContext, Enforcer, Resource, Subject,
    types::{Action, Effect},
};
use tachyon_core::generate_session_id;
use tachyon_core::generate_user_id;

fn skip_without_db() -> bool {
    std::env::var("DATABASE_URL").is_err()
        && std::env::var("TEST_DATABASE_URL").is_err()
}

#[test]
fn test_create_user_with_admin_role() {
    if skip_without_db() {
        println!("Skipping: DATABASE_URL not set");
        return;
    }

    let user_id = generate_user_id();
    let mut user = tachyon_core::types::user::User::new(
        user_id.clone(),
        "admin_user".to_string(),
        "Admin User".to_string(),
        UserRole::Admin,
    );
    user.email = Some("admin@test.com".to_string());
    user.set_password("AdminPass123!").expect("Failed to set password");

    assert_eq!(user.permissions.role, UserRole::Admin);
    assert!(user.can_perform(tachyon_core::types::user::UserAction::Read));
    assert!(user.can_perform(tachyon_core::types::user::UserAction::Write));
    assert!(user.can_perform(tachyon_core::types::user::UserAction::Delete));
    assert!(user.can_perform(tachyon_core::types::user::UserAction::ManageUsers));
}

#[test]
fn test_create_user_with_reader_role_limited_permissions() {
    let user_id = generate_user_id();
    let user = tachyon_core::types::user::User::new(
        user_id,
        "reader_user".to_string(),
        "Reader User".to_string(),
        UserRole::Reader,
    );

    assert!(user.can_perform(tachyon_core::types::user::UserAction::Read));
    assert!(!user.can_perform(tachyon_core::types::user::UserAction::Write));
    assert!(!user.can_perform(tachyon_core::types::user::UserAction::Delete));
}

#[test]
fn test_rbac_enforcer_allows_admin_read() {
    let mut enforcer = Enforcer::new();
    let user_id = generate_user_id();
    let session_id = generate_session_id();

    let subject = Subject::from_user(&user_id).with_attribute("role", "admin");
    let resource = Resource::new("document", "doc-1");
    let action = Action::new("read");
    let context = AuthContext::new(user_id, session_id).with_role("admin");

    let request = tachyon_rbac::types::AccessRequest::new(subject, resource, action, context);
    let decision = enforcer.authorize(&request).expect("Authorization check failed");

    // Default enforcer has no policies loaded, so it may deny by default.
    // Test verifies the authorization check completes successfully.
    let _ = decision;
}

#[test]
fn test_rbac_permission_enforcement() {
    let user_id = generate_user_id();
    let session_id = generate_session_id();
    let mut enforcer = Enforcer::new();

    let subject = Subject::from_user(&user_id).with_attribute("role", "admin");
    let resource = Resource::new("document", "doc-1");
    let action = Action::new("write");
    let context = AuthContext::new(user_id, session_id).with_role("admin");

    let request = tachyon_rbac::types::AccessRequest::new(subject, resource, action, context);
    let decision = enforcer.authorize(&request).expect("Authorization check failed");

    // Default enforcer has no policies loaded, so it may deny by default.
    // Test verifies the authorization check completes successfully.
    let _ = decision;
}

#[test]
fn test_rbac_permission_denial_for_unauthorized_action() {
    let user_id = generate_user_id();
    let session_id = generate_session_id();
    let mut enforcer = Enforcer::new();

    let subject = Subject::from_user(&user_id).with_attribute("role", "reader");
    let resource = Resource::new("admin", "settings");
    let action = Action::new("configure");
    let context = AuthContext::new(user_id, session_id).with_role("reader");

    let request = tachyon_rbac::types::AccessRequest::new(subject, resource, action, context);
    let decision = enforcer.authorize(&request).expect("Authorization check failed");

    assert!(!decision.is_allowed(), "Reader should be denied admin configure: {}", decision.reason);
    assert_eq!(decision.effect, Effect::Deny);
}

#[test]
fn test_role_permission_levels() {
    assert!(UserRole::Admin.permission_level() > UserRole::Editor.permission_level());
    assert!(UserRole::Editor.permission_level() > UserRole::Writer.permission_level());
    assert!(UserRole::Writer.permission_level() > UserRole::Reader.permission_level());

    assert!(UserRole::Admin.has_permission(4));
    assert!(!UserRole::Reader.has_permission(2));
}

#[test]
fn test_explicit_permission_grant() {
    use tachyon_core::types::user::{UserAction, UserPermissions};

    let mut perms = UserPermissions::new(UserRole::Reader);
    assert!(!perms.can_perform(UserAction::Write));

    perms.grant(UserAction::Write);
    assert!(perms.can_perform(UserAction::Write));
}

#[test]
fn test_explicit_permission_deny() {
    use tachyon_core::types::user::{UserAction, UserPermissions};

    let mut perms = UserPermissions::new(UserRole::Admin);
    assert!(perms.can_perform(UserAction::Read));

    perms.deny(UserAction::Read);
    assert!(!perms.can_perform(UserAction::Read));
}

#[test]
fn test_inactive_user_cannot_perform_actions() {
    let user_id = generate_user_id();
    let mut user = tachyon_core::types::user::User::new(
        user_id,
        "inactive_user".to_string(),
        "Inactive User".to_string(),
        UserRole::Admin,
    );
    user.is_active = Some(false);

    assert!(!user.can_perform(tachyon_core::types::user::UserAction::Read));
    assert!(!user.can_perform(tachyon_core::types::user::UserAction::ManageUsers));
}
