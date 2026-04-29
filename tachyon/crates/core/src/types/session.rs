// Session type definitions
// Represents user sessions and authentication tokens

use crate::id::SessionId;
use crate::id::UserId;
use crate::types::error::TachyonError;
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;

// ============================================================================
// Session Token
// ============================================================================

/// Session token for authentication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionToken {
    /// Token value
    pub value: String,
    /// Token type
    pub token_type: TokenType,
    /// When the token was issued
    pub issued_at: DateTime<Utc>,
    /// When the token expires
    pub expires_at: DateTime<Utc>,
}

/// Token type classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TokenType {
    /// JWT (JSON Web Token)
    #[serde(rename = "jwt")]
    Jwt,
    /// Bearer token
    #[serde(rename = "bearer")]
    Bearer,
    /// API key
    #[serde(rename = "api_key")]
    ApiKey,
}

impl SessionToken {
    /// Create a new session token
    ///
    /// # Arguments
    /// * `value` - Token value
    /// * `token_type` - Token type
    /// * `expires_in` - Duration until expiration
    pub fn new(value: String, token_type: TokenType, expires_in: Duration) -> Self {
        let now = Utc::now();
        Self {
            value,
            token_type,
            issued_at: now,
            expires_at: now + expires_in,
        }
    }

    /// Check if token is expired
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }

    /// Get remaining time until expiration
    pub fn time_remaining(&self) -> Option<Duration> {
        if self.is_expired() {
            None
        } else {
            Some(self.expires_at.signed_duration_since(Utc::now()))
        }
    }
}

// ============================================================================
// Session Type
// ============================================================================

/// Session type classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SessionType {
    /// Desktop application session
    #[serde(rename = "desktop")]
    Desktop,
    /// Web browser session
    #[serde(rename = "web")]
    Web,
    /// API session (service account)
    #[serde(rename = "api")]
    Api,
    /// Mobile application session
    #[serde(rename = "mobile")]
    Mobile,
}

impl fmt::Display for SessionType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let session_type = match self {
            Self::Desktop => "desktop",
            Self::Web => "web",
            Self::Api => "api",
            Self::Mobile => "mobile",
        };
        write!(f, "{}", session_type)
    }
}

// ============================================================================
// Session Status
// ============================================================================

/// Session status enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SessionStatus {
    /// Session is active
    #[serde(rename = "active")]
    Active,
    /// Session is expired
    #[serde(rename = "expired")]
    Expired,
    /// Session is revoked
    #[serde(rename = "revoked")]
    Revoked,
}

impl SessionStatus {
    /// Check if session is considered valid for authentication
    pub fn is_valid(&self) -> bool {
        matches!(self, Self::Active)
    }
}

impl fmt::Display for SessionStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let status = match self {
            Self::Active => "active",
            Self::Expired => "expired",
            Self::Revoked => "revoked",
        };
        write!(f, "{}", status)
    }
}

// ============================================================================
// Session Metadata
// ============================================================================

/// Session metadata for tracking session information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMetadata {
    /// User ID associated with this session
    pub user_id: UserId,
    /// Session type
    pub session_type: SessionType,
    /// Client IP address (if available)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ip_address: Option<String>,
    /// User agent string (if available)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_agent: Option<String>,
    /// Device information (if available)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub device_info: Option<String>,
}

// ============================================================================
// Session
// ============================================================================

/// User session for authentication and authorization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    /// Unique session identifier
    pub id: SessionId,
    /// Session metadata
    pub metadata: SessionMetadata,
    /// Session status
    pub status: SessionStatus,
    /// When the session was created
    pub created_at: DateTime<Utc>,
    /// When the session expires
    pub expires_at: DateTime<Utc>,
    /// Last activity timestamp
    pub last_activity: DateTime<Utc>,
    /// Session token
    pub token: SessionToken,
}

