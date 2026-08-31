use std::collections::HashMap;
use std::time::Duration;

use lettre::message::header::ContentType;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};
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

struct SmtpConfig {
    host: String,
    port: u16,
    username: Option<String>,
    password: Option<String>,
    tls: bool,
}

#[derive(Clone)]
pub struct EmailService {
    transport: Option<AsyncSmtpTransport<Tokio1Executor>>,
    from_address: String,
}

impl EmailService {
    pub fn new(config: &crate::config::ServerConfig) -> Self {
        let from_address = config
            .smtp_from
            .clone()
            .unwrap_or_else(|| "noreply@tachyon.app".to_string());

        let transport = config
            .smtp_url
            .as_ref()
            .and_then(|url| match parse_smtp_url(url) {
                Ok(mut parsed) => {
                    if let Some(ref username) = config.smtp_username {
                        parsed.username = Some(username.clone());
                    }
                    if let Some(ref password) = config.smtp_password {
                        parsed.password = Some(password.clone());
                    }
                    if let Some(port) = config.smtp_port {
                        parsed.port = port;
                    }
                    if let Some(tls_override) = {
                        let tls = config.smtp_tls;
                        if tls { None } else { Some(false) }
                    } {
                        parsed.tls = tls_override;
                    }
                    match build_transport(&parsed) {
                        Ok(t) => Some(t),
                        Err(e) => {
                            tracing::warn!("Failed to build SMTP transport: {}", e);
                            None
                        }
                    }
                }
                Err(e) => {
                    tracing::warn!("Failed to parse SMTP URL '{}': {}", url, e);
                    None
                }
            });

        Self {
            transport,
            from_address,
        }
    }

    pub async fn send(&self, message: &EmailMessage) -> Result<(), EmailError> {
        let transport = match &self.transport {
            Some(t) => t,
            None => {
                tracing::error!(
                    to = %message.to,
                    subject = %message.subject,
                    "Email delivery requested but SMTP is not configured"
                );
                return Err(EmailError::ConfigError(
                    "SMTP is not configured; email delivery is unavailable".to_string(),
                ));
            }
        };

        let email = Message::builder()
            .from(self.from_address.parse().map_err(|e| {
                EmailError::ConfigError(format!(
                    "Invalid from address '{}': {}",
                    self.from_address, e
                ))
            })?)
            .to(message.to.parse().map_err(|e| {
                EmailError::ConfigError(format!("Invalid to address '{}': {}", message.to, e))
            })?)
            .subject(&message.subject)
            .multipart(
                lettre::message::MultiPart::alternative()
                    .singlepart(
                        lettre::message::SinglePart::builder()
                            .header(ContentType::TEXT_PLAIN)
                            .body(message.body_text.clone()),
                    )
                    .singlepart(
                        lettre::message::SinglePart::builder()
                            .header(ContentType::TEXT_HTML)
                            .body(message.body_html.clone()),
                    ),
            )
            .map_err(|e| EmailError::SendFailed(format!("Failed to build email message: {}", e)))?;

        let delays = [
            Duration::from_secs(1),
            Duration::from_secs(5),
            Duration::from_secs(15),
        ];

        for (attempt, delay) in delays.iter().enumerate() {
            match transport.send(email.clone()).await {
                Ok(response) => {
                    tracing::info!(
                        "Email sent successfully: to={}, subject={}, response={:?}",
                        message.to,
                        message.subject,
                        response
                    );
                    return Ok(());
                }
                Err(e) => {
                    tracing::warn!(
                        "Email send attempt {}/{} failed for to={}: {}",
                        attempt + 1,
                        delays.len(),
                        message.to,
                        e
                    );
                    if attempt < delays.len() - 1 {
                        tokio::time::sleep(*delay).await;
                    } else {
                        return Err(EmailError::SendFailed(format!(
                            "Failed to send email to {} after {} attempts: {}",
                            message.to,
                            delays.len(),
                            e
                        )));
                    }
                }
            }
        }

        // The loop above always returns either Ok(()) on success or
        // Err(EmailError::SendFailed) on the final retry failure.
        // This is structurally unreachable but kept as a defensive fallback.
        Err(EmailError::SendFailed(
            "Email retry loop exhausted without result".to_string(),
        ))
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
        _notification_type: &str,
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
        })
        .await
    }
}

fn parse_smtp_url(url: &str) -> Result<SmtpConfig, EmailError> {
    let tls = url.starts_with("smtps://");
    let stripped = if tls {
        url.strip_prefix("smtps://")
    } else {
        url.strip_prefix("smtp://")
    }
    .ok_or_else(|| {
        EmailError::ConfigError(format!(
            "SMTP URL must start with smtp:// or smtps://, got: {}",
            url
        ))
    })?;

    let (username, password, host_port) = match stripped.find('@') {
        Some(at_idx) => {
            let creds = &stripped[..at_idx];
            let host_part = &stripped[at_idx + 1..];
            let (u, p) = match creds.find(':') {
                Some(colon_idx) => {
                    let user = &creds[..colon_idx];
                    let pass = &creds[colon_idx + 1..];
                    (
                        if user.is_empty() {
                            None
                        } else {
                            Some(user.to_string())
                        },
                        if pass.is_empty() {
                            None
                        } else {
                            Some(pass.to_string())
                        },
                    )
                }
                None => (
                    if creds.is_empty() {
                        None
                    } else {
                        Some(creds.to_string())
                    },
                    None,
                ),
            };
            (u, p, host_part)
        }
        None => (None, None, stripped),
    };

    let (host, port) = match host_port.find(':') {
        Some(colon_idx) => {
            let h = &host_port[..colon_idx];
            let p_str = &host_port[colon_idx + 1..];
            let p = p_str.parse::<u16>().map_err(|_| {
                EmailError::ConfigError(format!("Invalid port '{}' in SMTP URL: {}", p_str, url))
            })?;
            (h.to_string(), p)
        }
        None => (host_port.to_string(), if tls { 465 } else { 587 }),
    };

    Ok(SmtpConfig {
        host,
        port,
        username,
        password,
        tls,
    })
}

