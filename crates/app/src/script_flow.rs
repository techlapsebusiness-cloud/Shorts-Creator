use ai::{
    LlmProvider, ScriptBrief, SpendDecision, TokenUsage, config, cost_usd, decide, generate_script,
    regenerate_beat,
};
use core::Beat;
use store::{Library, UsageInsert};

use crate::meter::MeterError;

pub struct SpendGate<'a> {
    pub cap_usd: f64,
    pub alert_ratio: f64,
    pub created_at: &'a str,
}

pub fn generate_within_cap(
    library: &mut Library,
    provider: &dyn LlmProvider,
    brief: &ScriptBrief,
    gate: &SpendGate<'_>,
) -> Result<ai::GeneratedScript, MeterError> {
    let spent = library
        .usage_cost_usd()
        .map_err(|error| MeterError::Store(error.to_string()))?;
    if decide(spent, gate.cap_usd, gate.alert_ratio) == SpendDecision::Block {
        return Err(MeterError::Blocked);
    }
    let generated = generate_script(provider, brief)
        .map_err(|error| MeterError::Provider(error.to_string()))?;
    let usage = TokenUsage {
        input: generated.input_tokens,
        output: generated.output_tokens,
    };
    library
        .record_usage(&UsageInsert {
            provider: brief.provider.clone(),
            model: brief.model.clone(),
            task: "script".into(),
            input_tokens: usage.input,
            output_tokens: usage.output,
            cost_usd: cost_usd(&brief.provider, usage, &config().rates),
            created_at: gate.created_at.to_string(),
        })
        .map_err(|error| MeterError::Store(error.to_string()))?;
    Ok(generated)
}

pub fn regenerate_within_cap(
    library: &mut Library,
    provider: &dyn LlmProvider,
    brief: &ScriptBrief,
    beats: &[Beat],
    index: usize,
    gate: &SpendGate<'_>,
) -> Result<Beat, MeterError> {
    let spent = library
        .usage_cost_usd()
        .map_err(|error| MeterError::Store(error.to_string()))?;
    if decide(spent, gate.cap_usd, gate.alert_ratio) == SpendDecision::Block {
        return Err(MeterError::Blocked);
    }
    let regenerated = regenerate_beat(provider, brief, beats, index)
        .map_err(|error| MeterError::Provider(error.to_string()))?;
    let usage = TokenUsage {
        input: regenerated.input_tokens,
        output: regenerated.output_tokens,
    };
    library
        .record_usage(&UsageInsert {
            provider: brief.provider.clone(),
            model: brief.model.clone(),
            task: "script".into(),
            input_tokens: usage.input,
            output_tokens: usage.output,
            cost_usd: cost_usd(&brief.provider, usage, &config().rates),
            created_at: gate.created_at.to_string(),
        })
        .map_err(|error| MeterError::Store(error.to_string()))?;
    Ok(regenerated.beat)
}

#[cfg(test)]
mod tests {
    use super::{SpendGate, generate_within_cap, regenerate_within_cap};
    use ai::{Completion, MockProvider, ScriptBrief, TokenUsage};
    use core::{BeatKind, replace_beat_text};
    use store::{Library, UsageInsert};

    fn brief() -> ScriptBrief {
        ScriptBrief {
            provider: "claude".into(),
            model: "from-the-caller".into(),
            topic: "crash-safe saves".into(),
            language: "Hinglish".into(),
            length_seconds: 30,
            words_per_minute: 140.0,
            hook_in_3s: true,
            screen_cues: true,
            fact_check: true,
            cta: true,
        }
    }

    fn completion(text: &str) -> Completion {
        Completion {
            text: text.to_string(),
            deltas: vec![text.to_string()],
            usage: TokenUsage {
                input: 2,
                output: 1,
            },
        }
    }

    fn gate(second: &str) -> SpendGate<'_> {
        SpendGate {
            cap_usd: 20.0,
            alert_ratio: 0.8,
            created_at: match second {
                "02" => "2026-11-20T00:00:02Z",
                _ => "2026-11-20T00:00:01Z",
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

    #[test]
    fn the_cap_blocks_script_generation() {
        let mut library = Library::open_in_memory().unwrap();
        seed(&mut library, 20.0);
        let provider = MockProvider::script(vec![Ok(completion(include_str!(
            "../../ai/fixtures/script.json"
        )))]);
        let error =
            generate_within_cap(&mut library, &provider, &brief(), &gate("01")).unwrap_err();
        assert!(matches!(error, crate::meter::MeterError::Blocked));
        assert_eq!(provider.calls(), 0);
    }

    #[test]
    fn an_edit_keeps_the_raw_draft_distinct_from_the_beat() {
        let mut library = Library::open_in_memory().unwrap();
        let provider = MockProvider::script(vec![Ok(completion(include_str!(
            "../../ai/fixtures/script.json"
        )))]);
        let mut generated =
            generate_within_cap(&mut library, &provider, &brief(), &gate("01")).unwrap();
        let draft = generated.raw_draft.clone();
        replace_beat_text(&mut generated.beats, 0, "Edited in the panel").unwrap();
        assert_eq!(generated.raw_draft, draft);
        assert_eq!(generated.beats[0].text, "Edited in the panel");
        assert_eq!(generated.beats[0].kind, BeatKind::Hook);
        let again = MockProvider::script(vec![Ok(completion(
            r#"{"kind":"hook","start":0,"text":"Naya hook.","cues":["CUE: point"]}"#,
        ))]);
        let beat = regenerate_within_cap(
            &mut library,
            &again,
            &brief(),
            &generated.beats,
            0,
            &gate("02"),
        )
        .unwrap();
        assert_eq!(beat.text, "Naya hook.");
        assert_eq!(generated.raw_draft, draft);
    }
}
