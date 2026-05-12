use super::*;

/// Spaces API methods.
///
/// Reserved for future use: workspace space management.
impl ApiClient {
    /// List spaces, optionally filtered by owner.
    pub async fn list_spaces(
        &self,
        owner_id: Option<&str>,
    ) -> Result<Vec<crate::types::Space>, ApiError> {
        let mut url = format!("{}/spaces", self.base_url);
        if let Some(oid) = owner_id {
            url = format!("{}?owner_id={}", url, oid);
        }
        self.get(&url).await
    }

    pub async fn list_root_spaces(
        &self,
        owner_id: &str,
    ) -> Result<Vec<crate::types::Space>, ApiError> {
        let url = format!("{}/spaces/root?owner_id={}", self.base_url, owner_id);
        self.get(&url).await
    }

    pub async fn list_child_spaces(
        &self,
        parent_id: &str,
        owner_id: &str,
    ) -> Result<Vec<crate::types::Space>, ApiError> {
        let url = format!(
            "{}/spaces/{}/children?owner_id={}",
            self.base_url, parent_id, owner_id
        );
        self.get(&url).await
    }

    /// Fetch a single space by its ID.
    pub async fn get_space(&self, space_id: &str) -> Result<crate::types::Space, ApiError> {
        let url = format!("{}/spaces/{}", self.base_url, space_id);
        self.get(&url).await
    }

    pub async fn get_default_space(&self, owner_id: &str) -> Result<crate::types::Space, ApiError> {
        let url = format!("{}/spaces/default?owner_id={}", self.base_url, owner_id);
        self.get(&url).await
    }

    /// Create a new space with the given configuration.
    pub async fn create_space(
        &self,
        req: &crate::types::CreateSpaceRequest,
    ) -> Result<crate::types::Space, ApiError> {
        let url = format!("{}/spaces", self.base_url);
        self.post(&url, req).await
    }

    /// Update an existing space's metadata.
    pub async fn update_space(
        &self,
        space_id: &str,
        req: &crate::types::UpdateSpaceRequest,
    ) -> Result<crate::types::Space, ApiError> {
        let url = format!("{}/spaces/{}", self.base_url, space_id);
        self.put(&url, req).await
    }

    /// Delete a space by its ID.
    pub async fn delete_space(&self, space_id: &str) -> Result<(), ApiError> {
        let url = format!("{}/spaces/{}", self.base_url, space_id);
        self.delete(&url).await
    }

    /// List all members of a space.
    pub async fn list_space_members(
        &self,
        space_id: &str,
    ) -> Result<Vec<crate::types::SpaceMember>, ApiError> {
        let url = format!("{}/spaces/{}/members", self.base_url, space_id);
        self.get(&url).await
    }

    /// Add a member to a space.
    pub async fn add_space_member(
        &self,
        space_id: &str,
        req: &crate::types::AddSpaceMemberRequest,
    ) -> Result<crate::types::SpaceMember, ApiError> {
        let url = format!("{}/spaces/{}/members", self.base_url, space_id);
        self.post(&url, req).await
    }

    pub async fn update_space_member(
        &self,
        space_id: &str,
        user_id: &str,
        req: &crate::types::UpdateSpaceMemberRequest,
    ) -> Result<crate::types::SpaceMember, ApiError> {
        let url = format!("{}/spaces/{}/members/{}", self.base_url, space_id, user_id);
        self.put(&url, req).await
    }

    /// Remove a member from a space.
    pub async fn remove_space_member(&self, space_id: &str, user_id: &str) -> Result<(), ApiError> {
        let url = format!("{}/spaces/{}/members/{}", self.base_url, space_id, user_id);
        self.delete(&url).await
    }

    pub async fn move_document_to_space(
        &self,
        document_id: &str,
        space_id: Option<&str>,
    ) -> Result<(), ApiError> {
        let url = format!("{}/spaces/move-document/{}", self.base_url, document_id);
        let body = serde_json::json!({ "space_id": space_id });
        self.put(&url, &body).await
    }
}
