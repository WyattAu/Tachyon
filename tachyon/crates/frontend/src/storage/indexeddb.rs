#![allow(dead_code)]

use super::{StoredDocument, SyncStatus};
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::{IdbDatabase, IdbObjectStore, IdbTransactionMode};

pub struct IndexedDBStore {
    db: IdbDatabase,
}

fn to_js<T: serde::Serialize>(value: &T) -> Result<wasm_bindgen::JsValue, String> {
    let json = serde_json::to_string(value).map_err(|e| e.to_string())?;
    js_sys::JSON::parse(&json).map_err(|e| format!("JSON parse error: {:?}", e))
}

fn from_js<T: serde::de::DeserializeOwned>(value: &wasm_bindgen::JsValue) -> Result<T, String> {
    let json_obj = js_sys::JSON::stringify(value)
        .map_err(|e| format!("JSON stringify failed: {:?}", e))?;
    let json_str = json_obj
        .as_string()
        .ok_or_else(|| "JSON stringify result is not a string".to_string())?;
    serde_json::from_str(&json_str).map_err(|e| e.to_string())
}

async fn idb_await(request: &web_sys::IdbRequest) -> Result<wasm_bindgen::JsValue, String> {
    let req_js =
        <web_sys::IdbRequest as AsRef<wasm_bindgen::JsValue>>::as_ref(request).clone();

    let promise = js_sys::Promise::new(&mut |resolve: js_sys::Function, reject: js_sys::Function| {
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

impl IndexedDBStore {
    pub async fn open() -> Result<Self, String> {
        let window = web_sys::window().ok_or("no window")?;
        let idb_factory = window
            .indexed_db()
            .map_err(|e| format!("indexed_db error: {:?}", e))?
            .ok_or("indexed_db not available")?;

        let open_request = idb_factory
            .open_with_u32("tachyon_store", 1)
            .map_err(|e| format!("open error: {:?}", e))?;

        let on_upgrade = wasm_bindgen::closure::Closure::<
            dyn Fn(web_sys::IdbVersionChangeEvent),
        >::new(|event: web_sys::IdbVersionChangeEvent| {
            let target = event.target().unwrap();
            let request = target.unchecked_ref::<web_sys::IdbOpenDbRequest>();
            if let Ok(result) = request.result() {
                let db: IdbDatabase = result.unchecked_into();
                let _ = db.create_object_store("documents");
            }
        });

        open_request.set_onupgradeneeded(Some(on_upgrade.as_ref().unchecked_ref()));
        on_upgrade.forget();

        let result = idb_await(open_request.as_ref()).await?;
        let db: IdbDatabase = result.unchecked_into();
        Ok(Self { db })
    }

    fn store(&self, mode: IdbTransactionMode) -> Result<IdbObjectStore, String> {
        let tx = self
            .db
            .transaction_with_str_and_mode("documents", mode)
            .map_err(|e| format!("transaction error: {:?}", e))?;
        tx.object_store("documents")
            .map_err(|e| format!("object_store error: {:?}", e))
    }

    pub async fn get_all(&self) -> Result<Vec<StoredDocument>, String> {
        let store = self.store(IdbTransactionMode::Readonly)?;
        let request = store
            .get_all()
            .map_err(|e| format!("get_all error: {:?}", e))?;
        let result = idb_await(&request).await?;
        let array = js_sys::Array::from(&result);
        let mut docs = Vec::with_capacity(array.length() as usize);
        for i in 0..array.length() {
            let doc: StoredDocument = from_js(&array.get(i))?;
            docs.push(doc);
        }
        Ok(docs)
    }

    pub async fn get(&self, id: &str) -> Result<Option<StoredDocument>, String> {
        let store = self.store(IdbTransactionMode::Readonly)?;
        let key = wasm_bindgen::JsValue::from_str(id);
        let request = store.get(&key).map_err(|e| format!("get error: {:?}", e))?;
        let result = idb_await(&request).await?;

        if result.is_undefined() || result.is_null() {
            Ok(None)
        } else {
            Ok(Some(from_js(&result)?))
        }
    }

    pub async fn put(&self, doc: StoredDocument) -> Result<(), String> {
        let store = self.store(IdbTransactionMode::Readwrite)?;
        let js_val = to_js(&doc)?;
        let key = wasm_bindgen::JsValue::from_str(&doc.document.id);
        let request = store
            .put_with_key(&js_val, &key)
            .map_err(|e| format!("put error: {:?}", e))?;
        idb_await(&request).await?;
        Ok(())
    }

    pub async fn delete(&self, id: &str) -> Result<(), String> {
        let store = self.store(IdbTransactionMode::Readwrite)?;
        let key = wasm_bindgen::JsValue::from_str(id);
        let request = store
            .delete(&key)
            .map_err(|e| format!("delete error: {:?}", e))?;
        idb_await(&request).await?;
        Ok(())
    }

    pub async fn clear(&self) -> Result<(), String> {
        let store = self.store(IdbTransactionMode::Readwrite)?;
        let request = store.clear().map_err(|e| format!("clear error: {:?}", e))?;
        idb_await(&request).await?;
        Ok(())
    }

    pub async fn search(&self, query: &str) -> Result<Vec<StoredDocument>, String> {
        let all = self.get_all().await?;
        let query = query.to_lowercase();
        Ok(all
            .into_iter()
            .filter(|sd| {
                sd.document.title.to_lowercase().contains(&query)
                    || sd.document.content.to_lowercase().contains(&query)
            })
            .collect())
    }

    pub async fn get_by_tag(&self, tag: &str) -> Result<Vec<StoredDocument>, String> {
        let all = self.get_all().await?;
        Ok(all
            .into_iter()
            .filter(|sd| sd.document.tags.iter().any(|t| t == tag))
            .collect())
    }

    pub async fn get_pending(&self) -> Result<Vec<StoredDocument>, String> {
        let all = self.get_all().await?;
        Ok(all
            .into_iter()
            .filter(|sd| sd.sync_status != SyncStatus::Synced)
            .collect())
    }
}
