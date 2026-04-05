// WebSocket client wrapper
// Handles WebSocket connection for real-time collaboration

#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use web_sys::{CloseEvent, Event, MessageEvent, WebSocket as SysWebSocket};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionState {
    Disconnected,
    Connecting,
    Connected,
    Reconnecting,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WsMessage {
    #[serde(rename = "type")]
    pub message_type: String,
    pub document_id: Option<String>,
    pub user_id: Option<String>,
    pub data: Option<serde_json::Value>,
    pub timestamp: Option<String>,
}

impl WsMessage {
    pub fn join(document_id: String, user_id: String, user_name: String) -> Self {
        Self {
            message_type: "join".to_string(),
            document_id: Some(document_id),
            user_id: Some(user_id),
            data: Some(serde_json::json!({ "user_name": user_name })),
            timestamp: None,
        }
    }

    pub fn leave(document_id: String, user_id: String) -> Self {
        Self {
            message_type: "leave".to_string(),
            document_id: Some(document_id),
            user_id: Some(user_id),
            data: None,
            timestamp: None,
        }
    }

    pub fn edit(document_id: String, user_id: String, operation: serde_json::Value) -> Self {
        Self {
            message_type: "edit".to_string(),
            document_id: Some(document_id),
            user_id: Some(user_id),
            data: Some(operation),
            timestamp: None,
        }
    }

    pub fn activity(document_id: String, user_id: String, activity: serde_json::Value) -> Self {
        Self {
            message_type: "activity".to_string(),
            document_id: Some(document_id),
            user_id: Some(user_id),
            data: Some(activity),
            timestamp: None,
        }
    }

