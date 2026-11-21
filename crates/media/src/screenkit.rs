//! macOS display capture through ScreenCaptureKit. The prompter window is excluded
//! by process id when that window exists.

use std::path::Path;
use std::sync::mpsc;
use std::thread::{self, JoinHandle};

use screencapturekit::prelude::*;
use screencapturekit::recording_output::{
    SCRecordingOutput, SCRecordingOutputCodec, SCRecordingOutputConfiguration,
    SCRecordingOutputFileType,
};

pub struct KitRecording {
    stop: Option<mpsc::Sender<()>>,
    handle: Option<JoinHandle<Result<(), String>>>,
}

impl KitRecording {
    pub fn start(path: &Path) -> Result<Self, String> {
        let path = path.to_path_buf();
        let (ready_tx, ready_rx) = mpsc::channel();
        let (stop_tx, stop_rx) = mpsc::channel();
        let handle = thread::spawn(move || {
            let started: Result<(SCStream, SCRecordingOutput), String> = (|| {
                let content = SCShareableContent::get().map_err(|error| error.to_string())?;
                let display = content
                    .displays()
                    .into_iter()
                    .next()
                    .ok_or_else(|| "no display".to_string())?;
                let pid = std::process::id() as i32;
                let own_windows: Vec<SCWindow> = content
                    .windows()
                    .into_iter()
                    .filter(|window| {
                        window
                            .owning_application()
                            .is_some_and(|app| app.process_id() == pid)
                    })
                    .collect();
                let window_refs: Vec<&SCWindow> = own_windows.iter().collect();
                let filter = SCContentFilter::create()
                    .with_display(&display)
                    .with_excluding_windows(&window_refs)
                    .build();
                let config = SCStreamConfiguration::new()
                    .with_width(display.width())
                    .with_height(display.height());
                let recording_config = SCRecordingOutputConfiguration::new()
                    .with_output_url(&path)
                    .with_video_codec(SCRecordingOutputCodec::H264)
                    .with_output_file_type(SCRecordingOutputFileType::MOV);
                let recording = SCRecordingOutput::new(&recording_config).ok_or_else(|| {
                    "ScreenCaptureKit recording output is unavailable".to_string()
                })?;
                let stream = SCStream::new(&filter, &config);
                stream
                    .add_recording_output(&recording)
                    .map_err(|error| error.to_string())?;
                stream.start_capture().map_err(|error| error.to_string())?;
                Ok((stream, recording))
            })();
            match started {
                Ok(active) => {
                    let _ = ready_tx.send(Ok(()));
                    let _ = stop_rx.recv();
                    let (stream, recording) = active;
                    let _ = stream.remove_recording_output(&recording);
                    stream.stop_capture().map_err(|error| error.to_string())
                }
                Err(error) => {
                    let _ = ready_tx.send(Err(error.clone()));
                    Err(error)
                }
            }
        });
        match ready_rx.recv() {
            Ok(Ok(())) => Ok(Self {
                stop: Some(stop_tx),
                handle: Some(handle),
            }),
            Ok(Err(error)) => {
                let _ = handle.join();
                Err(format!(
                    "{error}. Allow Screen Recording for shorts-studio in System Settings → Privacy & Security."
                ))
            }
            Err(_) => Err("screen capture thread ended before it started".to_string()),
        }
    }

    pub fn stop(mut self) -> Result<(), String> {
        if let Some(stop) = self.stop.take() {
            let _ = stop.send(());
        }
        if let Some(handle) = self.handle.take() {
            return handle
                .join()
                .map_err(|_| "screen capture thread panicked".to_string())?;
        }
        Ok(())
    }
}

impl Drop for KitRecording {
    fn drop(&mut self) {
        if let Some(stop) = self.stop.take() {
            let _ = stop.send(());
        }
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}
