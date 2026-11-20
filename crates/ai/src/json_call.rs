use serde::de::DeserializeOwned;

use crate::error::AiError;
use crate::provider::LlmProvider;
use crate::types::{CompleteRequest, Message};

pub fn complete_json<T: DeserializeOwned>(
    provider: &dyn LlmProvider,
    mut request: CompleteRequest,
) -> Result<T, AiError> {
    if request.schema.is_none() {
        return Err(AiError::Schema("a JSON schema is required".into()));
    }
    let first = provider.complete(&request)?;
    match serde_json::from_str::<T>(&first.text) {
        Ok(value) => Ok(value),
        Err(error) => {
            let note = format!(
                "Validation failed: {error}. The reply was: {}. Reply with JSON only.",
                first.text
            );
            if let Some(last) = request.messages.last_mut()
                && last.role == "user"
            {
                last.content.push_str("\n\n");
                last.content.push_str(&note);
            } else {
                request.messages.push(Message {
                    role: "user".into(),
                    content: note,
                });
            }
            let second = provider.complete(&request)?;
            serde_json::from_str(&second.text).map_err(|error| AiError::Schema(error.to_string()))
        }
    }
}

#[cfg(test)]
mod tests {
    use serde::Deserialize;

    use super::complete_json;
    use crate::mock::MockProvider;
    use crate::types::{CompleteRequest, Completion, JsonSchema, Message, TokenUsage};

    #[derive(Debug, Deserialize, PartialEq)]
    struct OkFlag {
        ok: bool,
    }

    fn request() -> CompleteRequest {
        CompleteRequest {
            model: "from-the-caller".into(),
            messages: vec![Message {
                role: "user".into(),
                content: "Answer.".into(),
            }],
            schema: Some(JsonSchema {
                name: "result".into(),
                schema: serde_json::json!({"type": "object"}),
            }),
            max_tokens: 16,
        }
    }

    fn completion(text: &str) -> Completion {
        Completion {
            text: text.into(),
            deltas: vec![text.into()],
            usage: TokenUsage {
                input: 1,
                output: 1,
            },
        }
    }

    #[test]
    fn retry_includes_the_validation_error_once() {
        let invalid = "nope";
        let validation = serde_json::from_str::<OkFlag>(invalid).unwrap_err();
        let mock = MockProvider::script(vec![
            Ok(completion(invalid)),
            Ok(completion(r#"{"ok":true}"#)),
        ]);
        let parsed = complete_json::<OkFlag>(&mock, request()).unwrap();
        assert_eq!(parsed, OkFlag { ok: true });
        assert_eq!(mock.calls(), 2);
        let requests = mock.requests();
        let second = requests[1].messages.last().unwrap();
        assert!(second.content.contains(&validation.to_string()));
        assert!(second.content.contains(invalid));
    }

    #[test]
    fn second_failure_does_not_retry_again() {
        let mock = MockProvider::script(vec![Ok(completion("nope")), Ok(completion("still nope"))]);
        let error = complete_json::<OkFlag>(&mock, request()).unwrap_err();
        assert_eq!(mock.calls(), 2);
        assert!(error.to_string().contains("schema"));
    }
}
