// API Client
// HTTP client for communicating with the Tachyon backend API

use crate::types::*;
use crate::websocket::WebSocketClient;
use serde::{de::DeserializeOwned, Serialize};
use std::sync::{Arc, Mutex};

/// API Client for Tachyon backend
#[derive(Clone)]
pub struct ApiClient {
    base_url: String,
    auth_token: Arc<Mutex<Option<String>>>,
}

impl Default for ApiClient {
    fn default() -> Self {
        // Backend API is on port 8080, frontend dev server on 8081
        // Can be overridden by setting window.tachyonApiUrl in JavaScript
        let base_url = if let Some(window) = web_sys::window() {
            window
                .get("tachyonApiUrl")
                .and_then(|v| v.as_string())
                .unwrap_or_else(|| "http://localhost:8080/api/v1".to_string())
        } else {
            "http://localhost:8080/api/v1".to_string()
        };
        Self::new(&base_url)
    }
}

#[allow(dead_code)]
impl ApiClient {
    /// Create a new API client
    pub fn new(base_url: &str) -> Self {
        Self {
            base_url: base_url.to_string(),
            auth_token: Arc::new(Mutex::new(None)),
        }
    }

    /// Set the authentication token
    pub fn set_auth_token(&self, token: String) {
        *self.auth_token.lock().unwrap() = Some(token);
    }

    /// Clear the authentication token
    pub fn clear_auth_token(&self) {
        *self.auth_token.lock().unwrap() = None;
    }

    /// Get the authentication token
    pub fn get_auth_token(&self) -> Option<String> {
        self.auth_token.lock().unwrap().clone()
    }

    /// Get WebSocket URL from base URL
    pub fn websocket_url(&self) -> String {
        let ws_url = self.base_url
            .replace("http://", "ws://")
            .replace("https://", "wss://")
            .replace("/api/v1", "/ws");
        ws_url
    }

    /// Create a new WebSocket client
    pub fn websocket(&self) -> WebSocketClient {
        WebSocketClient::new(&self.websocket_url())
    }

    // ========================================================================
    // Catalog API
    // ========================================================================

    // ========================================================================
    // Authentication API
    // ========================================================================

    /// Login with username and password
    pub async fn login(&self, username: &str, password: &str) -> Result<AuthenticateResponse, ApiError> {
        let url = format!("{}/auth/login", self.base_url);
        let body = LoginRequest {
            username: username.to_string(),
            password: password.to_string(),
        };
        self.post(&url, &body).await
    }

    /// Register a new account
    pub async fn register(&self, username: &str, email: &str, password: &str) -> Result<AuthenticateResponse, ApiError> {
        let url = format!("{}/auth/register", self.base_url);
        let body = serde_json::json!({
            "username": username,
            "email": email,
            "password": password,
        });
        self.post(&url, &body).await
    }

    /// Login as guest user
    pub async fn guest_login(&self) -> Result<AuthenticateResponse, ApiError> {
        let url = format!("{}/auth/guest", self.base_url);
        self.post_empty_json(&url).await
    }

    /// Get guest status configuration
    pub async fn guest_status(&self) -> Result<GuestStatusResponse, ApiError> {
        let url = format!("{}/auth/guest-status", self.base_url);
        self.get(&url).await
    }

    /// Auto-authenticate as guest if public_notes is enabled
    /// Returns true if auto-authenticated, false otherwise
    pub async fn auto_authenticate_guest(&self) -> Result<bool, ApiError> {
        // Check if already authenticated
        if self.get_auth_token().is_some() {
            return Ok(false);
        }

        // Get guest status
        let status = self.guest_status().await?;
        
        // If public notes is enabled and guest login is enabled, auto-authenticate
        if status.public_notes_enabled && status.guest_login_enabled {
            let response = self.guest_login().await?;
            if response.success {
                if let Some(token) = &response.access_token {
                    self.set_auth_token(token.clone());
                    return Ok(true);
                }
            }
        }
        
        Ok(false)
    }

    /// Check authentication status
    pub async fn auth_status(&self) -> Result<AuthStatusResponse, ApiError> {
        let url = format!("{}/auth/status", self.base_url);
        self.get(&url).await
    }

    /// Logout current user
    pub async fn logout(&self) -> Result<(), ApiError> {
        let url = format!("{}/auth/logout", self.base_url);
        self.post_empty(&url).await
    }

    // ========================================================================
    // Catalog API
    // ========================================================================

