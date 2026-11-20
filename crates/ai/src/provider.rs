use crate::error::AiError;
use crate::types::{CompleteRequest, Completion};

pub trait LlmProvider {
    fn name(&self) -> &str;
    fn list_models(&self) -> Result<Vec<String>, AiError>;
    fn complete(&self, request: &CompleteRequest) -> Result<Completion, AiError>;
}
