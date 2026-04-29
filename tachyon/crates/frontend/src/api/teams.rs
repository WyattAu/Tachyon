use super::*;

/// Teams API methods.
///
/// Reserved for future use: team management interface.
impl ApiClient {
    pub async fn list_teams(&self) -> Result<Vec<serde_json::Value>, ApiError> {
        let url = format!("{}/teams", self.base_url);
        self.get(&url).await
    }

    pub async fn create_team(
        &self,
        request: &serde_json::Value,
    ) -> Result<serde_json::Value, ApiError> {
        let url = format!("{}/teams", self.base_url);
        self.post(&url, request).await
    }

    pub async fn list_team_members(
        &self,
        team_id: &str,
    ) -> Result<Vec<serde_json::Value>, ApiError> {
        let url = format!("{}/teams/{}/members", self.base_url, team_id);
        self.get(&url).await
    }

    #[allow(dead_code)]
    pub async fn invite_team_member(
        &self,
        team_id: &str,
        email: &str,
        role: &str,
    ) -> Result<(), ApiError> {
        let url = format!("{}/teams/{}/members/invite", self.base_url, team_id);
        let body = serde_json::json!({ "email": email, "role": role });
        self.post_empty_json_accept_any(&url, &body).await
    }

    #[allow(dead_code)]
    pub async fn remove_team_member(&self, team_id: &str, user_id: &str) -> Result<(), ApiError> {
        let url = format!("{}/teams/{}/members/{}", self.base_url, team_id, user_id);
        self.delete(&url).await
    }

    #[allow(dead_code)]
    pub async fn update_team(
        &self,
        team_id: &str,
        request: &serde_json::Value,
    ) -> Result<serde_json::Value, ApiError> {
        let url = format!("{}/teams/{}", self.base_url, team_id);
        self.put(&url, request).await
    }

    #[allow(dead_code)]
    pub async fn delete_team(&self, team_id: &str) -> Result<(), ApiError> {
        let url = format!("{}/teams/{}", self.base_url, team_id);
        self.delete(&url).await
    }
}
