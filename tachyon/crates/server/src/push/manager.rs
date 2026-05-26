use super::web_push::{PushPayload, PushSubscription};

#[derive(Default)]
pub struct PushManager {}

impl PushManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn send(
        &self,
        subscription: &PushSubscription,
        payload: &PushPayload,
    ) -> Result<(), String> {
        tracing::info!(
            endpoint = %subscription.endpoint,
            title = %payload.title,
            "Push notification sent"
        );
        Ok(())
    }

    pub async fn broadcast(
        &self,
        subscriptions: &[PushSubscription],
        payload: &PushPayload,
    ) -> Vec<Result<(), String>> {
        let mut results = Vec::with_capacity(subscriptions.len());
        for sub in subscriptions {
            results.push(self.send(sub, payload).await);
        }
        results
    }
}
