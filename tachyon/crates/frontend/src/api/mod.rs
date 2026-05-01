use crate::types::*;
use crate::websocket::WebSocketClient;
use serde::{de::DeserializeOwned, Serialize};
use std::sync::{Arc, Mutex};

pub mod activity;
pub mod auth;
pub mod billing;
pub mod documents;
pub mod files;
pub mod graph;
pub mod plugins;
pub mod projects;
pub mod search;
pub mod settings;
pub mod spaces;
pub mod ssg;
pub mod teams;
pub mod templates;

/// HTTP client for communicating with the Tachyon backend API.
#[derive(Clone)]
pub struct ApiClient {
    base_url: String,
    auth_token: Arc<Mutex<Option<String>>>,
}

impl Default for ApiClient {
    fn default() -> Self {
        let base_url = if let Some(window) = web_sys::window() {
            window
                .get("tachyonApiUrl")
                .and_then(|v| v.as_string())
                .unwrap_or_else(|| "http://localhost:8080/api/v1".to_string())
        } else {
            "http://localhost:8080/api/v1".to_string()
        };
        let client = Self {
            base_url,
            auth_token: Arc::new(Mutex::new(None)),
        };

        if let Some(window) = web_sys::window() {
            if let Ok(Some(storage)) = window.local_storage() {
                if let Ok(Some(token)) = storage.get_item("tachyon_token") {
                    if !token.is_empty() {
                        client.set_auth_token(token);
                    }
                }
            }
        }

        client
    }
}

/// API client for the Tachyon backend.
///
/// Reserved for future use: direct client instantiation (currently uses `default()`).
impl ApiClient {
    /// Create a new API client pointing at the given base URL.
    #[allow(dead_code)]
    pub fn new(base_url: &str) -> Self {
        Self {
            base_url: base_url.to_string(),
            auth_token: Arc::new(Mutex::new(None)),
        }
    }

    /// Store a bearer token for subsequent authenticated requests.
    pub fn set_auth_token(&self, token: String) {
        *self.auth_token.lock().unwrap_or_else(|e| e.into_inner()) = Some(token);
    }

    /// Remove the stored bearer token, reverting to anonymous requests.
    pub fn clear_auth_token(&self) {
        *self.auth_token.lock().unwrap_or_else(|e| e.into_inner()) = None;
    }

    /// Return a clone of the currently stored bearer token, if any.
    pub fn get_auth_token(&self) -> Option<String> {
        self.auth_token
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clone()
    }

    /// Derive the WebSocket URL from the configured HTTP base URL.
    pub fn websocket_url(&self) -> String {
        self.base_url
            .replace("http://", "ws://")
            .replace("https://", "wss://")
            .replace("/api/v1", "/ws")
    }

    /// Create a new [`WebSocketClient`] connected to the backend WebSocket endpoint.
    pub fn websocket(&self) -> WebSocketClient {
        WebSocketClient::new(&self.websocket_url())
    }

    async fn get<T: DeserializeOwned>(&self, url: &str) -> Result<T, ApiError> {
        use gloo_net::http::Request;

        let mut builder = Request::get(url);

        if let Some(token) = self.get_auth_token() {
            builder = builder.header("Authorization", &format!("Bearer {}", token));
        }

        let response = builder
            .send()
            .await
            .map_err(|e| ApiError::Network(e.to_string()))?;

        if response.ok() {
            response
                .json()
                .await
                .map_err(|e| ApiError::Serialization(e.to_string()))
        } else {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            Err(ApiError::Api(format!("HTTP {}: {}", status, text)))
        }
    }

    async fn post<T: Serialize, R: DeserializeOwned>(
        &self,
        url: &str,
        body: &T,
    ) -> Result<R, ApiError> {
        use gloo_net::http::Request;

        let mut builder = Request::post(url).header("Content-Type", "application/json");

        if let Some(token) = self.get_auth_token() {
            builder = builder.header("Authorization", &format!("Bearer {}", token));
        }

        let response = builder
            .json(body)
            .map_err(|e| ApiError::Serialization(e.to_string()))?
            .send()
            .await
            .map_err(|e| ApiError::Network(e.to_string()))?;

        if response.ok() {
            response
                .json()
                .await
                .map_err(|e| ApiError::Serialization(e.to_string()))
        } else {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            Err(ApiError::Api(format!("HTTP {}: {}", status, text)))
        }
    }

    async fn put<T: Serialize, R: DeserializeOwned>(
        &self,
        url: &str,
        body: &T,
    ) -> Result<R, ApiError> {
        use gloo_net::http::Request;

        let mut builder = Request::put(url).header("Content-Type", "application/json");

        if let Some(token) = self.get_auth_token() {
            builder = builder.header("Authorization", &format!("Bearer {}", token));
        }

        let response = builder
            .json(body)
            .map_err(|e| ApiError::Serialization(e.to_string()))?
            .send()
            .await
            .map_err(|e| ApiError::Network(e.to_string()))?;

        if response.ok() {
            response
                .json()
                .await
                .map_err(|e| ApiError::Serialization(e.to_string()))
        } else {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            Err(ApiError::Api(format!("HTTP {}: {}", status, text)))
        }
    }

