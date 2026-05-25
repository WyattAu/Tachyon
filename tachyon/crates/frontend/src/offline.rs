#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::{IdbDatabase, IdbObjectStore, IdbOpenDbRequest, IdbTransactionMode};

const DB_NAME: &str = "tachyon_offline";
const DB_VERSION: u32 = 1;
const DOC_STORE: &str = "offline_documents";
const CHANGES_STORE: &str = "pending_changes";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OfflineDocument {
    pub id: String,
    pub title: String,
    pub content: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingChange {
    pub id: String,
    pub document_id: String,
    pub operation: String,
    pub payload: String,
    pub created_at: String,
    pub retry_count: u32,
}

fn to_js<T: serde::Serialize>(value: &T) -> Result<wasm_bindgen::JsValue, String> {
    let json = serde_json::to_string(value).map_err(|e| e.to_string())?;
    js_sys::JSON::parse(&json).map_err(|e| format!("JSON parse error: {:?}", e))
}

fn from_js<T: serde::de::DeserializeOwned>(value: &wasm_bindgen::JsValue) -> Result<T, String> {
    let json_obj =
        js_sys::JSON::stringify(value).map_err(|e| format!("JSON stringify failed: {:?}", e))?;
    let json_str = json_obj
        .as_string()
        .ok_or_else(|| "JSON stringify result is not a string".to_string())?;
    serde_json::from_str(&json_str).map_err(|e| e.to_string())
}

async fn idb_await(request: &web_sys::IdbRequest) -> Result<wasm_bindgen::JsValue, String> {
    let req_js = <web_sys::IdbRequest as AsRef<wasm_bindgen::JsValue>>::as_ref(request).clone();

    let promise =
        js_sys::Promise::new(&mut |resolve: js_sys::Function, reject: js_sys::Function| {
            let r1 = req_js.clone();
            let r3 = req_js.clone();

            let on_success = wasm_bindgen::closure::Closure::once(move || {
                let req = r1.unchecked_ref::<web_sys::IdbRequest>();
                match req.result() {
                    Ok(result) => resolve
                        .call1(&wasm_bindgen::JsValue::UNDEFINED, &result)
                        .unwrap(),
                    Err(_) => resolve.call0(&wasm_bindgen::JsValue::UNDEFINED).unwrap(),
                }
            });

            let on_error = wasm_bindgen::closure::Closure::once(move || {
                reject
                    .call1(
                        &wasm_bindgen::JsValue::UNDEFINED,
                        &wasm_bindgen::JsValue::from_str("IDB request error"),
                    )
                    .unwrap();
            });

            let req = r3.unchecked_ref::<web_sys::IdbRequest>();
            req.set_onsuccess(Some(on_success.as_ref().unchecked_ref()));
            req.set_onerror(Some(on_error.as_ref().unchecked_ref()));

            on_success.forget();
            on_error.forget();
        });

    JsFuture::from(promise)
        .await
        .map_err(|e| format!("IDB request failed: {:?}", e))
}

pub struct OfflineStore {
    db: IdbDatabase,
}

impl OfflineStore {
    pub async fn open() -> Result<Self, String> {
        let window = web_sys::window().ok_or("no window")?;
        let idb_factory = window
            .indexed_db()
            .map_err(|e| format!("indexed_db error: {:?}", e))?
            .ok_or("indexed_db not available")?;

        let open_request: IdbOpenDbRequest = idb_factory
            .open_with_u32(DB_NAME, DB_VERSION)
            .map_err(|e| format!("open error: {:?}", e))?;

        {
            let on_upgrade =
                wasm_bindgen::closure::Closure::<dyn Fn(web_sys::IdbVersionChangeEvent)>::new(
                    |event: web_sys::IdbVersionChangeEvent| {
                        let target = event.target().unwrap();
                        let request = target.unchecked_ref::<IdbOpenDbRequest>();
                        if let Ok(result) = request.result() {
                            let db: IdbDatabase = result.unchecked_into();
                            let _ = db.create_object_store(DOC_STORE);
                            let _ = db.create_object_store(CHANGES_STORE);
                        }
                    },
                );

            open_request.set_onupgradeneeded(Some(on_upgrade.as_ref().unchecked_ref()));
            on_upgrade.forget();
        }

        let result = idb_await(open_request.as_ref()).await?;
        let db: IdbDatabase = result.unchecked_into();
        Ok(Self { db })
    }

    fn store(&self, name: &str, mode: IdbTransactionMode) -> Result<IdbObjectStore, String> {
        let tx = self
            .db
            .transaction_with_str_and_mode(name, mode)
            .map_err(|e| format!("transaction error: {:?}", e))?;
        tx.object_store(name)
            .map_err(|e| format!("object_store error: {:?}", e))
    }

    pub async fn save_document(&self, doc: OfflineDocument) -> Result<(), String> {
        let store = self.store(DOC_STORE, IdbTransactionMode::Readwrite)?;
        let js_val = to_js(&doc)?;
        let key = wasm_bindgen::JsValue::from_str(&doc.id);
        let request = store
            .put_with_key(&js_val, &key)
            .map_err(|e| format!("put error: {:?}", e))?;
        idb_await(&request).await?;
        Ok(())
    }

    pub async fn load_document(&self, id: &str) -> Result<Option<OfflineDocument>, String> {
        let store = self.store(DOC_STORE, IdbTransactionMode::Readonly)?;
        let key = wasm_bindgen::JsValue::from_str(id);
        let request = store.get(&key).map_err(|e| format!("get error: {:?}", e))?;
        let result = idb_await(&request).await?;

        if result.is_undefined() || result.is_null() {
            Ok(None)
        } else {
            Ok(Some(from_js(&result)?))
        }
    }

    pub async fn delete_document(&self, id: &str) -> Result<(), String> {
        let store = self.store(DOC_STORE, IdbTransactionMode::Readwrite)?;
        let key = wasm_bindgen::JsValue::from_str(id);
        let request = store
            .delete(&key)
            .map_err(|e| format!("delete error: {:?}", e))?;
        idb_await(&request).await?;
        Ok(())
    }

    pub async fn get_pending_changes(&self) -> Result<Vec<PendingChange>, String> {
        let store = self.store(CHANGES_STORE, IdbTransactionMode::Readonly)?;
        let request = store
            .get_all()
            .map_err(|e| format!("get_all error: {:?}", e))?;
        let result = idb_await(&request).await?;
        let array = js_sys::Array::from(&result);
        let mut changes = Vec::with_capacity(array.length() as usize);
        for i in 0..array.length() {
            let change: PendingChange = from_js(&array.get(i))?;
            changes.push(change);
        }
        Ok(changes)
    }

    pub async fn enqueue_change(&self, change: PendingChange) -> Result<(), String> {
        let store = self.store(CHANGES_STORE, IdbTransactionMode::Readwrite)?;
        let js_val = to_js(&change)?;
        let key = wasm_bindgen::JsValue::from_str(&change.id);
        let request = store
            .put_with_key(&js_val, &key)
            .map_err(|e| format!("put error: {:?}", e))?;
        idb_await(&request).await?;
        Ok(())
    }

    pub async fn remove_change(&self, id: &str) -> Result<(), String> {
        let store = self.store(CHANGES_STORE, IdbTransactionMode::Readwrite)?;
        let key = wasm_bindgen::JsValue::from_str(id);
        let request = store
            .delete(&key)
            .map_err(|e| format!("delete error: {:?}", e))?;
        idb_await(&request).await?;
        Ok(())
    }

    pub async fn clear_changes(&self) -> Result<(), String> {
        let store = self.store(CHANGES_STORE, IdbTransactionMode::Readwrite)?;
        let request = store.clear().map_err(|e| format!("clear error: {:?}", e))?;
        idb_await(&request).await?;
        Ok(())
    }

    pub async fn get_pending_changes_for_document(
        &self,
        document_id: &str,
    ) -> Result<Vec<PendingChange>, String> {
        let all = self.get_pending_changes().await?;
        Ok(all
            .into_iter()
            .filter(|c| c.document_id == document_id)
            .collect())
    }
}
