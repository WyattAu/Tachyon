// WebSocket client wrapper
// Handles WebSocket connection for real-time collaboration with
// automatic heartbeat, exponential backoff reconnection, and message queue.

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

/// Configuration for heartbeat and reconnection behavior.
#[derive(Debug, Clone)]
struct WsConfig {
    /// Interval between heartbeat pings in milliseconds
    heartbeat_interval_ms: u32,
    /// Maximum reconnect attempts before giving up
    max_reconnect_attempts: u32,
    /// Base delay for exponential backoff in milliseconds
    base_reconnect_delay_ms: u32,
    /// Maximum reconnect delay in milliseconds (caps exponential backoff)
    max_reconnect_delay_ms: u32,
}

impl Default for WsConfig {
    fn default() -> Self {
        Self {
            heartbeat_interval_ms: 30_000, // 30 seconds
            max_reconnect_attempts: 10,
            base_reconnect_delay_ms: 1_000, // 1 second
            max_reconnect_delay_ms: 30_000, // 30 seconds
        }
    }
}

struct WebSocketInner {
    ws: Option<SysWebSocket>,
    state: ConnectionState,
    on_message: Option<MessageCallback>,
    on_state_change: Option<StateCallback>,
    reconnect_attempts: u32,
    config: WsConfig,
    base_url: String,
    /// Queue of messages to send once reconnected
    message_queue: Vec<String>,
    /// Handle to the heartbeat interval timer
    heartbeat_handle: Option<i32>,
    /// Handle to the reconnect timeout timer
    reconnect_handle: Option<i32>,
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
                config: WsConfig::default(),
                base_url: ws_url,
                message_queue: Vec::new(),
                heartbeat_handle: None,
                reconnect_handle: None,
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

