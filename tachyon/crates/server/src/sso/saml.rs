use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SamlConfig {
    pub entity_id: String,
    pub acs_url: String,
    pub metadata_url: String,
    pub certificate: Option<String>,
    pub private_key: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SamlResponse {
    pub assertion_id: String,
    pub issuer: String,
    pub name_id: String,
    pub name_id_format: String,
    pub session_index: Option<String>,
    pub attributes: Vec<SamlAttribute>,
    pub conditions: Option<SamlConditions>,
    pub authn_instant: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SamlAttribute {
    pub name: String,
    pub friendly_name: Option<String>,
    pub values: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SamlConditions {
    pub not_before: Option<String>,
    pub not_on_or_after: Option<String>,
    pub audience_restrictions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SamlAuthRequest {
    pub sso_url: String,
    pub request_id: String,
    pub relay_state: Option<String>,
}
