#![allow(dead_code)]

use crate::api::ApiClient;
use super::{BrowserStore, LocalDocument, StoredDocument, SyncState, SyncStatus};
use leptos::prelude::*;
use std::sync::{Arc, Mutex};
use wasm_bindgen::JsCast;

pub struct SyncEngine {
    api: ApiClient,
    store: BrowserStore,
    sync_state: leptos::prelude::RwSignal<SyncState>,
    online: Arc<Mutex<bool>>,
}

impl Clone for SyncEngine {
    fn clone(&self) -> Self {
        Self {
            api: self.api.clone(),
            store: self.store.clone(),
            sync_state: self.sync_state,
            online: Arc::clone(&self.online),
        }
    }
}

impl SyncEngine {
    pub fn new(api: ApiClient, store: BrowserStore) -> Self {
        let is_online = web_sys::window()
            .map(|w| w.navigator().on_line())
            .unwrap_or(false);

        let initial_state = if is_online {
            SyncState::Idle
        } else {
            SyncState::Offline
        };

        let sync_state = leptos::prelude::RwSignal::new(initial_state);
        let online = Arc::new(Mutex::new(is_online));

        let engine = Self {
            api,
            store,
            sync_state,
            online,
        };

        engine.setup_connectivity_listeners();
        engine.start_periodic_sync();

        if is_online {
            engine.trigger_sync();
        }

        engine
    }

    pub fn get_sync_state(&self) -> leptos::prelude::RwSignal<SyncState> {
        self.sync_state
    }

    pub fn trigger_sync(&self) {
        if !self.is_online() {
            self.sync_state.set(SyncState::Offline);
            return;
        }

        if self.sync_state.get() == SyncState::Syncing {
            return;
        }

        self.sync_state.set(SyncState::Syncing);
        let api = self.api.clone();
        let store = self.store.clone();
        let ss = self.sync_state;

        wasm_bindgen_futures::spawn_local(async move {
            match Self::do_sync(&api, &store).await {
                Ok(()) => ss.set(SyncState::Idle),
                Err(e) => ss.set(SyncState::Error(e)),
            }
        });
    }

    fn is_online(&self) -> bool {
        self.online.lock().map(|o| *o).unwrap_or(false)
    }

    fn setup_connectivity_listeners(&self) {
        let sync_state = self.sync_state;
        let online_arc = Arc::clone(&self.online);
        let api = self.api.clone();
        let store = self.store.clone();

        if let Some(window) = web_sys::window() {
            {
                let closure = wasm_bindgen::closure::Closure::<dyn Fn()>::new(move || {
                    if let Ok(mut online) = online_arc.lock() {
                        *online = true;
                    }
                    sync_state.set(SyncState::Idle);
                    let api = api.clone();
                    let store = store.clone();
                    let ss = sync_state;
                    wasm_bindgen_futures::spawn_local(async move {
                        ss.set(SyncState::Syncing);
                        match Self::do_sync(&api, &store).await {
                            Ok(()) => ss.set(SyncState::Idle),
                            Err(e) => ss.set(SyncState::Error(e)),
                        }
                    });
                });
                let _ = window.add_event_listener_with_callback(
                    "online",
                    closure.as_ref().unchecked_ref(),
                );
                closure.forget();
            }

            {
                let online_arc = Arc::clone(&self.online);
                let sync_state = self.sync_state;
                let closure = wasm_bindgen::closure::Closure::<dyn Fn()>::new(move || {
                    if let Ok(mut online) = online_arc.lock() {
                        *online = false;
                    }
                    sync_state.set(SyncState::Offline);
                });
                let _ = window.add_event_listener_with_callback(
                    "offline",
                    closure.as_ref().unchecked_ref(),
                );
                closure.forget();
            }
        }
    }

    fn start_periodic_sync(&self) {
        let api = self.api.clone();
        let store = self.store.clone();
        let sync_state = self.sync_state;
        let online_arc = Arc::clone(&self.online);

        let closure = wasm_bindgen::closure::Closure::<dyn Fn()>::new(move || {
            let is_online = online_arc.lock().map(|o| *o).unwrap_or(false);
            if !is_online {
                return;
            }

            if sync_state.get() == SyncState::Syncing {
                return;
            }

            sync_state.set(SyncState::Syncing);
            let api = api.clone();
            let store = store.clone();
            let ss = sync_state;

            wasm_bindgen_futures::spawn_local(async move {
                match Self::do_sync(&api, &store).await {
                    Ok(()) => ss.set(SyncState::Idle),
                    Err(e) => ss.set(SyncState::Error(e)),
                }
            });
        });

        if let Some(window) = web_sys::window() {
            let _ = window.set_interval_with_callback_and_timeout_and_arguments_0(
                closure.as_ref().unchecked_ref(),
                30_000,
            );
        }

        closure.forget();
    }

