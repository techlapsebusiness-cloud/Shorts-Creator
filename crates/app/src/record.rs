//! Tauri-facing recording session state for M06.

use std::sync::{Arc, Mutex};

use media::devices::{DeviceList, list_devices};
use media::disk::{DiskEstimate, estimate};
use media::record::{
    CaptureBackend, DEFAULT_COUNTDOWN_SECS, RecordPlan, RecordingSession, TakeResult,
};
use serde::Deserialize;
use tauri::{AppHandle, Manager, State};

use core::LayoutPreset;

pub struct RecordState {
    session: Arc<Mutex<Option<RecordingSession>>>,
}

impl RecordState {
    pub fn session(&self) -> Arc<Mutex<Option<RecordingSession>>> {
        Arc::clone(&self.session)
    }
}

impl Default for RecordState {
    fn default() -> Self {
        Self {
            session: Arc::new(Mutex::new(None)),
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
    #[serde(default)]
    pub window_id: Option<u32>,
    #[serde(default)]
    pub region: Option<media::record::CaptureRegion>,
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RecordingStatus {
    pub phase: String,
    pub bytes: u64,
    pub message: Option<String>,
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

#[tauri::command(async)]
pub fn start_recording(
    app: AppHandle,
    state: State<'_, RecordState>,
    args: StartRecordingArgs,
) -> Result<(), String> {
    let mut guard = state.session.lock().map_err(|_| "record lock poisoned")?;
    if guard.is_some() {
        return Err("a recording is already in progress".to_string());
    }
    let backend = parse_backend(&args.backend)?;
    if backend == CaptureBackend::LinuxFfmpeg && app.get_webview_window("prompter").is_some() {
        linux_prompter_gate(&args)?;
    }
    let plan = RecordPlan {
        out_dir: args.out_dir.into(),
        layout: args.layout,
        backend,
        screen_id: args.screen_id,
        cam_id: args.cam_id,
        mic_id: args.mic_id,
        countdown_secs: args.countdown_secs.unwrap_or(0),
        fixed_duration_secs: args.fixed_duration_secs,
        window_id: args.window_id,
        region: args.region,
    };
    let session = RecordingSession::start(plan).map_err(|error| error.message)?;
    *guard = Some(session);
    Ok(())
}

#[tauri::command(async)]
pub fn pause_recording(state: State<'_, RecordState>) -> Result<(), String> {
    let mut guard = state.session.lock().map_err(|_| "record lock poisoned")?;
    let session = guard
        .as_mut()
        .ok_or_else(|| "no active recording".to_string())?;
    session.pause().map_err(|error| error.message)
}

#[tauri::command(async)]
pub fn resume_recording(state: State<'_, RecordState>) -> Result<(), String> {
    let mut guard = state.session.lock().map_err(|_| "record lock poisoned")?;
    let session = guard
        .as_mut()
        .ok_or_else(|| "no active recording".to_string())?;
    session.resume().map_err(|error| error.message)
}

#[tauri::command(async)]
pub fn stop_recording(state: State<'_, RecordState>) -> Result<TakeResult, String> {
    let mut guard = state.session.lock().map_err(|_| "record lock poisoned")?;
    let session = guard
        .take()
        .ok_or_else(|| "no active recording".to_string())?;
    let result = session.stop().map_err(|error| error.message)?;
    crate::takes::write_sync(std::path::Path::new(&result.out_dir), &result.sync);
    media::transcript::queue(&result.out_dir);
    Ok(result)
}

#[tauri::command]
pub fn recording_status(state: State<'_, RecordState>) -> Result<RecordingStatus, String> {
    let guard = state.session.lock().map_err(|_| "record lock poisoned")?;
    let Some(session) = guard.as_ref() else {
        return Ok(RecordingStatus {
            phase: "idle".to_string(),
            bytes: 0,
            message: None,
        });
    };
    Ok(RecordingStatus {
        phase: "recording".to_string(),
        bytes: session.bytes_written(),
        message: session.limit_message(),
    })
}

#[tauri::command]
pub fn default_countdown_secs() -> u32 {
    DEFAULT_COUNTDOWN_SECS
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CaptureWindow {
    pub id: u32,
    pub title: String,
}

#[tauri::command]
pub fn list_capture_windows() -> Vec<CaptureWindow> {
    #[cfg(target_os = "macos")]
    {
        media::screenkit::list_windows()
            .unwrap_or_default()
            .into_iter()
            .map(|window| CaptureWindow {
                id: window.id,
                title: window.title,
            })
            .collect()
    }
    #[cfg(not(target_os = "macos"))]
    Vec::new()
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DesktopIconSupport {
    pub supported: bool,
}

#[tauri::command]
pub fn desktop_icon_support() -> DesktopIconSupport {
    DesktopIconSupport {
        supported: cfg!(target_os = "macos"),
    }
}

#[tauri::command]
pub fn set_desktop_icons(visible: bool) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        media::icons::set_desktop_icons_visible(visible)
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = visible;
        Err("desktop icons are a macOS Finder setting".to_string())
    }
}

#[tauri::command]
pub fn log_overlay_event(
    state: State<'_, RecordState>,
    kind: String,
    label: String,
) -> Result<(), String> {
    let mut guard = state.session.lock().map_err(|_| "record lock poisoned")?;
    let session = guard
        .as_mut()
        .ok_or_else(|| "no active recording".to_string())?;
    session
        .log_overlay(&kind, &label)
        .map_err(|error| error.message)
}

fn linux_prompter_gate(args: &StartRecordingArgs) -> Result<(), String> {
    let prompter = core::PxRect {
        x: 80,
        y: 80,
        width: 480,
        height: 640,
    };
    let target = if let Some(region) = args.region {
        core::CaptureFrame::Region {
            monitor_id: 0,
            rect: core::PxRect {
                x: region.x as i32,
                y: region.y as i32,
                width: region.width as i32,
                height: region.height as i32,
            },
        }
    } else if args.window_id.is_some() {
        core::CaptureFrame::Region {
            monitor_id: 0,
            rect: core::PxRect {
                x: 2_000,
                y: 0,
                width: 10,
                height: 10,
            },
        }
    } else {
        core::CaptureFrame::Monitor { id: 0 }
    };
    core::allow_linux_capture(0, prompter, target)
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
