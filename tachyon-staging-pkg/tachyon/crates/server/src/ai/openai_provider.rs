use super::*;

pub struct OpenAiProvider {
    client: reqwest::Client,
    api_key: String,
    base_url: String,
    default_model: String,
    embedding_model: String,
}

impl OpenAiProvider {
    pub fn new(
        api_key: String,
        base_url: Option<String>,
        model: Option<String>,
        embedding_model: Option<String>,
    ) -> Self {
        Self {
            client: reqwest::Client::new(),
            api_key,
            base_url: base_url.unwrap_or_else(|| "https://api.openai.com".to_string()),
            default_model: model.unwrap_or_else(|| "gpt-4o".to_string()),
            embedding_model: embedding_model
                .unwrap_or_else(|| "text-embedding-3-small".to_string()),
        }
    }
}

#[derive(Serialize)]
struct OpenAiChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
}

#[derive(Deserialize)]
struct OpenAiChatResponse {
    choices: Vec<OpenAiChoice>,
    model: String,
    usage: OpenAiUsage,
}

#[derive(Deserialize)]
struct OpenAiChoice {
    message: OpenAiMessage,
    finish_reason: Option<String>,
}

#[derive(Deserialize)]
struct OpenAiMessage {
    content: String,
}

#[derive(Deserialize)]
struct OpenAiUsage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}

#[derive(Serialize)]
struct OpenAiEmbeddingRequest {
    model: String,
    input: String,
}

#[derive(Deserialize)]
struct OpenAiEmbeddingResponse {
    data: Vec<OpenAiEmbeddingData>,
    #[allow(dead_code)]
    model: String,
    #[allow(dead_code)]
    usage: OpenAiUsage,
}

#[derive(Deserialize)]
struct OpenAiEmbeddingData {
    embedding: Vec<f32>,
}

#[async_trait::async_trait]
impl AiProvider for OpenAiProvider {
    async fn chat_completion(&self, request: ChatRequest) -> Result<ChatResponse, AiError> {
        let url = format!("{}/v1/chat/completions", self.base_url);
        let body = OpenAiChatRequest {
            model: request.model.unwrap_or_else(|| self.default_model.clone()),
            messages: request.messages,
            max_tokens: request.max_tokens,
            temperature: request.temperature,
        };

        let resp = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&body)
            .send()
            .await
            .map_err(|e| AiError::HttpError(e.to_string()))?;

        let status = resp.status();
        if status == reqwest::StatusCode::UNAUTHORIZED {
            return Err(AiError::AuthenticationFailed("Invalid API key".to_string()));
        }
        if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
            return Err(AiError::RateLimited(
                "OpenAI rate limit exceeded".to_string(),
            ));
        }
        if !status.is_success() {
            let text = resp.text().await.unwrap_or_default();
            return Err(AiError::HttpError(format!("{}: {}", status, text)));
        }

        let chat_resp: OpenAiChatResponse = resp
            .json()
            .await
            .map_err(|e| AiError::HttpError(e.to_string()))?;

        let choice = chat_resp
            .choices
            .into_iter()
            .next()
            .ok_or_else(|| AiError::InvalidRequest("No response from OpenAI".to_string()))?;

        Ok(ChatResponse {
            content: choice.message.content,
            model: chat_resp.model,
            usage: TokenUsage {
                prompt_tokens: chat_resp.usage.prompt_tokens,
                completion_tokens: chat_resp.usage.completion_tokens,
                total_tokens: chat_resp.usage.total_tokens,
            },
            finish_reason: choice.finish_reason,
        })
    }

    async fn generate_embedding(&self, text: &str) -> Result<Vec<f32>, AiError> {
        let url = format!("{}/v1/embeddings", self.base_url);
        let body = OpenAiEmbeddingRequest {
            model: self.embedding_model.clone(),
            input: text.to_string(),
        };

        let resp = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&body)
            .send()
            .await
            .map_err(|e| AiError::HttpError(e.to_string()))?;

        let status = resp.status();
        if !status.is_success() {
            let text = resp.text().await.unwrap_or_default();
            return Err(AiError::HttpError(format!("{}: {}", status, text)));
        }

        let emb_resp: OpenAiEmbeddingResponse = resp
            .json()
            .await
            .map_err(|e| AiError::HttpError(e.to_string()))?;

        emb_resp
            .data
            .into_iter()
            .next()
            .map(|d| d.embedding)
            .ok_or_else(|| AiError::InvalidRequest("No embedding returned".to_string()))
    }

    fn name(&self) -> &str {
        "openai"
    }

    fn available_models(&self) -> Vec<String> {
        vec![
            "gpt-4o".to_string(),
            "gpt-4o-mini".to_string(),
            "gpt-4-turbo".to_string(),
            "text-embedding-3-small".to_string(),
            "text-embedding-3-large".to_string(),
        ]
    }
}
