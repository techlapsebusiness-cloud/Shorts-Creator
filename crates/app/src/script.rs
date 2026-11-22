//! Script commands. Without a saved key, Generate returns the versioned fixture.

use core::{
    BeatKind, DEFAULT_WPM, Project, ScriptDraft, apply_to_project, estimate_duration_secs,
    fixture_script, prompt_for, replace_beat, spoken_text,
};
use serde::Serialize;
use tauri::{AppHandle, Manager};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScriptView {
    pub draft: ScriptDraft,
    pub word_count: usize,
    pub duration_secs: f64,
    pub wpm: f64,
    pub prompt: String,
}

pub fn build_script(
    topic: &str,
    language: &str,
    length_secs: u32,
    wpm: f64,
) -> Result<ScriptView, String> {
    let draft = fixture_script(topic, language).map_err(|error| error.message)?;
    Ok(view(draft, wpm, topic, language, length_secs))
}

#[cfg(test)]
fn draft_from_model(
    json: &str,
    wpm: f64,
    topic: &str,
    language: &str,
    length_secs: u32,
) -> Result<ScriptView, String> {
    let draft = core::parse_script(json).map_err(|error| error.message)?;
    Ok(view(draft, wpm, topic, language, length_secs))
}

pub fn regenerate(
    mut draft: ScriptDraft,
    kind: &str,
    text: &str,
    cues: Vec<String>,
    wpm: f64,
) -> Result<ScriptView, String> {
    let beat_kind = parse_kind(kind)?;
    replace_beat(&mut draft, beat_kind, text, cues).map_err(|error| error.message)?;
    let topic = String::new();
    let language = String::new();
    Ok(view(draft, wpm, &topic, &language, 30))
}

pub fn project_with_script(topic: &str, language: &str, draft: &ScriptDraft) -> Project {
    let mut project = Project::new("script-current", 1, topic, "script-current");
    project.ai.topic = topic.to_string();
    project.ai.language = language.to_string();
    apply_to_project(&mut project, draft);
    project
}

fn view(draft: ScriptDraft, wpm: f64, topic: &str, language: &str, length_secs: u32) -> ScriptView {
    let wpm = if wpm <= 0.0 { DEFAULT_WPM } else { wpm };
    let text = spoken_text(&draft);
    ScriptView {
        word_count: core::word_count(&text),
        duration_secs: estimate_duration_secs(&text, wpm),
        wpm,
        prompt: prompt_for(topic, language, length_secs, wpm),
        draft,
    }
}

#[tauri::command]
pub fn generate_script(
    topic: String,
    language: String,
    length_secs: u32,
    wpm: f64,
) -> Result<ScriptView, String> {
    build_script(&topic, &language, length_secs, wpm)
}

#[tauri::command]
pub fn regenerate_beat(
    draft: ScriptDraft,
    kind: String,
    text: String,
    cues: Vec<String>,
    wpm: f64,
) -> Result<ScriptView, String> {
    regenerate(draft, &kind, &text, cues, wpm)
}

#[tauri::command]
pub fn save_script(
    app: AppHandle,
    topic: String,
    language: String,
    draft: ScriptDraft,
) -> Result<(), String> {
    let dir = app
        .path()
        .app_data_dir()
        .map_err(|error| error.to_string())?
        .join("script-current");
    std::fs::create_dir_all(&dir).map_err(|error| error.to_string())?;
    let project = project_with_script(&topic, &language, &draft);
    store::save_project(&dir, &project).map_err(|error| error.to_string())
}

fn parse_kind(kind: &str) -> Result<BeatKind, String> {
    match kind.to_ascii_lowercase().as_str() {
        "hook" => Ok(BeatKind::Hook),
        "concept" => Ok(BeatKind::Concept),
        "demo" => Ok(BeatKind::Demo),
        "fix" => Ok(BeatKind::Fix),
        "cta" => Ok(BeatKind::Cta),
        other => Err(format!("unknown beat {other}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ai::{
        CompleteRequest, Completion, JsonSchema, LlmProvider, Message, MockProvider, TokenUsage,
    };

    #[test]
    fn fixture_script_reports_duration() {
        let view = build_script("crash-safe saves", "Hinglish", 30, 140.0).unwrap();
        assert_eq!(view.draft.beats.len(), 5);
        assert!(view.word_count > 0);
        assert!(view.duration_secs > 0.0);
        assert!(view.prompt.contains("crash-safe saves"));
        assert!(view.draft.raw.contains("Hinglish"));
    }

    #[test]
    fn model_json_must_match_the_schema() {
        let error = draft_from_model("{}", 140.0, "topic", "Hinglish", 30).unwrap_err();
        assert!(error.contains("script json") || error.contains("expected"));
    }

    #[test]
    fn mock_provider_json_becomes_a_script() {
        let json = include_str!("../../core/fixtures/script-output.json");
        let provider = MockProvider::script(vec![Ok(Completion {
            text: json.into(),
            deltas: Vec::new(),
            usage: TokenUsage {
                input: 1,
                output: 1,
            },
        })]);
        let request = CompleteRequest {
            model: "fixture".into(),
            messages: vec![Message {
                role: "user".into(),
                content: prompt_for("crash-safe saves", "Hinglish", 30, 140.0),
            }],
            schema: Some(JsonSchema {
                name: "script".into(),
                schema: serde_json::json!({"type": "object"}),
            }),
            max_tokens: 800,
        };
        let completion = provider.complete(&request).unwrap();
        let view =
            draft_from_model(&completion.text, 140.0, "crash-safe saves", "Hinglish", 30).unwrap();
        assert_eq!(view.draft.beats[0].text, "This file survives a crash.");
        assert_eq!(provider.calls(), 1);
    }
}
