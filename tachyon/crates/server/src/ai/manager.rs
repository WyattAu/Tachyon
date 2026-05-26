use super::*;

pub struct AiManager {
    provider: Option<Box<dyn AiProvider>>,
    config: AiConfig,
}

impl AiManager {
    pub fn new(config: AiConfig) -> Self {
        let provider: Option<Box<dyn AiProvider>> = match config.provider.as_str() {
            "openai" => config.api_key.as_ref().map(|key| {
                Box::new(openai_provider::OpenAiProvider::new(
                    key.clone(),
                    config.base_url.clone(),
                    config.model.clone(),
                    config.embedding_model.clone(),
                )) as Box<dyn AiProvider>
            }),
            "anthropic" => config.api_key.as_ref().map(|key| {
                Box::new(anthropic_provider::AnthropicProvider::new(
                    key.clone(),
                    config.model.clone(),
                )) as Box<dyn AiProvider>
            }),
            "ollama" => Some(Box::new(ollama_provider::OllamaProvider::new(
                config.base_url.clone(),
                config.model.clone(),
                config.embedding_model.clone(),
            )) as Box<dyn AiProvider>),
            _ => None,
        };
        Self { provider, config }
    }

    pub fn from_env() -> Self {
        Self::new(AiConfig::from_env())
    }

    pub fn is_available(&self) -> bool {
        self.provider.is_some()
    }

    pub async fn chat(&self, request: ChatRequest) -> Result<ChatResponse, AiError> {
        match &self.provider {
            Some(p) => p.chat_completion(request).await,
            None => Err(AiError::NoProvider),
        }
    }

    pub async fn embed(&self, text: &str) -> Result<Vec<f32>, AiError> {
        match &self.provider {
            Some(p) => p.generate_embedding(text).await,
            None => Err(AiError::NoProvider),
        }
    }

    pub fn provider_name(&self) -> &str {
        match &self.provider {
            Some(p) => p.name(),
            None => "none",
        }
    }

    pub fn config(&self) -> &AiConfig {
        &self.config
    }
}
