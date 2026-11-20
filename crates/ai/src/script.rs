use core::{Beat, BeatKind, ScriptError, ScriptRules, validate_beats};
use serde::Deserialize;
use serde::Serialize;
use serde_json::{Value, json};

use crate::error::AiError;
use crate::provider::LlmProvider;
use crate::types::{CompleteRequest, Completion, JsonSchema, Message};

const SCRIPT_PROMPT: &str = include_str!("../../../prompts/script.md");
const BEAT_PROMPT: &str = include_str!("../../../prompts/regenerate-beat.md");

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScriptBrief {
    pub provider: String,
    pub model: String,
    pub topic: String,
    pub language: String,
    pub length_seconds: u32,
    pub words_per_minute: f64,
    pub hook_in_3s: bool,
    pub screen_cues: bool,
    pub fact_check: bool,
    pub cta: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegeneratedBeat {
    pub beat: Beat,
    pub input_tokens: i64,
    pub output_tokens: i64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GeneratedScript {
    pub raw_draft: String,
    pub beats: Vec<Beat>,
    pub input_tokens: i64,
    pub output_tokens: i64,
}

#[derive(Debug, Deserialize, Serialize)]
struct ScriptBody {
    beats: Vec<Beat>,
}

pub fn generate_script(
    provider: &dyn LlmProvider,
    brief: &ScriptBrief,
) -> Result<GeneratedScript, AiError> {
    if brief.model.trim().is_empty() {
        return Err(AiError::ModelRequired);
    }
    let mut request = script_request(brief)?;
    let mut input = 0;
    let mut output = 0;
    let first = take(provider, &request, &mut input, &mut output)?;
    match accept_script(&first.text, brief) {
        Ok(beats) => Ok(generated(first.text, beats, input, output)),
        Err(error) => {
            push_note(&mut request, &error.to_string());
            let second = take(provider, &request, &mut input, &mut output)?;
            let beats = accept_script(&second.text, brief)?;
            Ok(generated(second.text, beats, input, output))
        }
    }
}

pub fn regenerate_beat(
    provider: &dyn LlmProvider,
    brief: &ScriptBrief,
    beats: &[Beat],
    index: usize,
) -> Result<RegeneratedBeat, AiError> {
    if brief.model.trim().is_empty() {
        return Err(AiError::ModelRequired);
    }
    let Some(current) = beats.get(index) else {
        return Err(AiError::Schema(ScriptError::Index.to_string()));
    };
    let mut request = beat_request(brief, current)?;
    let mut input = 0;
    let mut output = 0;
    let first = take(provider, &request, &mut input, &mut output)?;
    let beat = match accept_beat(&first.text, current, brief) {
        Ok(beat) => beat,
        Err(error) => {
            push_note(&mut request, &error.to_string());
            let second = take(provider, &request, &mut input, &mut output)?;
            accept_beat(&second.text, current, brief)?
        }
    };
    Ok(RegeneratedBeat {
        beat,
        input_tokens: input,
        output_tokens: output,
    })
}

pub fn prompt_version(text: &str) -> Result<u32, AiError> {
    let line = text
        .lines()
        .find(|line| !line.trim().is_empty())
        .unwrap_or("");
    let Some(value) = line.trim().strip_prefix("version:") else {
        return Err(AiError::Parse("prompt is missing a version".into()));
    };
    value
        .trim()
        .parse()
        .map_err(|_| AiError::Parse("prompt version is not a number".into()))
}

fn generated(raw_draft: String, beats: Vec<Beat>, input: i64, output: i64) -> GeneratedScript {
    GeneratedScript {
        raw_draft,
        beats,
        input_tokens: input,
        output_tokens: output,
    }
}

fn take(
    provider: &dyn LlmProvider,
    request: &CompleteRequest,
    input: &mut i64,
    output: &mut i64,
) -> Result<Completion, AiError> {
    let completion = provider.complete(request)?;
    *input += completion.usage.input;
    *output += completion.usage.output;
    Ok(completion)
}

fn accept_script(text: &str, brief: &ScriptBrief) -> Result<Vec<Beat>, AiError> {
    let body: ScriptBody =
        serde_json::from_str(text).map_err(|error| AiError::Schema(error.to_string()))?;
    validate_beats(&body.beats, &rules(brief))
        .map_err(|error| AiError::Schema(error.to_string()))?;
    Ok(body.beats)
}

fn accept_beat(text: &str, current: &Beat, brief: &ScriptBrief) -> Result<Beat, AiError> {
    let beat: Beat =
        serde_json::from_str(text).map_err(|error| AiError::Schema(error.to_string()))?;
    if beat.kind != current.kind {
        return Err(AiError::Schema("beat kind changed".into()));
    }
    if (beat.start - current.start).abs() > 0.001 {
        return Err(AiError::Schema("beat start changed".into()));
    }
    if beat.text.trim().is_empty() {
        return Err(AiError::Schema(ScriptError::EmptyText.to_string()));
    }
    if beat.kind == BeatKind::Hook && brief.hook_in_3s && beat.start >= 3.0 {
        return Err(AiError::Schema(ScriptError::HookLate.to_string()));
    }
    Ok(beat)
}

fn rules(brief: &ScriptBrief) -> ScriptRules {
    ScriptRules {
        hook_within_seconds: brief.hook_in_3s.then_some(3.0),
        require_screen_cues: brief.screen_cues,
        require_fact_notes: brief.fact_check,
    }
}

fn script_request(brief: &ScriptBrief) -> Result<CompleteRequest, AiError> {
    Ok(CompleteRequest {
        model: brief.model.clone(),
        messages: vec![Message {
            role: "user".into(),
            content: render(checked_body(SCRIPT_PROMPT)?, brief, None)?,
        }],
        schema: Some(script_schema()),
        max_tokens: 1200,
    })
}

fn beat_request(brief: &ScriptBrief, beat: &Beat) -> Result<CompleteRequest, AiError> {
    let body = render(checked_body(BEAT_PROMPT)?, brief, Some(beat))?;
    Ok(CompleteRequest {
        model: brief.model.clone(),
        messages: vec![Message {
            role: "user".into(),
            content: body,
        }],
        schema: Some(beat_schema()),
        max_tokens: 400,
    })
}

fn render(template: &str, brief: &ScriptBrief, beat: Option<&Beat>) -> Result<String, AiError> {
    let pace = format_pace(brief.words_per_minute)?;
    let length = brief.length_seconds.to_string();
    let start = beat.map(|item| item.start.to_string()).unwrap_or_default();
    let mut text = template.to_string();
    let pairs = [
        ("topic", brief.topic.as_str()),
        ("language", brief.language.as_str()),
        ("length_seconds", length.as_str()),
        ("words_per_minute", pace.as_str()),
        ("hook_in_3s", yes_no(brief.hook_in_3s)),
        ("screen_cues", yes_no(brief.screen_cues)),
        ("fact_check", yes_no(brief.fact_check)),
        ("cta", yes_no(brief.cta)),
        ("kind", beat.map(kind_name).unwrap_or("")),
        ("text", beat.map(|item| item.text.as_str()).unwrap_or("")),
        ("start", start.as_str()),
    ];
    for (key, value) in pairs {
        text = text.replace(&placeholder(key), value);
    }
    Ok(text)
}

fn format_pace(words_per_minute: f64) -> Result<String, AiError> {
    if !words_per_minute.is_finite() || words_per_minute <= 0.0 {
        return Err(AiError::Provider(ScriptError::BadPace.to_string()));
    }
    Ok(words_per_minute.to_string())
}

fn yes_no(value: bool) -> &'static str {
    if value { "yes" } else { "no" }
}

fn kind_name(beat: &Beat) -> &'static str {
    match beat.kind {
        BeatKind::Hook => "hook",
        BeatKind::Concept => "concept",
        BeatKind::Demo => "demo",
        BeatKind::Fix => "fix",
        BeatKind::Cta => "cta",
    }
}

