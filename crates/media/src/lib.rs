//! FFmpeg child processes, capture, and render. M00 ships the doctor checks.

pub const NAME: &str = "media";

pub mod doctor;
pub mod exclusion;

pub use exclusion::{
    Capture, ListedWindow, MAGENTA, MacExclusion, MagentaRun, PROMPTER_TITLE, PrompterPlacement,
    Rect, RefuseReason, frame_shows_magenta, is_magenta, mac_exclusion, magenta_capture,
    match_prompter_window, overlaps, recording_allowed,
};
