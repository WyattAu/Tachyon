#![allow(dead_code)]

use crate::offline::{OfflineStore, PendingChange};
use crate::websocket::{ConnectionState, WebSocketClient};
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug, Clone, PartialEq)]
pub enum SyncQueueBridgeState {
    Idle,
    Draining,
    Offline,
    Error(String),
}

pub struct SyncQueueBridge {
    offline_store: Rc<RefCell<Option<OfflineStore>>>,
    ws_client: WebSocketClient,
    state: Rc<RefCell<SyncQueueBridgeState>>,
}

impl Clone for SyncQueueBridge {
    fn clone(&self) -> Self {
        Self {
            offline_store: Rc::clone(&self.offline_store),
            ws_client: self.ws_client.clone(),
            state: Rc::clone(&self.state),
        }
    }
}

impl SyncQueueBridge {
    pub fn new(ws_client: WebSocketClient) -> Self {
        let bridge = Self {
            offline_store: Rc::new(RefCell::new(None)),
            ws_client,
            state: Rc::new(RefCell::new(SyncQueueBridgeState::Idle)),
        };
        bridge.init_offline_store();
        bridge.setup_state_listener();
        bridge
    }

    fn init_offline_store(&self) {
        let store_cell = Rc::clone(&self.offline_store);
        wasm_bindgen_futures::spawn_local(async move {
            match OfflineStore::open().await {
                Ok(store) => {
                    *store_cell.borrow_mut() = Some(store);
                }
                Err(e) => {
                    web_sys::console::log_1(
                        &format!("[SyncQueueBridge] Failed to open offline store: {}", e).into(),
                    );
                }
            }
        });
    }

    fn setup_state_listener(&self) {
        let state = Rc::clone(&self.state);
        let ws = self.ws_client.clone();
        let offline_store = Rc::clone(&self.offline_store);

        let on_state = Rc::new(move |new_state: ConnectionState| match new_state {
            ConnectionState::Connected => {
                *state.borrow_mut() = SyncQueueBridgeState::Draining;
                let offline_store = Rc::clone(&offline_store);
                let ws = ws.clone();
                wasm_bindgen_futures::spawn_local(async move {
                    Self::drain_queue(&offline_store, &ws).await;
                });
            }
            ConnectionState::Disconnected | ConnectionState::Reconnecting => {
                *state.borrow_mut() = SyncQueueBridgeState::Offline;
            }
            _ => {}
        });

        self.ws_client.on_state_change(on_state);
    }

    pub fn queue_change(&self, document_id: &str, operation: &str, payload: &str) {
        let store_cell = Rc::clone(&self.offline_store);
        let doc_id = document_id.to_string();
        let op = operation.to_string();
        let pay = payload.to_string();

        wasm_bindgen_futures::spawn_local(async move {
            let change = PendingChange {
                id: uuid::Uuid::new_v4().to_string(),
                document_id: doc_id,
                operation: op,
                payload: pay,
                created_at: chrono::Utc::now().to_rfc3339(),
                retry_count: 0,
            };

            if let Some(store) = store_cell.borrow().as_ref() {
                if let Err(e) = store.enqueue_change(change).await {
                    web_sys::console::log_1(
                        &format!("[SyncQueueBridge] Failed to enqueue change: {}", e).into(),
                    );
                }
            }
        });

        if self.ws_client.state() == ConnectionState::Connected {
            let ws = self.ws_client.clone();
            let doc_id = document_id.to_string();
            let pay = payload.to_string();
            let op = operation.to_string();
            wasm_bindgen_futures::spawn_local(async move {
                let edit_data = serde_json::json!({
                    "operation": op,
                    "document_id": doc_id,
                    "data": pay,
                });
                let _ = ws.send_edit(&doc_id, "local", edit_data);
            });
        }
    }

    async fn drain_queue(offline_store: &Rc<RefCell<Option<OfflineStore>>>, ws: &WebSocketClient) {
        let changes = {
            let store_ref = offline_store.borrow();
            let store = match store_ref.as_ref() {
                Some(s) => s,
                None => return,
            };
            match store.get_pending_changes().await {
                Ok(c) => c,
                Err(e) => {
                    web_sys::console::log_1(
                        &format!("[SyncQueueBridge] Failed to get pending changes: {}", e).into(),
                    );
                    return;
                }
            }
        };

        if changes.is_empty() {
            return;
        }

        web_sys::console::log_1(
            &format!(
                "[SyncQueueBridge] Draining {} pending changes",
                changes.len()
            )
            .into(),
        );

        for change in &changes {
            let edit_data = serde_json::json!({
                "operation": change.operation,
                "document_id": change.document_id,
                "data": change.payload,
            });

            match ws.send_edit(&change.document_id, "local", edit_data) {
                Ok(()) => {
                    let store_ref = offline_store.borrow();
                    if let Some(store) = store_ref.as_ref() {
                        if let Err(e) = store.remove_change(&change.id).await {
                            web_sys::console::log_1(
                                &format!("[SyncQueueBridge] Failed to remove synced change: {}", e)
                                    .into(),
                            );
                        }
                    }
                }
                Err(_) => {
                    web_sys::console::log_1(
                        &"[SyncQueueBridge] Failed to send change, will retry"
                            .to_string()
                            .into(),
                    );
                    break;
                }
            }
        }
    }

    pub fn get_state(&self) -> SyncQueueBridgeState {
        self.state.borrow().clone()
    }

    pub async fn pending_count(&self) -> usize {
        match self.offline_store.borrow().as_ref() {
            Some(store) => store
                .get_pending_changes()
                .await
                .map(|c| c.len())
                .unwrap_or(0),
            None => 0,
        }
    }
}
