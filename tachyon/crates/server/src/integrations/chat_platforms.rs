use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::{debug, info, warn};

const RATE_LIMIT_INTERVAL: Duration = Duration::from_secs(1);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatPlatformConfig {
    pub slack_webhook_url: Option<String>,
    pub slack_channel: Option<String>,
    pub discord_webhook_url: Option<String>,
    pub discord_username: Option<String>,
    pub discord_avatar_url: Option<String>,
    #[serde(default)]
    pub enabled_platforms: Vec<String>,
}

impl Default for ChatPlatformConfig {
    fn default() -> Self {
        Self {
            slack_webhook_url: None,
            slack_channel: Some("#general".to_string()),
            discord_webhook_url: None,
            discord_username: Some("Tachyon".to_string()),
            discord_avatar_url: None,
            enabled_platforms: Vec::new(),
        }
    }
}

impl ChatPlatformConfig {
    pub fn slack_enabled(&self) -> bool {
        self.slack_webhook_url.is_some()
            && self
                .enabled_platforms
                .iter()
                .any(|p| p.eq_ignore_ascii_case("slack"))
    }

    pub fn discord_enabled(&self) -> bool {
        self.discord_webhook_url.is_some()
            && self
                .enabled_platforms
                .iter()
                .any(|p| p.eq_ignore_ascii_case("discord"))
    }

