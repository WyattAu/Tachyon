//! Integration tests for RBAC operations
//!
//! Tests the server's RBAC enforcement logic, Claims deserialization,
//! permission hierarchy, and role-based access patterns.

#[cfg(test)]
mod tests {
    use tachyon_core::{UserAction, UserRole};
    use tachyon_database::Permission;
    use tachyon_rbac::role::Role as RbacRole;
    use tachyon_rbac::types::{
        AccessRequest, Action, AuthContext as RbacAuthContext, Effect, Resource, Subject,
    };
    use tachyon_rbac::{Enforcer, SessionId, UserId};
    use tachyon_server::middleware::auth::{AuthContext, AuthMethod, PermissionGuard};

    fn make_auth_context(
        role: UserRole,
        permissions: Vec<&str>,
        team_id: Option<&str>,
    ) -> AuthContext {
        AuthContext {
            user_id: "test-user-123".to_string(),
            role,
            permissions: permissions.iter().map(|s| s.to_string()).collect(),
            team_id: team_id.map(|s| s.to_string()),
            auth_method: AuthMethod::Jwt,
        }
    }

    fn make_enforcer() -> Enforcer {
        Enforcer::new()
    }

    fn make_access_request(
        role_str: &str,
        resource_type: &str,
        resource_id: &str,
        action: &str,
    ) -> AccessRequest {
        let subject = Subject::from_role(role_str);
        let resource = Resource::new(resource_type, resource_id);
        let action = Action::new(action);
        let context = RbacAuthContext::new(UserId::new(), SessionId::new()).with_role(role_str);
        AccessRequest::new(subject, resource, action, context)
    }

    // ===================================================================
    // 1. Role-based permission verification (server AuthContext)
    // ===================================================================

    #[test]
    fn test_reader_role_has_read_only_permissions() {
        let auth = make_auth_context(UserRole::Reader, vec!["read"], None);

        assert!(auth.has_permission(Permission::Read));
        assert!(!auth.has_permission(Permission::Write));
        assert!(!auth.has_permission(Permission::Delete));
        assert!(!auth.has_permission(Permission::Admin));
    }

    #[test]
    fn test_writer_role_has_read_write_permissions() {
        let auth = make_auth_context(UserRole::Writer, vec!["read", "write"], None);

        assert!(auth.has_permission(Permission::Read));
        assert!(auth.has_permission(Permission::Write));
        assert!(!auth.has_permission(Permission::Delete));
    }

    #[test]
    fn test_admin_role_has_all_permissions() {
        let auth = make_auth_context(UserRole::Admin, vec![], None);

        assert!(auth.has_permission(Permission::Read));
        assert!(auth.has_permission(Permission::Write));
        assert!(auth.has_permission(Permission::Delete));
        assert!(auth.has_permission(Permission::Admin));
        assert!(auth.has_permission(Permission::Owner));
    }

    #[test]
    fn test_admin_role_bypasses_permission_list() {
        let auth = make_auth_context(UserRole::Admin, vec![], None);

        assert!(auth.has_permission(Permission::Read));
        assert!(auth.has_permission(Permission::Admin));
    }

    // ===================================================================
    // 2. Permission hierarchy and includes logic
    // ===================================================================

    #[test]
    fn test_permission_hierarchy_levels() {
        assert!(Permission::Owner.level() > Permission::Admin.level());
        assert!(Permission::Admin.level() > Permission::Delete.level());
        assert!(Permission::Delete.level() > Permission::Write.level());
        assert!(Permission::Write.level() > Permission::Read.level());
    }

    #[test]
    fn test_permission_includes_higher_includes_lower() {
        assert!(Permission::Admin.includes(&Permission::Read));
        assert!(Permission::Admin.includes(&Permission::Write));
        assert!(Permission::Admin.includes(&Permission::Delete));
        assert!(Permission::Delete.includes(&Permission::Write));
        assert!(Permission::Write.includes(&Permission::Read));
        assert!(!Permission::Read.includes(&Permission::Write));
        assert!(!Permission::Write.includes(&Permission::Delete));
    }

