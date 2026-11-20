use serde_json::json;

use crate::error::AiError;
use crate::provider::LlmProvider;
use crate::sse::{parse_model_ids, parse_openai_sse};
use crate::transport::{Method, ReqwestTransport, Transport};
use crate::types::{CompleteRequest, Completion};

const RESPONSES: &str = "https://api.openai.com/v1/responses";
const MODELS: &str = "https://api.openai.com/v1/models";

pub struct OpenAiClient<T> {
    transport: T,
    api_key: String,
    responses_url: String,
    models_url: String,
}

impl OpenAiClient<ReqwestTransport> {
    pub fn connect(api_key: impl Into<String>) -> Result<Self, AiError> {
        Ok(Self {
            transport: ReqwestTransport::new()?,
            api_key: api_key.into(),
            responses_url: RESPONSES.to_string(),
            models_url: MODELS.to_string(),
        })
    }
}

impl<T> OpenAiClient<T> {
    #[cfg(test)]
    pub fn with_transport(
        transport: T,
        api_key: impl Into<String>,
        responses_url: impl Into<String>,
        models_url: impl Into<String>,
    ) -> Self {
        Self {
            transport,
            api_key: api_key.into(),
            responses_url: responses_url.into(),
            models_url: models_url.into(),
        }
    }
}

impl<T: Transport> LlmProvider for OpenAiClient<T> {
    fn name(&self) -> &str {
        "openai"
    }

    fn list_models(&self) -> Result<Vec<String>, AiError> {
        let auth = format!("Bearer {}", self.api_key);
        let body = self.transport.request(
            Method::Get,
            &self.models_url,
            &[("authorization", auth.as_str())],
            None,
        )?;
        parse_model_ids(&body)
    }

    fn complete(&self, request: &CompleteRequest) -> Result<Completion, AiError> {
        let payload = openai_body(request)?;
        let auth = format!("Bearer {}", self.api_key);
        let body = self.transport.request(
            Method::Post,
            &self.responses_url,
            &[("authorization", auth.as_str())],
            Some(&payload),
        )?;
        parse_openai_sse(&body)
    }
}

pub fn openai_body(request: &CompleteRequest) -> Result<String, AiError> {
    if request.model.trim().is_empty() {
        return Err(AiError::ModelRequired);
    }
    if request.max_tokens == 0 {
        return Err(AiError::Provider("max_tokens is required".into()));
    }
    if request.messages.is_empty() {
        return Err(AiError::Provider("at least one message is required".into()));
    }
    let mut payload = json!({
        "model": request.model,
        "stream": true,
        "max_output_tokens": request.max_tokens,
        "input": request.messages.iter().map(|message| json!({
            "role": message.role,
            "content": message.content,
        })).collect::<Vec<_>>(),
    });
    if let Some(schema) = &request.schema {
        if schema.name.trim().is_empty() {
            return Err(AiError::Schema("schema name is required".into()));
        }
        payload["text"] = json!({
            "format": {
                "type": "json_schema",
                "name": schema.name,
                "strict": true,
                "schema": schema.schema,
            }
        });
    }
    serde_json::to_string(&payload).map_err(|error| AiError::Parse(error.to_string()))
}

#[cfg(test)]
mod tests {
    use super::openai_body;
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
                schema: serde_json::json!({
                    "type": "object",
                    "properties": {"ok": {"type": "boolean"}},
                    "required": ["ok"],
                    "additionalProperties": false
                }),
            }),
            max_tokens: 32,
        }
    }

    #[test]
    fn body_uses_responses_json_schema() {
        let body: serde_json::Value =
            serde_json::from_str(&openai_body(&request()).unwrap()).unwrap();
        assert_eq!(body["model"], "from-the-caller");
        assert_eq!(body["stream"], true);
        assert_eq!(body["text"]["format"]["type"], "json_schema");
        assert_eq!(body["text"]["format"]["strict"], true);
        assert_eq!(body["text"]["format"]["name"], "result");
    }

    #[test]
    fn client_parses_the_fixture_stream() {
        let transport = Scripted::new(vec![include_str!("../fixtures/openai.sse").into()]);
        let client = super::OpenAiClient::with_transport(
            transport,
            "sk-test-header-value",
            "https://example.test/responses",
            "https://example.test/models",
        );
        let completion = client.complete(&request()).unwrap();
        assert_eq!(completion.text, "{\"ok\":true}");
        let seen = client.transport.seen();
        assert_eq!(seen[0].method, Method::Post);
        assert!(seen[0].url.ends_with("/responses"));
        assert!(seen[0].headers[0].1.starts_with("Bearer "));
        assert!(seen[0].body.as_deref().unwrap().contains("json_schema"));
    }
}
