use tachyon_database::init_with_migrations;
use sqlx::PgPool;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use futures_util::{SinkExt, StreamExt};
use serde_json::json;

async fn setup_test_db() -> PgPool {
    let database_url = std::env::var("TEST_DATABASE_URL")
        .unwrap_or_else(|_| "postgres://tachyon:tachyon@localhost:5432/tachyon_test".to_string());
    
    init_with_migrations(&database_url)
        .await
        .expect("Failed to setup test database")
}

fn get_server_url() -> String {
    std::env::var("TEST_SERVER_URL")
        .unwrap_or_else(|_| "ws://localhost:3000/ws".to_string())
}

#[tokio::test]
#[ignore]
async fn test_websocket_connection() {
    let url = get_server_url();
    
    let result = connect_async(&url).await;
    assert!(result.is_ok(), "Failed to connect to WebSocket server");
    
    let (mut ws_stream, _) = result.unwrap();
    
    let ping_msg = json!({
        "type": "ping"
    });
    
    let result = ws_stream.send(Message::Text(ping_msg.to_string())).await;
    assert!(result.is_ok(), "Failed to send ping message");
    
    while let Some(msg_result) = ws_stream.next().await {
        match msg_result {
            Ok(Message::Text(text)) => {
                let response: serde_json::Value = serde_json::from_str(&text)
                    .expect("Failed to parse response");
                
                if response["type"] == "pong" {
                    break;
                }
            }
            Ok(Message::Ping(_)) => continue,
            Ok(Message::Pong(_)) => continue,
            Err(e) => panic!("WebSocket error: {:?}", e),
            _ => continue,
        }
    }
    
    let _ = ws_stream.close(None).await;
}

#[tokio::test]
#[ignore]
async fn test_websocket_document_subscribe() {
    let url = get_server_url();
    let (mut ws_stream, _) = connect_async(&url).await
        .expect("Failed to connect to WebSocket");
    
    let doc_id = uuid::Uuid::new_v4().to_string();
    
    let subscribe_msg = json!({
        "type": "subscribe",
        "channel": "document",
        "document_id": doc_id
    });
    
    ws_stream.send(Message::Text(subscribe_msg.to_string())).await
        .expect("Failed to send subscribe message");
    
    let mut confirmed = false;
    let timeout = tokio::time::Duration::from_secs(5);
    let start = tokio::time::Instant::now();
    
    while start.elapsed() < timeout {
        match ws_stream.next().await {
            Some(Ok(Message::Text(text))) => {
                let response: serde_json::Value = serde_json::from_str(&text).unwrap();
                if response["type"] == "subscribed" && response["channel"] == "document" {
                    confirmed = true;
                    break;
                }
            }
            Some(Ok(_)) => continue,
            _ => break,
        }
    }
    
    assert!(confirmed, "Should receive subscription confirmation");
    
    let _ = ws_stream.close(None).await;
}

#[tokio::test]
#[ignore]
async fn test_websocket_document_unsubscribe() {
    let url = get_server_url();
    let (mut ws_stream, _) = connect_async(&url).await
        .expect("Failed to connect to WebSocket");
    
    let doc_id = uuid::Uuid::new_v4().to_string();
    
    let subscribe_msg = json!({
        "type": "subscribe",
        "channel": "document",
        "document_id": doc_id
    });
    
    ws_stream.send(Message::Text(subscribe_msg.to_string())).await
        .expect("Failed to send subscribe message");
    
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    
    let unsubscribe_msg = json!({
        "type": "unsubscribe",
        "channel": "document",
        "document_id": doc_id
    });
    
    ws_stream.send(Message::Text(unsubscribe_msg.to_string())).await
        .expect("Failed to send unsubscribe message");
    
    let mut confirmed = false;
    let timeout = tokio::time::Duration::from_secs(5);
    let start = tokio::time::Instant::now();
    
    while start.elapsed() < timeout {
        match ws_stream.next().await {
            Some(Ok(Message::Text(text))) => {
                let response: serde_json::Value = serde_json::from_str(&text).unwrap();
                if response["type"] == "unsubscribed" {
                    confirmed = true;
                    break;
                }
            }
            Some(Ok(_)) => continue,
            _ => break,
        }
    }
    
    assert!(confirmed, "Should receive unsubscription confirmation");
    
    let _ = ws_stream.close(None).await;
}

#[tokio::test]
#[ignore]
async fn test_websocket_broadcast() {
    let url = get_server_url();
    
    let (mut ws1, _) = connect_async(&url).await.expect("Failed to connect ws1");
    let (mut ws2, _) = connect_async(&url).await.expect("Failed to connect ws2");
    
    let doc_id = uuid::Uuid::new_v4().to_string();
    
    let subscribe_msg = json!({
        "type": "subscribe",
        "channel": "document",
        "document_id": doc_id
    });
    
    ws1.send(Message::Text(subscribe_msg.to_string())).await.expect("Failed to subscribe ws1");
    ws2.send(Message::Text(subscribe_msg.to_string())).await.expect("Failed to subscribe ws2");
    
    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
    
    let broadcast_msg = json!({
        "type": "broadcast",
        "channel": "document",
        "document_id": doc_id,
        "data": {
            "action": "update",
            "content": "Test broadcast"
        }
    });
    
    ws1.send(Message::Text(broadcast_msg.to_string())).await
        .expect("Failed to send broadcast");
    
    let mut ws2_received = false;
    let timeout = tokio::time::Duration::from_secs(5);
    let start = tokio::time::Instant::now();
    
    while start.elapsed() < timeout {
        match ws2.next().await {
            Some(Ok(Message::Text(text))) => {
                let response: serde_json::Value = serde_json::from_str(&text).unwrap();
                if response["type"] == "broadcast" || response["data"]["action"] == "update" {
                    ws2_received = true;
                    break;
                }
            }
            Some(Ok(_)) => continue,
            _ => break,
        }
    }
    
    assert!(ws2_received, "WebSocket 2 should receive broadcast from WebSocket 1");
    
    let _ = ws1.close(None).await;
    let _ = ws2.close(None).await;
}

#[tokio::test]
#[ignore]
async fn test_websocket_invalid_message() {
    let url = get_server_url();
    let (mut ws_stream, _) = connect_async(&url).await
        .expect("Failed to connect to WebSocket");
    
    let invalid_msg = "not valid json";
    
    ws_stream.send(Message::Text(invalid_msg.to_string())).await
        .expect("Failed to send invalid message");
    
    let mut received_error = false;
    let timeout = tokio::time::Duration::from_secs(5);
    let start = tokio::time::Instant::now();
    
    while start.elapsed() < timeout {
        match ws_stream.next().await {
            Some(Ok(Message::Text(text))) => {
                let response: serde_json::Value = serde_json::from_str(&text).unwrap();
                if response["type"] == "error" {
                    received_error = true;
                    break;
                }
            }
            Some(Ok(_)) => continue,
            _ => break,
        }
    }
    
    assert!(received_error, "Should receive error response for invalid message");
    
    let _ = ws_stream.close(None).await;
}
