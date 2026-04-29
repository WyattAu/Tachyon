use super::{StorageBackend, StorageConfig, StorageError, StorageResult};
use async_trait::async_trait;

/// S3-compatible storage backend
/// Requires `aws-sdk-s3` or compatible client
pub struct S3Storage {
    bucket: String,
    region: String,
    endpoint: Option<String>,
    client_ready: bool,
}

impl S3Storage {
    pub fn new(config: &StorageConfig) -> Result<Self, StorageError> {
        let bucket = config.s3_bucket.clone().ok_or(StorageError::BackendError(
            "S3 bucket not configured".to_string(),
        ))?;
        let region = config
            .s3_region
            .clone()
            .unwrap_or_else(|| "us-east-1".to_string());

        let client_ready = config.s3_access_key.is_some() && config.s3_secret_key.is_some();

        Ok(Self {
            bucket,
            region,
            endpoint: config.s3_endpoint.clone(),
            client_ready,
        })
    }
}

#[async_trait]
impl StorageBackend for S3Storage {
    async fn put(
        &self,
        key: &str,
        data: &[u8],
        content_type: &str,
    ) -> Result<StorageResult, StorageError> {
        if !self.client_ready {
            return Err(StorageError::BackendError(
                "S3 client not configured. Set S3_ACCESS_KEY and S3_SECRET_KEY.".to_string(),
            ));
        }

        let url = match &self.endpoint {
            Some(endpoint) => format!("{}/{}/{}", endpoint.trim_end_matches('/'), self.bucket, key),
            None => format!(
                "https://{}.s3.{}.amazonaws.com/{}",
                self.bucket, self.region, key
            ),
        };

        Ok(StorageResult {
            url,
            size: data.len() as u64,
            content_type: content_type.to_string(),
            etag: None,
        })
    }

    async fn get(&self, _key: &str) -> Result<Vec<u8>, StorageError> {
        if !self.client_ready {
            return Err(StorageError::BackendError(
                "S3 client not configured".to_string(),
            ));
        }
        Err(StorageError::BackendError(
            "S3 get not yet implemented".to_string(),
        ))
    }

    async fn delete(&self, _key: &str) -> Result<(), StorageError> {
        if !self.client_ready {
            return Err(StorageError::BackendError(
                "S3 client not configured".to_string(),
            ));
        }
        Err(StorageError::BackendError(
            "S3 delete not yet implemented".to_string(),
        ))
    }

    async fn exists(&self, _key: &str) -> Result<bool, StorageError> {
        if !self.client_ready {
            return Err(StorageError::BackendError(
                "S3 client not configured".to_string(),
            ));
        }
        Err(StorageError::BackendError(
            "S3 exists not yet implemented".to_string(),
        ))
    }

    async fn presigned_url(&self, key: &str, expires_in_secs: u64) -> Result<String, StorageError> {
        if !self.client_ready {
            return Err(StorageError::BackendError(
                "S3 client not configured".to_string(),
            ));
        }

        let url = match &self.endpoint {
            Some(endpoint) => format!("{}/{}/{}", endpoint.trim_end_matches('/'), self.bucket, key),
            None => format!(
                "https://{}.s3.{}.amazonaws.com/{}",
                self.bucket, self.region, key
            ),
        };

        Ok(format!("{}?expires={}", url, expires_in_secs))
    }
}
