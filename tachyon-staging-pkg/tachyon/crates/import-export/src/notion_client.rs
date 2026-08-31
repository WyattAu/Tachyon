//! Notion API client with pagination, rate limiting, and retry support.
//!
//! Notion API v1 endpoints used:
//! - GET /v1/users/me - verify token
//! - GET /v1/search - list pages/databases
//! - GET /v1/pages/{page_id} - get page
//! - GET /v1/blocks/{block_id}/children - get block children
//! - GET /v1/databases/{database_id} - get database
//! - GET /v1/databases/{database_id}/query - query database

use crate::error::{ImportExportError, ImportExportResult};
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Default Notion API base URL.
pub const NOTION_API_BASE: &str = "https://api.notion.com";

/// Notion API version header value.
pub const NOTION_API_VERSION: &str = "2022-06-28";

/// Maximum number of retries for transient errors.
const MAX_RETRIES: u32 = 3;

/// Initial retry delay in milliseconds.
const INITIAL_RETRY_DELAY_MS: u64 = 1000;

/// Rate limit: 3 requests per second.
const RATE_LIMIT_PER_SECOND: u32 = 3;

/// Notion pagination response wrapper.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotionListResponse<T> {
    pub object: String,
    #[serde(default)]
    pub results: Vec<T>,
    /// Whether more results are available.
    pub has_more: bool,
    /// Cursor for the next page (None if no more results).
    #[serde(rename = "next_cursor")]
    pub next_cursor: Option<String>,
}

/// Notion block representation (simplified for import).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NotionBlock {
    pub object: String,
    pub id: String,
    pub parent: Option<serde_json::Value>,
    #[serde(rename = "type")]
    pub block_type: String,
    /// Block type-specific content (keyed by block_type).
    #[serde(flatten)]
    pub content: std::collections::HashMap<String, serde_json::Value>,
    #[serde(default)]
    pub has_children: bool,
    #[serde(default)]
    pub archived: bool,
    #[serde(rename = "created_time", default)]
    pub created_time: Option<String>,
    #[serde(rename = "last_edited_time", default)]
    pub last_edited_time: Option<String>,
}

/// Notion page representation (simplified for import).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NotionPage {
    pub object: String,
    pub id: String,
    pub parent: Option<serde_json::Value>,
    #[serde(default)]
    pub archived: bool,
    #[serde(rename = "created_time", default)]
    pub created_time: Option<String>,
    #[serde(rename = "last_edited_time", default)]
    pub last_edited_time: Option<String>,
    #[serde(default)]
    pub properties: serde_json::Value,
    #[serde(default)]
    pub title: Option<Vec<RichText>>,
}

/// Rich text element (used in Notion text blocks).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RichText {
    #[serde(rename = "type")]
    pub text_type: String,
    #[serde(default)]
    pub plain_text: Option<String>,
    #[serde(default)]
    pub annotations: Option<serde_json::Value>,
    #[serde(default)]
    pub href: Option<String>,
}

/// Notion API client with rate limiting and retry logic.
pub struct NotionClient {
    client: reqwest::Client,
    api_base_url: String,
    access_token: String,
    /// Timestamp of last request for rate limiting.
    last_request_time: tokio::sync::Mutex<Option<std::time::Instant>>,
    /// Number of requests made in current second window.
    request_count: tokio::sync::Mutex<u32>,
    /// Window start for rate limiting.
    rate_limit_window: tokio::sync::Mutex<std::time::Instant>,
}

impl NotionClient {
    /// Create a new Notion API client.
    pub fn new(access_token: String) -> Self {
        Self {
            client: reqwest::Client::builder()
                .timeout(Duration::from_secs(30))
                .build()
                .expect("Failed to create HTTP client"),
            api_base_url: NOTION_API_BASE.to_string(),
            access_token,
            last_request_time: tokio::sync::Mutex::new(None),
            request_count: tokio::sync::Mutex::new(0),
            rate_limit_window: tokio::sync::Mutex::new(std::time::Instant::now()),
        }
    }

    /// Create a new Notion API client with a custom base URL.
    pub fn with_base_url(access_token: String, api_base_url: String) -> Self {
        Self {
            client: reqwest::Client::builder()
                .timeout(Duration::from_secs(30))
                .build()
                .expect("Failed to create HTTP client"),
            api_base_url,
            access_token,
            last_request_time: tokio::sync::Mutex::new(None),
            request_count: tokio::sync::Mutex::new(0),
            rate_limit_window: tokio::sync::Mutex::new(std::time::Instant::now()),
        }
    }

