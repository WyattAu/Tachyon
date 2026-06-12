use crate::api::ApiClient;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PushSubscriptionInfo {
    pub endpoint: String,
    pub p256dh_key: String,
    pub auth_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationPreferences {
    pub enabled: bool,
    pub mention: bool,
    pub comment: bool,
    pub review: bool,
    pub assignment: bool,
}

impl Default for NotificationPreferences {
    fn default() -> Self {
        Self {
            enabled: true,
            mention: true,
            comment: true,
            review: true,
            assignment: true,
        }
    }
}

const STORAGE_KEY_PUSH_SUBSCRIPTION: &str = "tachyon_push_subscription";
const STORAGE_KEY_NOTIFICATION_PREFS: &str = "tachyon_notification_prefs";

pub fn get_notification_preferences() -> NotificationPreferences {
    if let Some(window) = web_sys::window() {
        if let Ok(Some(storage)) = window.local_storage() {
            if let Ok(Some(json)) = storage.get_item(STORAGE_KEY_NOTIFICATION_PREFS) {
                if let Ok(prefs) = serde_json::from_str::<NotificationPreferences>(&json) {
                    return prefs;
                }
            }
        }
    }
    NotificationPreferences::default()
}

pub fn save_notification_preferences(prefs: &NotificationPreferences) {
    if let Some(window) = web_sys::window() {
        if let Ok(Some(storage)) = window.local_storage() {
            if let Ok(json) = serde_json::to_string(prefs) {
                let _ = storage.set_item(STORAGE_KEY_NOTIFICATION_PREFS, &json);
            }
        }
    }
}

fn get_stored_subscription() -> Option<PushSubscriptionInfo> {
    if let Some(window) = web_sys::window() {
        if let Ok(Some(storage)) = window.local_storage() {
            if let Ok(Some(json)) = storage.get_item(STORAGE_KEY_PUSH_SUBSCRIPTION) {
                if let Ok(sub) = serde_json::from_str::<PushSubscriptionInfo>(&json) {
                    return Some(sub);
                }
            }
        }
    }
    None
}

fn store_subscription(sub: &PushSubscriptionInfo) {
    if let Some(window) = web_sys::window() {
        if let Ok(Some(storage)) = window.local_storage() {
            if let Ok(json) = serde_json::to_string(sub) {
                let _ = storage.set_item(STORAGE_KEY_PUSH_SUBSCRIPTION, &json);
            }
        }
    }
}

fn clear_stored_subscription() {
    if let Some(window) = web_sys::window() {
        if let Ok(Some(storage)) = window.local_storage() {
            let _ = storage.remove_item(STORAGE_KEY_PUSH_SUBSCRIPTION);
        }
    }
}

fn url_base64_encode(data: &[u8]) -> String {
    use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
    URL_SAFE_NO_PAD.encode(data)
}

fn url_base64_decode(s: &str) -> Result<Vec<u8>, String> {
    use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
    URL_SAFE_NO_PAD.decode(s).map_err(|e| e.to_string())
}

pub async fn request_notification_permission() -> Result<bool, String> {
    let window = web_sys::window().ok_or("No window")?;
    let navigator = window.navigator();
    let permission = js_sys::Reflect::get(&navigator, &JsValue::from_str("permissions"))
        .map_err(|_| "No permissions API")?;

    if permission.is_undefined() || permission.is_null() {
        return Err("Permissions API not available".to_string());
    }

    let get_method: js_sys::Function =
        js_sys::Reflect::get(&permission, &JsValue::from_str("query"))
            .map_err(|_| "No query method")?
            .unchecked_into();

    let options = js_sys::Object::new();
    js_sys::Reflect::set(
        &options,
        &JsValue::from_str("userVisibleOnly"),
        &JsValue::TRUE,
    )
    .map_err(|_| "Failed to set userVisibleOnly")?;

    let result: js_sys::Promise = get_method
        .call1(&permission, &JsValue::from_str("push"))
        .map_err(|e| format!("query call failed: {:?}", e))?
        .unchecked_into();

    let status_obj: JsValue = JsFuture::from(result)
        .await
        .map_err(|e| format!("Permission query failed: {:?}", e))?;

    let state = js_sys::Reflect::get(&status_obj, &JsValue::from_str("state"))
        .map_err(|_| "No state property")?
        .as_string()
        .unwrap_or_default();

    Ok(state == "granted")
}

pub async fn subscribe_to_push() -> Result<PushSubscriptionInfo, String> {
    let window = web_sys::window().ok_or("No window")?;
    let navigator = window.navigator();

    let registration = js_sys::Reflect::get(&navigator, &JsValue::from_str("serviceWorker"))
        .map_err(|_| "No serviceWorker")?;

    if registration.is_undefined() || registration.is_null() {
        return Err("Service Worker not available".to_string());
    }

    let get_reg: js_sys::Function =
        js_sys::Reflect::get(&registration, &JsValue::from_str("getRegistration"))
            .map_err(|_| "No getRegistration method")?
            .unchecked_into();

    let reg_result: js_sys::Promise = get_reg
        .call0(&registration)
        .map_err(|e| format!("getRegistration failed: {:?}", e))?
        .unchecked_into();

    let reg: JsValue = JsFuture::from(reg_result)
        .await
        .map_err(|e| format!("Service worker registration failed: {:?}", e))?;

    if reg.is_undefined() || reg.is_null() {
        return Err("No active service worker registration".to_string());
    }

    let api = ApiClient::default();
    let vapid_response: serde_json::Value = api
        .get(&format!(
            "{}/push/vapid-public-key",
            api.base_url.replace("/api/v1", "")
        ))
        .await
        .map_err(|e| format!("Failed to get VAPID key: {}", e))?;

    let vapid_public_key = vapid_response["public_key"]
        .as_str()
        .ok_or("VAPID public key not found in response")?
        .to_string();

    let application_server_key =
        url_base64_decode(&vapid_public_key).map_err(|e| format!("Invalid VAPID key: {}", e))?;

    let push_manager = js_sys::Reflect::get(&reg, &JsValue::from_str("pushManager"))
        .map_err(|_| "No pushManager")?;

    if push_manager.is_undefined() || push_manager.is_null() {
        return Err("Push manager not available".to_string());
    }

    let subscribe_method: js_sys::Function =
        js_sys::Reflect::get(&push_manager, &JsValue::from_str("subscribe"))
            .map_err(|_| "No subscribe method")?
            .unchecked_into();

    let options = js_sys::Object::new();
    let user_visible_only = js_sys::Object::new();
    js_sys::Reflect::set(
        &user_visible_only,
        &JsValue::from_str("userVisibleOnly"),
        &JsValue::TRUE,
    )
    .map_err(|_| "Failed to set userVisibleOnly")?;
    js_sys::Reflect::set(
        &user_visible_only,
        &JsValue::from_str("applicationServerKey"),
        &js_sys::Uint8Array::from(&application_server_key[..]).into(),
    )
    .map_err(|_| "Failed to set applicationServerKey")?;

    let subscribe_result: js_sys::Promise = subscribe_method
        .call1(&push_manager, &user_visible_only.into())
        .map_err(|e| format!("subscribe call failed: {:?}", e))?
        .unchecked_into();

    let subscription: JsValue = JsFuture::from(subscribe_result)
        .await
        .map_err(|e| format!("Push subscribe failed: {:?}", e))?;

    let endpoint = js_sys::Reflect::get(&subscription, &JsValue::from_str("endpoint"))
        .map_err(|_| "No endpoint")?
        .as_string()
        .ok_or("Endpoint is not a string")?;

    let keys_obj =
        js_sys::Reflect::get(&subscription, &JsValue::from_str("keys")).map_err(|_| "No keys")?;

    let p256dh = js_sys::Reflect::get(&keys_obj, &JsValue::from_str("p256dh"))
        .map_err(|_| "No p256dh")?
        .as_string()
        .ok_or("p256dh is not a string")?;

    let auth = js_sys::Reflect::get(&keys_obj, &JsValue::from_str("auth"))
        .map_err(|_| "No auth")?
        .as_string()
        .ok_or("auth is not a string")?;

    let sub_info = PushSubscriptionInfo {
        endpoint,
        p256dh_key: p256dh,
        auth_key: auth,
    };

    let api = ApiClient::default();
    let body = serde_json::json!({
        "endpoint": sub_info.endpoint,
        "p256dh_key": sub_info.p256dh_key,
        "auth_key": sub_info.auth_key,
    });

    api.post::<_, serde_json::Value>(&format!("{}/push/subscribe", api.base_url), &body)
        .await
        .map_err(|e| format!("Failed to register subscription: {}", e))?;

    store_subscription(&sub_info);
    Ok(sub_info)
}

pub async fn unsubscribe_from_push() -> Result<(), String> {
    let stored = get_stored_subscription().ok_or("No stored subscription")?;

    let api = ApiClient::default();
    let body = serde_json::json!({
        "endpoint": stored.endpoint,
    });

    api.post::<_, serde_json::Value>(&format!("{}/push/unsubscribe", api.base_url), &body)
        .await
        .map_err(|e| format!("Failed to unregister subscription: {}", e))?;

    let window = web_sys::window().ok_or("No window")?;
    let navigator = window.navigator();

    if let Ok(registration) = js_sys::Reflect::get(&navigator, &JsValue::from_str("serviceWorker"))
    {
        if !registration.is_undefined() && !registration.is_null() {
            if let Ok(get_reg) =
                js_sys::Reflect::get(&registration, &JsValue::from_str("getRegistration"))
            {
                let get_reg: js_sys::Function = get_reg.unchecked_into();
                if let Ok(reg_result) = get_reg.call0(&registration) {
                    let reg_result: js_sys::Promise = reg_result.unchecked_into();
                    if let Ok(reg) = JsFuture::from(reg_result).await {
                        if !reg.is_undefined() && !reg.is_null() {
                            if let Ok(push_mgr) =
                                js_sys::Reflect::get(&reg, &JsValue::from_str("pushManager"))
                            {
                                if !push_mgr.is_undefined() && !push_mgr.is_null() {
                                    if let Ok(get_sub) = js_sys::Reflect::get(
                                        &push_mgr,
                                        &JsValue::from_str("getSubscription"),
                                    ) {
                                        let get_sub: js_sys::Function = get_sub.unchecked_into();
                                        if let Ok(sub_result) = get_sub.call0(&push_mgr) {
                                            let sub_result: js_sys::Promise =
                                                sub_result.unchecked_into();
                                            if let Ok(sub) = JsFuture::from(sub_result).await {
                                                if !sub.is_undefined() && !sub.is_null() {
                                                    if let Ok(unsub) = js_sys::Reflect::get(
                                                        &sub,
                                                        &JsValue::from_str("unsubscribe"),
                                                    ) {
                                                        let unsub: js_sys::Function =
                                                            unsub.unchecked_into();
                                                        let _ = unsub.call0(&sub);
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    clear_stored_subscription();
    Ok(())
}

pub fn is_push_subscribed() -> bool {
    get_stored_subscription().is_some()
}

pub async fn send_test_notification() -> Result<(), String> {
    let api = ApiClient::default();
    let body = serde_json::json!({
        "title": "Test Notification",
        "body": "Push notifications are working!",
        "icon": "/icons/notification.png",
        "tag": "test-notification",
    });

    api.post::<_, serde_json::Value>(&format!("{}/admin/push/broadcast", api.base_url), &body)
        .await
        .map_err(|e| format!("Failed to send test notification: {}", e))?;

    Ok(())
}
