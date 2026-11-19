//! LLM providers. The trait and network clients arrive in M03.

pub const NAME: &str = "ai";

pub const PROVIDERS: [&str; 2] = ["claude", "openai"];

#[cfg(test)]
mod tests {
    #[test]
    fn both_providers_are_named() {
        assert_eq!(super::PROVIDERS, ["claude", "openai"]);
    }
}