    /// Get catalog statistics (returns raw response)
    pub async fn get_catalog_stats_raw(&self) -> Result<serde_json::Value, ApiError> {
        let url = format!("{}/catalog/stats", self.base_url);
        self.get(&url).await
    }

    /// Get catalog statistics
    pub async fn get_catalog_stats(&self) -> Result<CatalogStats, ApiError> {
        let url = format!("{}/catalog/stats", self.base_url);
        let response: ApiResponse<CatalogStats> = self.get(&url).await?;
        response.data.ok_or(ApiError::NotFound("Catalog stats".to_string()))
    }

    /// List all projects
    pub async fn list_projects(&self) -> Result<Vec<Project>, ApiError> {
        let url = format!("{}/projects", self.base_url);
        let response: ApiResponse<Vec<Project>> = self.get(&url).await?;
        Ok(response.data.unwrap_or_default())
    }

    /// Get a project by ID
    pub async fn get_project(&self, id: &str) -> Result<Project, ApiError> {
        let url = format!("{}/projects/{}", self.base_url, id);
        let response: ApiResponse<Project> = self.get(&url).await?;
        response.data.ok_or(ApiError::NotFound(format!("Project {}", id)))
    }

    /// Get a project by slug
    pub async fn get_project_by_slug(&self, slug: &str) -> Result<Project, ApiError> {
        let url = format!("{}/projects/slug/{}", self.base_url, slug);
        let response: ApiResponse<Project> = self.get(&url).await?;
        response.data.ok_or(ApiError::NotFound(format!("Project with slug {}", slug)))
    }

    /// Create a new project
    pub async fn create_project(&self, request: &CreateProjectRequest) -> Result<Project, ApiError> {
        let url = format!("{}/projects", self.base_url);
        let response: ApiResponse<Project> = self.post(&url, request).await?;
        response.data.ok_or(ApiError::Api("Failed to create project".into()))
    }

    /// Update a project
    pub async fn update_project(&self, id: &str, project: &Project) -> Result<Project, ApiError> {
        let url = format!("{}/projects/{}", self.base_url, id);
        let response: ApiResponse<Project> = self.put(&url, project).await?;
        response.data.ok_or(ApiError::Api("Failed to update project".into()))
    }

    /// Delete a project
    pub async fn delete_project(&self, id: &str) -> Result<(), ApiError> {
        let url = format!("{}/projects/{}", self.base_url, id);
        self.delete(&url).await
    }

    /// List project components
    pub async fn list_project_components(&self, project_id: &str) -> Result<Vec<Component>, ApiError> {
        let url = format!("{}/projects/{}/components", self.base_url, project_id);
        let response: ApiResponse<Vec<Component>> = self.get(&url).await?;
        Ok(response.data.unwrap_or_default())
    }

    /// List project members
    pub async fn list_project_members(&self, project_id: &str) -> Result<Vec<ProjectMember>, ApiError> {
        let url = format!("{}/projects/{}/members", self.base_url, project_id);
        let response: ApiResponse<Vec<ProjectMember>> = self.get(&url).await?;
        Ok(response.data.unwrap_or_default())
    }

    // ========================================================================
    // Documents API
    // ========================================================================

    /// List documents with pagination
    pub async fn list_documents(&self, page: Option<usize>, page_size: Option<usize>) -> Result<DocumentListResponse, ApiError> {
        let mut url = format!("{}/documents?", self.base_url);
        if let Some(p) = page {
            url = format!("{}page={}&", url, p);
        }
        if let Some(ps) = page_size {
            url = format!("{}page_size={}", url, ps);
        }
        let response: DocumentListResponse = self.get(&url).await?;
        Ok(response)
    }

    /// Get a single document by ID
    pub async fn get_document(&self, document_id: &str) -> Result<Document, ApiError> {
        let url = format!("{}/documents/{}", self.base_url, document_id);
        self.get(&url).await
    }

    /// Create a new document
    pub async fn create_document(&self, data: &serde_json::Value) -> Result<Document, ApiError> {
        let url = format!("{}/documents", self.base_url);
        self.post(&url, data).await
    }

    /// Update an existing document
    pub async fn update_document(&self, document_id: &str, data: &serde_json::Value) -> Result<Document, ApiError> {
        let url = format!("{}/documents/{}", self.base_url, document_id);
        self.put(&url, data).await
    }

    // ========================================================================
    // Rendering API
    // ========================================================================

