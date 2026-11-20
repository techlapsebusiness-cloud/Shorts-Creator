use serde_json::{Value, json};

use crate::ANTHROPIC_VERSION;
use crate::error::AiError;
use crate::provider::LlmProvider;
use crate::sse::{parse_anthropic_sse, parse_model_ids};
use crate::transport::{Method, ReqwestTransport, Transport};
use crate::types::{CompleteRequest, Completion};

const MESSAGES: &str = "https://api.anthropic.com/v1/messages";
const MODELS: &str = "https://api.anthropic.com/v1/models";

pub struct AnthropicClient<T> {
    transport: T,
    api_key: String,
    messages_url: String,
    models_url: String,
}

impl AnthropicClient<ReqwestTransport> {
    pub fn connect(api_key: impl Into<String>) -> Result<Self, AiError> {
        Ok(Self {
            transport: ReqwestTransport::new()?,
            api_key: api_key.into(),
            messages_url: MESSAGES.to_string(),
            models_url: MODELS.to_string(),
        })
    }
}

impl<T> AnthropicClient<T> {
    #[cfg(test)]
    pub fn with_transport(
        transport: T,
        api_key: impl Into<String>,
        messages_url: impl Into<String>,
        models_url: impl Into<String>,
    ) -> Self {
        Self {
            transport,
            api_key: api_key.into(),
            messages_url: messages_url.into(),
            models_url: models_url.into(),
        }
    }
}

impl<T: Transport> LlmProvider for AnthropicClient<T> {
    fn name(&self) -> &str {
        "claude"
    }

    fn list_models(&self) -> Result<Vec<String>, AiError> {
        let body = self.transport.request(
            Method::Get,
            &self.models_url,
            &[
                ("x-api-key", self.api_key.as_str()),
                ("anthropic-version", ANTHROPIC_VERSION),
            ],
            None,
        )?;
        parse_model_ids(&body)
    }

    fn complete(&self, request: &CompleteRequest) -> Result<Completion, AiError> {
        let payload = anthropic_body(request)?;
        let body = self.transport.request(
            Method::Post,
            &self.messages_url,
            &[
                ("x-api-key", self.api_key.as_str()),
                ("anthropic-version", ANTHROPIC_VERSION),
            ],
            Some(&payload),
        )?;
        parse_anthropic_sse(&body)
    }
}

pub fn anthropic_body(request: &CompleteRequest) -> Result<String, AiError> {
    validate(request)?;
    let mut payload = json!({
        "model": request.model,
        "max_tokens": request.max_tokens,
        "stream": true,
        "messages": request.messages.iter().map(|message| json!({
            "role": message.role,
            "content": message.content,
        })).collect::<Vec<_>>(),
    });
    if let Some(schema) = &request.schema {
        payload["system"] =
            Value::String(format!("Reply with JSON only. Schema: {}", schema.schema));
    }
    serde_json::to_string(&payload).map_err(|error| AiError::Parse(error.to_string()))
}

fn validate(request: &CompleteRequest) -> Result<(), AiError> {
    if request.model.trim().is_empty() {
        return Err(AiError::ModelRequired);
    }
    if request.max_tokens == 0 {
        return Err(AiError::Provider("max_tokens is required".into()));
    }
    if request.messages.is_empty() {
        return Err(AiError::Provider("at least one message is required".into()));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::anthropic_body;
    use crate::ANTHROPIC_VERSION;
    use crate::provider::LlmProvider;
    use crate::transport::{Method, Scripted};
    use crate::types::{CompleteRequest, JsonSchema, Message};

    fn request() -> CompleteRequest {
        CompleteRequest {
            model: "from-the-caller".into(),
            messages: vec![Message {
                role: "user".into(),
                content: "Reply with JSON.".into(),
            }],
            schema: Some(JsonSchema {
                name: "result".into(),
                schema: serde_json::json!({"type": "object"}),
            }),
            max_tokens: 32,
        }
    }

    #[test]
    fn body_uses_the_caller_model_and_streams() {
        let body: serde_json::Value =
            serde_json::from_str(&anthropic_body(&request()).unwrap()).unwrap();
        assert_eq!(body["model"], "from-the-caller");
        assert_eq!(body["stream"], true);
        assert!(body["system"].as_str().unwrap().contains("Schema"));
    }

    #[test]
    fn empty_model_is_rejected() {
        let mut request = request();
        request.model.clear();
        assert!(anthropic_body(&request).is_err());
    }

    #[test]
    fn client_sends_the_version_header_and_parses_the_fixture() {
        let key = "sk-ant-test-header-value";
        let transport = Scripted::new(vec![
            r#"{"data":[{"id":"model-from-payload"}]}"#.into(),
            include_str!("../fixtures/anthropic.sse").into(),
        ]);
        let client = super::AnthropicClient::with_transport(
            transport,
            key,
            "https://example.test/messages",
            "https://example.test/models",
        );
        let models = client.list_models().unwrap();
        assert_eq!(models, vec!["model-from-payload"]);
        let completion = client.complete(&request()).unwrap();
        assert_eq!(completion.deltas.len(), 2);
        let seen = client.transport.seen();
        assert_eq!(seen[0].method, Method::Get);
        assert!(seen[0].url.ends_with("/models"));
        assert_eq!(seen[1].method, Method::Post);
        assert_eq!(seen[0].headers[1].1, ANTHROPIC_VERSION);
        assert_eq!(seen[1].headers[0].1, key);
        assert!(seen[1].body.as_deref().unwrap().contains("from-the-caller"));
    }
}
