use serde::Serialize;
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

#[cfg(test)]
mod tests {
    use super::health;

    #[test]
    fn health_names_the_app() {
        let health = health();
        assert_eq!(health.name, "shorts-studio");
        assert!(!health.version.is_empty());
    }
}
