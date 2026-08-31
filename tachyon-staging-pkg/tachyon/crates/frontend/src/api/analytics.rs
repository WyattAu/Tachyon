use super::*;

#[derive(Debug, Clone, serde::Deserialize)]
pub struct AnalyticsOverview {
    pub total_documents: i64,
    pub total_users: i64,
    pub storage_bytes: i64,
    pub active_spaces: i64,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct DailyActivity {
    pub date: String,
    pub created: i64,
    pub updated: i64,
    pub deleted: i64,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct DailyActivityResponse {
    pub entries: Vec<DailyActivity>,
    pub total: usize,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct DailyUserActivity {
    pub date: String,
    pub active_users: i64,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct UserActivityResponse {
    pub entries: Vec<DailyUserActivity>,
    pub total: usize,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct DailySearchCount {
    pub date: String,
    pub query_count: i64,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct SearchActivityResponse {
    pub entries: Vec<DailySearchCount>,
    pub total: usize,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct ApiRequestVolume {
    pub date: String,
    pub total_requests: i64,
    pub successful: i64,
    pub failed: i64,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct ApiActivityResponse {
    pub entries: Vec<ApiRequestVolume>,
    pub total: usize,
}

/// Analytics API methods.
impl ApiClient {
    pub async fn get_analytics_overview(&self) -> Result<AnalyticsOverview, ApiError> {
        let url = format!("{}/analytics/overview", self.base_url);
        self.get(&url).await
    }

    pub async fn get_document_activity(&self, days: i32) -> Result<DailyActivityResponse, ApiError> {
        let url = format!("{}/analytics/activity?days={}", self.base_url, days);
        self.get(&url).await
    }

    pub async fn get_user_activity(&self, days: i32) -> Result<UserActivityResponse, ApiError> {
        let url = format!("{}/analytics/users?days={}", self.base_url, days);
        self.get(&url).await
    }

    pub async fn get_search_activity(&self, days: i32) -> Result<SearchActivityResponse, ApiError> {
        let url = format!("{}/analytics/search?days={}", self.base_url, days);
        self.get(&url).await
    }

    pub async fn get_api_activity(&self, days: i32) -> Result<ApiActivityResponse, ApiError> {
        let url = format!("{}/analytics/api?days={}", self.base_url, days);
        self.get(&url).await
    }
}