    #[test]
    fn test_wildcard_permission_owner_includes_all() {
        let auth = make_auth_context(UserRole::Admin, vec!["owner"], None);

        assert!(auth.has_permission(Permission::Read));
        assert!(auth.has_permission(Permission::Write));
        assert!(auth.has_permission(Permission::Delete));
        assert!(auth.has_permission(Permission::Admin));
        assert!(auth.has_permission(Permission::Owner));
    }

    #[test]
    fn test_specific_permission_overrides_via_hierarchy() {
        let auth = make_auth_context(UserRole::Reader, vec!["delete"], None);

        assert!(auth.has_permission(Permission::Delete));
        assert!(auth.has_permission(Permission::Write));
        assert!(auth.has_permission(Permission::Read));
    }

    #[test]
    fn test_unrecognised_permission_string_ignored() {
        let auth = make_auth_context(UserRole::Reader, vec!["foobar"], None);

        assert!(!auth.has_permission(Permission::Read));
        assert!(!auth.has_permission(Permission::Write));
    }

    // ===================================================================
    // 3. has_any / has_all helpers
    // ===================================================================

    #[test]
    fn test_has_any_permission() {
        let auth = make_auth_context(UserRole::Reader, vec!["read"], None);

        assert!(auth.has_any_permission(&[Permission::Read, Permission::Write]));
        assert!(auth.has_any_permission(&[Permission::Read]));
        assert!(!auth.has_any_permission(&[Permission::Write, Permission::Delete]));
    }

    #[test]
    fn test_has_all_permissions() {
        let auth = make_auth_context(UserRole::Reader, vec!["read"], None);

        assert!(auth.has_all_permissions(&[Permission::Read]));
        assert!(!auth.has_all_permissions(&[Permission::Read, Permission::Write]));

        let admin = make_auth_context(UserRole::Admin, vec![], None);
        assert!(admin.has_all_permissions(&[
            Permission::Read,
            Permission::Write,
            Permission::Delete,
            Permission::Admin,
        ]));
    }

    // ===================================================================
    // 4. is_admin checks
    // ===================================================================

    #[test]
    fn test_is_admin_for_admin_role() {
        let auth = make_auth_context(UserRole::Admin, vec![], None);
        assert!(auth.is_admin());
    }

    #[test]
    fn test_is_admin_via_permission_string() {
        let auth = make_auth_context(UserRole::Reader, vec!["admin"], None);
        assert!(auth.is_admin());
    }

    #[test]
    fn test_is_admin_false_for_regular_user() {
        let auth = make_auth_context(UserRole::Reader, vec!["read"], None);
        assert!(!auth.is_admin());
    }

    // ===================================================================
    // 5. Claims-based access control (JWT Claims deserialization)
    // ===================================================================

    #[test]
    fn test_claims_serialization_roundtrip() {
        let claims = serde_json::json!({
            "sub": "user-42",
            "iss": "tachyon-test",
            "aud": "tachyon-test",
            "exp": 4102444800_i64,
            "iat": 1700000000_i64,
            "role": "admin",
            "permissions": ["read", "write"],
            "team_id": "team-1"
        });

        let deserialized: tachyon_server::middleware::auth::Claims =
            serde_json::from_value(claims).unwrap();

        let reserialized = serde_json::to_value(&deserialized).unwrap();
        assert_eq!(reserialized["sub"], "user-42");
        assert_eq!(reserialized["role"], "admin");
        assert_eq!(reserialized["permissions"].as_array().unwrap().len(), 2);
        assert_eq!(reserialized["team_id"], "team-1");
    }

