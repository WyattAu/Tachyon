use super::*;

/// Push notification API methods.
impl ApiClient {
    pub async fn get_vapid_public_key(&self) -> Result<serde_json::Value, ApiError> {
        let url = format!("{}/push/vapid-public-key", self.base_url);
        self.get(&url).await
    }

    pub async fn subscribe_push(
        &self,
        endpoint: &str,
        p256dh_key: &str,
        auth_key: &str,
    ) -> Result<serde_json::Value, ApiError> {
        let url = format!("{}/push/subscribe", self.base_url);
        let body = serde_json::json!({
            "endpoint": endpoint,
            "p256dh_key": p256dh_key,
            "auth_key": auth_key,
        });
        self.post(&url, &body).await
    }

    pub async fn unsubscribe_push(&self, endpoint: &str) -> Result<serde_json::Value, ApiError> {
        let url = format!("{}/push/unsubscribe", self.base_url);
        let body = serde_json::json!({
            "endpoint": endpoint,
        });
        self.post(&url, &body).await
    }
}
