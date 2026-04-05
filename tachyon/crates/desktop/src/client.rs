// Desktop client for Tachyon
// Provides HTTP client for server communication

use reqwest::Client as HttpClient;
use serde::Serialize;
use std::sync::Arc;
use tachyon_core::{ErrorResult, TachyonError};
use tokio::sync::RwLock;

/// Desktop client for Tachyon server communication
pub struct DesktopClient {
    /// HTTP client for API calls
    http_client: HttpClient,
    /// Server URL
    server_url: Arc<RwLock<String>>,
    /// Authentication token
    auth_token: Arc<RwLock<Option<String>>>,
    /// Session ID
    session_id: Arc<RwLock<Option<String>>>,
}

impl DesktopClient {
    /// Create a new desktop client
    ///
    /// # Arguments
    /// * `server_url` - Server URL
    pub fn new(server_url: impl Into<String>) -> Self {
        Self {
            http_client: HttpClient::new(),
            server_url: Arc::new(RwLock::new(server_url.into())),
            auth_token: Arc::new(RwLock::new(None)),
            session_id: Arc::new(RwLock::new(None)),
        }
    }

    /// Get server URL
    ///
    /// # Returns
    /// Server URL
    pub async fn get_server_url(&self) -> String {
        self.server_url.read().await.clone()
    }

    /// Set server URL
    ///
    /// # Arguments
    /// * `url` - Server URL
    pub async fn set_server_url(&self, url: impl Into<String>) {
        *self.server_url.write().await = url.into();
    }

    /// Get authentication token
    ///
    /// # Returns
    /// Authentication token if set
    pub async fn get_auth_token(&self) -> Option<String> {
        self.auth_token.read().await.clone()
    }

    /// Set authentication token
    ///
    /// # Arguments
    /// * `token` - Authentication token
    pub async fn set_auth_token(&self, token: Option<String>) {
        *self.auth_token.write().await = token;
    }

    /// Get session ID
    ///
    /// # Returns
    /// Session ID if set
    pub async fn get_session_id(&self) -> Option<String> {
        self.session_id.read().await.clone()
    }

    /// Set session ID
    ///
    /// # Arguments
    /// * `session_id` - Session ID
    pub async fn set_session_id(&self, session_id: Option<String>) {
        *self.session_id.write().await = session_id;
    }

    /// Make a GET request to server
    ///
    /// # Arguments
    /// * `path` - API path
    ///
    /// # Returns
    /// Response body as string
    pub async fn get(&self, path: &str) -> ErrorResult<String> {
        let server_url = self.get_server_url().await;
        let auth_token = self.get_auth_token().await;

        let url = format!(
            "{}/{}",
            server_url.trim_end_matches('/'),
            path.trim_start_matches('/')
        );

        let mut request = self.http_client.get(&url);

        if let Some(token) = auth_token {
            request = request.header("Authorization", format!("Bearer {}", token));
        }

        let response = request.send().await.map_err(|e| {
            TachyonError::internal(
                "GET_REQUEST_ERROR",
                format!("Failed to send GET request: {}", e),
            )
        })?;

        if !response.status().is_success() {
            return Err(TachyonError::internal(
                "GET_REQUEST_ERROR",
                format!("GET request failed with status: {}", response.status()),
            ));
        }

        response.text().await.map_err(|e| {
            TachyonError::internal(
                "READ_RESPONSE_ERROR",
                format!("Failed to read response: {}", e),
            )
        })
    }

    /// Make a POST request to server
    ///
    /// # Arguments
    /// * `path` - API path
    /// * `body` - Request body
    ///
    /// # Returns
    /// Response body as string
    pub async fn post(&self, path: &str, body: &impl Serialize) -> ErrorResult<String> {
        let server_url = self.get_server_url().await;
        let auth_token = self.get_auth_token().await;

        let url = format!(
            "{}/{}",
            server_url.trim_end_matches('/'),
            path.trim_start_matches('/')
        );

        let mut request = self.http_client.post(&url).json(body);

        if let Some(token) = auth_token {
            request = request.header("Authorization", format!("Bearer {}", token));
        }

        let response = request.send().await.map_err(|e| {
            TachyonError::internal(
                "POST_REQUEST_ERROR",
                format!("Failed to send POST request: {}", e),
            )
        })?;

        if !response.status().is_success() {
            return Err(TachyonError::internal(
                "POST_REQUEST_ERROR",
                format!("POST request failed with status: {}", response.status()),
            ));
        }

        response.text().await.map_err(|e| {
            TachyonError::internal(
                "READ_RESPONSE_ERROR",
                format!("Failed to read response: {}", e),
            )
        })
    }
}

impl Default for DesktopClient {
    fn default() -> Self {
        Self::new("http://localhost:8080")
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_desktop_client_new() {
        let client = DesktopClient::new("http://localhost:8080");
        let rt = tokio::runtime::Runtime::new().unwrap();
        let url = rt.block_on(client.get_server_url());
        assert_eq!(url, "http://localhost:8080");
    }

    #[test]
    fn test_desktop_client_default() {
        let client = DesktopClient::default();
        let rt = tokio::runtime::Runtime::new().unwrap();
        let url = rt.block_on(client.get_server_url());
        assert_eq!(url, "http://localhost:8080");
    }
}
