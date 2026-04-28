use super::*;

/// Search API methods.
///
/// Reserved for future use: full-text search with filters and pagination.
impl ApiClient {
    pub async fn search(
        &self,
        query: &str,
        filters: Option<&SearchFilters>,
        page: Option<i64>,
        page_size: Option<i64>,
    ) -> Result<SearchResultsResponse, ApiError> {
        let mut url = format!("{}search?q={}", self.base_url, crate::types::url_encode(query));

        if let Some(f) = filters {
            if let Some(ref ct) = f.content_type {
                url = format!("{}&content_type={}", url, ct);
            }
            if let Some(ref s) = f.status {
                url = format!("{}&status={}", url, s);
            }
            if let Some(ref v) = f.visibility {
                url = format!("{}&visibility={}", url, v);
            }
            if let Some(ref pid) = f.project_id {
                url = format!("{}&project_id={}", url, pid);
            }
            if let Some(ref aid) = f.author_id {
                url = format!("{}&author_id={}", url, aid);
            }
            if let Some(ref tags) = f.tags {
                url = format!("{}&tags={}", url, tags.join(","));
            }
            if let Some(ref df) = f.date_from {
                url = format!("{}&date_from={}", url, df);
            }
            if let Some(ref dt) = f.date_to {
                url = format!("{}&date_to={}", url, dt);
            }
        }

        if let Some(p) = page {
            url = format!("{}&page={}", url, p);
        }
        if let Some(ps) = page_size {
            url = format!("{}&page_size={}", url, ps);
        }

        self.get(&url).await
    }

    pub async fn search_suggest(&self, query: &str, limit: Option<u32>) -> Result<Vec<String>, ApiError> {
        let limit = limit.unwrap_or(10);
        let url = format!(
            "{}/search/suggest?q={}&limit={}",
            self.base_url,
            crate::types::url_encode(query),
            limit
        );
        self.get(&url).await
    }

    pub async fn global_search(
        &self,
        query: &str,
        filters: Option<&SearchFilters>,
        page: Option<i64>,
        page_size: Option<i64>,
    ) -> Result<GlobalSearchResponse, ApiError> {
        let mut url = format!("{}search/global?q={}", self.base_url, crate::types::url_encode(query));

        if let Some(f) = filters {
            if let Some(ref ct) = f.content_type {
                url = format!("{}&content_type={}", url, ct);
            }
            if let Some(ref s) = f.status {
                url = format!("{}&status={}", url, s);
            }
            if let Some(ref v) = f.visibility {
                url = format!("{}&visibility={}", url, v);
            }
            if let Some(ref pid) = f.project_id {
                url = format!("{}&project_id={}", url, pid);
            }
            if let Some(ref aid) = f.author_id {
                url = format!("{}&author_id={}", url, aid);
            }
            if let Some(ref tags) = f.tags {
                url = format!("{}&tags={}", url, tags.join(","));
            }
            if let Some(ref df) = f.date_from {
                url = format!("{}&date_from={}", url, df);
            }
            if let Some(ref dt) = f.date_to {
                url = format!("{}&date_to={}", url, dt);
            }
        }

        if let Some(p) = page {
            url = format!("{}&page={}", url, p);
        }
        if let Some(ps) = page_size {
            url = format!("{}&page_size={}", url, ps);
        }

        self.get(&url).await
    }

    pub async fn create_saved_search(&self, request: &CreateSavedSearchRequest) -> Result<SavedSearch, ApiError> {
        let url = format!("{}/search/saved", self.base_url);
        self.post(&url, request).await
    }

    pub async fn list_saved_searches(&self) -> Result<Vec<SavedSearch>, ApiError> {
        let url = format!("{}/search/saved", self.base_url);
        self.get(&url).await
    }

    #[allow(dead_code)]
    pub async fn get_saved_search(&self, id: &str) -> Result<SavedSearch, ApiError> {
        let url = format!("{}/search/saved/{}", self.base_url, id);
        self.get(&url).await
    }

    #[allow(dead_code)]
    pub async fn update_saved_search(&self, id: &str, request: &UpdateSavedSearchRequest) -> Result<SavedSearch, ApiError> {
        let url = format!("{}/search/saved/{}", self.base_url, id);
        self.put(&url, request).await
    }

    #[allow(dead_code)]
    pub async fn delete_saved_search(&self, id: &str) -> Result<(), ApiError> {
        let url = format!("{}/search/saved/{}", self.base_url, id);
        self.delete(&url).await
    }

    pub async fn list_tags(&self) -> Result<crate::types::TagsResponse, ApiError> {
        let url = format!("{}/tags", self.base_url);
        self.get(&url).await
    }
}