    pub fn any_enabled(&self) -> bool {
        self.slack_enabled() || self.discord_enabled()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationData {
    pub title: String,
    pub body: Option<String>,
    pub link: Option<String>,
    pub notification_type: String,
    pub author: Option<String>,
    pub timestamp: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeliveryResult {
    pub platform: String,
    pub success: bool,
    pub status_code: Option<u16>,
    pub error: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ChatPlatformDispatcher {
    http_client: reqwest::Client,
    config: ChatPlatformConfig,
}

impl ChatPlatformDispatcher {
    pub fn new(http_client: reqwest::Client, config: ChatPlatformConfig) -> Self {
        Self {
            http_client,
            config,
        }
    }

    pub fn config(&self) -> &ChatPlatformConfig {
        &self.config
    }

    pub async fn send_notification(&self, notification: &NotificationData) -> Vec<DeliveryResult> {
        let mut results = Vec::new();

        if self.config.slack_enabled() {
            let payload = build_slack_payload(notification, &self.config);
            let result = self.send_to_slack(&payload).await;
            results.push(result);
        }

        if !results.is_empty() {
            tokio::time::sleep(RATE_LIMIT_INTERVAL).await;
        }

        if self.config.discord_enabled() {
            let payload = build_discord_payload(notification, &self.config);
            let result = self.send_to_discord(&payload).await;
            results.push(result);
        }

        results
    }

    pub async fn send_test(&self) -> Vec<DeliveryResult> {
        let test_notification = NotificationData {
            title: "Tachyon Test Notification".to_string(),
            body: Some(
                "This is a test message from Tachyon. Your webhook integration is working correctly."
                    .to_string(),
            ),
            link: None,
            notification_type: "system_test".to_string(),
            author: Some("Tachyon System".to_string()),
            timestamp: Some(chrono::Utc::now().to_rfc3339()),
        };
        self.send_notification(&test_notification).await
    }

    async fn send_to_slack(&self, payload: &serde_json::Value) -> DeliveryResult {
        let url = match &self.config.slack_webhook_url {
            Some(u) => u.clone(),
            None => {
                return DeliveryResult {
                    platform: "slack".to_string(),
                    success: false,
                    status_code: None,
                    error: Some("No Slack webhook URL configured".to_string()),
                };
            }
        };

        debug!(platform = "slack", "Sending Slack webhook notification");

        match self
            .http_client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(payload)
            .send()
            .await
        {
            Ok(resp) => {
                let status = resp.status();
                if status.is_success() {
                    info!(platform = "slack", status = %status, "Slack notification delivered");
                    DeliveryResult {
                        platform: "slack".to_string(),
                        success: true,
                        status_code: Some(status.as_u16()),
                        error: None,
                    }
                } else {
                    let body = resp.text().await.unwrap_or_default();
                    warn!(
                        platform = "slack",
                        status = %status,
                        body = %body,
                        "Slack webhook returned non-success status"
                    );
                    DeliveryResult {
                        platform: "slack".to_string(),
                        success: false,
                        status_code: Some(status.as_u16()),
                        error: Some(format!("HTTP {}: {}", status.as_u16(), body)),
                    }
                }
            }
            Err(e) => {
                warn!(platform = "slack", error = %e, "Failed to send Slack notification");
                DeliveryResult {
                    platform: "slack".to_string(),
                    success: false,
                    status_code: None,
                    error: Some(format!("Request error: {}", e)),
                }
            }
        }
    }

    async fn send_to_discord(&self, payload: &serde_json::Value) -> DeliveryResult {
        let url = match &self.config.discord_webhook_url {
            Some(u) => u.clone(),
            None => {
                return DeliveryResult {
                    platform: "discord".to_string(),
                    success: false,
                    status_code: None,
                    error: Some("No Discord webhook URL configured".to_string()),
                };
            }
        };

        debug!(platform = "discord", "Sending Discord webhook notification");

        match self
            .http_client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(payload)
            .send()
            .await
        {
            Ok(resp) => {
                let status = resp.status();
                if status.is_success() {
                    info!(platform = "discord", status = %status, "Discord notification delivered");
                    DeliveryResult {
                        platform: "discord".to_string(),
                        success: true,
                        status_code: Some(status.as_u16()),
                        error: None,
                    }
                } else {
                    let body = resp.text().await.unwrap_or_default();
                    warn!(
                        platform = "discord",
                        status = %status,
                        body = %body,
                        "Discord webhook returned non-success status"
                    );
                    DeliveryResult {
                        platform: "discord".to_string(),
                        success: false,
                        status_code: Some(status.as_u16()),
                        error: Some(format!("HTTP {}: {}", status.as_u16(), body)),
                    }
                }
            }
            Err(e) => {
                warn!(platform = "discord", error = %e, "Failed to send Discord notification");
                DeliveryResult {
                    platform: "discord".to_string(),
                    success: false,
                    status_code: None,
                    error: Some(format!("Request error: {}", e)),
                }
            }
        }
    }
}

fn notification_color_hex(notification_type: &str) -> String {
    match notification_type {
        "review_requested" => "#7B68EE".to_string(),
        "review_approved" | "approved" | "mandate_approved" | "mandate_active" => {
            "#36a64f".to_string()
        }
        "review_rejected" | "rejected" => "#e01e5a".to_string(),
        "document_created" => "#36a64f".to_string(),
        "document_updated" => "#3F51B5".to_string(),
        "document_deleted" => "#e01e5a".to_string(),
        "comment_added" | "review_commented" => "#FFA500".to_string(),
        "system_test" => "#00BCD4".to_string(),
        _ => "#808080".to_string(),
    }
}

fn notification_color_decimal(notification_type: &str) -> u32 {
    let hex = notification_color_hex(notification_type);
    let hex = hex.trim_start_matches('#');
    u32::from_str_radix(hex, 16).unwrap_or(8421504)
}

fn build_slack_payload(
    notification: &NotificationData,
    config: &ChatPlatformConfig,
) -> serde_json::Value {
    let channel = config.slack_channel.as_deref().unwrap_or("#general");
    let color = notification_color_hex(&notification.notification_type);

    let mut fields = serde_json::json!([]);
    if let Some(ref author) = notification.author {
        fields.as_array_mut().unwrap().push(serde_json::json!({
            "title": "Author",
            "value": author,
            "short": true
        }));
    }
    if let Some(ref ts) = notification.timestamp {
        fields.as_array_mut().unwrap().push(serde_json::json!({
            "title": "Time",
            "value": ts,
            "short": true
        }));
    }

    let fallback_text = notification.body.as_deref().unwrap_or(&notification.title);

    let mut attachment = serde_json::json!({
        "color": color,
        "title": notification.title,
        "text": notification.body.as_deref().unwrap_or(""),
        "fields": fields,
        "footer": "Tachyon"
    });

    if let Some(ref link) = notification.link {
        attachment["title_link"] = serde_json::json!(link);
    }

    if let Some(ref ts) = notification.timestamp
        && let Ok(dt) = chrono::DateTime::parse_from_rfc3339(ts)
    {
        attachment["ts"] = serde_json::json!(dt.timestamp());
    }

    serde_json::json!({
        "channel": channel,
        "username": "Tachyon",
        "text": fallback_text,
        "attachments": [attachment]
    })
}

fn build_discord_payload(
    notification: &NotificationData,
    config: &ChatPlatformConfig,
) -> serde_json::Value {
    let username = config.discord_username.as_deref().unwrap_or("Tachyon");
    let color = notification_color_decimal(&notification.notification_type);

    let mut fields = serde_json::json!([]);
    if let Some(ref author) = notification.author {
        fields.as_array_mut().unwrap().push(serde_json::json!({
            "name": "Author",
            "value": author,
            "inline": true
        }));
    }
    if let Some(ref ts) = notification.timestamp {
        fields.as_array_mut().unwrap().push(serde_json::json!({
            "name": "Time",
            "value": ts,
            "inline": true
        }));
    }

    let mut embed = serde_json::json!({
        "title": notification.title,
        "description": notification.body.as_deref().unwrap_or(""),
        "color": color,
        "fields": fields,
        "footer": { "text": "Tachyon" }
    });

    if let Some(ref link) = notification.link {
        embed["url"] = serde_json::json!(link);
    }

    if let Some(ref ts) = notification.timestamp
        && chrono::DateTime::parse_from_rfc3339(ts).is_ok()
    {
        embed["timestamp"] = serde_json::json!(ts);
    }

    let mut payload = serde_json::json!({
        "username": username,
        "embeds": [embed]
    });

    if let Some(ref avatar_url) = config.discord_avatar_url {
        payload["avatar_url"] = serde_json::json!(avatar_url);
    }

    payload
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_notification() -> NotificationData {
        NotificationData {
            title: "Document Updated".to_string(),
            body: Some("Getting Started guide was updated".to_string()),
            link: Some("https://tachyon.dev/docs/getting-started".to_string()),
            notification_type: "document_updated".to_string(),
            author: Some("johndoe".to_string()),
            timestamp: Some("2026-06-01T12:00:00Z".to_string()),
        }
    }

    #[test]
    fn test_slack_payload_format() {
        let config = ChatPlatformConfig {
            slack_webhook_url: Some("https://hooks.slack.com/test".to_string()),
            slack_channel: Some("#general".to_string()),
            enabled_platforms: vec!["slack".to_string()],
            ..Default::default()
        };
        let notification = sample_notification();
        let payload = build_slack_payload(&notification, &config);

        assert_eq!(payload["channel"], "#general");
        assert_eq!(payload["username"], "Tachyon");
        assert_eq!(payload["text"], "Getting Started guide was updated");

        let attachments = payload["attachments"].as_array().unwrap();
        assert_eq!(attachments.len(), 1);

        let att = &attachments[0];
        assert_eq!(att["title"], "Document Updated");
        assert_eq!(
            att["title_link"],
            "https://tachyon.dev/docs/getting-started"
        );
        assert_eq!(att["color"], "#3F51B5");
        assert_eq!(att["footer"], "Tachyon");

        let fields = att["fields"].as_array().unwrap();
        assert_eq!(fields.len(), 2);
        assert_eq!(fields[0]["title"], "Author");
        assert_eq!(fields[0]["value"], "johndoe");
        assert_eq!(fields[0]["short"], true);
        assert_eq!(fields[1]["title"], "Time");
        assert_eq!(fields[1]["value"], "2026-06-01T12:00:00Z");
        assert_eq!(fields[1]["short"], true);
    }

    #[test]
    fn test_slack_payload_no_link() {
        let config = ChatPlatformConfig {
            slack_webhook_url: Some("https://hooks.slack.com/test".to_string()),
            enabled_platforms: vec!["slack".to_string()],
            ..Default::default()
        };
        let notification = NotificationData {
            title: "System Alert".to_string(),
            body: Some("Something happened".to_string()),
            link: None,
            notification_type: "system_test".to_string(),
            author: None,
            timestamp: None,
        };
        let payload = build_slack_payload(&notification, &config);

        let attachments = payload["attachments"].as_array().unwrap();
        let att = &attachments[0];
        assert_eq!(att["title"], "System Alert");
        assert!(att.get("title_link").is_none());
        assert_eq!(att["color"], "#00BCD4");
        assert!(att["fields"].as_array().unwrap().is_empty());
    }

    #[test]
    fn test_slack_payload_fallback_text_when_no_body() {
        let config = ChatPlatformConfig {
            slack_webhook_url: Some("https://hooks.slack.com/test".to_string()),
            enabled_platforms: vec!["slack".to_string()],
            ..Default::default()
        };
        let notification = NotificationData {
            title: "No Body Notification".to_string(),
            body: None,
            link: None,
            notification_type: "system_test".to_string(),
            author: None,
            timestamp: None,
        };
        let payload = build_slack_payload(&notification, &config);
        assert_eq!(payload["text"], "No Body Notification");
    }

    #[test]
    fn test_discord_payload_format() {
        let config = ChatPlatformConfig {
            discord_webhook_url: Some("https://discord.com/api/webhooks/test".to_string()),
            discord_username: Some("Tachyon".to_string()),
            discord_avatar_url: Some("https://tachyon.dev/avatar.png".to_string()),
            enabled_platforms: vec!["discord".to_string()],
            ..Default::default()
        };
        let notification = sample_notification();
        let payload = build_discord_payload(&notification, &config);

        assert_eq!(payload["username"], "Tachyon");
        assert_eq!(payload["avatar_url"], "https://tachyon.dev/avatar.png");

        let embeds = payload["embeds"].as_array().unwrap();
        assert_eq!(embeds.len(), 1);

        let embed = &embeds[0];
        assert_eq!(embed["title"], "Document Updated");
        assert_eq!(embed["url"], "https://tachyon.dev/docs/getting-started");
        assert_eq!(embed["description"], "Getting Started guide was updated");
        // 0x3F51B5 = 4149685
        assert_eq!(embed["color"], 4149685);
        assert_eq!(embed["footer"]["text"], "Tachyon");
        assert_eq!(embed["timestamp"], "2026-06-01T12:00:00Z");

        let fields = embed["fields"].as_array().unwrap();
        assert_eq!(fields.len(), 2);
        assert_eq!(fields[0]["name"], "Author");
        assert_eq!(fields[0]["value"], "johndoe");
        assert_eq!(fields[0]["inline"], true);
        assert_eq!(fields[1]["name"], "Time");
        assert_eq!(fields[1]["value"], "2026-06-01T12:00:00Z");
        assert_eq!(fields[1]["inline"], true);
    }

    #[test]
    fn test_discord_payload_no_link() {
        let config = ChatPlatformConfig {
            discord_webhook_url: Some("https://discord.com/api/webhooks/test".to_string()),
            discord_username: Some("Tachyon".to_string()),
            enabled_platforms: vec!["discord".to_string()],
            ..Default::default()
        };
        let notification = NotificationData {
            title: "System Alert".to_string(),
            body: Some("Something happened".to_string()),
            link: None,
            notification_type: "system_test".to_string(),
            author: None,
            timestamp: None,
        };
        let payload = build_discord_payload(&notification, &config);

        let embeds = payload["embeds"].as_array().unwrap();
        let embed = &embeds[0];
        assert!(embed.get("url").is_none());
        assert!(embed.get("timestamp").is_none());
        assert!(embed.get("avatar_url").is_none());
        assert!(embed["fields"].as_array().unwrap().is_empty());
    }

    #[test]
    fn test_discord_payload_no_avatar() {
        let config = ChatPlatformConfig {
            discord_webhook_url: Some("https://discord.com/api/webhooks/test".to_string()),
            enabled_platforms: vec!["discord".to_string()],
            ..Default::default()
        };
        let notification = sample_notification();
        let payload = build_discord_payload(&notification, &config);
        assert_eq!(payload["username"], "Tachyon");
        assert!(payload.get("avatar_url").is_none());
    }

    #[test]
    fn test_color_mapping_review_requested() {
        let hex = notification_color_hex("review_requested");
        assert_eq!(hex, "#7B68EE");
        // 0x7B68EE = 8087790
        let dec = notification_color_decimal("review_requested");
        assert_eq!(dec, 8087790);
    }

    #[test]
    fn test_color_mapping_approved() {
        let hex = notification_color_hex("review_approved");
        assert_eq!(hex, "#36a64f");
        // 0x36a64f = 3581519
        let dec = notification_color_decimal("approved");
        assert_eq!(dec, 3581519);
    }

    #[test]
    fn test_color_mapping_rejected() {
        let hex = notification_color_hex("review_rejected");
        assert_eq!(hex, "#e01e5a");
        // 0xe01e5a = 14687834
        let dec = notification_color_decimal("rejected");
        assert_eq!(dec, 14687834);
    }

    #[test]
    fn test_color_mapping_document_created() {
        assert_eq!(notification_color_hex("document_created"), "#36a64f");
    }

    #[test]
    fn test_color_mapping_document_updated() {
        assert_eq!(notification_color_hex("document_updated"), "#3F51B5");
    }

    #[test]
    fn test_color_mapping_document_deleted() {
        assert_eq!(notification_color_hex("document_deleted"), "#e01e5a");
    }

    #[test]
    fn test_color_mapping_comment() {
        assert_eq!(notification_color_hex("comment_added"), "#FFA500");
        assert_eq!(notification_color_hex("review_commented"), "#FFA500");
    }

    #[test]
    fn test_color_mapping_unknown() {
        assert_eq!(notification_color_hex("custom_event"), "#808080");
    }

    #[test]
    fn test_color_mapping_system_test() {
        assert_eq!(notification_color_hex("system_test"), "#00BCD4");
    }

    #[test]
    fn test_chat_platform_config_slack_enabled() {
        let config = ChatPlatformConfig {
            slack_webhook_url: Some("https://hooks.slack.com/test".to_string()),
            enabled_platforms: vec!["slack".to_string()],
            ..Default::default()
        };
        assert!(config.slack_enabled());
        assert!(!config.discord_enabled());
        assert!(config.any_enabled());
    }

    #[test]
    fn test_chat_platform_config_discord_enabled() {
        let config = ChatPlatformConfig {
            discord_webhook_url: Some("https://discord.com/api/webhooks/test".to_string()),
            enabled_platforms: vec!["discord".to_string()],
            ..Default::default()
        };
        assert!(!config.slack_enabled());
        assert!(config.discord_enabled());
        assert!(config.any_enabled());
    }

    #[test]
    fn test_chat_platform_config_case_insensitive() {
        let config = ChatPlatformConfig {
            slack_webhook_url: Some("https://hooks.slack.com/test".to_string()),
            discord_webhook_url: Some("https://discord.com/api/webhooks/test".to_string()),
            enabled_platforms: vec!["Slack".to_string(), "DISCORD".to_string()],
            ..Default::default()
        };
        assert!(config.slack_enabled());
        assert!(config.discord_enabled());
    }

    #[test]
    fn test_chat_platform_config_url_without_enabled_flag() {
        let config = ChatPlatformConfig {
            slack_webhook_url: Some("https://hooks.slack.com/test".to_string()),
            enabled_platforms: Vec::new(),
            ..Default::default()
        };
        assert!(!config.slack_enabled());
        assert!(!config.any_enabled());
    }

    #[test]
    fn test_delivery_result_serialization() {
        let result = DeliveryResult {
            platform: "slack".to_string(),
            success: true,
            status_code: Some(200),
            error: None,
        };
        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("\"platform\":\"slack\""));
        assert!(json.contains("\"success\":true"));
        assert!(json.contains("\"status_code\":200"));
    }

    #[test]
    fn test_notification_data_serialization() {
        let data = sample_notification();
        let json = serde_json::to_string(&data).unwrap();
        assert!(json.contains("Document Updated"));
        assert!(json.contains("johndoe"));
        assert!(json.contains("document_updated"));
    }

    #[tokio::test]
    async fn test_send_notification_no_platforms_enabled() {
        let dispatcher =
            ChatPlatformDispatcher::new(reqwest::Client::new(), ChatPlatformConfig::default());
        let notification = sample_notification();
        let results = dispatcher.send_notification(&notification).await;
        assert!(results.is_empty());
    }

    #[tokio::test]
    async fn test_send_test_no_platforms_enabled() {
        let dispatcher =
            ChatPlatformDispatcher::new(reqwest::Client::new(), ChatPlatformConfig::default());
        let results = dispatcher.send_test().await;
        assert!(results.is_empty());
    }
}
