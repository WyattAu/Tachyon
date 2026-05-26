pub mod anthropic_provider;
pub mod manager;
pub mod ollama_provider;
pub mod openai_provider;

pub use manager::AiManager;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatRequest {
    pub messages: Vec<ChatMessage>,
    pub model: Option<String>,
    pub max_tokens: Option<u32>,
    pub temperature: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatResponse {
    pub content: String,
    pub model: String,
    pub usage: TokenUsage,
    pub finish_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingRequest {
    pub input: String,
    pub model: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingResponse {
    pub embedding: Vec<f32>,
    pub model: String,
    pub usage: TokenUsage,
}

#[derive(Debug, thiserror::Error)]
pub enum AiError {
    #[error("Provider unavailable: {0}")]
    ProviderUnavailable(String),
    #[error("Rate limited: {0}")]
    RateLimited(String),
    #[error("Invalid request: {0}")]
    InvalidRequest(String),
    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),
    #[error("HTTP error: {0}")]
    HttpError(String),
    #[error("No provider configured")]
    NoProvider,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiConfig {
    pub provider: String,
    pub api_key: Option<String>,
    pub model: Option<String>,
    pub base_url: Option<String>,
    pub embedding_model: Option<String>,
}

impl AiConfig {
    pub fn from_env() -> Self {
        Self {
            provider: std::env::var("TACHYON_AI_PROVIDER").unwrap_or_default(),
            api_key: std::env::var("TACHYON_AI_API_KEY").ok(),
            model: std::env::var("TACHYON_AI_MODEL").ok(),
            base_url: std::env::var("TACHYON_AI_BASE_URL").ok(),
            embedding_model: std::env::var("TACHYON_AI_EMBEDDING_MODEL").ok(),
        }
    }

    pub fn is_configured(&self) -> bool {
        !self.provider.is_empty()
    }
}

#[async_trait::async_trait]
pub trait AiProvider: Send + Sync {
    async fn chat_completion(&self, request: ChatRequest) -> Result<ChatResponse, AiError>;
    async fn generate_embedding(&self, text: &str) -> Result<Vec<f32>, AiError>;
    fn name(&self) -> &str;
    fn available_models(&self) -> Vec<String>;
}
