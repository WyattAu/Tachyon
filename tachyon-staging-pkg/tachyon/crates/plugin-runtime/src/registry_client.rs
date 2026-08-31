//! Remote Plugin Registry Client.
//!
//! HTTP client for interacting with a remote plugin registry API.
//! Enable with the `registry-client` feature flag.

use crate::marketplace::{MarketplaceError, MarketplaceResult, PluginManifest, PluginVersion};
use serde::{Deserialize, Serialize};

#[cfg(feature = "registry-client")]
use reqwest::Client;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryConfig {
    pub base_url: String,
    pub api_key: Option<String>,
    pub timeout_seconds: u64,
    pub max_retries: u32,
}

impl Default for RegistryConfig {
    fn default() -> Self {
        Self {
            base_url: "https://registry.tachyon.dev".to_string(),
            api_key: None,
            timeout_seconds: 30,
            max_retries: 3,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginListResponse {
    pub plugins: Vec<PluginManifest>,
    pub total: u64,
    pub page: i32,
    pub per_page: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginDownloadResponse {
    pub download_url: String,
    pub checksum: String,
    pub size_bytes: u64,
    pub content_type: String,
}

#[derive(Debug, Clone, Default)]
pub struct SearchQuery {
    pub query: Option<String>,
    pub tags: Vec<String>,
    pub extension_points: Vec<String>,
    pub page: Option<i32>,
    pub per_page: Option<i32>,
    pub sort_by: Option<String>,
    pub sort_order: Option<String>,
}

#[cfg(feature = "registry-client")]
#[derive(Debug, Clone)]
pub struct RegistryClient {
    config: RegistryConfig,
    client: Client,
}

#[cfg(feature = "registry-client")]
impl RegistryClient {
    pub fn new(config: RegistryConfig) -> MarketplaceResult<Self> {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(config.timeout_seconds))
            .user_agent(format!(
                "tachyon-plugin-runtime/{}",
                env!("CARGO_PKG_VERSION")
            ))
            .build()
            .map_err(|e| MarketplaceError::Io(std::io::Error::other(e.to_string())))?;

        Ok(Self { config, client })
    }

    pub fn base_url(&self) -> &str {
        &self.config.base_url
    }

    fn build_request(
        &self,
        method: reqwest::Method,
        path: &str,
    ) -> MarketplaceResult<reqwest::RequestBuilder> {
        let url = format!("{}{}", self.config.base_url.trim_end_matches('/'), path);
        let mut req = self.client.request(method, &url);
        if let Some(key) = &self.config.api_key {
            req = req.header("Authorization", format!("Bearer {}", key));
        }
        req = req.header("Accept", "application/json");
        Ok(req)
    }

    pub async fn search(&self, query: SearchQuery) -> MarketplaceResult<PluginListResponse> {
        let mut params: Vec<(&str, String)> = vec![];
        if let Some(q) = &query.query {
            params.push(("q", q.clone()));
        }
        for tag in &query.tags {
            params.push(("tag", tag.clone()));
        }
        for ep in &query.extension_points {
            params.push(("extension_point", ep.clone()));
        }
        if let Some(page) = query.page {
            params.push(("page", page.to_string()));
        }
        if let Some(per_page) = query.per_page {
            params.push(("per_page", per_page.to_string()));
        }
        if let Some(sort) = &query.sort_by {
            params.push(("sort_by", sort.clone()));
        }
        if let Some(order) = &query.sort_order {
            params.push(("sort_order", order.clone()));
        }
        let params_refs: Vec<(&str, &str)> = params.iter().map(|(k, v)| (*k, v.as_str())).collect();

        let response = self
            .build_request(reqwest::Method::GET, "/api/v1/plugins")?
            .query(&params_refs)
            .send()
            .await
            .map_err(|e| {
                MarketplaceError::Io(std::io::Error::other(format!(
                    "Registry request failed: {e}"
                )))
            })?;

        if !response.status().is_success() {
            return Err(MarketplaceError::NotFound(format!(
                "Registry returned status {}",
                response.status()
            )));
        }

        response.json::<PluginListResponse>().await.map_err(|e| {
            MarketplaceError::Io(std::io::Error::other(format!(
                "Failed to parse response: {e}"
            )))
        })
    }

    pub async fn get_plugin(&self, plugin_id: &str) -> MarketplaceResult<PluginManifest> {
        let path = format!("/api/v1/plugins/{}", plugin_id);
        let response = self
            .build_request(reqwest::Method::GET, &path)?
            .send()
            .await
            .map_err(|e| {
                MarketplaceError::Io(std::io::Error::other(format!(
                    "Registry request failed: {e}"
                )))
            })?;

        if response.status() == reqwest::StatusCode::NOT_FOUND {
            return Err(MarketplaceError::NotFound(format!(
                "Plugin not found: {plugin_id}"
            )));
        }
        if !response.status().is_success() {
            return Err(MarketplaceError::NotFound(format!(
                "Registry returned status {}",
                response.status()
            )));
        }

        response.json::<PluginManifest>().await.map_err(|e| {
            MarketplaceError::Io(std::io::Error::other(format!(
                "Failed to parse response: {e}"
            )))
        })
    }

    pub async fn get_download_url(
        &self,
        plugin_id: &str,
        version: &PluginVersion,
    ) -> MarketplaceResult<PluginDownloadResponse> {
        let path = format!(
            "/api/v1/plugins/{}/versions/{}/download",
            plugin_id, version
        );
        let response = self
            .build_request(reqwest::Method::GET, &path)?
            .send()
            .await
            .map_err(|e| {
                MarketplaceError::Io(std::io::Error::other(format!(
                    "Registry request failed: {e}"
                )))
            })?;

        if !response.status().is_success() {
            return Err(MarketplaceError::NotFound(format!(
                "Plugin version not found: {plugin_id}@{version}"
            )));
        }

        response
            .json::<PluginDownloadResponse>()
            .await
            .map_err(|e| {
                MarketplaceError::Io(std::io::Error::other(format!(
                    "Failed to parse response: {e}"
                )))
            })
    }

    pub async fn download_plugin(
        &self,
        plugin_id: &str,
        version: &PluginVersion,
    ) -> MarketplaceResult<Vec<u8>> {
        let download = self.get_download_url(plugin_id, version).await?;
        let response = self
            .client
            .get(&download.download_url)
            .send()
            .await
            .map_err(|e| {
                MarketplaceError::Io(std::io::Error::other(format!("Download failed: {e}")))
            })?;

        if !response.status().is_success() {
            return Err(MarketplaceError::NotFound(format!(
                "Download failed with status {}",
                response.status()
            )));
        }

        let bytes = response.bytes().await.map_err(|e| {
            MarketplaceError::Io(std::io::Error::other(format!(
                "Failed to read response body: {e}"
            )))
        })?;

        Ok(bytes.to_vec())
    }

    pub async fn check_updates<'a>(
        &'a self,
        installed: &'a [PluginManifest],
    ) -> MarketplaceResult<Vec<(&'a PluginManifest, PluginVersion)>> {
        let mut updates = Vec::new();
        for plugin in installed {
            match self.get_plugin(&plugin.id.0).await {
                Ok(latest) => {
                    if latest.version.major > plugin.version.major
                        || (latest.version.major == plugin.version.major
                            && latest.version.minor > plugin.version.minor)
                        || (latest.version.major == plugin.version.major
                            && latest.version.minor == plugin.version.minor
                            && latest.version.patch > plugin.version.patch)
                    {
                        updates.push((plugin, latest.version));
                    }
                }
                Err(_) => continue,
            }
        }
        Ok(updates)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_config_default() {
        let config = RegistryConfig::default();
        assert_eq!(config.base_url, "https://registry.tachyon.dev");
        assert!(config.api_key.is_none());
        assert_eq!(config.timeout_seconds, 30);
        assert_eq!(config.max_retries, 3);
    }

    #[test]
    fn test_search_query_default() {
        let query = SearchQuery::default();
        assert!(query.query.is_none());
        assert!(query.tags.is_empty());
        assert!(query.page.is_none());
    }

    #[test]
    fn test_plugin_list_response_deserialize() {
        let json = r#"{
            "plugins": [],
            "total": 0,
            "page": 1,
            "per_page": 20
        }"#;
        let response: PluginListResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.total, 0);
        assert!(response.plugins.is_empty());
    }

    #[test]
    fn test_plugin_download_response_deserialize() {
        let json = r#"{
            "download_url": "https://example.com/plugin.wasm",
            "checksum": "abc123",
            "size_bytes": 1024,
            "content_type": "application/wasm"
        }"#;
        let response: PluginDownloadResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.checksum, "abc123");
        assert_eq!(response.size_bytes, 1024);
    }

    #[test]
    fn test_registry_config_serialize() {
        let config = RegistryConfig {
            base_url: "https://custom.registry.com".to_string(),
            api_key: Some("secret".to_string()),
            timeout_seconds: 60,
            max_retries: 5,
        };
        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("custom.registry.com"));
        assert!(json.contains("secret"));
    }
}
