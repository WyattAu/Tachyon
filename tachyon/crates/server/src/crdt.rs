#![allow(dead_code)]

use dashmap::DashMap;
use std::sync::Arc;
use yrs::updates::decoder::Decode;
use yrs::{Doc, GetString, ReadTxn, Text, TextRef, Transact};

pub(crate) struct CrdtDocument {
    doc: Doc,
    text: TextRef,
}

pub struct CrdtDocumentManager {
    documents: DashMap<String, Arc<CrdtDocument>>,
}

impl CrdtDocumentManager {
    pub fn new() -> Self {
        Self {
            documents: DashMap::new(),
        }
    }

    #[allow(private_interfaces)]
    pub fn get_or_create(&self, document_id: &str) -> Arc<CrdtDocument> {
        self.documents
            .entry(document_id.to_string())
            .or_insert_with(|| {
                let doc = Doc::new();
                let text = doc.get_or_insert_text("content");
                Arc::new(CrdtDocument { doc, text })
            })
            .value()
            .clone()
    }

    pub fn apply_update(&self, document_id: &str, update: &[u8]) -> Result<Vec<u8>, String> {
        let doc_ref = self.get_or_create(document_id);
        let update = yrs::Update::decode_v1(update)
            .map_err(|e| format!("Failed to decode update: {}", e))?;
        let mut txn = doc_ref.doc.transact_mut();
        txn.apply_update(update)
            .map_err(|e| format!("Failed to apply update: {}", e))?;
        drop(txn);
        self.encode_state(document_id)
    }

    pub fn get_state(&self, document_id: &str) -> Result<Vec<u8>, String> {
        self.encode_state(document_id)
    }

    pub fn get_text(&self, document_id: &str) -> Result<String, String> {
        let doc_ref = self.get_or_create(document_id);
        let txn = doc_ref.doc.transact();
        Ok(doc_ref.text.get_string(&txn))
    }

    pub fn set_text(&self, document_id: &str, text: &str) -> Result<(), String> {
        let doc_ref = self.get_or_create(document_id);
        let mut txn = doc_ref.doc.transact_mut();
        let len = doc_ref.text.len(&txn);
        if len > 0 {
            doc_ref.text.remove_range(&mut txn, 0, len);
        }
        doc_ref.text.insert(&mut txn, 0, text);
        Ok(())
    }

    fn encode_state(&self, document_id: &str) -> Result<Vec<u8>, String> {
        let doc_ref = self.get_or_create(document_id);
        let txn = doc_ref.doc.transact();
        let sv = txn.state_vector();
        let encoded = txn.encode_state_as_update_v1(&sv);
        Ok(encoded)
    }
}

