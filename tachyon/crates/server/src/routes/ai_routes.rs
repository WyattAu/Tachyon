use crate::ai::{AiError, AiManager, ChatMessage, ChatRequest};
use axum::{extract::State, http::StatusCode, response::Json};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct CompleteRequest {
    pub prompt: String,
    #[serde(default = "default_max_tokens")]
    pub max_tokens: u32,
    #[serde(default = "default_temperature")]
    pub temperature: f32,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct CompleteResponse {
    pub completion: String,
    pub model: String,
    pub usage: TokenUsageResponse,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct TokenUsageResponse {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct SummarizeRequest {
    pub text: String,
    #[serde(default = "default_max_tokens")]
    pub max_tokens: u32,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct SummarizeResponse {
    pub summary: String,
    pub model: String,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct ImproveRequest {
    pub text: String,
    #[serde(default = "default_max_tokens")]
    pub max_tokens: u32,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct ImproveResponse {
    pub improved: String,
    pub model: String,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct TagsRequest {
    pub text: String,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct TagsResponse {
    pub tags: Vec<String>,
    pub model: String,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct EmbedRequest {
    pub text: String,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct EmbedResponse {
    pub embedding: Vec<f32>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct QuestionRequest {
    pub question: String,
    pub document_ids: Option<Vec<String>>,
    #[serde(default = "default_max_tokens")]
    pub max_tokens: u32,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct QuestionResponse {
    pub answer: String,
    pub sources: Vec<String>,
    pub model: String,
}

fn default_max_tokens() -> u32 {
    1024
}

fn default_temperature() -> f32 {
    0.7
}

type AiState = Arc<AiManager>;
type ApiError = (StatusCode, Json<serde_json::Value>);

fn ai_unavailable() -> ApiError {
    (
        StatusCode::SERVICE_UNAVAILABLE,
        Json(serde_json::json!({
            "code": "AI_UNAVAILABLE",
            "message": "AI provider is not configured"
        })),
    )
}

fn ai_error(err: AiError) -> ApiError {
    let status = match &err {
        AiError::RateLimited(_) => StatusCode::TOO_MANY_REQUESTS,
        AiError::InvalidRequest(_) => StatusCode::BAD_REQUEST,
        AiError::AuthenticationFailed(_) => StatusCode::UNAUTHORIZED,
        _ => StatusCode::INTERNAL_SERVER_ERROR,
    };
    (
        status,
        Json(serde_json::json!({
            "code": "AI_ERROR",
            "message": err.to_string()
        })),
    )
}

#[utoipa::path(
    post,
    path = "/ai/complete",
    request_body = CompleteRequest,
    responses(
        (status = 200, description = "Completion result", body = CompleteResponse),
        (status = 503, description = "AI not configured"),
    ),
    tag = "ai",
    security(("bearer_auth" = [])),
)]
pub async fn complete(
    State(manager): State<AiState>,
    Json(req): Json<CompleteRequest>,
) -> Result<Json<CompleteResponse>, ApiError> {
    if !manager.is_available() {
        return Err(ai_unavailable());
    }

    let chat_req = ChatRequest {
        messages: vec![ChatMessage {
            role: "user".to_string(),
            content: req.prompt,
        }],
        model: None,
        max_tokens: Some(req.max_tokens),
        temperature: Some(req.temperature),
    };

    let resp = manager.chat(chat_req).await.map_err(ai_error)?;

    Ok(Json(CompleteResponse {
        completion: resp.content,
        model: resp.model,
        usage: TokenUsageResponse {
            prompt_tokens: resp.usage.prompt_tokens,
            completion_tokens: resp.usage.completion_tokens,
            total_tokens: resp.usage.total_tokens,
        },
    }))
}

#[utoipa::path(
    post,
    path = "/ai/summarize",
    request_body = SummarizeRequest,
    responses(
        (status = 200, description = "Summary result", body = SummarizeResponse),
        (status = 503, description = "AI not configured"),
    ),
    tag = "ai",
    security(("bearer_auth" = [])),
)]
pub async fn summarize(
    State(manager): State<AiState>,
    Json(req): Json<SummarizeRequest>,
) -> Result<Json<SummarizeResponse>, ApiError> {
    if !manager.is_available() {
        return Err(ai_unavailable());
    }

    let chat_req = ChatRequest {
        messages: vec![ChatMessage {
            role: "user".to_string(),
            content: format!("Summarize the following text concisely:\n\n{}", req.text),
        }],
        model: None,
        max_tokens: Some(req.max_tokens),
        temperature: Some(0.3),
    };

    let resp = manager.chat(chat_req).await.map_err(ai_error)?;

    Ok(Json(SummarizeResponse {
        summary: resp.content,
        model: resp.model,
    }))
}

#[utoipa::path(
    post,
    path = "/ai/improve",
    request_body = ImproveRequest,
    responses(
        (status = 200, description = "Improved text", body = ImproveResponse),
        (status = 503, description = "AI not configured"),
    ),
    tag = "ai",
    security(("bearer_auth" = [])),
)]
pub async fn improve(
    State(manager): State<AiState>,
    Json(req): Json<ImproveRequest>,
) -> Result<Json<ImproveResponse>, ApiError> {
    if !manager.is_available() {
        return Err(ai_unavailable());
    }

    let chat_req = ChatRequest {
        messages: vec![ChatMessage {
            role: "user".to_string(),
            content: format!(
                "Improve the following text for clarity, grammar, and style. Return only the improved text:\n\n{}",
                req.text
            ),
        }],
        model: None,
        max_tokens: Some(req.max_tokens),
        temperature: Some(0.4),
    };

    let resp = manager.chat(chat_req).await.map_err(ai_error)?;

    Ok(Json(ImproveResponse {
        improved: resp.content,
        model: resp.model,
    }))
}

#[utoipa::path(
    post,
    path = "/ai/tags",
    request_body = TagsRequest,
    responses(
        (status = 200, description = "Suggested tags", body = TagsResponse),
        (status = 503, description = "AI not configured"),
    ),
    tag = "ai",
    security(("bearer_auth" = [])),
)]
pub async fn tags(
    State(manager): State<AiState>,
    Json(req): Json<TagsRequest>,
) -> Result<Json<TagsResponse>, ApiError> {
    if !manager.is_available() {
        return Err(ai_unavailable());
    }

    let chat_req = ChatRequest {
        messages: vec![ChatMessage {
            role: "user".to_string(),
            content: format!(
                "Generate relevant tags for the following text. Return ONLY a JSON array of lowercase tag strings, nothing else. Example: [\"tag1\", \"tag2\"]\n\n{}",
                req.text
            ),
        }],
        model: None,
        max_tokens: Some(256),
        temperature: Some(0.3),
    };

    let resp = manager.chat(chat_req).await.map_err(ai_error)?;

    let parsed_tags: Vec<String> = serde_json::from_str(resp.content.trim()).unwrap_or_else(|_| {
        resp.content
            .split(',')
            .flat_map(|s: &str| {
                s.trim()
                    .trim_matches(|c: char| c == '"' || c == '[' || c == ']' || c == '`')
                    .to_lowercase()
                    .split_whitespace()
                    .map(String::from)
                    .collect::<Vec<_>>()
            })
            .collect()
    });

    Ok(Json(TagsResponse {
        tags: parsed_tags,
        model: resp.model,
    }))
}

#[utoipa::path(
    post,
    path = "/ai/embed",
    request_body = EmbedRequest,
    responses(
        (status = 200, description = "Embedding vector", body = EmbedResponse),
        (status = 503, description = "AI not configured"),
    ),
    tag = "ai",
    security(("bearer_auth" = [])),
)]
pub async fn embed(
    State(manager): State<AiState>,
    Json(req): Json<EmbedRequest>,
) -> Result<Json<EmbedResponse>, ApiError> {
    if !manager.is_available() {
        return Err(ai_unavailable());
    }

    let embedding = manager.embed(&req.text).await.map_err(ai_error)?;

    Ok(Json(EmbedResponse { embedding }))
}

#[utoipa::path(
    post,
    path = "/ai/question",
    request_body = QuestionRequest,
    responses(
        (status = 200, description = "Answer with sources", body = QuestionResponse),
        (status = 503, description = "AI not configured"),
    ),
    tag = "ai",
    security(("bearer_auth" = [])),
)]
pub async fn question(
    State(manager): State<AiState>,
    Json(req): Json<QuestionRequest>,
) -> Result<Json<QuestionResponse>, ApiError> {
    if !manager.is_available() {
        return Err(ai_unavailable());
    }

    let context_note = match &req.document_ids {
        Some(ids) => format!(" (referencing documents: {})", ids.join(", ")),
        None => String::new(),
    };

    let chat_req = ChatRequest {
        messages: vec![ChatMessage {
            role: "user".to_string(),
            content: format!(
                "Answer the following question{}:\n\n{}",
                context_note, req.question
            ),
        }],
        model: None,
        max_tokens: Some(req.max_tokens),
        temperature: Some(0.5),
    };

    let resp = manager.chat(chat_req).await.map_err(ai_error)?;

    let sources = req.document_ids.unwrap_or_default();

    Ok(Json(QuestionResponse {
        answer: resp.content,
        sources,
        model: resp.model,
    }))
}

pub fn create_ai_router() -> axum::Router<Arc<AiManager>> {
    axum::Router::new()
        .route("/ai/complete", axum::routing::post(complete))
        .route("/ai/summarize", axum::routing::post(summarize))
        .route("/ai/improve", axum::routing::post(improve))
        .route("/ai/tags", axum::routing::post(tags))
        .route("/ai/embed", axum::routing::post(embed))
        .route("/ai/question", axum::routing::post(question))
}
