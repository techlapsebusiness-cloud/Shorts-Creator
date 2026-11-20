//! FFmpeg child processes, capture, and render. M00 ships the doctor checks.

pub const NAME: &str = "media";

pub mod avfoundation;
pub mod doctor;
pub mod exclusion;
pub mod take;

pub use avfoundation::{begin_live_take, end_live_take};
pub use exclusion::{
    Capture, ListedWindow, MAGENTA, MacExclusion, MagentaRun, PROMPTER_TITLE, PrompterPlacement,
    Rect, RefuseReason, frame_shows_magenta, is_magenta, mac_exclusion, magenta_capture,
    match_prompter_window, overlaps, recording_allowed,
};
pub use take::{
    AUDIO_BITRATE_KBPS, COUNTDOWN_MS, CURSOR_HZ, CaptureDevice, CursorMode, CursorSample,
    DesktopIcons, DeviceCatalog, SessionClock, TakeOutput, VIDEO_BITRATE_KBPS, choose_cursor_mode,
    cursor_is_monotonic, desktop_icons_plan, estimate_bytes, fixture_devices,
    media_duration_seconds, mic_level, parse_cursor_jsonl, pick_h264_encoder, record_fixture_take,
    sample_cursor, write_cursor_jsonl,
};
