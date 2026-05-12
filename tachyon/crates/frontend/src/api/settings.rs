use super::*;

/// Settings API methods.
///
/// Reserved for future use: admin settings and audit log management.
impl ApiClient {
    pub async fn list_audit_logs(
        &self,
        page: Option<u32>,
        page_size: Option<u32>,
        action: Option<&str>,
        actor_id: Option<&str>,
    ) -> Result<serde_json::Value, ApiError> {
        let mut params = vec![];
        if let Some(p) = page {
            params.push(format!("page={}", p));
        }
        if let Some(ps) = page_size {
            params.push(format!("page_size={}", ps));
        }
        if let Some(a) = action {
            params.push(format!("action={}", a));
        }
        if let Some(aid) = actor_id {
            params.push(format!("actor_id={}", aid));
        }
        let query = if params.is_empty() {
            String::new()
        } else {
            format!("?{}", params.join("&"))
        };
        let url = format!("{}/audit{}", self.base_url, query);
        self.get(&url).await
    }

    pub async fn list_roles(&self) -> Result<Vec<serde_json::Value>, ApiError> {
        let url = format!("{}/roles", self.base_url);
        self.get(&url).await
    }

    pub async fn create_role(
        &self,
        request: &serde_json::Value,
    ) -> Result<serde_json::Value, ApiError> {
        let url = format!("{}/roles", self.base_url);
        self.post(&url, request).await
    }

    pub async fn delete_role(&self, role_id: &str) -> Result<(), ApiError> {
        let url = format!("{}/roles/{}", self.base_url, role_id);
        self.delete(&url).await
    }

    pub async fn list_webhooks(&self) -> Result<Vec<crate::types::WebhookInfo>, ApiError> {
        let url = format!("{}/webhooks", self.base_url);
        self.get(&url).await
    }

    pub async fn create_webhook(
        &self,
        webhook_url: &str,
        events: Vec<&str>,
        secret: Option<&str>,
    ) -> Result<crate::types::WebhookInfo, ApiError> {
        let url = format!("{}/webhooks", self.base_url);
        let body = serde_json::json!({
            "url": webhook_url,
            "events": events,
            "secret": secret,
        });
        self.post(&url, &body).await
    }

    pub async fn delete_webhook(&self, id: &str) -> Result<(), ApiError> {
        let url = format!("{}/webhooks/{}", self.base_url, id);
        self.delete(&url).await
    }
}
