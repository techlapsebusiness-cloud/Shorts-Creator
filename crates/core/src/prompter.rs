//! Time-based prompter scroll and voice-follow decisions.

use std::fmt;

pub const COUNTDOWN_MS: u64 = 3_000;
pub const CHUNK_MIN_MS: u64 = 2_000;
pub const CHUNK_MAX_MS: u64 = 3_000;
pub const NUDGE_PX: f64 = 48.0;
pub const MIN_CONFIDENCE: f64 = 0.6;
pub const MIN_MATCH: f64 = 0.6;

#[derive(Debug, PartialEq, Eq)]
pub enum PrompterError {
    BadPace,
    BadSize,
}

impl fmt::Display for PrompterError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BadPace => write!(formatter, "words per minute must be above zero"),
            Self::BadSize => write!(formatter, "text size must be above zero"),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ScrollInput {
    pub now_ms: u64,
    pub start_ms: u64,
    pub paused_ms: u64,
    pub words_per_minute: f64,
    pub text_size: f64,
    pub nudge_px: f64,
}

pub fn countdown_label(elapsed_ms: u64) -> Option<u8> {
    if elapsed_ms >= COUNTDOWN_MS {
        None
    } else {
        Some(3 - u8::try_from(elapsed_ms / 1_000).unwrap_or(3))
    }
}

