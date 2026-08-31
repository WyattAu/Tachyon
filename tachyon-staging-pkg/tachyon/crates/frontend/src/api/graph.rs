use super::*;

/// Graph API methods.
///
/// Reserved for future use: knowledge graph visualization.
impl ApiClient {
    pub async fn list_graph_nodes(
        &self,
        node_type: Option<&str>,
        search: Option<&str>,
        page: Option<usize>,
        page_size: Option<usize>,
    ) -> Result<serde_json::Value, ApiError> {
        let mut url = format!("{}/nodes?", self.base_url);
        if let Some(nt) = node_type {
            url = format!("{}node_type={}&", url, nt);
        }
        if let Some(s) = search {
            url = format!("{}search={}&", url, s);
        }
        if let Some(p) = page {
            url = format!("{}page={}&", url, p);
        }
        if let Some(ps) = page_size {
            url = format!("{}page_size={}", url, ps);
        }
        self.get(&url).await
    }

    pub async fn get_graph_node(&self, node_id: &str) -> Result<serde_json::Value, ApiError> {
        let url = format!("{}/nodes/{}", self.base_url, node_id);
        self.get(&url).await
    }

    pub async fn create_graph_node(
        &self,
        data: &serde_json::Value,
    ) -> Result<serde_json::Value, ApiError> {
        let url = format!("{}/nodes", self.base_url);
        self.post(&url, data).await
    }

    pub async fn get_node_edges(&self, node_id: &str) -> Result<serde_json::Value, ApiError> {
        let url = format!("{}/nodes/{}/edges", self.base_url, node_id);
        self.get(&url).await
    }

    pub async fn create_graph_edge(
        &self,
        data: &serde_json::Value,
    ) -> Result<serde_json::Value, ApiError> {
        let url = format!("{}/edges", self.base_url);
        self.post(&url, data).await
    }

    pub async fn query_graph(
        &self,
        data: &serde_json::Value,
    ) -> Result<serde_json::Value, ApiError> {
        let url = format!("{}/graph/query", self.base_url);
        self.post(&url, data).await
    }

    pub async fn get_graph_stats(&self) -> Result<serde_json::Value, ApiError> {
        let url = format!("{}/graph/stats", self.base_url);
        self.get(&url).await
    }

    pub async fn get_graph_at_time(&self, at: &str) -> Result<serde_json::Value, ApiError> {
        let url = format!("{}/graph/at?at={}", self.base_url, at);
        self.get(&url).await
    }

    pub async fn get_graph_diff(
        &self,
        from: &str,
        to: &str,
    ) -> Result<serde_json::Value, ApiError> {
        let url = format!("{}/graph/diff?from={}&to={}", self.base_url, from, to);
        self.get(&url).await
    }
}
