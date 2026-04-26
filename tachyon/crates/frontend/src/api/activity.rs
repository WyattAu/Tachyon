use super::*;

#[allow(dead_code)]
impl ApiClient {
    pub async fn list_activity(&self, limit: Option<u32>, offset: Option<u32>) -> Result<ActivityListResponse, ApiError> {
        let mut params = vec![];
        if let Some(l) = limit { params.push(format!("limit={}", l)); }
        if let Some(o) = offset { params.push(format!("offset={}", o)); }
        let query = if params.is_empty() { String::new() } else { format!("?{}", params.join("&")) };
        let url = format!("{}/activity{}", self.base_url, query);
        self.get(&url).await
    }

    pub async fn list_notifications(&self, limit: Option<u32>, include_read: bool) -> Result<NotificationListResponse, ApiError> {
        let mut params = vec![format!("include_read={}", include_read)];
        if let Some(l) = limit { params.push(format!("limit={}", l)); }
        let url = format!("{}/notifications?{}", self.base_url, params.join("&"));
        self.get(&url).await
    }

    pub async fn get_unread_notification_count(&self) -> Result<u32, ApiError> {
        let url = format!("{}/notifications/unread-count", self.base_url);
        self.get(&url).await
    }

    pub async fn mark_notification_read(&self, notification_id: &str) -> Result<(), ApiError> {
        let url = format!("{}/notifications/{}/read", self.base_url, notification_id);
        self.post_empty(&url).await
    }

    pub async fn mark_all_notifications_read(&self) -> Result<(), ApiError> {
        let url = format!("{}/notifications/read-all", self.base_url);
        self.post_empty(&url).await
    }
}
