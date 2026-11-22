//! macOS display capture through ScreenCaptureKit. The prompter window is excluded
//! by process id when that window exists. Output is capped at a 1920 long edge
//! and 30 fps so a retina display cannot write an unbounded file.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, mpsc};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use screencapturekit::prelude::*;
use screencapturekit::recording_output::{
    SCRecordingOutput, SCRecordingOutputCodec, SCRecordingOutputConfiguration,
    SCRecordingOutputFileType,
};
use screencapturekit::stream::configuration::SCCaptureResolutionType;

pub const MAX_LONG_EDGE: u32 = 1920;

/// Fit `width`×`height` inside a 1920 long edge. Both sides stay even.
pub fn capped_size(width: u32, height: u32) -> (u32, u32) {
    let even = |value: u32| value.max(2) & !1;
    let long = width.max(height);
    if width == 0 || height == 0 || long <= MAX_LONG_EDGE {
        return (even(width), even(height));
    }
    let scale = f64::from(MAX_LONG_EDGE) / f64::from(long);
    let scaled_w = (f64::from(width) * scale).round() as u32;
    let scaled_h = (f64::from(height) * scale).round() as u32;
    (even(scaled_w), even(scaled_h))
}

enum KitCmd {
    Pause,
    Resume,
    Stop,
}

struct KitRequest {
    cmd: KitCmd,
    ack: mpsc::Sender<Result<(), String>>,
}

pub struct KitRecording {
    cmd: Option<mpsc::Sender<KitRequest>>,
    handle: Option<JoinHandle<Result<(), String>>>,
    finished: Option<mpsc::Receiver<()>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListedWindow {
    pub id: u32,
    pub title: String,
}

pub fn list_windows() -> Result<Vec<ListedWindow>, String> {
    let content = SCShareableContent::get().map_err(|error| error.to_string())?;
    let mut windows = Vec::new();
    for window in content.windows() {
        let Some(title) = window.title().filter(|title| !title.trim().is_empty()) else {
            continue;
        };
        windows.push(ListedWindow {
            id: window.window_id(),
            title,
        });
        if windows.len() == 40 {
            break;
        }
    }
    Ok(windows)
}

impl KitRecording {
    pub fn start(path: &Path) -> Result<Self, String> {
        Self::start_with(path, None, None, Arc::new(AtomicBool::new(false)))
    }

