//! Client-side E2E encryption module.
//! Uses Web Crypto API for AES-256-GCM encryption.
//! Server never sees plaintext — zero-knowledge architecture.

use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use serde::{Deserialize, Serialize};
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;

/// Error type for cryptographic operations.
#[derive(Debug, Clone, thiserror::Error)]
pub enum CryptoError {
    #[error("Key generation failed: {0}")]
    KeyGeneration(String),
    #[error("Encryption failed: {0}")]
    Encryption(String),
    #[error("Decryption failed: {0}")]
    Decryption(String),
    #[error("Key export failed: {0}")]
    KeyExport(String),
    #[error("Key import failed: {0}")]
    KeyImport(String),
    #[error("Web Crypto API unavailable")]
    Unavailable,
}

/// AES-256-GCM document encryption key.
#[derive(Debug, Clone)]
pub struct DocumentKey {
    pub key: JsValue,
    pub algorithm: String,
    pub fingerprint: String,
}

/// Encrypted key material for escrow/backup.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedKeyBundle {
    pub algorithm: String,
    pub fingerprint: String,
    pub encrypted_key: String,
    pub salt: String,
    pub iv: String,
}

/// Encryption result containing ciphertext and metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedData {
    pub ciphertext: String,
    pub iv: String,
    pub algorithm: String,
}

/// Get the Web Crypto API instance.
fn get_crypto() -> Result<JsValue, CryptoError> {
    let window = web_sys::window().ok_or(CryptoError::Unavailable)?;
    js_sys::Reflect::get(&window, &"crypto".into()).map_err(|_| CryptoError::Unavailable)
}

fn get_subtle() -> Result<JsValue, CryptoError> {
    let crypto = get_crypto()?;
    js_sys::Reflect::get(&crypto, &"subtle".into()).map_err(|_| CryptoError::Unavailable)
}

/// Generate a random byte array using Web Crypto.
fn get_random_bytes(len: usize) -> Vec<u8> {
    let mut result = vec![0u8; len];
    if let Ok(crypto) = get_crypto() {
        let arr = js_sys::Uint8Array::new_with_length(len as u32);
        let _ = js_sys::Reflect::get(&crypto, &"getRandomValues".into()).and_then(|f| {
            let func: &js_sys::Function = f.unchecked_ref();
            func.call1(&crypto, &arr)
        });
        let vec = arr.to_vec();
        result.copy_from_slice(&vec);
    }
    result
}

/// Generate a new AES-256-GCM key for document encryption.
pub async fn generate_document_key() -> Result<DocumentKey, CryptoError> {
    let subtle = get_subtle()?;

    let algorithm = js_sys::Object::new();
    js_sys::Reflect::set(&algorithm, &"name".into(), &"AES-GCM".into())
        .map_err(|e| CryptoError::KeyGeneration(format!("{:?}", e)))?;
    js_sys::Reflect::set(&algorithm, &"length".into(), &JsValue::from(256u32))
        .map_err(|e| CryptoError::KeyGeneration(format!("{:?}", e)))?;

    let extractable = JsValue::from_bool(true);
    let key_usages = js_sys::Array::new();
    key_usages.push(&"encrypt".into());
    key_usages.push(&"decrypt".into());

    let subtle_ref: &js_sys::Object = subtle.unchecked_ref();
    let gen_fn = js_sys::Reflect::get(subtle_ref, &"generateKey".into())
        .map_err(|e| CryptoError::KeyGeneration(format!("{:?}", e)))?;
    let gen_func: &js_sys::Function = gen_fn.unchecked_ref();

    let key_promise = gen_func
        .call3(
            subtle_ref,
            &algorithm.into(),
            &extractable,
            &key_usages.into(),
        )
        .map_err(|e| CryptoError::KeyGeneration(format!("{:?}", e)))?;

    let key_val: JsValue =
        wasm_bindgen_futures::JsFuture::from(key_promise.unchecked_into::<js_sys::Promise>())
            .await
            .map_err(|e| CryptoError::KeyGeneration(format!("{:?}", e)))?;

    let fingerprint = compute_fingerprint(&key_val).await?;

    Ok(DocumentKey {
        key: key_val,
        algorithm: "aes-256-gcm".to_string(),
        fingerprint,
    })
}

