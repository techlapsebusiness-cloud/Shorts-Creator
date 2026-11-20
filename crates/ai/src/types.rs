#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Message {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct JsonSchema {
    pub name: String,
    pub schema: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CompleteRequest {
    pub model: String,
    pub messages: Vec<Message>,
    pub schema: Option<JsonSchema>,
    pub max_tokens: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TokenUsage {
    pub input: i64,
    pub output: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Completion {
    pub text: String,
    pub deltas: Vec<String>,
    pub usage: TokenUsage,
}
