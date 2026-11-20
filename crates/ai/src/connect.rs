use crate::anthropic::AnthropicClient;
use crate::error::AiError;
use crate::openai::OpenAiClient;
use crate::provider::LlmProvider;

pub fn connect(provider: &str, api_key: &str) -> Result<Box<dyn LlmProvider>, AiError> {
    match provider {
        "claude" => Ok(Box::new(AnthropicClient::connect(api_key)?)),
        "openai" => Ok(Box::new(OpenAiClient::connect(api_key)?)),
        other => Err(AiError::UnknownProvider(other.to_string())),
    }
}