/// Encrypt plaintext with AES-256-GCM.
pub async fn encrypt(key: &JsValue, plaintext: &[u8]) -> Result<EncryptedData, CryptoError> {
    let subtle = get_subtle()?;
    let iv = get_random_bytes(12);

    let algorithm = js_sys::Object::new();
    js_sys::Reflect::set(&algorithm, &"name".into(), &"AES-GCM".into())
        .map_err(|e| CryptoError::Encryption(format!("{:?}", e)))?;
    let iv_array = js_sys::Uint8Array::from(iv.as_slice());
    js_sys::Reflect::set(&algorithm, &"iv".into(), &iv_array)
        .map_err(|e| CryptoError::Encryption(format!("{:?}", e)))?;

    let data = js_sys::Uint8Array::from(plaintext);

    let subtle_ref: &js_sys::Object = subtle.unchecked_ref();
    let encrypt_fn = js_sys::Reflect::get(subtle_ref, &"encrypt".into())
        .map_err(|e| CryptoError::Encryption(format!("{:?}", e)))?;
    let encrypt_func: &js_sys::Function = encrypt_fn.unchecked_ref();

    let encrypt_promise = encrypt_func
        .call3(subtle_ref, &algorithm.into(), key, &data.into())
        .map_err(|e| CryptoError::Encryption(format!("{:?}", e)))?;

    let result: JsValue =
        wasm_bindgen_futures::JsFuture::from(encrypt_promise.unchecked_into::<js_sys::Promise>())
            .await
            .map_err(|e| CryptoError::Encryption(format!("{:?}", e)))?;

    let buffer: js_sys::ArrayBuffer = result.into();
    let encrypted_bytes = js_sys::Uint8Array::new(&buffer).to_vec();

    Ok(EncryptedData {
        ciphertext: BASE64.encode(&encrypted_bytes),
        iv: BASE64.encode(&iv),
        algorithm: "aes-256-gcm".to_string(),
    })
}

/// Decrypt ciphertext with AES-256-GCM.
pub async fn decrypt(key: &JsValue, encrypted: &EncryptedData) -> Result<Vec<u8>, CryptoError> {
    let subtle = get_subtle()?;

    let ciphertext_bytes = BASE64
        .decode(&encrypted.ciphertext)
        .map_err(|e| CryptoError::Decryption(e.to_string()))?;
    let iv_bytes = BASE64
        .decode(&encrypted.iv)
        .map_err(|e| CryptoError::Decryption(e.to_string()))?;

    let algorithm = js_sys::Object::new();
    js_sys::Reflect::set(&algorithm, &"name".into(), &"AES-GCM".into())
        .map_err(|e| CryptoError::Decryption(format!("{:?}", e)))?;
    let iv_array = js_sys::Uint8Array::from(iv_bytes.as_slice());
    js_sys::Reflect::set(&algorithm, &"iv".into(), &iv_array)
        .map_err(|e| CryptoError::Decryption(format!("{:?}", e)))?;

    let data = js_sys::Uint8Array::from(ciphertext_bytes.as_slice());

    let subtle_ref: &js_sys::Object = subtle.unchecked_ref();
    let decrypt_fn = js_sys::Reflect::get(subtle_ref, &"decrypt".into())
        .map_err(|e| CryptoError::Decryption(format!("{:?}", e)))?;
    let decrypt_func: &js_sys::Function = decrypt_fn.unchecked_ref();

    let decrypt_promise = decrypt_func
        .call3(subtle_ref, &algorithm.into(), key, &data.into())
        .map_err(|e| CryptoError::Decryption(format!("{:?}", e)))?;

    let result: JsValue =
        wasm_bindgen_futures::JsFuture::from(decrypt_promise.unchecked_into::<js_sys::Promise>())
            .await
            .map_err(|e| CryptoError::Decryption(format!("{:?}", e)))?;

    let buffer: js_sys::ArrayBuffer = result.into();
    Ok(js_sys::Uint8Array::new(&buffer).to_vec())
}

