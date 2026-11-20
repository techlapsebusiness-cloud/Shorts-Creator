use ai::SecretStore;
use serde::{Deserialize, Serialize};
use tauri::Manager;

use crate::meter::{effective_cap, read_cap, write_cap};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Health {
    pub name: String,
    pub version: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SpendStatus {
    pub spent_usd: f64,
    pub cap_usd: f64,
    pub decision: String,
}

#[tauri::command]
pub fn health() -> Health {
    Health {
        name: "shorts-studio".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    }
}

#[tauri::command]
pub fn test_provider(provider: String, key: String) -> ai::ProviderTest {
    let mut secrets = ai::OsStore;
    ai::execute_test(&provider, &key, &mut secrets, &mut |provider, api_key| {
        ai::list_models(provider, api_key)
    })
}

#[tauri::command]
pub fn spend_status(app: tauri::AppHandle) -> Result<SpendStatus, String> {
    let dir = data_dir(&app)?;
    let cap = effective_cap(read_cap(&dir.join("spend-cap.txt")), ai::config().cap_usd);
    let library =
        store::Library::open(&dir.join(store::LIBRARY_FILE)).map_err(|error| error.to_string())?;
    let spent = library
        .usage_cost_usd()
        .map_err(|error| error.to_string())?;
    let decision = ai::decide(spent, cap, ai::config().alert_ratio);
    Ok(SpendStatus {
        spent_usd: spent,
        cap_usd: cap,
        decision: decision.as_str().to_string(),
    })
}

#[tauri::command]
pub fn set_spend_cap(app: tauri::AppHandle, cap_usd: f64) -> Result<(), String> {
    let dir = data_dir(&app)?;
    write_cap(&dir.join("spend-cap.txt"), cap_usd)
}

fn data_dir(app: &tauri::AppHandle) -> Result<std::path::PathBuf, String> {
    app.path().app_data_dir().map_err(|error| error.to_string())
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScriptPayload {
    pub raw_draft: String,
    pub beats: Vec<core::Beat>,
}

#[tauri::command]
pub fn generate_script(
    app: tauri::AppHandle,
    brief: ai::ScriptBrief,
) -> Result<ScriptPayload, String> {
    if brief.model.trim().is_empty() {
        return Err("Pick a model in Settings.".into());
    }
    let key = provider_key(&brief.provider)?;
    let provider = ai::connect(&brief.provider, &key)
        .map_err(|error| ai::redact(&error.to_string(), &[&key]))?;
    let mut library = open_library(&app)?;
    let stamp = stamp();
    let generated = crate::script_flow::generate_within_cap(
        &mut library,
        provider.as_ref(),
        &brief,
        &crate::script_flow::SpendGate {
            cap_usd: current_cap(&app)?,
            alert_ratio: ai::config().alert_ratio,
            created_at: &stamp,
        },
    )
    .map_err(|error| ai::redact(&error.to_string(), &[&key]))?;
    Ok(ScriptPayload {
        raw_draft: generated.raw_draft,
        beats: generated.beats,
    })
}

#[tauri::command]
pub fn regenerate_beat(
    app: tauri::AppHandle,
    brief: ai::ScriptBrief,
    index: usize,
    beats: Vec<core::Beat>,
) -> Result<core::Beat, String> {
    if brief.model.trim().is_empty() {
        return Err("Pick a model in Settings.".into());
    }
    let key = provider_key(&brief.provider)?;
    let provider = ai::connect(&brief.provider, &key)
        .map_err(|error| ai::redact(&error.to_string(), &[&key]))?;
    let mut library = open_library(&app)?;
    let stamp = stamp();
    crate::script_flow::regenerate_within_cap(
        &mut library,
        provider.as_ref(),
        &brief,
        &beats,
        index,
        &crate::script_flow::SpendGate {
            cap_usd: current_cap(&app)?,
            alert_ratio: ai::config().alert_ratio,
            created_at: &stamp,
        },
    )
    .map_err(|error| ai::redact(&error.to_string(), &[&key]))
}

fn provider_key(provider: &str) -> Result<String, String> {
    let secrets = ai::OsStore;
    match secrets.get(provider) {
        Ok(Some(key)) if !key.is_empty() => Ok(key),
        Ok(_) => Err("No key saved".into()),
        Err(error) => Err(ai::redact(&error.to_string(), &[])),
    }
}

fn open_library(app: &tauri::AppHandle) -> Result<store::Library, String> {
    let dir = data_dir(app)?;
    store::Library::open(&dir.join(store::LIBRARY_FILE)).map_err(|error| error.to_string())
}

fn current_cap(app: &tauri::AppHandle) -> Result<f64, String> {
    let dir = data_dir(app)?;
    Ok(effective_cap(
        read_cap(&dir.join("spend-cap.txt")),
        ai::config().cap_usd,
    ))
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GateRect {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GateRequest {
    pub prompter_monitor: u32,
    pub capture: String,
    pub capture_monitor: u32,
    pub prompter: GateRect,
    pub capture_bounds: GateRect,
}

fn placed(rect: &GateRect) -> media::Rect {
    media::Rect {
        x: rect.x,
        y: rect.y,
        width: rect.width,
        height: rect.height,
    }
}

#[tauri::command]
pub fn recording_gate(gate: GateRequest) -> Result<(), String> {
    let capture = match gate.capture.as_str() {
        "monitor" => media::Capture::Monitor {
            monitor_id: gate.capture_monitor,
        },
        "window" => media::Capture::Window {
            monitor_id: gate.capture_monitor,
            bounds: placed(&gate.capture_bounds),
        },
        "region" => media::Capture::Region {
            monitor_id: gate.capture_monitor,
            bounds: placed(&gate.capture_bounds),
        },
        _ => return Err("Unknown capture".into()),
    };
    media::recording_allowed(
        media::PrompterPlacement {
            monitor_id: gate.prompter_monitor,
            bounds: placed(&gate.prompter),
        },
        capture,
    )
    .map_err(|reason| reason.to_string())
}

pub const PROMPTER_LABEL: &str = "prompter";

#[tauri::command]
pub fn show_prompter(app: tauri::AppHandle) -> Result<(), String> {
    let window = app
        .get_webview_window(PROMPTER_LABEL)
        .ok_or("Prompter window is missing")?;
    window.show().map_err(|error| error.to_string())?;
    window
        .set_always_on_top(true)
        .map_err(|error| error.to_string())?;
    window
        .set_ignore_cursor_events(true)
        .map_err(|error| error.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn prompter_click_through(app: tauri::AppHandle, ignore: bool) -> Result<(), String> {
    let window = app
        .get_webview_window(PROMPTER_LABEL)
        .ok_or("Prompter window is missing")?;
    window
        .set_ignore_cursor_events(ignore)
        .map_err(|error| error.to_string())
}

fn stamp() -> String {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_secs().to_string())
        .unwrap_or_else(|_| "0".into())
}

#[cfg(test)]
mod tests {
    use super::health;

    #[test]
    fn health_names_the_app() {
        let health = health();
        assert_eq!(health.name, "shorts-studio");
        assert!(!health.version.is_empty());
    }

    #[test]
    fn a_same_monitor_capture_is_refused_with_a_reason() {
        let error = super::recording_gate(super::GateRequest {
            prompter_monitor: 0,
            capture: "monitor".into(),
            capture_monitor: 0,
            prompter: super::GateRect {
                x: 0,
                y: 0,
                width: 100,
                height: 100,
            },
            capture_bounds: super::GateRect {
                x: 0,
                y: 0,
                width: 100,
                height: 100,
            },
        })
        .unwrap_err();
        assert!(error.contains("another monitor"));
    }
}