pub fn scroll_offset_px(input: &ScrollInput) -> Result<f64, PrompterError> {
    if !input.words_per_minute.is_finite() || input.words_per_minute <= 0.0 {
        return Err(PrompterError::BadPace);
    }
    if !input.text_size.is_finite() || input.text_size <= 0.0 {
        return Err(PrompterError::BadSize);
    }
    if input.now_ms < input.start_ms {
        return Ok(0.0);
    }
    let playing = input
        .now_ms
        .saturating_sub(input.start_ms)
        .saturating_sub(input.paused_ms);
    let words = (playing as f64 / 1_000.0) * (input.words_per_minute / 60.0);
    Ok(words * input.text_size * 0.5 + input.nudge_px)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpokenLine {
    pub text: String,
    pub cue: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Heard {
    pub text: String,
    pub confidence: f64,
    pub silent: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Follow {
    ScrollTo { index: usize, score: f64 },
    Pause,
    FixedSpeed,
}

pub fn chunk_ready(buffered_ms: u64, silent: bool) -> bool {
    buffered_ms >= CHUNK_MAX_MS || (silent && buffered_ms >= CHUNK_MIN_MS)
}

pub fn follow(lines: &[SpokenLine], heard: &Heard) -> Follow {
    if heard.silent || heard.text.trim().is_empty() {
        return Follow::Pause;
    }
    if !heard.confidence.is_finite() || heard.confidence < MIN_CONFIDENCE {
        return Follow::FixedSpeed;
    }
    let Some((index, score)) = best_line(lines, &heard.text) else {
        return Follow::FixedSpeed;
    };
    if score < MIN_MATCH {
        Follow::FixedSpeed
    } else {
        Follow::ScrollTo { index, score }
    }
}

fn best_line(lines: &[SpokenLine], heard: &str) -> Option<(usize, f64)> {
    let heard_tokens = tokens(heard);
    if heard_tokens.is_empty() {
        return None;
    }
    let mut best: Option<(usize, f64)> = None;
    for (index, line) in lines.iter().enumerate() {
        if line.cue {
            continue;
        }
        let score = token_score(&tokens(&line.text), &heard_tokens);
        let replace = best.is_none_or(|(_, current)| score > current);
        if replace {
            best = Some((index, score));
        }
    }
    best
}

fn token_score(line: &[String], heard: &[String]) -> f64 {
    if line.is_empty() || heard.is_empty() {
        return 0.0;
    }
    let hits = heard
        .iter()
        .filter(|token| line.iter().any(|item| item == *token))
        .count();
    hits as f64 / heard.len() as f64
}

fn tokens(text: &str) -> Vec<String> {
    let mut word = String::new();
    let mut out = Vec::new();
    for ch in text.chars() {
        if ch.is_alphanumeric() {
            for lower in ch.to_lowercase() {
                word.push(lower);
            }
        } else if !word.is_empty() {
            out.push(std::mem::take(&mut word));
        }
    }
    if !word.is_empty() {
        out.push(word);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::{
        Follow, Heard, ScrollInput, SpokenLine, chunk_ready, countdown_label, follow,
        scroll_offset_px,
    };

    fn scroll(now_ms: u64, paused_ms: u64, nudge_px: f64) -> f64 {
        scroll_offset_px(&ScrollInput {
            now_ms,
            start_ms: 3_000,
            paused_ms,
            words_per_minute: 60.0,
            text_size: 20.0,
            nudge_px,
        })
        .unwrap()
    }

    fn near(actual: f64, expected: f64) {
        assert!((actual - expected).abs() < 0.001, "{actual} vs {expected}");
    }

    fn hinglish() -> Vec<SpokenLine> {
        vec![
            SpokenLine {
                text: "Ye file crash ke baad bhi bachi rehti hai.".into(),
                cue: false,
            },
            SpokenLine {
                text: "CUE: point at the folder".into(),
                cue: true,
            },
            SpokenLine {
                text: "Temp file ko save ke bagal mein dikhao.".into(),
                cue: false,
            },
        ]
    }

    #[test]
    fn time_becomes_a_scroll_offset() {
        near(scroll(3_000, 0, 0.0), 0.0);
        near(scroll(63_000, 0, 0.0), 600.0);
        near(scroll(63_000, 30_000, 0.0), 300.0);
        near(scroll(63_000, 0, 40.0), 640.0);
        near(scroll(1_000, 0, 12.0), 0.0);
        near(scroll(63_000, 90_000, 8.0), 8.0);
    }

    #[test]
    fn a_bad_pace_is_rejected() {
        let error = scroll_offset_px(&ScrollInput {
            now_ms: 1_000,
            start_ms: 0,
            paused_ms: 0,
            words_per_minute: 0.0,
            text_size: 48.0,
            nudge_px: 0.0,
        })
        .unwrap_err();
        assert_eq!(error, super::PrompterError::BadPace);
    }

    #[test]
    fn countdown_counts_down_from_three() {
        assert_eq!(countdown_label(0), Some(3));
        assert_eq!(countdown_label(1_000), Some(2));
        assert_eq!(countdown_label(2_000), Some(1));
        assert_eq!(countdown_label(3_000), None);
    }

    #[test]
    fn chunks_wait_two_seconds_and_flush_at_three() {
        assert!(!chunk_ready(1_999, false));
        assert!(!chunk_ready(1_999, true));
        assert!(chunk_ready(2_000, true));
        assert!(!chunk_ready(2_000, false));
        assert!(chunk_ready(3_000, false));
    }

    #[test]
    fn hinglish_speech_scrolls_to_the_spoken_line() {
        let lines = hinglish();
        let matched = follow(
            &lines,
            &Heard {
                text: "ye file crash ke baad bhi bachi".into(),
                confidence: 0.9,
                silent: false,
            },
        );
        assert_eq!(
            matched,
            Follow::ScrollTo {
                index: 0,
                score: 1.0
            }
        );
        let demo = follow(
            &lines,
            &Heard {
                text: "temp file ko save ke bagal mein dikhao".into(),
                confidence: 0.8,
                silent: false,
            },
        );
        assert_eq!(
            demo,
            Follow::ScrollTo {
                index: 2,
                score: 1.0
            }
        );
    }

    #[test]
    fn silence_pauses_and_a_weak_match_keeps_fixed_speed() {
        let lines = hinglish();
        assert_eq!(
            follow(
                &lines,
                &Heard {
                    text: String::new(),
                    confidence: 0.9,
                    silent: true,
                },
            ),
            Follow::Pause
        );
        assert_eq!(
            follow(
                &lines,
                &Heard {
                    text: "ye file crash ke baad".into(),
                    confidence: 0.2,
                    silent: false,
                },
            ),
            Follow::FixedSpeed
        );
        assert_eq!(
            follow(
                &lines,
                &Heard {
                    text: "mausam bahut garam hai aaj".into(),
                    confidence: 0.95,
                    silent: false,
                },
            ),
            Follow::FixedSpeed
        );
    }
}
