//! E2E encryption API client methods.

use super::*;
use serde::{Deserialize, Serialize};

/// Encryption key metadata from the server.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptionKeyMeta {
    pub id: String,
    pub document_id: String,
    pub owner_id: String,
    pub key_algorithm: String,
    pub public_key_fingerprint: String,
    pub created_at: String,
}

/// Encryption status for a document.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptionStatusResponse {
    pub document_id: String,
    pub encrypted: bool,
    pub key_algorithm: Option<String>,
    pub key_fingerprint: Option<String>,
}

/// Register key request body.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterKeyRequest {
    pub document_id: String,
    pub key_algorithm: String,
    pub public_key_fingerprint: String,
}

/// Key list response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyListResponse {
    pub keys: Vec<EncryptionKeyMeta>,
    pub total: usize,
}

/// Delete key response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteKeyResponse {
    pub deleted: bool,
    pub key_id: String,
}

/// E2E encryption API methods.
impl ApiClient {
    /// Register an encryption key for a document.
    pub async fn register_encryption_key(
        &self,
        document_id: &str,
        key_algorithm: &str,
        public_key_fingerprint: &str,
    ) -> Result<serde_json::Value, ApiError> {
        let url = format!("{}/e2e/keys", self.base_url);
        let body = RegisterKeyRequest {
            document_id: document_id.to_string(),
            key_algorithm: key_algorithm.to_string(),
            public_key_fingerprint: public_key_fingerprint.to_string(),
        };
        self.post(&url, &body).await
    }

    /// Get the encryption key metadata for a document.
    pub async fn get_encryption_key(
        &self,
        document_id: &str,
    ) -> Result<EncryptionKeyMeta, ApiError> {
        let url = format!("{}/e2e/keys/{}", self.base_url, document_id);
        self.get(&url).await
    }

    /// Delete the encryption key for a document.
    pub async fn delete_encryption_key(&self, document_id: &str) -> Result<(), ApiError> {
        let url = format!("{}/e2e/keys/{}", self.base_url, document_id);
        self.delete(&url).await
    }

    /// Get encryption status for a document.
    pub async fn get_encryption_status(
        &self,
        document_id: &str,
    ) -> Result<EncryptionStatusResponse, ApiError> {
        let url = format!("{}/e2e/status/{}", self.base_url, document_id);
        self.get(&url).await
    }

    /// List all encryption keys owned by the authenticated user.
    pub async fn list_encryption_keys(&self) -> Result<KeyListResponse, ApiError> {
        let url = format!("{}/e2e/keys", self.base_url);
        self.get(&url).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_key_request() {
        let req = RegisterKeyRequest {
            document_id: "abc-123".to_string(),
            key_algorithm: "aes-256-gcm".to_string(),
            public_key_fingerprint: "sha256:abc123".to_string(),
        };
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("aes-256-gcm"));
    }

    #[test]
    fn test_encryption_status_response() {
        let status = EncryptionStatusResponse {
            document_id: "doc-1".to_string(),
            encrypted: true,
            key_algorithm: Some("aes-256-gcm".to_string()),
            key_fingerprint: Some("sha256:fp".to_string()),
        };
        assert!(status.encrypted);
    }

    #[test]
    fn test_delete_key_response() {
        let resp = DeleteKeyResponse {
            deleted: true,
            key_id: "abc".to_string(),
        };
        assert!(resp.deleted);
    }
}
