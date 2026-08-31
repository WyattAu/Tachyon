//! Notion OAuth 2.0 authorization code grant flow.
//!
//! Implements the OAuth 2.0 flow for Notion integration:
//! 1. Build authorization URL for user redirect
//! 2. Exchange authorization code for access/refresh tokens
//! 3. Refresh expired tokens
//!
//! Notion OAuth endpoints:
//! - Authorization: https://api.notion.com/v1/oauth/authorize
//! - Token: https://api.notion.com/v1/oauth/token

use crate::error::{ImportExportError, ImportExportResult};
use serde::{Deserialize, Serialize};

/// Notion OAuth 2.0 token response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotionToken {
    /// The access token used to authenticate requests to the Notion API.
    pub access_token: String,
    /// The type of token (always "bearer").
    pub token_type: String,
    /// The Notion workspace this token is associated with.
    pub workspace_id: String,
    /// The name of the Notion workspace.
    pub workspace_name: String,
    /// The Notion user who authorized the integration.
    pub bot_id: String,
    /// The owner of the workspace (user or workspace object).
    #[serde(default)]
    pub owner: Option<serde_json::Value>,
    /// Scopes granted to the integration.
    #[serde(default)]
    pub scope: Option<String>,
}

/// Notion OAuth 2.0 configuration.
#[derive(Debug, Clone)]
pub struct NotionOAuthConfig {
    /// OAuth client ID (from Notion integration settings).
    pub client_id: String,
    /// OAuth client secret (from Notion integration settings).
    pub client_secret: String,
    /// Redirect URI registered with Notion.
    pub redirect_uri: String,
    /// Base URL of the Notion API (default: https://api.notion.com).
    pub api_base_url: String,
}

impl NotionOAuthConfig {
    pub fn new(client_id: String, client_secret: String, base_url: String) -> Self {
        Self {
            client_id,
            client_secret,
            redirect_uri: format!("{}/api/v1/import/notion/callback", base_url),
            api_base_url: "https://api.notion.com".to_string(),
        }
    }
}

/// Required scopes for Notion import.
pub const NOTION_SCOPES: &str = "read_content read_databases";

/// Build the Notion OAuth authorization URL.
///
/// Redirects the user to Notion's consent screen where they can grant
/// the integration access to their workspace pages and databases.
pub fn build_authorization_url(config: &NotionOAuthConfig, state: &str) -> String {
    format!(
        "{}/v1/oauth/authorize?client_id={}&redirect_uri={}&response_type=code&owner=user&state={}",
        config.api_base_url,
        urlencoding::encode(&config.client_id),
        urlencoding::encode(&config.redirect_uri),
        urlencoding::encode(state),
    )
}

/// Exchange an authorization code for an access token.
///
/// Called after the user authorizes the Notion integration and is redirected
/// back to the callback URL with an authorization code.
pub async fn exchange_code(
    client: &reqwest::Client,
    config: &NotionOAuthConfig,
    code: &str,
) -> ImportExportResult<NotionToken> {
    let credentials = base64_encode(&format!("{}:{}", config.client_id, config.client_secret));

    let resp = client
        .post(format!("{}/v1/oauth/token", config.api_base_url))
        .header("Authorization", format!("Basic {}", credentials))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .form(&[
            ("grant_type", "authorization_code"),
            ("code", code),
            ("redirect_uri", &config.redirect_uri),
        ])
        .send()
        .await
        .map_err(|e| ImportExportError::OAuth2(format!("HTTP error: {}", e)))?;

    let status = resp.status();
    let body = resp
        .text()
        .await
        .map_err(|e| ImportExportError::OAuth2(format!("Failed to read response: {}", e)))?;

    if !status.is_success() {
        return Err(ImportExportError::OAuth2(format!(
            "Token exchange failed ({}): {}",
            status, body
        )));
    }

    serde_json::from_str::<NotionToken>(&body)
        .map_err(|e| ImportExportError::OAuth2(format!("Failed to parse token: {}", e)))
}

/// Encode a value as base64.
fn base64_encode(input: &str) -> String {
    use base64::Engine;
    base64::engine::general_purpose::STANDARD.encode(input.as_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_authorization_url() {
        let config = NotionOAuthConfig::new(
            "test-client-id".to_string(),
            "test-client-secret".to_string(),
            "https://tachyon.example.com".to_string(),
        );
        let url = build_authorization_url(&config, "test-state-123");
        assert!(url.contains("client_id=test-client-id"));
        assert!(url.contains("redirect_uri="));
        assert!(url.contains("state=test-state-123"));
        assert!(url.contains("response_type=code"));
        assert!(url.contains("owner=user"));
    }

    #[test]
    fn test_notion_token_deserialization() {
        let json = r#"{
            "access_token": "ntn_test123",
            "token_type": "bearer",
            "workspace_id": "workspace-123",
            "workspace_name": "Test Workspace",
            "bot_id": "bot-456",
            "scope": "read_content read_databases"
        }"#;
        let token: NotionToken = serde_json::from_str(json).unwrap();
        assert_eq!(token.access_token, "ntn_test123");
        assert_eq!(token.workspace_name, "Test Workspace");
        assert_eq!(token.scope, Some("read_content read_databases".to_string()));
    }

    #[test]
    fn test_notion_token_with_owner() {
        let json = r#"{
            "access_token": "ntn_test456",
            "token_type": "bearer",
            "workspace_id": "ws-789",
            "workspace_name": "My Team",
            "bot_id": "bot-101",
            "owner": {"type": "user", "user": {"object": "user", "id": "user-123"}}
        }"#;
        let token: NotionToken = serde_json::from_str(json).unwrap();
        assert!(token.owner.is_some());
        assert!(token.scope.is_none());
    }
}