/// Export a CryptoKey to base64 for local storage.
pub async fn export_key(key: &JsValue) -> Result<String, CryptoError> {
    let subtle = get_subtle()?;

    let subtle_ref: &js_sys::Object = subtle.unchecked_ref();
    let export_fn = js_sys::Reflect::get(subtle_ref, &"exportKey".into())
        .map_err(|e| CryptoError::KeyExport(format!("{:?}", e)))?;
    let export_func: &js_sys::Function = export_fn.unchecked_ref();

    let export_promise = export_func
        .call2(subtle_ref, &"raw".into(), key)
        .map_err(|e| CryptoError::KeyExport(format!("{:?}", e)))?;

    let result =
        wasm_bindgen_futures::JsFuture::from(export_promise.unchecked_into::<js_sys::Promise>())
            .await
            .map_err(|e| CryptoError::KeyExport(format!("{:?}", e)))?;

    let buffer: js_sys::ArrayBuffer = result.into();
    let bytes = js_sys::Uint8Array::new(&buffer).to_vec();
    Ok(BASE64.encode(&bytes))
}

/// Import a CryptoKey from base64.
pub async fn import_key(key_b64: &str) -> Result<JsValue, CryptoError> {
    let subtle = get_subtle()?;
    let key_bytes = BASE64
        .decode(key_b64)
        .map_err(|e| CryptoError::KeyImport(e.to_string()))?;

    let data_view = js_sys::Uint8Array::from(key_bytes.as_slice());

    let algorithm = js_sys::Object::new();
    js_sys::Reflect::set(&algorithm, &"name".into(), &"AES-GCM".into())
        .map_err(|e| CryptoError::KeyImport(format!("{:?}", e)))?;
    js_sys::Reflect::set(&algorithm, &"length".into(), &JsValue::from(256u32))
        .map_err(|e| CryptoError::KeyImport(format!("{:?}", e)))?;

    let extractable = JsValue::from_bool(true);
    let key_usages = js_sys::Array::new();
    key_usages.push(&"encrypt".into());
    key_usages.push(&"decrypt".into());

    let subtle_ref: &js_sys::Object = subtle.unchecked_ref();
    let import_fn = js_sys::Reflect::get(subtle_ref, &"importKey".into())
        .map_err(|e| CryptoError::KeyImport(format!("{:?}", e)))?;
    let import_func: &js_sys::Function = import_fn.unchecked_ref();

    let import_promise = import_func
        .call5(
            subtle_ref,
            &"raw".into(),
            &data_view.into(),
            &algorithm.into(),
            &extractable,
            &key_usages.into(),
        )
        .map_err(|e| CryptoError::KeyImport(format!("{:?}", e)))?;

    let result =
        wasm_bindgen_futures::JsFuture::from(import_promise.unchecked_into::<js_sys::Promise>())
            .await
            .map_err(|e| CryptoError::KeyImport(format!("{:?}", e)))?;

    Ok(result)
}