    async fn do_sync(api: &ApiClient, store: &BrowserStore) -> Result<(), String> {
        Self::push_pending(api, store).await?;
        Self::pull_from_server(api, store).await?;
        Ok(())
    }

    async fn push_pending(api: &ApiClient, store: &BrowserStore) -> Result<(), String> {
        let pending = store.get_pending();

        for sd in &pending {
            match &sd.sync_status {
                SyncStatus::PendingCreate => {
                    let body = serde_json::json!({
                        "title": sd.document.title,
                        "content": sd.document.content,
                        "tags": sd.document.tags,
                    });
                    match api.create_document(&body).await {
                        Ok(server_doc) => {
                            let mut updated = sd.clone();
                            updated.document = LocalDocument::from(server_doc);
                            updated.sync_status = SyncStatus::Synced;
                            updated.server_version = Some(updated.local_version);
                            updated.last_modified = chrono::Utc::now().to_rfc3339();
                            store.put(updated);
                        }
                        Err(_) => {
                            // Will retry on next sync cycle
                        }
                    }
                }
                SyncStatus::PendingUpdate => {
                    let body = serde_json::json!({
                        "title": sd.document.title,
                        "content": sd.document.content,
                        "tags": sd.document.tags,
                    });
                    match api.update_document(&sd.document.id, &body).await {
                        Ok(server_doc) => {
                            let mut updated = sd.clone();
                            updated.document = LocalDocument::from(server_doc);
                            updated.sync_status = SyncStatus::Synced;
                            updated.server_version = Some(updated.local_version);
                            updated.last_modified = chrono::Utc::now().to_rfc3339();
                            store.put(updated);
                        }
                        Err(e) => {
                            if e.to_string().contains("404") {
                                let mut updated = sd.clone();
                                updated.sync_status = SyncStatus::Conflict;
                                store.put(updated);
                            }
                        }
                    }
                }
                SyncStatus::PendingDelete => {
                    match api.delete_document(&sd.document.id).await {
                        Ok(()) => {
                            store.delete(&sd.document.id);
                        }
                        Err(e) => {
                            if e.to_string().contains("404") {
                                store.delete(&sd.document.id);
                            }
                        }
                    }
                }
                SyncStatus::Conflict | SyncStatus::Synced => {}
            }
        }

        Ok(())
    }

    async fn pull_from_server(api: &ApiClient, store: &BrowserStore) -> Result<(), String> {
        let mut page = 1usize;
        let page_size = 100usize;
        let mut all_docs = Vec::new();

        loop {
            match api.list_documents(Some(page), Some(page_size)).await {
                Ok(response) => {
                    if response.results.is_empty() {
                        break;
                    }
                    all_docs.extend(response.results);
                    if all_docs.len() >= response.total {
                        break;
                    }
                    page += 1;
                }
                Err(e) => {
                    return Err(format!("Failed to fetch documents: {}", e));
                }
            }
        }

        for server_doc in all_docs {
            let id = server_doc.id.clone();
            let server_updated = server_doc.updated_at.clone();

            if let Some(local) = store.get(&id) {
                match local.sync_status {
                    SyncStatus::Synced => {
                        let stored = StoredDocument {
                            document: LocalDocument::from(server_doc),
                            sync_status: SyncStatus::Synced,
                            local_version: local.local_version.saturating_add(1),
                            server_version: Some(
                                local.server_version.unwrap_or(0).saturating_add(1),
                            ),
                            last_modified: chrono::Utc::now().to_rfc3339(),
                        };
                        store.put(stored);
                    }
                    SyncStatus::PendingUpdate | SyncStatus::PendingCreate => {
                        if server_updated != local.document.updated_at {
                            let mut updated = local.clone();
                            updated.sync_status = SyncStatus::Conflict;
                            store.put(updated);
                        }
                    }
                    SyncStatus::PendingDelete => {
                        let mut updated = local.clone();
                        updated.sync_status = SyncStatus::Conflict;
                        store.put(updated);
                    }
                    SyncStatus::Conflict => {}
                }
            } else {
                let stored = StoredDocument {
                    document: LocalDocument::from(server_doc),
                    sync_status: SyncStatus::Synced,
                    local_version: 1,
                    server_version: Some(1),
                    last_modified: chrono::Utc::now().to_rfc3339(),
                };
                store.put(stored);
            }
        }

        Ok(())
    }
}
