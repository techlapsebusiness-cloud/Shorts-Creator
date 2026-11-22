//! The teleprompter window. macOS also marks it content-protected.

use std::sync::atomic::{AtomicBool, Ordering};

use tauri::{AppHandle, Manager, State, WebviewUrl, WebviewWindowBuilder};

pub struct PrompterRun {
    running: AtomicBool,
}

impl Default for PrompterRun {
    fn default() -> Self {
        Self {
            running: AtomicBool::new(false),
        }
    }
}

#[tauri::command]
pub fn open_prompter(app: AppHandle) -> Result<(), String> {
    if app.get_webview_window("prompter").is_some() {
        return Ok(());
    }
    let window = WebviewWindowBuilder::new(&app, "prompter", WebviewUrl::App("prompter".into()))
        .title("Prompter")
        .inner_size(480.0, 640.0)
        .transparent(true)
        .always_on_top(true)
        .decorations(false)
        .build()
        .map_err(|error| error.to_string())?;
    // Sharing type none. ScreenCaptureKit also drops this window by id.
    #[cfg(target_os = "macos")]
    window
        .set_content_protected(true)
        .map_err(|error| error.to_string())?;
    #[cfg(not(target_os = "macos"))]
    let _ = window;
    Ok(())
}

#[tauri::command]
pub fn prompter_set_running(state: State<'_, PrompterRun>, running: bool) {
    state.running.store(running, Ordering::Relaxed);
}

#[tauri::command]
pub fn prompter_running(state: State<'_, PrompterRun>) -> bool {
    state.running.load(Ordering::Relaxed)
}

#[tauri::command]
pub fn prompter_lines(app: AppHandle) -> Vec<String> {
    let Ok(dir) = app.path().app_data_dir() else {
        return default_lines();
    };
    let loaded = store::load_project(&dir.join("script-current"));
    let Ok(project) = loaded else {
        return default_lines();
    };
    if project.ai.beats.is_empty() {
        return default_lines();
    }
    let mut lines = Vec::new();
    for beat in project.ai.beats {
        lines.push(beat.text);
        for cue in beat.cues {
            lines.push(format!("[CUE] {cue}"));
        }
    }
    lines
}

fn default_lines() -> Vec<String> {
    vec![
        "This file survives a crash.".into(),
        "[CUE] point at the folder".into(),
        "The edit lives in project.json.".into(),
        "Build the next short.".into(),
    ]
}
