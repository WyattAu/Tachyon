//! Ed25519 plugin signing.
//!
//! Provides cryptographic signing and verification for WASM plugins using Ed25519.

use chrono::Utc;
use ed25519_dalek::{Signature, Signer, Verifier, VerifyingKey};
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::error::PluginRuntimeError;

/// A cryptographic signature over a plugin's WASM binary.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PluginSignature {
    /// First 8 bytes of the public key (hex), identifies the signer.
    pub key_id: String,
    /// 64-byte Ed25519 signature over the WASM binary.
    pub signature: Vec<u8>,
    /// Unix timestamp when the signature was created.
    pub timestamp: i64,
}

/// An Ed25519 signing key pair for plugin signing.
#[derive(Debug)]
pub struct SigningKeyPair {
    signing_key: ed25519_dalek::SigningKey,
    verifying_key: VerifyingKey,
}

impl SigningKeyPair {
    /// Generate a new random Ed25519 key pair.
    pub fn generate() -> Self {
        let signing_key = ed25519_dalek::SigningKey::generate(&mut OsRng);
        let verifying_key = signing_key.verifying_key();
        Self {
            signing_key,
            verifying_key,
        }
    }

    /// Import from raw 32-byte seed.
    pub fn from_bytes(seed: &[u8; 32]) -> Result<Self, PluginRuntimeError> {
        let signing_key = ed25519_dalek::SigningKey::from_bytes(seed);
        let verifying_key = signing_key.verifying_key();
        Ok(Self {
            signing_key,
            verifying_key,
        })
    }

    /// Export the public key as 32 bytes.
    pub fn public_key_bytes(&self) -> [u8; 32] {
        self.verifying_key.to_bytes()
    }

    /// Export the seed (secret key) as a slice.
    pub fn seed_bytes(&self) -> &[u8] {
        self.signing_key.as_bytes()
    }

    /// Get the key ID (first 8 bytes of public key, hex-encoded).
    pub fn key_id(&self) -> String {
        hex::encode(&self.public_key_bytes()[..8])
    }

    /// Sign data and return a PluginSignature.
    pub fn sign(&self, data: &[u8]) -> PluginSignature {
        let signature = self.signing_key.sign(data);
        PluginSignature {
            key_id: self.key_id(),
            signature: signature.to_bytes().to_vec(),
            timestamp: Utc::now().timestamp(),
        }
    }

    /// Verify a signature against this key pair's public key.
    pub fn verify(&self, data: &[u8], sig: &PluginSignature) -> bool {
        if let Ok(signature) = Signature::from_slice(&sig.signature) {
            self.verifying_key.verify(data, &signature).is_ok()
        } else {
            false
        }
    }

    /// Verify a signature using only a public key (for third-party verification).
    pub fn verify_with_public_key(
        public_key_bytes: &[u8],
        data: &[u8],
        sig: &PluginSignature,
    ) -> bool {
        let key_array: [u8; 32] = match public_key_bytes.try_into() {
            Ok(arr) => arr,
            Err(_) => return false,
        };
        let Ok(verifying_key) = VerifyingKey::from_bytes(&key_array) else {
            return false;
        };
        let Ok(signature) = Signature::from_slice(&sig.signature) else {
            return false;
        };
        verifying_key.verify(data, &signature).is_ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sign_and_verify() {
        let keypair = SigningKeyPair::generate();
        let data = b"hello, world!";
        let sig = keypair.sign(data);
        assert_eq!(sig.key_id, keypair.key_id());
        assert_eq!(sig.signature.len(), 64);
        assert!(keypair.verify(data, &sig));
    }

    #[test]
    fn test_verify_wrong_data() {
        let keypair = SigningKeyPair::generate();
        let sig = keypair.sign(b"correct data");
        assert!(!keypair.verify(b"wrong data", &sig));
    }

    #[test]
    fn test_verify_with_public_key() {
        let keypair = SigningKeyPair::generate();
        let data = b"test data";
        let sig = keypair.sign(data);
        let pub_key = keypair.public_key_bytes();
        assert!(SigningKeyPair::verify_with_public_key(&pub_key, data, &sig));
    }

    #[test]
    fn test_verify_with_public_key_wrong_data() {
        let keypair = SigningKeyPair::generate();
        let sig = keypair.sign(b"test data");
        let pub_key = keypair.public_key_bytes();
        assert!(!SigningKeyPair::verify_with_public_key(
            &pub_key, b"wrong", &sig
        ));
    }

    #[test]
    fn test_key_id_consistency() {
        let keypair = SigningKeyPair::generate();
        let id1 = keypair.key_id();
        let id2 = keypair.key_id();
        assert_eq!(id1, id2);
        assert_eq!(id1.len(), 16);
    }

    #[test]
    fn test_from_bytes_roundtrip() {
        let keypair = SigningKeyPair::generate();
        let seed = keypair.seed_bytes();
        let seed_array: [u8; 32] = seed.try_into().unwrap();
        let restored = SigningKeyPair::from_bytes(&seed_array).unwrap();
        assert_eq!(keypair.public_key_bytes(), restored.public_key_bytes());

        let data = b"roundtrip test";
        let sig = keypair.sign(data);
        assert!(restored.verify(data, &sig));
    }

    #[test]
    fn test_signature_serialization() {
        let keypair = SigningKeyPair::generate();
        let sig = keypair.sign(b"serialize me");
        let json = serde_json::to_string(&sig).unwrap();
        let deserialized: PluginSignature = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.key_id, sig.key_id);
        assert_eq!(deserialized.signature, sig.signature);
        assert_eq!(deserialized.timestamp, sig.timestamp);
    }

    #[test]
    fn test_verify_with_public_key_invalid_length() {
        let keypair = SigningKeyPair::generate();
        let sig = keypair.sign(b"test");
        assert!(!SigningKeyPair::verify_with_public_key(
            &[0u8; 16], b"test", &sig
        ));
    }

    #[test]
    fn test_verify_with_public_key_invalid_signature() {
        let keypair = SigningKeyPair::generate();
        let pub_key = keypair.public_key_bytes();
        let bad_sig = PluginSignature {
            key_id: "00000000".to_string(),
            signature: vec![0u8; 64],
            timestamp: 0,
        };
        assert!(!SigningKeyPair::verify_with_public_key(
            &pub_key, b"test", &bad_sig
        ));
    }
}
