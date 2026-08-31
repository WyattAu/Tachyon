use super::{StorageBackend, StorageConfig, StorageError, StorageResult};

#[cfg(not(feature = "s3-storage"))]
mod imp {
    use super::*;
    use async_trait::async_trait;

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
                Some(endpoint) => {
                    format!("{}/{}/{}", endpoint.trim_end_matches('/'), self.bucket, key)
                }
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
                "S3 get not yet implemented. Enable the `s3-storage` feature for full S3 support."
                    .to_string(),
            ))
        }

        async fn delete(&self, _key: &str) -> Result<(), StorageError> {
            if !self.client_ready {
                return Err(StorageError::BackendError(
                    "S3 client not configured".to_string(),
                ));
            }
            Err(StorageError::BackendError(
                "S3 delete not yet implemented. Enable the `s3-storage` feature for full S3 support."
                    .to_string(),
            ))
        }

        async fn exists(&self, _key: &str) -> Result<bool, StorageError> {
            if !self.client_ready {
                return Err(StorageError::BackendError(
                    "S3 client not configured".to_string(),
                ));
            }
            Err(StorageError::BackendError(
                "S3 exists not yet implemented. Enable the `s3-storage` feature for full S3 support."
                    .to_string(),
            ))
        }

        async fn presigned_url(
            &self,
            key: &str,
            expires_in_secs: u64,
        ) -> Result<String, StorageError> {
            if !self.client_ready {
                return Err(StorageError::BackendError(
                    "S3 client not configured".to_string(),
                ));
            }

            let url = match &self.endpoint {
                Some(endpoint) => {
                    format!("{}/{}/{}", endpoint.trim_end_matches('/'), self.bucket, key)
                }
                None => format!(
                    "https://{}.s3.{}.amazonaws.com/{}",
                    self.bucket, self.region, key
                ),
            };

            Ok(format!("{}?expires={}", url, expires_in_secs))
        }
    }
}

#[cfg(feature = "s3-storage")]
mod imp {
    use super::*;
    use async_trait::async_trait;
    use aws_sdk_s3::error::ProvideErrorMetadata;
    use aws_sdk_s3::primitives::ByteStream;
    use std::fmt;
    use std::time::Duration;

    pub struct S3Storage {
        bucket: String,
        region: String,
        endpoint: Option<String>,
        client: aws_sdk_s3::Client,
    }

