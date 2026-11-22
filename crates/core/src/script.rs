//! Hinglish Short scripts: schema, duration, and the raw draft kept beside edits.

use crate::{AiMetadata, Beat, BeatKind, Project};
use serde::{Deserialize, Serialize};

pub const DEFAULT_WPM: f64 = 140.0;
pub const HOOK_LIMIT_SECS: f64 = 3.0;
pub const SCRIPT_PROMPT: &str = include_str!("../../../prompts/script.md");
const FIXTURE_JSON: &str = include_str!("../fixtures/script-output.json");

const BEAT_ORDER: [BeatKind; 5] = [
    BeatKind::Hook,
    BeatKind::Concept,
    BeatKind::Demo,
    BeatKind::Fix,
    BeatKind::Cta,
];

#[derive(Debug, Clone, PartialEq)]
pub struct ScriptError {
    pub message: String,
}

impl std::fmt::Display for ScriptError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}", self.message)
    }
}

impl std::error::Error for ScriptError {}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScriptDraft {
    pub raw: String,
    pub fact_check: Vec<String>,
    pub beats: Vec<Beat>,
}

pub fn word_count(text: &str) -> usize {
    text.split_whitespace()
        .filter(|word| !word.is_empty())
        .count()
}

pub fn estimate_duration_secs(text: &str, wpm: f64) -> f64 {
    if wpm <= 0.0 {
        return 0.0;
    }
    (word_count(text) as f64) / wpm * 60.0
}

pub fn spoken_text(draft: &ScriptDraft) -> String {
    draft
        .beats
        .iter()
        .map(|beat| beat.text.as_str())
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn parse_script(json: &str) -> Result<ScriptDraft, ScriptError> {
    let draft: ScriptDraft = serde_json::from_str(json).map_err(|error| ScriptError {
        message: format!("script json: {error}"),
    })?;
    validate_script(&draft)?;
    Ok(draft)
}

pub fn fixture_script(topic: &str, language: &str) -> Result<ScriptDraft, ScriptError> {
    let mut draft = parse_script(FIXTURE_JSON)?;
    draft.raw = format!("{language} draft about {topic}. {}", draft.raw);
    Ok(draft)
}

pub fn validate_script(draft: &ScriptDraft) -> Result<(), ScriptError> {
    if draft.beats.len() != BEAT_ORDER.len() {
        return Err(error(format!(
            "expected {} beats, found {}",
            BEAT_ORDER.len(),
            draft.beats.len()
        )));
    }
    for (expected, beat) in BEAT_ORDER.iter().zip(draft.beats.iter()) {
        if beat.kind != *expected {
            return Err(error(format!(
                "beat order must be hook, concept, demo, fix, cta; found {expected:?} mismatch"
            )));
        }
        if beat.text.trim().is_empty() {
            return Err(error("a beat needs spoken text"));
        }
        if beat.start < 0.0 {
            return Err(error("a beat cannot start before zero"));
        }
    }
    let hook = &draft.beats[0];
    if hook.start > HOOK_LIMIT_SECS {
        return Err(error(format!(
            "the hook starts at {} s and must land within {HOOK_LIMIT_SECS} s",
            hook.start
        )));
    }
    if draft.raw.trim().is_empty() {
        return Err(error("the raw draft is required"));
    }
    Ok(())
}

pub fn replace_beat(
    draft: &mut ScriptDraft,
    kind: BeatKind,
    text: &str,
    cues: Vec<String>,
) -> Result<(), ScriptError> {
    let beat = draft
        .beats
        .iter_mut()
        .find(|beat| beat.kind == kind)
        .ok_or_else(|| error("that beat is not in the script"))?;
    if text.trim().is_empty() {
        return Err(error("a beat needs spoken text"));
    }
    beat.text = text.to_string();
    beat.cues = cues;
    validate_script(draft)
}

pub fn apply_to_project(project: &mut Project, draft: &ScriptDraft) {
    project.ai = AiMetadata {
        provider: project.ai.provider.clone(),
        model: project.ai.model.clone(),
        topic: project.ai.topic.clone(),
        language: project.ai.language.clone(),
        raw_draft: draft.raw.clone(),
        beats: draft.beats.clone(),
    };
}

pub fn prompt_for(topic: &str, language: &str, length_secs: u32, wpm: f64) -> String {
    SCRIPT_PROMPT
        .replace("{{topic}}", topic)
        .replace("{{language}}", language)
        .replace("{{length_secs}}", &length_secs.to_string())
        .replace("{{wpm}}", &format!("{wpm:.0}"))
}

fn error(message: impl Into<String>) -> ScriptError {
    ScriptError {
        message: message.into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fixture_output_validates() {
        let draft = parse_script(FIXTURE_JSON).unwrap();
        assert_eq!(draft.beats.len(), 5);
        assert_eq!(draft.beats[0].kind, BeatKind::Hook);
        assert!(draft.beats[0].start <= HOOK_LIMIT_SECS);
        assert!(!draft.fact_check.is_empty());
        assert!(draft.beats.iter().any(|beat| !beat.cues.is_empty()));
    }

    #[test]
    fn duration_estimator_uses_words_per_minute() {
        assert_eq!(word_count("one two  three"), 3);
        assert!(
            (estimate_duration_secs("one two three", 140.0) - (3.0 / 140.0 * 60.0)).abs()
                < 0.000_001
        );
        assert!((estimate_duration_secs(&"word ".repeat(140), 140.0) - 60.0).abs() < 0.000_001);
        assert_eq!(estimate_duration_secs("hello", 0.0), 0.0);
        assert_eq!(estimate_duration_secs("   ", 140.0), 0.0);
    }

    #[test]
    fn hook_after_three_seconds_is_rejected() {
        let mut draft = parse_script(FIXTURE_JSON).unwrap();
        draft.beats[0].start = 4.0;
        let error = validate_script(&draft).unwrap_err();
        assert!(error.message.contains("3"));
    }

    #[test]
    fn replacing_a_beat_keeps_the_raw_draft() {
        let mut draft = parse_script(FIXTURE_JSON).unwrap();
        let raw = draft.raw.clone();
        replace_beat(
            &mut draft,
            BeatKind::Cta,
            "Ship the next short.",
            vec!["wave".into()],
        )
        .unwrap();
        assert_eq!(draft.raw, raw);
        assert_eq!(draft.beats[4].text, "Ship the next short.");
    }

    #[test]
    fn prompt_fills_the_topic() {
        let prompt = prompt_for("crash-safe saves", "Hinglish", 30, DEFAULT_WPM);
        assert!(prompt.contains("crash-safe saves"));
        assert!(prompt.contains("Hinglish"));
        assert!(prompt.contains("version: 1"));
    }
}
