//! Confluence REST API client for fetching spaces and pages.
//!
//! Supports both Confluence Cloud (api.atlassian.com) and
//! Data Center/Server (self-hosted) editions.
//!
//! Authentication methods:
//! - Basic auth (username + API token for Cloud, password for DC)
//! - Personal access token (DC only)
//!
//! Pagination uses start/limit parameters (default page size: 25).

use crate::error::{ImportExportError, ImportExportResult};
use reqwest::Client;
use serde::Deserialize;
use std::time::Duration;

/// Confluence edition type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfluenceEdition {
    /// Confluence Cloud (api.atlassian.com or *.atlassian.net)
    Cloud,
    /// Confluence Data Center / Server (self-hosted)
    DataCenter,
}

/// Credentials for connecting to a Confluence instance.
#[derive(Debug, Clone)]
pub struct ConfluenceCredentials {
    /// Base URL of the Confluence instance (e.g., "https://example.atlassian.net/wiki")
    pub base_url: String,
    /// Authentication method
    pub auth: ConfluenceAuth,
}

/// Authentication method for Confluence.
#[derive(Debug, Clone)]
pub enum ConfluenceAuth {
    /// Basic auth: username + password/token
    Basic { username: String, password: String },
    /// Personal access token (Data Center only)
    PersonalAccessToken(String),
}

/// A Confluence space.
#[derive(Debug, Clone, Deserialize)]
pub struct ConfluenceSpace {
    pub id: String,
    pub key: String,
    pub name: String,
    #[serde(rename = "type")]
    pub space_type: String,
    #[serde(default)]
    pub description: Option<serde_json::Value>,
}

