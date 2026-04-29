use super::*;

/// Auth API methods.
///
/// Reserved for future use: authentication flows from the frontend.
impl ApiClient {
    pub async fn login(
        &self,
        username: &str,
        password: &str,
    ) -> Result<AuthenticateResponse, ApiError> {
        let url = format!("{}/auth/login", self.base_url);
        let body = LoginRequest {
            username: username.to_string(),
            password: password.to_string(),
        };
        self.post(&url, &body).await
    }

    pub async fn register(
        &self,
        username: &str,
        email: &str,
        password: &str,
    ) -> Result<AuthenticateResponse, ApiError> {
        let url = format!("{}/auth/register", self.base_url);
        let body = serde_json::json!({
            "username": username,
            "display_name": username,
            "email": email,
            "password": password,
        });
        self.post(&url, &body).await
    }

    pub async fn guest_login(&self) -> Result<AuthenticateResponse, ApiError> {
        let url = format!("{}/auth/guest", self.base_url);
        self.post_empty_json(&url).await
    }

    #[allow(dead_code)]
    pub async fn guest_status(&self) -> Result<GuestStatusResponse, ApiError> {
        let url = format!("{}/auth/guest-status", self.base_url);
        self.get(&url).await
    }

    #[allow(dead_code)]
    pub async fn auto_authenticate_guest(&self) -> Result<bool, ApiError> {
        if self.get_auth_token().is_some() {
            return Ok(false);
        }

        let status = self.guest_status().await?;

        if status.public_notes_enabled && status.guest_login_enabled {
            let response = self.guest_login().await?;
            if response.success {
                if let Some(token) = &response.access_token {
                    self.set_auth_token(token.clone());
                    return Ok(true);
                }
            }
        }

        Ok(false)
    }

    #[allow(dead_code)]
    pub async fn auth_status(&self) -> Result<AuthStatusResponse, ApiError> {
        let url = format!("{}/auth/status", self.base_url);
        self.get(&url).await
    }

    pub async fn get_current_user(&self) -> Result<serde_json::Value, ApiError> {
        let url = format!("{}/auth/me", self.base_url);
        self.get(&url).await
    }

    pub async fn update_profile(
        &self,
        display_name: Option<&str>,
        email: Option<&str>,
    ) -> Result<serde_json::Value, ApiError> {
        let mut body = serde_json::Map::new();
        if let Some(name) = display_name {
            body.insert("display_name".to_string(), serde_json::json!(name));
        }
        if let Some(email) = email {
            body.insert("email".to_string(), serde_json::json!(email));
        }
        let url = format!("{}/auth/me", self.base_url);
        self.put(&url, &body).await
    }

    pub async fn change_password(
        &self,
        old_password: &str,
        new_password: &str,
    ) -> Result<(), ApiError> {
        let url = format!("{}/auth/change-password", self.base_url);
        let body = serde_json::json!({
            "old_password": old_password,
            "new_password": new_password,
        });
        self.post_empty_json_accept_any(&url, &body).await
    }

    pub async fn delete_account(&self) -> Result<(), ApiError> {
        let url = format!("{}/auth/me", self.base_url);
        self.delete(&url).await
    }

    #[allow(dead_code)]
    pub async fn logout(&self) -> Result<(), ApiError> {
        let url = format!("{}/auth/logout", self.base_url);
        self.post_empty(&url).await
    }

    pub async fn enable_mfa(&self) -> Result<serde_json::Value, ApiError> {
        let url = format!("{}/auth/mfa", self.base_url);
        self.post(&url, &serde_json::json!({})).await
    }

    pub async fn disable_mfa(&self) -> Result<(), ApiError> {
        let url = format!("{}/auth/mfa", self.base_url);
        self.delete(&url).await
    }

    pub async fn update_user_settings(
        &self,
        settings: &serde_json::Map<String, serde_json::Value>,
    ) -> Result<serde_json::Value, ApiError> {
        let url = format!("{}/auth/me", self.base_url);
        self.put(&url, settings).await
    }
}