impl Session {
    /// Create a new session
    ///
    /// # Arguments
    /// * `id` - Session ID
    /// * `user_id` - User ID
    /// * `session_type` - Session type
    /// * `token_value` - Token value
    /// * `token_type` - Token type
    /// * `expires_in` - Duration until expiration
    pub fn new(
        id: SessionId,
        user_id: UserId,
        session_type: SessionType,
        token_value: String,
        token_type: TokenType,
        expires_in: Duration,
    ) -> Self {
        let now = Utc::now();
        let metadata = SessionMetadata {
            user_id,
            session_type,
            ip_address: None,
            user_agent: None,
            device_info: None,
        };
        let token = SessionToken::new(token_value, token_type, expires_in);

        Self {
            id,
            metadata,
            status: SessionStatus::Active,
            created_at: now,
            expires_at: token.expires_at,
            last_activity: now,
            token,
        }
    }

    /// Set IP address
    pub fn with_ip_address(mut self, ip_address: String) -> Self {
        self.metadata.ip_address = Some(ip_address);
        self
    }

    /// Set user agent
    pub fn with_user_agent(mut self, user_agent: String) -> Self {
        self.metadata.user_agent = Some(user_agent);
        self
    }

    /// Set device info
    pub fn with_device_info(mut self, device_info: String) -> Self {
        self.metadata.device_info = Some(device_info);
        self
    }

    /// Update last activity timestamp
    pub fn touch(&mut self) {
        self.last_activity = Utc::now();
    }

    /// Check if session is valid for authentication
    pub fn is_valid(&self) -> bool {
        self.status.is_valid() && !self.is_expired()
    }

    /// Check if session is expired
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }

    /// Check if session is idle for longer than specified duration
    pub fn is_idle(&self, max_idle_duration: Duration) -> bool {
        Utc::now().signed_duration_since(self.last_activity) > max_idle_duration
    }

    /// Revoke the session
    pub fn revoke(&mut self) {
        self.status = SessionStatus::Revoked;
        self.touch();
    }

    /// Validate session
    ///
    /// # Returns
    /// Result indicating if session is valid or error
    pub fn validate(&self) -> Result<(), TachyonError> {
        if !self.status.is_valid() {
            return Err(TachyonError::authentication(
                "SESSION_INVALID",
                format!("Session is {}", self.status),
            ));
        }

        if self.is_expired() {
            return Err(TachyonError::authentication(
                "SESSION_EXPIRED",
                "Session has expired",
            ));
        }

        Ok(())
    }

    /// Get user ID
    pub fn user_id(&self) -> &UserId {
        &self.metadata.user_id
    }

    /// Get session type
    pub fn session_type(&self) -> SessionType {
        self.metadata.session_type
    }

    /// Extend session expiration
    ///
    /// # Arguments
    /// * `additional_time` - Additional duration to add to expiration
    pub fn extend(&mut self, additional_time: Duration) {
        self.expires_at = Utc::now() + additional_time;
        self.token.expires_at = self.expires_at;
        self.touch();
    }
}

// ============================================================================
// SessionBuilder for fluent construction
// ============================================================================

/// Builder for creating Session instances
pub struct SessionBuilder {
    id: SessionId,
    user_id: UserId,
    session_type: SessionType,
    token_value: String,
    token_type: TokenType,
    expires_in: Duration,
    ip_address: Option<String>,
    user_agent: Option<String>,
    device_info: Option<String>,
}

impl SessionBuilder {
    /// Create a new SessionBuilder
    ///
    /// # Arguments
    /// * `id` - Session ID
    /// * `user_id` - User ID
    /// * `token_value` - Token value
    pub fn new(id: SessionId, user_id: UserId, token_value: String) -> Self {
        Self {
            id,
            user_id,
            session_type: SessionType::Web,
            token_value,
            token_type: TokenType::Bearer,
            expires_in: Duration::hours(24),
            ip_address: None,
            user_agent: None,
            device_info: None,
        }
    }

    /// Set session type
    pub fn session_type(mut self, session_type: SessionType) -> Self {
        self.session_type = session_type;
        self
    }

    /// Set token type
    pub fn token_type(mut self, token_type: TokenType) -> Self {
        self.token_type = token_type;
        self
    }

    /// Set expiration duration
    pub fn expires_in(mut self, duration: Duration) -> Self {
        self.expires_in = duration;
        self
    }

    /// Set IP address
    pub fn ip_address(mut self, ip_address: String) -> Self {
        self.ip_address = Some(ip_address);
        self
    }

    /// Set user agent
    pub fn user_agent(mut self, user_agent: String) -> Self {
        self.user_agent = Some(user_agent);
        self
    }

