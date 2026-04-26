use super::*;

#[allow(dead_code)]
impl ApiClient {
    pub async fn list_documents(&self, page: Option<usize>, page_size: Option<usize>) -> Result<DocumentListResponse, ApiError> {
        let mut url = format!("{}/documents?", self.base_url);
        if let Some(p) = page {
            url = format!("{}page={}&", url, p);
        }
        if let Some(ps) = page_size {
            url = format!("{}page_size={}", url, ps);
        }
        let response: DocumentListResponse = self.get(&url).await?;
        Ok(response)
    }

    pub async fn get_document(&self, document_id: &str) -> Result<Document, ApiError> {
        let url = format!("{}/documents/{}", self.base_url, document_id);
        self.get(&url).await
    }

    pub async fn create_document(&self, data: &serde_json::Value) -> Result<Document, ApiError> {
        let url = format!("{}/documents", self.base_url);
        self.post(&url, data).await
    }

    pub async fn update_document(&self, document_id: &str, data: &serde_json::Value) -> Result<Document, ApiError> {
        let url = format!("{}/documents/{}", self.base_url, document_id);
        self.put(&url, data).await
    }

    pub async fn delete_document(&self, document_id: &str) -> Result<(), ApiError> {
        let url = format!("{}/documents/{}", self.base_url, document_id);
        self.delete(&url).await
    }

    pub async fn render_markdown(&self, content: &str) -> Result<RenderMarkdownResponse, ApiError> {
        let url = format!("{}/render/markdown", self.base_url);
        let body = serde_json::json!({ "content": content });
        self.post(&url, &body).await
    }

    pub async fn list_versions(&self, document_id: &str) -> Result<Vec<DocumentVersion>, ApiError> {
        let url = format!("{}/documents/{}/versions", self.base_url, document_id);
        self.get(&url).await
    }

    pub async fn get_version(&self, document_id: &str, version_number: i32) -> Result<DocumentVersion, ApiError> {
        let url = format!("{}/documents/{}/versions/{}", self.base_url, document_id, version_number);
        self.get(&url).await
    }

    pub async fn create_version(&self, document_id: &str, content: &str, commit_message: Option<&str>) -> Result<DocumentVersion, ApiError> {
        let url = format!("{}/documents/{}/versions", self.base_url, document_id);
        let body = serde_json::json!({
            "content": content,
            "commit_message": commit_message
        });
        self.post(&url, &body).await
    }

    pub async fn create_review(&self, document_id: &str, reviewer_id: &str, summary: Option<&str>) -> Result<DocumentReview, ApiError> {
        let url = format!("{}/documents/{}/reviews", self.base_url, document_id);
        let body = serde_json::json!({
            "reviewer_id": reviewer_id,
            "summary": summary,
        });
        self.post(&url, &body).await
    }

    pub async fn list_reviews(&self, document_id: &str) -> Result<Vec<DocumentReview>, ApiError> {
        let url = format!("{}/documents/{}/reviews", self.base_url, document_id);
        self.get(&url).await
    }

    pub async fn get_review_status(&self, document_id: &str) -> Result<ReviewStatusSummary, ApiError> {
        let url = format!("{}/documents/{}/reviews/status", self.base_url, document_id);
        self.get(&url).await
    }

    pub async fn update_review(&self, document_id: &str, review_id: &str, status: &str, summary: Option<&str>) -> Result<DocumentReview, ApiError> {
        let url = format!("{}/documents/{}/reviews/{}", self.base_url, document_id, review_id);
        let body = serde_json::json!({
            "status": status,
            "summary": summary,
        });
        self.put(&url, &body).await
    }

    pub async fn create_review_comment(&self, document_id: &str, review_id: &str, author_id: &str, content: &str) -> Result<ReviewComment, ApiError> {
        let url = format!("{}/documents/{}/reviews/{}/comments", self.base_url, document_id, review_id);
        let body = serde_json::json!({
            "author_id": author_id,
            "content": content,
        });
        self.post(&url, &body).await
    }

    pub async fn list_review_comments(&self, document_id: &str, review_id: &str) -> Result<Vec<ReviewComment>, ApiError> {
        let url = format!("{}/documents/{}/reviews/{}/comments", self.base_url, document_id, review_id);
        self.get(&url).await
    }

    pub async fn diff_versions(&self, document_id: &str, v1: i32, v2: i32) -> Result<DocumentDiffResponse, ApiError> {
        let url = format!("{}/documents/{}/versions/{}/diff/{}", self.base_url, document_id, v1, v2);
        self.get(&url).await
    }

    pub async fn get_conflict_info(&self, document_id: &str) -> Result<ConflictInfo, ApiError> {
        let url = format!("{}/documents/{}/conflict", self.base_url, document_id);
        self.get(&url).await
    }

    pub async fn resolve_conflict(&self, document_id: &str, resolution: &str, content: Option<&str>) -> Result<serde_json::Value, ApiError> {
        let url = format!("{}/documents/{}/conflict/resolve", self.base_url, document_id);
        let body = serde_json::json!({
            "resolution": resolution,
            "content": content,
        });
        self.post(&url, &body).await
    }

    pub async fn list_documents_by_tag(&self, tag: &str, page: Option<i64>, page_size: Option<i64>) -> Result<crate::types::SearchResultsResponse, ApiError> {
        let filters = crate::types::SearchFilters {
            tags: Some(vec![tag.to_string()]),
            ..Default::default()
        };
        self.search("", Some(&filters), page, page_size).await
    }

    pub async fn get_backlinks(&self, document_id: &str) -> Result<crate::types::BacklinksResponse, ApiError> {
        let url = format!("{}/documents/{}/backlinks", self.base_url, document_id);
        self.get(&url).await
    }
}
