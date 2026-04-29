use super::*;

/// SSG API methods.
///
/// Reserved for future use: static site generation from the frontend.
impl ApiClient {
    pub async fn build_site(&self, config: &SsgBuildRequest) -> Result<SsgBuildResponse, ApiError> {
        let url = format!("{}/ssg/build", self.base_url);
        self.post(&url, config).await
    }

    pub async fn download_ssg_build(&self) -> Result<(), ApiError> {
        use gloo_net::http::Request;
        use wasm_bindgen::JsCast;

        let url = format!("{}/ssg/download", self.base_url);
        let mut builder = Request::get(&url);

        if let Some(token) = self.get_auth_token() {
            builder = builder.header("Authorization", &format!("Bearer {}", token));
        }

        let response = builder
            .send()
            .await
            .map_err(|e| ApiError::Network(e.to_string()))?;

        if response.ok() {
            let blob = response
                .binary()
                .await
                .map_err(|e| ApiError::Serialization(e.to_string()))?;

            if let Some(window) = web_sys::window() {
                let js_bytes = js_sys::Uint8Array::new_with_length(blob.len() as u32);
                js_bytes.copy_from(&blob);

                let parts = js_sys::Array::new();
                parts.push(&js_bytes.buffer());

                let bag = web_sys::BlobPropertyBag::new();
                bag.set_type("application/zip");
                let blob = web_sys::Blob::new_with_u8_array_sequence_and_options(&parts, &bag)
                    .map_err(|e| ApiError::Api(format!("Failed to create blob: {:?}", e)))?;

                let url = web_sys::Url::create_object_url_with_blob(&blob)
                    .map_err(|e| ApiError::Api(format!("Failed to create object URL: {:?}", e)))?;

                let document = window.document().unwrap();
                let a = document.create_element("a").unwrap();
                a.set_attribute("href", &url).unwrap();
                a.set_attribute("download", "tachyon-site.zip").unwrap();

                let body = document.body().unwrap();
                body.append_child(&a).unwrap();
                a.dyn_ref::<web_sys::HtmlElement>().unwrap().click();
                body.remove_child(&a).unwrap();
                web_sys::Url::revoke_object_url(&url).unwrap();
            }

            Ok(())
        } else {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            Err(ApiError::Api(format!("HTTP {}: {}", status, text)))
        }
    }
}