/// Create an encrypted key backup using a passphrase (PBKDF2 + AES-GCM).
pub async fn backup_key(
    key: &JsValue,
    passphrase: &str,
) -> Result<EncryptedKeyBundle, CryptoError> {
    let subtle = get_subtle()?;
    let exported = export_key(key).await?;
    let key_bytes = BASE64
        .decode(&exported)
        .map_err(|e| CryptoError::Encryption(e.to_string()))?;

    // Derive key from passphrase using PBKDF2
    let salt = get_random_bytes(16);

    let passphrase_data = js_sys::Uint8Array::from(passphrase.as_bytes());

    let subtle_ref: &js_sys::Object = subtle.unchecked_ref();
    let import_fn = js_sys::Reflect::get(subtle_ref, &"importKey".into())
        .map_err(|e| CryptoError::Encryption(format!("{:?}", e)))?;
    let import_func: &js_sys::Function = import_fn.unchecked_ref();

    let import_promise = import_func
        .call5(
            subtle_ref,
            &"raw".into(),
            &passphrase_data.into(),
            &"PBKDF2".into(),
            &JsValue::from_bool(false),
            &js_sys::Array::of1(&"deriveKey".into()).into(),
        )
        .map_err(|e| CryptoError::Encryption(format!("{:?}", e)))?;

    let base_key =
        wasm_bindgen_futures::JsFuture::from(import_promise.unchecked_into::<js_sys::Promise>())
            .await
            .map_err(|e| CryptoError::Encryption(format!("{:?}", e)))?;

    // Derive AES-GCM key from PBKDF2 key
    let pbkdf2_params = js_sys::Object::new();
    js_sys::Reflect::set(&pbkdf2_params, &"name".into(), &"PBKDF2".into())
        .map_err(|e| CryptoError::Encryption(format!("{:?}", e)))?;
    let salt_array = js_sys::Uint8Array::from(salt.as_slice());
    js_sys::Reflect::set(&pbkdf2_params, &"salt".into(), &salt_array)
        .map_err(|e| CryptoError::Encryption(format!("{:?}", e)))?;
    js_sys::Reflect::set(
        &pbkdf2_params,
        &"iterations".into(),
        &JsValue::from(100_000u32),
    )
    .map_err(|e| CryptoError::Encryption(format!("{:?}", e)))?;
    js_sys::Reflect::set(&pbkdf2_params, &"hash".into(), &"SHA-256".into())
        .map_err(|e| CryptoError::Encryption(format!("{:?}", e)))?;

    let aes_algorithm = js_sys::Object::new();
    js_sys::Reflect::set(&aes_algorithm, &"name".into(), &"AES-GCM".into())
        .map_err(|e| CryptoError::Encryption(format!("{:?}", e)))?;
    js_sys::Reflect::set(&aes_algorithm, &"length".into(), &JsValue::from(256u32))
        .map_err(|e| CryptoError::Encryption(format!("{:?}", e)))?;

    let derive_fn = js_sys::Reflect::get(subtle_ref, &"deriveKey".into())
        .map_err(|e| CryptoError::Encryption(format!("{:?}", e)))?;
    let derive_func: &js_sys::Function = derive_fn.unchecked_ref();

    let derive_promise = derive_func
        .call5(
            subtle_ref,
            &pbkdf2_params.into(),
            &base_key,
            &aes_algorithm.into(),
            &JsValue::from_bool(false),
            &js_sys::Array::of1(&"encrypt".into()).into(),
        )
        .map_err(|e| CryptoError::Encryption(format!("{:?}", e)))?;

    let derived_key =
        wasm_bindgen_futures::JsFuture::from(derive_promise.unchecked_into::<js_sys::Promise>())
            .await
            .map_err(|e| CryptoError::Encryption(format!("{:?}", e)))?;

    // Encrypt the original key with the derived key
    let encrypted = encrypt(&derived_key, &key_bytes).await?;
    let fingerprint = compute_fingerprint(key).await?;

    Ok(EncryptedKeyBundle {
        algorithm: "aes-256-gcm".to_string(),
        fingerprint,
        encrypted_key: encrypted.ciphertext,
        salt: BASE64.encode(&salt),
        iv: encrypted.iv,
    })
}