    #[test]
    fn test_claims_default_permissions_empty() {
        let json = serde_json::json!({
            "sub": "u1",
            "iss": "test",
            "aud": "test",
            "exp": 4102444800_i64,
            "iat": 1700000000_i64,
            "role": "reader"
        });

        let claims: tachyon_server::middleware::auth::Claims =
            serde_json::from_value(json).unwrap();
        let v = serde_json::to_value(&claims).unwrap();
        assert_eq!(v["role"], "reader");
        assert_eq!(v["sub"], "u1");

        let json_with_perms = serde_json::json!({
            "sub": "u2",
            "iss": "test",
            "aud": "test",
            "exp": 4102444800_i64,
            "iat": 1700000000_i64,
            "role": "writer",
            "permissions": ["read", "write"],
            "team_id": "team-1"
        });

        let claims2: tachyon_server::middleware::auth::Claims =
            serde_json::from_value(json_with_perms).unwrap();
        let v2 = serde_json::to_value(&claims2).unwrap();
        assert_eq!(v2["role"], "writer");
        assert_eq!(v2["sub"], "u2");
        assert_eq!(v2["permissions"].as_array().unwrap().len(), 2);
        assert_eq!(v2["team_id"], "team-1");
    }

    #[test]
    fn test_claims_with_team_id() {
        let auth = make_auth_context(UserRole::Writer, vec!["read", "write"], Some("team-xyz"));

        assert_eq!(auth.team_id.as_deref(), Some("team-xyz"));
        assert!(auth.has_permission(Permission::Read));
        assert!(auth.has_permission(Permission::Write));
        assert!(!auth.has_permission(Permission::Admin));
    }

    #[test]
    fn test_claims_without_team_id() {
        let auth = make_auth_context(UserRole::Reader, vec!["read"], None);
        assert!(auth.team_id.is_none());
    }

    // ===================================================================
    // 6. PermissionGuard
    // ===================================================================

    #[test]
    fn test_permission_guard_allows_matching() {
        let guard = PermissionGuard::new(Permission::Write);
        let auth = make_auth_context(UserRole::Writer, vec!["read", "write"], None);
        assert!(guard.check(&auth));
    }

    #[test]
    fn test_permission_guard_denies_non_matching() {
        let guard = PermissionGuard::new(Permission::Delete);
        let auth = make_auth_context(UserRole::Writer, vec!["read", "write"], None);
        assert!(!guard.check(&auth));
    }

    #[test]
    fn test_permission_guard_allways_allows_admin() {
        let guard = PermissionGuard::new(Permission::Owner);
        let auth = make_auth_context(UserRole::Admin, vec![], None);
        assert!(guard.check(&auth));
    }

    // ===================================================================
    // 7. require_permission helper (check_permission)
    // ===================================================================

    #[test]
    fn test_check_permission_admin_allows_everything() {
        let auth = make_auth_context(UserRole::Admin, vec![], None);
        assert!(tachyon_server::middleware::auth::check_permission(
            &auth,
            UserAction::ManageUsers
        ));
        assert!(tachyon_server::middleware::auth::check_permission(
            &auth,
            UserAction::Configure
        ));
    }

    #[test]
    fn test_check_permission_reader_can_only_read() {
        let auth = make_auth_context(UserRole::Reader, vec!["read"], None);
        assert!(tachyon_server::middleware::auth::check_permission(
            &auth,
            UserAction::Read
        ));
        assert!(!tachyon_server::middleware::auth::check_permission(
            &auth,
            UserAction::Write
        ));
        assert!(!tachyon_server::middleware::auth::check_permission(
            &auth,
            UserAction::Delete
        ));
        assert!(!tachyon_server::middleware::auth::check_permission(
            &auth,
            UserAction::ManageUsers
        ));
    }

    #[test]
    fn test_check_permission_writer_can_read_and_write() {
        let auth = make_auth_context(UserRole::Writer, vec!["read", "write"], None);
        assert!(tachyon_server::middleware::auth::check_permission(
            &auth,
            UserAction::Read
        ));
        assert!(tachyon_server::middleware::auth::check_permission(
            &auth,
            UserAction::Write
        ));
        assert!(!tachyon_server::middleware::auth::check_permission(
            &auth,
            UserAction::Edit
        ));
        assert!(!tachyon_server::middleware::auth::check_permission(
            &auth,
            UserAction::Delete
        ));
    }