    /// Render markdown content to HTML
    pub async fn render_markdown(&self, content: &str) -> Result<RenderMarkdownResponse, ApiError> {
        let url = format!("{}/render/markdown", self.base_url);
        let body = serde_json::json!({ "content": content });
        self.post(&url, &body).await
    }

    // ========================================================================
    // Document Versions API
    // ========================================================================

    /// List document versions
    pub async fn list_versions(&self, document_id: &str) -> Result<Vec<DocumentVersion>, ApiError> {
        let url = format!("{}/documents/{}/versions", self.base_url, document_id);
        self.get(&url).await
    }

    /// Get a specific document version
    pub async fn get_version(&self, document_id: &str, version_number: i32) -> Result<DocumentVersion, ApiError> {
        let url = format!("{}/documents/{}/versions/{}", self.base_url, document_id, version_number);
        self.get(&url).await
    }

    /// Create a new document version
    pub async fn create_version(&self, document_id: &str, content: &str, commit_message: Option<&str>) -> Result<DocumentVersion, ApiError> {
        let url = format!("{}/documents/{}/versions", self.base_url, document_id);
        let body = serde_json::json!({
            "content": content,
            "commit_message": commit_message
        });
        self.post(&url, &body).await
    }

    // ========================================================================
    // Attachments API
    // ========================================================================

    /// List document attachments
    pub async fn list_attachments(&self, document_id: &str) -> Result<Vec<Attachment>, ApiError> {
        let url = format!("{}/documents/{}/attachments", self.base_url, document_id);
        self.get(&url).await
    }

    /// Upload an attachment
    pub async fn upload_attachment(&self, document_id: &str, file: &web_sys::File) -> Result<Attachment, ApiError> {
        use gloo_net::http::Request;
        
        let url = format!("{}/documents/{}/attachments", self.base_url, document_id);
        let form_data = web_sys::FormData::new().map_err(|e| ApiError::Api(format!("Failed to create FormData: {:?}", e)))?;
        form_data.append_with_blob("file", file).map_err(|e| ApiError::Api(format!("Failed to append file: {:?}", e)))?;
        
        let mut builder = Request::post(&url);
        
        if let Some(token) = self.get_auth_token() {
            builder = builder.header("Authorization", &format!("Bearer {}", token));
        }
        
        let response = builder
            .body(form_data)
            .map_err(|e| ApiError::Serialization(e.to_string()))?
            .send()
            .await
            .map_err(|e| ApiError::Network(e.to_string()))?;
        
        if response.ok() {
            response.json().await.map_err(|e| ApiError::Serialization(e.to_string()))
        } else {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            Err(ApiError::Api(format!("HTTP {}: {}", status, text)))
        }
    }

    /// Delete an attachment
    pub async fn delete_attachment(&self, document_id: &str, attachment_id: &str) -> Result<(), ApiError> {
        let url = format!("{}/documents/{}/attachments/{}", self.base_url, document_id, attachment_id);
        self.delete(&url).await
    }

    // ========================================================================
    // Templates API
    // ========================================================================

    /// List templates
    pub async fn list_templates(&self, category: Option<&str>) -> Result<Vec<DocumentTemplate>, ApiError> {
        let mut url = format!("{}/templates?", self.base_url);
        if let Some(cat) = category {
            url = format!("{}category={}", url, cat);
        }
        self.get(&url).await
    }

    /// Get a template by ID
    pub async fn get_template(&self, template_id: &str) -> Result<DocumentTemplate, ApiError> {
        let url = format!("{}/templates/{}", self.base_url, template_id);
        self.get(&url).await
    }

    /// Create a template
    pub async fn create_template(&self, request: &CreateTemplateRequest) -> Result<DocumentTemplate, ApiError> {
        let url = format!("{}/templates", self.base_url);
        self.post(&url, request).await
    }

    /// List template categories
    pub async fn list_template_categories(&self) -> Result<Vec<String>, ApiError> {
        let url = format!("{}/templates/categories", self.base_url);
        self.get(&url).await
    }

    // ========================================================================
    // Search API
    // ========================================================================

