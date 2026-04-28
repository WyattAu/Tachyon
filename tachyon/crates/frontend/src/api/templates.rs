use super::*;

/// Templates API methods.
///
/// Reserved for future use: document template management.
impl ApiClient {
    pub async fn list_templates(&self, category: Option<&str>) -> Result<Vec<DocumentTemplate>, ApiError> {
        let mut url = format!("{}/templates?", self.base_url);
        if let Some(cat) = category {
            url = format!("{}category={}", url, cat);
        }
        self.get(&url).await
    }

    #[allow(dead_code)]
    pub async fn get_template(&self, template_id: &str) -> Result<DocumentTemplate, ApiError> {
        let url = format!("{}/templates/{}", self.base_url, template_id);
        self.get(&url).await
    }

    pub async fn create_template(&self, request: &CreateTemplateRequest) -> Result<DocumentTemplate, ApiError> {
        let url = format!("{}/templates", self.base_url);
        self.post(&url, request).await
    }

    pub async fn update_template(&self, template_id: &str, request: &UpdateTemplateRequest) -> Result<DocumentTemplate, ApiError> {
        let url = format!("{}/templates/{}", self.base_url, template_id);
        self.put(&url, request).await
    }

    pub async fn delete_template(&self, template_id: &str) -> Result<(), ApiError> {
        let url = format!("{}/templates/{}", self.base_url, template_id);
        self.delete(&url).await
    }

    pub async fn list_template_categories(&self) -> Result<Vec<String>, ApiError> {
        let url = format!("{}/templates/categories", self.base_url);
        self.get(&url).await
    }
}