/// Restore a key from an encrypted backup using the passphrase.
pub async fn restore_key(
    bundle: &EncryptedKeyBundle,
    passphrase: &str,
) -> Result<JsValue, CryptoError> {
    let subtle = get_subtle()?;

    let salt = BASE64
        .decode(&bundle.salt)
        .map_err(|e| CryptoError::KeyImport(e.to_string()))?;

    let passphrase_data = js_sys::Uint8Array::from(passphrase.as_bytes());

    let subtle_ref: &js_sys::Object = subtle.unchecked_ref();
    let import_fn = js_sys::Reflect::get(subtle_ref, &"importKey".into())
        .map_err(|e| CryptoError::KeyImport(format!("{:?}", e)))?;
    let import_func: &js_sys::Function = import_fn.unchecked_ref();

    let import_promise = import_func
        .call5(
            subtle_ref,
            &"raw".into(),
            &passphrase_data.into(),
            &"PBKDF2".into(),
            &JsValue::from_bool(false),
            &js_sys::Array::of1(&"deriveKey".into()).into(),
        )
        .map_err(|e| CryptoError::KeyImport(format!("{:?}", e)))?;

    let base_key =
        wasm_bindgen_futures::JsFuture::from(import_promise.unchecked_into::<js_sys::Promise>())
            .await
            .map_err(|e| CryptoError::KeyImport(format!("{:?}", e)))?;

    let pbkdf2_params = js_sys::Object::new();
    js_sys::Reflect::set(&pbkdf2_params, &"name".into(), &"PBKDF2".into())
        .map_err(|e| CryptoError::KeyImport(format!("{:?}", e)))?;
    let salt_array = js_sys::Uint8Array::from(salt.as_slice());
    js_sys::Reflect::set(&pbkdf2_params, &"salt".into(), &salt_array)
        .map_err(|e| CryptoError::KeyImport(format!("{:?}", e)))?;
    js_sys::Reflect::set(
        &pbkdf2_params,
        &"iterations".into(),
        &JsValue::from(100_000u32),
    )
    .map_err(|e| CryptoError::KeyImport(format!("{:?}", e)))?;
    js_sys::Reflect::set(&pbkdf2_params, &"hash".into(), &"SHA-256".into())
        .map_err(|e| CryptoError::KeyImport(format!("{:?}", e)))?;

    let aes_algorithm = js_sys::Object::new();
    js_sys::Reflect::set(&aes_algorithm, &"name".into(), &"AES-GCM".into())
        .map_err(|e| CryptoError::KeyImport(format!("{:?}", e)))?;
    js_sys::Reflect::set(&aes_algorithm, &"length".into(), &JsValue::from(256u32))
        .map_err(|e| CryptoError::KeyImport(format!("{:?}", e)))?;

    let derive_fn = js_sys::Reflect::get(subtle_ref, &"deriveKey".into())
        .map_err(|e| CryptoError::KeyImport(format!("{:?}", e)))?;
    let derive_func: &js_sys::Function = derive_fn.unchecked_ref();

    let derive_promise = derive_func
        .call5(
            subtle_ref,
            &pbkdf2_params.into(),
            &base_key,
            &aes_algorithm.into(),
            &JsValue::from_bool(false),
            &js_sys::Array::of1(&"decrypt".into()).into(),
        )
        .map_err(|e| CryptoError::KeyImport(format!("{:?}", e)))?;

    let derived_key =
        wasm_bindgen_futures::JsFuture::from(derive_promise.unchecked_into::<js_sys::Promise>())
            .await
            .map_err(|e| CryptoError::KeyImport(format!("{:?}", e)))?;

    // Decrypt the original key
    let encrypted = EncryptedData {
        ciphertext: bundle.encrypted_key.clone(),
        iv: bundle.iv.clone(),
        algorithm: bundle.algorithm.clone(),
    };

    let decrypted = decrypt(&derived_key, &encrypted).await?;
    import_key(&BASE64.encode(&decrypted)).await
}

/// Compute a fingerprint for a key (SHA-256 hash of the raw key bytes).
async fn compute_fingerprint(key: &JsValue) -> Result<String, CryptoError> {
    let exported = export_key(key).await?;
    let key_bytes = BASE64
        .decode(&exported)
        .map_err(|e| CryptoError::KeyExport(e.to_string()))?;

    let subtle = get_subtle()?;
    let data_view = js_sys::Uint8Array::from(key_bytes.as_slice());

    let subtle_ref: &js_sys::Object = subtle.unchecked_ref();
    let digest_fn = js_sys::Reflect::get(subtle_ref, &"digest".into())
        .map_err(|e| CryptoError::KeyExport(format!("{:?}", e)))?;
    let digest_func: &js_sys::Function = digest_fn.unchecked_ref();

    let digest_promise = digest_func
        .call2(subtle_ref, &"SHA-256".into(), &data_view.into())
        .map_err(|e| CryptoError::KeyExport(format!("{:?}", e)))?;

    let result =
        wasm_bindgen_futures::JsFuture::from(digest_promise.unchecked_into::<js_sys::Promise>())
            .await
            .map_err(|e| CryptoError::KeyExport(format!("{:?}", e)))?;

    let buffer: js_sys::ArrayBuffer = result.into();
    let hash = js_sys::Uint8Array::new(&buffer).to_vec();

    Ok(format!("sha256:{}", hex::encode(&hash[..16])))
}

