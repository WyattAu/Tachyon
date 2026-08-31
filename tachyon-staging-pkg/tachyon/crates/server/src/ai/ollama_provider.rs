use super::*;

pub struct OllamaProvider {
    client: reqwest::Client,
    base_url: String,
    default_model: String,
    embedding_model: String,
}

impl OllamaProvider {
    pub fn new(
        base_url: Option<String>,
        model: Option<String>,
        embedding_model: Option<String>,
    ) -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url: base_url.unwrap_or_else(|| "http://localhost:11434".to_string()),
            default_model: model.unwrap_or_else(|| "llama3".to_string()),
            embedding_model: embedding_model.unwrap_or_else(|| "nomic-embed-text".to_string()),
        }
    }
}

#[derive(Serialize)]
struct OllamaChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    options: Option<OllamaOptions>,
    stream: bool,
}

#[derive(Serialize)]
struct OllamaOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    num_predict: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
}

#[derive(Deserialize)]
struct OllamaChatResponse {
    message: OllamaMessageResponse,
    model: String,
    #[allow(dead_code)]
    done: bool,
}

#[derive(Deserialize)]
struct OllamaMessageResponse {
    content: String,
}

#[derive(Serialize)]
struct OllamaEmbeddingRequest {
    model: String,
    prompt: String,
}

#[derive(Deserialize)]
struct OllamaEmbeddingResponse {
    embedding: Vec<f32>,
}

#[async_trait::async_trait]
impl AiProvider for OllamaProvider {
    async fn chat_completion(&self, request: ChatRequest) -> Result<ChatResponse, AiError> {
        let url = format!("{}/api/chat", self.base_url);

        let options = if request.max_tokens.is_some() || request.temperature.is_some() {
            Some(OllamaOptions {
                num_predict: request.max_tokens,
                temperature: request.temperature,
            })
        } else {
            None
        };

        let body = OllamaChatRequest {
            model: request.model.unwrap_or_else(|| self.default_model.clone()),
            messages: request.messages,
            options,
            stream: false,
        };

        let resp = self
            .client
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| AiError::HttpError(e.to_string()))?;

        let status = resp.status();
        if !status.is_success() {
            let text = resp.text().await.unwrap_or_default();
            return Err(AiError::HttpError(format!("{}: {}", status, text)));
        }

        let chat_resp: OllamaChatResponse = resp
            .json()
            .await
            .map_err(|e| AiError::HttpError(e.to_string()))?;

        Ok(ChatResponse {
            content: chat_resp.message.content,
            model: chat_resp.model,
            usage: TokenUsage {
                prompt_tokens: 0,
                completion_tokens: 0,
                total_tokens: 0,
            },
            finish_reason: Some("stop".to_string()),
        })
    }

    async fn generate_embedding(&self, text: &str) -> Result<Vec<f32>, AiError> {
        let url = format!("{}/api/embeddings", self.base_url);
        let body = OllamaEmbeddingRequest {
            model: self.embedding_model.clone(),
            prompt: text.to_string(),
        };

        let resp = self
            .client
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| AiError::HttpError(e.to_string()))?;

        let status = resp.status();
        if !status.is_success() {
            let text = resp.text().await.unwrap_or_default();
            return Err(AiError::HttpError(format!("{}: {}", status, text)));
        }

        let emb_resp: OllamaEmbeddingResponse = resp
            .json()
            .await
            .map_err(|e| AiError::HttpError(e.to_string()))?;

        Ok(emb_resp.embedding)
    }

    fn name(&self) -> &str {
        "ollama"
    }

    fn available_models(&self) -> Vec<String> {
        vec![
            "llama3".to_string(),
            "mistral".to_string(),
            "nomic-embed-text".to_string(),
        ]
    }
}
