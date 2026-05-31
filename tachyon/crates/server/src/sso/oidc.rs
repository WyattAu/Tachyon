use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OidcConfig {
    pub discovery_url: String,
    pub client_id: String,
    pub client_secret: String,
    pub redirect_uri: String,
    pub scope: Vec<String>,
}

impl Default for OidcConfig {
    fn default() -> Self {
        Self {
            discovery_url: String::new(),
            client_id: String::new(),
            client_secret: String::new(),
            redirect_uri: String::new(),
            scope: vec!["openid".into(), "profile".into(), "email".into()],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OidcDiscovery {
    pub issuer: String,
    pub authorization_endpoint: String,
    pub token_endpoint: String,
    pub userinfo_endpoint: String,
    pub jwks_uri: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OidcTokenResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: Option<u64>,
    pub refresh_token: Option<String>,
    pub id_token: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct OidcUserInfo {
    pub sub: String,
    pub email: Option<String>,
    pub name: Option<String>,
    pub given_name: Option<String>,
    pub family_name: Option<String>,
    pub picture: Option<String>,
}
