//! Teleprompter scroll, voice-follow matching, and the Linux overlap rule.

pub const FOLLOW_CONFIDENCE: f64 = 0.6;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PxRect {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CaptureFrame {
    /// A whole monitor. The portal cannot drop one window from this.
    Monitor { id: u32 },
    /// A window or region on `monitor_id`.
    Region { monitor_id: u32, rect: PxRect },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScrollMode {
    Follow,
    Fixed,
    Paused,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LineMatch {
    pub index: usize,
    pub confidence: f64,
}

pub fn scroll_offset_px(elapsed_ms: f64, paused_ms: f64, px_per_sec: f64) -> f64 {
    let active = (elapsed_ms - paused_ms).max(0.0);
    px_per_sec * active / 1000.0
}

pub fn nudge_offset(offset_px: f64, delta_px: f64) -> f64 {
    (offset_px + delta_px).max(0.0)
}

pub fn scroll_mode(confidence: f64, silent: bool) -> ScrollMode {
    if silent {
        ScrollMode::Paused
    } else if confidence >= FOLLOW_CONFIDENCE {
        ScrollMode::Follow
    } else {
        ScrollMode::Fixed
    }
}

pub fn follow_offset_px(line_index: usize, line_height_px: f64) -> f64 {
    line_index as f64 * line_height_px
}

pub fn fuzzy_match(lines: &[&str], heard: &str) -> LineMatch {
    let heard_tokens = tokens(heard);
    if heard_tokens.is_empty() || lines.is_empty() {
        return LineMatch {
            index: 0,
            confidence: 0.0,
        };
    }
    let mut best = LineMatch {
        index: 0,
        confidence: 0.0,
    };
    for (index, line) in lines.iter().enumerate() {
        let line_tokens = tokens(line);
        if line_tokens.is_empty() {
            continue;
        }
        let shared = heard_tokens
            .iter()
            .filter(|token| line_tokens.iter().any(|other| other == *token))
            .count();
        let confidence = shared as f64 / heard_tokens.len() as f64;
        if confidence > best.confidence {
            best = LineMatch { index, confidence };
        }
    }
    best
}

/// Linux cannot exclude one window from a monitor capture.
pub fn allow_linux_capture(
    prompter_monitor: u32,
    prompter: PxRect,
    target: CaptureFrame,
) -> Result<(), String> {
    match target {
        CaptureFrame::Monitor { id } if id != prompter_monitor => Ok(()),
        CaptureFrame::Monitor { id } => Err(format!(
            "Linux cannot hide the prompter on monitor {id}. Move it to another monitor, or capture a window or region that does not overlap it."
        )),
        CaptureFrame::Region { monitor_id, rect } => {
            if monitor_id != prompter_monitor || !rects_overlap(prompter, rect) {
                Ok(())
            } else {
                Err(
                    "The capture region overlaps the prompter. Move one of them, or capture a different monitor."
                        .to_string(),
                )
            }
        }
    }
}

pub fn rects_overlap(a: PxRect, b: PxRect) -> bool {
    let ax2 = a.x.saturating_add(a.width);
    let ay2 = a.y.saturating_add(a.height);
    let bx2 = b.x.saturating_add(b.width);
    let by2 = b.y.saturating_add(b.height);
    a.x < bx2 && b.x < ax2 && a.y < by2 && b.y < ay2
}

pub fn frame_contains_magenta(rgb: &[u8]) -> bool {
    rgb.as_chunks::<3>()
        .0
        .iter()
        .any(|pixel| pixel[0] > 200 && pixel[1] < 40 && pixel[2] > 200)
}

fn tokens(text: &str) -> Vec<String> {
    text.split(|ch: char| !ch.is_alphanumeric())
        .filter(|token| !token.is_empty())
        .map(|token| token.to_lowercase())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn time_becomes_a_scroll_offset() {
        assert!((scroll_offset_px(0.0, 0.0, 100.0)).abs() < 0.000_001);
        assert!((scroll_offset_px(1_000.0, 0.0, 120.0) - 120.0).abs() < 0.000_001);
        assert!((scroll_offset_px(2_000.0, 500.0, 100.0) - 150.0).abs() < 0.000_001);
        assert_eq!(nudge_offset(10.0, -40.0), 0.0);
        assert!((nudge_offset(10.0, 5.0) - 15.0).abs() < 0.000_001);
    }

    #[test]
    fn hinglish_lines_match_and_low_confidence_falls_back() {
        let lines = [
            "yeh file crash ke baad bhi bachegi",
            "project.json mein edit rehti hai",
            "agla short banao",
        ];
        let hit = fuzzy_match(&lines, "file crash ke baad bachegi");
        assert_eq!(hit.index, 0);
        assert!(hit.confidence >= FOLLOW_CONFIDENCE);
        assert_eq!(scroll_mode(hit.confidence, false), ScrollMode::Follow);
        let miss = fuzzy_match(&lines, "asdf qwer zxcv");
        assert!(miss.confidence < FOLLOW_CONFIDENCE);
        assert_eq!(scroll_mode(miss.confidence, false), ScrollMode::Fixed);
        assert_eq!(scroll_mode(1.0, true), ScrollMode::Paused);
        assert!((follow_offset_px(2, 48.0) - 96.0).abs() < 0.000_001);
    }

    #[test]
    fn linux_refuses_an_overlapping_monitor_capture() {
        let prompter = PxRect {
            x: 100,
            y: 100,
            width: 400,
            height: 300,
        };
        let refused = allow_linux_capture(0, prompter, CaptureFrame::Monitor { id: 0 });
        assert!(refused.unwrap_err().contains("another monitor"));
        assert!(allow_linux_capture(0, prompter, CaptureFrame::Monitor { id: 1 }).is_ok());
        let beside = PxRect {
            x: 600,
            y: 100,
            width: 200,
            height: 200,
        };
        assert!(
            allow_linux_capture(
                0,
                prompter,
                CaptureFrame::Region {
                    monitor_id: 0,
                    rect: beside,
                },
            )
            .is_ok()
        );
        let over = PxRect {
            x: 200,
            y: 150,
            width: 50,
            height: 50,
        };
        assert!(
            allow_linux_capture(
                0,
                prompter,
                CaptureFrame::Region {
                    monitor_id: 0,
                    rect: over,
                },
            )
            .is_err()
        );
    }

    #[test]
    fn magenta_pixels_are_recognised() {
        assert!(frame_contains_magenta(&[255, 0, 255]));
        assert!(!frame_contains_magenta(&[255, 0, 0, 0, 255, 0, 0, 0, 255]));
        assert!(frame_contains_magenta(&[10, 10, 10, 255, 0, 255]));
    }
}