    pub fn start_with(
        path: &Path,
        window_id: Option<u32>,
        region: Option<(f64, f64, f64, f64)>,
        halt: Arc<AtomicBool>,
    ) -> Result<Self, String> {
        let path = path.to_path_buf();
        let (ready_tx, ready_rx) = mpsc::channel();
        let (cmd_tx, cmd_rx) = mpsc::channel::<KitRequest>();
        let (finished_tx, finished_rx) = mpsc::channel();
        let handle = thread::spawn(move || {
            let started: Result<(SCStream, Option<SCRecordingOutput>, Vec<PathBuf>), String> =
                (|| {
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
                            let ours = window
                                .owning_application()
                                .is_some_and(|app| app.process_id() == pid);
                            let prompter = window.title().as_deref() == Some("Prompter");
                            ours || prompter
                        })
                        .collect();
                    let window_refs: Vec<&SCWindow> = own_windows.iter().collect();
                    let filter = if let Some(id) = window_id {
                        let window = content
                            .windows()
                            .into_iter()
                            .find(|window| window.window_id() == id)
                            .ok_or_else(|| format!("window {id} is not available"))?;
                        SCContentFilter::create().with_window(&window).build()
                    } else {
                        SCContentFilter::create()
                            .with_display(&display)
                            .with_excluding_windows(&window_refs)
                            .build()
                    };
                    let (source_w, source_h) = if let Some((_, _, width, height)) = region {
                        (width.max(1.0) as u32, height.max(1.0) as u32)
                    } else {
                        (display.width(), display.height())
                    };
                    let (width, height) = capped_size(source_w, source_h);
                    let mut config = SCStreamConfiguration::new()
                        .with_width(width)
                        .with_height(height)
                        .with_fps(30)
                        .with_pixel_format(PixelFormat::YCbCr_420v)
                        .with_scales_to_fit(true)
                        .with_capture_resolution_type(SCCaptureResolutionType::Nominal);
                    if let Some((x, y, rect_w, rect_h)) = region {
                        config = config.with_source_rect(CGRect::new(x, y, rect_w, rect_h));
                    }
                    let stream = SCStream::new(&filter, &config);
                    let mut segments = Vec::new();
                    let recording = Some(open_segment(&stream, &path, &mut segments)?);
                    stream.start_capture().map_err(|error| error.to_string())?;
                    Ok((stream, recording, segments))
                })();
            match started {
                Ok((stream, mut recording, mut segments)) => {
                    let _ = ready_tx.send(Ok(()));
                    let result = command_loop(
                        &stream,
                        &path,
                        &halt,
                        &cmd_rx,
                        &mut recording,
                        &mut segments,
                    );
                    let stopped = stream.stop_capture().map_err(|error| error.to_string());
                    if let Some(recording) = recording.take() {
                        let _ = stream.remove_recording_output(&recording);
                    }
                    let joined = concat_segments(&path, &segments);
                    let outcome = result.and(stopped).and(joined);
                    let _ = finished_tx.send(());
                    outcome
                }
                Err(error) => {
                    let _ = ready_tx.send(Err(error.clone()));
                    let _ = finished_tx.send(());
                    Err(error)
                }
            }
        });
        match ready_rx.recv() {
            Ok(Ok(())) => Ok(Self {
                cmd: Some(cmd_tx),
                handle: Some(handle),
                finished: Some(finished_rx),
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

    pub fn pause(&self) -> Result<(), String> {
        self.send(KitCmd::Pause)
    }

    pub fn resume(&self) -> Result<(), String> {
        self.send(KitCmd::Resume)
    }

    pub fn stop(mut self) -> Result<(), String> {
        let sent = self.send(KitCmd::Stop);
        self.finish(sent)
    }

    fn send(&self, cmd: KitCmd) -> Result<(), String> {
        let Some(tx) = &self.cmd else {
            return Err("screen capture ended".to_string());
        };
        let (ack_tx, ack_rx) = mpsc::channel();
        if tx.send(KitRequest { cmd, ack: ack_tx }).is_err() {
            return Err("screen capture ended".to_string());
        }
        match ack_rx.recv_timeout(Duration::from_secs(2)) {
            Ok(result) => result,
            // The size watch may already be closing the file.
            Err(mpsc::RecvTimeoutError::Timeout) => Ok(()),
            Err(mpsc::RecvTimeoutError::Disconnected) => Err("screen capture ended".to_string()),
        }
    }

    fn finish(&mut self, sent: Result<(), String>) -> Result<(), String> {
        self.cmd.take();
        let timed_out = self
            .finished
            .take()
            .is_some_and(|rx| rx.recv_timeout(Duration::from_secs(8)).is_err());
        if timed_out {
            // Dropping the handle detaches the thread so Stop can return.
            self.handle.take();
            return Err("screen capture did not finish in time".to_string());
        }
        if let Some(handle) = self.handle.take() {
            return handle
                .join()
                .map_err(|_| "screen capture thread panicked".to_string())?;
        }
        sent
    }
}

fn open_segment(
    stream: &SCStream,
    final_path: &Path,
    segments: &mut Vec<PathBuf>,
) -> Result<SCRecordingOutput, String> {
    let part = part_path(final_path, segments.len() as u32 + 1);
    let recording_config = SCRecordingOutputConfiguration::new()
        .with_output_url(&part)
        .with_video_codec(SCRecordingOutputCodec::H264)
        .with_output_file_type(SCRecordingOutputFileType::MOV);
    let recording = SCRecordingOutput::new(&recording_config)
        .ok_or_else(|| "ScreenCaptureKit recording output is unavailable".to_string())?;
    stream
        .add_recording_output(&recording)
        .map_err(|error| error.to_string())?;
    segments.push(part);
    Ok(recording)
}

fn command_loop(
    stream: &SCStream,
    final_path: &Path,
    halt: &AtomicBool,
    cmd_rx: &mpsc::Receiver<KitRequest>,
    recording: &mut Option<SCRecordingOutput>,
    segments: &mut Vec<PathBuf>,
) -> Result<(), String> {
    loop {
        if halt.load(Ordering::Relaxed) {
            return Ok(());
        }
        match cmd_rx.recv_timeout(Duration::from_millis(200)) {
            Ok(request) => match request.cmd {
                KitCmd::Pause => {
                    if let Some(current) = recording.take() {
                        let removed = stream
                            .remove_recording_output(&current)
                            .map_err(|error| error.to_string());
                        let _ = request.ack.send(removed);
                    } else {
                        let _ = request.ack.send(Ok(()));
                    }
                }
                KitCmd::Resume => {
                    if recording.is_none() {
                        match open_segment(stream, final_path, segments) {
                            Ok(next) => {
                                *recording = Some(next);
                                let _ = request.ack.send(Ok(()));
                            }
                            Err(error) => {
                                let _ = request.ack.send(Err(error));
                            }
                        }
                    } else {
                        let _ = request.ack.send(Ok(()));
                    }
                }
                KitCmd::Stop => {
                    let _ = request.ack.send(Ok(()));
                    return Ok(());
                }
            },
            Err(mpsc::RecvTimeoutError::Timeout) => {}
            Err(mpsc::RecvTimeoutError::Disconnected) => return Ok(()),
        }
    }
}

fn part_path(final_path: &Path, index: u32) -> PathBuf {
    let stem = final_path
        .file_stem()
        .map(|stem| stem.to_string_lossy().into_owned())
        .unwrap_or_else(|| "screen".to_string());
    let ext = final_path
        .extension()
        .map(|ext| ext.to_string_lossy().into_owned())
        .unwrap_or_else(|| "mov".to_string());
    final_path.with_file_name(format!("{stem}.part{index}.{ext}"))
}

fn concat_segments(final_path: &Path, segments: &[PathBuf]) -> Result<(), String> {
    let ready: Vec<&PathBuf> = segments.iter().filter(|path| path.is_file()).collect();
    if ready.is_empty() {
        return Err("screen capture wrote no video".to_string());
    }
    if ready.len() == 1 {
        if ready[0] != final_path {
            std::fs::rename(ready[0], final_path).map_err(|error| error.to_string())?;
        }
        return Ok(());
    }
    let list_path = final_path.with_extension("txt");
    let mut list = std::fs::File::create(&list_path).map_err(|error| error.to_string())?;
    for path in &ready {
        let escaped = path.display().to_string().replace('\'', r"'\''");
        std::io::Write::write_fmt(&mut list, format_args!("file '{escaped}'\n"))
            .map_err(|error| error.to_string())?;
    }
    drop(list);
    let tmp = final_path.with_extension("joining.mov");
    let mut command = crate::ffmpeg::ffmpeg();
    command
        .args(["-f", "concat", "-safe", "0", "-i"])
        .arg(&list_path)
        .args(["-c", "copy"])
        .arg(&tmp)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null());
    let status = command
        .status()
        .map_err(|error| format!("concat ffmpeg: {error}"))?;
    if !status.success() {
        return Err(format!("concat ffmpeg exited {status}"));
    }
    std::fs::rename(&tmp, final_path).map_err(|error| error.to_string())?;
    for path in &ready {
        let _ = std::fs::remove_file(path);
    }
    let _ = std::fs::remove_file(list_path);
    Ok(())
}

impl Drop for KitRecording {
    fn drop(&mut self) {
        if self.handle.is_some() {
            let sent = self.send(KitCmd::Stop);
            let _ = self.finish(sent);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::capped_size;

    #[test]
    fn retina_display_fits_the_long_edge() {
        assert_eq!(capped_size(2880, 1800), (1920, 1200));
        assert_eq!(capped_size(1800, 2880), (1200, 1920));
        assert_eq!(capped_size(1280, 720), (1280, 720));
    }
}
