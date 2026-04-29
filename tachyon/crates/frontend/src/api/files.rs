use super::*;

/// Files API methods.
///
/// Reserved for future use: file upload and management.
impl ApiClient {
    #[allow(dead_code)]
    pub async fn upload_file(&self, file: &web_sys::File) -> Result<UploadResponse, ApiError> {
        use gloo_net::http::Request;

        let url = format!("{}/files/upload", self.base_url);
        let form_data = web_sys::FormData::new()
            .map_err(|e| ApiError::Api(format!("Failed to create FormData: {:?}", e)))?;
        form_data
            .append_with_blob("file", file)
            .map_err(|e| ApiError::Api(format!("Failed to append file: {:?}", e)))?;

        let mut builder = Request::post(&url);

        if let Some(token) = self.get_auth_token() {
            builder = builder.header("Authorization", &format!("Bearer {}", token));
        }

        let response = builder
            .body(form_data)
            .map_err(|e| ApiError::Serialization(e.to_string()))?
            .send()
            .await
            .map_err(|e| ApiError::Network(e.to_string()))?;

        if response.ok() {
            response
                .json()
                .await
                .map_err(|e| ApiError::Serialization(e.to_string()))
        } else {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            Err(ApiError::Api(format!("HTTP {}: {}", status, text)))
        }
    }

    pub async fn list_attachments(&self, document_id: &str) -> Result<Vec<Attachment>, ApiError> {
        let url = format!("{}/documents/{}/attachments", self.base_url, document_id);
        self.get(&url).await
    }

    pub async fn upload_attachment(
        &self,
        document_id: &str,
        file: &web_sys::File,
    ) -> Result<Attachment, ApiError> {
        use gloo_net::http::Request;

        let url = format!("{}/documents/{}/attachments", self.base_url, document_id);
        let form_data = web_sys::FormData::new()
            .map_err(|e| ApiError::Api(format!("Failed to create FormData: {:?}", e)))?;
        form_data
            .append_with_blob("file", file)
            .map_err(|e| ApiError::Api(format!("Failed to append file: {:?}", e)))?;

        let mut builder = Request::post(&url);

        if let Some(token) = self.get_auth_token() {
            builder = builder.header("Authorization", &format!("Bearer {}", token));
        }

        let response = builder
            .body(form_data)
            .map_err(|e| ApiError::Serialization(e.to_string()))?
            .send()
            .await
            .map_err(|e| ApiError::Network(e.to_string()))?;

        if response.ok() {
            response
                .json()
                .await
                .map_err(|e| ApiError::Serialization(e.to_string()))
        } else {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            Err(ApiError::Api(format!("HTTP {}: {}", status, text)))
        }
    }

    pub async fn delete_attachment(
        &self,
        document_id: &str,
        attachment_id: &str,
    ) -> Result<(), ApiError> {
        let url = format!(
            "{}/documents/{}/attachments/{}",
            self.base_url, document_id, attachment_id
        );
        self.delete(&url).await
    }
}
