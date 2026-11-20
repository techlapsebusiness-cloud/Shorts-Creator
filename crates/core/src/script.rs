use std::fmt;

use crate::project::{Beat, BeatKind};

const ORDER: [BeatKind; 5] = [
    BeatKind::Hook,
    BeatKind::Concept,
    BeatKind::Demo,
    BeatKind::Fix,
    BeatKind::Cta,
];

#[derive(Debug, Clone, Copy)]
pub struct ScriptRules {
    pub hook_within_seconds: Option<f64>,
    pub require_screen_cues: bool,
    pub require_fact_notes: bool,
}

#[derive(Debug, PartialEq, Eq)]
pub enum ScriptError {
    Shape,
    Order,
    Start,
    EmptyText,
    HookLate,
    MissingCue,
    MissingFact,
    BadPace,
    Index,
}

impl fmt::Display for ScriptError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Shape => write!(formatter, "script needs hook, concept, demo, fix, and cta"),
            Self::Order => write!(
                formatter,
                "beats must stay in hook, concept, demo, fix, cta order"
            ),
            Self::Start => write!(
                formatter,
                "beat starts must be finite, zero or later, and in order"
            ),
            Self::EmptyText => write!(formatter, "a beat needs spoken text"),
            Self::HookLate => write!(formatter, "hook starts outside the first 3 seconds"),
            Self::MissingCue => write!(formatter, "a screen cue starting with CUE: is required"),
            Self::MissingFact => write!(
                formatter,
                "a fact-check note starting with FACT: is required"
            ),
            Self::BadPace => write!(formatter, "words per minute must be positive"),
            Self::Index => write!(formatter, "beat index is outside the script"),
        }
    }
}

pub fn word_count(text: &str) -> u32 {
    u32::try_from(text.split_whitespace().count()).unwrap_or(u32::MAX)
}

pub fn estimate_seconds(text: &str, words_per_minute: f64) -> Result<f64, ScriptError> {
    if !words_per_minute.is_finite() || words_per_minute <= 0.0 {
        return Err(ScriptError::BadPace);
    }
    Ok(f64::from(word_count(text)) / words_per_minute * 60.0)
}

pub fn spoken_text(beats: &[Beat]) -> String {
    beats
        .iter()
        .map(|beat| beat.text.as_str())
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn script_seconds(beats: &[Beat], words_per_minute: f64) -> Result<f64, ScriptError> {
    estimate_seconds(&spoken_text(beats), words_per_minute)
}

pub fn validate_beats(beats: &[Beat], rules: &ScriptRules) -> Result<(), ScriptError> {
    if beats.len() != ORDER.len() {
        return Err(ScriptError::Shape);
    }
    let mut previous = 0.0;
    for (index, beat) in beats.iter().enumerate() {
        if beat.kind != ORDER[index] {
            return Err(ScriptError::Order);
        }
        if !beat.start.is_finite() || beat.start < 0.0 {
            return Err(ScriptError::Start);
        }
        if index > 0 && beat.start < previous {
            return Err(ScriptError::Start);
        }
        previous = beat.start;
        if beat.text.trim().is_empty() {
            return Err(ScriptError::EmptyText);
        }
        if beat.kind == BeatKind::Hook
            && rules
                .hook_within_seconds
                .is_some_and(|limit| beat.start >= limit)
        {
            return Err(ScriptError::HookLate);
        }
    }
    if rules.require_screen_cues && !has_prefix(beats, "CUE:") {
        return Err(ScriptError::MissingCue);
    }
    if rules.require_fact_notes && !has_prefix(beats, "FACT:") {
        return Err(ScriptError::MissingFact);
    }
    Ok(())
}

pub fn replace_beat_text(beats: &mut [Beat], index: usize, text: &str) -> Result<(), ScriptError> {
    let Some(beat) = beats.get_mut(index) else {
        return Err(ScriptError::Index);
    };
    if text.trim().is_empty() {
        return Err(ScriptError::EmptyText);
    }
    beat.text = text.to_string();
    Ok(())
}

fn has_prefix(beats: &[Beat], prefix: &str) -> bool {
    beats
        .iter()
        .any(|beat| beat.cues.iter().any(|cue| cue.starts_with(prefix)))
}

#[cfg(test)]
mod tests {
    use super::{
        ScriptError, ScriptRules, estimate_seconds, script_seconds, validate_beats, word_count,
    };
    use crate::project::{Beat, BeatKind};

    fn beat(kind: BeatKind, start: f64, text: &str, cues: &[&str]) -> Beat {
        Beat {
            kind,
            start,
            text: text.to_string(),
            cues: cues.iter().map(|cue| (*cue).to_string()).collect(),
        }
    }

    fn sample() -> Vec<Beat> {
        vec![
            beat(BeatKind::Hook, 0.0, "ek do teen", &["CUE: open"]),
            beat(BeatKind::Concept, 3.0, "chaar", &[]),
            beat(BeatKind::Demo, 8.0, "paanch chhe", &[]),
            beat(BeatKind::Fix, 15.0, "saat", &["FACT: checked"]),
            beat(BeatKind::Cta, 22.0, "aath", &[]),
        ]
    }

    #[test]
    fn duration_uses_words_per_minute() {
        assert_eq!(word_count("  ek   do teen  "), 3);
        assert_eq!(word_count(""), 0);
        let seconds = estimate_seconds("ek do teen", 60.0).unwrap();
        assert!((seconds - 3.0).abs() < 0.000_001);
        let script = script_seconds(&sample(), 140.0).unwrap();
        let words = word_count(&super::spoken_text(&sample()));
        assert_eq!(words, 8);
        assert!((script - f64::from(words) / 140.0 * 60.0).abs() < 0.000_001);
        assert_eq!(estimate_seconds("ek", 0.0), Err(ScriptError::BadPace));
    }

    #[test]
    fn hook_at_three_seconds_is_rejected() {
        let mut beats = sample();
        beats[0].start = 3.0;
        let rules = ScriptRules {
            hook_within_seconds: Some(3.0),
            require_screen_cues: true,
            require_fact_notes: true,
        };
        assert_eq!(validate_beats(&beats, &rules), Err(ScriptError::HookLate));
        beats[0].start = 0.0;
        assert!(validate_beats(&beats, &rules).is_ok());
    }
}
