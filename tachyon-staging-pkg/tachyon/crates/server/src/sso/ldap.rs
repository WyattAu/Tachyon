use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LdapConfig {
    pub server_url: String,
    pub bind_dn: String,
    pub bind_password: String,
    pub base_dn: String,
    pub user_filter: String,
    pub group_filter: Option<String>,
    pub attribute_mapping: LdapAttributeMapping,
    pub use_tls: bool,
    pub sync_interval_secs: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LdapAttributeMapping {
    pub username: String,
    pub email: String,
    pub display_name: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub groups: Option<String>,
}

impl Default for LdapAttributeMapping {
    fn default() -> Self {
        Self {
            username: "sAMAccountName".into(),
            email: "mail".into(),
            display_name: "displayName".into(),
            first_name: Some("givenName".into()),
            last_name: Some("sn".into()),
            groups: Some("memberOf".into()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LdapSyncResult {
    pub users_synced: u64,
    pub users_created: u64,
    pub users_updated: u64,
    pub users_deactivated: u64,
    pub groups_synced: u64,
    pub errors: Vec<String>,
}
