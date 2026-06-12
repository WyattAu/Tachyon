use crate::types::*;
use crate::websocket::WebSocketClient;
use serde::{Serialize, de::DeserializeOwned};
use std::sync::{Arc, Mutex};

pub mod activity;
pub mod auth;
pub mod billing;
pub mod canvas;
pub mod documents;
pub mod files;
pub mod graph;
pub mod plugins;
pub mod projects;
pub mod push;
pub mod search;
pub mod settings;
pub mod spaces;
pub mod ssg;
pub mod teams;
pub mod templates;

// ---------------------------------------------------------------------------
// Tauri IPC bridge — calls `window.__TAURI__.core.invoke("api_proxy", ...)`
// from WASM to make HTTP requests via the native Rust reqwest client.
// This bypasses WebView CORS which blocks `tauri://` → `http://` requests.
// ---------------------------------------------------------------------------

/// Check if we're running inside a Tauri WebView.
fn is_tauri() -> bool {
    web_sys::window()
        .and_then(|w| w.get("__TAURI__"))
        .map(|v| !v.is_undefined() && !v.is_null())
        .unwrap_or(false)
}

/// Call the Tauri `api_proxy` command via IPC.
///
/// The Tauri Rust side receives: `{ method, path, body?, headers? }`
/// and makes the HTTP request with reqwest (no CORS restrictions).
///
/// Returns the response body as `serde_json::Value`.
async fn tauri_invoke(
    method: &str,
    path: &str,
    body: Option<&serde_json::Value>,
    auth_header: Option<&str>,
) -> Result<(u16, serde_json::Value), String> {
    use js_sys::{Function, Object, Promise};
    use wasm_bindgen::{JsCast, JsValue};
    use wasm_bindgen_futures::JsFuture;

    let window = web_sys::window().ok_or("No window")?;
    let tauri = js_sys::Reflect::get(&window, &JsValue::from_str("__TAURI__"))
        .map_err(|_| "__TAURI__ not found")?;
    let core = js_sys::Reflect::get(&tauri, &JsValue::from_str("core"))
        .map_err(|_| "__TAURI__.core not found")?;
    let invoke_fn = js_sys::Reflect::get(&core, &JsValue::from_str("invoke"))
        .map_err(|_| "__TAURI__.core.invoke not found")?;
    let invoke: &Function = invoke_fn.unchecked_ref();

    // Build args object: { method, path, body, headers }
    let args_obj = Object::new();
    js_sys::Reflect::set(
        &args_obj,
        &JsValue::from_str("method"),
        &JsValue::from_str(method),
    )
    .map_err(|_| "failed to set method")?;
    js_sys::Reflect::set(
        &args_obj,
        &JsValue::from_str("path"),
        &JsValue::from_str(path),
    )
    .map_err(|_| "failed to set path")?;

    if let Some(b) = body {
        let js_val = serde_wasm_bindgen::to_value(b).map_err(|e| e.to_string())?;
        js_sys::Reflect::set(&args_obj, &JsValue::from_str("body"), &js_val)
            .map_err(|_| "failed to set body")?;
    }

    if let Some(auth) = auth_header {
        let headers_obj = Object::new();
        js_sys::Reflect::set(
            &headers_obj,
            &JsValue::from_str("Authorization"),
            &JsValue::from_str(auth),
        )
        .map_err(|_| "failed to set auth header")?;
        js_sys::Reflect::set(&args_obj, &JsValue::from_str("headers"), &headers_obj)
            .map_err(|_| "failed to set headers")?;
    }

    // invoke(cmd, args) → returns a Promise
    // call2(this, arg0, arg1) → func.call(this, cmd, args)
    let cmd = JsValue::from_str("api_proxy");
    let result: Promise = invoke
        .call2(&core, &cmd, &args_obj.into())
        .map_err(|e| format!("invoke call failed: {:?}", e))?
        .unchecked_into();
    let resp: serde_json::Value = JsFuture::from(result)
        .await
        .map_err(|e| format!("invoke promise rejected: {:?}", e))
        .and_then(|v| serde_wasm_bindgen::from_value(v).map_err(|e| e.to_string()))?;

    // Extract result from Tauri 2 invoke response.
    // Tauri 2 returns: { status: "ok", data: <ApiResponse> } or { status: "error", error: "..." }
    if let Some(status_str) = resp.get("status").and_then(|v| v.as_str()) {
        if status_str == "ok" {
            let data = resp.get("data").cloned().unwrap_or(serde_json::Value::Null);
            let status = data.get("status").and_then(|v| v.as_u64()).unwrap_or(0) as u16;
            let body = data.get("body").cloned().unwrap_or(serde_json::Value::Null);
            Ok((status, body))
        } else {
            let error = resp
                .get("error")
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown Tauri invoke error");
            Err(error.to_string())
        }
    } else {
        // Fallback: try to parse as ApiResponse directly (for Tauri 1 compat)
        let status = resp.get("status").and_then(|v| v.as_u64()).unwrap_or(0) as u16;
        let body = resp.get("body").cloned().unwrap_or(serde_json::Value::Null);
        Ok((status, body))
    }
}

