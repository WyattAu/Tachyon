#[cfg(test)]
mod tests {
    use super::super::*;
    use tachyon_core::{User, UserRole};

    #[test]
    fn test_register_request_construction() {
        let req = RegisterRequest {
            username: "testuser".to_string(),
            display_name: "Test User".to_string(),
            email: Some("test@example.com".to_string()),
            password: "password123".to_string(),
        };
        assert_eq!(req.username, "testuser");
        assert_eq!(req.display_name, "Test User");
    }

    #[test]
    fn test_authenticate_response_serialization() {
        let resp = AuthenticateResponse {
            success: true,
            user_id: Some("user-1".to_string()),
            access_token: Some("token-123".to_string()),
            refresh_token: None,
            token_type: "Bearer".to_string(),
            expires_in: 3600,
            error: None,
            user: None,
            mfa_required: false,
            mfa_user_id: None,
        };

        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("Bearer"));
        assert!(json.contains("token-123"));
    }

    #[test]
    fn test_user_response_from_user() {
        let user_id = tachyon_core::generate_user_id();
        let user = User::new(
            user_id,
            "testuser".to_string(),
            "Test User".to_string(),
            UserRole::Writer,
        );

        let response = UserResponse::from(user);
        assert_eq!(response.username, "testuser");
        assert_eq!(response.role, "writer");
        assert!(response.is_active);
    }

    #[test]
    fn test_user_list_response_serialization() {
        let resp = UserListResponse {
            users: vec![],
            total: 0,
            page: 1,
            page_size: 20,
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("\"total\":0"));
    }

    #[test]
    fn test_update_profile_request_construction() {
        let req = UpdateProfileRequest {
            display_name: Some("New Name".to_string()),
            email: None,
        };
        assert_eq!(req.display_name.as_deref(), Some("New Name"));
        assert!(req.email.is_none());
    }
}
