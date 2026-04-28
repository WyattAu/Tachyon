use hmac::{Hmac, Mac};
use sha2::Sha256;
use std::time::Duration;
use tachyon_database::{DatabasePool, WebhookRepository};
use tracing::{debug, warn};

type HmacSha256 = Hmac<Sha256>;

const MAX_RETRIES: u32 = 3;
const INITIAL_BACKOFF: Duration = Duration::from_secs(1);
const MAX_BACKOFF: Duration = Duration::from_secs(60);

fn compute_signature(secret: &str, body: &[u8]) -> String {
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
        .unwrap_or_else(|_| HmacSha256::new_from_slice(b"").expect("HMAC accepts any key length"));
    mac.update(body);
    let result = mac.finalize();
    hex::encode(result.into_bytes())
}

async fn try_deliver(
    client: &reqwest::Client,
    webhook_url: &str,
    body: &str,
    event_type: &str,
    webhook_id: &str,
    secret: Option<&str>,
) -> Result<(), String> {
    let mut req = client
        .post(webhook_url)
        .header("Content-Type", "application/json")
        .header("X-Tachyon-Event", event_type)
        .header("X-Tachyon-Delivery", webhook_id);

    if let Some(secret) = secret {
        let sig = compute_signature(secret, body.as_bytes());
        debug!("Webhook {}: computed signature", webhook_id);
        req = req.header("X-Tachyon-Signature", format!("sha256={}", sig));
    }

    req.body(body.to_string())
        .send()
        .await
        .map_err(|e| format!("HTTP error: {}", e))?
        .error_for_status()
        .map(|_| ())
        .map_err(|e| format!("status error: {}", e))
}

async fn deliver_with_retry(
    client: &reqwest::Client,
    webhook_url: &str,
    body: &str,
    event_type: &str,
    webhook_id: &str,
    secret: Option<&str>,
) -> Result<(), String> {
    let mut backoff = INITIAL_BACKOFF;

    for attempt in 0..MAX_RETRIES {
        match try_deliver(client, webhook_url, body, event_type, webhook_id, secret).await {
            Ok(_) => return Ok(()),
            Err(e) if attempt < MAX_RETRIES - 1 => {
                warn!(
                    "Webhook delivery attempt {}/{} failed for {}: {}",
                    attempt + 1,
                    MAX_RETRIES,
                    webhook_url,
                    e
                );
                tokio::time::sleep(backoff).await;
                backoff = (backoff * 2).min(MAX_BACKOFF);
            }
            Err(e) => {
                warn!(
                    "Webhook delivery failed after {} attempts for {}: {}",
                    MAX_RETRIES, webhook_url, e
                );
                return Err(e);
            }
        }
    }
    Ok(())
}

pub async fn deliver_event(pool: DatabasePool, http_client: reqwest::Client, event_type: &str, payload: &serde_json::Value) {
    let webhooks = match WebhookRepository::get_active_by_event(&pool, event_type).await {
        Ok(w) => w,
        Err(e) => {
            warn!("Failed to fetch webhooks for event '{}': {}", event_type, e);
            return;
        }
    };

    if webhooks.is_empty() {
        return;
    }

    let body = serde_json::to_string(payload).unwrap_or_default();
    let event_type_owned = event_type.to_string();

    for webhook in webhooks {
        let pool_clone = pool.clone();
        let body_clone = body.clone();
        let event_type_clone = event_type_owned.clone();
        let webhook_id = webhook.id;
        let webhook_url = webhook.url.clone();
        let webhook_secret = webhook.secret.clone();

        let http_client = http_client.clone();
        tokio::spawn(async move {
            let client = http_client;

            let _ = deliver_with_retry(
                &client,
                &webhook_url,
                &body_clone,
                &event_type_clone,
                &webhook_id.to_string(),
                webhook_secret.as_deref(),
            )
            .await;

            if let Err(e) = WebhookRepository::update_last_triggered(&pool_clone, webhook_id).await {
                warn!("Failed to update last_triggered_at for webhook {}: {}", webhook_id, e);
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_signature() {
        let sig = compute_signature("secret", b"payload");
        assert!(!sig.is_empty());
        assert_eq!(sig.len(), 64);
    }

    #[test]
    fn test_compute_signature_deterministic() {
        let sig1 = compute_signature("my-secret", b"hello world");
        let sig2 = compute_signature("my-secret", b"hello world");
        assert_eq!(sig1, sig2);
    }

    #[test]
    fn test_compute_signature_different_secrets() {
        let sig1 = compute_signature("secret-a", b"data");
        let sig2 = compute_signature("secret-b", b"data");
        assert_ne!(sig1, sig2);
    }

    #[test]
    fn test_backoff_constants() {
        assert_eq!(MAX_RETRIES, 3);
        assert_eq!(INITIAL_BACKOFF, Duration::from_secs(1));
        assert_eq!(MAX_BACKOFF, Duration::from_secs(60));
    }
}
