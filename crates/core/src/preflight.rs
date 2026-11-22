//! Export checks. A failed check is shown. It never blocks the render.

use serde::Deserialize;

use crate::{CamPosition, CamSize, cam_rect, caption_rect, loudness_ok, safe_rect};

pub const TRIGRAM_WARN: f64 = 0.4;
pub const CHANGE_WARN: f64 = 0.2;

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct PolicyPack {
    pub max_duration_secs: f64,
    pub checked_on: String,
    pub source: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PastShort {
    pub topic: String,
    pub title: String,
    pub layout_key: String,
}

#[derive(Debug, Clone)]
pub struct ShotCheck {
    pub width: u32,
    pub height: u32,
    pub duration_secs: f64,
    pub lufs: Option<f64>,
    pub true_peak_db: Option<f64>,
    pub cam: CamPosition,
    pub cam_size: CamSize,
    pub safe_zone: bool,
    pub caption_lines: usize,
    pub music_licensed: bool,
    pub topic: String,
    pub title: String,
    pub script: String,
    pub raw_draft: String,
    pub layout_key: String,
    pub history: Vec<PastShort>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CheckState {
    Pass,
    Fail,
    Warn,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Check {
    pub id: &'static str,
    pub label: String,
    pub state: CheckState,
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Preflight {
    pub policy: PolicyPack,
    pub checks: Vec<Check>,
}

pub fn bundled_policy() -> PolicyPack {
    serde_json::from_str(include_str!("../../../policy/youtube-shorts.json")).expect("policy pack")
}

pub fn preflight(shot: &ShotCheck) -> Preflight {
    let policy = bundled_policy();
    let mut checks = vec![
        aspect(shot),
        duration(shot, policy.max_duration_secs),
        loudness(shot),
        placement(shot),
        licence(shot),
    ];
    checks.extend(originality(shot));
    Preflight { policy, checks }
}

pub fn blocks(_report: &Preflight) -> bool {
    false
}

pub fn trigram_jaccard(left: &str, right: &str) -> f64 {
    let left = trigrams(left);
    let right = trigrams(right);
    let union = left.union(&right).count();
    if union == 0 {
        0.0
    } else {
        left.intersection(&right).count() as f64 / union as f64
    }
}

pub fn change_ratio(draft: &str, script: &str) -> f64 {
    let left = words(draft);
    let right = words(script);
    let denom = left.len().max(right.len()).max(1) as f64;
    levenshtein(&left, &right) as f64 / denom
}

fn aspect(shot: &ShotCheck) -> Check {
    let ratio = f64::from(shot.height) / f64::from(shot.width.max(1));
    let ok = (ratio - 16.0 / 9.0).abs() < 0.02;
    mark(
        "aspect",
        "Vertical 9:16",
        ok,
        format!("{:.0}×{:.0}", shot.width, shot.height),
    )
}

fn duration(shot: &ShotCheck, max_secs: f64) -> Check {
    mark(
        "length",
        format!("Under {max_secs:.0} seconds"),
        shot.duration_secs <= max_secs,
        format!("{:.0}s", shot.duration_secs),
    )
}

fn loudness(shot: &ShotCheck) -> Check {
    match (shot.lufs, shot.true_peak_db) {
        (Some(lufs), Some(peak)) => mark(
            "loudness",
            "Loudness -14 LUFS",
            loudness_ok(lufs, peak),
            format!("{lufs:.1} LUFS, {peak:.1} dBTP"),
        ),
        _ => Check {
            id: "loudness",
            label: "Loudness -14 LUFS".to_string(),
            state: CheckState::Warn,
            detail: "Not measured yet".to_string(),
        },
    }
}

fn placement(shot: &ShotCheck) -> Check {
    let frame_w = f64::from(shot.width);
    let frame_h = f64::from(shot.height);
    let safe = safe_rect(frame_w, frame_h);
    let cam = cam_rect(frame_w, frame_h, shot.cam, shot.cam_size, shot.safe_zone);
    let captions = caption_rect(frame_w, frame_h, shot.caption_lines, shot.safe_zone);
    let inside = rect_inside(safe, cam) && rect_inside(safe, captions);
    mark(
        "safe",
        "Captions and camera inside the safe zone",
        inside,
        if shot.safe_zone {
            "Safe zone on".to_string()
        } else {
            "Safe zone off".to_string()
        },
    )
}

fn licence(shot: &ShotCheck) -> Check {
    if shot.music_licensed {
        mark("licence", "Music licence", true, "Ticked".to_string())
    } else {
        Check {
            id: "licence",
            label: "Music licence".to_string(),
            state: CheckState::Warn,
            detail: "Tick this. It does not block export.".to_string(),
        }
    }
}

fn originality(shot: &ShotCheck) -> Vec<Check> {
    let recent: Vec<&PastShort> = shot.history.iter().rev().take(10).collect();
    let similar = recent.iter().any(|past| {
        trigram_jaccard(&shot.topic, &past.topic) >= TRIGRAM_WARN
            || trigram_jaccard(&shot.title, &past.title) >= TRIGRAM_WARN
    });
    let changed = change_ratio(&shot.raw_draft, &shot.script);
    let repeated = recent.iter().any(|past| past.layout_key == shot.layout_key);
    vec![
        if similar {
            Check {
                id: "similar",
                label: "Topic and title".to_string(),
                state: CheckState::Warn,
                detail: "Close to one of the last 10 Shorts.".to_string(),
            }
        } else {
            mark(
                "similar",
                "Topic and title",
                true,
                "Distinct from the last 10".to_string(),
            )
        },
        if changed < CHANGE_WARN {
            Check {
                id: "draft",
                label: "Script vs the raw draft".to_string(),
                state: CheckState::Warn,
                detail: format!("{:.0}% changed. Under 20% is a warning.", changed * 100.0),
            }
        } else {
            mark(
                "draft",
                "Script vs the raw draft",
                true,
                format!("{:.0}% changed", changed * 100.0),
            )
        },
        if repeated {
            Check {
                id: "layout",
                label: "Layout and template".to_string(),
                state: CheckState::Warn,
                detail: "Same layout as a recent Short.".to_string(),
            }
        } else {
            mark(
                "layout",
                "Layout and template",
                true,
                "Not a repeat".to_string(),
            )
        },
    ]
}

fn mark(id: &'static str, label: impl Into<String>, ok: bool, detail: String) -> Check {
    Check {
        id,
        label: label.into(),
        state: if ok {
            CheckState::Pass
        } else {
            CheckState::Fail
        },
        detail,
    }
}

fn rect_inside(outer: crate::Rect, inner: crate::Rect) -> bool {
    inner.x + 0.5 >= outer.x
        && inner.y + 0.5 >= outer.y
        && inner.x + inner.width <= outer.x + outer.width + 0.5
        && inner.y + inner.height <= outer.y + outer.height + 0.5
}

fn trigrams(text: &str) -> std::collections::BTreeSet<String> {
    let folded: String = text
        .chars()
        .flat_map(|ch| ch.to_lowercase())
        .filter(|ch| ch.is_alphanumeric() || ch.is_whitespace())
        .collect();
    let compact = folded.split_whitespace().collect::<Vec<_>>().join(" ");
    let chars: Vec<char> = compact.chars().collect();
    if chars.len() < 3 {
        return std::iter::once(compact).collect();
    }
    chars
        .windows(3)
        .map(|window| window.iter().collect())
        .collect()
}

fn words(text: &str) -> Vec<String> {
    text.split_whitespace()
        .map(|word| word.to_lowercase())
        .collect()
}

fn levenshtein(left: &[String], right: &[String]) -> usize {
    let mut previous: Vec<usize> = (0..=right.len()).collect();
    let mut current = vec![0; right.len() + 1];
    for (row, left_word) in left.iter().enumerate() {
        current[0] = row + 1;
        for (col, right_word) in right.iter().enumerate() {
            let cost = usize::from(left_word != right_word);
            current[col + 1] = (current[col] + 1)
                .min(previous[col + 1] + 1)
                .min(previous[col] + cost);
        }
        std::mem::swap(&mut previous, &mut current);
    }
    previous[right.len()]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn good() -> ShotCheck {
        ShotCheck {
            width: 1080,
            height: 1920,
            duration_secs: 42.0,
            lufs: Some(crate::TARGET_LUFS),
            true_peak_db: Some(-1.4),
            cam: CamPosition::BottomRight,
            cam_size: CamSize::L,
            safe_zone: true,
            caption_lines: 2,
            music_licensed: true,
            topic: "atomic rename".into(),
            title: "Rename without losing the file".into(),
            script: "The temp name is the trick and the real file appears last.".into(),
            raw_draft: "Just rename the file quickly and hope.".into(),
            layout_key: "screen-cam|hook".into(),
            history: vec![PastShort {
                topic: "camera exposure".into(),
                title: "Expose for the face".into(),
                layout_key: "cam-only|error".into(),
            }],
        }
    }

    fn bad() -> ShotCheck {
        let mut shot = good();
        shot.width = 1920;
        shot.height = 1080;
        shot.duration_secs = 90.0;
        shot.lufs = Some(-24.0);
        shot.true_peak_db = Some(-0.2);
        shot.safe_zone = false;
        shot.cam_size = CamSize::L;
        shot.music_licensed = false;
        shot.script = shot.raw_draft.clone();
        shot.history.push(PastShort {
            topic: shot.topic.clone(),
            title: shot.title.clone(),
            layout_key: shot.layout_key.clone(),
        });
        shot
    }

    fn state<'a>(report: &'a Preflight, id: &str) -> &'a CheckState {
        &report
            .checks
            .iter()
            .find(|check| check.id == id)
            .unwrap()
            .state
    }

    #[test]
    fn each_rule_passes_once_and_fails_once() {
        let ok = preflight(&good());
        let no = preflight(&bad());
        assert_eq!(ok.policy.max_duration_secs, 60.0);
        assert_eq!(ok.policy.checked_on, "2026-11-22");
        for id in [
            "aspect", "length", "loudness", "safe", "licence", "similar", "draft", "layout",
        ] {
            assert_eq!(state(&ok, id), &CheckState::Pass, "{id}");
            assert_ne!(state(&no, id), &CheckState::Pass, "{id}");
        }
        assert!(!blocks(&no));
        assert!(!blocks(&ok));
    }

    #[test]
    fn a_near_copy_warns_and_a_distinct_short_stays_quiet() {
        let quiet = preflight(&good());
        assert_eq!(state(&quiet, "similar"), &CheckState::Pass);
        let mut near = good();
        near.history.push(PastShort {
            topic: near.topic.clone(),
            title: "Something else entirely here".into(),
            layout_key: "other".into(),
        });
        near.script = near.raw_draft.clone();
        let warned = preflight(&near);
        assert_eq!(state(&warned, "similar"), &CheckState::Warn);
        assert_eq!(state(&warned, "draft"), &CheckState::Warn);
        assert_eq!(state(&warned, "layout"), &CheckState::Pass);
    }
}
