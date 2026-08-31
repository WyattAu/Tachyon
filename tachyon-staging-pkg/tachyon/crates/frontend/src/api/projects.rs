use super::*;

/// Projects API methods.
///
/// Reserved for future use: project catalog and management.
impl ApiClient {
    pub async fn get_catalog_stats(&self) -> Result<CatalogStats, ApiError> {
        let url = format!("{}/catalog/stats", self.base_url);
        let response: ApiResponse<CatalogStats> = self.get(&url).await?;
        response
            .data
            .ok_or(ApiError::NotFound("Catalog stats".to_string()))
    }

    pub async fn list_projects(&self) -> Result<Vec<Project>, ApiError> {
        let url = format!("{}/projects", self.base_url);
        let response: ApiResponse<Vec<Project>> = self.get(&url).await?;
        Ok(response.data.unwrap_or_default())
    }

    pub async fn get_project(&self, id: &str) -> Result<Project, ApiError> {
        let url = format!("{}/projects/{}", self.base_url, id);
        let response: ApiResponse<Project> = self.get(&url).await?;
        response
            .data
            .ok_or(ApiError::NotFound(format!("Project {}", id)))
    }

    pub async fn get_project_by_slug(&self, slug: &str) -> Result<Project, ApiError> {
        let url = format!("{}/projects/slug/{}", self.base_url, slug);
        let response: ApiResponse<Project> = self.get(&url).await?;
        response
            .data
            .ok_or(ApiError::NotFound(format!("Project with slug {}", slug)))
    }

    pub async fn create_project(
        &self,
        request: &CreateProjectRequest,
    ) -> Result<Project, ApiError> {
        let url = format!("{}/projects", self.base_url);
        let response: ApiResponse<Project> = self.post(&url, request).await?;
        response
            .data
            .ok_or(ApiError::Api("Failed to create project".into()))
    }

    pub async fn update_project(&self, id: &str, project: &Project) -> Result<Project, ApiError> {
        let url = format!("{}/projects/{}", self.base_url, id);
        let response: ApiResponse<Project> = self.put(&url, project).await?;
        response
            .data
            .ok_or(ApiError::Api("Failed to update project".into()))
    }

    pub async fn delete_project(&self, id: &str) -> Result<(), ApiError> {
        let url = format!("{}/projects/{}", self.base_url, id);
        self.delete(&url).await
    }

    pub async fn list_project_components(
        &self,
        project_id: &str,
    ) -> Result<Vec<Component>, ApiError> {
        let url = format!("{}/projects/{}/components", self.base_url, project_id);
        let response: ApiResponse<Vec<Component>> = self.get(&url).await?;
        Ok(response.data.unwrap_or_default())
    }

    pub async fn list_project_members(
        &self,
        project_id: &str,
    ) -> Result<Vec<ProjectMember>, ApiError> {
        let url = format!("{}/projects/{}/members", self.base_url, project_id);
        let response: ApiResponse<Vec<ProjectMember>> = self.get(&url).await?;
        Ok(response.data.unwrap_or_default())
    }
}
