use std::cell::RefCell;
use std::collections::VecDeque;

use crate::error::AiError;
use crate::provider::LlmProvider;
use crate::types::{CompleteRequest, Completion};

struct Inner {
    script: VecDeque<Result<Completion, AiError>>,
    requests: Vec<CompleteRequest>,
    calls: u32,
    models: Vec<String>,
}

pub struct MockProvider {
    inner: RefCell<Inner>,
}

impl MockProvider {
    pub fn script(steps: Vec<Result<Completion, AiError>>) -> Self {
        Self {
            inner: RefCell::new(Inner {
                script: steps.into(),
                requests: Vec::new(),
                calls: 0,
                models: Vec::new(),
            }),
        }
    }

    pub fn with_models(models: Vec<String>) -> Self {
        let provider = Self::script(Vec::new());
        provider.inner.borrow_mut().models = models;
        provider
    }

    pub fn calls(&self) -> u32 {
        self.inner.borrow().calls
    }

    pub fn requests(&self) -> Vec<CompleteRequest> {
        self.inner.borrow().requests.clone()
    }
}

impl LlmProvider for MockProvider {
    fn name(&self) -> &str {
        "mock"
    }

    fn list_models(&self) -> Result<Vec<String>, AiError> {
        Ok(self.inner.borrow().models.clone())
    }

    fn complete(&self, request: &CompleteRequest) -> Result<Completion, AiError> {
        let mut inner = self.inner.borrow_mut();
        inner.calls += 1;
        inner.requests.push(request.clone());
        inner.script.pop_front().ok_or(AiError::Empty)?
    }
}

#[cfg(test)]
mod tests {
    use super::MockProvider;
    use crate::provider::LlmProvider;
    use crate::sse::parse_openai_sse;
    use crate::types::CompleteRequest;

    #[test]
    fn mock_replays_fixture_deltas_in_order() {
        let completion = parse_openai_sse(include_str!("../fixtures/openai.sse")).unwrap();
        let mock = MockProvider::script(vec![Ok(completion)]);
        let got = mock
            .complete(&CompleteRequest {
                model: "from-the-caller".into(),
                messages: Vec::new(),
                schema: None,
                max_tokens: 8,
            })
            .unwrap();
        assert_eq!(got.deltas, vec!["{\"ok\":", "true}"]);
        assert_eq!(got.text, "{\"ok\":true}");
        assert_eq!(mock.calls(), 1);
        assert_eq!(mock.requests()[0].model, "from-the-caller");
    }
}
