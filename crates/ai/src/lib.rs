//! LLM providers. Calls go out through reqwest. Keys stay in the OS keychain.

pub const NAME: &str = "ai";

pub const PROVIDERS: [&str; 2] = ["claude", "openai"];

pub const KEYCHAIN_SERVICE: &str = "shorts-studio";

pub const ANTHROPIC_VERSION: &str = "2026-10-21";

mod anthropic;
mod config;
mod error;
mod json_call;
mod mock;
mod openai;
mod provider;
mod redact;
mod secrets;
mod spend;
mod sse;
mod test_key;
mod transport;
mod types;

pub use config::{AiConfig, Defaults, Rates, Routing, config, default_model, provider_for};
pub use error::AiError;
pub use json_call::complete_json;
pub use mock::MockProvider;
pub use provider::LlmProvider;
pub use redact::redact;
pub use secrets::{MemoryStore, OsStore, SecretStore};
pub use spend::{SpendDecision, cost_usd, decide};
pub use test_key::{ProviderTest, execute_test, list_models};
pub use types::{CompleteRequest, Completion, JsonSchema, Message, TokenUsage};

#[cfg(test)]
mod live {
    use crate::provider::LlmProvider;
    use crate::types::{CompleteRequest, Message};

    fn enabled() -> bool {
        std::env::var("LIVE_AI").ok().as_deref() == Some("1")
    }

    #[test]
    fn live_lists_models_when_enabled() {
        if !enabled() {
            return;
        }
        let anthropic_key = std::env::var("LIVE_AI_ANTHROPIC_KEY").expect("LIVE_AI_ANTHROPIC_KEY");
        let openai_key = std::env::var("LIVE_AI_OPENAI_KEY").expect("LIVE_AI_OPENAI_KEY");
        let claude = crate::anthropic::AnthropicClient::connect(anthropic_key).unwrap();
        assert!(!claude.list_models().unwrap().is_empty());
        let openai = crate::openai::OpenAiClient::connect(openai_key).unwrap();
        assert!(!openai.list_models().unwrap().is_empty());
    }

    #[test]
    fn live_streams_when_model_env_is_set() {
        if !enabled() {
            return;
        }
        let Ok(claude_model) = std::env::var("LIVE_AI_ANTHROPIC_MODEL") else {
            return;
        };
        let Ok(openai_model) = std::env::var("LIVE_AI_OPENAI_MODEL") else {
            return;
        };
        if claude_model.is_empty() || openai_model.is_empty() {
            return;
        }
        let anthropic_key = std::env::var("LIVE_AI_ANTHROPIC_KEY").expect("LIVE_AI_ANTHROPIC_KEY");
        let openai_key = std::env::var("LIVE_AI_OPENAI_KEY").expect("LIVE_AI_OPENAI_KEY");
        let claude = crate::anthropic::AnthropicClient::connect(anthropic_key).unwrap();
        assert!(
            !claude
                .complete(&prompt(&claude_model))
                .unwrap()
                .deltas
                .is_empty()
        );
        let openai = crate::openai::OpenAiClient::connect(openai_key).unwrap();
        assert!(
            !openai
                .complete(&prompt(&openai_model))
                .unwrap()
                .deltas
                .is_empty()
        );
    }

    fn prompt(model: &str) -> CompleteRequest {
        CompleteRequest {
            model: model.to_string(),
            messages: vec![Message {
                role: "user".into(),
                content: "Reply with the word ok.".into(),
            }],
            schema: None,
            max_tokens: 16,
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn both_providers_are_named() {
        assert_eq!(super::PROVIDERS, ["claude", "openai"]);
    }

    #[test]
    fn provider_sources_do_not_name_a_model() {
        let files = [
            include_str!("anthropic.rs"),
            include_str!("openai.rs"),
            include_str!("config.rs"),
        ];
        for source in files {
            assert!(!source.contains("gpt-"), "model id in source");
            assert!(!source.contains("claude-"), "model id in source");
        }
    }
}
