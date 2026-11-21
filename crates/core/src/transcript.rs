//! Word timestamps, fillers, silences, and chapters. Whisper itself lives in `media`.

use serde::{Deserialize, Serialize};

use crate::{Beat, BeatKind, WordStamp};

pub const WORD_TOLERANCE_SECS: f64 = 0.150;
pub const SILENCE_MIN_SECS: f64 = 0.6;
pub const DEFAULT_FILLERS: &[&str] = &["umm", "uh", "matlab", "basically", "you know"];

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FillerHit {
    pub text: String,
    pub start: f64,
    pub end: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SilenceSpan {
    pub start: f64,
    pub end: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Chapter {
    pub kind: BeatKind,
    pub start: f64,
    pub end: f64,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TranscriptReport {
    pub status: String,
    pub message: String,
    pub words: Vec<WordStamp>,
    pub fillers: Vec<FillerHit>,
    pub silences: Vec<SilenceSpan>,
    pub chapters: Vec<Chapter>,
}

pub fn normalize_token(text: &str) -> String {
    text.chars()
        .filter(|character| character.is_ascii_alphanumeric())
        .flat_map(|character| character.to_lowercase())
        .collect()
}

pub fn fillers_in_words(words: &[WordStamp], list: &[&str]) -> Vec<FillerHit> {
    let mut phrases: Vec<Vec<String>> = list
        .iter()
        .map(|phrase| {
            phrase
                .split_whitespace()
                .map(normalize_token)
                .filter(|token| !token.is_empty())
                .collect::<Vec<_>>()
        })
        .filter(|phrase| !phrase.is_empty())
        .collect();
    phrases.sort_by_key(|phrase| std::cmp::Reverse(phrase.len()));

    let tokens: Vec<String> = words
        .iter()
        .map(|word| normalize_token(&word.text))
        .collect();
    let mut hits = Vec::new();
    let mut index = 0;
    while index < words.len() {
        let matched = phrases.iter().find(|phrase| {
            !phrase.is_empty()
                && index + phrase.len() <= tokens.len()
                && phrase
                    .iter()
                    .enumerate()
                    .all(|(offset, token)| tokens[index + offset] == *token)
        });
        if let Some(phrase) = matched {
            let end = index + phrase.len() - 1;
            hits.push(FillerHit {
                text: phrase.join(" "),
                start: words[index].start,
                end: words[end].end,
            });
            index += phrase.len();
        } else {
            index += 1;
        }
    }
    hits
}

/// Energy gaps in mono PCM. `samples` are in -1..1.
pub fn silences_in_pcm(samples: &[f32], sample_rate: u32, min_secs: f64) -> Vec<SilenceSpan> {
    if sample_rate == 0 || samples.is_empty() {
        return Vec::new();
    }
    let frame = (sample_rate / 100).max(1) as usize;
    let mut spans = Vec::new();
    let mut silent_from: Option<usize> = None;
    let mut index = 0;
    while index < samples.len() {
        let end = (index + frame).min(samples.len());
        let rms = rms(&samples[index..end]);
        let silent = rms < 0.02;
        match (silent, silent_from) {
            (true, None) => silent_from = Some(index),
            (false, Some(start)) => {
                push_silence(&mut spans, start, index, sample_rate, min_secs);
                silent_from = None;
            }
            _ => {}
        }
        index = end;
    }
    if let Some(start) = silent_from {
        push_silence(&mut spans, start, samples.len(), sample_rate, min_secs);
    }
    spans
}

fn push_silence(
    spans: &mut Vec<SilenceSpan>,
    start_sample: usize,
    end_sample: usize,
    sample_rate: u32,
    min_secs: f64,
) {
    let start = start_sample as f64 / f64::from(sample_rate);
    let end = end_sample as f64 / f64::from(sample_rate);
    if end - start >= min_secs {
        spans.push(SilenceSpan { start, end });
    }
}

fn rms(frame: &[f32]) -> f32 {
    if frame.is_empty() {
        return 0.0;
    }
    let energy = frame.iter().map(|sample| sample * sample).sum::<f32>() / frame.len() as f32;
    energy.sqrt()
}

/// Expected words must appear, in order, within `tolerance` seconds of `actual`.
pub fn words_within(actual: &[WordStamp], expected: &[WordStamp], tolerance: f64) -> bool {
    let mut cursor = 0;
    for expected_word in expected {
        let expected_token = normalize_token(&expected_word.text);
        let found = actual[cursor..].iter().position(|word| {
            normalize_token(&word.text) == expected_token
                && (word.start - expected_word.start).abs() <= tolerance
        });
        match found {
            Some(offset) => cursor += offset + 1,
            None => return false,
        }
    }
    true
}

pub fn chapters_from_beats(beats: &[Beat], words: &[WordStamp]) -> Vec<Chapter> {
    let mut matched: Vec<(usize, &Beat)> = Vec::new();
    let mut cursor = 0;
    for beat in beats {
        let tokens: Vec<String> = beat
            .text
            .split_whitespace()
            .map(normalize_token)
            .filter(|token| !token.is_empty())
            .collect();
        if tokens.is_empty() {
            continue;
        }
        if let Some(index) = best_window(&words[cursor..], &tokens) {
            matched.push((cursor + index, beat));
            cursor += index + 1;
        }
    }
    let last_end = words.last().map(|word| word.end).unwrap_or(0.0);
    matched
        .iter()
        .enumerate()
        .map(|(index, (word_index, beat))| {
            let start = words[*word_index].start;
            let end = matched
                .get(index + 1)
                .map(|(next, _)| words[*next].start)
                .unwrap_or(last_end);
            Chapter {
                kind: beat.kind,
                start,
                end,
                text: beat.text.clone(),
            }
        })
        .collect()
}

fn best_window(words: &[WordStamp], tokens: &[String]) -> Option<usize> {
    let mut best: Option<(usize, usize)> = None;
    for start in 0..words.len() {
        let mut score = 0;
        for (offset, token) in tokens.iter().enumerate() {
            if words
                .get(start + offset)
                .is_some_and(|word| normalize_token(&word.text) == *token)
            {
                score += 1;
            }
        }
        if score == 0 {
            continue;
        }
        if best.is_none_or(|(_, best_score)| score > best_score) {
            best = Some((start, score));
        }
    }
    best.map(|(start, _)| start)
}

pub fn shift_words(words: &[WordStamp], delta: f64) -> Vec<WordStamp> {
    words
        .iter()
        .map(|word| WordStamp {
            text: word.text.clone(),
            start: word.start + delta,
            end: word.end + delta,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::BeatKind;

    fn word(text: &str, start: f64, end: f64) -> WordStamp {
        WordStamp {
            text: text.to_string(),
            start,
            end,
        }
    }

    fn known_words() -> Vec<WordStamp> {
        let raw = include_str!("../fixtures/known-words.json");
        let words: Vec<WordStamp> = serde_json::from_str(raw).unwrap();
        words
    }

    #[test]
    fn fixture_words_land_within_150ms() {
        let expected = known_words();
        let close = shift_words(&expected, 0.12);
        assert!(words_within(&close, &expected, WORD_TOLERANCE_SECS));
        let late = shift_words(&expected, 0.20);
        assert!(!words_within(&late, &expected, WORD_TOLERANCE_SECS));
    }

    #[test]
    fn synthetic_audio_finds_fillers_and_a_silence() {
        let sample_rate = 16_000u32;
        let mut samples = vec![0.0f32; sample_rate as usize * 2];
        for sample in &mut samples[..sample_rate as usize * 3 / 10] {
            *sample = 0.5;
        }
        for sample in &mut samples[sample_rate as usize * 12 / 10..sample_rate as usize * 16 / 10] {
            *sample = 0.5;
        }
        let silences = silences_in_pcm(&samples, sample_rate, SILENCE_MIN_SECS);
        assert_eq!(silences.len(), 1);
        assert!(silences[0].end - silences[0].start >= SILENCE_MIN_SECS);
        assert!(silences[0].start >= 0.2 && silences[0].start < 0.5);

        let words = vec![
            word("Umm,", 0.05, 0.25),
            word("you", 1.25, 1.40),
            word("know", 1.42, 1.55),
            word("build", 1.60, 1.90),
        ];
        let hits = fillers_in_words(&words, DEFAULT_FILLERS);
        assert_eq!(hits.len(), 2);
        assert_eq!(hits[0].text, "umm");
        assert_eq!(hits[1].text, "you know");
    }

    #[test]
    fn chapters_follow_script_beats() {
        let beats = vec![
            Beat {
                kind: BeatKind::Hook,
                start: 0.0,
                text: "this file survives".to_string(),
                cues: Vec::new(),
            },
            Beat {
                kind: BeatKind::Cta,
                start: 2.0,
                text: "build the next short".to_string(),
                cues: Vec::new(),
            },
        ];
        let words = vec![
            word("this", 0.10, 0.30),
            word("file", 0.40, 0.70),
            word("survives", 0.80, 1.20),
            word("build", 2.10, 2.40),
            word("the", 2.45, 2.55),
            word("next", 2.60, 2.90),
            word("short", 3.00, 3.40),
        ];
        let chapters = chapters_from_beats(&beats, &words);
        assert_eq!(chapters.len(), 2);
        assert_eq!(chapters[0].kind, BeatKind::Hook);
        assert!((chapters[0].start - 0.10).abs() < 0.001);
        assert!((chapters[0].end - 2.10).abs() < 0.001);
        assert_eq!(chapters[1].kind, BeatKind::Cta);
        assert!((chapters[1].end - 3.40).abs() < 0.001);
    }
}