    async fn delete(&self, url: &str) -> Result<(), ApiError> {
        use gloo_net::http::Request;

        let mut builder = Request::delete(url);

        if let Some(token) = self.get_auth_token() {
            builder = builder.header("Authorization", &format!("Bearer {}", token));
        }

        let response = builder
            .send()
            .await
            .map_err(|e| ApiError::Network(e.to_string()))?;

        if response.ok() {
            Ok(())
        } else {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            Err(ApiError::Api(format!("HTTP {}: {}", status, text)))
        }
    }

    async fn post_empty(&self, url: &str) -> Result<(), ApiError> {
        use gloo_net::http::Request;

        let mut builder = Request::post(url).header("Content-Type", "application/json");

        if let Some(token) = self.get_auth_token() {
            builder = builder.header("Authorization", &format!("Bearer {}", token));
        }

        let response = builder
            .send()
            .await
            .map_err(|e| ApiError::Network(e.to_string()))?;

        if response.ok() {
            Ok(())
        } else {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            Err(ApiError::Api(format!("HTTP {}: {}", status, text)))
        }
    }

    async fn post_empty_json<R: DeserializeOwned>(&self, url: &str) -> Result<R, ApiError> {
        use gloo_net::http::Request;

        let mut builder = Request::post(url).header("Content-Type", "application/json");

        if let Some(token) = self.get_auth_token() {
            builder = builder.header("Authorization", &format!("Bearer {}", token));
        }

        let response = builder
            .send()
            .await
            .map_err(|e| ApiError::Network(e.to_string()))?;

        if response.ok() {
            response
                .json()
                .await
                .map_err(|e| ApiError::Serialization(e.to_string()))
        } else {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            Err(ApiError::Api(format!("HTTP {}: {}", status, text)))
        }
    }

    async fn post_empty_json_accept_any(
        &self,
        url: &str,
        body: &impl Serialize,
    ) -> Result<(), ApiError> {
        use gloo_net::http::Request;

        let mut builder = Request::post(url).header("Content-Type", "application/json");

        if let Some(token) = self.get_auth_token() {
            builder = builder.header("Authorization", &format!("Bearer {}", token));
        }

        let response = builder
            .json(body)
            .map_err(|e| ApiError::Serialization(e.to_string()))?
            .send()
            .await
            .map_err(|e| ApiError::Network(e.to_string()))?;

        if response.ok() {
            Ok(())
        } else {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            Err(ApiError::Api(format!("HTTP {}: {}", status, text)))
        }
    }
}

/// Errors that can occur during API calls.
#[derive(Debug, Clone, thiserror::Error)]
pub enum ApiError {
    /// A network-level failure (e.g. timeout, DNS resolution).
    #[error("Network error: {0}")]
    Network(String),
    /// Failure to serialize a request body or deserialize a response.
    #[error("Serialization error: {0}")]
    Serialization(String),
    /// The server returned a non-success HTTP status code.
    #[error("API error: {0}")]
    Api(String),
    /// The requested resource was not found.
    #[error("Not found: {0}")]
    NotFound(String),
}

/// API result type.
///
/// Reserved for future use: unified error handling across API calls.
#[allow(dead_code)]
pub type ApiResult<T> = Result<T, ApiError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_error_display() {
        let err = ApiError::Network("timeout".to_string());
        assert_eq!(format!("{}", err), "Network error: timeout");
    }

    #[test]
    fn test_api_error_serialization_display() {
        let err = ApiError::Serialization("bad json".to_string());
        assert_eq!(format!("{}", err), "Serialization error: bad json");
    }

    #[test]
    fn test_api_error_api_display() {
        let err = ApiError::Api("HTTP 500: server error".to_string());
        assert_eq!(format!("{}", err), "API error: HTTP 500: server error");
    }

    #[test]
    fn test_api_error_not_found_display() {
        let err = ApiError::NotFound("document 123".to_string());
        assert_eq!(format!("{}", err), "Not found: document 123");
    }

    #[test]
    fn test_api_error_clone() {
        let err = ApiError::Network("conn reset".to_string());
        let cloned = err.clone();
        assert_eq!(format!("{}", err), format!("{}", cloned));
    }

    #[test]
    fn test_api_error_debug() {
        let err = ApiError::Api("test".to_string());
        let debug = format!("{:?}", err);
        assert!(debug.contains("Api"));
        assert!(debug.contains("test"));
    }

    #[test]
    fn test_api_client_new() {
        let client = ApiClient::new("http://example.com/api");
        assert!(client.get_auth_token().is_none());
    }

    #[test]
    fn test_api_client_token_management() {
        let client = ApiClient::new("http://example.com/api");
        assert!(client.get_auth_token().is_none());

        client.set_auth_token("test-token".to_string());
        assert_eq!(client.get_auth_token(), Some("test-token".to_string()));

        client.clear_auth_token();
        assert!(client.get_auth_token().is_none());
    }

    #[test]
    fn test_websocket_url_http() {
        let client = ApiClient::new("http://localhost:8080/api/v1");
        assert_eq!(client.websocket_url(), "ws://localhost:8080/ws");
    }

    #[test]
    fn test_websocket_url_https() {
        let client = ApiClient::new("https://example.com/api/v1");
        assert_eq!(client.websocket_url(), "wss://example.com/ws");
    }
}