    pub fn presence(document_id: String, users: Vec<PresenceUserInfo>) -> Self {
        Self {
            message_type: "presence".to_string(),
            document_id: Some(document_id),
            user_id: None,
            data: Some(serde_json::to_value(users).unwrap_or(serde_json::Value::Null)),
            timestamp: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresenceUserInfo {
    pub user_id: String,
    pub user_name: String,
    pub cursor_position: usize,
    pub selection: Option<SelectionRange>,
    pub color: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectionRange {
    pub start: usize,
    pub end: usize,
}

pub type MessageCallback = Rc<dyn Fn(WsMessage)>;
pub type StateCallback = Rc<dyn Fn(ConnectionState)>;

struct WebSocketInner {
    ws: Option<SysWebSocket>,
    state: ConnectionState,
    on_message: Option<MessageCallback>,
    on_state_change: Option<StateCallback>,
    reconnect_attempts: u32,
    max_reconnect_attempts: u32,
    base_url: String,
}

pub struct WebSocketClient {
    inner: Rc<RefCell<WebSocketInner>>,
}

impl Clone for WebSocketClient {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl WebSocketClient {
    pub fn new(base_url: &str) -> Self {
        let ws_url = Self::get_ws_url(base_url);
        Self {
            inner: Rc::new(RefCell::new(WebSocketInner {
                ws: None,
                state: ConnectionState::Disconnected,
                on_message: None,
                on_state_change: None,
                reconnect_attempts: 0,
                max_reconnect_attempts: 5,
                base_url: ws_url,
            })),
        }
    }

    fn get_ws_url(base_url: &str) -> String {
        if let Some(window) = web_sys::window() {
            let location = window.location();
            let protocol = location.protocol().unwrap_or_default();
            let host = location
                .host()
                .unwrap_or_else(|_| "localhost:8081".to_string());

            let ws_protocol = if protocol == "https:" { "wss" } else { "ws" };

            if base_url.is_empty() {
                format!("{}://{}/ws", ws_protocol, host)
            } else {
                base_url.to_string()
            }
        } else {
            "ws://localhost:8081/ws".to_string()
        }
    }

    pub fn state(&self) -> ConnectionState {
        self.inner.borrow().state
    }

    pub fn on_message(&self, callback: MessageCallback) {
        self.inner.borrow_mut().on_message = Some(callback);
    }

    pub fn on_state_change(&self, callback: StateCallback) {
        self.inner.borrow_mut().on_state_change = Some(callback);
    }

    fn set_state(&self, new_state: ConnectionState) {
        self.inner.borrow_mut().state = new_state;
        if let Some(callback) = self.inner.borrow().on_state_change.clone() {
            callback(new_state);
        }
    }

    pub fn connect(&self) {
        let mut inner = self.inner.borrow_mut();
        if inner.state != ConnectionState::Disconnected {
            return;
        }

        inner.state = ConnectionState::Connecting;
        let base_url = inner.base_url.clone();
        drop(inner);

        web_sys::console::log_1(&format!("Connecting to WebSocket: {}", base_url).into());

        match SysWebSocket::new(&base_url) {
            Ok(ws) => {
                let mut inner = self.inner.borrow_mut();
                inner.ws = Some(ws.clone());
                inner.state = ConnectionState::Connected;
                inner.reconnect_attempts = 0;
                let on_message = inner.on_message.clone();
                drop(inner);

                let _inner_clone = self.inner.clone(); // Reserved for reconnection logic
                let onopen_closure = Closure::<dyn Fn(Event)>::new(move |_| {
                    web_sys::console::log_1(&"WebSocket connected".into());
                });
                ws.set_onopen(Some(onopen_closure.as_ref().unchecked_ref()));
                onopen_closure.forget();

                let inner_clone2 = self.inner.clone();
                let onclose_closure = Closure::<dyn Fn(CloseEvent)>::new(move |_| {
                    web_sys::console::log_1(&"WebSocket closed".into());
                    inner_clone2.borrow_mut().ws = None;
                    inner_clone2.borrow_mut().state = ConnectionState::Disconnected;
                });
                ws.set_onclose(Some(onclose_closure.as_ref().unchecked_ref()));
                onclose_closure.forget();

                let onerror_closure = Closure::<dyn Fn(Event)>::new(move |_| {
                    web_sys::console::log_1(&"WebSocket error".into());
                });
                ws.set_onerror(Some(onerror_closure.as_ref().unchecked_ref()));
                onerror_closure.forget();

                let onmessage_closure =
                    Closure::<dyn Fn(MessageEvent)>::new(move |event: MessageEvent| {
                        if let Some(txt) = event.data().as_string() {
                            if let Ok(msg) = serde_json::from_str::<WsMessage>(&txt) {
                                if let Some(callback) = &on_message {
                                    callback(msg);
                                }
                            }
                        }
                    });
                ws.set_onmessage(Some(onmessage_closure.as_ref().unchecked_ref()));
                onmessage_closure.forget();
            }
            Err(_) => {
                self.set_state(ConnectionState::Disconnected);
            }
        }
    }

    pub fn disconnect(&self) {
        let mut inner = self.inner.borrow_mut();
        if let Some(ws) = inner.ws.take() {
            let _ = ws.close();
        }
        inner.state = ConnectionState::Disconnected;
    }

    pub fn send(&self, message: &WsMessage) -> Result<(), String> {
        let inner = self.inner.borrow();
        if inner.state != ConnectionState::Connected {
            return Err("Not connected".to_string());
        }

        let json = serde_json::to_string(message).map_err(|e| e.to_string())?;

        if let Some(ws) = &inner.ws {
            ws.send_with_str(&json).map_err(|e| format!("{:?}", e))?;
            Ok(())
        } else {
            Err("WebSocket not initialized".to_string())
        }
    }

    pub fn join_document(
        &self,
        document_id: &str,
        user_id: &str,
        user_name: &str,
    ) -> Result<(), String> {
        let msg = WsMessage::join(
            document_id.to_string(),
            user_id.to_string(),
            user_name.to_string(),
        );
        self.send(&msg)
    }

    pub fn leave_document(&self, document_id: &str, user_id: &str) -> Result<(), String> {
        let msg = WsMessage::leave(document_id.to_string(), user_id.to_string());
        self.send(&msg)
    }

    pub fn send_edit(
        &self,
        document_id: &str,
        user_id: &str,
        operation: serde_json::Value,
    ) -> Result<(), String> {
        let msg = WsMessage::edit(document_id.to_string(), user_id.to_string(), operation);
        self.send(&msg)
    }

    pub fn send_activity(
        &self,
        document_id: &str,
        user_id: &str,
        activity: serde_json::Value,
    ) -> Result<(), String> {
        let msg = WsMessage::activity(document_id.to_string(), user_id.to_string(), activity);
        self.send(&msg)
    }
}

impl Default for WebSocketClient {
    fn default() -> Self {
        Self::new("")
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditOperation {
    #[serde(rename = "op")]
    pub operation_type: String,
    pub position: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub length: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
}

impl EditOperation {
    pub fn insert(position: usize, text: String) -> Self {
        Self {
            operation_type: "insert".to_string(),
            position,
            length: None,
            text: Some(text),
        }
    }

    pub fn delete(position: usize, length: usize) -> Self {
        Self {
            operation_type: "delete".to_string(),
            position,
            length: Some(length),
            text: None,
        }
    }

    pub fn replace(position: usize, length: usize, text: String) -> Self {
        Self {
            operation_type: "replace".to_string(),
            position,
            length: Some(length),
            text: Some(text),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentEditMessage {
    pub operation_id: String,
    pub operation: EditOperation,
    pub version: u64,
}