    /// Search documents with filters
    pub async fn search(
        &self,
        query: &str,
        filters: Option<&SearchFilters>,
        page: Option<i64>,
        page_size: Option<i64>,
    ) -> Result<SearchResultsResponse, ApiError> {
        let mut url = format!("{}search?q={}", self.base_url, crate::types::url_encode(query));

        if let Some(f) = filters {
            if let Some(ref ct) = f.content_type {
                url = format!("{}&content_type={}", url, ct);
            }
            if let Some(ref s) = f.status {
                url = format!("{}&status={}", url, s);
            }
            if let Some(ref v) = f.visibility {
                url = format!("{}&visibility={}", url, v);
            }
            if let Some(ref pid) = f.project_id {
                url = format!("{}&project_id={}", url, pid);
            }
            if let Some(ref aid) = f.author_id {
                url = format!("{}&author_id={}", url, aid);
            }
            if let Some(ref tags) = f.tags {
                url = format!("{}&tags={}", url, tags.join(","));
            }
            if let Some(ref df) = f.date_from {
                url = format!("{}&date_from={}", url, df);
            }
            if let Some(ref dt) = f.date_to {
                url = format!("{}&date_to={}", url, dt);
            }
        }

        if let Some(p) = page {
            url = format!("{}&page={}", url, p);
        }
        if let Some(ps) = page_size {
            url = format!("{}&page_size={}", url, ps);
        }

        self.get(&url).await
    }

    /// Global search (documents + projects)
    pub async fn global_search(
        &self,
        query: &str,
        filters: Option<&SearchFilters>,
        page: Option<i64>,
        page_size: Option<i64>,
    ) -> Result<GlobalSearchResponse, ApiError> {
        let mut url = format!("{}search/global?q={}", self.base_url, crate::types::url_encode(query));

        if let Some(f) = filters {
            if let Some(ref ct) = f.content_type {
                url = format!("{}&content_type={}", url, ct);
            }
            if let Some(ref s) = f.status {
                url = format!("{}&status={}", url, s);
            }
            if let Some(ref v) = f.visibility {
                url = format!("{}&visibility={}", url, v);
            }
            if let Some(ref pid) = f.project_id {
                url = format!("{}&project_id={}", url, pid);
            }
            if let Some(ref aid) = f.author_id {
                url = format!("{}&author_id={}", url, aid);
            }
            if let Some(ref tags) = f.tags {
                url = format!("{}&tags={}", url, tags.join(","));
            }
            if let Some(ref df) = f.date_from {
                url = format!("{}&date_from={}", url, df);
            }
            if let Some(ref dt) = f.date_to {
                url = format!("{}&date_to={}", url, dt);
            }
        }

        if let Some(p) = page {
            url = format!("{}&page={}", url, p);
        }
        if let Some(ps) = page_size {
            url = format!("{}&page_size={}", url, ps);
        }

        self.get(&url).await
    }

    // ========================================================================
    // Saved Search API
    // ========================================================================

    /// Create a saved search
    pub async fn create_saved_search(&self, request: &CreateSavedSearchRequest) -> Result<SavedSearch, ApiError> {
        let url = format!("{}/search/saved", self.base_url);
        self.post(&url, request).await
    }

    /// List saved searches
    pub async fn list_saved_searches(&self) -> Result<Vec<SavedSearch>, ApiError> {
        let url = format!("{}/search/saved", self.base_url);
        self.get(&url).await
    }

    /// Get a saved search by ID
    pub async fn get_saved_search(&self, id: &str) -> Result<SavedSearch, ApiError> {
        let url = format!("{}/search/saved/{}", self.base_url, id);
        self.get(&url).await
    }

    /// Update a saved search
    pub async fn update_saved_search(&self, id: &str, request: &UpdateSavedSearchRequest) -> Result<SavedSearch, ApiError> {
        let url = format!("{}/search/saved/{}", self.base_url, id);
        self.put(&url, request).await
    }

    /// Delete a saved search
    pub async fn delete_saved_search(&self, id: &str) -> Result<(), ApiError> {
        let url = format!("{}/search/saved/{}", self.base_url, id);
        self.delete(&url).await
    }

    // ========================================================================
    // Teams API
    // ========================================================================

    /// List all teams
    pub async fn list_teams(&self) -> Result<Vec<serde_json::Value>, ApiError> {
        let url = format!("{}/teams", self.base_url);
        self.get(&url).await
    }

    /// Create a new team
    pub async fn create_team(&self, request: &serde_json::Value) -> Result<serde_json::Value, ApiError> {
        let url = format!("{}/teams", self.base_url);
        self.post(&url, request).await
    }

    /// Get team members
    pub async fn list_team_members(&self, team_id: &str) -> Result<Vec<serde_json::Value>, ApiError> {
        let url = format!("{}/teams/{}/members", self.base_url, team_id);
        self.get(&url).await
    }