fn build_transport(config: &SmtpConfig) -> Result<AsyncSmtpTransport<Tokio1Executor>, EmailError> {
    let mut builder = AsyncSmtpTransport::<Tokio1Executor>::relay(&config.host)
        .map_err(|e| {
            EmailError::ConfigError(format!(
                "Failed to create SMTP transport for host '{}': {}",
                config.host, e
            ))
        })?
        .port(config.port);

    if config.tls {
        builder = builder.tls(lettre::transport::smtp::client::Tls::Wrapper(
            lettre::transport::smtp::client::TlsParameters::builder(config.host.clone())
                .build()
                .map_err(|e| EmailError::ConfigError(format!("TLS configuration error: {}", e)))?,
        ));
    } else {
        builder = builder.tls(lettre::transport::smtp::client::Tls::None);
    }

    if let (Some(username), Some(password)) = (&config.username, &config.password) {
        builder = builder.credentials(Credentials::new(username.clone(), password.clone()));
    }

    Ok(builder
        .pool_config(lettre::transport::smtp::PoolConfig::new().max_size(10))
        .build())
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
        assert!(matches!(result, Err(EmailError::ConfigError(_))));
    }

    #[tokio::test]
    async fn test_notification_html() {
        let config = crate::config::ServerConfig::default();
        let service = EmailService::new(&config);
        let result = service
            .send_notification(
                "user@example.com",
                "mention",
                "You were mentioned",
                "Alice mentioned you in a comment.",
                Some("https://example.com/doc/123"),
            )
            .await;
        assert!(matches!(result, Err(EmailError::ConfigError(_))));
    }

    #[test]
    fn test_parse_smtp_url() {
        let config = parse_smtp_url("smtps://user:pass@smtp.example.com:465").unwrap();
        assert_eq!(config.host, "smtp.example.com");
        assert_eq!(config.port, 465);
        assert_eq!(config.username.as_deref(), Some("user"));
        assert_eq!(config.password.as_deref(), Some("pass"));
        assert!(config.tls);

        let config = parse_smtp_url("smtp://smtp.example.com").unwrap();
        assert_eq!(config.host, "smtp.example.com");
        assert_eq!(config.port, 587);
        assert!(config.username.is_none());
        assert!(!config.tls);

        let config = parse_smtp_url("smtps://smtp.example.com").unwrap();
        assert_eq!(config.host, "smtp.example.com");
        assert_eq!(config.port, 465);
        assert!(config.tls);

        let config = parse_smtp_url("smtp://user@smtp.example.com:2525").unwrap();
        assert_eq!(config.host, "smtp.example.com");
        assert_eq!(config.port, 2525);
        assert_eq!(config.username.as_deref(), Some("user"));
        assert!(config.password.is_none());
        assert!(!config.tls);

        assert!(parse_smtp_url("http://example.com").is_err());
        assert!(parse_smtp_url("smtp://host:badport").is_err());
    }

    #[tokio::test]
    async fn test_build_smtp_transport_parse_url() {
        let config = SmtpConfig {
            host: "smtp.example.com".to_string(),
            port: 465,
            username: Some("user".to_string()),
            password: Some("pass".to_string()),
            tls: true,
        };
        let result = build_transport(&config);
        assert!(result.is_ok());
        // Drop transport within async context to avoid panic in destructor
        drop(result);

        let config = SmtpConfig {
            host: "smtp.example.com".to_string(),
            port: 587,
            username: None,
            password: None,
            tls: false,
        };
        let result = build_transport(&config);
        assert!(result.is_ok());
        drop(result);
    }

    #[test]
    fn test_email_message_from_fields() {
        let msg = EmailMessage {
            to: "recipient@example.com".to_string(),
            subject: "Hello World".to_string(),
            body_html: "<h1>Hello</h1>".to_string(),
            body_text: "Hello".to_string(),
        };
        assert_eq!(msg.to, "recipient@example.com");
        assert_eq!(msg.subject, "Hello World");
        assert_eq!(msg.body_html, "<h1>Hello</h1>");
        assert_eq!(msg.body_text, "Hello");
    }

    #[tokio::test]
    async fn test_retry_on_failure() {
        let config = crate::config::ServerConfig::default();
        let mut service = EmailService::new(&config);
        // Use a non-routable high port to get immediate connection refused
        service.transport = Some(
            build_transport(&SmtpConfig {
                host: "127.0.0.1".to_string(),
                port: 19, // port 19 is chargen, likely closed and fast to refuse
                username: None,
                password: None,
                tls: false,
            })
            .unwrap(),
        );

        let msg = EmailMessage {
            to: "test@example.com".to_string(),
            subject: "Retry Test".to_string(),
            body_html: "<p>Test</p>".to_string(),
            body_text: "Test".to_string(),
        };

        let result = service.send(&msg).await;

        assert!(result.is_err());
        match result {
            Err(EmailError::SendFailed(msg)) => {
                assert!(msg.contains("3 attempts"));
            }
            _ => panic!("Expected SendFailed error, got: {:?}", result),
        }
    }
}
