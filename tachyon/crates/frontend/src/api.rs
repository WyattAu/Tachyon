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
        let client = Self::new(&base_url);

        // Restore auth token from localStorage so every ApiClient instance
        // picks up the session that was persisted by the login page.
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
            "display_name": username,
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

    /// Get current user profile
    pub async fn get_current_user(&self) -> Result<serde_json::Value, ApiError> {
        let url = format!("{}/auth/me", self.base_url);
        self.get(&url).await
    }

    /// Update current user profile
    pub async fn update_profile(&self, display_name: Option<&str>, email: Option<&str>) -> Result<serde_json::Value, ApiError> {
        let mut body = serde_json::Map::new();
        if let Some(name) = display_name {
            body.insert("display_name".to_string(), serde_json::json!(name));
        }
        if let Some(email) = email {
            body.insert("email".to_string(), serde_json::json!(email));
        }
        let url = format!("{}/auth/me", self.base_url);
        self.put(&url, &body).await
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

    /// Delete a document (soft delete)
    pub async fn delete_document(&self, document_id: &str) -> Result<(), ApiError> {
        let url = format!("{}/documents/{}", self.base_url, document_id);
        self.delete(&url).await
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

    /// Update a template
    pub async fn update_template(&self, template_id: &str, request: &UpdateTemplateRequest) -> Result<DocumentTemplate, ApiError> {
        let url = format!("{}/templates/{}", self.base_url, template_id);
        self.put(&url, request).await
    }

    /// Delete a template
    pub async fn delete_template(&self, template_id: &str) -> Result<(), ApiError> {
        let url = format!("{}/templates/{}", self.base_url, template_id);
        self.delete(&url).await
    }

    /// List template categories
    pub async fn list_template_categories(&self) -> Result<Vec<String>, ApiError> {
        let url = format!("{}/templates/categories", self.base_url);
        self.get(&url).await
    }

    // ========================================================================
    // Plugins API
    // ========================================================================

    /// List plugins
    pub async fn list_plugins(&self, enabled_only: Option<bool>) -> Result<Vec<Plugin>, ApiError> {
        let mut url = format!("{}/plugins?", self.base_url);
        if let Some(e) = enabled_only {
            url = format!("{}enabled={}", url, e);
        }
        self.get(&url).await
    }

    /// Get a plugin by ID
    pub async fn get_plugin(&self, plugin_id: &str) -> Result<Plugin, ApiError> {
        let url = format!("{}/plugins/{}", self.base_url, plugin_id);
        self.get(&url).await
    }

    /// Create (install) a plugin
    pub async fn create_plugin(&self, request: &CreatePluginRequest) -> Result<Plugin, ApiError> {
        let url = format!("{}/plugins", self.base_url);
        self.post(&url, request).await
    }

    /// Update a plugin
    pub async fn update_plugin(&self, plugin_id: &str, request: &UpdatePluginRequest) -> Result<Plugin, ApiError> {
        let url = format!("{}/plugins/{}", self.base_url, plugin_id);
        self.put(&url, request).await
    }

    /// Delete (uninstall) a plugin
    pub async fn delete_plugin(&self, plugin_id: &str) -> Result<(), ApiError> {
        let url = format!("{}/plugins/{}", self.base_url, plugin_id);
        self.delete(&url).await
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

    /// Search autocomplete suggestions
    pub async fn search_suggest(&self, query: &str, limit: Option<u32>) -> Result<Vec<String>, ApiError> {
        let limit = limit.unwrap_or(10);
        let url = format!(
            "{}/search/suggest?q={}&limit={}",
            self.base_url,
            crate::types::url_encode(query),
            limit
        );
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

    /// Invite a member to a team
    pub async fn invite_team_member(&self, team_id: &str, email: &str, role: &str) -> Result<(), ApiError> {
        let url = format!("{}/teams/{}/members/invite", self.base_url, team_id);
        let body = serde_json::json!({ "email": email, "role": role });
        self.post_empty_json_accept_any(&url, &body).await
    }

    /// Remove a member from a team
    pub async fn remove_team_member(&self, team_id: &str, user_id: &str) -> Result<(), ApiError> {
        let url = format!("{}/teams/{}/members/{}", self.base_url, team_id, user_id);
        self.delete(&url).await
    }

    /// Update a team
    pub async fn update_team(&self, team_id: &str, request: &serde_json::Value) -> Result<serde_json::Value, ApiError> {
        let url = format!("{}/teams/{}", self.base_url, team_id);
        self.put(&url, request).await
    }

    /// Delete a team
    pub async fn delete_team(&self, team_id: &str) -> Result<(), ApiError> {
        let url = format!("{}/teams/{}", self.base_url, team_id);
        self.delete(&url).await
    }

    // ========================================================================
    // Billing API
    // ========================================================================

    /// List available billing plans
    pub async fn get_billing_plans(&self) -> Result<crate::types::BillingPlansResponse, ApiError> {
        let url = format!("{}/billing/plans", self.base_url);
        self.get(&url).await
    }

    /// Get current subscription for an organization
    pub async fn get_subscription(&self, org_id: &str) -> Result<crate::types::SubscriptionResponse, ApiError> {
        let url = format!("{}/billing/subscriptions/{}", self.base_url, org_id);
        self.get(&url).await
    }

    /// Create a subscription
    pub async fn create_subscription(&self, org_id: &str, plan: &str) -> Result<crate::types::SubscriptionResponse, ApiError> {
        let url = format!("{}/billing/subscriptions", self.base_url);
        let body = serde_json::json!({ "organization_id": org_id, "plan": plan });
        self.post(&url, &body).await
    }

    /// Cancel subscription
    pub async fn cancel_subscription(&self, org_id: &str) -> Result<serde_json::Value, ApiError> {
        let url = format!("{}/billing/subscriptions/{}/cancel", self.base_url, org_id);
        self.post_empty_json(&url).await
    }

    /// Get invoices for an organization
    pub async fn get_invoices(&self, org_id: &str) -> Result<crate::types::InvoicesResponse, ApiError> {
        let url = format!("{}/billing/invoices/{}", self.base_url, org_id);
        self.get(&url).await
    }

    /// Get usage metrics for an organization
    pub async fn get_usage(&self, org_id: &str) -> Result<crate::types::UsageResponse, ApiError> {
        let url = format!("{}/billing/usage/{}", self.base_url, org_id);
        self.get(&url).await
    }

    /// Create a payment mandate (TrueLayer)
    pub async fn create_mandate(&self, org_id: &str, return_url: &str) -> Result<crate::types::MandateResponse, ApiError> {
        let url = format!("{}/billing/mandates", self.base_url);
        let body = serde_json::json!({ "organization_id": org_id, "return_url": return_url });
        self.post(&url, &body).await
    }

    /// Get mandate status
    pub async fn get_mandate_status(&self, mandate_id: &str) -> Result<crate::types::MandateStatusResponse, ApiError> {
        let url = format!("{}/billing/mandates/{}", self.base_url, mandate_id);
        self.get(&url).await
    }

    /// Get payment status
    pub async fn get_payment_status(&self, payment_id: &str) -> Result<crate::types::PaymentStatusResponse, ApiError> {
        let url = format!("{}/billing/payments/{}", self.base_url, payment_id);
        self.get(&url).await
    }

    // ========================================================================
    // Audit Log API
    // ========================================================================

    /// List audit log entries
    pub async fn list_audit_logs(
        &self,
        page: Option<u32>,
        page_size: Option<u32>,
        action: Option<&str>,
        actor_id: Option<&str>,
    ) -> Result<serde_json::Value, ApiError> {
        let mut params = vec![];
        if let Some(p) = page { params.push(format!("page={}", p)); }
        if let Some(ps) = page_size { params.push(format!("page_size={}", ps)); }
        if let Some(a) = action { params.push(format!("action={}", a)); }
        if let Some(aid) = actor_id { params.push(format!("actor_id={}", aid)); }
        let query = if params.is_empty() { String::new() } else { format!("?{}", params.join("&")) };
        let url = format!("{}/audit{}", self.base_url, query);
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

    /// Get graph state at a specific point in time
    pub async fn get_graph_at_time(&self, at: &str) -> Result<serde_json::Value, ApiError> {
        let url = format!("{}/graph/at?at={}", self.base_url, at);
        self.get(&url).await
    }

    /// Get graph diff between two timestamps
    pub async fn get_graph_diff(&self, from: &str, to: &str) -> Result<serde_json::Value, ApiError> {
        let url = format!("{}/graph/diff?from={}&to={}", self.base_url, from, to);
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

    async fn post_empty_json_accept_any(&self, url: &str, body: &impl Serialize) -> Result<(), ApiError> {
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
            Ok(())
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

// ========================================================================
// Review API
// ========================================================================

impl ApiClient {
    /// Create a review request for a document
    pub async fn create_review(&self, document_id: &str, reviewer_id: &str, summary: Option<&str>) -> Result<DocumentReview, ApiError> {
        let url = format!("{}/documents/{}/reviews", self.base_url, document_id);
        let body = serde_json::json!({
            "reviewer_id": reviewer_id,
            "summary": summary,
        });
        self.post(&url, &body).await
    }

    /// List reviews for a document
    pub async fn list_reviews(&self, document_id: &str) -> Result<Vec<DocumentReview>, ApiError> {
        let url = format!("{}/documents/{}/reviews", self.base_url, document_id);
        self.get(&url).await
    }

    /// Get review status summary for a document
    pub async fn get_review_status(&self, document_id: &str) -> Result<ReviewStatusSummary, ApiError> {
        let url = format!("{}/documents/{}/reviews/status", self.base_url, document_id);
        self.get(&url).await
    }

    /// Update a review's status (approve, reject, request changes, cancel)
    pub async fn update_review(&self, document_id: &str, review_id: &str, status: &str, summary: Option<&str>) -> Result<DocumentReview, ApiError> {
        let url = format!("{}/documents/{}/reviews/{}", self.base_url, document_id, review_id);
        let body = serde_json::json!({
            "status": status,
            "summary": summary,
        });
        self.put(&url, &body).await
    }

    /// Add a comment to a review
    pub async fn create_review_comment(&self, document_id: &str, review_id: &str, author_id: &str, content: &str) -> Result<ReviewComment, ApiError> {
        let url = format!("{}/documents/{}/reviews/{}/comments", self.base_url, document_id, review_id);
        let body = serde_json::json!({
            "author_id": author_id,
            "content": content,
        });
        self.post(&url, &body).await
    }

    /// List comments on a review
    pub async fn list_review_comments(&self, document_id: &str, review_id: &str) -> Result<Vec<ReviewComment>, ApiError> {
        let url = format!("{}/documents/{}/reviews/{}/comments", self.base_url, document_id, review_id);
        self.get(&url).await
    }

    /// Get server-side diff between two versions
    #[allow(dead_code)] // Will be used when diff UI is wired into version history
    pub async fn diff_versions(&self, document_id: &str, v1: i32, v2: i32) -> Result<DocumentDiffResponse, ApiError> {
        let url = format!("{}/documents/{}/versions/{}/diff/{}", self.base_url, document_id, v1, v2);
        self.get(&url).await
    }

    // ========================================================================
    // Conflict API
    // ========================================================================

    /// Get conflict information for a document
    pub async fn get_conflict_info(&self, document_id: &str) -> Result<ConflictInfo, ApiError> {
        let url = format!("{}/documents/{}/conflict", self.base_url, document_id);
        self.get(&url).await
    }

    /// Resolve a document conflict
    pub async fn resolve_conflict(&self, document_id: &str, resolution: &str, content: Option<&str>) -> Result<serde_json::Value, ApiError> {
        let url = format!("{}/documents/{}/conflict/resolve", self.base_url, document_id);
        let body = serde_json::json!({
            "resolution": resolution,
            "content": content,
        });
        self.post(&url, &body).await
    }

    // ========================================================================
    // Activity API
    // ========================================================================

    /// List activity events
    pub async fn list_activity(&self, limit: Option<u32>, offset: Option<u32>) -> Result<ActivityListResponse, ApiError> {
        let mut params = vec![];
        if let Some(l) = limit { params.push(format!("limit={}", l)); }
        if let Some(o) = offset { params.push(format!("offset={}", o)); }
        let query = if params.is_empty() { String::new() } else { format!("?{}", params.join("&")) };
        let url = format!("{}/activity{}", self.base_url, query);
        self.get(&url).await
    }

    // ========================================================================
    // Notification API
    // ========================================================================

    /// List notifications
    pub async fn list_notifications(&self, limit: Option<u32>, include_read: bool) -> Result<NotificationListResponse, ApiError> {
        let mut params = vec![format!("include_read={}", include_read)];
        if let Some(l) = limit { params.push(format!("limit={}", l)); }
        let url = format!("{}/notifications?{}", self.base_url, params.join("&"));
        self.get(&url).await
    }

    /// Get unread notification count
    pub async fn get_unread_notification_count(&self) -> Result<u32, ApiError> {
        let url = format!("{}/notifications/unread-count", self.base_url);
        self.get(&url).await
    }

    /// Mark a notification as read
    pub async fn mark_notification_read(&self, notification_id: &str) -> Result<(), ApiError> {
        let url = format!("{}/notifications/{}/read", self.base_url, notification_id);
        self.post_empty(&url).await
    }

    /// Mark all notifications as read
    pub async fn mark_all_notifications_read(&self) -> Result<(), ApiError> {
        let url = format!("{}/notifications/read-all", self.base_url);
        self.post_empty(&url).await
    }

    /// List documents filtered by a specific tag
    pub async fn list_documents_by_tag(&self, tag: &str, page: Option<i64>, page_size: Option<i64>) -> Result<crate::types::SearchResultsResponse, ApiError> {
        let filters = crate::types::SearchFilters {
            tags: Some(vec![tag.to_string()]),
            ..Default::default()
        };
        self.search("", Some(&filters), page, page_size).await
    }

    // ========================================================================
    // Backlinks API
    // ========================================================================

    /// Get backlinks for a document
    pub async fn get_backlinks(&self, document_id: &str) -> Result<crate::types::BacklinksResponse, ApiError> {
        let url = format!("{}/documents/{}/backlinks", self.base_url, document_id);
        self.get(&url).await
    }

    // ========================================================================
    // Tags API
    // ========================================================================

    /// List all tags with document counts
    pub async fn list_tags(&self) -> Result<crate::types::TagsResponse, ApiError> {
        let url = format!("{}/tags", self.base_url);
        self.get(&url).await
    }

    // ========================================================================
    // Webhooks API
    // ========================================================================

    /// List all webhooks
    pub async fn list_webhooks(&self) -> Result<Vec<crate::types::WebhookInfo>, ApiError> {
        let url = format!("{}/webhooks", self.base_url);
        self.get(&url).await
    }

    /// Create a new webhook
    pub async fn create_webhook(
        &self,
        webhook_url: &str,
        events: Vec<&str>,
        secret: Option<&str>,
    ) -> Result<crate::types::WebhookInfo, ApiError> {
        let url = format!("{}/webhooks", self.base_url);
        let body = serde_json::json!({
            "url": webhook_url,
            "events": events,
            "secret": secret,
        });
        self.post(&url, &body).await
    }

    /// Delete a webhook
    pub async fn delete_webhook(&self, id: &str) -> Result<(), ApiError> {
        let url = format!("{}/webhooks/{}", self.base_url, id);
        self.delete(&url).await
    }

    // ========================================================================
    // Spaces API
    // ========================================================================

    /// List all spaces accessible to a user
    pub async fn list_spaces(&self, owner_id: Option<&str>) -> Result<Vec<crate::types::Space>, ApiError> {
        let mut url = format!("{}/spaces", self.base_url);
        if let Some(oid) = owner_id {
            url = format!("{}?owner_id={}", url, oid);
        }
        self.get(&url).await
    }

    /// List root spaces (no parent) for a user
    #[allow(dead_code)]
    pub async fn list_root_spaces(&self, owner_id: &str) -> Result<Vec<crate::types::Space>, ApiError> {
        let url = format!("{}/spaces/root?owner_id={}", self.base_url, owner_id);
        self.get(&url).await
    }

    /// List child spaces of a parent
    #[allow(dead_code)]
    pub async fn list_child_spaces(&self, parent_id: &str, owner_id: &str) -> Result<Vec<crate::types::Space>, ApiError> {
        let url = format!("{}/spaces/{}/children?owner_id={}", self.base_url, parent_id, owner_id);
        self.get(&url).await
    }

    /// Get a single space
    #[allow(dead_code)]
    pub async fn get_space(&self, space_id: &str) -> Result<crate::types::Space, ApiError> {
        let url = format!("{}/spaces/{}", self.base_url, space_id);
        self.get(&url).await
    }

    /// Get the default (personal) space for a user
    #[allow(dead_code)]
    pub async fn get_default_space(&self, owner_id: &str) -> Result<crate::types::Space, ApiError> {
        let url = format!("{}/spaces/default?owner_id={}", self.base_url, owner_id);
        self.get(&url).await
    }

    /// Create a new space
    pub async fn create_space(&self, req: &crate::types::CreateSpaceRequest) -> Result<crate::types::Space, ApiError> {
        let url = format!("{}/spaces", self.base_url);
        self.post(&url, req).await
    }

    /// Update a space
    pub async fn update_space(&self, space_id: &str, req: &crate::types::UpdateSpaceRequest) -> Result<crate::types::Space, ApiError> {
        let url = format!("{}/spaces/{}", self.base_url, space_id);
        self.put(&url, req).await
    }

    /// Delete a space
    pub async fn delete_space(&self, space_id: &str) -> Result<(), ApiError> {
        let url = format!("{}/spaces/{}", self.base_url, space_id);
        self.delete(&url).await
    }

    /// List members of a space
    pub async fn list_space_members(&self, space_id: &str) -> Result<Vec<crate::types::SpaceMember>, ApiError> {
        let url = format!("{}/spaces/{}/members", self.base_url, space_id);
        self.get(&url).await
    }

    /// Add a member to a space
    pub async fn add_space_member(&self, space_id: &str, req: &crate::types::AddSpaceMemberRequest) -> Result<crate::types::SpaceMember, ApiError> {
        let url = format!("{}/spaces/{}/members", self.base_url, space_id);
        self.post(&url, req).await
    }

    /// Update a space member's role
    pub async fn update_space_member(&self, space_id: &str, user_id: &str, req: &crate::types::UpdateSpaceMemberRequest) -> Result<crate::types::SpaceMember, ApiError> {
        let url = format!("{}/spaces/{}/members/{}", self.base_url, space_id, user_id);
        self.put(&url, req).await
    }

    /// Remove a member from a space
    pub async fn remove_space_member(&self, space_id: &str, user_id: &str) -> Result<(), ApiError> {
        let url = format!("{}/spaces/{}/members/{}", self.base_url, space_id, user_id);
        self.delete(&url).await
    }

    /// Move a document to a space
    #[allow(dead_code)]
    pub async fn move_document_to_space(&self, document_id: &str, space_id: Option<&str>) -> Result<(), ApiError> {
        let url = format!("{}/spaces/move-document/{}", self.base_url, document_id);
        let body = serde_json::json!({ "space_id": space_id });
        self.put(&url, &body).await
    }

    // ========================================================================
    // SSG API
    // ========================================================================

    /// Build a static site
    pub async fn build_site(&self, config: &SsgBuildRequest) -> Result<SsgBuildResponse, ApiError> {
        let url = format!("{}/ssg/build", self.base_url);
        self.post(&url, config).await
    }

    /// Download the generated site as ZIP
    pub async fn download_ssg_build(&self) -> Result<(), ApiError> {
        use gloo_net::http::Request;
        use wasm_bindgen::JsCast;

        let url = format!("{}/ssg/download", self.base_url);
        let mut builder = Request::get(&url);

        if let Some(token) = self.get_auth_token() {
            builder = builder.header("Authorization", &format!("Bearer {}", token));
        }

        let response = builder
            .send()
            .await
            .map_err(|e| ApiError::Network(e.to_string()))?;

        if response.ok() {
            let blob = response
                .binary()
                .await
                .map_err(|e| ApiError::Serialization(e.to_string()))?;

            if let Some(window) = web_sys::window() {
                let js_bytes = js_sys::Uint8Array::new_with_length(blob.len() as u32);
                js_bytes.copy_from(&blob);

                let parts = js_sys::Array::new();
                parts.push(&js_bytes.buffer());

                let bag = web_sys::BlobPropertyBag::new();
                bag.set_type("application/zip");
                let blob = web_sys::Blob::new_with_u8_array_sequence_and_options(
                    &parts,
                    &bag,
                ).map_err(|e| ApiError::Api(format!("Failed to create blob: {:?}", e)))?;

                let url = web_sys::Url::create_object_url_with_blob(&blob)
                    .map_err(|e| ApiError::Api(format!("Failed to create object URL: {:?}", e)))?;

                let document = window.document().unwrap();
                let a = document.create_element("a").unwrap();
                a.set_attribute("href", &url).unwrap();
                a.set_attribute("download", "tachyon-site.zip").unwrap();

                let body = document.body().unwrap();
                body.append_child(&a).unwrap();
                a.dyn_ref::<web_sys::HtmlElement>().unwrap().click();
                body.remove_child(&a).unwrap();
                web_sys::Url::revoke_object_url(&url).unwrap();
            }

            Ok(())
        } else {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            Err(ApiError::Api(format!("HTTP {}: {}", status, text)))
        }
    }
}
