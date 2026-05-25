// Security Audit Logging
// Logs all security-relevant events: authentication, permissions, admin actions

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{error, info, warn};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AuditEventType {
    AuthenticationSuccess,
    AuthenticationFailure,
    AuthenticationLocked,
    AuthenticationExpired,
    Logout,
    SessionCreated,
    SessionRevoked,
    SessionExpired,
    PasswordChanged,
    PasswordReset,
    PasswordResetRequested,
    UserCreated,
    UserUpdated,
    UserDeleted,
    UserRoleChanged,
    UserActivated,
    UserDeactivated,
    ApiKeyCreated,
    ApiKeyRevoked,
    ApiKeyUsed,
    PermissionGranted,
    PermissionRevoked,
    RoleCreated,
    RoleUpdated,
    RoleDeleted,
    TeamCreated,
    TeamUpdated,
    TeamDeleted,
    TeamMemberAdded,
    TeamMemberRemoved,
    DocumentCreated,
    DocumentUpdated,
    DocumentDeleted,
    DocumentAccessed,
    DocumentShared,
    DocumentUnshared,
    GuestLoginAttempt,
    GuestLoginDisabled,
    RateLimitExceeded,
    SuspiciousActivity,
    SecurityHeaderViolation,
    CorsViolation,
    InputValidationFailure,
    XssAttempt,
    SqlInjectionAttempt,
    CsrfValidationFailure,
    TokenRefreshSuccess,
    TokenRefreshFailure,
    TokenRevoked,
    ImpersonationStarted,
    ImpersonationEnded,
    ConfigurationChanged,
    BackupCreated,
    BackupRestored,
    ExportCreated,
    ImportCompleted,
    NodeCreated,
    NodeUpdated,
    NodeDeleted,
    EdgeCreated,
    EdgeDeleted,
    OrganizationCreated,
    OrganizationUpdated,
    OrganizationDeleted,
    OrganizationMemberAdded,
    OrganizationMemberUpdated,
    OrganizationMemberRemoved,
    SpaceCreated,
    SpaceUpdated,
    SpaceDeleted,
    SpaceMemberAdded,
    SpaceMemberUpdated,
    SpaceMemberRemoved,
    DocumentMoved,
    WebhookCreated,
    WebhookDeleted,
    PluginInstalled,
    PluginUpdated,
    PluginUninstalled,
    SavedSearchCreated,
    SavedSearchUpdated,
    SavedSearchDeleted,
    RepositoryInitialized,
    RepositoryCloned,
    RepositoryDeleted,
    RepositoryCommitted,
    SubscriptionCreated,
    SubscriptionCancelled,
    PlanChanged,
    FileUploaded,
    ReviewCreated,
    ReviewUpdated,
    ReviewCommentAdded,
    MfaEnabled,
    MfaDisabled,
    ProjectCreated,
    ProjectUpdated,
    ProjectDeleted,
    ComponentCreated,
    ComponentDeleted,
    CatalogMemberAdded,
    CatalogMemberRemoved,
    UserRegistered,
    PasswordResetConfirmed,
    TeamMemberUpdated,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AuditSeverity {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AuditContext {
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub request_id: Option<String>,
    pub session_id: Option<String>,
    pub device_id: Option<String>,
    pub geo_location: Option<String>,
}

impl AuditContext {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_ip(mut self, ip: impl Into<String>) -> Self {
        self.ip_address = Some(ip.into());
        self
    }

    pub fn with_user_agent(mut self, user_agent: impl Into<String>) -> Self {
        self.user_agent = Some(user_agent.into());
        self
    }

    pub fn with_request_id(mut self, request_id: impl Into<String>) -> Self {
        self.request_id = Some(request_id.into());
        self
    }

    pub fn with_session_id(mut self, session_id: impl Into<String>) -> Self {
        self.session_id = Some(session_id.into());
        self
    }

    pub fn from_headers(headers: &axum::http::HeaderMap) -> Self {
        let mut context = Self::new();

        if let Some(forwarded) = headers.get("x-forwarded-for") {
            if let Ok(forwarded_str) = forwarded.to_str() {
                if let Some(first_ip) = forwarded_str.split(',').next() {
                    context = context.with_ip(first_ip.trim());
                }
            }
        } else if let Some(real_ip) = headers.get("x-real-ip") {
            if let Ok(ip_str) = real_ip.to_str() {
                context = context.with_ip(ip_str);
            }
        }

        if let Some(user_agent) = headers.get("user-agent") {
            if let Ok(ua_str) = user_agent.to_str() {
                context = context.with_user_agent(ua_str);
            }
        }

        if let Some(request_id) = headers.get("x-request-id") {
            if let Ok(rid_str) = request_id.to_str() {
                context = context.with_request_id(rid_str);
            }
        }

        context
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    pub id: String,
    pub event_type: AuditEventType,
    pub severity: AuditSeverity,
    pub timestamp: DateTime<Utc>,
    pub actor_id: Option<String>,
    pub actor_type: Option<String>,
    pub actor_username: Option<String>,
    pub target_id: Option<String>,
    pub target_type: Option<String>,
    pub action: String,
    pub description: String,
    pub context: AuditContext,
    pub metadata: BTreeMap<String, serde_json::Value>,
    pub outcome: AuditOutcome,
    pub correlation_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AuditOutcome {
    Success,
    Failure,
    Partial,
    Pending,
}

impl AuditEvent {
    pub fn new(
        event_type: AuditEventType,
        severity: AuditSeverity,
        action: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            event_type,
            severity,
            timestamp: Utc::now(),
            actor_id: None,
            actor_type: None,
            actor_username: None,
            target_id: None,
            target_type: None,
            action: action.into(),
            description: description.into(),
            context: AuditContext::default(),
            metadata: BTreeMap::new(),
            outcome: AuditOutcome::Success,
            correlation_id: None,
        }
    }

    pub fn with_actor(
        mut self,
        actor_id: impl Into<String>,
        actor_type: impl Into<String>,
    ) -> Self {
        self.actor_id = Some(actor_id.into());
        self.actor_type = Some(actor_type.into());
        self
    }

    pub fn with_actor_username(mut self, username: impl Into<String>) -> Self {
        self.actor_username = Some(username.into());
        self
    }

    pub fn with_target(
        mut self,
        target_id: impl Into<String>,
        target_type: impl Into<String>,
    ) -> Self {
        self.target_id = Some(target_id.into());
        self.target_type = Some(target_type.into());
        self
    }

    pub fn with_context(mut self, context: AuditContext) -> Self {
        self.context = context;
        self
    }

    pub fn with_metadata(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.metadata.insert(key.into(), value);
        self
    }

    pub fn with_outcome(mut self, outcome: AuditOutcome) -> Self {
        self.outcome = outcome;
        self
    }

    pub fn with_correlation_id(mut self, correlation_id: impl Into<String>) -> Self {
        self.correlation_id = Some(correlation_id.into());
        self
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_else(|_| "{}".to_string())
    }

    pub fn to_json_pretty(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_else(|_| "{}".to_string())
    }
}

#[derive(Debug, Clone)]
pub struct AuditLogger {
    store: Arc<RwLock<Vec<AuditEvent>>>,
    max_events: usize,
    enabled: bool,
    log_to_console: bool,
}

impl AuditLogger {
    pub fn new(max_events: usize) -> Self {
        Self {
            store: Arc::new(RwLock::new(Vec::with_capacity(max_events))),
            max_events,
            enabled: true,
            log_to_console: true,
        }
    }

    pub fn disabled() -> Self {
        Self {
            store: Arc::new(RwLock::new(Vec::new())),
            max_events: 0,
            enabled: false,
            log_to_console: false,
        }
    }

    pub fn with_console_logging(mut self, enabled: bool) -> Self {
        self.log_to_console = enabled;
        self
    }

    pub async fn log(&self, event: AuditEvent) {
        if !self.enabled {
            return;
        }

        if self.log_to_console {
            let json = event.to_json();
            match event.severity {
                AuditSeverity::Critical => {
                    error!(
                        target: "audit",
                        event_type = ?event.event_type,
                        actor_id = ?event.actor_id,
                        target_id = ?event.target_id,
                        outcome = ?event.outcome,
                        "{}", json
                    );
                }
                AuditSeverity::High => {
                    warn!(
                        target: "audit",
                        event_type = ?event.event_type,
                        actor_id = ?event.actor_id,
                        target_id = ?event.target_id,
                        outcome = ?event.outcome,
                        "{}", json
                    );
                }
                AuditSeverity::Medium | AuditSeverity::Low => {
                    info!(
                        target: "audit",
                        event_type = ?event.event_type,
                        actor_id = ?event.actor_id,
                        target_id = ?event.target_id,
                        outcome = ?event.outcome,
                        "{}", json
                    );
                }
            }
        }

        let mut store = self.store.write().await;
        store.push(event);

        if store.len() > self.max_events {
            store.remove(0);
        }
    }

    pub async fn get_events(&self, limit: Option<usize>) -> Vec<AuditEvent> {
        let store = self.store.read().await;
        let limit = limit.unwrap_or(100).min(store.len());
        store.iter().rev().take(limit).cloned().collect()
    }

    pub async fn get_events_by_actor(
        &self,
        actor_id: &str,
        limit: Option<usize>,
    ) -> Vec<AuditEvent> {
        let store = self.store.read().await;
        let limit = limit.unwrap_or(100);
        store
            .iter()
            .rev()
            .filter(|e| e.actor_id.as_deref() == Some(actor_id))
            .take(limit)
            .cloned()
            .collect()
    }

    pub async fn get_events_by_type(
        &self,
        event_type: AuditEventType,
        limit: Option<usize>,
    ) -> Vec<AuditEvent> {
        let store = self.store.read().await;
        let limit = limit.unwrap_or(100);
        store
            .iter()
            .rev()
            .filter(|e| e.event_type == event_type)
            .take(limit)
            .cloned()
            .collect()
    }

    pub async fn get_events_by_target(
        &self,
        target_id: &str,
        limit: Option<usize>,
    ) -> Vec<AuditEvent> {
        let store = self.store.read().await;
        let limit = limit.unwrap_or(100);
        store
            .iter()
            .rev()
            .filter(|e| e.target_id.as_deref() == Some(target_id))
            .take(limit)
            .cloned()
            .collect()
    }

    pub async fn get_failed_authentications(&self, limit: Option<usize>) -> Vec<AuditEvent> {
        let store = self.store.read().await;
        let limit = limit.unwrap_or(100);
        store
            .iter()
            .rev()
            .filter(|e| {
                matches!(
                    e.event_type,
                    AuditEventType::AuthenticationFailure
                        | AuditEventType::AuthenticationLocked
                        | AuditEventType::AuthenticationExpired
                )
            })
            .take(limit)
            .cloned()
            .collect()
    }

    pub async fn get_security_events(&self, limit: Option<usize>) -> Vec<AuditEvent> {
        let store = self.store.read().await;
        let limit = limit.unwrap_or(100);
        store
            .iter()
            .rev()
            .filter(|e| {
                matches!(
                    e.event_type,
                    AuditEventType::XssAttempt
                        | AuditEventType::SqlInjectionAttempt
                        | AuditEventType::SuspiciousActivity
                        | AuditEventType::RateLimitExceeded
                        | AuditEventType::SecurityHeaderViolation
                        | AuditEventType::CorsViolation
                        | AuditEventType::CsrfValidationFailure
                )
            })
            .take(limit)
            .cloned()
            .collect()
    }

    pub async fn clear(&self) {
        let mut store = self.store.write().await;
        store.clear();
    }

    pub async fn count(&self) -> usize {
        self.store.read().await.len()
    }
}

pub fn auth_success(
    user_id: &str,
    username: &str,
    method: &str,
    context: AuditContext,
) -> AuditEvent {
    AuditEvent::new(
        AuditEventType::AuthenticationSuccess,
        AuditSeverity::Low,
        "authentication",
        format!(
            "User '{}' authenticated successfully via {}",
            username, method
        ),
    )
    .with_actor(user_id, "user")
    .with_actor_username(username)
    .with_context(context)
    .with_metadata("method", serde_json::json!(method))
    .with_outcome(AuditOutcome::Success)
}

pub fn auth_failure(username: &str, reason: &str, context: AuditContext) -> AuditEvent {
    AuditEvent::new(
        AuditEventType::AuthenticationFailure,
        AuditSeverity::Medium,
        "authentication",
        format!("Authentication failed for user '{}': {}", username, reason),
    )
    .with_actor_username(username)
    .with_context(context)
    .with_metadata("reason", serde_json::json!(reason))
    .with_metadata("username", serde_json::json!(username))
    .with_outcome(AuditOutcome::Failure)
}

pub fn auth_locked(username: &str, reason: &str, context: AuditContext) -> AuditEvent {
    AuditEvent::new(
        AuditEventType::AuthenticationLocked,
        AuditSeverity::High,
        "authentication",
        format!("Account locked for user '{}': {}", username, reason),
    )
    .with_actor_username(username)
    .with_context(context)
    .with_metadata("reason", serde_json::json!(reason))
    .with_outcome(AuditOutcome::Failure)
}

pub fn logout(user_id: &str, username: &str, context: AuditContext) -> AuditEvent {
    AuditEvent::new(
        AuditEventType::Logout,
        AuditSeverity::Low,
        "logout",
        format!("User '{}' logged out", username),
    )
    .with_actor(user_id, "user")
    .with_actor_username(username)
    .with_context(context)
    .with_outcome(AuditOutcome::Success)
}

pub fn password_changed(user_id: &str, username: &str, context: AuditContext) -> AuditEvent {
    AuditEvent::new(
        AuditEventType::PasswordChanged,
        AuditSeverity::Medium,
        "password_change",
        format!("Password changed for user '{}'", username),
    )
    .with_actor(user_id, "user")
    .with_actor_username(username)
    .with_target(user_id, "user")
    .with_context(context)
    .with_outcome(AuditOutcome::Success)
}

pub fn password_reset_requested(email: &str, context: AuditContext) -> AuditEvent {
    AuditEvent::new(
        AuditEventType::PasswordResetRequested,
        AuditSeverity::Medium,
        "password_reset_request",
        format!("Password reset requested for email: {}", email),
    )
    .with_context(context)
    .with_metadata("email", serde_json::json!(email))
    .with_outcome(AuditOutcome::Success)
}

pub fn user_created(
    actor_id: &str,
    actor_username: &str,
    new_user_id: &str,
    new_username: &str,
    role: &str,
    context: AuditContext,
) -> AuditEvent {
    AuditEvent::new(
        AuditEventType::UserCreated,
        AuditSeverity::Medium,
        "user_create",
        format!("User '{}' created with role '{}'", new_username, role),
    )
    .with_actor(actor_id, "user")
    .with_actor_username(actor_username)
    .with_target(new_user_id, "user")
    .with_context(context)
    .with_metadata("new_username", serde_json::json!(new_username))
    .with_metadata("role", serde_json::json!(role))
    .with_outcome(AuditOutcome::Success)
}

pub fn user_role_changed(
    actor_id: &str,
    target_user_id: &str,
    old_role: &str,
    new_role: &str,
    context: AuditContext,
) -> AuditEvent {
    AuditEvent::new(
        AuditEventType::UserRoleChanged,
        AuditSeverity::High,
        "role_change",
        format!("User role changed from '{}' to '{}'", old_role, new_role),
    )
    .with_actor(actor_id, "admin")
    .with_target(target_user_id, "user")
    .with_context(context)
    .with_metadata("old_role", serde_json::json!(old_role))
    .with_metadata("new_role", serde_json::json!(new_role))
    .with_outcome(AuditOutcome::Success)
}

pub fn permission_granted(
    actor_id: &str,
    target_id: &str,
    target_type: &str,
    permission: &str,
    context: AuditContext,
) -> AuditEvent {
    AuditEvent::new(
        AuditEventType::PermissionGranted,
        AuditSeverity::High,
        "permission_grant",
        format!(
            "Permission '{}' granted to {} '{}'",
            permission, target_type, target_id
        ),
    )
    .with_actor(actor_id, "admin")
    .with_target(target_id, target_type)
    .with_context(context)
    .with_metadata("permission", serde_json::json!(permission))
    .with_outcome(AuditOutcome::Success)
}

pub fn permission_revoked(
    actor_id: &str,
    target_id: &str,
    target_type: &str,
    permission: &str,
    context: AuditContext,
) -> AuditEvent {
    AuditEvent::new(
        AuditEventType::PermissionRevoked,
        AuditSeverity::High,
        "permission_revoke",
        format!(
            "Permission '{}' revoked from {} '{}'",
            permission, target_type, target_id
        ),
    )
    .with_actor(actor_id, "admin")
    .with_target(target_id, target_type)
    .with_context(context)
    .with_metadata("permission", serde_json::json!(permission))
    .with_outcome(AuditOutcome::Success)
}

pub fn rate_limit_exceeded(ip: &str, endpoint: &str, context: AuditContext) -> AuditEvent {
    AuditEvent::new(
        AuditEventType::RateLimitExceeded,
        AuditSeverity::Medium,
        "rate_limit_exceeded",
        format!(
            "Rate limit exceeded for IP '{}' on endpoint '{}'",
            ip, endpoint
        ),
    )
    .with_context(context)
    .with_metadata("ip", serde_json::json!(ip))
    .with_metadata("endpoint", serde_json::json!(endpoint))
    .with_outcome(AuditOutcome::Failure)
}

pub fn suspicious_activity(description: &str, ip: &str, context: AuditContext) -> AuditEvent {
    AuditEvent::new(
        AuditEventType::SuspiciousActivity,
        AuditSeverity::High,
        "suspicious_activity",
        description.to_string(),
    )
    .with_context(context)
    .with_metadata("ip", serde_json::json!(ip))
    .with_outcome(AuditOutcome::Failure)
}

pub fn xss_attempt(
    ip: &str,
    endpoint: &str,
    payload_preview: &str,
    context: AuditContext,
) -> AuditEvent {
    AuditEvent::new(
        AuditEventType::XssAttempt,
        AuditSeverity::High,
        "xss_attempt",
        format!("Potential XSS attempt detected on endpoint '{}'", endpoint),
    )
    .with_context(context)
    .with_metadata("ip", serde_json::json!(ip))
    .with_metadata("endpoint", serde_json::json!(endpoint))
    .with_metadata("payload_preview", serde_json::json!(payload_preview))
    .with_outcome(AuditOutcome::Failure)
}

pub fn sql_injection_attempt(
    ip: &str,
    endpoint: &str,
    payload_preview: &str,
    context: AuditContext,
) -> AuditEvent {
    AuditEvent::new(
        AuditEventType::SqlInjectionAttempt,
        AuditSeverity::Critical,
        "sql_injection_attempt",
        format!(
            "Potential SQL injection attempt detected on endpoint '{}'",
            endpoint
        ),
    )
    .with_context(context)
    .with_metadata("ip", serde_json::json!(ip))
    .with_metadata("endpoint", serde_json::json!(endpoint))
    .with_metadata("payload_preview", serde_json::json!(payload_preview))
    .with_outcome(AuditOutcome::Failure)
}

pub fn input_validation_failure(
    ip: &str,
    endpoint: &str,
    field: &str,
    reason: &str,
    context: AuditContext,
) -> AuditEvent {
    AuditEvent::new(
        AuditEventType::InputValidationFailure,
        AuditSeverity::Low,
        "input_validation_failure",
        format!("Input validation failed for field '{}': {}", field, reason),
    )
    .with_context(context)
    .with_metadata("ip", serde_json::json!(ip))
    .with_metadata("endpoint", serde_json::json!(endpoint))
    .with_metadata("field", serde_json::json!(field))
    .with_metadata("reason", serde_json::json!(reason))
    .with_outcome(AuditOutcome::Failure)
}

pub fn api_key_used(
    user_id: &str,
    key_prefix: &str,
    endpoint: &str,
    context: AuditContext,
) -> AuditEvent {
    AuditEvent::new(
        AuditEventType::ApiKeyUsed,
        AuditSeverity::Low,
        "api_key_use",
        format!("API key '{}' used for endpoint '{}'", key_prefix, endpoint),
    )
    .with_actor(user_id, "user")
    .with_context(context)
    .with_metadata("key_prefix", serde_json::json!(key_prefix))
    .with_metadata("endpoint", serde_json::json!(endpoint))
    .with_outcome(AuditOutcome::Success)
}

pub fn api_key_created(actor_id: &str, key_prefix: &str, context: AuditContext) -> AuditEvent {
    AuditEvent::new(
        AuditEventType::ApiKeyCreated,
        AuditSeverity::Medium,
        "api_key_create",
        format!("API key '{}' created", key_prefix),
    )
    .with_actor(actor_id, "user")
    .with_context(context)
    .with_metadata("key_prefix", serde_json::json!(key_prefix))
    .with_outcome(AuditOutcome::Success)
}

pub fn api_key_revoked(actor_id: &str, key_prefix: &str, context: AuditContext) -> AuditEvent {
    AuditEvent::new(
        AuditEventType::ApiKeyRevoked,
        AuditSeverity::Medium,
        "api_key_revoke",
        format!("API key '{}' revoked", key_prefix),
    )
    .with_actor(actor_id, "user")
    .with_context(context)
    .with_metadata("key_prefix", serde_json::json!(key_prefix))
    .with_outcome(AuditOutcome::Success)
}

pub fn session_created(user_id: &str, session_id: &str, context: AuditContext) -> AuditEvent {
    AuditEvent::new(
        AuditEventType::SessionCreated,
        AuditSeverity::Low,
        "session_create",
        "New session created",
    )
    .with_actor(user_id, "user")
    .with_target(session_id, "session")
    .with_context(context)
    .with_outcome(AuditOutcome::Success)
}

pub fn session_revoked(actor_id: &str, session_id: &str, context: AuditContext) -> AuditEvent {
    AuditEvent::new(
        AuditEventType::SessionRevoked,
        AuditSeverity::Medium,
        "session_revoke",
        "Session revoked",
    )
    .with_actor(actor_id, "user")
    .with_target(session_id, "session")
    .with_context(context)
    .with_outcome(AuditOutcome::Success)
}

pub fn guest_login_attempt(
    ip: &str,
    success: bool,
    reason: Option<&str>,
    context: AuditContext,
) -> AuditEvent {
    let event_type = if success {
        AuditEventType::AuthenticationSuccess
    } else {
        AuditEventType::GuestLoginAttempt
    };

    let severity = if success {
        AuditSeverity::Low
    } else {
        AuditSeverity::Medium
    };

    let description = if success {
        "Guest login successful".to_string()
    } else {
        format!("Guest login failed: {}", reason.unwrap_or("unknown"))
    };

    let mut event = AuditEvent::new(event_type, severity, "guest_login", description)
        .with_context(context)
        .with_metadata("ip", serde_json::json!(ip));

    if let Some(r) = reason {
        event = event.with_metadata("reason", serde_json::json!(r));
    }

    event.with_outcome(if success {
        AuditOutcome::Success
    } else {
        AuditOutcome::Failure
    })
}

pub fn document_accessed(
    user_id: &str,
    document_id: &str,
    action: &str,
    context: AuditContext,
) -> AuditEvent {
    AuditEvent::new(
        AuditEventType::DocumentAccessed,
        AuditSeverity::Low,
        "document_access",
        format!("Document '{}' accessed ({})", document_id, action),
    )
    .with_actor(user_id, "user")
    .with_target(document_id, "document")
    .with_context(context)
    .with_metadata("action", serde_json::json!(action))
    .with_outcome(AuditOutcome::Success)
}

pub fn document_shared(
    actor_id: &str,
    document_id: &str,
    shared_with_id: &str,
    permission: &str,
    context: AuditContext,
) -> AuditEvent {
    AuditEvent::new(
        AuditEventType::DocumentShared,
        AuditSeverity::Medium,
        "document_share",
        format!(
            "Document '{}' shared with '{}' ({})",
            document_id, shared_with_id, permission
        ),
    )
    .with_actor(actor_id, "user")
    .with_target(document_id, "document")
    .with_context(context)
    .with_metadata("shared_with", serde_json::json!(shared_with_id))
    .with_metadata("permission", serde_json::json!(permission))
    .with_outcome(AuditOutcome::Success)
}

pub fn team_member_added(
    actor_id: &str,
    team_id: &str,
    member_id: &str,
    role: &str,
    context: AuditContext,
) -> AuditEvent {
    AuditEvent::new(
        AuditEventType::TeamMemberAdded,
        AuditSeverity::Medium,
        "team_member_add",
        format!(
            "Member '{}' added to team '{}' with role '{}'",
            member_id, team_id, role
        ),
    )
    .with_actor(actor_id, "admin")
    .with_target(team_id, "team")
    .with_context(context)
    .with_metadata("member_id", serde_json::json!(member_id))
    .with_metadata("role", serde_json::json!(role))
    .with_outcome(AuditOutcome::Success)
}

pub fn team_member_removed(
    actor_id: &str,
    team_id: &str,
    member_id: &str,
    context: AuditContext,
) -> AuditEvent {
    AuditEvent::new(
        AuditEventType::TeamMemberRemoved,
        AuditSeverity::Medium,
        "team_member_remove",
        format!("Member '{}' removed from team '{}'", member_id, team_id),
    )
    .with_actor(actor_id, "admin")
    .with_target(team_id, "team")
    .with_context(context)
    .with_metadata("member_id", serde_json::json!(member_id))
    .with_outcome(AuditOutcome::Success)
}

pub fn csrf_validation_failure(ip: &str, endpoint: &str, context: AuditContext) -> AuditEvent {
    AuditEvent::new(
        AuditEventType::CsrfValidationFailure,
        AuditSeverity::High,
        "csrf_validation_failure",
        format!("CSRF validation failed on endpoint '{}'", endpoint),
    )
    .with_context(context)
    .with_metadata("ip", serde_json::json!(ip))
    .with_metadata("endpoint", serde_json::json!(endpoint))
    .with_outcome(AuditOutcome::Failure)
}

pub fn configuration_changed(
    actor_id: &str,
    config_key: &str,
    old_value: &str,
    new_value: &str,
    context: AuditContext,
) -> AuditEvent {
    AuditEvent::new(
        AuditEventType::ConfigurationChanged,
        AuditSeverity::High,
        "config_change",
        format!("Configuration '{}' changed", config_key),
    )
    .with_actor(actor_id, "admin")
    .with_context(context)
    .with_metadata("config_key", serde_json::json!(config_key))
    .with_metadata("old_value", serde_json::json!(old_value))
    .with_metadata("new_value", serde_json::json!(new_value))
    .with_outcome(AuditOutcome::Success)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audit_event_creation() {
        let event = AuditEvent::new(
            AuditEventType::AuthenticationSuccess,
            AuditSeverity::Low,
            "login",
            "User logged in",
        );

        assert_eq!(event.event_type, AuditEventType::AuthenticationSuccess);
        assert_eq!(event.severity, AuditSeverity::Low);
        assert_eq!(event.action, "login");
        assert_eq!(event.description, "User logged in");
        assert_eq!(event.outcome, AuditOutcome::Success);
    }

    #[test]
    fn test_audit_event_builder() {
        let event = AuditEvent::new(
            AuditEventType::UserCreated,
            AuditSeverity::Medium,
            "user_create",
            "New user created",
        )
        .with_actor("user-1", "admin")
        .with_actor_username("admin")
        .with_target("user-2", "user")
        .with_metadata("role", serde_json::json!("editor"));

        assert_eq!(event.actor_id, Some("user-1".to_string()));
        assert_eq!(event.actor_type, Some("admin".to_string()));
        assert_eq!(event.target_id, Some("user-2".to_string()));
        assert_eq!(event.metadata.get("role").unwrap(), "editor");
    }

    #[test]
    fn test_audit_context_from_headers() {
        let mut headers = axum::http::HeaderMap::new();
        headers.insert("x-forwarded-for", "192.168.1.1, 10.0.0.1".parse().unwrap());
        headers.insert("user-agent", "Mozilla/5.0".parse().unwrap());
        headers.insert("x-request-id", "req-123".parse().unwrap());

        let context = AuditContext::from_headers(&headers);

        assert_eq!(context.ip_address, Some("192.168.1.1".to_string()));
        assert_eq!(context.user_agent, Some("Mozilla/5.0".to_string()));
        assert_eq!(context.request_id, Some("req-123".to_string()));
    }

    #[test]
    fn test_auth_success_event() {
        let context = AuditContext::new().with_ip("192.168.1.1");
        let event = auth_success("user-1", "john", "password", context);

        assert_eq!(event.event_type, AuditEventType::AuthenticationSuccess);
        assert_eq!(event.actor_id, Some("user-1".to_string()));
        assert_eq!(event.actor_username, Some("john".to_string()));
        assert_eq!(event.outcome, AuditOutcome::Success);
    }

    #[test]
    fn test_auth_failure_event() {
        let context = AuditContext::new().with_ip("192.168.1.1");
        let event = auth_failure("john", "invalid_password", context);

        assert_eq!(event.event_type, AuditEventType::AuthenticationFailure);
        assert_eq!(event.severity, AuditSeverity::Medium);
        assert_eq!(event.outcome, AuditOutcome::Failure);
    }

    #[test]
    fn test_audit_event_serialization() {
        let event = AuditEvent::new(
            AuditEventType::AuthenticationSuccess,
            AuditSeverity::Low,
            "login",
            "User logged in",
        )
        .with_actor("user-1", "user");

        let json = event.to_json();
        assert!(json.contains("authentication_success"));
        assert!(json.contains("user-1"));
    }

    #[tokio::test]
    async fn test_audit_logger() {
        let logger = AuditLogger::new(100);

        let event1 = AuditEvent::new(
            AuditEventType::AuthenticationSuccess,
            AuditSeverity::Low,
            "login",
            "User 1 logged in",
        );

        let event2 = AuditEvent::new(
            AuditEventType::AuthenticationFailure,
            AuditSeverity::Medium,
            "login",
            "User 2 failed login",
        );

        logger.log(event1).await;
        logger.log(event2).await;

        let events = logger.get_events(None).await;
        assert_eq!(events.len(), 2);

        let auth_failures = logger.get_failed_authentications(None).await;
        assert_eq!(auth_failures.len(), 1);
    }

    #[tokio::test]
    async fn test_audit_logger_max_events() {
        let logger = AuditLogger::new(5);

        for i in 0..10 {
            let event = AuditEvent::new(
                AuditEventType::AuthenticationSuccess,
                AuditSeverity::Low,
                "login",
                format!("Login {}", i),
            );
            logger.log(event).await;
        }

        let count = logger.count().await;
        assert_eq!(count, 5);
    }

    #[test]
    fn test_suspicious_activity_event() {
        let context = AuditContext::new().with_ip("10.0.0.1");
        let event = suspicious_activity("Multiple failed login attempts", "10.0.0.1", context);

        assert_eq!(event.event_type, AuditEventType::SuspiciousActivity);
        assert_eq!(event.severity, AuditSeverity::High);
        assert_eq!(event.outcome, AuditOutcome::Failure);
    }

    #[test]
    fn test_xss_attempt_event() {
        let context = AuditContext::new().with_ip("10.0.0.1");
        let event = xss_attempt("10.0.0.1", "/api/v1/documents", "<script>", context);

        assert_eq!(event.event_type, AuditEventType::XssAttempt);
        assert_eq!(event.severity, AuditSeverity::High);
        assert_eq!(event.metadata.get("payload_preview").unwrap(), "<script>");
    }

    #[test]
    fn test_sql_injection_attempt_event() {
        let context = AuditContext::new().with_ip("10.0.0.1");
        let event = sql_injection_attempt("10.0.0.1", "/api/v1/search", "'; DROP TABLE", context);

        assert_eq!(event.event_type, AuditEventType::SqlInjectionAttempt);
        assert_eq!(event.severity, AuditSeverity::Critical);
    }
}
