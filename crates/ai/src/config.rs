use std::sync::OnceLock;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct AiConfig {
    pub cap_usd: f64,
    pub alert_ratio: f64,
    pub routing: Routing,
    pub defaults: Defaults,
    pub rates: Rates,
}

#[derive(Debug, Deserialize)]
pub struct Routing {
    pub script: String,
    pub upload_kit: String,
    pub cover_text: String,
    pub fact_check: String,
}

#[derive(Debug, Deserialize)]
pub struct Defaults {
    pub claude: String,
    pub openai: String,
}

#[derive(Debug, Deserialize)]
pub struct Rates {
    pub claude_input: f64,
    pub claude_output: f64,
    pub openai_input: f64,
    pub openai_output: f64,
}

pub fn config() -> &'static AiConfig {
    static CONFIG: OnceLock<AiConfig> = OnceLock::new();
    CONFIG.get_or_init(|| {
        toml::from_str(include_str!("../../../config/ai.toml")).expect("config/ai.toml parses")
    })
}

pub fn provider_for(task: &str) -> Option<&str> {
    let routing = &config().routing;
    match task {
        "script" => Some(routing.script.as_str()),
        "upload_kit" => Some(routing.upload_kit.as_str()),
        "cover_text" => Some(routing.cover_text.as_str()),
        "fact_check" => Some(routing.fact_check.as_str()),
        _ => None,
    }
}

pub fn default_model(provider: &str) -> &str {
    match provider {
        "claude" => config().defaults.claude.as_str(),
        "openai" => config().defaults.openai.as_str(),
        _ => "",
    }
}

#[cfg(test)]
mod tests {
    use super::{config, provider_for};

    #[test]
    fn routing_and_cap_come_from_the_config_file() {
        let config = config();
        assert!((config.cap_usd - 20.0).abs() < 0.000_001);
        assert!((config.alert_ratio - 0.8).abs() < 0.000_001);
        assert_eq!(provider_for("script"), Some("claude"));
        assert_eq!(provider_for("upload_kit"), Some("claude"));
        assert_eq!(provider_for("cover_text"), Some("openai"));
        assert_eq!(provider_for("fact_check"), Some("claude"));
        assert_eq!(config.defaults.claude, "");
        assert_eq!(config.defaults.openai, "");
        assert!((config.rates.claude_input).abs() < 0.000_001);
        assert!((config.rates.openai_output).abs() < 0.000_001);
    }
}