fn placeholder(key: &str) -> String {
    format!("{{{{{key}}}}}")
}

fn checked_body(text: &str) -> Result<&str, AiError> {
    prompt_version(text)?;
    Ok(prompt_body(text))
}

fn prompt_body(text: &str) -> &str {
    let Some(rest) = text.strip_prefix("version:") else {
        return text;
    };
    rest.split_once('\n')
        .map(|(_, body)| body.trim_start())
        .unwrap_or(text)
}

fn push_note(request: &mut CompleteRequest, error: &str) {
    let note = format!("Validation failed: {error}. Reply with JSON only.");
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
}

fn script_schema() -> JsonSchema {
    JsonSchema {
        name: "script".into(),
        schema: json!({
            "type": "object",
            "properties": { "beats": { "type": "array", "items": beat_item() } },
            "required": ["beats"],
            "additionalProperties": false
        }),
    }
}

fn beat_schema() -> JsonSchema {
    JsonSchema {
        name: "beat".into(),
        schema: beat_item(),
    }
}

fn beat_item() -> Value {
    json!({
        "type": "object",
        "properties": {
            "kind": { "type": "string", "enum": ["hook", "concept", "demo", "fix", "cta"] },
            "start": { "type": "number" },
            "text": { "type": "string" },
            "cues": { "type": "array", "items": { "type": "string" } }
        },
        "required": ["kind", "start", "text", "cues"],
        "additionalProperties": false
    })
}

