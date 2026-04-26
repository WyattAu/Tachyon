use super::*;

#[allow(dead_code)]
impl ApiClient {
    pub async fn list_plugins(&self, enabled_only: Option<bool>) -> Result<Vec<Plugin>, ApiError> {
        let mut url = format!("{}/plugins?", self.base_url);
        if let Some(e) = enabled_only {
            url = format!("{}enabled={}", url, e);
        }
        self.get(&url).await
    }

    pub async fn get_plugin(&self, plugin_id: &str) -> Result<Plugin, ApiError> {
        let url = format!("{}/plugins/{}", self.base_url, plugin_id);
        self.get(&url).await
    }

    pub async fn create_plugin(&self, request: &CreatePluginRequest) -> Result<Plugin, ApiError> {
        let url = format!("{}/plugins", self.base_url);
        self.post(&url, request).await
    }

    pub async fn update_plugin(&self, plugin_id: &str, request: &UpdatePluginRequest) -> Result<Plugin, ApiError> {
        let url = format!("{}/plugins/{}", self.base_url, plugin_id);
        self.put(&url, request).await
    }

    pub async fn delete_plugin(&self, plugin_id: &str) -> Result<(), ApiError> {
        let url = format!("{}/plugins/{}", self.base_url, plugin_id);
        self.delete(&url).await
    }
}
