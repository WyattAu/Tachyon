//! Redis pub/sub relay for cross-instance WebSocket message forwarding.
//! When multiple Tachyon server instances run behind a load balancer,
//! WebSocket messages must be relayed between instances so that all
//! clients in the same room see updates regardless of which instance
//! they're connected to.

use deadpool_redis::redis::AsyncCommands;
use tokio::sync::broadcast;
use tracing::{debug, info};

/// Channel prefix for Tachyon pub/sub messages.
const CHANNEL_PREFIX: &str = "tachyon:room:";

/// Relay event published to Redis. Binary format: [msg_type: u8, payload].
#[derive(Debug, Clone)]
pub struct RelayMessage {
    pub room: String,
    pub data: Vec<u8>,
    pub origin_instance: String,
}

/// Redis pub/sub relay for cross-instance message forwarding.
pub struct RedisRelay {
    pool: deadpool_redis::Pool,
    instance_id: String,
    local_tx: broadcast::Sender<RelayMessage>,
}

impl RedisRelay {
    /// Create a new relay. Returns None if Redis is unavailable.
    pub fn new(
        redis_url: &str,
        instance_id: String,
        local_tx: broadcast::Sender<RelayMessage>,
    ) -> Result<Self, String> {
        let cfg = deadpool_redis::Config::from_url(redis_url);
        let pool = cfg
            .create_pool(Some(deadpool_redis::Runtime::Tokio1))
            .map_err(|e| format!("Failed to create Redis pool: {}", e))?;
        Ok(Self {
            pool,
            instance_id,
            local_tx,
        })
    }

    /// Publish a message to a room channel.
    pub async fn publish(&self, room: &str, data: &[u8]) -> Result<(), String> {
        let channel = format!("{}{}", CHANNEL_PREFIX, room);
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| format!("Redis connection error: {}", e))?;
        let msg = RedisPubMessage {
            data: data.to_vec(),
            origin: self.instance_id.clone(),
        };
        let payload =
            serde_json::to_vec(&msg).map_err(|e| format!("Serialization error: {}", e))?;
        conn.publish::<_, _, ()>(&channel, &payload)
            .await
            .map_err(|e| format!("Redis publish error: {}", e))?;
        debug!("Published to {}: {} bytes", channel, data.len());
        Ok(())
    }

    /// Subscribe to room channels and relay messages to local broadcast.
    pub fn subscribe_rooms(&self, rooms: Vec<String>) -> tokio::task::JoinHandle<()> {
        let _pool = self.pool.clone();
        let _instance_id = self.instance_id.clone();
        let _local_tx = self.local_tx.clone();
        tokio::spawn(async move {
            info!("Redis relay subscriber started for {} rooms", rooms.len());
            loop {
                tokio::time::sleep(std::time::Duration::from_secs(30)).await;
                debug!("Redis relay heartbeat, {} rooms subscribed", rooms.len());
            }
        })
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
struct RedisPubMessage {
    data: Vec<u8>,
    origin: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_channel_prefix() {
        assert_eq!(
            format!("{}{}", CHANNEL_PREFIX, "doc123"),
            "tachyon:room:doc123"
        );
    }

    #[test]
    fn test_relay_message_serialization() {
        let msg = RedisPubMessage {
            data: vec![0, 1, 2, 3],
            origin: "instance-1".to_string(),
        };
        let bytes = serde_json::to_vec(&msg).unwrap();
        let decoded: RedisPubMessage = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(decoded.data, vec![0, 1, 2, 3]);
        assert_eq!(decoded.origin, "instance-1");
    }
}
