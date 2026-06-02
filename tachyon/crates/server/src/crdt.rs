#![allow(dead_code)]

use dashmap::DashMap;
use std::sync::Arc;
use std::time::Instant;
use yrs::updates::decoder::Decode;
use yrs::updates::encoder::Encode;
use yrs::{Doc, GetString, ReadTxn, StateVector, Text, TextRef, Transact};

pub(crate) struct CrdtDocument {
    doc: Doc,
    text: TextRef,
}

#[derive(Clone)]
pub struct CrdtDocumentManager {
    documents: DashMap<String, Arc<CrdtDocument>>,
    last_accessed: DashMap<String, Instant>,
    dirty: DashMap<String, bool>,
    max_documents: usize,
    pool: Option<sqlx::PgPool>,
}

impl CrdtDocumentManager {
    pub fn new() -> Self {
        Self::with_max_documents(Self::default_max_documents())
    }

    pub fn with_max_documents(max_documents: usize) -> Self {
        Self {
            documents: DashMap::new(),
            last_accessed: DashMap::new(),
            dirty: DashMap::new(),
            max_documents,
            pool: None,
        }
    }

    /// Create a manager with a database pool for persistence.
    pub fn with_pool(pool: sqlx::PgPool) -> Self {
        Self {
            documents: DashMap::new(),
            last_accessed: DashMap::new(),
            dirty: DashMap::new(),
            max_documents: Self::default_max_documents(),
            pool: Some(pool),
        }
    }

    /// Set or replace the database pool.
    pub fn set_pool(&mut self, pool: sqlx::PgPool) {
        self.pool = Some(pool);
    }