    /// Set device info
    pub fn device_info(mut self, device_info: String) -> Self {
        self.device_info = Some(device_info);
        self
    }

    /// Build the Session
    pub fn build(self) -> Session {
        let mut session = Session::new(
            self.id,
            self.user_id,
            self.session_type,
            self.token_value,
            self.token_type,
            self.expires_in,
        );

        if let Some(ip_address) = self.ip_address {
            session = session.with_ip_address(ip_address);
        }

        if let Some(user_agent) = self.user_agent {
            session = session.with_user_agent(user_agent);
        }

        if let Some(device_info) = self.device_info {
            session = session.with_device_info(device_info);
        }

        session
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_token_creation() {
        let token = SessionToken::new(
            "test-token".to_string(),
            TokenType::Bearer,
            Duration::hours(1),
        );

        assert_eq!(token.value, "test-token");
        assert_eq!(token.token_type, TokenType::Bearer);
        assert!(!token.is_expired());
    }

    #[test]
    fn test_session_token_expiration() {
        let token = SessionToken::new(
            "test-token".to_string(),
            TokenType::Bearer,
            Duration::seconds(-1),
        );

        assert!(token.is_expired());
    }

    #[test]
    fn test_session_creation() {
        let session_id = crate::id::generate_session_id();
        let user_id = crate::id::generate_user_id();

        let session = Session::new(
            session_id,
            user_id,
            SessionType::Web,
            "test-token".to_string(),
            TokenType::Bearer,
            Duration::hours(1),
        );

        assert_eq!(session.id, session_id);
        assert_eq!(session.user_id(), &user_id);
        assert_eq!(session.status, SessionStatus::Active);
        assert!(session.is_valid());
    }

    #[test]
    fn test_session_validation() {
        let session_id = crate::id::generate_session_id();
        let user_id = crate::id::generate_user_id();

        let mut session = Session::new(
            session_id,
            user_id,
            SessionType::Web,
            "test-token".to_string(),
            TokenType::Bearer,
            Duration::hours(1),
        );

        assert!(session.validate().is_ok());

        session.revoke();
        assert!(session.validate().is_err());
    }

    #[test]
    fn test_session_revocation() {
        let session_id = crate::id::generate_session_id();
        let user_id = crate::id::generate_user_id();

        let mut session = Session::new(
            session_id,
            user_id,
            SessionType::Web,
            "test-token".to_string(),
            TokenType::Bearer,
            Duration::hours(1),
        );

        assert!(session.is_valid());
        session.revoke();
        assert!(!session.is_valid());
    }

    #[test]
    fn test_session_idle() {
        let session_id = crate::id::generate_session_id();
        let user_id = crate::id::generate_user_id();

        let session = Session::new(
            session_id,
            user_id,
            SessionType::Web,
            "test-token".to_string(),
            TokenType::Bearer,
            Duration::hours(1),
        );

        // Not idle
        assert!(!session.is_idle(Duration::minutes(30)));

        // Note: Can't easily test idle since we can't modify last_activity directly
        // This would require mocking time or using a different approach
    }

    #[test]
    fn test_session_builder() {
        let session_id = crate::id::generate_session_id();
        let user_id = crate::id::generate_user_id();

        let session = SessionBuilder::new(session_id, user_id, "test-token".to_string())
            .session_type(SessionType::Desktop)
            .token_type(TokenType::Jwt)
            .ip_address("192.168.1.1".to_string())
            .user_agent("Tachyon Desktop/1.0".to_string())
            .build();

        assert_eq!(session.id, session_id);
        assert_eq!(session.session_type(), SessionType::Desktop);
        assert_eq!(session.token.token_type, TokenType::Jwt);
        assert_eq!(session.metadata.ip_address, Some("192.168.1.1".to_string()));
    }

    #[test]
    fn test_session_extend() {
        let session_id = crate::id::generate_session_id();
        let user_id = crate::id::generate_user_id();

        let mut session = Session::new(
            session_id,
            user_id,
            SessionType::Web,
            "test-token".to_string(),
            TokenType::Bearer,
            Duration::hours(1),
        );

        let original_expires_at = session.expires_at;
        session.extend(Duration::hours(1));

        assert!(session.expires_at > original_expires_at);
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
    }
}