    #[test]
    fn test_check_permission_editor_can_edit() {
        let auth = make_auth_context(UserRole::Editor, vec!["read", "write", "delete"], None);
        assert!(tachyon_server::middleware::auth::check_permission(
            &auth,
            UserAction::Edit
        ));
        assert!(tachyon_server::middleware::auth::check_permission(
            &auth,
            UserAction::Read
        ));
    }

    // ===================================================================
    // 8. Enforcer integration (RBAC crate)
    // ===================================================================

    #[test]
    fn test_enforcer_admin_role_allowed_everywhere() {
        let mut enforcer = make_enforcer();
        let req = make_access_request("admin", "document", "doc-1", "delete");
        let decision = enforcer.authorize(&req).unwrap();
        assert!(decision.is_allowed());
    }

    #[test]
    fn test_enforcer_reader_can_read_documents() {
        let mut enforcer = make_enforcer();
        let req = make_access_request("reader", "document", "doc-1", "read");
        let decision = enforcer.authorize(&req).unwrap();
        assert!(decision.is_allowed());
    }

    #[test]
    fn test_enforcer_reader_cannot_write_documents() {
        let mut enforcer = make_enforcer();
        let req = make_access_request("reader", "document", "doc-1", "write");
        let decision = enforcer.authorize(&req).unwrap();
        assert!(!decision.is_allowed());
    }

    #[test]
    fn test_enforcer_writer_can_read_and_write_documents() {
        let mut enforcer = make_enforcer();

        let read_req = make_access_request("writer", "document", "doc-1", "read");
        assert!(enforcer.authorize(&read_req).unwrap().is_allowed());

        let write_req = make_access_request("writer", "document", "doc-1", "write");
        assert!(enforcer.authorize(&write_req).unwrap().is_allowed());
    }

    #[test]
    fn test_enforcer_writer_cannot_delete_documents() {
        let mut enforcer = make_enforcer();
        let req = make_access_request("writer", "document", "doc-1", "delete");
        let decision = enforcer.authorize(&req).unwrap();
        assert!(!decision.is_allowed());
    }

    #[test]
    fn test_enforcer_editor_has_full_document_access() {
        let mut enforcer = make_enforcer();

        for action in ["read", "write", "edit", "delete"] {
            let req = make_access_request("editor", "document", "doc-1", action);
            assert!(
                enforcer.authorize(&req).unwrap().is_allowed(),
                "editor should be able to {}",
                action
            );
        }
    }

    // ===================================================================
    // 9. Cross-role boundary enforcement
    // ===================================================================

    #[test]
    fn test_reader_cannot_access_admin_resources() {
        let auth = make_auth_context(UserRole::Reader, vec!["read"], None);
        assert!(!auth.has_permission(Permission::Admin));
        assert!(!tachyon_server::middleware::auth::check_permission(
            &auth,
            UserAction::Configure
        ));
        assert!(!tachyon_server::middleware::auth::check_permission(
            &auth,
            UserAction::ManageUsers
        ));
    }

    #[test]
    fn test_writer_cannot_manage_roles() {
        let auth = make_auth_context(UserRole::Writer, vec!["read", "write"], None);
        assert!(!auth.has_permission(Permission::Admin));
        assert!(!tachyon_server::middleware::auth::check_permission(
            &auth,
            UserAction::ManageUsers
        ));
        assert!(!tachyon_server::middleware::auth::check_permission(
            &auth,
            UserAction::Configure
        ));
    }

    #[test]
    fn test_editor_cannot_perform_admin_actions_via_role() {
        let auth = make_auth_context(UserRole::Editor, vec!["read", "write", "delete"], None);
        assert!(!tachyon_server::middleware::auth::check_permission(
            &auth,
            UserAction::ManageUsers
        ));
        assert!(!tachyon_server::middleware::auth::check_permission(
            &auth,
            UserAction::Configure
        ));
    }