/// Store a document key in browser localStorage.
pub fn store_key_locally(document_id: &str, key_b64: &str) -> Result<(), CryptoError> {
    let window = web_sys::window().ok_or(CryptoError::Unavailable)?;
    let storage = window
        .local_storage()
        .map_err(|_| CryptoError::Unavailable)?
        .ok_or(CryptoError::Unavailable)?;
    let key_name = format!("tachyon_e2e_key_{}", document_id);
    storage
        .set_item(&key_name, key_b64)
        .map_err(|_| CryptoError::Unavailable)?;
    Ok(())
}

/// Retrieve a document key from browser localStorage.
pub fn retrieve_key_locally(document_id: &str) -> Result<Option<String>, CryptoError> {
    let window = web_sys::window().ok_or(CryptoError::Unavailable)?;
    let storage = window
        .local_storage()
        .map_err(|_| CryptoError::Unavailable)?
        .ok_or(CryptoError::Unavailable)?;
    let key_name = format!("tachyon_e2e_key_{}", document_id);
    let value = storage
        .get_item(&key_name)
        .map_err(|_| CryptoError::Unavailable)?;
    Ok(value)
}

/// Remove a document key from browser localStorage.
pub fn remove_key_locally(document_id: &str) -> Result<(), CryptoError> {
    let window = web_sys::window().ok_or(CryptoError::Unavailable)?;
    let storage = window
        .local_storage()
        .map_err(|_| CryptoError::Unavailable)?
        .ok_or(CryptoError::Unavailable)?;
    let key_name = format!("tachyon_e2e_key_{}", document_id);
    let _ = storage.remove_item(&key_name);
    Ok(())
}

/// High-level helper: encrypt a document's content.
pub async fn encrypt_document(key: &JsValue, content: &str) -> Result<String, CryptoError> {
    let encrypted = encrypt(key, content.as_bytes()).await?;
    serde_json::to_string(&encrypted).map_err(|e| CryptoError::Encryption(e.to_string()))
}

/// High-level helper: decrypt a document's content.
pub async fn decrypt_document(key: &JsValue, encrypted_json: &str) -> Result<String, CryptoError> {
    let encrypted: EncryptedData =
        serde_json::from_str(encrypted_json).map_err(|e| CryptoError::Decryption(e.to_string()))?;
    let bytes = decrypt(key, &encrypted).await?;
    String::from_utf8(bytes).map_err(|e| CryptoError::Decryption(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypted_key_bundle_serialization() {
        let bundle = EncryptedKeyBundle {
            algorithm: "aes-256-gcm".to_string(),
            fingerprint: "sha256:abc123".to_string(),
            encrypted_key: "dGVzdA==".to_string(),
            salt: "c2FsdA==".to_string(),
            iv: "aXZ2YWx1ZQ==".to_string(),
        };
        let json = serde_json::to_string(&bundle).unwrap();
        assert!(json.contains("aes-256-gcm"));
        let parsed: EncryptedKeyBundle = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.fingerprint, "sha256:abc123");
    }

    #[test]
    fn test_encrypted_data_serialization() {
        let data = EncryptedData {
            ciphertext: "dGVzdA==".to_string(),
            iv: "aXZ2YWx1ZQ==".to_string(),
            algorithm: "aes-256-gcm".to_string(),
        };
        let json = serde_json::to_string(&data).unwrap();
        assert!(json.contains("aes-256-gcm"));
        let parsed: EncryptedData = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.ciphertext, "dGVzdA==");
    }
}