impl Default for CrdtDocumentManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_update(text: &str) -> Vec<u8> {
        let doc = Doc::new();
        let txt = doc.get_or_insert_text("content");
        {
            let mut txn = doc.transact_mut();
            txt.insert(&mut txn, 0, text);
        }
        let txn = doc.transact();
        let sv = yrs::StateVector::default();
        txn.encode_state_as_update_v1(&sv)
    }

    #[test]
    fn test_get_or_create_creates_new() {
        let manager = CrdtDocumentManager::new();
        let doc_ref = manager.get_or_create("doc-1");
        let txn = doc_ref.doc.transact();
        assert_eq!(doc_ref.text.get_string(&txn), "");
    }

    #[test]
    fn test_get_or_create_returns_same_for_same_id() {
        let manager = CrdtDocumentManager::new();
        let a = manager.get_or_create("doc-same");
        let b = manager.get_or_create("doc-same");
        assert!(Arc::ptr_eq(&a, &b));
    }

    #[test]
    fn test_get_or_create_different_ids() {
        let manager = CrdtDocumentManager::new();
        let a = manager.get_or_create("doc-a");
        let b = manager.get_or_create("doc-b");
        assert!(!Arc::ptr_eq(&a, &b));
    }

    #[test]
    fn test_apply_update_valid() {
        let manager = CrdtDocumentManager::new();
        let update = make_update("Hello CRDT");
        let result = manager.apply_update("doc-apply", &update);
        assert!(result.is_ok());
        let text = manager.get_text("doc-apply").unwrap();
        assert_eq!(text, "Hello CRDT");
    }

    #[test]
    fn test_apply_update_invalid_binary() {
        let manager = CrdtDocumentManager::new();
        let garbage = vec![0xDE, 0xAD, 0xBE, 0xEF];
        let result = manager.apply_update("doc-invalid", &garbage);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Failed to decode"));
    }

    #[test]
    fn test_apply_multiple_updates() {
        let manager = CrdtDocumentManager::new();

        let u1_bytes = make_update("Hello");
        let u2_bytes = make_update(" World");

        let _ = manager.apply_update("doc-multi", &u1_bytes);
        let _ = manager.apply_update("doc-multi", &u2_bytes);

        let text = manager.get_text("doc-multi").unwrap();
        assert!(text.contains("Hello"));
        assert!(text.contains("World"));
        assert_eq!(text.len(), "Hello World".len());
    }

    #[test]
    fn test_get_text_nonexistent() {
        let manager = CrdtDocumentManager::new();
        let text = manager.get_text("no-such-doc");
        assert!(text.is_ok());
        assert_eq!(text.unwrap(), "");
    }

    #[test]
    fn test_get_text_after_set() {
        let manager = CrdtDocumentManager::new();
        manager.set_text("doc-set", "initial content").unwrap();
        let text = manager.get_text("doc-set").unwrap();
        assert_eq!(text, "initial content");
    }

    #[test]
    fn test_set_text_overwrites() {
        let manager = CrdtDocumentManager::new();
        manager.set_text("doc-overwrite", "first").unwrap();
        manager.set_text("doc-overwrite", "second").unwrap();
        let text = manager.get_text("doc-overwrite").unwrap();
        assert_eq!(text, "second");
    }

    #[test]
    fn test_set_text_empty() {
        let manager = CrdtDocumentManager::new();
        manager.set_text("doc-empty", "something").unwrap();
        manager.set_text("doc-empty", "").unwrap();
        let text = manager.get_text("doc-empty").unwrap();
        assert_eq!(text, "");
    }

    #[test]
    fn test_get_state_returns_binary() {
        let manager = CrdtDocumentManager::new();
        let state = manager.get_state("doc-state");
        assert!(state.is_ok());
        assert!(!state.unwrap().is_empty());
    }

    #[test]
    fn test_get_state_nonempty_after_set_text() {
        let manager = CrdtDocumentManager::new();
        manager.set_text("doc-state-content", "some content").unwrap();
        let doc_ref = manager.get_or_create("doc-state-content");
        let txn = doc_ref.doc.transact();
        let sv = yrs::StateVector::default();
        let encoded = txn.encode_state_as_update_v1(&sv);
        assert!(encoded.len() > 2);
    }

    #[test]
    fn test_encode_state_via_apply_update_return() {
        let manager = CrdtDocumentManager::new();
        let update = make_update("sync me");
        let encoded = manager.apply_update("doc-encode", &update).unwrap();
        assert!(!encoded.is_empty());

        let state = manager.get_state("doc-encode").unwrap();
        assert_eq!(encoded, state);
    }

    #[test]
    fn test_concurrent_access_multiple_threads() {
        use std::thread;

        let manager = Arc::new(CrdtDocumentManager::new());
        let mut handles = Vec::new();

        for i in 0..10 {
            let mgr = Arc::clone(&manager);
            let handle = thread::spawn(move || {
                let doc_id = format!("concurrent-{}", i);
                mgr.set_text(&doc_id, &format!("value-{}", i)).unwrap();
                let text = mgr.get_text(&doc_id).unwrap();
                assert_eq!(text, format!("value-{}", i));
            });
            handles.push(handle);
        }

        for h in handles {
            h.join().unwrap();
        }

        assert_eq!(manager.documents.len(), 10);
    }

    #[test]
    fn test_concurrent_same_document() {
        use std::thread;

        let manager = Arc::new(CrdtDocumentManager::new());
        let mut handles = Vec::new();

        for _ in 0..20 {
            let mgr = Arc::clone(&manager);
            let handle = thread::spawn(move || {
                let _doc_ref = mgr.get_or_create("race-doc");
                let text = mgr.get_text("race-doc").unwrap();
                let _ = text;
            });
            handles.push(handle);
        }

        for h in handles {
            h.join().unwrap();
        }

        assert_eq!(manager.documents.len(), 1);
    }

    #[test]
    fn test_default_trait() {
        let manager = CrdtDocumentManager::default();
        let _ = manager.get_or_create("default-doc");
        assert!(manager.get_text("default-doc").is_ok());
    }
}
