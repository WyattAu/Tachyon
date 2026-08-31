use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PushSubscription {
    pub endpoint: String,
    pub p256dh_key: String,
    pub auth_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PushPayload {
    pub title: String,
    pub body: String,
    pub icon: Option<String>,
    pub url: Option<String>,
    pub tag: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VapidConfig {
    pub subject: String,
    pub public_key: String,
    pub private_key: String,
}

impl VapidConfig {
    pub fn from_env() -> Option<Self> {
        let public_key = std::env::var("TACHYON_VAPID_PUBLIC_KEY").ok()?;
        let private_key = std::env::var("TACHYON_VAPID_PRIVATE_KEY").ok()?;
        Some(Self {
            subject: std::env::var("TACHYON_VAPID_SUBJECT")
                .unwrap_or_else(|_| "mailto:admin@tachyon.dev".into()),
            public_key,
            private_key,
        })
    }
}