    // ========================================================================
    // Roles API
    // ========================================================================

    /// List all roles
    pub async fn list_roles(&self) -> Result<Vec<serde_json::Value>, ApiError> {
        let url = format!("{}/roles", self.base_url);
        self.get(&url).await
    }

    /// Create a new role
    pub async fn create_role(&self, request: &serde_json::Value) -> Result<serde_json::Value, ApiError> {
        let url = format!("{}/roles", self.base_url);
        self.post(&url, request).await
    }

    /// Delete a role
    pub async fn delete_role(&self, role_id: &str) -> Result<(), ApiError> {
        let url = format!("{}/roles/{}", self.base_url, role_id);
        self.delete(&url).await
    }

    // ========================================================================
    // Knowledge Graph API
    // ========================================================================

    /// List graph nodes with optional filters
    pub async fn list_graph_nodes(
        &self,
        node_type: Option<&str>,
        search: Option<&str>,
        page: Option<usize>,
        page_size: Option<usize>,
    ) -> Result<serde_json::Value, ApiError> {
        let mut url = format!("{}/nodes?", self.base_url);
        if let Some(nt) = node_type {
            url = format!("{}node_type={}&", url, nt);
        }
        if let Some(s) = search {
            url = format!("{}search={}&", url, s);
        }
        if let Some(p) = page {
            url = format!("{}page={}&", url, p);
        }
        if let Some(ps) = page_size {
            url = format!("{}page_size={}", url, ps);
        }
        self.get(&url).await
    }

    /// Get a graph node by ID
    pub async fn get_graph_node(&self, node_id: &str) -> Result<serde_json::Value, ApiError> {
        let url = format!("{}/nodes/{}", self.base_url, node_id);
        self.get(&url).await
    }

    /// Create a graph node
    pub async fn create_graph_node(&self, data: &serde_json::Value) -> Result<serde_json::Value, ApiError> {
        let url = format!("{}/nodes", self.base_url);
        self.post(&url, data).await
    }

    /// Get edges connected to a node
    pub async fn get_node_edges(&self, node_id: &str) -> Result<serde_json::Value, ApiError> {
        let url = format!("{}/nodes/{}/edges", self.base_url, node_id);
        self.get(&url).await
    }

    /// Create a graph edge
    pub async fn create_graph_edge(&self, data: &serde_json::Value) -> Result<serde_json::Value, ApiError> {
        let url = format!("{}/edges", self.base_url);
        self.post(&url, data).await
    }

    /// Query the graph (neighbors, shortest path)
    pub async fn query_graph(&self, data: &serde_json::Value) -> Result<serde_json::Value, ApiError> {
        let url = format!("{}/graph/query", self.base_url);
        self.post(&url, data).await
    }

    /// Get graph statistics
    pub async fn get_graph_stats(&self) -> Result<serde_json::Value, ApiError> {
        let url = format!("{}/graph/stats", self.base_url);
        self.get(&url).await
    }

    // ========================================================================
    // HTTP Helper Methods
    // ========================================================================

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

    async fn post<T: Serialize, R: DeserializeOwned>(&self, url: &str, body: &T) -> Result<R, ApiError> {
        use gloo_net::http::Request;

        let mut builder = Request::post(url)
            .header("Content-Type", "application/json");
        
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

    async fn put<T: Serialize, R: DeserializeOwned>(&self, url: &str, body: &T) -> Result<R, ApiError> {
        use gloo_net::http::Request;

        let mut builder = Request::put(url)
            .header("Content-Type", "application/json");
        
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

        let mut builder = Request::post(url)
            .header("Content-Type", "application/json");
        
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

        let mut builder = Request::post(url)
            .header("Content-Type", "application/json");
        
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
}

/// API Error type
#[derive(Debug, Clone)]
pub enum ApiError {
    Network(String),
    Serialization(String),
    Api(String),
    NotFound(String),
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ApiError::Network(s) => write!(f, "Network error: {}", s),
            ApiError::Serialization(s) => write!(f, "Serialization error: {}", s),
            ApiError::Api(s) => write!(f, "API error: {}", s),
            ApiError::NotFound(s) => write!(f, "Not found: {}", s),
        }
    }
}

impl std::error::Error for ApiError {}

/// Result type for API operations
/// Note: Reserved for future use when more API methods return Result types
#[allow(dead_code)]
pub type ApiResult<T> = Result<T, ApiError>;
