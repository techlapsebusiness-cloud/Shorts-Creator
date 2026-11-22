//! Today's Short. The pipeline pauses only on the steps marked YOU.

use crate::{WordStamp, fillers_in_words};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Actor {
    Ai,
    You,
    Auto,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DailyStep {
    pub id: &'static str,
    pub label: &'static str,
    pub actor: Actor,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StudioFault {
    FfmpegMissing,
    PermissionDenied,
    DiskFull,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FirstRun {
    pub permissions: bool,
    pub whisper: bool,
    pub api_keys: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AutoEdit {
    pub cut_fillers: bool,
    pub cut_silences: bool,
    pub captions: bool,
    pub cursor_zoom: bool,
    pub ducking: bool,
}

const STEPS: &[DailyStep] = &[
    DailyStep {
        id: "topic",
        label: "Pick a topic",
        actor: Actor::Ai,
    },
    DailyStep {
        id: "script",
        label: "Generate the script",
        actor: Actor::Ai,
    },
    DailyStep {
        id: "review",
        label: "Review the script",
        actor: Actor::You,
    },
    DailyStep {
        id: "record",
        label: "Record with the prompter",
        actor: Actor::You,
    },
    DailyStep {
        id: "edit",
        label: "Cut fillers, captions, zoom, ducking",
        actor: Actor::Auto,
    },
    DailyStep {
        id: "assets",
        label: "Cover and end card",
        actor: Actor::Auto,
    },
    DailyStep {
        id: "export",
        label: "Export 4K",
        actor: Actor::Auto,
    },
    DailyStep {
        id: "kit",
        label: "Write the upload kit",
        actor: Actor::Auto,
    },
];

pub fn daily_steps() -> &'static [DailyStep] {
    STEPS
}

pub fn pauses(step: &DailyStep) -> bool {
    step.actor == Actor::You
}

pub fn next_pause(done: &[&str]) -> Option<&'static DailyStep> {
    STEPS
        .iter()
        .find(|step| pauses(step) && !done.contains(&step.id))
}

pub fn auto_edit() -> AutoEdit {
    AutoEdit {
        cut_fillers: true,
        cut_silences: true,
        captions: true,
        cursor_zoom: true,
        ducking: true,
    }
}

pub fn filler_count(words: &[WordStamp]) -> usize {
    fillers_in_words(words, crate::DEFAULT_FILLERS).len()
}

pub fn captions_srt(words: &[WordStamp]) -> String {
    let mut cues = String::new();
    for (index, chunk) in words.chunks(5).enumerate() {
        let Some(first) = chunk.first() else { continue };
        let last = chunk.last().unwrap_or(first);
        let text = chunk
            .iter()
            .map(|word| word.text.as_str())
            .collect::<Vec<_>>()
            .join(" ");
        cues.push_str(&format!(
            "{}\n{} --> {}\n{text}\n\n",
            index + 1,
            stamp(first.start),
            stamp(last.end)
        ));
    }
    cues
}

pub fn classify_fault(message: &str) -> Option<StudioFault> {
    let lower = message.to_lowercase();
    if lower.contains("ffmpeg") {
        Some(StudioFault::FfmpegMissing)
    } else if lower.contains("permission") || lower.contains("denied") {
        Some(StudioFault::PermissionDenied)
    } else if lower.contains("disk") || lower.contains("no space") {
        Some(StudioFault::DiskFull)
    } else {
        None
    }
}

pub fn explain(fault: StudioFault) -> &'static str {
    match fault {
        StudioFault::FfmpegMissing => "ffmpeg is not installed. Install it, then run just doctor.",
        StudioFault::PermissionDenied => "Screen or microphone permission was denied.",
        StudioFault::DiskFull => "The disk does not have enough free space for this take.",
    }
}

pub fn first_run_ready(state: FirstRun) -> bool {
    state.permissions && state.whisper && state.api_keys
}

fn stamp(seconds: f64) -> String {
    let total_ms = (seconds.max(0.0) * 1000.0).round() as u64;
    let hours = total_ms / 3_600_000;
    let minutes = (total_ms / 60_000) % 60;
    let secs = (total_ms / 1000) % 60;
    let millis = total_ms % 1000;
    format!("{hours:02}:{minutes:02}:{secs:02},{millis:03}")
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, Instant};

    use super::*;
    use crate::{TimelineSpan, normalize_spans};

    #[test]
    fn the_pipeline_pauses_only_for_review_and_recording() {
        let pauses: Vec<_> = daily_steps()
            .iter()
            .filter(|step| pauses(step))
            .map(|step| step.id)
            .collect();
        assert_eq!(pauses, ["review", "record"]);
        assert_eq!(next_pause(&[]).unwrap().id, "review");
        assert_eq!(next_pause(&["review"]).unwrap().id, "record");
        assert!(next_pause(&["review", "record"]).is_none());
        let edit = auto_edit();
        assert!(edit.cut_fillers && edit.captions && edit.ducking && edit.cursor_zoom);
        let started = Instant::now();
        let _ = daily_steps();
        assert!(started.elapsed() < Duration::from_secs(2));
    }

    #[test]
    fn ten_clips_stay_inside_one_frame_and_faults_are_named() {
        let spans: Vec<TimelineSpan> = (0..10)
            .map(|index| TimelineSpan {
                id: index.to_string(),
                start_ms: index * 500,
                duration_ms: 800,
            })
            .collect();
        let started = Instant::now();
        for _ in 0..60 {
            assert!(crate::spans_are_ordered(&normalize_spans(spans.clone())));
        }
        assert!(started.elapsed() < Duration::from_millis(16));
        assert_eq!(
            classify_fault("ffmpeg not found"),
            Some(StudioFault::FfmpegMissing)
        );
        assert_eq!(
            classify_fault("permission denied"),
            Some(StudioFault::PermissionDenied)
        );
        assert_eq!(
            classify_fault("no space left on disk"),
            Some(StudioFault::DiskFull)
        );
        assert!(!explain(StudioFault::DiskFull).is_empty());
        assert!(!first_run_ready(FirstRun {
            permissions: true,
            whisper: false,
            api_keys: true
        }));
        assert!(first_run_ready(FirstRun {
            permissions: true,
            whisper: true,
            api_keys: true
        }));
        let words = [
            WordStamp {
                text: "umm".into(),
                start: 0.0,
                end: 0.2,
            },
            WordStamp {
                text: "survives".into(),
                start: 0.2,
                end: 0.8,
            },
        ];
        assert_eq!(filler_count(&words), 1);
        let srt = captions_srt(&words);
        assert!(srt.contains("00:00:00,000"));
        assert!(srt.contains("survives"));
    }
}