    // ===================================================================
    // 10. RBAC Role hierarchy (rbac::role module)
    // ===================================================================

    #[test]
    fn test_rbac_role_privilege_ordering() {
        use RbacRole::*;

        assert!(Owner.privilege_level() > Admin.privilege_level());
        assert!(Admin.privilege_level() > Moderator.privilege_level());
        assert!(Moderator.privilege_level() > Editor.privilege_level());
        assert!(Editor.privilege_level() > PowerUser.privilege_level());
        assert!(PowerUser.privilege_level() > User.privilege_level());
        assert!(User.privilege_level() > Guest.privilege_level());
    }

    #[test]
    fn test_rbac_role_has_equal_or_higher_privilege() {
        use RbacRole::*;

        assert!(Admin.has_equal_or_higher_privilege(&User));
        assert!(Admin.has_equal_or_higher_privilege(&Admin));
        assert!(!User.has_equal_or_higher_privilege(&Admin));
        assert!(!Guest.has_equal_or_higher_privilege(&User));
    }

    #[test]
    fn test_rbac_role_permissions_guest_read_only() {
        let perms = RbacRole::Guest.permissions();
        assert!(perms.contains("read"));
        assert_eq!(perms.len(), 1);
    }

    #[test]
    fn test_rbac_role_permissions_admin_broad() {
        let perms = RbacRole::Admin.permissions();
        for perm in [
            "read", "create", "edit", "delete", "share", "admin", "review", "approve",
        ] {
            assert!(
                perms.contains(perm),
                "Admin should have '{}' permission",
                perm
            );
        }
    }

    #[test]
    fn test_rbac_role_permissions_owner_wildcard() {
        let perms = RbacRole::Owner.permissions();
        assert!(perms.contains("*"));
        assert_eq!(perms.len(), 1);
    }

    // ===================================================================
    // 11. Effect combination
    // ===================================================================

    #[test]
    fn test_effect_deny_overrides_allow() {
        assert_eq!(Effect::Allow.combine(Effect::Deny), Effect::Deny);
        assert_eq!(Effect::Deny.combine(Effect::Allow), Effect::Deny);
        assert_eq!(Effect::Deny.combine(Effect::Deny), Effect::Deny);
        assert_eq!(Effect::Allow.combine(Effect::Allow), Effect::Allow);
    }

    // ===================================================================
    // 12. RbacAuthContext (rbac crate's AuthContext)
    // ===================================================================

    #[test]
    fn test_rbac_auth_context_role_management() {
        let ctx = RbacAuthContext::new(UserId::new(), SessionId::new())
            .with_role("admin")
            .with_roles(&["editor", "viewer"]);

        assert!(ctx.has_role("admin"));
        assert!(ctx.has_role("editor"));
        assert!(ctx.has_role("viewer"));
        assert!(!ctx.has_role("guest"));
    }

    #[test]
    fn test_rbac_auth_context_attributes() {
        let ctx = RbacAuthContext::new(UserId::new(), SessionId::new())
            .with_attribute("team_id", "team-1")
            .with_attribute("ip", "10.0.0.1");

        assert_eq!(ctx.get_attribute("team_id").unwrap(), "team-1");
        assert_eq!(ctx.get_attribute("ip").unwrap(), "10.0.0.1");
        assert!(ctx.get_attribute("missing").is_none());
    }

    #[test]
    fn test_rbac_auth_context_as_subject() {
        let ctx = RbacAuthContext::new(UserId::new(), SessionId::new()).with_role("writer");
        let subject = ctx.as_subject();
        assert_eq!(subject.subject_type, "user");
        assert_eq!(subject.get_attribute("role").unwrap(), "writer");
    }

    // ===================================================================
    // 13. Subject / Resource validation
    // ===================================================================

    #[test]
    fn test_subject_validation() {
        let valid = Subject::new("user", "u1");
        assert!(valid.validate().is_ok());

        let empty_type = Subject::new("", "u1");
        assert!(empty_type.validate().is_err());

        let empty_id = Subject::new("user", "");
        assert!(empty_id.validate().is_err());
    }

