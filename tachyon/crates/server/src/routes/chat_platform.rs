use crate::error::ServerError;
use axum::{extract::State, response::Json, routing::post, Router};
use serde::Serialize;

#[derive(Clone)]
pub struct ChatPlatformState {
    pub dispatcher: Option<crate::integrations::chat_platforms::ChatPlatformDispatcher>,
}

#[derive(Debug, Serialize)]
pub struct ChatTestResponse {
    pub results: Vec<crate::integrations::chat_platforms::DeliveryResult>,
    pub message: String,
}

pub async fn test_chat_platforms(
    State(state): State<ChatPlatformState>,
) -> Result<Json<ChatTestResponse>, ServerError> {
    let dispatcher = match &state.dispatcher {
        Some(d) => d,
        None => {
            return Err(ServerError::bad_request(
                "No chat platforms configured. Set TACHYON_SLACK_WEBHOOK_URL or TACHYON_DISCORD_WEBHOOK_URL.",
            ));
        }
    };

    let results = dispatcher.send_test().await;

    let success_count = results.iter().filter(|r| r.success).count();
    let fail_count = results.len() - success_count;
    let message = if fail_count == 0 {
        format!(
            "Test notification sent successfully to {} platform(s).",
            success_count
        )
    } else {
        format!(
            "Test notification sent. {} succeeded, {} failed.",
            success_count, fail_count
        )
    };

    Ok(Json(ChatTestResponse {
        results,
        message,
    }))
}

pub fn create_chat_platform_router() -> Router<ChatPlatformState> {
    Router::new().route(
        "/admin/integrations/chat/test",
        post(test_chat_platforms),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chat_test_response_serialization() {
        let response = ChatTestResponse {
            results: vec![
                crate::integrations::chat_platforms::DeliveryResult {
                    platform: "slack".to_string(),
                    success: true,
                    status_code: Some(200),
                    error: None,
                },
            ],
            message: "Test notification sent successfully to 1 platform(s).".to_string(),
        };
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"success\":true"));
        assert!(json.contains("\"platform\":\"slack\""));
        assert!(json.contains("successfully"));
    }
}