    impl fmt::Debug for S3Storage {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.debug_struct("S3Storage")
                .field("bucket", &self.bucket)
                .field("region", &self.region)
                .field("endpoint", &self.endpoint)
                .field("client", &"<aws_sdk_s3::Client>")
                .finish()
        }
    }

    impl S3Storage {
        pub fn new(config: &StorageConfig) -> Result<Self, StorageError> {
            let bucket = config.s3_bucket.clone().ok_or_else(|| {
                StorageError::BackendError("S3 bucket not configured".to_string())
            })?;

            let access_key = config.s3_access_key.as_ref().ok_or_else(|| {
                StorageError::BackendError("S3 access key not configured".to_string())
            })?;

            let secret_key = config.s3_secret_key.as_ref().ok_or_else(|| {
                StorageError::BackendError("S3 secret key not configured".to_string())
            })?;

            let region = config
                .s3_region
                .clone()
                .unwrap_or_else(|| "us-east-1".to_string());

            let credentials = aws_sdk_s3::config::Credentials::new(
                access_key,
                secret_key,
                None,
                None,
                "tachyon-s3",
            );

            let mut builder = aws_sdk_s3::config::Builder::new()
                .behavior_version(aws_sdk_s3::config::BehaviorVersion::latest())
                .credentials_provider(credentials)
                .region(aws_sdk_s3::config::Region::new(region.clone()));

            if let Some(endpoint) = &config.s3_endpoint {
                builder = builder.endpoint_url(endpoint).force_path_style(true);
            }

            let s3_config = builder.build();
            let client = aws_sdk_s3::Client::from_conf(s3_config);

            tracing::info!(
                bucket = %bucket,
                region = %region,
                has_custom_endpoint = config.s3_endpoint.is_some(),
                "S3 storage backend initialized"
            );

            Ok(Self {
                bucket,
                region,
                endpoint: config.s3_endpoint.clone(),
                client,
            })
        }

        fn object_url(&self, key: &str) -> String {
            match &self.endpoint {
                Some(ep) => {
                    format!("{}/{}/{}", ep.trim_end_matches('/'), self.bucket, key)
                }
                None => {
                    format!(
                        "https://{}.s3.{}.amazonaws.com/{}",
                        self.bucket, self.region, key
                    )
                }
            }
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
            let uploaded_at = chrono::Utc::now().to_rfc3339();
            let size = data.len() as u64;

            let response = self
                .client
                .put_object()
                .bucket(&self.bucket)
                .key(key)
                .body(ByteStream::from(data.to_vec()))
                .content_type(content_type)
                .metadata("uploaded_at", &uploaded_at)
                .send()
                .await
                .map_err(|e| {
                    tracing::warn!(key = %key, error = %e, "S3 put_object failed");
                    StorageError::BackendError(format!("S3 upload failed: {}", e))
                })?;

            let etag = response.e_tag().map(|s| s.to_string());
            let url = self.object_url(key);

            tracing::info!(
                key = %key,
                size = size,
                etag = ?etag,
                url = %url,
                "S3 object uploaded"
            );

            Ok(StorageResult {
                url,
                size,
                content_type: content_type.to_string(),
                etag,
            })
        }

        async fn get(&self, key: &str) -> Result<Vec<u8>, StorageError> {
            let response = self
                .client
                .get_object()
                .bucket(&self.bucket)
                .key(key)
                .send()
                .await
                .map_err(|e| {
                    let code = e.code();
                    if code == Some("NoSuchKey") || code == Some("NotFound") {
                        StorageError::NotFound(key.to_string())
                    } else {
                        tracing::warn!(key = %key, error = %e, "S3 get_object failed");
                        StorageError::BackendError(format!("S3 download failed: {}", e))
                    }
                })?;

            let collected = response.body.collect().await.map_err(|e| {
                tracing::warn!(key = %key, error = ?e, "Failed to read S3 response body");
                StorageError::BackendError(format!("Failed to read S3 response body: {:?}", e))
            })?;
            let data = collected.into_bytes().to_vec();

            tracing::info!(key = %key, size = data.len(), "S3 object downloaded");

            Ok(data)
        }

        async fn delete(&self, key: &str) -> Result<(), StorageError> {
            self.client
                .delete_object()
                .bucket(&self.bucket)
                .key(key)
                .send()
                .await
                .map_err(|e| {
                    tracing::warn!(key = %key, error = %e, "S3 delete_object failed");
                    StorageError::BackendError(format!("S3 delete failed: {}", e))
                })?;

            tracing::info!(key = %key, "S3 object deleted");

            Ok(())
        }

        async fn exists(&self, key: &str) -> Result<bool, StorageError> {
            match self
                .client
                .head_object()
                .bucket(&self.bucket)
                .key(key)
                .send()
                .await
            {
                Ok(_) => Ok(true),
                Err(e) => {
                    let code = e.code();
                    if code == Some("NotFound")
                        || code == Some("Not Found")
                        || code == Some("NoSuchKey")
                        || code == Some("404")
                    {
                        return Ok(false);
                    }
                    tracing::warn!(key = %key, error = %e, "S3 head_object failed");
                    Err(StorageError::BackendError(format!(
                        "S3 head_object failed: {}",
                        e
                    )))
                }
            }
        }

        async fn presigned_url(
            &self,
            key: &str,
            expires_in_secs: u64,
        ) -> Result<String, StorageError> {
            use aws_sdk_s3::presigning::PresigningConfig;

            let presigned = self
                .client
                .get_object()
                .bucket(&self.bucket)
                .key(key)
                .presigned(
                    PresigningConfig::expires_in(Duration::from_secs(expires_in_secs)).map_err(
                        |e| StorageError::BackendError(format!("Invalid presign duration: {}", e)),
                    )?,
                )
                .await
                .map_err(|e| {
                    tracing::warn!(
                        key = %key,
                        error = %e,
                        "Failed to generate presigned URL"
                    );
                    StorageError::BackendError(format!("Failed to generate presigned URL: {}", e))
                })?;

            let url = presigned.uri().to_string();

            tracing::info!(
                key = %key,
                expires_in_secs = expires_in_secs,
                "Presigned URL generated"
            );

            Ok(url)
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        fn full_config() -> StorageConfig {
            StorageConfig {
                backend: "s3".to_string(),
                base_url: "/files".to_string(),
                local_path: "./uploads".to_string(),
                s3_bucket: Some("test-bucket".to_string()),
                s3_region: Some("us-east-1".to_string()),
                s3_endpoint: Some("http://localhost:9000".to_string()),
                s3_access_key: Some("minioadmin".to_string()),
                s3_secret_key: Some("minioadmin".to_string()),
            }
        }

        #[test]
        fn test_new_with_all_fields() {
            let config = full_config();
            let storage = S3Storage::new(&config).unwrap();
            assert_eq!(storage.bucket, "test-bucket");
            assert_eq!(storage.region, "us-east-1");
            assert_eq!(storage.endpoint, Some("http://localhost:9000".to_string()));
        }

        #[test]
        fn test_new_missing_bucket() {
            let mut config = full_config();
            config.s3_bucket = None;
            let err = S3Storage::new(&config).unwrap_err();
            assert!(err.to_string().contains("bucket"));
        }

        #[test]
        fn test_new_missing_access_key() {
            let mut config = full_config();
            config.s3_access_key = None;
            let err = S3Storage::new(&config).unwrap_err();
            assert!(err.to_string().contains("access key"));
        }

        #[test]
        fn test_new_missing_secret_key() {
            let mut config = full_config();
            config.s3_secret_key = None;
            let err = S3Storage::new(&config).unwrap_err();
            assert!(err.to_string().contains("secret key"));
        }

        #[test]
        fn test_new_defaults_region_when_missing() {
            let mut config = full_config();
            config.s3_region = None;
            let storage = S3Storage::new(&config).unwrap();
            assert_eq!(storage.region, "us-east-1");
        }

        #[test]
        fn test_new_aws_without_endpoint() {
            let mut config = full_config();
            config.s3_endpoint = None;
            let storage = S3Storage::new(&config).unwrap();
            assert!(storage.endpoint.is_none());
        }

        #[test]
        fn test_object_url_custom_endpoint() {
            let storage = S3Storage::new(&full_config()).unwrap();
            let url = storage.object_url("docs/file.pdf");
            assert_eq!(url, "http://localhost:9000/test-bucket/docs/file.pdf");
        }

        #[test]
        fn test_object_url_aws() {
            let mut config = full_config();
            config.s3_endpoint = None;
            config.s3_region = Some("eu-west-2".to_string());
            let storage = S3Storage::new(&config).unwrap();
            let url = storage.object_url("docs/file.pdf");
            assert_eq!(
                url,
                "https://test-bucket.s3.eu-west-2.amazonaws.com/docs/file.pdf"
            );
        }

        #[test]
        fn test_object_url_trailing_slash_stripped() {
            let mut config = full_config();
            config.s3_endpoint = Some("http://localhost:9000/".to_string());
            let storage = S3Storage::new(&config).unwrap();
            let url = storage.object_url("a/b");
            assert_eq!(url, "http://localhost:9000/test-bucket/a/b");
        }

        #[tokio::test]
        async fn test_exists_returns_not_found_for_missing_object() {
            let storage = S3Storage::new(&full_config()).unwrap();
            let result = storage.exists("no-such-key").await;
            assert!(
                result.is_err(),
                "Expected error when connecting to non-existent S3 endpoint"
            );
        }
    }
}

pub use imp::*;
