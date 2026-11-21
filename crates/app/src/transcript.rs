use core::{Beat, TranscriptReport};

#[tauri::command]
pub fn transcribe_take(dir: String, beats: Option<Vec<Beat>>) -> TranscriptReport {
    media::transcript::analyze_take(std::path::Path::new(&dir), beats.as_deref().unwrap_or(&[]))
}
