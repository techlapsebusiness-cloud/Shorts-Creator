//! Tauri-facing recording session state for M06.

use std::sync::Mutex;

use media::devices::{DeviceList, list_devices};
use media::disk::{DiskEstimate, estimate};
use media::record::{
    CaptureBackend, DEFAULT_COUNTDOWN_SECS, RecordPlan, RecordingSession, TakeResult,
};
use serde::Deserialize;
use tauri::{AppHandle, Manager, State};

use core::LayoutPreset;

pub struct RecordState {
    session: Mutex<Option<RecordingSession>>,
}

impl Default for RecordState {
    fn default() -> Self {
        Self {
            session: Mutex::new(None),
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StartRecordingArgs {
    pub out_dir: String,
    pub layout: LayoutPreset,
    pub backend: String,
    pub screen_id: Option<String>,
    pub cam_id: Option<String>,
    pub mic_id: Option<String>,
    pub countdown_secs: Option<u32>,
    pub fixed_duration_secs: Option<f64>,
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RecordingStatus {
    pub phase: String,
}

#[tauri::command]
pub fn list_capture_devices() -> DeviceList {
    list_devices()
}

#[tauri::command]
pub fn disk_estimate(path: String) -> Result<DiskEstimate, String> {
    estimate(std::path::Path::new(&path))
}

#[tauri::command]
pub fn prepare_take(app: AppHandle) -> Result<String, String> {
    let root = app
        .path()
        .app_data_dir()
        .map_err(|error| error.to_string())?
        .join("takes");
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|error| error.to_string())?
        .as_millis();
    let dir = root.join(stamp.to_string());
    std::fs::create_dir_all(&dir).map_err(|error| error.to_string())?;
    Ok(dir.to_string_lossy().into_owned())
}

#[tauri::command]
pub fn start_recording(
    state: State<'_, RecordState>,
    args: StartRecordingArgs,
) -> Result<(), String> {
    let mut guard = state.session.lock().map_err(|_| "record lock poisoned")?;
    if guard.is_some() {
        return Err("a recording is already in progress".to_string());
    }
    let backend = parse_backend(&args.backend)?;
    let plan = RecordPlan {
        out_dir: args.out_dir.into(),
        layout: args.layout,
        backend,
        screen_id: args.screen_id,
        cam_id: args.cam_id,
        mic_id: args.mic_id,
        countdown_secs: args.countdown_secs.unwrap_or(0),
        fixed_duration_secs: args.fixed_duration_secs,
    };
    let session = RecordingSession::start(plan).map_err(|error| error.message)?;
    *guard = Some(session);
    Ok(())
}

#[tauri::command]
pub fn pause_recording(state: State<'_, RecordState>) -> Result<(), String> {
    let mut guard = state.session.lock().map_err(|_| "record lock poisoned")?;
    let session = guard
        .as_mut()
        .ok_or_else(|| "no active recording".to_string())?;
    session.pause().map_err(|error| error.message)
}

#[tauri::command]
pub fn resume_recording(state: State<'_, RecordState>) -> Result<(), String> {
    let mut guard = state.session.lock().map_err(|_| "record lock poisoned")?;
    let session = guard
        .as_mut()
        .ok_or_else(|| "no active recording".to_string())?;
    session.resume().map_err(|error| error.message)
}

#[tauri::command]
pub fn stop_recording(state: State<'_, RecordState>) -> Result<TakeResult, String> {
    let mut guard = state.session.lock().map_err(|_| "record lock poisoned")?;
    let session = guard
        .take()
        .ok_or_else(|| "no active recording".to_string())?;
    let result = session.stop().map_err(|error| error.message)?;
    media::transcript::queue(&result.out_dir);
    Ok(result)
}

#[tauri::command]
pub fn recording_status(state: State<'_, RecordState>) -> Result<RecordingStatus, String> {
    let guard = state.session.lock().map_err(|_| "record lock poisoned")?;
    let phase = if guard.is_some() { "recording" } else { "idle" };
    Ok(RecordingStatus {
        phase: phase.to_string(),
    })
}

#[tauri::command]
pub fn default_countdown_secs() -> u32 {
    DEFAULT_COUNTDOWN_SECS
}

fn parse_backend(value: &str) -> Result<CaptureBackend, String> {
    match value {
        "lavfi" => Ok(CaptureBackend::Lavfi),
        "avfoundation" => Ok(CaptureBackend::AvFoundation),
        "linux" => Ok(CaptureBackend::LinuxFfmpeg),
        other => Err(format!("unknown capture backend: {other}")),
    }
}

#[cfg(test)]
mod tests {
    use super::parse_backend;
    use media::record::CaptureBackend;

    #[test]
    fn parses_backends() {
        assert_eq!(parse_backend("lavfi").unwrap(), CaptureBackend::Lavfi);
        assert_eq!(
            parse_backend("avfoundation").unwrap(),
            CaptureBackend::AvFoundation
        );
        assert!(parse_backend("nope").is_err());
    }
}
