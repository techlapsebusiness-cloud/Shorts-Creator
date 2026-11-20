#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            app::health,
            app::test_provider,
            app::spend_status,
            app::set_spend_cap,
            app::generate_script,
            app::regenerate_beat,
            app::recording_gate,
            app::show_prompter,
            app::prompter_click_through
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
