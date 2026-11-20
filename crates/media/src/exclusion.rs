//! Whether a capture is allowed to start while the prompter is on screen.

use std::fmt;

pub const PROMPTER_TITLE: &str = "Prompter";
pub const MAGENTA: [u8; 3] = [255, 0, 255];
const MAGENTA_PIXELS: usize = 64;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PrompterPlacement {
    pub monitor_id: u32,
    pub bounds: Rect,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Capture {
    Monitor { monitor_id: u32 },
    Window { monitor_id: u32, bounds: Rect },
    Region { monitor_id: u32, bounds: Rect },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RefuseReason {
    MonitorCaptureIncludesPrompter,
    CaptureOverlapsPrompter,
}

impl fmt::Display for RefuseReason {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MonitorCaptureIncludesPrompter => write!(
                formatter,
                "A monitor capture includes the prompter. Move the prompter to another monitor, or capture a window or region that does not overlap it."
            ),
            Self::CaptureOverlapsPrompter => write!(
                formatter,
                "The capture overlaps the prompter. Move the prompter, or capture a window or region that does not overlap it."
            ),
        }
    }
}

pub fn overlaps(left: Rect, right: Rect) -> bool {
    let Some((left_start, left_end)) = span(left.x, left.width) else {
        return false;
    };
    let Some((right_start, right_end)) = span(right.x, right.width) else {
        return false;
    };
    let Some((top, bottom)) = span(left.y, left.height) else {
        return false;
    };
    let Some((other_top, other_bottom)) = span(right.y, right.height) else {
        return false;
    };
    left_start < right_end && right_start < left_end && top < other_bottom && other_top < bottom
}

fn span(origin: i32, length: i32) -> Option<(i64, i64)> {
    if length <= 0 {
        return None;
    }
    let start = i64::from(origin);
    Some((start, start + i64::from(length)))
}

pub fn recording_allowed(
    prompter: PrompterPlacement,
    capture: Capture,
) -> Result<(), RefuseReason> {
    match capture {
        Capture::Monitor { monitor_id } => {
            if monitor_id == prompter.monitor_id {
                Err(RefuseReason::MonitorCaptureIncludesPrompter)
            } else {
                Ok(())
            }
        }
        Capture::Window { monitor_id, bounds } | Capture::Region { monitor_id, bounds } => {
            if monitor_id != prompter.monitor_id || !overlaps(prompter.bounds, bounds) {
                Ok(())
            } else {
                Err(RefuseReason::CaptureOverlapsPrompter)
            }
        }
    }
}

pub fn is_magenta(red: u8, green: u8, blue: u8) -> bool {
    red >= 220 && green <= 30 && blue >= 220
}

