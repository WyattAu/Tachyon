use super::*;

/// Canvas API methods
impl ApiClient {
    pub async fn create_canvas(
        &self,
        data: &serde_json::Value,
    ) -> Result<serde_json::Value, ApiError> {
        let url = format!("{}/canvases", self.base_url);
        self.post(&url, data).await
    }

    pub async fn list_canvases(&self) -> Result<serde_json::Value, ApiError> {
        let url = format!("{}/canvases", self.base_url);
        self.get(&url).await
    }

    pub async fn get_canvas(&self, canvas_id: &str) -> Result<serde_json::Value, ApiError> {
        let url = format!("{}/canvases/{}", self.base_url, canvas_id);
        self.get(&url).await
    }

    pub async fn update_canvas(
        &self,
        canvas_id: &str,
        data: &serde_json::Value,
    ) -> Result<serde_json::Value, ApiError> {
        let url = format!("{}/canvases/{}", self.base_url, canvas_id);
        self.put(&url, data).await
    }

    pub async fn delete_canvas(&self, canvas_id: &str) -> Result<(), ApiError> {
        let url = format!("{}/canvases/{}", self.base_url, canvas_id);
        self.delete(&url).await
    }

    pub async fn create_canvas_node(
        &self,
        canvas_id: &str,
        data: &serde_json::Value,
    ) -> Result<serde_json::Value, ApiError> {
        let url = format!("{}/canvases/{}/nodes", self.base_url, canvas_id);
        self.post(&url, data).await
    }

    pub async fn list_canvas_nodes(&self, canvas_id: &str) -> Result<serde_json::Value, ApiError> {
        let url = format!("{}/canvases/{}/nodes", self.base_url, canvas_id);
        self.get(&url).await
    }

    pub async fn update_canvas_node(
        &self,
        node_id: &str,
        data: &serde_json::Value,
    ) -> Result<serde_json::Value, ApiError> {
        let url = format!("{}/canvases/nodes/{}", self.base_url, node_id);
        self.put(&url, data).await
    }

    pub async fn delete_canvas_node(&self, node_id: &str) -> Result<(), ApiError> {
        let url = format!("{}/canvases/nodes/{}", self.base_url, node_id);
        self.delete(&url).await
    }

    pub async fn create_canvas_edge(
        &self,
        canvas_id: &str,
        data: &serde_json::Value,
    ) -> Result<serde_json::Value, ApiError> {
        let url = format!("{}/canvases/{}/edges", self.base_url, canvas_id);
        self.post(&url, data).await
    }

    pub async fn list_canvas_edges(&self, canvas_id: &str) -> Result<serde_json::Value, ApiError> {
        let url = format!("{}/canvases/{}/edges", self.base_url, canvas_id);
        self.get(&url).await
    }

    pub async fn update_canvas_edge(
        &self,
        edge_id: &str,
        data: &serde_json::Value,
    ) -> Result<serde_json::Value, ApiError> {
        let url = format!("{}/canvases/edges/{}", self.base_url, edge_id);
        self.put(&url, data).await
    }

    pub async fn delete_canvas_edge(&self, edge_id: &str) -> Result<(), ApiError> {
        let url = format!("{}/canvases/edges/{}", self.base_url, edge_id);
        self.delete(&url).await
    }
}
