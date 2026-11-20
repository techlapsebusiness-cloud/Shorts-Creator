use serde::Serialize;

use crate::config::default_model;
use crate::error::AiError;
use crate::provider::LlmProvider;
use crate::redact::redact;
use crate::secrets::SecretStore;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderTest {
    pub ok: bool,
    pub message: String,
    pub models: Vec<String>,
    pub default_model: String,
}

type ModelLister<'a> = &'a mut dyn FnMut(&str, &str) -> Result<Vec<String>, AiError>;

pub fn execute_test(
    provider: &str,
    key: &str,
    secrets: &mut dyn SecretStore,
    list_models: ModelLister<'_>,
) -> ProviderTest {
    if provider != "claude" && provider != "openai" {
        return failed(provider, key, format!("unknown provider {provider}"));
    }
    if !key.is_empty()
        && let Err(error) = secrets.set(provider, key)
    {
        return failed(provider, key, error.to_string());
    }
    let stored = match secrets.get(provider) {
        Ok(Some(value)) if !value.is_empty() => value,
        Ok(_) => {
            return ProviderTest {
                ok: false,
                message: "No key saved".into(),
                models: Vec::new(),
                default_model: default_model(provider).to_string(),
            };
        }
        Err(error) => return failed(provider, key, error.to_string()),
    };
    match list_models(provider, &stored) {
        Ok(models) => ProviderTest {
            ok: true,
            message: "Key works".into(),
            models,
            default_model: default_model(provider).to_string(),
        },
        Err(error) => {
            let message = clip(redact(&error.to_string(), &[key, stored.as_str()]));
            ProviderTest {
                ok: false,
                message,
                models: Vec::new(),
                default_model: default_model(provider).to_string(),
            }
        }
    }
}

fn failed(provider: &str, key: &str, message: String) -> ProviderTest {
    let stored = key;
    ProviderTest {
        ok: false,
        message: clip(redact(&message, &[key, stored])),
        models: Vec::new(),
        default_model: default_model(provider).to_string(),
    }
}

fn clip(message: String) -> String {
    const LIMIT: usize = 400;
    if message.chars().count() <= LIMIT {
        return message;
    }
    let mut clipped: String = message.chars().take(LIMIT).collect();
    clipped.push('…');
    clipped
}

pub fn list_models(provider: &str, api_key: &str) -> Result<Vec<String>, AiError> {
    match provider {
        "claude" => crate::anthropic::AnthropicClient::connect(api_key)?.list_models(),
        "openai" => crate::openai::OpenAiClient::connect(api_key)?.list_models(),
        other => Err(AiError::UnknownProvider(other.to_string())),
    }
}

#[cfg(test)]
mod tests {
    use super::execute_test;
    use crate::error::AiError;
    use crate::secrets::{MemoryStore, SecretStore};

    #[test]
    fn echoed_key_is_absent_from_the_result_and_from_project_json() {
        let key = "sk-ant-stored-example";
        let mut secrets = MemoryStore::default();
        let result = execute_test("claude", key, &mut secrets, &mut |_, stored| {
            Err(AiError::Http(401, format!("unauthorized {stored}")))
        });
        assert!(!result.ok);
        assert!(!result.message.contains(key));
        assert!(result.message.contains("[redacted]"));
        assert_eq!(secrets.get("claude").unwrap().as_deref(), Some(key));
        let project = include_str!("../../../crates/core/fixtures/project-v1.json");
        assert!(!project.contains(key));
        assert!(!project.contains("sk-"));
    }

    #[test]
    fn missing_key_does_not_call_the_provider() {
        let mut secrets = MemoryStore::default();
        let mut called = false;
        let result = execute_test("openai", "", &mut secrets, &mut |_, _| {
            called = true;
            Ok(Vec::new())
        });
        assert!(!called);
        assert_eq!(result.message, "No key saved");
    }
}
