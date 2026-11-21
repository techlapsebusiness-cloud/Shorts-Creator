#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(app::RecordState::default())
        .invoke_handler(tauri::generate_handler![
            app::health,
            app::test_provider,
            app::spend_status,
            app::set_spend_cap,
            app::list_capture_devices,
            app::disk_estimate,
            app::prepare_take,
            app::start_recording,
            app::pause_recording,
            app::resume_recording,
            app::stop_recording,
            app::recording_status,
            app::default_countdown_secs,
            app::transcribe_take
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