    /// Enforce rate limiting: max 3 requests per second.
    async fn rate_limit(&self) {
        let mut window = self.rate_limit_window.lock().await;
        let now = std::time::Instant::now();

        // Reset window if more than 1 second has passed
        if now.duration_since(*window) > Duration::from_secs(1) {
            *window = now;
            let mut count = self.request_count.lock().await;
            *count = 0;
        }

        let count = self.request_count.lock().await;
        if *count >= RATE_LIMIT_PER_SECOND {
            // Wait until window resets
            let wait = Duration::from_secs(1) - now.duration_since(*window);
            drop(count);
            drop(window);
            tokio::time::sleep(wait).await;
            // After sleeping, reset window
            let mut window = self.rate_limit_window.lock().await;
            *window = std::time::Instant::now();
            let mut count = self.request_count.lock().await;
            *count = 0;
        }

        let mut count = self.request_count.lock().await;
        *count += 1;
        drop(count);
        let mut last = self.last_request_time.lock().await;
        *last = Some(now);
    }

    /// Make a GET request with retry logic and rate limiting.
    async fn get<T: serde::de::DeserializeOwned>(&self, path: &str) -> ImportExportResult<T> {
        let url = format!("{}{}", self.api_base_url, path);

        let mut last_error = None;
        let mut delay = INITIAL_RETRY_DELAY_MS;

        for attempt in 0..MAX_RETRIES {
            self.rate_limit().await;

            let resp = self
                .client
                .get(&url)
                .header("Authorization", format!("Bearer {}", self.access_token))
                .header("Notion-Version", NOTION_API_VERSION)
                .send()
                .await
                .map_err(|e| ImportExportError::NotionApi(format!("HTTP error: {}", e)))?;

            let status = resp.status();

            if status.is_success() {
                return resp.json().await.map_err(|e| {
                    ImportExportError::NotionApi(format!("Failed to parse response: {}", e))
                });
            }

            // Parse rate limit error
            if status.as_u16() == 429 {
                let retry_after = resp
                    .headers()
                    .get("Retry-After")
                    .and_then(|v| v.to_str().ok())
                    .and_then(|v| v.parse::<u64>().ok())
                    .unwrap_or(delay / 1000 + 1);

                tracing::warn!(
                    "Notion API rate limited (attempt {}/{}), retry after {}s",
                    attempt + 1,
                    MAX_RETRIES,
                    retry_after
                );

                tokio::time::sleep(Duration::from_secs(retry_after)).await;
                delay *= 2;
                continue;
            }

            // 5xx errors are transient
            if status.is_server_error() {
                let body = resp.text().await.unwrap_or_default();
                last_error = Some(ImportExportError::NotionApi(format!(
                    "Server error ({}): {}",
                    status, body
                )));
                tracing::warn!(
                    "Notion API server error (attempt {}/{}), retrying in {}ms: {}",
                    attempt + 1,
                    MAX_RETRIES,
                    delay,
                    body
                );
                tokio::time::sleep(Duration::from_millis(delay)).await;
                delay *= 2;
                continue;
            }

            // Client errors (except 429) are not retryable
            let body = resp.text().await.unwrap_or_default();
            return Err(ImportExportError::NotionApi(format!(
                "Request failed ({}): {}",
                status, body
            )));
        }

        Err(last_error
            .unwrap_or_else(|| ImportExportError::NotionApi("Max retries exceeded".to_string())))
    }

    /// Verify the access token by fetching the bot user.
    pub async fn verify_token(&self) -> ImportExportResult<serde_json::Value> {
        self.get("/v1/users/me").await
    }

    /// Search for all pages the integration has access to.
    /// Uses cursor-based pagination to fetch all results.
    pub async fn search_all_pages(&self) -> ImportExportResult<Vec<NotionPage>> {
        let mut all_pages = Vec::new();
        let mut cursor: Option<String> = None;

        loop {
            let body = if let Some(ref c) = cursor {
                serde_json::json!({
                    "filter": {"value": "page", "property": "object"},
                    "start_cursor": c,
                    "page_size": 100
                })
            } else {
                serde_json::json!({
                    "filter": {"value": "page", "property": "object"},
                    "page_size": 100
                })
            };

            let url = format!("{}/v1/search", self.api_base_url);
            self.rate_limit().await;

            let resp = self
                .client
                .post(&url)
                .header("Authorization", format!("Bearer {}", self.access_token))
                .header("Notion-Version", NOTION_API_VERSION)
                .json(&body)
                .send()
                .await
                .map_err(|e| ImportExportError::NotionApi(format!("HTTP error: {}", e)))?;

            let status = resp.status();
            if !status.is_success() {
                let text = resp.text().await.unwrap_or_default();
                return Err(ImportExportError::NotionApi(format!(
                    "Search failed ({}): {}",
                    status, text
                )));
            }

            let page_resp: NotionListResponse<NotionPage> = resp.json().await.map_err(|e| {
                ImportExportError::NotionApi(format!("Failed to parse search response: {}", e))
            })?;

            all_pages.extend(page_resp.results);

            if !page_resp.has_more {
                break;
            }
            cursor = page_resp.next_cursor;
        }

        Ok(all_pages)
    }

    /// Get a page by ID.
    pub async fn get_page(&self, page_id: &str) -> ImportExportResult<NotionPage> {
        self.get(&format!("/v1/pages/{}", page_id)).await
    }

