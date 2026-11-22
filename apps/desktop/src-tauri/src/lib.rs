use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(app::RecordState::default())
        .manage(app::PrompterRun::default())
        .manage(app::RenderState::default())
        .on_window_event(|window, event| {
            if window.label() != "main" {
                return;
            }
            let tauri::WindowEvent::CloseRequested { api, .. } = event else {
                return;
            };
            if !CLOSE_STARTED.swap(true, std::sync::atomic::Ordering::SeqCst) {
                api.prevent_close();
                let window = window.clone();
                tauri::async_runtime::spawn(async move {
                    close_studio(window).await;
                });
            }
        })
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
            app::list_capture_windows,
            app::desktop_icon_support,
            app::set_desktop_icons,
            app::log_overlay_event,
            app::transcribe_take,
            app::generate_script,
            app::regenerate_beat,
            app::save_script,
            app::open_prompter,
            app::prompter_set_running,
            app::prompter_running,
            app::prompter_lines,
            app::start_render,
            app::cancel_render,
            app::render_status,
            app::list_takes,
            app::open_take,
            app::save_opened_project
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

static CLOSE_STARTED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

async fn close_studio(window: tauri::Window) {
    let slot = {
        let app = window.app_handle();
        if let Some(prompter) = app.get_webview_window("prompter") {
            let _ = prompter.close();
        }
        app.state::<app::RecordState>().session()
    };
    let _ = tauri::async_runtime::spawn_blocking(move || {
        if let Ok(mut guard) = slot.lock()
            && let Some(session) = guard.take()
        {
            let _ = session.stop();
        }
    })
    .await;
    let _ = window.destroy();
}