    #[test]
    fn test_resource_validation() {
        let valid = Resource::new("document", "doc-1");
        assert!(valid.validate().is_ok());

        let empty_type = Resource::new("", "doc-1");
        assert!(empty_type.validate().is_err());

        let empty_id = Resource::new("document", "");
        assert!(empty_id.validate().is_err());
    }

    #[test]
    fn test_resource_with_owner() {
        let res = Resource::new("document", "doc-1").with_owner("user-42");
        assert_eq!(res.owner_id.as_deref(), Some("user-42"));
    }

    // ===================================================================
    // 14. AccessRequest validation
    // ===================================================================

    #[test]
    fn test_access_request_validation() {
        let req = make_access_request("reader", "document", "doc-1", "read");
        assert!(req.validate().is_ok());
    }

    #[test]
    fn test_access_request_invalid_subject_rejected() {
        let subject = Subject::new("", "u1");
        let resource = Resource::new("document", "doc-1");
        let action = Action::new("read");
        let context = RbacAuthContext::new(UserId::new(), SessionId::new());
        let req = AccessRequest::new(subject, resource, action, context);
        assert!(req.validate().is_err());
    }

    // ===================================================================
    // 15. AuthMethod & AuthError
    // ===================================================================

    #[test]
    fn test_auth_method_equality() {
        assert_eq!(AuthMethod::Jwt, AuthMethod::Jwt);
        assert_eq!(AuthMethod::ApiKey, AuthMethod::ApiKey);
        assert_eq!(AuthMethod::Bearer, AuthMethod::Bearer);
        assert_ne!(AuthMethod::Jwt, AuthMethod::ApiKey);
    }

    #[test]
    fn test_auth_error_display() {
        use tachyon_server::middleware::auth::AuthError;

        assert_eq!(
            AuthError::MissingAuthHeader.to_string(),
            "Missing authorization header"
        );
        assert_eq!(
            AuthError::InvalidTokenFormat.to_string(),
            "Invalid token format"
        );
        assert_eq!(AuthError::TokenExpired.to_string(), "Token expired");
        assert_eq!(AuthError::InvalidSignature.to_string(), "Invalid signature");
        assert_eq!(AuthError::InvalidApiKey.to_string(), "Invalid API key");
        assert_eq!(AuthError::UserNotFound.to_string(), "User not found");
        assert_eq!(
            AuthError::InsufficientPermissions.to_string(),
            "Insufficient permissions"
        );
    }

    // ===================================================================
    // 16. UserRole Display / permission level
    // ===================================================================

    #[test]
    fn test_user_role_display() {
        assert_eq!(UserRole::Reader.to_string(), "reader");
        assert_eq!(UserRole::Writer.to_string(), "writer");
        assert_eq!(UserRole::Editor.to_string(), "editor");
        assert_eq!(UserRole::Admin.to_string(), "admin");
    }

    #[test]
    fn test_user_role_permission_level_ordering() {
        assert!(UserRole::Admin.permission_level() > UserRole::Editor.permission_level());
        assert!(UserRole::Editor.permission_level() > UserRole::Writer.permission_level());
        assert!(UserRole::Writer.permission_level() > UserRole::Reader.permission_level());
    }

    #[test]
    fn test_user_role_can_perform() {
        assert!(UserRole::Reader.can_perform(UserAction::Read));
        assert!(!UserRole::Reader.can_perform(UserAction::Write));

        assert!(UserRole::Writer.can_perform(UserAction::Read));
        assert!(UserRole::Writer.can_perform(UserAction::Write));
        assert!(!UserRole::Writer.can_perform(UserAction::Edit));

        assert!(UserRole::Editor.can_perform(UserAction::Edit));

        assert!(UserRole::Admin.can_perform(UserAction::ManageUsers));
        assert!(UserRole::Admin.can_perform(UserAction::Configure));
    }
}