    /// Get all children blocks of a block (page or block with children).
    /// Uses cursor-based pagination to fetch all results.
    pub async fn get_block_children(&self, block_id: &str) -> ImportExportResult<Vec<NotionBlock>> {
        let mut all_blocks = Vec::new();
        let mut cursor: Option<String> = None;

        loop {
            let mut path = format!("/v1/blocks/{}/children?page_size=100", block_id);
            if let Some(ref c) = cursor {
                path = format!("{}&start_cursor={}", path, urlencoding::encode(c));
            }

            let blocks: NotionListResponse<NotionBlock> = self.get(&path).await?;

            all_blocks.extend(blocks.results);

            if !blocks.has_more {
                break;
            }
            cursor = blocks.next_cursor;
        }

        Ok(all_blocks)
    }

    /// Get all children blocks recursively (for nested blocks like toggle, list items).
    pub async fn get_block_children_recursive(
        &self,
        block_id: &str,
    ) -> ImportExportResult<Vec<NotionBlock>> {
        let mut stack = vec![block_id.to_string()];
        let mut all_blocks = Vec::new();

        while let Some(current_id) = stack.pop() {
            let children = self.get_block_children(&current_id).await?;
            for block in children {
                if block.has_children {
                    stack.push(block.id.clone());
                }
                all_blocks.push(block);
            }
        }

        Ok(all_blocks)
    }

    /// Query a Notion database for all entries.
    /// Uses cursor-based pagination to fetch all results.
    pub async fn query_database(
        &self,
        database_id: &str,
    ) -> ImportExportResult<Vec<serde_json::Value>> {
        let mut all_results = Vec::new();
        let mut cursor: Option<String> = None;

        loop {
            let body = if let Some(ref c) = cursor {
                serde_json::json!({
                    "start_cursor": c,
                    "page_size": 100
                })
            } else {
                serde_json::json!({
                    "page_size": 100
                })
            };

            let url = format!("{}/v1/databases/{}/query", self.api_base_url, database_id);
            self.rate_limit().await;

            let resp = self
                .client
                .post(&url)
                .header("Authorization", format!("Bearer {}", self.access_token))
                .header("Notion-Version", NOTION_API_VERSION)
                .json(&body)
                .send()
                .await
                .map_err(|e| ImportExportError::NotionApi(format!("HTTP error: {}", e)))?;

            let status = resp.status();
            if !status.is_success() {
                let text = resp.text().await.unwrap_or_default();
                return Err(ImportExportError::NotionApi(format!(
                    "Database query failed ({}): {}",
                    status, text
                )));
            }

            let query_resp: NotionListResponse<serde_json::Value> =
                resp.json().await.map_err(|e| {
                    ImportExportError::NotionApi(format!("Failed to parse query response: {}", e))
                })?;

            all_results.extend(query_resp.results);

            if !query_resp.has_more {
                break;
            }
            cursor = query_resp.next_cursor;
        }

        Ok(all_results)
    }

    /// Get a database by ID.
    pub async fn get_database(&self, database_id: &str) -> ImportExportResult<serde_json::Value> {
        self.get(&format!("/v1/databases/{}", database_id)).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_notion_list_response_deserialization() {
        let json = r#"{
            "object": "list",
            "results": [],
            "has_more": false,
            "next_cursor": null
        }"#;
        let resp: NotionListResponse<NotionBlock> = serde_json::from_str(json).unwrap();
        assert_eq!(resp.object, "list");
        assert!(!resp.has_more);
        assert!(resp.next_cursor.is_none());
        assert!(resp.results.is_empty());
    }

    #[test]
    fn test_notion_list_response_with_cursor() {
        let json = r#"{
            "object": "list",
            "results": [],
            "has_more": true,
            "next_cursor": "abc-123"
        }"#;
        let resp: NotionListResponse<serde_json::Value> = serde_json::from_str(json).unwrap();
        assert!(resp.has_more);
        assert_eq!(resp.next_cursor.as_deref(), Some("abc-123"));
    }

    #[test]
    fn test_notion_block_deserialization() {
        let json = r#"{
            "object": "block",
            "id": "block-123",
            "type": "paragraph",
            "paragraph": {
                "rich_text": [{"type": "text", "text": {"content": "Hello world"}}]
            },
            "has_children": false,
            "archived": false,
            "created_time": "2024-01-15T10:00:00Z",
            "last_edited_time": "2024-01-15T10:00:00Z"
        }"#;
        let block: NotionBlock = serde_json::from_str(json).unwrap();
        assert_eq!(block.block_type, "paragraph");
        assert!(!block.has_children);
        assert!(block.content.contains_key("paragraph"));
    }

    #[test]
    fn test_notion_client_creation() {
        let client = NotionClient::new("test-token".to_string());
        assert_eq!(client.access_token, "test-token");
        assert_eq!(client.api_base_url, NOTION_API_BASE);
    }

    #[test]
    fn test_notion_client_custom_base_url() {
        let client = NotionClient::with_base_url(
            "test-token".to_string(),
            "https://custom.notion.api".to_string(),
        );
        assert_eq!(client.api_base_url, "https://custom.notion.api");
    }
}
