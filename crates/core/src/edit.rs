//! Non-destructive timeline edits. Times inside this module are integer milliseconds.

use std::collections::VecDeque;

use crate::order::TimelineSpan;
use crate::{Clip, ClipBody};

pub const SPEED_MIN: f64 = 0.25;
pub const SPEED_MAX: f64 = 4.0;
pub const PROXY_WIDTH: u32 = 1280;
pub const PROXY_HEIGHT: u32 = 720;
pub const UNDO_LIMIT: usize = 100;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EditError {
    pub message: String,
}

impl std::fmt::Display for EditError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for EditError {}

fn error(message: impl Into<String>) -> EditError {
    EditError {
        message: message.into(),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct History<T> {
    past: VecDeque<T>,
    present: T,
    future: VecDeque<T>,
    limit: usize,
}

impl<T: Clone> History<T> {
    pub fn new(present: T) -> Self {
        Self {
            past: VecDeque::new(),
            present,
            future: VecDeque::new(),
            limit: UNDO_LIMIT,
        }
    }

    pub fn present(&self) -> &T {
        &self.present
    }

    pub fn edit(&mut self, next: T) {
        self.past.push_back(self.present.clone());
        if self.past.len() > self.limit {
            self.past.pop_front();
        }
        self.present = next;
        self.future.clear();
    }

    pub fn undo(&mut self) -> bool {
        let Some(previous) = self.past.pop_back() else {
            return false;
        };
        self.future.push_back(self.present.clone());
        self.present = previous;
        true
    }

    pub fn redo(&mut self) -> bool {
        let Some(next) = self.future.pop_back() else {
            return false;
        };
        self.past.push_back(self.present.clone());
        self.present = next;
        true
    }
}

pub fn total_duration_ms(spans: &[TimelineSpan]) -> u64 {
    spans.iter().map(|span| span.duration_ms).sum()
}

pub fn split_spans(
    spans: &[TimelineSpan],
    id: &str,
    at_ms: i64,
) -> Result<Vec<TimelineSpan>, EditError> {
    let index = spans
        .iter()
        .position(|span| span.id == id)
        .ok_or_else(|| error(format!("missing clip {id}")))?;
    let span = &spans[index];
    let end = span.start_ms.saturating_add(span.duration_ms as i64);
    if at_ms <= span.start_ms || at_ms >= end {
        return Err(error("split point is outside the clip"));
    }
    let left_duration = (at_ms - span.start_ms) as u64;
    let right_duration = span.duration_ms - left_duration;
    let left = TimelineSpan {
        id: format!("{id}-a"),
        start_ms: span.start_ms,
        duration_ms: left_duration,
    };
    let right = TimelineSpan {
        id: format!("{id}-b"),
        start_ms: at_ms,
        duration_ms: right_duration,
    };
    let mut next = spans.to_vec();
    next.splice(index..index + 1, [left, right]);
    Ok(next)
}

pub fn trim_span(
    spans: &[TimelineSpan],
    id: &str,
    new_duration_ms: u64,
) -> Result<Vec<TimelineSpan>, EditError> {
    if new_duration_ms == 0 {
        return Err(error("a clip duration cannot be zero"));
    }
    let index = spans
        .iter()
        .position(|span| span.id == id)
        .ok_or_else(|| error(format!("missing clip {id}")))?;
    let mut next = spans.to_vec();
    let start = next[index].start_ms;
    let new_end = start.saturating_add(new_duration_ms as i64);
    for (other_index, other) in next.iter().enumerate() {
        if other_index == index {
            continue;
        }
        let other_end = other.start_ms.saturating_add(other.duration_ms as i64);
        if start < other_end && other.start_ms < new_end {
            return Err(error("trim would overlap another clip"));
        }
    }
    next[index].duration_ms = new_duration_ms;
    Ok(next)
}

pub fn ripple_delete(spans: &[TimelineSpan], id: &str) -> Result<Vec<TimelineSpan>, EditError> {
    let span = spans
        .iter()
        .find(|span| span.id == id)
        .cloned()
        .ok_or_else(|| error(format!("missing clip {id}")))?;
    let end = span.start_ms.saturating_add(span.duration_ms as i64);
    let next = spans
        .iter()
        .filter(|item| item.id != id)
        .cloned()
        .map(|mut item| {
            if item.start_ms >= end {
                item.start_ms -= span.duration_ms as i64;
            }
            item
        })
        .collect();
    Ok(next)
}

pub fn move_span(
    spans: &[TimelineSpan],
    id: &str,
    new_start_ms: i64,
) -> Result<Vec<TimelineSpan>, EditError> {
    if new_start_ms < 0 {
        return Err(error("a clip cannot start before zero"));
    }
    let index = spans
        .iter()
        .position(|span| span.id == id)
        .ok_or_else(|| error(format!("missing clip {id}")))?;
    let mut next = spans.to_vec();
    let duration = next[index].duration_ms;
    let new_end = new_start_ms.saturating_add(duration as i64);
    for (other_index, other) in next.iter().enumerate() {
        if other_index == index {
            continue;
        }
        let other_end = other.start_ms.saturating_add(other.duration_ms as i64);
        if new_start_ms < other_end && other.start_ms < new_end {
            return Err(error("move would overlap another clip"));
        }
    }
    next[index].start_ms = new_start_ms;
    next.sort_by_key(|span| span.start_ms);
    Ok(next)
}

pub fn snap_time(time_ms: i64, edges_ms: &[i64], threshold_ms: i64) -> i64 {
    edges_ms
        .iter()
        .copied()
        .filter(|edge| (edge - time_ms).abs() <= threshold_ms)
        .min_by_key(|edge| (edge - time_ms).abs())
        .unwrap_or(time_ms)
}

pub fn clamp_speed(speed: f64) -> f64 {
    speed.clamp(SPEED_MIN, SPEED_MAX)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransitionKind {
    Cut,
    Fade,
    Wipe,
    Glitch,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Marker {
    pub id: String,
    pub time_ms: i64,
}

pub fn toggle_selection(selected: &[String], id: &str) -> Vec<String> {
    if selected.iter().any(|item| item == id) {
        selected
            .iter()
            .filter(|item| item.as_str() != id)
            .cloned()
            .collect()
    } else {
        let mut next = selected.to_vec();
        next.push(id.to_string());
        next
    }
}

pub fn waveform_peaks(samples: &[f32], buckets: usize) -> Vec<f32> {
    if buckets == 0 {
        return Vec::new();
    }
    if samples.is_empty() {
        return vec![0.0; buckets];
    }
    let mut peaks = vec![0.0f32; buckets];
    for (index, sample) in samples.iter().enumerate() {
        let bucket = index * buckets / samples.len();
        let bucket = bucket.min(buckets - 1);
        peaks[bucket] = peaks[bucket].max(sample.abs());
    }
    peaks
}

pub fn proxy_file_name(source: &str) -> String {
    let stem = source
        .rsplit_once('.')
        .map(|(stem, _)| stem)
        .unwrap_or(source);
    format!("{stem}.720p.mp4")
}

pub fn split_clips(clips: &[Clip], id: &str, at: f64) -> Result<Vec<Clip>, EditError> {
    let index = clips
        .iter()
        .position(|clip| clip.id == id)
        .ok_or_else(|| error(format!("missing clip {id}")))?;
    let clip = &clips[index];
    let end = clip.timeline_start + clip.duration;
    if at <= clip.timeline_start || at >= end {
        return Err(error("split point is outside the clip"));
    }
    let local = at - clip.timeline_start;
    let (left_body, right_body) = split_body(&clip.body, local);
    let mut left = clip.clone();
    let mut right = clip.clone();
    left.id = format!("{id}-a");
    right.id = format!("{id}-b");
    left.duration = local;
    right.duration = clip.duration - local;
    right.timeline_start = at;
    left.body = left_body;
    right.body = right_body;
    let mut next = clips.to_vec();
    next.splice(index..index + 1, [left, right]);
    Ok(next)
}

fn split_body(body: &ClipBody, local: f64) -> (ClipBody, ClipBody) {
    match body {
        ClipBody::Video {
            source,
            source_in,
            source_out,
            speed,
        } => {
            let mid = (source_in + local * speed).min(*source_out);
            (
                ClipBody::Video {
                    source: source.clone(),
                    source_in: *source_in,
                    source_out: mid,
                    speed: *speed,
                },
                ClipBody::Video {
                    source: source.clone(),
                    source_in: mid,
                    source_out: *source_out,
                    speed: *speed,
                },
            )
        }
        ClipBody::Audio {
            source,
            source_in,
            source_out,
            speed,
        } => {
            let mid = (source_in + local * speed).min(*source_out);
            (
                ClipBody::Audio {
                    source: source.clone(),
                    source_in: *source_in,
                    source_out: mid,
                    speed: *speed,
                },
                ClipBody::Audio {
                    source: source.clone(),
                    source_in: mid,
                    source_out: *source_out,
                    speed: *speed,
                },
            )
        }
        other => (other.clone(), other.clone()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::order::spans_are_ordered;
    use proptest::prelude::*;

    fn span(id: &str, start_ms: i64, duration_ms: u64) -> TimelineSpan {
        TimelineSpan {
            id: id.to_string(),
            start_ms,
            duration_ms,
        }
    }

    fn ordered_track() -> impl Strategy<Value = Vec<TimelineSpan>> {
        prop::collection::vec((1u64..1_500, 0u64..400), 1..8).prop_map(|parts| {
            let mut start = 0i64;
            parts
                .into_iter()
                .enumerate()
                .map(|(index, (duration_ms, gap))| {
                    let span = span(&format!("c{index}"), start, duration_ms);
                    start += duration_ms as i64 + gap as i64;
                    span
                })
                .collect()
        })
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(48))]
        #[test]
        fn split_trim_and_ripple_keep_order_and_duration(
            track in ordered_track(),
            index in 0usize..8
        ) {
            let index = index % track.len();
            let clip = &track[index];
            prop_assume!(clip.duration_ms >= 2);
            let at = clip.start_ms + 1;
            let split = split_spans(&track, &clip.id, at).unwrap();
            prop_assert!(spans_are_ordered(&split));
            prop_assert_eq!(total_duration_ms(&split), total_duration_ms(&track));

            let trimmed = trim_span(&track, &clip.id, clip.duration_ms - 1).unwrap();
            prop_assert!(spans_are_ordered(&trimmed));
            prop_assert_eq!(total_duration_ms(&trimmed), total_duration_ms(&track) - 1);

            let deleted = ripple_delete(&track, &clip.id).unwrap();
            prop_assert!(spans_are_ordered(&deleted));
            prop_assert_eq!(
                total_duration_ms(&deleted),
                total_duration_ms(&track) - clip.duration_ms
            );
        }
    }

    #[test]
    fn undo_then_redo_restores_the_track() {
        let mut history = History::new(vec![span("a", 0, 1_000)]);
        let split = split_spans(history.present(), "a", 400).unwrap();
        history.edit(split);
        assert!(history.undo());
        assert_eq!(history.present(), &vec![span("a", 0, 1_000)]);
        assert!(history.redo());
        assert_eq!(history.present().len(), 2);
    }

    #[test]
    fn undo_keeps_only_one_hundred_steps() {
        let mut history = History::new(0u32);
        for step in 1..=101 {
            history.edit(step);
        }
        let mut undone = 0;
        while history.undo() {
            undone += 1;
        }
        assert_eq!(undone, 100);
        assert_eq!(*history.present(), 1);
    }

    #[test]
    fn split_clip_divides_source_time() {
        let clip = Clip {
            id: "screen".to_string(),
            timeline_start: 0.0,
            duration: 4.0,
            body: ClipBody::Video {
                source: "media/screen.mov".to_string(),
                source_in: 1.0,
                source_out: 5.0,
                speed: 1.0,
            },
            keyframes: Vec::new(),
        };
        let split = split_clips(&[clip], "screen", 2.0).unwrap();
        assert_eq!(split.len(), 2);
        assert!((split[0].duration - 2.0).abs() < 0.0001);
        assert!((split[1].timeline_start - 2.0).abs() < 0.0001);
        match &split[0].body {
            ClipBody::Video { source_out, .. } => assert!((source_out - 3.0).abs() < 0.0001),
            other => panic!("unexpected body {other:?}"),
        }
        match &split[1].body {
            ClipBody::Video { source_in, .. } => assert!((source_in - 3.0).abs() < 0.0001),
            other => panic!("unexpected body {other:?}"),
        }
    }

    #[test]
    fn speed_snaps_and_peaks() {
        assert_eq!(clamp_speed(0.1), SPEED_MIN);
        assert_eq!(clamp_speed(9.0), SPEED_MAX);
        assert_eq!(snap_time(12, &[0, 10, 40], 5), 10);
        let peaks = waveform_peaks(&[0.0, 0.2, 0.0, 0.8], 2);
        assert_eq!(peaks.len(), 2);
        assert!(peaks[1] >= 0.8);
        assert_eq!(proxy_file_name("media/screen.mov"), "media/screen.720p.mp4");
        assert_eq!(
            toggle_selection(&["a".to_string()], "b"),
            vec!["a".to_string(), "b".to_string()]
        );
        assert!(toggle_selection(&["a".to_string()], "a").is_empty());
    }
}
