use super::*;

pub struct AnthropicProvider {
    client: reqwest::Client,
    api_key: String,
    default_model: String,
}

impl AnthropicProvider {
    pub fn new(api_key: String, model: Option<String>) -> Self {
        Self {
            client: reqwest::Client::new(),
            api_key,
            default_model: model.unwrap_or_else(|| "claude-sonnet-4-20250514".to_string()),
        }
    }
}

#[derive(Serialize)]
struct AnthropicChatRequest {
    model: String,
    messages: Vec<AnthropicMessage>,
    max_tokens: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
}

#[derive(Serialize, Deserialize)]
struct AnthropicMessage {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct AnthropicChatResponse {
    content: Vec<AnthropicContentBlock>,
    model: String,
    usage: AnthropicUsage,
    stop_reason: Option<String>,
}

#[derive(Deserialize)]
struct AnthropicContentBlock {
    text: Option<String>,
    #[allow(dead_code)]
    r#type: Option<String>,
}

#[derive(Deserialize)]
struct AnthropicUsage {
    input_tokens: u32,
    output_tokens: u32,
}

#[async_trait::async_trait]
impl AiProvider for AnthropicProvider {
    async fn chat_completion(&self, request: ChatRequest) -> Result<ChatResponse, AiError> {
        let url = "https://api.anthropic.com/v1/messages";
        let messages: Vec<AnthropicMessage> = request
            .messages
            .into_iter()
            .filter(|m| m.role != "system")
            .map(|m| AnthropicMessage {
                role: m.role,
                content: m.content,
            })
            .collect();

        let body = AnthropicChatRequest {
            model: request.model.unwrap_or_else(|| self.default_model.clone()),
            messages,
            max_tokens: request.max_tokens.unwrap_or(4096),
            temperature: request.temperature,
        };

        let resp = self
            .client
            .post(url)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| AiError::HttpError(e.to_string()))?;

        let status = resp.status();
        if status == reqwest::StatusCode::UNAUTHORIZED {
            return Err(AiError::AuthenticationFailed(
                "Invalid Anthropic API key".to_string(),
            ));
        }
        if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
            return Err(AiError::RateLimited(
                "Anthropic rate limit exceeded".to_string(),
            ));
        }
        if !status.is_success() {
            let text = resp.text().await.unwrap_or_default();
            return Err(AiError::HttpError(format!("{}: {}", status, text)));
        }

        let chat_resp: AnthropicChatResponse = resp
            .json()
            .await
            .map_err(|e| AiError::HttpError(e.to_string()))?;

        let content = chat_resp
            .content
            .iter()
            .filter_map(|b| b.text.clone())
            .collect::<Vec<_>>()
            .join("");

        Ok(ChatResponse {
            content,
            model: chat_resp.model,
            usage: TokenUsage {
                prompt_tokens: chat_resp.usage.input_tokens,
                completion_tokens: chat_resp.usage.output_tokens,
                total_tokens: chat_resp.usage.input_tokens + chat_resp.usage.output_tokens,
            },
            finish_reason: chat_resp.stop_reason,
        })
    }

    async fn generate_embedding(&self, _text: &str) -> Result<Vec<f32>, AiError> {
        Err(AiError::InvalidRequest(
            "Anthropic does not provide embeddings directly".to_string(),
        ))
    }

    fn name(&self) -> &str {
        "anthropic"
    }

    fn available_models(&self) -> Vec<String> {
        vec![
            "claude-sonnet-4-20250514".to_string(),
            "claude-3-5-sonnet-20241022".to_string(),
            "claude-3-haiku-20240307".to_string(),
        ]
    }
}