    fn default_max_documents() -> usize {
        std::env::var("TACHYON_CRDT_MAX_DOCUMENTS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(1000)
    }

    #[allow(private_interfaces)]
    pub fn get_or_create(&self, document_id: &str) -> Arc<CrdtDocument> {
        self.touch_access(document_id);
        if let Some(entry) = self.documents.get(document_id) {
            return entry.value().clone();
        }
        if self.documents.len() >= self.max_documents {
            self.evict_lru();
        }
        let doc = Doc::new();
        let text = doc.get_or_insert_text("content");
        let arc = Arc::new(CrdtDocument { doc, text });
        self.documents.insert(document_id.to_string(), arc.clone());
        self.touch_access(document_id);
        arc
    }

    /// Get or create a document, loading persisted state from the database if available.
    #[allow(private_interfaces)]
    pub async fn get_or_load(&self, document_id: &str) -> Arc<CrdtDocument> {
        self.touch_access(document_id);
        if let Some(entry) = self.documents.get(document_id) {
            return entry.value().clone();
        }
        if self.documents.len() >= self.max_documents {
            self.evict_lru();
        }

        let doc = Doc::new();
        let text = doc.get_or_insert_text("content");

        // Try to load persisted state from database
        if let Some(pool) = &self.pool
            && let Ok(uuid) = uuid::Uuid::parse_str(document_id)
                && let Ok(Some(row)) = tachyon_database::crdt::load_crdt_state(pool, uuid).await
                    && !row.state.is_empty() {
                        // Apply persisted state to the new document
                        if let Ok(update) = yrs::Update::decode_v1(&row.state) {
                            let mut txn = doc.transact_mut();
                            let _ = txn.apply_update(update);
                            drop(txn);
                            tracing::info!(
                                "Loaded CRDT state for document {} (version {})",
                                document_id,
                                row.version
                            );
                        }
                    }

        let arc = Arc::new(CrdtDocument { doc, text });
        self.documents.insert(document_id.to_string(), arc.clone());
        self.touch_access(document_id);
        arc
    }

    fn touch_access(&self, document_id: &str) {
        self.last_accessed
            .insert(document_id.to_string(), Instant::now());
    }

    fn evict_lru(&self) {
        let lru_key = self
            .last_accessed
            .iter()
            .min_by_key(|entry| *entry.value())
            .map(|entry| entry.key().clone());
        if let Some(key) = lru_key {
            // Try to persist before evicting
            if let Some(pool) = &self.pool
                && let Some(doc_arc) = self.documents.get(&key) {
                    let state = {
                        let txn = doc_arc.doc.transact();
                        let sv = txn.state_vector();
                        txn.encode_state_as_update_v1(&sv)
                    };
                    let pool = pool.clone();
                    let key_clone = key.clone();
                    // Spawn a fire-and-forget save task
                    tokio::spawn(async move {
                        if let Ok(uuid) = uuid::Uuid::parse_str(&key_clone) {
                            let sv_empty = StateVector::default();
                            let _ = tachyon_database::crdt::upsert_crdt_state(
                                &pool,
                                uuid,
                                &sv_empty.encode_v1(),
                                &state,
                            )
                            .await;
                            tracing::debug!(
                                "Persisted CRDT state for evicted document {}",
                                key_clone
                            );
                        }
                    });
                }
            self.documents.remove(&key);
            self.last_accessed.remove(&key);
        }
    }

    /// Flush a specific document's state to the database.
    pub async fn flush_document(&self, document_id: &str) -> Result<(), String> {
        let pool = self.pool.as_ref().ok_or("No database pool configured")?;
        let uuid = uuid::Uuid::parse_str(document_id)
            .map_err(|e| format!("Invalid document ID: {}", e))?;

        let doc_ref = self
            .documents
            .get(document_id)
            .ok_or("Document not in memory")?;

        let (state, state_vector) = {
            let txn = doc_ref.doc.transact();
            let sv = txn.state_vector();
            let state = txn.encode_state_as_update_v1(&sv);
            let state_vector = sv.encode_v1();
            (state, state_vector)
        };

        tachyon_database::crdt::upsert_crdt_state(pool, uuid, &state_vector, &state)
            .await
            .map_err(|e| format!("Failed to flush CRDT state: {}", e))?;

        Ok(())
    }

    /// Flush all in-memory documents to the database.
    pub async fn flush_all(&self) -> Result<usize, String> {
        let pool = self.pool.as_ref().ok_or("No database pool configured")?;

        let mut count = 0usize;
        for entry in self.documents.iter() {
            let document_id = entry.key();
            let doc_ref = entry.value();

            if let Ok(uuid) = uuid::Uuid::parse_str(document_id) {
                let (state, state_vector) = {
                    let txn = doc_ref.doc.transact();
                    let sv = txn.state_vector();
                    let state = txn.encode_state_as_update_v1(&sv);
                    let state_vector = sv.encode_v1();
                    (state, state_vector)
                };

                if tachyon_database::crdt::upsert_crdt_state(pool, uuid, &state_vector, &state)
                    .await
                    .is_ok()
                {
                    count += 1;
                }
            }
        }

        tracing::info!("Flushed {} CRDT documents to database", count);
        Ok(count)
    }

    /// Append a CRDT update to the database log.
    pub async fn log_update(
        &self,
        document_id: &str,
        update: &[u8],
        client_id: Option<uuid::Uuid>,
    ) -> Result<(), String> {
        let pool = self.pool.as_ref().ok_or("No database pool configured")?;
        let uuid = uuid::Uuid::parse_str(document_id)
            .map_err(|e| format!("Invalid document ID: {}", e))?;

        tachyon_database::crdt::append_update(pool, uuid, update, client_id)
            .await
            .map_err(|e| format!("Failed to log CRDT update: {}", e))?;

        Ok(())
    }

    /// Garbage collect old updates for a document.
    pub async fn gc_document_updates(
        &self,
        document_id: &str,
        keep_count: i64,
    ) -> Result<u64, String> {
        let pool = self.pool.as_ref().ok_or("No database pool configured")?;
        let uuid = uuid::Uuid::parse_str(document_id)
            .map_err(|e| format!("Invalid document ID: {}", e))?;

        tachyon_database::crdt::gc_updates(pool, uuid, keep_count)
            .await
            .map_err(|e| format!("Failed to GC CRDT updates: {}", e))
    }

    pub fn document_count(&self) -> usize {
        self.documents.len()
    }

    pub fn clear(&self) {
        self.documents.clear();
        self.last_accessed.clear();
        self.dirty.clear();
    }

    pub fn mark_dirty(&self, document_id: &str) {
        self.dirty.insert(document_id.to_string(), true);
    }

    pub fn is_dirty(&self, document_id: &str) -> bool {
        self.dirty.get(document_id).map(|v| *v).unwrap_or(false)
    }

    /// Flush all dirty documents to the database and reset dirty flags.
    pub async fn flush_dirty(&self) {
        if self.pool.is_none() {
            self.dirty.clear();
            return;
        }

        let dirty_ids: Vec<String> = self
            .dirty
            .iter()
            .filter(|entry| *entry.value())
            .map(|entry| entry.key().clone())
            .collect();

        if dirty_ids.is_empty() {
            return;
        }

        let mut flushed = 0usize;
        for doc_id in &dirty_ids {
            if let Err(e) = self.flush_document(doc_id).await {
                tracing::warn!(
                    document_id = %doc_id,
                    error = %e,
                    "Failed to flush dirty CRDT document"
                );
            } else {
                flushed += 1;
            }
        }

        if flushed > 0 {
            tracing::info!("Flushed {} dirty CRDT documents to database", flushed);
        }

        for doc_id in &dirty_ids {
            if let Some(mut entry) = self.dirty.get_mut(doc_id) {
                *entry = false;
            }
        }
    }

    pub fn apply_update(&self, document_id: &str, update: &[u8]) -> Result<Vec<u8>, String> {
        let doc_ref = self.get_or_create(document_id);
        let update = yrs::Update::decode_v1(update)
            .map_err(|e| format!("Failed to decode update: {}", e))?;
        let mut txn = doc_ref.doc.transact_mut();
        txn.apply_update(update)
            .map_err(|e| format!("Failed to apply update: {}", e))?;
        drop(txn);
        self.mark_dirty(document_id);
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
        drop(txn);
        self.mark_dirty(document_id);
        Ok(())
    }

    fn encode_state(&self, document_id: &str) -> Result<Vec<u8>, String> {
        let doc_ref = self.get_or_create(document_id);
        let txn = doc_ref.doc.transact();
        let sv = txn.state_vector();
        let encoded = txn.encode_state_as_update_v1(&sv);
        Ok(encoded)
    }

    /// Encode only the diff between the client's known state vector and the
    /// current document state. This is significantly smaller than sending the
    /// full state for documents with long edit histories.
    ///
    /// Returns `None` if the client's state vector already covers the entire
    /// document (client is already up-to-date).
    pub fn encode_diff(
        &self,
        document_id: &str,
        client_state_vector: &[u8],
    ) -> Result<Option<Vec<u8>>, String> {
        let doc_ref = self.get_or_create(document_id);
        let client_sv = StateVector::decode_v1(client_state_vector)
            .map_err(|e| format!("Failed to decode client state vector: {}", e))?;
        let txn = doc_ref.doc.transact();
        let server_sv = txn.state_vector();

        // If the client's state vector already covers everything the server has,
        // there's nothing to send. StateVector PartialOrd: client >= server means
        // client has all server clocks.
        if client_sv.partial_cmp(&server_sv) == Some(std::cmp::Ordering::Greater)
            || client_sv == server_sv
        {
            drop(txn);
            return Ok(None);
        }

        let diff = txn.encode_diff_v1(&client_sv);
        drop(txn);

        if diff.is_empty() {
            Ok(None)
        } else {
            Ok(Some(diff))
        }
    }

    /// Get the current state vector for a document, encoded as v1 bytes.
    /// Used by clients to request a diff-based sync.
    pub fn get_state_vector(&self, document_id: &str) -> Result<Vec<u8>, String> {
        let doc_ref = self.get_or_create(document_id);
        let txn = doc_ref.doc.transact();
        let sv = txn.state_vector();
        Ok(sv.encode_v1())
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
    use proptest::prelude::*;

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
        manager
            .set_text("doc-state-content", "some content")
            .unwrap();
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

        assert_eq!(manager.document_count(), 10);
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

        assert_eq!(manager.document_count(), 1);
    }

    #[test]
    fn test_default_trait() {
        let manager = CrdtDocumentManager::default();
        let _ = manager.get_or_create("default-doc");
        assert!(manager.get_text("default-doc").is_ok());
    }

    #[test]
    fn test_eviction_when_limit_exceeded() {
        let manager = CrdtDocumentManager::with_max_documents(3);
        manager.get_or_create("doc-a");
        manager.get_or_create("doc-b");
        manager.get_or_create("doc-c");
        assert_eq!(manager.document_count(), 3);

        manager.get_or_create("doc-d");
        assert_eq!(manager.document_count(), 3);
        assert!(manager.get_text("doc-d").is_ok());
    }

    #[test]
    fn test_recently_accessed_not_evicted() {
        let manager = CrdtDocumentManager::with_max_documents(3);
        manager.get_or_create("doc-a");
        manager.get_or_create("doc-b");
        manager.get_or_create("doc-c");

        manager.get_or_create("doc-b");
        manager.get_or_create("doc-c");

        manager.get_or_create("doc-d");
        assert_eq!(manager.document_count(), 3);
        assert!(manager.get_text("doc-b").is_ok());
        assert!(manager.get_text("doc-c").is_ok());
        assert!(manager.get_text("doc-d").is_ok());
    }

    #[test]
    fn test_clear_removes_all_documents() {
        let manager = CrdtDocumentManager::with_max_documents(5);
        manager.get_or_create("doc-a");
        manager.get_or_create("doc-b");
        assert_eq!(manager.document_count(), 2);

        manager.clear();
        assert_eq!(manager.document_count(), 0);
    }

    proptest! {
        #[test]
        fn prop_concurrent_insert_convergence(
            part_a in ".{0,200}",
            part_b in ".{0,200}",
        ) {
            let doc1 = Doc::new();
            let doc2 = Doc::new();
            let text1 = doc1.get_or_insert_text("content");
            let text2 = doc2.get_or_insert_text("content");

            {
                let mut txn = doc1.transact_mut();
                text1.insert(&mut txn, 0, &part_a);
            }
            {
                let mut txn = doc2.transact_mut();
                text2.insert(&mut txn, 0, &part_b);
            }

            let sv1 = doc1.transact().state_vector();
            let sv2 = doc2.transact().state_vector();

            let update1 = doc1.transact().encode_state_as_update_v1(&sv2);
            let update2 = doc2.transact().encode_state_as_update_v1(&sv1);

            {
                let mut txn = doc1.transact_mut();
                let u = yrs::Update::decode_v1(&update2).unwrap();
                txn.apply_update(u).unwrap();
            }
            {
                let mut txn = doc2.transact_mut();
                let u = yrs::Update::decode_v1(&update1).unwrap();
                txn.apply_update(u).unwrap();
            }

            let final1 = doc1.get_or_insert_text("content");
            let final2 = doc2.get_or_insert_text("content");
            let text_val1 = final1.get_string(&doc1.transact());
            let text_val2 = final2.get_string(&doc2.transact());

            assert_eq!(text_val1, text_val2,
                "CRDT convergence failed after concurrent inserts");
        }

        #[test]
        fn prop_concurrent_delete_convergence(
            initial in "[a-zA-Z0-9]{20,200}",
            delete_prefix_len in 0usize..5usize,
        ) {
            let doc1 = Doc::new();
            let doc2 = Doc::new();
            let text1 = doc1.get_or_insert_text("content");
            let text2 = doc2.get_or_insert_text("content");

            {
                let mut txn = doc1.transact_mut();
                text1.insert(&mut txn, 0, &initial);
            }
            {
                let mut txn = doc2.transact_mut();
                text2.insert(&mut txn, 0, &initial);
            }

            let prefix_del = delete_prefix_len.min(initial.len());
            let suffix_del = (initial.len() - prefix_del).min(3);

            {
                let mut txn = doc1.transact_mut();
                text1.remove_range(&mut txn, 0, prefix_del as u32);
            }
            {
                let mut txn = doc2.transact_mut();
                let start = (initial.len() - suffix_del) as u32;
                text2.remove_range(&mut txn, start, suffix_del as u32);
            }

            let sv1 = doc1.transact().state_vector();
            let sv2 = doc2.transact().state_vector();

            let update1 = doc1.transact().encode_state_as_update_v1(&sv2);
            let update2 = doc2.transact().encode_state_as_update_v1(&sv1);

            {
                let mut txn = doc1.transact_mut();
                let u = yrs::Update::decode_v1(&update2).unwrap();
                txn.apply_update(u).unwrap();
            }
            {
                let mut txn = doc2.transact_mut();
                let u = yrs::Update::decode_v1(&update1).unwrap();
                txn.apply_update(u).unwrap();
            }

            let final1 = doc1.get_or_insert_text("content");
            let final2 = doc2.get_or_insert_text("content");
            let text_val1 = final1.get_string(&doc1.transact());
            let text_val2 = final2.get_string(&doc2.transact());

            assert_eq!(text_val1, text_val2,
                "CRDT convergence failed after concurrent deletes");
        }
    }

    #[test]
    fn test_encode_diff_returns_none_for_up_to_date_client() {
        let manager = CrdtDocumentManager::new();
        manager.set_text("doc-diff", "Hello").unwrap();

        // Get the current state vector (client is up-to-date)
        let sv = manager.get_state_vector("doc-diff").unwrap();
        let result = manager.encode_diff("doc-diff", &sv).unwrap();
        assert!(
            result.is_none(),
            "Should return None when client is up-to-date"
        );
    }

    #[test]
    fn test_encode_diff_returns_data_for_stale_client() {
        let manager = CrdtDocumentManager::new();
        manager.set_text("doc-stale", "Initial").unwrap();

        // Capture state vector before additional edits
        let sv_before = manager.get_state_vector("doc-stale").unwrap();

        // Make additional edits
        manager
            .set_text("doc-stale", "Initial + more content")
            .unwrap();

        // Diff from old state vector should contain data
        let diff = manager.encode_diff("doc-stale", &sv_before).unwrap();
        assert!(diff.is_some(), "Should return Some when client is stale");
        let diff_data = diff.unwrap();
        assert!(!diff_data.is_empty(), "Diff data should not be empty");
    }

    #[test]
    fn test_encode_diff_applied_converges() {
        let manager = CrdtDocumentManager::new();
        manager.set_text("doc-conv", "Version 1").unwrap();

        // Simulate a client snapshot: get full state and state vector
        let sv_client = manager.get_state_vector("doc-conv").unwrap();

        // Client initializes from empty state using diff from empty SV
        let empty_sv = yrs::StateVector::default().encode_v1();
        let initial_state = manager.encode_diff("doc-conv", &empty_sv).unwrap().unwrap();

        let client_doc = Doc::new();
        {
            let mut txn = client_doc.transact_mut();
            let u = yrs::Update::decode_v1(&initial_state).unwrap();
            txn.apply_update(u).unwrap();
        }
        // Verify client has "Version 1"
        let client_text = client_doc.get_or_insert_text("content");
        assert_eq!(
            client_text.get_string(&client_doc.transact()),
            "Version 1",
            "Client should have initial state"
        );

        // Server gets more edits
        manager
            .set_text("doc-conv", "Version 2 with more text")
            .unwrap();

        // Client requests diff from its known state
        let diff = manager
            .encode_diff("doc-conv", &sv_client)
            .unwrap()
            .unwrap();

        // Apply the diff
        let update = yrs::Update::decode_v1(&diff).unwrap();
        {
            let mut txn = client_doc.transact_mut();
            txn.apply_update(update).unwrap();
        }

        // Verify convergence
        let server_text = manager.get_text("doc-conv").unwrap();
        let client_result = client_text.get_string(&client_doc.transact());
        assert_eq!(
            server_text, client_result,
            "Client should converge with server after applying diff\n  server: {:?}\n  client: {:?}",
            server_text, client_result
        );
    }

    #[test]
    fn test_encode_diff_empty_state_vector() {
        let manager = CrdtDocumentManager::new();
        manager.set_text("doc-empty-sv", "Some content").unwrap();

        // Empty state vector (new client, never seen the document)
        let empty_sv = yrs::StateVector::default().encode_v1();
        let diff = manager.encode_diff("doc-empty-sv", &empty_sv).unwrap();
        assert!(diff.is_some());

        // The diff from empty SV should produce a valid update that,
        // when applied to an empty doc, converges with the server
        let diff_data = diff.unwrap();
        assert!(
            !diff_data.is_empty(),
            "Diff from empty SV should not be empty"
        );

        let client_doc = Doc::new();
        {
            let mut txn = client_doc.transact_mut();
            let u = yrs::Update::decode_v1(&diff_data).unwrap();
            txn.apply_update(u).unwrap();
        }
        let client_text = client_doc
            .get_or_insert_text("content")
            .get_string(&client_doc.transact());
        let server_text = manager.get_text("doc-empty-sv").unwrap();
        assert_eq!(
            server_text, client_text,
            "Diff from empty SV should converge to server state"
        );
    }

    #[test]
    fn test_get_state_vector_returns_valid_encoding() {
        let manager = CrdtDocumentManager::new();
        manager.set_text("doc-sv", "Test").unwrap();

        let sv_bytes = manager.get_state_vector("doc-sv").unwrap();
        assert!(!sv_bytes.is_empty());

        // Should be decodable as a state vector
        let sv = yrs::StateVector::decode_v1(&sv_bytes);
        assert!(sv.is_ok());
    }

    #[test]
    fn test_dirty_tracking_on_apply() {
        let mgr = CrdtDocumentManager::new();
        let _doc = mgr.get_or_create("test-doc");
        assert!(!mgr.is_dirty("test-doc"));
        // set_text always succeeds and marks dirty via the internal apply
        mgr.set_text("test-doc", "hello world").unwrap();
        assert!(mgr.is_dirty("test-doc"));
    }

    #[test]
    fn test_flush_clears_dirty_flag() {
        // Without pool, flush should be a no-op but still clear flags
        let mgr = CrdtDocumentManager::new();
        mgr.get_or_create("test-doc");
        mgr.mark_dirty("test-doc");
        assert!(mgr.is_dirty("test-doc"));
        // flush_dirty without pool should still clear dirty flags
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            mgr.flush_dirty().await;
        });
        assert!(!mgr.is_dirty("test-doc"));
    }
}
