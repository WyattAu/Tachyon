use async_trait::async_trait;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StorageUrl {
    pub url: String,
    pub size: u64,
    pub content_type: String,
    pub etag: Option<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum StorageError {
    #[error("not found: {0}")]
    NotFound(String),
    #[error("permission denied: {0}")]
    PermissionDenied(String),
    #[error("backend error: {0}")]
    BackendError(String),
    #[error("invalid key: {0}")]
    InvalidKey(String),
    #[error("quota exceeded")]
    QuotaExceeded,
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

#[async_trait]
pub trait StorageBackend: Send + Sync {
    async fn put(
        &self,
        key: &str,
        data: &[u8],
        content_type: &str,
    ) -> Result<StorageUrl, StorageError>;

    async fn get(&self, key: &str) -> Result<Vec<u8>, StorageError>;

    async fn delete(&self, key: &str) -> Result<(), StorageError>;

    async fn exists(&self, key: &str) -> Result<bool, StorageError>;

    async fn list(&self, prefix: &str) -> Result<Vec<String>, StorageError>;

    async fn presigned_url(&self, key: &str, expires_in_secs: u64) -> Result<String, StorageError>;
}
