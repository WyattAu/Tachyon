#![allow(dead_code)]

use std::sync::Arc;
use dashmap::DashMap;
use yrs::{Doc, GetString, ReadTxn, Text, TextRef, Transact};
use yrs::updates::decoder::Decode;

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
