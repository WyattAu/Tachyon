use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct WebSocketMessage {
    #[serde(rename = "type")]
    msg_type: String,
    channel: Option<String>,
    data: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
struct DocumentUpdate {
    document_id: String,
    content: String,
    version: u64,
}

#[tokio::test]
async fn test_websocket_message_serialization() {
    let msg = WebSocketMessage {
        msg_type: "ping".to_string(),
        channel: None,
        data: None,
    };
    
    let json = serde_json::to_string(&msg).unwrap();
    assert!(json.contains("ping"));
    
    let decoded: WebSocketMessage = serde_json::from_str(&json).unwrap();
    assert_eq!(decoded.msg_type, "ping");
}

#[tokio::test]
async fn test_websocket_subscribe_message() {
    let msg = WebSocketMessage {
        msg_type: "subscribe".to_string(),
        channel: Some("document".to_string()),
        data: Some(serde_json::json!({"document_id": "doc-123"})),
    };
    
    let json = serde_json::to_string(&msg).unwrap();
    assert!(json.contains("subscribe"));
    assert!(json.contains("document"));
    
    let decoded: WebSocketMessage = serde_json::from_str(&json).unwrap();
    assert_eq!(decoded.msg_type, "subscribe");
    assert_eq!(decoded.channel, Some("document".to_string()));
}

#[tokio::test]
async fn test_websocket_unsubscribe_message() {
    let msg = WebSocketMessage {
        msg_type: "unsubscribe".to_string(),
        channel: Some("document".to_string()),
        data: Some(serde_json::json!({"document_id": "doc-123"})),
    };
    
    let json = serde_json::to_string(&msg).unwrap();
    let decoded: WebSocketMessage = serde_json::from_str(&json).unwrap();
    
    assert_eq!(decoded.msg_type, "unsubscribe");
}

#[tokio::test]
async fn test_websocket_broadcast_message() {
    let msg = WebSocketMessage {
        msg_type: "broadcast".to_string(),
        channel: Some("document".to_string()),
        data: Some(serde_json::json!({
            "document_id": "doc-123",
            "action": "update",
            "content": "Updated content"
        })),
    };
    
    let json = serde_json::to_string(&msg).unwrap();
    let decoded: WebSocketMessage = serde_json::from_str(&json).unwrap();
    
    assert_eq!(decoded.msg_type, "broadcast");
    assert!(decoded.data.is_some());
}

#[tokio::test]
async fn test_document_update_message() {
    let update = DocumentUpdate {
        document_id: "doc-456".to_string(),
        content: "New content".to_string(),
        version: 5,
    };
    
    let json = serde_json::to_string(&update).unwrap();
    let decoded: DocumentUpdate = serde_json::from_str(&json).unwrap();
    
    assert_eq!(decoded.document_id, "doc-456");
    assert_eq!(decoded.content, "New content");
    assert_eq!(decoded.version, 5);
}

#[tokio::test]
async fn test_websocket_error_message() {
    let msg = WebSocketMessage {
        msg_type: "error".to_string(),
        channel: None,
        data: Some(serde_json::json!({
            "code": "INVALID_MESSAGE",
            "message": "Could not parse message"
        })),
    };
    
    let json = serde_json::to_string(&msg).unwrap();
    let decoded: WebSocketMessage = serde_json::from_str(&json).unwrap();
    
    assert_eq!(decoded.msg_type, "error");
}

#[tokio::test]
async fn test_websocket_pong_message() {
    let msg = WebSocketMessage {
        msg_type: "pong".to_string(),
        channel: None,
        data: None,
    };
    
    let json = serde_json::to_string(&msg).unwrap();
    let decoded: WebSocketMessage = serde_json::from_str(&json).unwrap();
    
    assert_eq!(decoded.msg_type, "pong");
}

#[tokio::test]
async fn test_channel_types() {
    let channels = vec!["document", "project", "search", "system"];
    
    for channel in channels {
        let msg = WebSocketMessage {
            msg_type: "subscribe".to_string(),
            channel: Some(channel.to_string()),
            data: None,
        };
        
        let json = serde_json::to_string(&msg).unwrap();
        let decoded: WebSocketMessage = serde_json::from_str(&json).unwrap();
        
        assert_eq!(decoded.channel, Some(channel.to_string()));
    }
}

#[tokio::test]
async fn test_message_with_complex_data() {
    let complex_data = serde_json::json!({
        "document": {
            "id": "doc-789",
            "title": "Test Document",
            "content": "# Heading\n\nContent here",
            "metadata": {
                "author": "user-1",
                "tags": ["test", "websocket"]
            }
        },
        "operations": [
            {"type": "insert", "position": 10, "text": "new text"},
            {"type": "delete", "position": 5, "length": 3}
        ]
    });
    
    let msg = WebSocketMessage {
        msg_type: "update".to_string(),
        channel: Some("document".to_string()),
        data: Some(complex_data.clone()),
    };
    
    let json = serde_json::to_string(&msg).unwrap();
    let decoded: WebSocketMessage = serde_json::from_str(&json).unwrap();
    
    assert!(decoded.data.is_some());
    let data = decoded.data.unwrap();
    assert_eq!(data["document"]["id"], "doc-789");
    assert_eq!(data["operations"].as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn test_invalid_message_deserialization() {
    let invalid_json = r#"{"type": 123}"#;
    
    let result: Result<WebSocketMessage, _> = serde_json::from_str(invalid_json);
    assert!(result.is_err());
}

#[tokio::test]
async fn test_missing_required_field() {
    let missing_type = r#"{"channel": "document"}"#;
    
    let result: Result<WebSocketMessage, _> = serde_json::from_str(missing_type);
    assert!(result.is_err());
}

#[tokio::test]
async fn test_empty_message() {
    let empty = r#"{}"#;
    
    let result: Result<WebSocketMessage, _> = serde_json::from_str(empty);
    assert!(result.is_err());
}

mod operational_transform {
    use super::*;
    
    #[derive(Debug, Serialize, Deserialize)]
    struct TextOperation {
        #[serde(rename = "type")]
        op_type: String,
        position: usize,
        #[serde(skip_serializing_if = "Option::is_none")]
        text: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        length: Option<usize>,
    }
    
    #[tokio::test]
    async fn test_insert_operation() {
        let op = TextOperation {
            op_type: "insert".to_string(),
            position: 10,
            text: Some("new text".to_string()),
            length: None,
        };
        
        let json = serde_json::to_string(&op).unwrap();
        let decoded: TextOperation = serde_json::from_str(&json).unwrap();
        
        assert_eq!(decoded.op_type, "insert");
        assert_eq!(decoded.position, 10);
        assert_eq!(decoded.text, Some("new text".to_string()));
    }
    
    #[tokio::test]
    async fn test_delete_operation() {
        let op = TextOperation {
            op_type: "delete".to_string(),
            position: 5,
            text: None,
            length: Some(3),
        };
        
        let json = serde_json::to_string(&op).unwrap();
        let decoded: TextOperation = serde_json::from_str(&json).unwrap();
        
        assert_eq!(decoded.op_type, "delete");
        assert_eq!(decoded.length, Some(3));
        assert!(decoded.text.is_none());
    }
    
    #[tokio::test]
    async fn test_retain_operation() {
        let op = TextOperation {
            op_type: "retain".to_string(),
            position: 0,
            text: None,
            length: Some(10),
        };
        
        let json = serde_json::to_string(&op).unwrap();
        assert!(json.contains("retain"));
    }
}
