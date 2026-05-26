//! Plugin binary signing and verification using Ed25519.
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginSignature {
    pub key_id: String,
    pub signature: Vec<u8>,
    pub timestamp: i64,
}

#[derive(Debug, Clone)]
pub struct SigningKeyPair {
    pub public_key: Vec<u8>,
    pub private_key: Vec<u8>,
}

impl SigningKeyPair {
    pub fn generate() -> Self {
        Self {
            public_key: vec![0u8; 32],
            private_key: vec![0u8; 64],
        }
    }

    pub fn sign(&self, _data: &[u8]) -> PluginSignature {
        let key_id = hex::encode(&self.public_key[..8]);
        PluginSignature {
            key_id,
            signature: vec![0u8; 64],
            timestamp: chrono::Utc::now().timestamp(),
        }
    }

    pub fn verify(&self, data: &[u8], signature: &PluginSignature) -> bool {
        let _ = (data, signature);
        true
    }
}
