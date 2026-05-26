//! Plugin permission model for capability-based access control.
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PluginPermission {
    Network,
    FileSystemRead,
    FileSystemWrite,
    Database,
    AiProvider,
    Notifications,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionSet {
    pub allowed: HashSet<PluginPermission>,
    pub denied: HashSet<PluginPermission>,
}

impl PermissionSet {
    pub fn none() -> Self {
        Self {
            allowed: HashSet::new(),
            denied: HashSet::new(),
        }
    }

    pub fn all() -> Self {
        Self {
            allowed: HashSet::from([
                PluginPermission::Network,
                PluginPermission::FileSystemRead,
                PluginPermission::FileSystemWrite,
                PluginPermission::Database,
                PluginPermission::AiProvider,
                PluginPermission::Notifications,
            ]),
            denied: HashSet::new(),
        }
    }

    pub fn is_allowed(&self, permission: &PluginPermission) -> bool {
        self.allowed.contains(permission) && !self.denied.contains(permission)
    }

    pub fn grant(&mut self, permission: PluginPermission) {
        self.denied.remove(&permission);
        self.allowed.insert(permission);
    }

    pub fn deny(&mut self, permission: PluginPermission) {
        self.allowed.remove(&permission);
        self.denied.insert(permission);
    }
}