    /// Start the heartbeat timer that sends periodic pings to keep the connection alive.
    fn start_heartbeat(&self) {
        self.stop_heartbeat();

        let inner = self.inner.borrow();
        let interval_ms = inner.config.heartbeat_interval_ms;
        drop(inner);

        let self_clone = self.clone();

        let closure = Closure::<dyn Fn()>::new(move || {
            let inner = self_clone.inner.borrow();
            if inner.state == ConnectionState::Connected {
                if let Some(ws) = &inner.ws {
                    // Send a lightweight ping JSON message
                    let _ = ws.send_with_str(r#"{"type":"ping","timestamp":null}"#);
                }
            } else {
                // Connection lost during heartbeat — stop pinging
                self_clone.stop_heartbeat();
            }
        });

        let handle = web_sys::window()
            .and_then(|w| {
                w.set_interval_with_callback_and_timeout_and_arguments_0(
                    closure.as_ref().unchecked_ref(),
                    interval_ms as i32,
                )
                .ok()
            })
            .unwrap_or(0);

        closure.forget();

        self.inner.borrow_mut().heartbeat_handle = Some(handle);
    }

    /// Stop the heartbeat timer.
    fn stop_heartbeat(&self) {
        let handle = self.inner.borrow_mut().heartbeat_handle.take();
        if let Some(h) = handle {
            if let Some(window) = web_sys::window() {
                window.clear_interval_with_handle(h);
            }
        }
    }

    /// Schedule a reconnection attempt with exponential backoff.
    fn schedule_reconnect(&self) {
        let inner = self.inner.borrow();
        if inner.reconnect_attempts >= inner.config.max_reconnect_attempts {
            web_sys::console::log_1(
                &format!(
                    "Max reconnect attempts ({}) reached, giving up",
                    inner.config.max_reconnect_attempts
                )
                .into(),
            );
            drop(inner);
            self.set_state(ConnectionState::Disconnected);
            return;
        }

        // Exponential backoff with jitter: delay = base * 2^attempt + random jitter
        let delay_ms = std::cmp::min(
            inner.config.base_reconnect_delay_ms * (1 << inner.reconnect_attempts),
            inner.config.max_reconnect_delay_ms,
        );

        let attempts = inner.reconnect_attempts;
        drop(inner);

        web_sys::console::log_1(
            &format!(
                "Reconnecting in {}ms (attempt {}/{})",
                delay_ms,
                attempts + 1,
                self.inner.borrow().config.max_reconnect_attempts,
            )
            .into(),
        );

        let self_clone = self.clone();

        let closure = Closure::<dyn Fn()>::new(move || {
            self_clone.inner.borrow_mut().reconnect_handle = None;
            self_clone.do_connect();
        });

        let handle = web_sys::window()
            .and_then(|w| {
                w.set_timeout_with_callback_and_timeout_and_arguments_0(
                    closure.as_ref().unchecked_ref(),
                    delay_ms as i32,
                )
                .ok()
            })
            .unwrap_or(0);

        closure.forget();

        self.inner.borrow_mut().reconnect_handle = Some(handle);
    }

    /// Cancel any pending reconnection timer.
    fn cancel_reconnect(&self) {
        let handle = self.inner.borrow_mut().reconnect_handle.take();
        if let Some(h) = handle {
            if let Some(window) = web_sys::window() {
                window.clear_timeout_with_handle(h);
            }
        }
    }

    /// Flush the message queue (send all queued messages after reconnection).
    fn flush_message_queue(&self) {
        let mut inner = self.inner.borrow_mut();
        let queue: Vec<String> = inner.message_queue.drain(..).collect();
        drop(inner);

        for msg_json in queue {
            if let Some(ws) = &self.inner.borrow().ws {
                if ws.send_with_str(&msg_json).is_err() {
                    // Re-queue on failure
                    self.inner.borrow_mut().message_queue.push(msg_json);
                    break;
                }
            }
        }
    }

    pub fn connect(&self) {
        self.cancel_reconnect();
        self.do_connect();
    }

    /// Internal connect implementation — creates the WebSocket and sets up callbacks.
    fn do_connect(&self) {
        let mut inner = self.inner.borrow_mut();

        // Don't connect if already connecting or connected
        if inner.state == ConnectionState::Connecting || inner.state == ConnectionState::Connected {
            return;
        }

        let is_reconnect = inner.reconnect_attempts > 0;
        if is_reconnect {
            inner.state = ConnectionState::Reconnecting;
        } else {
            inner.state = ConnectionState::Connecting;
        }

        let base_url = inner.base_url.clone();
        let on_message = inner.on_message.clone();
        drop(inner);

        let state_label = if is_reconnect {
            "Reconnecting"
        } else {
            "Connecting"
        };
        web_sys::console::log_1(&format!("{} to WebSocket: {}", state_label, base_url).into());

        match SysWebSocket::new(&base_url) {
            Ok(ws) => {
                self.inner.borrow_mut().ws = Some(ws.clone());

                // === onopen: NOW set Connected (not before) ===
                let self_clone = self.clone();
                let onopen_closure = Closure::<dyn Fn(Event)>::new(move |_| {
                    web_sys::console::log_1(&"WebSocket connected".into());
                    self_clone.inner.borrow_mut().reconnect_attempts = 0;
                    self_clone.set_state(ConnectionState::Connected);
                    self_clone.start_heartbeat();
                    self_clone.flush_message_queue();
                });
                ws.set_onopen(Some(onopen_closure.as_ref().unchecked_ref()));
                onopen_closure.forget();

                // === onclose: trigger reconnection ===
                let self_clone = self.clone();
                let onclose_closure =
                    Closure::<dyn Fn(CloseEvent)>::new(move |event: CloseEvent| {
                        web_sys::console::log_1(
                            &format!(
                                "WebSocket closed: code={}, reason={}",
                                event.code(),
                                event.reason(),
                            )
                            .into(),
                        );

                        self_clone.inner.borrow_mut().ws = None;
                        self_clone.stop_heartbeat();
                        self_clone.set_state(ConnectionState::Disconnected);

                        // Only auto-reconnect on abnormal closures or clean closures with reconnect intent
                        // Code 1000 = normal closure (don't reconnect)
                        if event.code() != 1000 {
                            self_clone.inner.borrow_mut().reconnect_attempts += 1;
                            self_clone.schedule_reconnect();
                        }
                    });
                ws.set_onclose(Some(onclose_closure.as_ref().unchecked_ref()));
                onclose_closure.forget();

                // === onerror ===
                let onerror_closure = Closure::<dyn Fn(Event)>::new(move |_| {
                    web_sys::console::log_1(&"WebSocket error".into());
                });
                ws.set_onerror(Some(onerror_closure.as_ref().unchecked_ref()));
                onerror_closure.forget();

                // === onmessage ===
                let onmessage_closure =
                    Closure::<dyn Fn(MessageEvent)>::new(move |event: MessageEvent| {
                        if let Some(txt) = event.data().as_string() {
                            // Ignore pong/ping messages
                            if txt == r#"{"type":"ping"}"# || txt == r#"{"type":"pong"}"# {
                                return;
                            }
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
            Err(e) => {
                web_sys::console::log_1(&format!("WebSocket creation failed: {:?}", e).into());
                self.set_state(ConnectionState::Disconnected);

                // Schedule reconnect even on creation failure
                self.inner.borrow_mut().reconnect_attempts += 1;
                self.schedule_reconnect();
            }
        }
    }

    pub fn disconnect(&self) {
        self.cancel_reconnect();
        self.stop_heartbeat();

        let mut inner = self.inner.borrow_mut();
        inner.reconnect_attempts = inner.config.max_reconnect_attempts; // prevent auto-reconnect
        if let Some(ws) = inner.ws.take() {
            // Close with code 1000 (normal) to signal intentional close
            let _ = ws.close_with_code_and_reason(1000, "Client disconnect");
        }
        inner.state = ConnectionState::Disconnected;
        inner.message_queue.clear();
        drop(inner);

        self.set_state(ConnectionState::Disconnected);
    }

    pub fn send(&self, message: &WsMessage) -> Result<(), String> {
        let mut inner = self.inner.borrow_mut();
        let json = serde_json::to_string(message).map_err(|e| e.to_string())?;

        if inner.state == ConnectionState::Connected {
            if let Some(ws) = &inner.ws {
                ws.send_with_str(&json).map_err(|e| format!("{:?}", e))?;
                return Ok(());
            }
        }

        // Not connected — queue the message for when we reconnect
        inner.message_queue.push(json);
        Err("Not connected — message queued".to_string())
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
