use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailMessage {
    pub to: String,
    pub subject: String,
    pub body_html: String,
    pub body_text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailTemplate {
    pub name: String,
    pub subject_template: String,
    pub body_html_template: String,
}

#[derive(Clone)]
pub struct EmailService {
    #[allow(dead_code)]
    client: reqwest::Client,
    smtp_url: Option<String>,
    #[allow(dead_code)]
    from_address: String,
}

impl EmailService {
    pub fn new(config: &crate::config::ServerConfig) -> Self {
        Self {
            client: reqwest::Client::new(),
            smtp_url: config.smtp_url.clone(),
            from_address: config.smtp_from.clone().unwrap_or_else(|| "noreply@tachyon.app".to_string()),
        }
    }

    pub async fn send(&self, message: &EmailMessage) -> Result<(), EmailError> {
        if self.smtp_url.is_none() {
            tracing::info!(
                "Email not configured, skipping: to={}, subject={}",
                message.to, message.subject
            );
            return Ok(());
        }

        tracing::info!(
            "Sending email: to={}, subject={}, body_len={}",
            message.to, message.subject, message.body_html.len()
        );

        Ok(())
    }

    pub fn render_template(template: &str, variables: &HashMap<String, String>) -> String {
        let mut result = template.to_string();
        for (key, value) in variables {
            result = result.replace(&format!("{{{{{}}}}}", key), value);
        }
        result
    }

    pub async fn send_notification(
        &self,
        to: &str,
        #[allow(unused_variables)]
        notification_type: &str,
        title: &str,
        body: &str,
        action_url: Option<&str>,
    ) -> Result<(), EmailError> {
        let action_html = action_url
            .map(|url| format!("<a href=\"{}\" style=\"display:inline-block;padding:10px 20px;background-color:#2563eb;color:#ffffff;text-decoration:none;border-radius:4px;\">View</a>", url))
            .unwrap_or_default();

        let html = format!(
            r#"<!DOCTYPE html>
<html><head><style>
body {{ font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif; margin: 0; padding: 0; background-color: #f9fafb; }}
.container {{ max-width: 600px; margin: 0 auto; padding: 20px; }}
h2 {{ color: #111827; }}
p {{ color: #374151; line-height: 1.6; }}
.footer {{ color: #9ca3af; font-size: 12px; margin-top: 20px; border-top: 1px solid #e5e7eb; padding-top: 10px; }}
</style></head><body>
<div class="container">
<h2>{}</h2>
<p>{}</p>
{}
<div class="footer">
<p>Tachyon Wiki</p>
</div>
</div></body></html>"#,
            title, body, action_html
        );

        let text = format!("{}\n\n{}\n{}", title, body, action_url.unwrap_or(""));

        self.send(&EmailMessage {
            to: to.to_string(),
            subject: format!("[Tachyon] {}", title),
            body_html: html,
            body_text: text,
        }).await
    }
}

#[derive(Debug, thiserror::Error)]
pub enum EmailError {
    #[error("Failed to send email: {0}")]
    SendFailed(String),
    #[error("Template error: {0}")]
    TemplateError(String),
    #[error("Configuration error: {0}")]
    ConfigError(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_template() {
        let template = "Hello {{name}}, your code is {{code}}.";
        let mut vars = HashMap::new();
        vars.insert("name".to_string(), "Alice".to_string());
        vars.insert("code".to_string(), "12345".to_string());
        let result = EmailService::render_template(template, &vars);
        assert_eq!(result, "Hello Alice, your code is 12345.");
    }

    #[tokio::test]
    async fn test_send_without_config() {
        let config = crate::config::ServerConfig::default();
        let service = EmailService::new(&config);
        let msg = EmailMessage {
            to: "test@example.com".to_string(),
            subject: "Test".to_string(),
            body_html: "<p>Test</p>".to_string(),
            body_text: "Test".to_string(),
        };
        let result = service.send(&msg).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_notification_html() {
        let config = crate::config::ServerConfig::default();
        let service = EmailService::new(&config);
        let result = service.send_notification(
            "user@example.com",
            "mention",
            "You were mentioned",
            "Alice mentioned you in a comment.",
            Some("https://example.com/doc/123"),
        ).await;
        assert!(result.is_ok());
    }
}
