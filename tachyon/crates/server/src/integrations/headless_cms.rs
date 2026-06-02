//! Headless CMS integration — pull content from Decap/Sanity via API.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, warn};

/// Supported headless CMS providers.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum CmsProvider {
    Decap,
    Sanity,
    Generic,
}

impl std::fmt::Display for CmsProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Decap => write!(f, "decap"),
            Self::Sanity => write!(f, "sanity"),
            Self::Generic => write!(f, "generic"),
        }
    }
}

/// Configuration for a headless CMS integration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CmsConfig {
    pub provider: CmsProvider,
    /// Base URL for the CMS API.
    pub api_url: String,
    /// API token or access key.
    pub api_token: Option<String>,
    /// Sync interval in seconds.
    pub sync_interval_secs: u64,
    /// Collection/model mapping: CMS collection -> Tachyon document type.
    pub collection_mapping: HashMap<String, String>,
    /// Whether to automatically sync on startup.
    pub auto_sync: bool,
}

impl Default for CmsConfig {
    fn default() -> Self {
        Self {
            provider: CmsProvider::Generic,
            api_url: String::new(),
            api_token: None,
            sync_interval_secs: 300,
            collection_mapping: HashMap::new(),
            auto_sync: false,
        }
    }
}

/// A CMS document fetched from a headless CMS.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CmsDocument {
    pub id: String,
    pub title: String,
    pub slug: String,
    pub content: String,
    pub collection: String,
    pub published_at: Option<String>,
    pub updated_at: Option<String>,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Result of a CMS sync operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CmsSyncResult {
    pub provider: String,
    pub documents_fetched: usize,
    pub documents_imported: usize,
    pub documents_skipped: usize,
    pub errors: Vec<String>,
    pub synced_at: String,
}

/// Errors that can occur during CMS operations.
#[derive(Debug)]
pub enum CmsError {
    Fetch(String),
    Database(String),
}

impl std::fmt::Display for CmsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Fetch(e) => write!(f, "CMS fetch error: {}", e),
            Self::Database(e) => write!(f, "CMS database error: {}", e),
        }
    }
}

impl std::error::Error for CmsError {}

/// Headless CMS client that pulls content from configured providers.
pub struct HeadlessCmsClient {
    config: CmsConfig,
    http_client: reqwest::Client,
}

impl HeadlessCmsClient {
    pub fn new(config: CmsConfig) -> Self {
        Self {
            config,
            http_client: reqwest::Client::new(),
        }
    }

    pub fn provider(&self) -> &CmsProvider {
        &self.config.provider
    }

    /// Fetch documents from the CMS API.
    pub async fn fetch_documents(&self) -> Result<Vec<CmsDocument>, String> {
        let url = format!(
            "{}/api/documents",
            self.config.api_url.trim_end_matches('/')
        );
        let mut req = self.http_client.get(&url);
        if let Some(token) = &self.config.api_token {
            req = req.bearer_auth(token);
        }
        let resp = req
            .send()
            .await
            .map_err(|e| format!("CMS fetch error: {}", e))?;
        if !resp.status().is_success() {
            return Err(format!("CMS API returned status {}", resp.status()));
        }
        let docs: Vec<CmsDocument> = resp
            .json()
            .await
            .map_err(|e| format!("CMS parse error: {}", e))?;
        Ok(docs)
    }

    /// Map CMS collection to Tachyon document type.
    pub fn map_collection(&self, collection: &str) -> Option<&str> {
        self.config
            .collection_mapping
            .get(collection)
            .map(|s| s.as_str())
    }

    /// Build a sync result placeholder (actual sync requires DB access).
    pub fn build_sync_result(
        &self,
        fetched: usize,
        imported: usize,
        skipped: usize,
        errors: Vec<String>,
    ) -> CmsSyncResult {
        CmsSyncResult {
            provider: format!("{:?}", self.config.provider).to_lowercase(),
            documents_fetched: fetched,
            documents_imported: imported,
            documents_skipped: skipped,
            errors,
            synced_at: chrono::Utc::now().to_rfc3339(),
        }
    }

