use std::fmt;
use std::path::Path;

use ai::{CompleteRequest, LlmProvider, SpendDecision, decide};
use store::{Library, UsageInsert};

#[allow(dead_code)] // Later modules call the model through this. The cap tests cover it.
pub struct MeterRequest<'a> {
    pub provider: &'a dyn LlmProvider,
    pub request: &'a CompleteRequest,
    pub task: &'a str,
    pub cap_usd: f64,
    pub alert_ratio: f64,
    pub cost_usd: f64,
    pub created_at: &'a str,
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct MeterOutcome {
    pub text: String,
    pub deltas: Vec<String>,
    pub decision: SpendDecision,
}

#[allow(dead_code)]
#[derive(Debug)]
pub enum MeterError {
    Blocked,
    Provider(String),
    Store(String),
}

impl fmt::Display for MeterError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Blocked => write!(formatter, "monthly cap reached"),
            Self::Provider(message) => write!(formatter, "{message}"),
            Self::Store(message) => write!(formatter, "{message}"),
        }
    }
}

impl std::error::Error for MeterError {}

#[allow(dead_code)]
pub fn complete_within_cap(
    library: &mut Library,
    call: &MeterRequest<'_>,
) -> Result<MeterOutcome, MeterError> {
    let spent = library
        .usage_cost_usd()
        .map_err(|error| MeterError::Store(error.to_string()))?;
    let decision = decide(spent, call.cap_usd, call.alert_ratio);
    if decision == SpendDecision::Block {
        return Err(MeterError::Blocked);
    }
    let completion = call
        .provider
        .complete(call.request)
        .map_err(|error| MeterError::Provider(error.to_string()))?;
    library
        .record_usage(&UsageInsert {
            provider: call.provider.name().to_string(),
            model: call.request.model.clone(),
            task: call.task.to_string(),
            input_tokens: completion.usage.input,
            output_tokens: completion.usage.output,
            cost_usd: call.cost_usd,
            created_at: call.created_at.to_string(),
        })
        .map_err(|error| MeterError::Store(error.to_string()))?;
    Ok(MeterOutcome {
        text: completion.text,
        deltas: completion.deltas,
        decision,
    })
}

pub fn read_cap(path: &Path) -> Option<f64> {
    let text = std::fs::read_to_string(path).ok()?;
    let value: f64 = text.trim().parse().ok()?;
    if value.is_finite() && value > 0.0 {
        Some(value)
    } else {
        None
    }
}

pub fn write_cap(path: &Path, cap: f64) -> Result<(), String> {
    if !cap.is_finite() || cap <= 0.0 {
        return Err("Cap must be a positive amount".into());
    }
    if let Some(parent) = path.parent()
        && !parent.as_os_str().is_empty()
    {
        std::fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    std::fs::write(path, format!("{cap}\n")).map_err(|error| error.to_string())
}

pub fn effective_cap(override_cap: Option<f64>, fallback: f64) -> f64 {
    override_cap.unwrap_or(fallback)
}

#[cfg(test)]
mod tests {
    use super::{
        MeterError, MeterRequest, complete_within_cap, effective_cap, read_cap, write_cap,
    };
    use ai::{Completion, LlmProvider, MockProvider, SpendDecision, TokenUsage};
    use store::{Library, UsageInsert};

    fn request() -> ai::CompleteRequest {
        ai::CompleteRequest {
            model: "from-the-caller".into(),
            messages: Vec::new(),
            schema: None,
            max_tokens: 8,
        }
    }

    fn completion() -> Completion {
        Completion {
            text: "ok".into(),
            deltas: vec!["o".into(), "k".into()],
            usage: TokenUsage {
                input: 3,
                output: 2,
            },
        }
    }

    fn seed(library: &mut Library, cost: f64) {
        library
            .record_usage(&UsageInsert {
                provider: "claude".into(),
                model: "from-the-caller".into(),
                task: "script".into(),
                input_tokens: 1,
                output_tokens: 1,
                cost_usd: cost,
                created_at: "2026-11-20T00:00:00Z".into(),
            })
            .unwrap();
    }

    fn call<'a>(
        provider: &'a dyn LlmProvider,
        request: &'a ai::CompleteRequest,
        cost: f64,
    ) -> MeterRequest<'a> {
        MeterRequest {
            provider,
            request,
            task: "script",
            cap_usd: 20.0,
            alert_ratio: 0.8,
            cost_usd: cost,
            created_at: "2026-11-20T00:00:01Z",
        }
    }

    #[test]
    fn cap_blocks_before_the_provider_is_called() {
        let mut library = Library::open_in_memory().unwrap();
        seed(&mut library, 20.0);
        let provider = MockProvider::script(vec![Ok(completion())]);
        let request = request();
        let error = complete_within_cap(&mut library, &call(&provider, &request, 1.0)).unwrap_err();
        assert!(matches!(error, MeterError::Blocked));
        assert_eq!(provider.calls(), 0);
        assert!((library.usage_cost_usd().unwrap() - 20.0).abs() < 0.000_001);
    }

    #[test]
    fn cap_alerts_and_still_records_the_call() {
        let mut library = Library::open_in_memory().unwrap();
        seed(&mut library, 19.0);
        let provider = MockProvider::script(vec![Ok(completion())]);
        let request = request();
        let outcome = complete_within_cap(&mut library, &call(&provider, &request, 1.0)).unwrap();
        assert_eq!(outcome.decision, SpendDecision::Alert);
        assert_eq!(outcome.deltas, vec!["o", "k"]);
        assert_eq!(provider.calls(), 1);
        assert!((library.usage_cost_usd().unwrap() - 20.0).abs() < 0.000_001);
    }

    #[test]
    fn spend_under_the_alert_line_is_allowed() {
        let mut library = Library::open_in_memory().unwrap();
        let provider = MockProvider::script(vec![Ok(completion())]);
        let request = request();
        let outcome = complete_within_cap(&mut library, &call(&provider, &request, 1.0)).unwrap();
        assert_eq!(outcome.decision, SpendDecision::Allow);
        assert!((library.usage_cost_usd().unwrap() - 1.0).abs() < 0.000_001);
    }

    #[test]
    fn cap_override_round_trips() {
        let dir = std::env::temp_dir().join(format!("shorts-studio-cap-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let path = dir.join("spend-cap.txt");
        write_cap(&path, 25.0).unwrap();
        let saved = read_cap(&path).unwrap();
        assert!((saved - 25.0).abs() < 0.000_001);
        assert!((effective_cap(Some(saved), 20.0) - 25.0).abs() < 0.000_001);
        assert!((effective_cap(None, 20.0) - 20.0).abs() < 0.000_001);
        assert!(write_cap(&path, 0.0).is_err());
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
