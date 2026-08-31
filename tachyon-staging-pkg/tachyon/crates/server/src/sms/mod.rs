use async_trait::async_trait;
use reqwest::Client;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SmsError {
    #[error("Failed to send SMS: {0}")]
    SendFailed(String),
    #[error("Configuration error: {0}")]
    ConfigError(String),
    #[error("Provider error: {status} - {body}")]
    ProviderError { status: u16, body: String },
}

#[async_trait]
pub trait SmsProvider: Send + Sync {
    async fn send_sms(&self, phone: &str, message: &str) -> Result<(), SmsError>;
}

pub struct TwilioProvider {
    account_sid: String,
    auth_token: String,
    from_number: String,
    http_client: Client,
}

impl TwilioProvider {
    pub fn new(
        account_sid: String,
        auth_token: String,
        from_number: String,
        http_client: Client,
    ) -> Result<Self, SmsError> {
        if account_sid.is_empty() || auth_token.is_empty() || from_number.is_empty() {
            return Err(SmsError::ConfigError(
                "Twilio account_sid, auth_token, and from_number are all required".to_string(),
            ));
        }
        Ok(Self {
            account_sid,
            auth_token,
            from_number,
            http_client,
        })
    }
}

#[async_trait]
impl SmsProvider for TwilioProvider {
    async fn send_sms(&self, phone: &str, message: &str) -> Result<(), SmsError> {
        let url = format!(
            "https://api.twilio.com/2010-04-01/Accounts/{}/Messages.json",
            self.account_sid
        );

        let body = [
            ("To", phone),
            ("From", &self.from_number),
            ("Body", message),
        ];

        let resp = self
            .http_client
            .post(&url)
            .basic_auth(&self.account_sid, Some(&self.auth_token))
            .form(&body)
            .send()
            .await
            .map_err(|e| SmsError::SendFailed(format!("Twilio request failed: {}", e)))?;

        let status = resp.status().as_u16();
        let resp_body = resp
            .text()
            .await
            .unwrap_or_else(|_| "unable to read response body".to_string());

        if (200..300).contains(&status) {
            tracing::info!(phone = %phone, status = status, "SMS sent via Twilio");
            Ok(())
        } else {
            tracing::warn!(
                phone = %phone,
                status = status,
                body = %resp_body,
                "Twilio SMS delivery failed"
            );
            Err(SmsError::ProviderError {
                status,
                body: resp_body,
            })
        }
    }
}

pub struct GenericSmsProvider {
    api_url: String,
    api_key: String,
    from_number: String,
    http_client: Client,
}

impl GenericSmsProvider {
    pub fn new(
        api_url: String,
        api_key: String,
        from_number: String,
        http_client: Client,
    ) -> Result<Self, SmsError> {
        if api_url.is_empty() || api_key.is_empty() || from_number.is_empty() {
            return Err(SmsError::ConfigError(
                "Generic SMS api_url, api_key, and from_number are all required".to_string(),
            ));
        }
        Ok(Self {
            api_url,
            api_key,
            from_number,
            http_client,
        })
    }
}

#[async_trait]
impl SmsProvider for GenericSmsProvider {
    async fn send_sms(&self, phone: &str, message: &str) -> Result<(), SmsError> {
        let payload = serde_json::json!({
            "to": phone,
            "from": self.from_number,
            "body": message,
        });

        let resp = self
            .http_client
            .post(&self.api_url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&payload)
            .send()
            .await
            .map_err(|e| SmsError::SendFailed(format!("Generic SMS request failed: {}", e)))?;

        let status = resp.status().as_u16();
        let resp_body = resp
            .text()
            .await
            .unwrap_or_else(|_| "unable to read response body".to_string());

        if (200..300).contains(&status) {
            tracing::info!(phone = %phone, status = status, "SMS sent via generic provider");
            Ok(())
        } else {
            tracing::warn!(
                phone = %phone,
                status = status,
                body = %resp_body,
                "Generic SMS delivery failed"
            );
            Err(SmsError::ProviderError {
                status,
                body: resp_body,
            })
        }
    }
}

pub fn build_sms_provider(
    config: &crate::config::SmsOtpConfig,
    client: Client,
) -> Option<Box<dyn SmsProvider>> {
    if !config.enabled {
        return None;
    }

    match config.provider.as_str() {
        "twilio" => {
            let sid = config.twilio_account_sid.as_ref()?;
            let token = config.twilio_auth_token.as_ref()?;
            let from = config.twilio_from_number.as_ref()?;
            match TwilioProvider::new(sid.clone(), token.clone(), from.clone(), client) {
                Ok(p) => Some(Box::new(p)),
                Err(e) => {
                    tracing::warn!(error = %e, "Failed to create Twilio SMS provider");
                    None
                }
            }
        }
        "generic" => {
            let url = config.sms_api_url.as_ref()?;
            let key = config.sms_api_key.as_ref()?;
            let from = config.sms_from_number.as_ref()?;
            match GenericSmsProvider::new(url.clone(), key.clone(), from.clone(), client) {
                Ok(p) => Some(Box::new(p)),
                Err(e) => {
                    tracing::warn!(error = %e, "Failed to create generic SMS provider");
                    None
                }
            }
        }
        other => {
            tracing::warn!(provider = %other, "Unknown SMS provider, SMS OTP disabled");
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_twilio_provider_missing_config() {
        let result = TwilioProvider::new(
            String::new(),
            "token".to_string(),
            "+1234567890".to_string(),
            Client::new(),
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_generic_provider_missing_config() {
        let result = GenericSmsProvider::new(
            String::new(),
            "key".to_string(),
            "+1234567890".to_string(),
            Client::new(),
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_build_sms_provider_disabled() {
        let config = crate::config::SmsOtpConfig::default();
        let result = build_sms_provider(&config, Client::new());
        assert!(result.is_none());
    }

    #[test]
    fn test_sms_error_send_failed() {
        let err = SmsError::SendFailed("network error".to_string());
        assert!(err.to_string().contains("network error"));
    }

    #[test]
    fn test_sms_error_provider_error() {
        let err = SmsError::ProviderError {
            status: 401,
            body: "Unauthorized".to_string(),
        };
        assert!(err.to_string().contains("401"));
        assert!(err.to_string().contains("Unauthorized"));
    }
}