    /// Fetch documents from the CMS and upsert them into the local database.
    ///
    /// For each CMS document:
    /// - Checks if a document with the same `external_id` (CMS id) already exists
    /// - If not, checks by slug
    /// - New documents are inserted; existing ones are updated
    pub async fn sync_to_database(
        &self,
        pool: &tachyon_database::DatabasePool,
    ) -> Result<CmsSyncResult, CmsError> {
        let docs = self
            .fetch_documents()
            .await
            .map_err(CmsError::Fetch)?;

        let fetched = docs.len();
        let mut imported = 0usize;
        let mut skipped = 0usize;
        let mut errors = Vec::new();

        for doc in &docs {
            let doc_type = self
                .map_collection(&doc.collection)
                .unwrap_or("document")
                .to_string();

            let external_id = format!("cms:{}:{}", self.config.provider, doc.id);

            let now = chrono::Utc::now();
            let published_at = doc
                .published_at
                .as_deref()
                .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
                .map(|dt| dt.with_timezone(&chrono::Utc));

            let existing_id: Option<String> = sqlx::query_scalar(
                "SELECT id FROM documents WHERE external_id = $1 LIMIT 1",
            )
            .bind(&external_id)
            .fetch_optional(pool.inner())
            .await
            .map_err(|e| {
                CmsError::Database(format!("Failed to check existing document: {}", e))
            })?;

            let slug = if doc.slug.is_empty() {
                doc.title.to_lowercase().replace(' ', "-")
            } else {
                doc.slug.clone()
            };

            if let Some(existing_id) = existing_id {
                debug!(doc_id = %external_id, "Updating existing synced document");
                let result = sqlx::query(
                    "UPDATE documents SET title = $1, slug = $2, content = $3, updated_at = $4, status = 'published', tags = $5 WHERE id = $6",
                )
                .bind(&doc.title)
                .bind(&slug)
                .bind(&doc.content)
                .bind(now)
                .bind(serde_json::json!([]))
                .bind(&existing_id)
                .execute(pool.inner())
                .await;

                match result {
                    Ok(_) => imported += 1,
                    Err(e) => {
                        warn!(doc_id = %external_id, error = %e, "Failed to update synced document");
                        errors.push(format!("Update failed for '{}': {}", doc.title, e));
                    }
                }
            } else {
                let existing_by_slug: Option<String> = sqlx::query_scalar(
                    "SELECT id FROM documents WHERE slug = $1 LIMIT 1",
                )
                .bind(&slug)
                .fetch_optional(pool.inner())
                .await
                .map_err(|e| {
                    CmsError::Database(format!("Failed to check slug: {}", e))
                })?;

                if existing_by_slug.is_some() {
                    debug!(slug = %slug, "Document slug already exists, skipping");
                    skipped += 1;
                    continue;
                }

                debug!(doc_id = %external_id, "Inserting new synced document");
                let new_id = uuid::Uuid::new_v4().to_string();
                let result = sqlx::query(
                    r#"INSERT INTO documents (
                        id, title, slug, author_id, description, tags, frontmatter,
                        project_id, visibility, status, content_type,
                        word_count, character_count, read_count, edit_count,
                        content, html,
                        created_at, updated_at, published_at,
                        content_hash, conflict_detected, external_id
                    ) VALUES ($1::uuid, $2, $3, '00000000-0000-0000-0000-000000000000'::uuid, NULL, $4::jsonb, NULL, NULL, 'private', $5, 'markdown', $6, $7, 0, 1, $8, NULL, $9, $10, $11, NULL, false, $12)"#,
                )
                .bind(&new_id)
                .bind(&doc.title)
                .bind(&slug)
                .bind(serde_json::json!([]))
                .bind(&doc_type)
                .bind(doc.content.split_whitespace().count() as i32)
                .bind(doc.content.len() as i32)
                .bind(&doc.content)
                .bind(now)
                .bind(now)
                .bind(published_at)
                .bind(&external_id)
                .execute(pool.inner())
                .await;

                match result {
                    Ok(_) => imported += 1,
                    Err(e) => {
                        warn!(doc_id = %external_id, error = %e, "Failed to insert synced document");
                        errors.push(format!("Insert failed for '{}': {}", doc.title, e));
                    }
                }
            }
        }

        Ok(CmsSyncResult {
            provider: format!("{:?}", self.config.provider).to_lowercase(),
            documents_fetched: fetched,
            documents_imported: imported,
            documents_skipped: skipped,
            errors,
            synced_at: chrono::Utc::now().to_rfc3339(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cms_config_default() {
        let config = CmsConfig::default();
        assert_eq!(config.provider, CmsProvider::Generic);
        assert_eq!(config.sync_interval_secs, 300);
        assert!(!config.auto_sync);
    }

    #[test]
    fn test_cms_provider_serialization() {
        assert_eq!(
            serde_json::to_string(&CmsProvider::Decap).unwrap(),
            r#""decap""#
        );
        assert_eq!(
            serde_json::to_string(&CmsProvider::Sanity).unwrap(),
            r#""sanity""#
        );
    }

    #[test]
    fn test_cms_config_from_json() {
        let json = r#"{
            "provider": "decap",
            "api_url": "https://cms.example.com",
            "api_token": "secret",
            "sync_interval_secs": 600,
            "auto_sync": true,
            "collection_mapping": {"posts": "document", "pages": "document"}
        }"#;
        let config: CmsConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.provider, CmsProvider::Decap);
        assert_eq!(config.sync_interval_secs, 600);
        assert!(config.auto_sync);
        assert_eq!(config.collection_mapping.get("posts").unwrap(), "document");
    }

    #[test]
    fn test_cms_document_serialization() {
        let doc = CmsDocument {
            id: "1".to_string(),
            title: "Test".to_string(),
            slug: "test".to_string(),
            content: "Hello".to_string(),
            collection: "posts".to_string(),
            published_at: None,
            updated_at: None,
            metadata: HashMap::new(),
        };
        let json = serde_json::to_string(&doc).unwrap();
        assert!(json.contains("Test"));
    }

    #[test]
    fn test_collection_mapping() {
        let mut config = CmsConfig::default();
        config
            .collection_mapping
            .insert("posts".to_string(), "document".to_string());
        let client = HeadlessCmsClient::new(config);
        assert_eq!(client.map_collection("posts"), Some("document"));
        assert_eq!(client.map_collection("unknown"), None);
    }

    #[test]
    fn test_sync_result() {
        let config = CmsConfig::default();
        let client = HeadlessCmsClient::new(config);
        let result = client.build_sync_result(10, 8, 2, vec![]);
        assert_eq!(result.documents_fetched, 10);
        assert_eq!(result.documents_imported, 8);
        assert_eq!(result.provider, "generic");
    }
}
