//! Unit tests for session management
//!
//! Tests for session creation, expiration, validation, token management,
//! session builder, and session status transitions.

use chrono::Duration;
#[allow(unused_imports)]
use tachyon_core::{
    generate_session_id, generate_user_id,
    types::session::{
        Session, SessionBuilder, SessionMetadata, SessionStatus, SessionToken, SessionType,
        TokenType,
    },
};

#[allow(dead_code)]
fn create_test_session(expires_in: Duration) -> Session {
    Session::new(
        generate_session_id(),
        generate_user_id(),
        SessionType::Web,
        "test-token".to_string(),
        TokenType::Bearer,
        expires_in,
    )
}

#[test]
fn test_session_creation() {
    let session = create_test_session(Duration::hours(24));
    assert_eq!(session.status, SessionStatus::Active);
    assert_eq!(session.session_type(), SessionType::Web);
    assert!(!session.is_expired());
    assert!(session.is_valid());
}

#[test]
fn test_session_expiration() {
    let session = create_test_session(Duration::seconds(-1));
    assert!(session.is_expired());
    assert!(!session.is_valid());
}

#[test]
fn test_session_validation_ok() {
    let session = create_test_session(Duration::hours(1));
    assert!(session.validate().is_ok());
}

#[test]
fn test_session_validation_expired() {
    let session = create_test_session(Duration::seconds(-1));
    let err = session.validate().unwrap_err();
    let msg = format!("{:?}", err);
    assert!(msg.contains("SESSION_EXPIRED") || msg.contains("expired"));
}

#[test]
fn test_session_revocation() {
    let mut session = create_test_session(Duration::hours(1));
    assert!(session.is_valid());
    session.revoke();
    assert_eq!(session.status, SessionStatus::Revoked);
    assert!(!session.is_valid());
}

#[test]
fn test_session_revocation_invalidates() {
    let mut session = create_test_session(Duration::hours(1));
    session.revoke();
    let err = session.validate().unwrap_err();
    let msg = format!("{:?}", err);
    assert!(msg.contains("SESSION_INVALID") || msg.contains("Revoked"));
}

#[test]
fn test_session_extend() {
    let mut session = create_test_session(Duration::hours(1));
    let original_expires = session.expires_at;
    session.extend(Duration::hours(1));
    assert!(session.expires_at > original_expires);
    assert!(!session.is_expired());
}

#[test]
fn test_session_touch() {
    let mut session = create_test_session(Duration::hours(1));
    let original_activity = session.last_activity;
    session.touch();
    assert!(session.last_activity >= original_activity);
}

#[test]
fn test_session_idle() {
    let session = create_test_session(Duration::hours(1));
    assert!(!session.is_idle(Duration::minutes(30)));
    assert!(session.is_idle(Duration::nanoseconds(1)));
}

#[test]
fn test_session_with_metadata() {
    let session = create_test_session(Duration::hours(1))
        .with_ip_address("192.168.1.1".to_string())
        .with_user_agent("Mozilla/5.0".to_string())
        .with_device_info("Desktop".to_string());

    assert_eq!(session.metadata.ip_address.as_deref(), Some("192.168.1.1"));
    assert_eq!(session.metadata.user_agent.as_deref(), Some("Mozilla/5.0"));
    assert_eq!(session.metadata.device_info.as_deref(), Some("Desktop"));
}

#[test]
fn test_session_builder() {
    let id = generate_session_id();
    let user_id = generate_user_id();
    let session = SessionBuilder::new(id.clone(), user_id.clone(), "tok".to_string())
        .session_type(SessionType::Desktop)
        .token_type(TokenType::Jwt)
        .expires_in(Duration::hours(48))
        .ip_address("10.0.0.1".to_string())
        .user_agent("TestClient/1.0".to_string())
        .device_info("Linux".to_string())
        .build();

    assert_eq!(session.id, id);
    assert_eq!(*session.user_id(), user_id);
    assert_eq!(session.session_type(), SessionType::Desktop);
    assert_eq!(session.token.token_type, TokenType::Jwt);
    assert_eq!(session.metadata.ip_address.as_deref(), Some("10.0.0.1"));
}

#[test]
fn test_session_token_creation() {
    let token = SessionToken::new("value".to_string(), TokenType::Bearer, Duration::hours(1));
    assert_eq!(token.value, "value");
    assert_eq!(token.token_type, TokenType::Bearer);
    assert!(!token.is_expired());
}

#[test]
fn test_session_token_expired() {
    let token = SessionToken::new("value".to_string(), TokenType::Jwt, Duration::seconds(-1));
    assert!(token.is_expired());
    assert!(token.time_remaining().is_none());
}

#[test]
fn test_session_token_time_remaining() {
    let token = SessionToken::new("value".to_string(), TokenType::Bearer, Duration::hours(1));
    assert!(token.time_remaining().is_some());
    assert!(token.time_remaining().unwrap().num_seconds() > 3500);
}

#[test]
fn test_session_status_is_valid() {
    assert!(SessionStatus::Active.is_valid());
    assert!(!SessionStatus::Expired.is_valid());
    assert!(!SessionStatus::Revoked.is_valid());
}

#[test]
fn test_session_status_display() {
    assert_eq!(format!("{}", SessionStatus::Active), "active");
    assert_eq!(format!("{}", SessionStatus::Expired), "expired");
    assert_eq!(format!("{}", SessionStatus::Revoked), "revoked");
}

#[test]
fn test_session_type_display() {
    assert_eq!(format!("{}", SessionType::Desktop), "desktop");
    assert_eq!(format!("{}", SessionType::Web), "web");
    assert_eq!(format!("{}", SessionType::Api), "api");
    assert_eq!(format!("{}", SessionType::Mobile), "mobile");
}

#[test]
fn test_session_serde_roundtrip() {
    let session = create_test_session(Duration::hours(1))
        .with_ip_address("127.0.0.1".to_string());
    let json = serde_json::to_string(&session).expect("serialize");
    let de: Session = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(session.id, de.id);
    assert_eq!(session.metadata.ip_address, de.metadata.ip_address);
}

#[test]
fn test_all_session_types() {
    for st in [SessionType::Desktop, SessionType::Web, SessionType::Api, SessionType::Mobile] {
        let session = Session::new(
            generate_session_id(),
            generate_user_id(),
            st,
            "token".to_string(),
            TokenType::Bearer,
            Duration::hours(1),
        );
        assert_eq!(session.session_type(), st);
    }
}