#[cfg(test)]
mod tests {
    use super::{
        BEAT_PROMPT, SCRIPT_PROMPT, generate_script, placeholder, prompt_version, regenerate_beat,
    };
    use crate::mock::MockProvider;
    use crate::script::ScriptBrief;
    use crate::types::{Completion, TokenUsage};

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
                input: 11,
                output: 4,
            },
        }
    }

    #[test]
    fn prompts_are_versioned_and_do_not_name_a_model() {
        assert_eq!(prompt_version(SCRIPT_PROMPT).unwrap(), 1);
        assert_eq!(prompt_version(BEAT_PROMPT).unwrap(), 1);
        assert_eq!(placeholder("topic"), "{{topic}}");
        for source in [SCRIPT_PROMPT, BEAT_PROMPT] {
            for needle in [format!("{}-", "gpt"), format!("{}-", "claude")] {
                assert!(!source.contains(&needle));
            }
            assert!(source.contains("CUE:"));
        }
    }

    #[test]
    fn fixture_output_validates_and_keeps_the_raw_draft() {
        let fixture = include_str!("../fixtures/script.json");
        let mock = MockProvider::script(vec![Ok(completion(fixture))]);
        let generated = generate_script(&mock, &brief()).unwrap();
        assert_eq!(generated.raw_draft, fixture);
        assert_eq!(generated.beats.len(), 5);
        assert!(generated.beats[0].start < 3.0);
        assert_eq!(generated.input_tokens, 11);
        assert_eq!(mock.calls(), 1);
        assert_eq!(mock.requests()[0].model, "from-the-caller");
        assert!(
            mock.requests()[0].messages[0]
                .content
                .contains("crash-safe saves")
        );
        assert!(mock.requests()[0].schema.is_some());
    }

    #[test]
    fn a_late_hook_retries_once_with_the_validation_error() {
        let late = r#"{"beats":[
            {"kind":"hook","start":9,"text":"late","cues":["CUE: wait"]},
            {"kind":"concept","start":10,"text":"c","cues":[]},
            {"kind":"demo","start":11,"text":"d","cues":[]},
            {"kind":"fix","start":12,"text":"f","cues":["FACT: note"]},
            {"kind":"cta","start":13,"text":"go","cues":[]}
        ]}"#;
        let mock = MockProvider::script(vec![
            Ok(completion(late)),
            Ok(completion(include_str!("../fixtures/script.json"))),
        ]);
        let generated = generate_script(&mock, &brief()).unwrap();
        assert_eq!(mock.calls(), 2);
        let requests = mock.requests();
        let second = requests[1].messages.last().unwrap();
        assert!(
            second
                .content
                .contains("hook starts outside the first 3 seconds")
        );
        assert!(generated.beats[0].start < 3.0);
        assert_eq!(generated.input_tokens, 22);
    }

    #[test]
    fn regenerating_one_beat_leaves_the_raw_draft() {
        let fixture = include_str!("../fixtures/script.json");
        let mock = MockProvider::script(vec![Ok(completion(fixture))]);
        let generated = generate_script(&mock, &brief()).unwrap();
        let rewritten = r#"{"kind":"hook","start":0,"text":"Naya hook.","cues":["CUE: point"]}"#;
        let again = MockProvider::script(vec![Ok(completion(rewritten))]);
        let beat = regenerate_beat(&again, &brief(), &generated.beats, 0).unwrap();
        assert_eq!(beat.beat.text, "Naya hook.");
        assert_eq!(generated.raw_draft, fixture);
        assert_eq!(again.calls(), 1);
    }
}