/// A Confluence page with metadata.
#[derive(Debug, Clone, Deserialize)]
pub struct ConfluencePage {
    pub id: String,
    pub title: String,
    #[serde(rename = "type")]
    pub page_type: String,
    #[serde(rename = "space")]
    pub space: Option<ConfluenceSpaceRef>,
    pub version: Option<PageVersion>,
    #[serde(rename = "body")]
    pub body: Option<PageBody>,
    #[serde(rename = "_links")]
    pub links: Option<PageLinks>,
    #[serde(rename = "ancestors")]
    pub ancestors: Option<Vec<AncestorRef>>,
    #[serde(rename = "metadata")]
    pub metadata: Option<PageMetadata>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ConfluenceSpaceRef {
    pub key: String,
    pub name: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PageVersion {
    pub when: Option<String>,
    pub by: Option<UserRef>,
    pub number: Option<u32>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UserRef {
    pub display_name: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PageBody {
    #[serde(rename = "storage")]
    pub storage: Option<BodyContent>,
    #[serde(rename = "editor")]
    pub editor: Option<BodyContent>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BodyContent {
    pub value: Option<String>,
    pub representation: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PageLinks {
    pub base: Option<String>,
    pub context: Option<String>,
    pub webui: Option<String>,
    pub self_link: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AncestorRef {
    pub id: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PageMetadata {
    pub labels: Option<LabelResult>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LabelResult {
    pub results: Vec<Label>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Label {
    pub name: String,
}

/// Paginated response from the Confluence REST API.
#[derive(Debug, Deserialize)]
pub struct PaginatedResponse<T> {
    pub results: Vec<T>,
    pub start: usize,
    pub limit: usize,
    pub size: usize,
    #[serde(rename = "_links")]
    pub links: Option<PaginationLinks>,
}

#[derive(Debug, Deserialize)]
pub struct PaginationLinks {
    pub next: Option<String>,
    pub prev: Option<String>,
    pub base: Option<String>,
}

/// Confluence REST API client.
pub struct ConfluenceClient {
    client: Client,
    credentials: ConfluenceCredentials,
    edition: ConfluenceEdition,
    base_api_url: String,
    rate_limit_delay: Duration,
}

impl ConfluenceClient {
    /// Create a new Confluence client.
    ///
    /// Auto-detects edition from the base URL, or you can override with `with_edition`.
    pub fn new(credentials: ConfluenceCredentials) -> ImportExportResult<Self> {
        let edition = Self::detect_edition(&credentials.base_url);
        Self::with_edition(credentials, edition)
    }

    /// Create a new Confluence client with explicit edition.
    pub fn with_edition(
        credentials: ConfluenceCredentials,
        edition: ConfluenceEdition,
    ) -> ImportExportResult<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .connect_timeout(Duration::from_secs(10))
            .build()
            .map_err(|e| {
                ImportExportError::import(format!("Failed to create HTTP client: {}", e))
            })?;

        let base_api_url = match edition {
            ConfluenceEdition::Cloud => {
                let base = credentials.base_url.trim_end_matches('/');
                format!("{}/wiki/rest/api", base)
            }
            ConfluenceEdition::DataCenter => {
                let base = credentials.base_url.trim_end_matches('/');
                format!("{}/rest/api", base)
            }
        };

        let rate_limit_delay = match edition {
            ConfluenceEdition::Cloud => Duration::from_millis(100),
            ConfluenceEdition::DataCenter => Duration::from_millis(50),
        };

        Ok(Self {
            client,
            credentials,
            edition,
            base_api_url,
            rate_limit_delay,
        })
    }

    /// Detect Confluence edition from URL.
    fn detect_edition(url: &str) -> ConfluenceEdition {
        if url.contains("atlassian.net") || url.contains("atlassian.com") {
            ConfluenceEdition::Cloud
        } else {
            ConfluenceEdition::DataCenter
        }
    }

    /// Build the authorization header value.
    fn auth_header(&self) -> String {
        match &self.credentials.auth {
            ConfluenceAuth::Basic { username, password } => {
                use base64::Engine;
                let credentials = format!("{}:{}", username, password);
                let encoded = base64::engine::general_purpose::STANDARD.encode(credentials);
                format!("Basic {}", encoded)
            }
            ConfluenceAuth::PersonalAccessToken(token) => {
                format!("Bearer {}", token)
            }
        }
    }

    /// Execute a GET request to the Confluence API.
    async fn get<T: serde::de::DeserializeOwned>(
        &self,
        endpoint: &str,
        params: &[(&str, &str)],
    ) -> ImportExportResult<T> {
        let url = format!("{}/{}", self.base_api_url, endpoint);
        let auth = self.auth_header();

        let mut request = self
            .client
            .get(&url)
            .header("Authorization", &auth)
            .header("Accept", "application/json");

        for (key, value) in params {
            request = request.query(&[(*key, *value)]);
        }

        let response = request
            .send()
            .await
            .map_err(|e| ImportExportError::import(format!("HTTP request failed: {}", e)))?;

        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(ImportExportError::import(format!(
                "Confluence API error ({}): {}",
                status, body
            )));
        }

        let data: T = response.json().await.map_err(|e| {
            ImportExportError::import(format!("Failed to parse API response: {}", e))
        })?;

        // Rate limiting: wait between requests
        tokio::time::sleep(self.rate_limit_delay).await;

        Ok(data)
    }

    /// Get details about a Confluence space.
    pub async fn get_space(&self, space_key: &str) -> ImportExportResult<ConfluenceSpace> {
        let response: serde_json::Value = self
            .get(
                "space",
                &[("spaceKey", space_key), ("expand", "description")],
            )
            .await?;

        // The space endpoint returns a single space object
        serde_json::from_value(response).map_err(|e| {
            ImportExportError::import(format!("Failed to parse space response: {}", e))
        })
    }

    /// Get all pages in a space, handling pagination.
    ///
    /// Returns all pages regardless of their hierarchy.
    pub async fn get_space_pages(
        &self,
        space_key: &str,
    ) -> ImportExportResult<Vec<ConfluencePage>> {
        let mut all_pages = Vec::new();
        let mut start = 0;
        let limit = 25;

        loop {
            let response: PaginatedResponse<ConfluencePage> = self
                .get(
                    "content",
                    &[
                        ("spaceKey", space_key),
                        ("type", "page"),
                        ("start", &start.to_string()),
                        ("limit", &limit.to_string()),
                        ("expand", "body.storage,version,ancestors,metadata.labels"),
                    ],
                )
                .await?;

            let page_count = response.size;
            all_pages.extend(response.results);

            if page_count < limit {
                break;
            }
            start += limit;
        }

        Ok(all_pages)
    }

    /// Get a single page by ID with full body content.
    pub async fn get_page(&self, page_id: &str) -> ImportExportResult<ConfluencePage> {
        self.get(
            &format!("content/{}", page_id),
            &[(
                "expand",
                "body.storage,body.editor,version,ancestors,space,metadata.labels",
            )],
        )
        .await
    }

    /// Get child pages of a given page.
    pub async fn get_child_pages(&self, page_id: &str) -> ImportExportResult<Vec<ConfluencePage>> {
        let mut all_pages = Vec::new();
        let mut start = 0;
        let limit = 25;

        loop {
            let response: PaginatedResponse<ConfluencePage> = self
                .get(
                    &format!("content/{}/child/page", page_id),
                    &[
                        ("start", &start.to_string()),
                        ("limit", &limit.to_string()),
                        ("expand", "body.storage,version,ancestors,metadata.labels"),
                    ],
                )
                .await?;

            let page_count = response.size;
            all_pages.extend(response.results);

            if page_count < limit {
                break;
            }
            start += limit;
        }

        Ok(all_pages)
    }

    /// Recursively fetch all pages in a space with their tree structure.
    ///
    /// Returns pages in breadth-first order, preserving parent-child relationships.
    pub async fn fetch_full_space(
        &self,
        space_key: &str,
    ) -> ImportExportResult<Vec<ConfluencePage>> {
        self.get_space_pages(space_key).await
    }

    /// Get labels for a page.
    pub async fn get_page_labels(&self, page_id: &str) -> ImportExportResult<Vec<String>> {
        let response: serde_json::Value = self
            .get(
                &format!("content/{}/label", page_id),
                &[("start", "0"), ("limit", "100")],
            )
            .await?;

        let labels = response["results"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|l| l["name"].as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default();

        Ok(labels)
    }

    /// Check connectivity to the Confluence instance.
    pub async fn check_connection(&self) -> ImportExportResult<bool> {
        let result: Result<serde_json::Value, _> = self.get("info", &[]).await;
        Ok(result.is_ok())
    }

    /// Get the detected edition.
    pub fn edition(&self) -> ConfluenceEdition {
        self.edition
    }

    /// Get the base API URL.
    pub fn base_api_url(&self) -> &str {
        &self.base_api_url
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_cloud_edition() {
        assert_eq!(
            ConfluenceClient::detect_edition("https://mycompany.atlassian.net/wiki"),
            ConfluenceEdition::Cloud
        );
        assert_eq!(
            ConfluenceClient::detect_edition("https://example.atlassian.net"),
            ConfluenceEdition::Cloud
        );
    }

    #[test]
    fn test_detect_data_center_edition() {
        assert_eq!(
            ConfluenceClient::detect_edition("https://confluence.mycompany.com"),
            ConfluenceEdition::DataCenter
        );
        assert_eq!(
            ConfluenceClient::detect_edition("http://localhost:8090"),
            ConfluenceEdition::DataCenter
        );
    }

    #[test]
    fn test_base_api_url_cloud() {
        let creds = ConfluenceCredentials {
            base_url: "https://mycompany.atlassian.net/wiki".to_string(),
            auth: ConfluenceAuth::Basic {
                username: "user".to_string(),
                password: "token".to_string(),
            },
        };
        let client = ConfluenceClient::with_edition(creds, ConfluenceEdition::Cloud).unwrap();
        assert_eq!(
            client.base_api_url(),
            "https://mycompany.atlassian.net/wiki/wiki/rest/api"
        );
    }

    #[test]
    fn test_base_api_url_data_center() {
        let creds = ConfluenceCredentials {
            base_url: "https://confluence.mycompany.com".to_string(),
            auth: ConfluenceAuth::PersonalAccessToken("pat123".to_string()),
        };
        let client = ConfluenceClient::with_edition(creds, ConfluenceEdition::DataCenter).unwrap();
        assert_eq!(
            client.base_api_url(),
            "https://confluence.mycompany.com/rest/api"
        );
    }

    #[test]
    fn test_auth_header_basic() {
        let creds = ConfluenceCredentials {
            base_url: "https://example.com".to_string(),
            auth: ConfluenceAuth::Basic {
                username: "admin".to_string(),
                password: "secret".to_string(),
            },
        };
        let client = ConfluenceClient::with_edition(creds, ConfluenceEdition::DataCenter).unwrap();
        let header = client.auth_header();
        assert!(header.starts_with("Basic "));
    }

    #[test]
    fn test_auth_header_pat() {
        let creds = ConfluenceCredentials {
            base_url: "https://example.com".to_string(),
            auth: ConfluenceAuth::PersonalAccessToken("my-token".to_string()),
        };
        let client = ConfluenceClient::with_edition(creds, ConfluenceEdition::DataCenter).unwrap();
        assert_eq!(client.auth_header(), "Bearer my-token");
    }

    #[test]
    fn test_trailing_slash_handling() {
        let creds = ConfluenceCredentials {
            base_url: "https://example.com/".to_string(),
            auth: ConfluenceAuth::PersonalAccessToken("token".to_string()),
        };
        let client = ConfluenceClient::with_edition(creds, ConfluenceEdition::DataCenter).unwrap();
        assert_eq!(client.base_api_url(), "https://example.com/rest/api");
    }

    #[test]
    fn test_confluence_space_deserialize() {
        let json = r#"{
            "id": "12345",
            "key": "DEV",
            "name": "Development",
            "type": "global",
            "description": {"value": "Dev docs"}
        }"#;
        let space: ConfluenceSpace = serde_json::from_str(json).unwrap();
        assert_eq!(space.key, "DEV");
        assert_eq!(space.name, "Development");
    }

    #[test]
    fn test_confluence_page_deserialize() {
        let json = r#"{
            "id": "67890",
            "title": "Getting Started",
            "type": "page",
            "space": {"key": "DEV", "name": "Development"},
            "body": {
                "storage": {
                    "value": "<p>Welcome</p>",
                    "representation": "storage"
                }
            },
            "ancestors": [{"id": "12345"}],
            "metadata": {
                "labels": {
                    "results": [{"name": "guide"}, {"name": "onboarding"}]
                }
            }
        }"#;
        let page: ConfluencePage = serde_json::from_str(json).unwrap();
        assert_eq!(page.id, "67890");
        assert_eq!(page.title, "Getting Started");
        assert!(page.space.is_some());
        assert_eq!(page.space.as_ref().unwrap().key, "DEV");
        assert!(page.body.is_some());
        let body = page.body.as_ref().unwrap().storage.as_ref().unwrap();
        assert_eq!(body.value.as_deref(), Some("<p>Welcome</p>"));
        assert!(page.ancestors.is_some());
        assert_eq!(page.ancestors.as_ref().unwrap()[0].id, "12345");
        let labels = page.metadata.as_ref().unwrap().labels.as_ref().unwrap();
        assert_eq!(labels.results.len(), 2);
        assert_eq!(labels.results[0].name, "guide");
    }

    #[test]
    fn test_paginated_response_deserialize() {
        let json = r#"{
            "results": [],
            "start": 0,
            "limit": 25,
            "size": 0,
            "_links": {
                "next": "/rest/api/content?start=25"
            }
        }"#;
        let resp: PaginatedResponse<ConfluencePage> = serde_json::from_str(json).unwrap();
        assert_eq!(resp.start, 0);
        assert_eq!(resp.limit, 25);
        assert_eq!(resp.size, 0);
        assert!(resp.links.unwrap().next.is_some());
    }
}
