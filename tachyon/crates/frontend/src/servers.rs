//! Server registry — manages multiple Tachyon server connections.
//!
//! Each server has: id, name, base_url, auth_token, last_connected.
//! Stored in localStorage as JSON under key "tachyon_servers".
//! The "active" server ID is stored under "tachyon_active_server".

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use wasm_bindgen::JsValue;
use web_sys::js_sys;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerEntry {
    pub id: String,
    pub name: String,
    /// Base URL, e.g. "http://localhost:8080" (no trailing /api/v1)
    pub base_url: String,
    pub auth_token: Option<String>,
    pub last_connected: Option<String>,
}

impl ServerEntry {
    /// Derive the API base URL for this server.
    pub fn api_url(&self) -> String {
        format!("{}/api/v1", self.base_url.trim_end_matches('/'))
    }

    /// Derive the WebSocket URL for this server.
    pub fn ws_url(&self) -> String {
        self.base_url
            .replace("http://", "ws://")
            .replace("https://", "wss://")
            .trim_end_matches('/')
            .to_string()
            + "/ws"
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ServerRegistry {
    pub servers: HashMap<String, ServerEntry>,
    pub active_server_id: Option<String>,
}

impl ServerRegistry {
    /// Load the server registry from localStorage.
    pub fn load() -> Self {
        let window = match web_sys::window() {
            Some(w) => w,
            None => return Self::default(),
        };
        let storage = match window.local_storage() {
            Ok(Some(s)) => s,
            _ => return Self::default(),
        };

        let servers_json = storage
            .get_item("tachyon_servers")
            .ok()
            .flatten()
            .unwrap_or_else(|| "null".to_string());

        let servers: HashMap<String, ServerEntry> =
            serde_json::from_str(&servers_json).unwrap_or_default();

        let active_id = storage.get_item("tachyon_active_server").ok().flatten();

        Self {
            servers,
            active_server_id: active_id,
        }
    }

    /// Persist the server registry to localStorage.
    pub fn save(&self) {
        if let Some(window) = web_sys::window() {
            if let Ok(Some(storage)) = window.local_storage() {
                let _ = storage.set_item(
                    "tachyon_servers",
                    &serde_json::to_string(&self.servers).unwrap_or_else(|_| "{}".to_string()),
                );
                if let Some(ref id) = self.active_server_id {
                    let _ = storage.set_item("tachyon_active_server", id);
                } else {
                    let _ = storage.remove_item("tachyon_active_server");
                }
            }
        }
    }

    /// Get the currently active server, if any.
    pub fn active_server(&self) -> Option<&ServerEntry> {
        self.active_server_id
            .as_ref()
            .and_then(|id| self.servers.get(id))
    }

    /// Get a mutable reference to the currently active server.
    pub fn active_server_mut(&mut self) -> Option<&mut ServerEntry> {
        self.active_server_id
            .as_ref()
            .and_then(|id| self.servers.get_mut(id))
    }

    /// Set the active server by ID.
    pub fn set_active(&mut self, server_id: &str) {
        if self.servers.contains_key(server_id) {
            self.active_server_id = Some(server_id.to_string());
            // Also set window.tachyonApiUrl for the ApiClient to pick up.
            if let Some(server) = self.servers.get(server_id) {
                if let Some(window) = web_sys::window() {
                    let _ = js_sys::Reflect::set(
                        &window.into(),
                        &JsValue::from_str("tachyonApiUrl"),
                        &JsValue::from_str(&server.api_url()),
                    );
                }
            }
        }
    }

    /// Add a new server to the registry.
    pub fn add_server(&mut self, entry: ServerEntry) {
        let id = entry.id.clone();
        self.servers.insert(id, entry);
    }

    /// Remove a server from the registry.
    pub fn remove_server(&mut self, server_id: &str) {
        self.servers.remove(server_id);
        if self.active_server_id.as_deref() == Some(server_id) {
            self.active_server_id = self.servers.keys().next().cloned();
        }
    }

    /// Get all servers as a vector, sorted by last_connected (most recent first).
    pub fn sorted_servers(&self) -> Vec<ServerEntry> {
        let mut servers: Vec<ServerEntry> = self.servers.values().cloned().collect();
        servers.sort_by(|a, b| {
            let a_time = a.last_connected.as_deref().unwrap_or("");
            let b_time = b.last_connected.as_deref().unwrap_or("");
            b_time.cmp(a_time)
        });
        servers
    }
}
