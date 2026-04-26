#![allow(dead_code)]

pub mod indexeddb;
pub mod sync;

use crate::types::Document;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LocalDocument {
    pub id: String,
    pub title: String,
    pub slug: Option<String>,
    pub content: String,
    pub html: Option<String>,
    pub status: String,
    pub visibility: String,
    pub tags: Vec<String>,
    pub author_id: String,
    pub word_count: usize,
    pub character_count: usize,
    pub created_at: String,
    pub updated_at: String,
    pub published_at: Option<String>,
    pub description: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum SyncStatus {
    Synced,
    PendingCreate,
    PendingUpdate,
    PendingDelete,
    Conflict,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StoredDocument {
    pub document: LocalDocument,
    pub sync_status: SyncStatus,
    pub local_version: u64,
    pub server_version: Option<u64>,
    pub last_modified: String,
}

#[derive(Clone, Debug, PartialEq)]
pub enum SyncState {
    Idle,
    Syncing,
    Offline,
    Error(String),
}

const DOCS_KEY: &str = "tachyon_documents";
const SYNC_QUEUE_KEY: &str = "tachyon_sync_queue";
const IDB_READY_KEY: &str = "tachyon_idb_migrated";

/// BrowserStore uses an in-memory HashMap for zero-latency reads.
/// Writes are persisted to IndexedDB (primary) with localStorage as fallback.
/// On startup, loads from IndexedDB if available, otherwise localStorage.
pub struct BrowserStore {
    documents: Arc<Mutex<HashMap<String, StoredDocument>>>,
    /// Tracks whether IndexedDB is available and initialized.
    idb_ready: Arc<Mutex<bool>>,
}

impl Clone for BrowserStore {
    fn clone(&self) -> Self {
        Self {
            documents: Arc::clone(&self.documents),
            idb_ready: Arc::clone(&self.idb_ready),
        }
    }
}

impl Default for BrowserStore {
    fn default() -> Self {
        Self::new()
    }
}

impl BrowserStore {
    pub fn new() -> Self {
        let documents = Self::load_from_localstorage();
        let store = Self {
            documents: Arc::new(Mutex::new(documents)),
            idb_ready: Arc::new(Mutex::new(false)),
        };
        store.init_indexeddb();
        store
    }

    /// Attempt to initialize IndexedDB as the primary backing store.
    /// If IDB has data, it replaces the localStorage data.
    /// If IDB is empty but localStorage has data, migrates localStorage → IDB.
    /// Fires-and-forgets — does not block the caller.
    fn init_indexeddb(&self) {
        let documents = Arc::clone(&self.documents);
        let idb_ready = Arc::clone(&self.idb_ready);

        wasm_bindgen_futures::spawn_local(async move {
            match indexeddb::IndexedDBStore::open().await {
                Ok(idb) => {
                    // Try to load from IDB first
                    match idb.get_all().await {
                        Ok(idb_docs) if !idb_docs.is_empty() => {
                            // IDB has data — use it as source of truth
                            if let Ok(mut docs) = documents.lock() {
                                docs.clear();
                                for doc in idb_docs {
                                    docs.insert(doc.document.id.clone(), doc);
                                }
                            }
                            // Remove localStorage data since IDB is now primary
                            if let Some(window) = web_sys::window() {
                                if let Ok(Some(storage)) = window.local_storage() {
                                    let _ = storage.remove_item(DOCS_KEY);
                                    let _ = storage.set_item(IDB_READY_KEY, "true");
                                }
                            }
                        }
                        Ok(_) => {
                            // IDB is empty — migrate localStorage data to IDB
                            let local_docs: Vec<StoredDocument> = documents
                                .lock()
                                .map(|docs| docs.values().cloned().collect())
                                .unwrap_or_default();
                            for doc in &local_docs {
                                let _ = idb.put(doc.clone()).await;
                            }
                            if let Some(window) = web_sys::window() {
                                if let Ok(Some(storage)) = window.local_storage() {
                                    let _ = storage.set_item(IDB_READY_KEY, "true");
                                }
                            }
                        }
                        Err(e) => {
                            web_sys::console::log_1(
                                &format!("[BrowserStore] IndexedDB init failed, using localStorage: {}", e).into(),
                            );
                        }
                    }
                    // Mark IDB as ready regardless
                    if let Ok(mut ready) = idb_ready.lock() {
                        *ready = true;
                    }
                }
                Err(e) => {
                    web_sys::console::log_1(
                        &format!("[BrowserStore] IndexedDB unavailable, using localStorage: {}", e).into(),
                    );
                }
            }
        });
    }

    fn load_from_localstorage() -> HashMap<String, StoredDocument> {
        if let Some(window) = web_sys::window() {
            if let Ok(Some(storage)) = window.local_storage() {
                if let Ok(Some(json)) = storage.get_item(DOCS_KEY) {
                    if let Ok(map) = serde_json::from_str(&json) {
                        return map;
                    }
                }
            }
        }
        HashMap::new()
    }

    /// Persist to IndexedDB (async, fire-and-forget) and localStorage (sync fallback).
    fn persist(&self) {
        // Sync: persist to localStorage as fallback
        if let Ok(docs) = self.documents.lock() {
            if let Ok(json) = serde_json::to_string(&*docs) {
                if let Some(window) = web_sys::window() {
                    if let Ok(Some(storage)) = window.local_storage() {
                        let _ = storage.set_item(DOCS_KEY, &json);
                    }
                }
            }
        }

        // Async: persist to IndexedDB if ready
        let docs = self.documents.lock()
            .ok()
            .and_then(|docs| {
                serde_json::to_string(&*docs).ok()
            });
        let idb_ready_val = self.idb_ready.lock().map(|r| *r).unwrap_or(false);

        if idb_ready_val {
            if let Some(json) = docs {
                wasm_bindgen_futures::spawn_local(async move {
                    if let Ok(idb) = indexeddb::IndexedDBStore::open().await {
                        if let Ok(map) = serde_json::from_str::<HashMap<String, StoredDocument>>(&json) {
                            for (_, doc) in map {
                                let _ = idb.put(doc).await;
                            }
                        }
                    }
                });
            }
        }

        self.update_sync_queue();
    }

    fn persist_sync_queue(&self, queue: &[String]) {
        if let Ok(json) = serde_json::to_string(queue) {
            if let Some(window) = web_sys::window() {
                if let Ok(Some(storage)) = window.local_storage() {
                    let _ = storage.set_item(SYNC_QUEUE_KEY, &json);
                }
            }
        }
    }

    pub fn get_all(&self) -> Vec<StoredDocument> {
        self.documents
            .lock()
            .map(|docs| docs.values().cloned().collect())
            .unwrap_or_default()
    }

    pub fn get(&self, id: &str) -> Option<StoredDocument> {
        self.documents.lock().ok()?.get(id).cloned()
    }

    pub fn put(&self, doc: StoredDocument) {
        if let Ok(mut docs) = self.documents.lock() {
            docs.insert(doc.document.id.clone(), doc);
            drop(docs);
            self.persist();
        }
    }

    pub fn delete(&self, id: &str) {
        if let Ok(mut docs) = self.documents.lock() {
            if docs.contains_key(id) {
                docs.remove(id);
                drop(docs);
                self.persist();
            }
        }
    }

    pub fn search(&self, query: &str) -> Vec<StoredDocument> {
        let query = query.to_lowercase();
        self.documents
            .lock()
            .map(|docs| {
                docs.values()
                    .filter(|sd| {
                        sd.document.title.to_lowercase().contains(&query)
                            || sd.document.content.to_lowercase().contains(&query)
                    })
                    .cloned()
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn get_by_tag(&self, tag: &str) -> Vec<StoredDocument> {
        self.documents
            .lock()
            .map(|docs| {
                docs.values()
                    .filter(|sd| sd.document.tags.iter().any(|t| t == tag))
                    .cloned()
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn get_pending(&self) -> Vec<StoredDocument> {
        self.documents
            .lock()
            .map(|docs| {
                docs.values()
                    .filter(|sd| sd.sync_status != SyncStatus::Synced)
                    .cloned()
                    .collect()
            })
            .unwrap_or_default()
    }

    fn update_sync_queue(&self) {
        let queue: Vec<String> = self
            .documents
            .lock()
            .map(|docs| {
                docs.values()
                    .filter(|sd| sd.sync_status != SyncStatus::Synced)
                    .map(|sd| sd.document.id.clone())
                    .collect()
            })
            .unwrap_or_default();
        self.persist_sync_queue(&queue);
    }

    pub fn clear(&self) {
        if let Ok(mut docs) = self.documents.lock() {
            docs.clear();
            drop(docs);
        }
        // Clear localStorage
        if let Some(window) = web_sys::window() {
            if let Ok(Some(storage)) = window.local_storage() {
                let _ = storage.remove_item(DOCS_KEY);
                let _ = storage.remove_item(SYNC_QUEUE_KEY);
            }
        }
        // Clear IndexedDB (async)
        let idb_ready_val = self.idb_ready.lock().map(|r| *r).unwrap_or(false);
        if idb_ready_val {
            wasm_bindgen_futures::spawn_local(async move {
                if let Ok(idb) = indexeddb::IndexedDBStore::open().await { let _ = idb.clear().await; }
            });
        }
    }

    pub fn is_idb_ready(&self) -> bool {
        self.idb_ready.lock().map(|r| *r).unwrap_or(false)
    }

    /// Manually trigger migration from localStorage to IndexedDB.
    /// Called after IDB initialization completes.
    pub async fn migrate_to_indexeddb(&self) -> Result<(), String> {
        let idb = indexeddb::IndexedDBStore::open().await?;
        let docs = self.get_all();
        for doc in docs {
            idb.put(doc).await?;
        }
        // Clear localStorage after successful migration
        if let Some(window) = web_sys::window() {
            if let Ok(Some(storage)) = window.local_storage() {
                let _ = storage.remove_item(DOCS_KEY);
                let _ = storage.set_item(IDB_READY_KEY, "true");
            }
        }
        if let Ok(mut ready) = self.idb_ready.lock() {
            *ready = true;
        }
        Ok(())
    }
}

impl From<Document> for LocalDocument {
    fn from(doc: Document) -> Self {
        Self {
            id: doc.id,
            title: doc.title,
            slug: doc.slug,
            content: doc.content,
            html: doc.html,
            status: doc.status,
            visibility: doc.visibility,
            tags: doc.tags,
            author_id: doc.author_id,
            word_count: doc.word_count,
            character_count: doc.character_count,
            created_at: doc.created_at,
            updated_at: doc.updated_at,
            published_at: doc.published_at,
            description: None,
        }
    }
}

pub fn stored_to_document(sd: &StoredDocument) -> Document {
    Document {
        id: sd.document.id.clone(),
        title: sd.document.title.clone(),
        slug: sd.document.slug.clone(),
        html: sd.document.html.clone(),
        content: sd.document.content.clone(),
        status: sd.document.status.clone(),
        visibility: sd.document.visibility.clone(),
        tags: sd.document.tags.clone(),
        author_id: sd.document.author_id.clone(),
        repository_id: None,
        word_count: sd.document.word_count,
        character_count: sd.document.character_count,
        created_at: sd.document.created_at.clone(),
        updated_at: sd.document.updated_at.clone(),
        published_at: sd.document.published_at.clone(),
    }
}

// ============================================================================
// Locale persistence (simple localStorage helpers)
// ============================================================================

pub fn get_locale() -> String {
    let window = match web_sys::window() {
        Some(w) => w,
        None => return "en".to_string(),
    };
    let storage = match window.local_storage() {
        Ok(Some(s)) => s,
        _ => return "en".to_string(),
    };
    match storage.get_item("tachyon_locale") {
        Ok(Some(val)) => val,
        _ => "en".to_string(),
    }
}

pub fn set_locale(locale: &str) {
    if let Some(window) = web_sys::window() {
        if let Ok(Some(storage)) = window.local_storage() {
            let _ = storage.set_item("tachyon_locale", locale);
        }
    }
}
