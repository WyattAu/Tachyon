pub mod local;
pub mod s3;

use async_trait::async_trait;

/// Result of a storage operation
#[derive(Debug, Clone)]
pub struct StorageResult {
    pub url: String,
    pub size: u64,
    pub content_type: String,
    pub etag: Option<String>,
}

/// Abstract storage backend
#[async_trait]
pub trait StorageBackend: Send + Sync {
    /// Upload a file
    async fn put(
        &self,
        key: &str,
        data: &[u8],
        content_type: &str,
    ) -> Result<StorageResult, StorageError>;

    /// Download a file
    async fn get(&self, key: &str) -> Result<Vec<u8>, StorageError>;

    /// Delete a file
    async fn delete(&self, key: &str) -> Result<(), StorageError>;

    /// Check if a file exists
    async fn exists(&self, key: &str) -> Result<bool, StorageError>;

    /// Get a presigned URL for direct access (if supported)
    async fn presigned_url(&self, key: &str, expires_in_secs: u64) -> Result<String, StorageError>;
}

#[derive(Debug, thiserror::Error)]
pub enum StorageError {
    #[error("File not found: {0}")]
    NotFound(String),
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
    #[error("Storage backend error: {0}")]
    BackendError(String),
    #[error("Invalid key: {0}")]
    InvalidKey(String),
    #[error("Quota exceeded")]
    QuotaExceeded,
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// Storage configuration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct StorageConfig {
    /// Storage backend type: "local" or "s3"
    #[serde(default = "default_backend")]
    pub backend: String,

    /// Base URL for serving files
    #[serde(default = "default_base_url")]
    pub base_url: String,

    /// Local storage path (for "local" backend)
    #[serde(default = "default_local_path")]
    pub local_path: String,

    /// S3 bucket name
    #[serde(default)]
    pub s3_bucket: Option<String>,

    /// S3 region
    #[serde(default)]
    pub s3_region: Option<String>,

    /// S3 endpoint (for MinIO etc.)
    #[serde(default)]
    pub s3_endpoint: Option<String>,

    /// S3 access key
    #[serde(default)]
    pub s3_access_key: Option<String>,

    /// S3 secret key
    #[serde(default)]
    pub s3_secret_key: Option<String>,
}

fn default_backend() -> String {
    "local".to_string()
}
fn default_base_url() -> String {
    "/files".to_string()
}
fn default_local_path() -> String {
    "./uploads".to_string()
}

/// Create storage backend from config
pub fn create_storage(config: &StorageConfig) -> Result<Box<dyn StorageBackend>, StorageError> {
    match config.backend.as_str() {
        "local" => Ok(Box::new(local::LocalStorage::new(
            &config.local_path,
            &config.base_url,
        )?)),
        "s3" => Ok(Box::new(s3::S3Storage::new(config)?)),
        other => Err(StorageError::BackendError(format!(
            "Unknown storage backend: {}",
            other
        ))),
    }
}