pub fn frame_shows_magenta(pixels: &[u8], channels: usize) -> bool {
    if channels < 3 {
        return false;
    }
    let hits = pixels
        .chunks(channels)
        .filter(|pixel| pixel.len() >= 3 && is_magenta(pixel[0], pixel[1], pixel[2]))
        .count();
    hits >= MAGENTA_PIXELS
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MagentaRun {
    Manual(&'static str),
}

pub fn magenta_capture() -> MagentaRun {
    if cfg!(target_os = "macos") {
        MagentaRun::Manual(
            "Screen recording permission is required before the 3 second magenta capture can run. That capture stays MANUAL until it has been seen on the Mini.",
        )
    } else {
        MagentaRun::Manual(
            "The Wayland portal cannot exclude one window from a monitor capture. The 3 second magenta recording is MANUAL on Linux.",
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListedWindow {
    pub id: u32,
    pub owner_pid: i32,
    pub title: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MacExclusion {
    pub window_id: u32,
    pub sharing_type_none: bool,
}

pub fn match_prompter_window(windows: &[ListedWindow], pid: i32, title: &str) -> Option<u32> {
    windows
        .iter()
        .find(|window| window.owner_pid == pid && window.title == title)
        .map(|window| window.id)
}

pub fn mac_exclusion(window_id: u32) -> MacExclusion {
    MacExclusion {
        window_id,
        sharing_type_none: true,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        Capture, ListedWindow, MagentaRun, PrompterPlacement, Rect, frame_shows_magenta,
        mac_exclusion, magenta_capture, match_prompter_window, overlaps, recording_allowed,
    };

    fn prompter(monitor_id: u32) -> PrompterPlacement {
        PrompterPlacement {
            monitor_id,
            bounds: Rect {
                x: 100,
                y: 100,
                width: 400,
                height: 200,
            },
        }
    }

    #[test]
    fn a_monitor_capture_on_the_prompter_display_is_refused() {
        let error = recording_allowed(prompter(1), Capture::Monitor { monitor_id: 1 }).unwrap_err();
        let message = error.to_string();
        assert!(message.contains("another monitor"));
        assert!(message.contains("window or region"));
    }

    #[test]
    fn another_monitor_or_a_clear_region_is_allowed() {
        assert!(recording_allowed(prompter(1), Capture::Monitor { monitor_id: 2 }).is_ok());
        let clear = Rect {
            x: 600,
            y: 100,
            width: 200,
            height: 200,
        };
        assert!(
            recording_allowed(
                prompter(1),
                Capture::Region {
                    monitor_id: 1,
                    bounds: clear,
                },
            )
            .is_ok()
        );
        assert!(
            recording_allowed(
                prompter(1),
                Capture::Window {
                    monitor_id: 2,
                    bounds: Rect {
                        x: 100,
                        y: 100,
                        width: 400,
                        height: 200,
                    },
                },
            )
            .is_ok()
        );
    }

    #[test]
    fn overlap_refuses_and_a_shared_edge_does_not() {
        let overlapping = Rect {
            x: 499,
            y: 100,
            width: 20,
            height: 20,
        };
        let error = recording_allowed(
            prompter(1),
            Capture::Window {
                monitor_id: 1,
                bounds: overlapping,
            },
        )
        .unwrap_err();
        assert!(error.to_string().contains("overlaps"));
        assert!(!overlaps(
            prompter(1).bounds,
            Rect {
                x: 500,
                y: 100,
                width: 20,
                height: 20,
            },
        ));
        assert!(!overlaps(
            prompter(1).bounds,
            Rect {
                x: 100,
                y: 100,
                width: 0,
                height: 20,
            },
        ));
    }

    #[test]
    fn a_magenta_block_is_visible_and_a_clean_frame_is_not() {
        let mut frame = vec![0_u8; 64 * 4];
        assert!(!frame_shows_magenta(&frame, 4));
        for pixel in frame.chunks_mut(4) {
            pixel[0] = 255;
            pixel[1] = 0;
            pixel[2] = 255;
            pixel[3] = 255;
        }
        assert!(frame_shows_magenta(&frame, 4));
        let mut sparse = vec![0_u8; 64 * 4];
        sparse[0] = 255;
        sparse[2] = 255;
        assert!(!frame_shows_magenta(&sparse, 4));
    }

    #[test]
    fn magenta_capture_stays_manual_on_this_platform() {
        let MagentaRun::Manual(message) = magenta_capture();
        assert!(message.contains("MANUAL"));
        if cfg!(target_os = "macos") {
            assert!(message.contains("Mini"));
        } else {
            assert!(message.contains("Wayland"));
        }
    }

    #[test]
    fn the_prompter_window_matches_on_pid_and_title() {
        let windows = [
            ListedWindow {
                id: 7,
                owner_pid: 42,
                title: "Shorts Studio".into(),
            },
            ListedWindow {
                id: 9,
                owner_pid: 42,
                title: "Prompter".into(),
            },
        ];
        assert_eq!(match_prompter_window(&windows, 42, "Prompter"), Some(9));
        assert_eq!(match_prompter_window(&windows, 42, "Other"), None);
        let exclusion = mac_exclusion(9);
        assert_eq!(exclusion.window_id, 9);
        assert!(exclusion.sharing_type_none);
    }
}
