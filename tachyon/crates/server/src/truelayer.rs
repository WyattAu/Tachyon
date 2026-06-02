//! TrueLayer open banking payment client
//!
//! Provides payment mandate and direct debit integration via the TrueLayer API.

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

// ============================================================================
// Error type
// ============================================================================

#[derive(Debug, thiserror::Error)]
pub enum TrueLayerError {
    #[error("API error {status}: {message}")]
    ApiError { status: u16, message: String },
    #[error("Authentication error: {0}")]
    AuthError(String),
    #[error("Configuration error: {0}")]
    ConfigError(String),
    #[error("Request error: {0}")]
    RequestError(#[from] reqwest::Error),
    #[error("TrueLayer is not enabled")]
    Disabled,
}

// ============================================================================
// Response types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateMandateResponse {
    pub id: String,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authorization_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MandateStatus {
    pub id: String,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authorization_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePaymentResponse {
    pub id: String,
    pub status: String,
    pub amount_in_minor: u64,
    pub currency: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reference: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentStatusResponse {
    pub id: String,
    pub status: String,
    pub amount_in_minor: u64,
    pub currency: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub settled_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TokenResponse {
    access_token: String,
    expires_in: u64,
}

#[derive(Debug, Clone)]
struct TokenCache {
    token: String,
    expires_at: Instant,
}

// ============================================================================
// TrueLayer client
// ============================================================================

pub struct TrueLayerClient {
    client: reqwest::Client,
    client_id: String,
    client_secret: String,
    base_url: String,
    merchant_account_id: String,
    webhook_secret: String,
    token_cache: Arc<Mutex<Option<TokenCache>>>,
    enabled: bool,
}

impl Clone for TrueLayerClient {
    fn clone(&self) -> Self {
        Self {
            client: self.client.clone(),
            client_id: self.client_id.clone(),
            client_secret: self.client_secret.clone(),
            base_url: self.base_url.clone(),
            merchant_account_id: self.merchant_account_id.clone(),
            webhook_secret: self.webhook_secret.clone(),
            token_cache: Arc::clone(&self.token_cache),
            enabled: self.enabled,
        }
    }
}

impl TrueLayerClient {
    pub fn new(config: &crate::config::TrueLayerConfig, http_client: reqwest::Client) -> Self {
        let base_url = match config.environment.as_str() {
            "production" => "https://api.truelayer.com",
            _ => "https://api.truelayer-sandbox.com",
        };

        let enabled = config.enabled
            && !config.client_id.is_empty()
            && !config.client_secret.is_empty()
            && !config.merchant_account_id.is_empty();

        Self {
            client: http_client,
            client_id: config.client_id.clone(),
            client_secret: config.client_secret.clone(),
            base_url: base_url.to_string(),
            merchant_account_id: config.merchant_account_id.clone(),
            webhook_secret: config.webhook_secret.clone(),
            token_cache: Arc::new(Mutex::new(None)),
            enabled,
        }
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    async fn get_access_token(&self) -> Result<String, TrueLayerError> {
        if !self.enabled {
            return Err(TrueLayerError::Disabled);
        }

        {
            let cache = self.token_cache.lock().await;
            if let Some(cached) = cache.as_ref()
                && cached.expires_at > Instant::now() {
                    return Ok(cached.token.clone());
                }
        }

        let resp = self
            .client
            .post(format!("{}/auth/token", self.base_url))
            .form(&[
                ("grant_type", "client_credentials"),
                ("client_id", &self.client_id),
                ("client_secret", &self.client_secret),
                ("scope", "payments"),
            ])
            .send()
            .await?;

        let status = resp.status().as_u16();
        if !resp.status().is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(TrueLayerError::ApiError {
                status,
                message: body,
            });
        }

        let token_resp: TokenResponse = resp.json().await?;
        let expires_at =
            Instant::now() + Duration::from_secs(token_resp.expires_in.saturating_sub(60));

        {
            let mut cache = self.token_cache.lock().await;
            *cache = Some(TokenCache {
                token: token_resp.access_token.clone(),
                expires_at,
            });
        }

        Ok(token_resp.access_token)
    }

    pub async fn create_payment_mandate(
        &self,
        user_id: &str,
        return_url: &str,
    ) -> Result<CreateMandateResponse, TrueLayerError> {
        if !self.enabled {
            return Err(TrueLayerError::Disabled);
        }

        let token = self.get_access_token().await?;
        let mandate_reference = format!("tachyon-{}", user_id);

        let body = serde_json::json!({
            "type": "sweeping",
            "mandate_reference": mandate_reference,
            "merchant_account_id": self.merchant_account_id,
            "consent_settings": {
                "max_amount_in_minor": 5000000,
                "period_alignment": "calendar",
                "period_type": "month",
            },
            "user": {
                "name": user_id,
                "email": null,
            },
            "return_url": return_url,
        });

        let resp = self
            .client
            .post(format!("{}/mandates", self.base_url))
            .header("Authorization", format!("Bearer {}", token))
            .json(&body)
            .send()
            .await?;

        let status = resp.status().as_u16();
        if !resp.status().is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(TrueLayerError::ApiError {
                status,
                message: body,
            });
        }

        let result: CreateMandateResponse = resp.json().await?;
        Ok(result)
    }

    pub async fn get_mandate_status(
        &self,
        mandate_id: &str,
    ) -> Result<MandateStatus, TrueLayerError> {
        if !self.enabled {
            return Err(TrueLayerError::Disabled);
        }

        let token = self.get_access_token().await?;

        let resp = self
            .client
            .get(format!("{}/mandates/{}", self.base_url, mandate_id))
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await?;

        let status = resp.status().as_u16();
        if !resp.status().is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(TrueLayerError::ApiError {
                status,
                message: body,
            });
        }

        let result: MandateStatus = resp.json().await?;
        Ok(result)
    }

    pub async fn create_payment(
        &self,
        mandate_id: &str,
        amount_in_pence: u64,
        reference: &str,
    ) -> Result<CreatePaymentResponse, TrueLayerError> {
        if !self.enabled {
            return Err(TrueLayerError::Disabled);
        }

        let token = self.get_access_token().await?;

        let body = serde_json::json!({
            "mandate_id": mandate_id,
            "amount_in_minor": amount_in_pence,
            "currency": "GBP",
            "reference": reference,
        });

        let resp = self
            .client
            .post(format!("{}/payments", self.base_url))
            .header("Authorization", format!("Bearer {}", token))
            .json(&body)
            .send()
            .await?;

        let status = resp.status().as_u16();
        if !resp.status().is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(TrueLayerError::ApiError {
                status,
                message: body,
            });
        }

        let result: CreatePaymentResponse = resp.json().await?;
        Ok(result)
    }

    pub async fn get_payment_status(
        &self,
        payment_id: &str,
    ) -> Result<PaymentStatusResponse, TrueLayerError> {
        if !self.enabled {
            return Err(TrueLayerError::Disabled);
        }

        let token = self.get_access_token().await?;

        let resp = self
            .client
            .get(format!("{}/payments/{}", self.base_url, payment_id))
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await?;

        let status = resp.status().as_u16();
        if !resp.status().is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(TrueLayerError::ApiError {
                status,
                message: body,
            });
        }

        let result: PaymentStatusResponse = resp.json().await?;
        Ok(result)
    }

    pub fn webhook_secret(&self) -> &str {
        &self.webhook_secret
    }
}
