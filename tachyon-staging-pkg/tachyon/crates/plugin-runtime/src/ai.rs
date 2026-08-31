//! AI assistant plugin interface.
//! Provides a standard interface for AI plugins loaded via WASM.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiCapability {
    pub name: String,
    pub version: String,
    pub capabilities: Vec<AiCapabilityType>,
    pub max_context_tokens: u32,
    pub supported_models: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum AiCapabilityType {
    ChatCompletion,
    EmbeddingGeneration,
    Summarization,
    CodeCompletion,
    SemanticSearch,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiRequest {
    pub prompt: String,
    pub context: Option<String>,
    pub max_tokens: Option<u32>,
    pub temperature: Option<f32>,
    pub model: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiResponse {
    pub content: String,
    pub tokens_used: u32,
    pub model: String,
    pub finish_reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Embedding {
    pub vector: Vec<f32>,
    pub model: String,
    pub dimensions: usize,
}

#[derive(Debug, thiserror::Error)]
pub enum AiError {
    #[error("Plugin not loaded: {0}")]
    PluginNotLoaded(String),
    #[error("Model not available: {0}")]
    ModelNotAvailable(String),
    #[error("Context too long: {0} tokens exceeds limit of {1}")]
    ContextTooLong(u32, u32),
    #[error("AI request failed: {0}")]
    RequestFailed(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capability_serialization() {
        let cap = AiCapability {
            name: "test-ai".to_string(),
            version: "1.0.0".to_string(),
            capabilities: vec![
                AiCapabilityType::ChatCompletion,
                AiCapabilityType::EmbeddingGeneration,
            ],
            max_context_tokens: 4096,
            supported_models: vec!["gpt-4".to_string()],
        };
        let json = serde_json::to_string(&cap).unwrap();
        let parsed: AiCapability = serde_json::from_str(&json).unwrap();
        assert_eq!(cap.name, parsed.name);
        assert_eq!(cap.capabilities, parsed.capabilities);
        assert_eq!(cap.max_context_tokens, parsed.max_context_tokens);
    }

    #[test]
    fn test_capability_type_serde() {
        let chat = AiCapabilityType::ChatCompletion;
        let json = serde_json::to_string(&chat).unwrap();
        assert_eq!(json, "\"chat_completion\"");
        let parsed: AiCapabilityType = serde_json::from_str(&json).unwrap();
        assert_eq!(chat, parsed);
    }

    #[test]
    fn test_ai_request_optional_fields() {
        let req = AiRequest {
            prompt: "hello".to_string(),
            context: None,
            max_tokens: None,
            temperature: None,
            model: None,
        };
        let json = serde_json::to_string(&req).unwrap();
        let parsed: AiRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.prompt, "hello");
        assert!(parsed.context.is_none());
        assert!(parsed.max_tokens.is_none());
    }

    #[test]
    fn test_ai_response_roundtrip() {
        let resp = AiResponse {
            content: "Hello!".to_string(),
            tokens_used: 42,
            model: "gpt-4".to_string(),
            finish_reason: "stop".to_string(),
        };
        let json = serde_json::to_string(&resp).unwrap();
        let parsed: AiResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.content, "Hello!");
        assert_eq!(parsed.tokens_used, 42);
        assert_eq!(parsed.finish_reason, "stop");
    }

    #[test]
    fn test_embedding() {
        let emb = Embedding {
            vector: vec![0.1, 0.2, 0.3],
            model: "text-embedding-3-small".to_string(),
            dimensions: 3,
        };
        assert_eq!(emb.dimensions, emb.vector.len());
        let json = serde_json::to_string(&emb).unwrap();
        let parsed: Embedding = serde_json::from_str(&json).unwrap();
        assert_eq!(emb.vector, parsed.vector);
        assert_eq!(emb.model, parsed.model);
    }

    #[test]
    fn test_ai_error_display() {
        let err = AiError::PluginNotLoaded("my-plugin".to_string());
        assert_eq!(err.to_string(), "Plugin not loaded: my-plugin");

        let err = AiError::ContextTooLong(5000, 4096);
        assert_eq!(
            err.to_string(),
            "Context too long: 5000 tokens exceeds limit of 4096"
        );

        let err = AiError::ModelNotAvailable("gpt-5".to_string());
        assert_eq!(err.to_string(), "Model not available: gpt-5");
    }
}