/// Convert a full URL like `http://localhost:8080/api/v1/documents` to just `/api/v1/documents`.
fn url_to_path(url: &str) -> String {
    // If the URL starts with the base_url prefix, strip it.
    // Otherwise return the URL as-is (it may already be a path).
    if url.contains("://") {
        // Parse out just the path portion
        if let Some(pos) = url.find("://") {
            let after_scheme = &url[pos + 3..];
            if let Some(slash) = after_scheme.find('/') {
                after_scheme[slash..].to_string()
            } else {
                "/".to_string()
            }
        } else {
            url.to_string()
        }
    } else {
        url.to_string()
    }
}

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
                .unwrap_or_else(|| {
                    // Use the current page origin instead of hardcoded localhost
                    let origin = window.location().origin().unwrap_or_default();
                    format!("{}/api/v1", origin)
                })
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
        let auth = self.get_auth_token().map(|t| format!("Bearer {}", t));

        if is_tauri() {
            let path = url_to_path(url);
            let (status, body) = tauri_invoke("GET", &path, None, auth.as_deref())
                .await
                .map_err(ApiError::Network)?;
            if (200..300).contains(&status) {
                serde_json::from_value(body).map_err(|e| ApiError::Serialization(e.to_string()))
            } else {
                Err(ApiError::Api(format!("HTTP {}: {}", status, body)))
            }
        } else {
            use gloo_net::http::Request;
            let mut builder = Request::get(url);
            if let Some(ref token) = auth {
                builder = builder.header("Authorization", token);
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
    }

    async fn post<T: Serialize, R: DeserializeOwned>(
        &self,
        url: &str,
        body: &T,
    ) -> Result<R, ApiError> {
        let auth = self.get_auth_token().map(|t| format!("Bearer {}", t));
        let body_val =
            serde_json::to_value(body).map_err(|e| ApiError::Serialization(e.to_string()))?;

        if is_tauri() {
            let path = url_to_path(url);
            let (status, resp_body) = tauri_invoke("POST", &path, Some(&body_val), auth.as_deref())
                .await
                .map_err(ApiError::Network)?;
            if (200..300).contains(&status) {
                serde_json::from_value(resp_body)
                    .map_err(|e| ApiError::Serialization(e.to_string()))
            } else {
                Err(ApiError::Api(format!("HTTP {}: {}", status, resp_body)))
            }
        } else {
            use gloo_net::http::Request;
            let mut builder = Request::post(url).header("Content-Type", "application/json");
            if let Some(ref token) = auth {
                builder = builder.header("Authorization", token);
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
    }

    async fn put<T: Serialize, R: DeserializeOwned>(
        &self,
        url: &str,
        body: &T,
    ) -> Result<R, ApiError> {
        let auth = self.get_auth_token().map(|t| format!("Bearer {}", t));
        let body_val =
            serde_json::to_value(body).map_err(|e| ApiError::Serialization(e.to_string()))?;

        if is_tauri() {
            let path = url_to_path(url);
            let (status, resp_body) = tauri_invoke("PUT", &path, Some(&body_val), auth.as_deref())
                .await
                .map_err(ApiError::Network)?;
            if (200..300).contains(&status) {
                serde_json::from_value(resp_body)
                    .map_err(|e| ApiError::Serialization(e.to_string()))
            } else {
                Err(ApiError::Api(format!("HTTP {}: {}", status, resp_body)))
            }
        } else {
            use gloo_net::http::Request;
            let mut builder = Request::put(url).header("Content-Type", "application/json");
            if let Some(ref token) = auth {
                builder = builder.header("Authorization", token);
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
    }

    async fn delete(&self, url: &str) -> Result<(), ApiError> {
        let auth = self.get_auth_token().map(|t| format!("Bearer {}", t));

        if is_tauri() {
            let path = url_to_path(url);
            let (status, body) = tauri_invoke("DELETE", &path, None, auth.as_deref())
                .await
                .map_err(ApiError::Network)?;
            if (200..300).contains(&status) {
                Ok(())
            } else {
                Err(ApiError::Api(format!("HTTP {}: {}", status, body)))
            }
        } else {
            use gloo_net::http::Request;
            let mut builder = Request::delete(url);
            if let Some(ref token) = auth {
                builder = builder.header("Authorization", token);
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
    }

    async fn post_empty(&self, url: &str) -> Result<(), ApiError> {
        let auth = self.get_auth_token().map(|t| format!("Bearer {}", t));

        if is_tauri() {
            let path = url_to_path(url);
            let (status, body) = tauri_invoke("POST", &path, None, auth.as_deref())
                .await
                .map_err(ApiError::Network)?;
            if (200..300).contains(&status) {
                Ok(())
            } else {
                Err(ApiError::Api(format!("HTTP {}: {}", status, body)))
            }
        } else {
            use gloo_net::http::Request;
            let mut builder = Request::post(url).header("Content-Type", "application/json");
            if let Some(ref token) = auth {
                builder = builder.header("Authorization", token);
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
    }

    async fn post_empty_json<R: DeserializeOwned>(&self, url: &str) -> Result<R, ApiError> {
        let auth = self.get_auth_token().map(|t| format!("Bearer {}", t));

        if is_tauri() {
            let path = url_to_path(url);
            let (status, body) = tauri_invoke("POST", &path, None, auth.as_deref())
                .await
                .map_err(ApiError::Network)?;
            if (200..300).contains(&status) {
                serde_json::from_value(body).map_err(|e| ApiError::Serialization(e.to_string()))
            } else {
                Err(ApiError::Api(format!("HTTP {}: {}", status, body)))
            }
        } else {
            use gloo_net::http::Request;
            let mut builder = Request::post(url).header("Content-Type", "application/json");
            if let Some(ref token) = auth {
                builder = builder.header("Authorization", token);
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
    }

    async fn post_empty_json_accept_any(
        &self,
        url: &str,
        body: &impl Serialize,
    ) -> Result<(), ApiError> {
        let auth = self.get_auth_token().map(|t| format!("Bearer {}", t));
        let body_val =
            serde_json::to_value(body).map_err(|e| ApiError::Serialization(e.to_string()))?;

        if is_tauri() {
            let path = url_to_path(url);
            let (status, resp_body) = tauri_invoke("POST", &path, Some(&body_val), auth.as_deref())
                .await
                .map_err(ApiError::Network)?;
            if (200..300).contains(&status) {
                Ok(())
            } else {
                Err(ApiError::Api(format!("HTTP {}: {}", status, resp_body)))
            }
        } else {
            use gloo_net::http::Request;
            let mut builder = Request::post(url).header("Content-Type", "application/json");
            if let Some(ref token) = auth {
                builder = builder.header("Authorization", token);
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
