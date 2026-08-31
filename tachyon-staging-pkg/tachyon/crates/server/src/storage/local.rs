use super::{StorageBackend, StorageError, StorageResult};
use async_trait::async_trait;
use std::path::PathBuf;
use tokio::fs;
use tokio::io::AsyncWriteExt;

pub struct LocalStorage {
    base_path: PathBuf,
    base_url: String,
}

impl LocalStorage {
    pub fn new(base_path: &str, base_url: &str) -> Result<Self, StorageError> {
        let path = PathBuf::from(base_path);
        Ok(Self {
            base_path: path,
            base_url: base_url.trim_end_matches('/').to_string(),
        })
    }

    fn safe_path(&self, key: &str) -> Result<PathBuf, StorageError> {
        if key.contains("..") || key.contains('\0') || key.starts_with('/') {
            return Err(StorageError::InvalidKey(key.to_string()));
        }
        Ok(self.base_path.join(key))
    }
}

#[async_trait]
impl StorageBackend for LocalStorage {
    async fn put(
        &self,
        key: &str,
        data: &[u8],
        content_type: &str,
    ) -> Result<StorageResult, StorageError> {
        let path = self.safe_path(key)?;

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).await?;
        }

        let mut file = fs::File::create(&path).await?;
        file.write_all(data).await?;
        file.flush().await?;

        Ok(StorageResult {
            url: format!("{}/{}", self.base_url, key),
            size: data.len() as u64,
            content_type: content_type.to_string(),
            etag: None,
        })
    }

    async fn get(&self, key: &str) -> Result<Vec<u8>, StorageError> {
        let path = self.safe_path(key)?;

        if !path.exists() {
            return Err(StorageError::NotFound(key.to_string()));
        }

        Ok(fs::read(&path).await?)
    }

    async fn delete(&self, key: &str) -> Result<(), StorageError> {
        let path = self.safe_path(key)?;

        if path.exists() {
            fs::remove_file(&path).await?;
        }

        Ok(())
    }

    async fn exists(&self, key: &str) -> Result<bool, StorageError> {
        let path = self.safe_path(key)?;
        Ok(path.exists())
    }

    async fn presigned_url(
        &self,
        key: &str,
        _expires_in_secs: u64,
    ) -> Result<String, StorageError> {
        Ok(format!("{}/{}", self.base_url, key))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_put_get_delete() {
        let dir = tempdir().unwrap();
        let storage = LocalStorage::new(dir.path().to_str().unwrap(), "/files").unwrap();

        storage
            .put("test/hello.txt", b"Hello, World!", "text/plain")
            .await
            .unwrap();

        let data = storage.get("test/hello.txt").await.unwrap();
        assert_eq!(data, b"Hello, World!");

        assert!(storage.exists("test/hello.txt").await.unwrap());

        storage.delete("test/hello.txt").await.unwrap();
        assert!(!storage.exists("test/hello.txt").await.unwrap());
    }

    #[test]
    fn test_path_traversal_prevented() {
        let storage = LocalStorage::new("/tmp", "/files").unwrap();
        assert!(storage.safe_path("../etc/passwd").is_err());
        assert!(storage.safe_path("/etc/passwd").is_err());
        assert!(storage.safe_path("foo/bar.txt").is_ok());
    }
}
