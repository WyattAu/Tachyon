use hmac::{Hmac, Mac};
use sha2::Sha256;
use tachyon_database::{DatabasePool, WebhookRepository};
use tracing::{debug, warn};

type HmacSha256 = Hmac<Sha256>;

fn compute_signature(secret: &str, body: &[u8]) -> String {
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).expect("HMAC can take key of any size");
    mac.update(body);
    let result = mac.finalize();
    hex::encode(result.into_bytes())
}

pub async fn deliver_event(pool: DatabasePool, event_type: &str, payload: &serde_json::Value) {
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

        tokio::spawn(async move {
            let client = match reqwest::Client::new()
                .post(&webhook.url)
                .header("Content-Type", "application/json")
                .header("X-Tachyon-Event", &event_type_clone)
                .header("X-Tachyon-Delivery", webhook_id.to_string())
            {
                req => req,
            };

            let client = if let Some(ref secret) = webhook.secret {
                let sig = compute_signature(secret, body_clone.as_bytes());
                debug!("Webhook {}: computed signature", webhook_id);
                client.header("X-Tachyon-Signature", format!("sha256={}", sig))
            } else {
                client
            };

            match client.body(body_clone.clone()).send().await {
                Ok(resp) if resp.status().is_success() => {
                    debug!("Webhook {} delivered successfully (status {})", webhook_id, resp.status());
                }
                Ok(resp) => {
                    warn!("Webhook {} delivery failed: status {}", webhook_id, resp.status());
                }
                Err(e) => {
                    warn!("Webhook {} delivery error: {}", webhook_id, e);
                }
            }

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
}
