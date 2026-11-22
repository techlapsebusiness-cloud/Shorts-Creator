//! Tauri commands. The webview binary lives in `apps/desktop/src-tauri`.
//!
//! Commands are a submodule. `#[tauri::command]` on a public function in the
//! crate root defines its helper macros twice on tauri-macros 2.7.

pub const NAME: &str = "app";

mod commands;
mod meter;
mod prompter;
mod record;
mod render;
mod script;
mod takes;
mod transcript;

pub use commands::{Health, SpendStatus, health, set_spend_cap, spend_status, test_provider};
pub use prompter::{
    PrompterRun, open_prompter, prompter_lines, prompter_running, prompter_set_running,
};
pub use record::{
    CaptureWindow, DesktopIconSupport, RecordState, StartRecordingArgs, default_countdown_secs,
    desktop_icon_support, disk_estimate, list_capture_devices, list_capture_windows,
    log_overlay_event, pause_recording, prepare_take, recording_status, resume_recording,
    set_desktop_icons, start_recording, stop_recording,
};
pub use render::{RenderState, RenderStatus, cancel_render, render_status, start_render};
pub use script::{generate_script, regenerate_beat, save_script};
pub use takes::{OpenedTake, TakeSummary, list_takes, open_take, save_opened_project, write_sync};
pub use transcript::transcribe_take;

pub fn linked_crates() -> [&'static str; 4] {
    [core::NAME, media::NAME, ai::NAME, store::NAME]
}

#[cfg(test)]
mod tests {
    use super::linked_crates;

    #[test]
    fn workspace_crates_are_linked() {
        assert_eq!(linked_crates(), ["core", "media", "ai", "store"]);
    }
}
