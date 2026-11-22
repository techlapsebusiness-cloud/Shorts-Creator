//! Three-track recording session (screen, cam, mic) plus cursor.jsonl.

use crate::cursor::{CursorWriter, assert_monotonic, parse_cursor_jsonl};
use crate::ffmpeg::{
    FfmpegError, encoder_or_fallback, ffmpeg, preferred_video_encoder, probe_duration_secs,
};
use core::{LayoutPreset, TakeSync};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

pub const SCREEN_FILE: &str = "screen.mov";
pub const CAM_FILE: &str = "cam.mov";
pub const MIC_FILE: &str = "mic.wav";
pub const CURSOR_FILE: &str = "cursor.jsonl";
pub const OVERLAY_FILE: &str = "overlay.jsonl";
pub const DEFAULT_COUNTDOWN_SECS: u32 = 3;
pub const SCREEN_BITRATE: &str = "20M";
pub const CAM_BITRATE: &str = "8M";
pub const MAX_TAKE_BYTES: u64 = 2_000_000_000;
pub const MIN_FREE_BYTES: u64 = 1_000_000_000;

pub fn take_limit_reason(take_bytes: u64, free_bytes: u64) -> Option<&'static str> {
    if take_bytes >= MAX_TAKE_BYTES {
        Some("Recording stopped at 2 GB so the take cannot fill the disk.")
    } else if free_bytes < MIN_FREE_BYTES {
        Some("Recording stopped because less than 1 GB is free.")
    } else {
        None
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum CaptureBackend {
    Lavfi,
    AvFoundation,
    LinuxFfmpeg,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RecordPlan {
    pub out_dir: PathBuf,
    pub layout: LayoutPreset,
    pub backend: CaptureBackend,
    pub screen_id: Option<String>,
    pub cam_id: Option<String>,
    pub mic_id: Option<String>,
    pub countdown_secs: u32,
    /// When set, ffmpeg exits after this many seconds (tests). UI leaves it None.
    pub fixed_duration_secs: Option<f64>,
    /// ScreenCaptureKit window. None records the display.
    pub window_id: Option<u32>,
    pub region: Option<CaptureRegion>,
}

#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CaptureRegion {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TakeResult {
    pub screen: Option<String>,
    pub cam: Option<String>,
    pub mic: Option<String>,
    pub cursor: String,
    pub sync: TakeSync,
    pub screen_duration_ms: Option<f64>,
    pub cam_duration_ms: Option<f64>,
    pub mic_duration_ms: Option<f64>,
    pub out_dir: String,
    pub overlay: String,
}

#[derive(Debug)]
pub struct RecordError {
    pub message: String,
}

impl std::fmt::Display for RecordError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for RecordError {}

impl From<FfmpegError> for RecordError {
    fn from(value: FfmpegError) -> Self {
        Self {
            message: value.message,
        }
    }
}

impl From<crate::cursor::CursorError> for RecordError {
    fn from(value: crate::cursor::CursorError) -> Self {
        Self {
            message: value.message,
        }
    }
}

pub struct RecordingSession {
    plan: RecordPlan,
    origin: Instant,
    screen_offset_ms: f64,
    cam_offset_ms: f64,
    mic_offset_ms: f64,
    screen: Option<ScreenHandle>,
    cam: Option<Child>,
    mic: Option<Child>,
    cursor: Option<CursorWriter>,
    overlay: Option<File>,
    paused: bool,
    screen_path: Option<PathBuf>,
    cam_path: Option<PathBuf>,
    mic_path: Option<PathBuf>,
    halt: Arc<AtomicBool>,
    limit_reason: Arc<Mutex<Option<String>>>,
    watch: Option<JoinHandle<()>>,
    watch_stop: Arc<AtomicBool>,
}

impl RecordingSession {
    pub fn start(plan: RecordPlan) -> Result<Self, RecordError> {
        std::fs::create_dir_all(&plan.out_dir).map_err(|error| RecordError {
            message: format!("create {}: {error}", plan.out_dir.display()),
        })?;

        if plan.countdown_secs > 0 {
            std::thread::sleep(Duration::from_secs(u64::from(plan.countdown_secs)));
        }

        let origin = Instant::now();
        let halt = Arc::new(AtomicBool::new(false));
        let limit_reason = Arc::new(Mutex::new(None));
        let pids = Arc::new(Mutex::new(Vec::new()));
        let watch_stop = Arc::new(AtomicBool::new(false));
        let encoder = video_encoder_for(&plan, preferred_video_encoder());
        let want_screen = matches!(
            plan.layout,
            LayoutPreset::ScreenCam | LayoutPreset::ScreenOnly
        );
        let want_cam = matches!(plan.layout, LayoutPreset::ScreenCam | LayoutPreset::CamOnly);
        let want_mic = true;

        let mut screen = None;
        let mut screen_path = None;
        let mut screen_offset_ms = 0.0;
        if want_screen {
            let path = plan.out_dir.join(SCREEN_FILE);
            screen = Some(start_screen(&plan, &path, &encoder, Arc::clone(&halt))?);
            screen_offset_ms = origin.elapsed().as_secs_f64() * 1000.0;
            screen_path = Some(path);
        }

        let mut cam = None;
        let mut cam_path = None;
        let mut cam_offset_ms = 0.0;
        if want_cam {
            let path = plan.out_dir.join(CAM_FILE);
            cam = Some(spawn_cam(&plan, &path, &encoder)?);
            cam_offset_ms = origin.elapsed().as_secs_f64() * 1000.0;
            cam_path = Some(path);
        }

        let mut mic = None;
        let mut mic_path = None;
        let mut mic_offset_ms = 0.0;
        if want_mic {
            let path = plan.out_dir.join(MIC_FILE);
            mic = Some(spawn_mic(&plan, &path)?);
            mic_offset_ms = origin.elapsed().as_secs_f64() * 1000.0;
            mic_path = Some(path);
        }

        remember_pids(&pids, screen.as_ref(), cam.as_ref(), mic.as_ref());

        let cursor_path = plan.out_dir.join(CURSOR_FILE);
        let cursor = CursorWriter::start_synthetic(cursor_path, origin)?;
        let overlay_path = plan.out_dir.join(OVERLAY_FILE);
        let overlay = File::create(&overlay_path).map_err(|error| RecordError {
            message: format!("create {}: {error}", overlay_path.display()),
        })?;
        let watch = if plan.fixed_duration_secs.is_none() {
            let dir = plan.out_dir.clone();
            let halt_flag = Arc::clone(&halt);
            let reason = Arc::clone(&limit_reason);
            let watch_pids = Arc::clone(&pids);
            let stop_flag = Arc::clone(&watch_stop);
            Some(thread::spawn(move || {
                watch_take(dir, halt_flag, reason, watch_pids, stop_flag);
            }))
        } else {
            None
        };

        Ok(Self {
            plan,
            origin,
            screen_offset_ms,
            cam_offset_ms,
            mic_offset_ms,
            screen,
            cam,
            mic,
            cursor: Some(cursor),
            overlay: Some(overlay),
            paused: false,
            screen_path,
            cam_path,
            mic_path,
            halt,
            limit_reason,
            watch,
            watch_stop,
        })
    }

    pub fn bytes_written(&self) -> u64 {
        crate::disk::directory_bytes(&self.plan.out_dir)
    }

    pub fn limit_message(&self) -> Option<String> {
        self.limit_reason
            .lock()
            .ok()
            .and_then(|guard| guard.clone())
    }

    pub fn pause(&mut self) -> Result<(), RecordError> {
        if self.paused {
            return Ok(());
        }
        if let Some(screen) = self.screen.as_mut() {
            screen.pause()?;
        }
        signal_children([&mut self.cam, &mut self.mic], Signal::Stop)?;
        self.paused = true;
        Ok(())
    }

    pub fn resume(&mut self) -> Result<(), RecordError> {
        if !self.paused {
            return Ok(());
        }
        if let Some(screen) = self.screen.as_mut() {
            screen.resume()?;
        }
        signal_children([&mut self.cam, &mut self.mic], Signal::Cont)?;
        self.paused = false;
        Ok(())
    }

    pub fn log_overlay(&mut self, kind: &str, label: &str) -> Result<(), RecordError> {
        let Some(file) = self.overlay.as_mut() else {
            return Ok(());
        };
        let event = OverlayEvent {
            t_ms: self.origin.elapsed().as_secs_f64() * 1000.0,
            kind,
            label,
        };
        let line = serde_json::to_string(&event).map_err(|error| RecordError {
            message: format!("overlay json: {error}"),
        })?;
        writeln!(file, "{line}").map_err(|error| RecordError {
            message: format!("write overlay: {error}"),
        })?;
        file.flush().map_err(|error| RecordError {
            message: format!("flush overlay: {error}"),
        })
    }

    pub fn stop(mut self) -> Result<TakeResult, RecordError> {
        self.end_watch();
        if self.paused {
            self.resume_for_stop()?;
        }
        let fixed = self.plan.fixed_duration_secs.is_some();
        // Camera and mic first, so a slow screen finalize cannot leave them running.
        let cam_error = quit_child(&mut self.cam, fixed).err();
        let mic_error = quit_child(&mut self.mic, fixed).err();
        let screen_error = if let Some(screen) = self.screen.take() {
            screen.stop(fixed).err()
        } else {
            None
        };

        let cursor_path = self
            .cursor
            .take()
            .map(|writer| writer.stop())
            .transpose()?
            .unwrap_or_else(|| self.plan.out_dir.join(CURSOR_FILE));

        let samples = parse_cursor_jsonl(&cursor_path)?;
        assert_monotonic(&samples)?;

        let screen_duration_ms = self
            .screen_path
            .as_ref()
            .map(|path| probe_duration_ms(path))
            .transpose()?;
        let cam_duration_ms = self
            .cam_path
            .as_ref()
            .map(|path| probe_duration_ms(path))
            .transpose()?;
        let mic_duration_ms = self
            .mic_path
            .as_ref()
            .map(|path| probe_duration_ms(path))
            .transpose()?;

        // The 50 ms sync gate is the lavfi acceptance test. Interactive stops can
        // land a little wider because each ffmpeg process drains separately.
        if self.plan.fixed_duration_secs.is_some() {
            ensure_within_50ms(screen_duration_ms, cam_duration_ms, mic_duration_ms)?;
        } else if let Some(path) = &self.screen_path
            && path.is_file()
        {
            let source = path.clone();
            let proxy = self.plan.out_dir.join(core::proxy_file_name(SCREEN_FILE));
            thread::spawn(move || {
                let _ = crate::ffmpeg::encode_proxy(&source, &proxy);
            });
        }
        if let Some(error) = screen_error.or(cam_error).or(mic_error) {
            return Err(error);
        }

        let overlay_path = self.plan.out_dir.join(OVERLAY_FILE);
        drop(self.overlay.take());

        let _elapsed = self.origin.elapsed();
        let out_dir = self.plan.out_dir.display().to_string();
        let reason = self.limit_message();

        let screen_path = self.screen_path.take();
        let cam_path = self.cam_path.take();
        let mic_path = self.mic_path.take();
        let result = TakeResult {
            screen: screen_path.map(|path| relative_media(&self.plan.out_dir, &path)),
            cam: cam_path.map(|path| relative_media(&self.plan.out_dir, &path)),
            mic: mic_path.map(|path| relative_media(&self.plan.out_dir, &path)),
            cursor: relative_media(&self.plan.out_dir, &cursor_path),
            sync: TakeSync {
                screen_offset_ms: self.screen_offset_ms,
                cam_offset_ms: self.cam_offset_ms,
                mic_offset_ms: self.mic_offset_ms,
            },
            screen_duration_ms,
            cam_duration_ms,
            mic_duration_ms,
            out_dir: out_dir.clone(),
            overlay: relative_media(&self.plan.out_dir, &overlay_path),
        };
        if let Some(reason) = reason {
            return Err(RecordError {
                message: format!("{reason} Files are in {out_dir}."),
            });
        }
        Ok(result)
    }

    fn end_watch(&mut self) {
        self.watch_stop.store(true, Ordering::Relaxed);
        if let Some(handle) = self.watch.take() {
            let _ = handle.join();
        }
    }

    fn resume_for_stop(&mut self) -> Result<(), RecordError> {
        let halted = self.halt.load(Ordering::Relaxed);
        if let Some(screen) = self.screen.as_mut()
            && let Err(error) = screen.resume()
            && !halted
        {
            return Err(error);
        }
        if let Err(error) = signal_children([&mut self.cam, &mut self.mic], Signal::Cont)
            && !halted
        {
            return Err(error);
        }
        self.paused = false;
        Ok(())
    }
}

impl Drop for RecordingSession {
    fn drop(&mut self) {
        self.watch_stop.store(true, Ordering::Relaxed);
        if let Some(handle) = self.watch.take() {
            let _ = handle.join();
        }
    }
}

#[derive(Serialize)]
struct OverlayEvent<'a> {
    #[serde(rename = "tMs")]
    t_ms: f64,
    kind: &'a str,
    label: &'a str,
}

fn probe_duration_ms(path: &Path) -> Result<f64, RecordError> {
    Ok(probe_duration_secs(path)? * 1000.0)
}

fn ensure_within_50ms(
    screen: Option<f64>,
    cam: Option<f64>,
    mic: Option<f64>,
) -> Result<(), RecordError> {
    let values: Vec<f64> = [screen, cam, mic].into_iter().flatten().collect();
    if values.len() < 2 {
        return Ok(());
    }
    let min = values.iter().cloned().fold(f64::INFINITY, f64::min);
    let max = values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    if max - min > 50.0 {
        return Err(RecordError {
            message: format!(
                "track durations diverge by {:.1} ms (limit 50 ms): screen={screen:?} cam={cam:?} mic={mic:?}",
                max - min
            ),
        });
    }
    Ok(())
}

fn relative_media(out_dir: &Path, path: &Path) -> String {
    path.strip_prefix(out_dir)
        .map(|relative| format!("media/{}", relative.display()))
        .unwrap_or_else(|_| path.display().to_string())
}

enum ScreenHandle {
    Ffmpeg(Child),
    #[cfg(target_os = "macos")]
    Kit(crate::screenkit::KitRecording),
    #[cfg(target_os = "linux")]
    Portal(crate::portal::PortalRecording),
}

impl ScreenHandle {
    fn pause(&mut self) -> Result<(), RecordError> {
        match self {
            Self::Ffmpeg(child) => signal_one(child, Signal::Stop),
            #[cfg(target_os = "macos")]
            Self::Kit(recording) => recording.pause().map_err(|message| RecordError { message }),
            #[cfg(target_os = "linux")]
            Self::Portal(recording) => recording.pause().map_err(|message| RecordError { message }),
        }
    }

    fn resume(&mut self) -> Result<(), RecordError> {
        match self {
            Self::Ffmpeg(child) => signal_one(child, Signal::Cont),
            #[cfg(target_os = "macos")]
            Self::Kit(recording) => recording
                .resume()
                .map_err(|message| RecordError { message }),
            #[cfg(target_os = "linux")]
            Self::Portal(recording) => recording
                .resume()
                .map_err(|message| RecordError { message }),
        }
    }

    fn stop(self, fixed: bool) -> Result<(), RecordError> {
        match self {
            Self::Ffmpeg(child) => {
                let mut child = Some(child);
                quit_child(&mut child, fixed)
            }
            #[cfg(target_os = "macos")]
            Self::Kit(recording) => recording.stop().map_err(|message| RecordError { message }),
            #[cfg(target_os = "linux")]
            Self::Portal(recording) => recording.stop().map_err(|message| RecordError { message }),
        }
    }

    fn process_id(&self) -> Option<u32> {
        match self {
            Self::Ffmpeg(child) => Some(child.id()),
            #[cfg(target_os = "macos")]
            Self::Kit(_) => None,
            #[cfg(target_os = "linux")]
            Self::Portal(recording) => Some(recording.process_id()),
        }
    }
}

fn start_screen(
    plan: &RecordPlan,
    path: &Path,
    encoder: &str,
    halt: Arc<AtomicBool>,
) -> Result<ScreenHandle, RecordError> {
    #[cfg(target_os = "macos")]
    if plan.backend == CaptureBackend::AvFoundation {
        let region = plan
            .region
            .map(|region| (region.x, region.y, region.width, region.height));
        return crate::screenkit::KitRecording::start_with(path, plan.window_id, region, halt)
            .map(ScreenHandle::Kit)
            .map_err(|message| RecordError { message });
    }
    #[cfg(target_os = "linux")]
    if plan.backend == CaptureBackend::LinuxFfmpeg {
        let region = plan
            .region
            .map(|region| (region.x, region.y, region.width, region.height));
        return crate::portal::PortalRecording::start(
            path,
            plan.window_id.is_some(),
            region,
            plan.fixed_duration_secs,
            halt,
        )
        .map(ScreenHandle::Portal)
        .map_err(|message| RecordError { message });
    }
    let _ = halt;
    spawn_screen(plan, path, encoder).map(ScreenHandle::Ffmpeg)
}

fn video_encoder_for(plan: &RecordPlan, preferred: &str) -> String {
    // Lavfi fixture takes stay on libx264 so short MOVs always get a moov atom.
    if plan.backend == CaptureBackend::Lavfi {
        "libx264".to_string()
    } else {
        encoder_or_fallback(preferred)
    }
}

fn spawn_screen(plan: &RecordPlan, path: &Path, encoder: &str) -> Result<Child, RecordError> {
    let mut command = ffmpeg();
    match plan.backend {
        CaptureBackend::Lavfi => {
            command.args(["-f", "lavfi", "-i", "testsrc2=size=1280x720:rate=30"]);
        }
        CaptureBackend::AvFoundation => {
            let id = plan.screen_id.as_deref().unwrap_or("1");
            command.args([
                "-f",
                "avfoundation",
                "-framerate",
                "30",
                "-capture_cursor",
                "0",
                "-i",
                &format!("{id}:none"),
            ]);
        }
        CaptureBackend::LinuxFfmpeg => {
            return Err(RecordError {
                message: "ScreenCast portal capture runs on Linux.".to_string(),
            });
        }
    }
    apply_duration(&mut command, plan.fixed_duration_secs);
    command.args([
        "-an",
        "-c:v",
        encoder,
        "-b:v",
        SCREEN_BITRATE,
        "-pix_fmt",
        "yuv420p",
        "-movflags",
        "+faststart",
    ]);
    if encoder == "libx264" {
        command.args(["-preset", "veryfast"]);
    }
    command.arg(path);
    spawn_ffmpeg(command)
}

fn spawn_cam(plan: &RecordPlan, path: &Path, encoder: &str) -> Result<Child, RecordError> {
    let mut command = ffmpeg();
    match plan.backend {
        CaptureBackend::Lavfi => {
            command.args(["-f", "lavfi", "-i", "testsrc=size=640x480:rate=30"]);
        }
        CaptureBackend::AvFoundation => {
            let id = plan
                .cam_id
                .clone()
                .or_else(|| {
                    crate::devices::pick_camera(&crate::devices::list_devices())
                        .map(|device| device.id.clone())
                })
                .unwrap_or_else(|| "0".to_string());
            command.args([
                "-f",
                "avfoundation",
                "-framerate",
                "30",
                "-video_size",
                "1280x720",
                "-i",
                &format!("{id}:none"),
            ]);
        }
        CaptureBackend::LinuxFfmpeg => {
            let id = plan.cam_id.as_deref().unwrap_or("/dev/video0");
            command.args(["-f", "v4l2", "-framerate", "30", "-i", id]);
        }
    }
    apply_duration(&mut command, plan.fixed_duration_secs);
    command.args([
        "-an",
        "-c:v",
        encoder,
        "-b:v",
        CAM_BITRATE,
        "-pix_fmt",
        "yuv420p",
        "-movflags",
        "+faststart",
    ]);
    if encoder == "libx264" {
        command.args(["-preset", "veryfast"]);
    }
    command.arg(path);
    spawn_ffmpeg(command)
}

fn spawn_mic(plan: &RecordPlan, path: &Path) -> Result<Child, RecordError> {
    let mut command = ffmpeg();
    match plan.backend {
        CaptureBackend::Lavfi => {
            command.args(["-f", "lavfi", "-i", "sine=frequency=440:sample_rate=48000"]);
        }
        CaptureBackend::AvFoundation => {
            let id = plan
                .mic_id
                .clone()
                .or_else(|| {
                    crate::devices::pick_microphone(&crate::devices::list_devices())
                        .map(|device| device.id.clone())
                })
                .unwrap_or_else(|| "0".to_string());
            command.args(["-f", "avfoundation", "-i", &format!("none:{id}")]);
        }
        CaptureBackend::LinuxFfmpeg => {
            let id = plan.mic_id.as_deref().unwrap_or("default");
            command.args(["-f", "pulse", "-i", id]);
        }
    }
    apply_duration(&mut command, plan.fixed_duration_secs);
    command.args(["-vn", "-c:a", "pcm_s16le", "-ar", "48000", "-ac", "1"]);
    command.arg(path);
    spawn_ffmpeg(command)
}

fn apply_duration(command: &mut Command, fixed: Option<f64>) {
    if let Some(secs) = fixed {
        command.args(["-t", &format!("{secs:.3}")]);
    }
}

fn spawn_ffmpeg(mut command: Command) -> Result<Child, RecordError> {
    // stderr must not be a pipe nobody reads — ffmpeg will block once the buffer fills.
    command
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    command.spawn().map_err(|error| RecordError {
        message: format!("spawn ffmpeg: {error}"),
    })
}

fn quit_child(child: &mut Option<Child>, fixed: bool) -> Result<(), RecordError> {
    let Some(child) = child.as_mut() else {
        return Ok(());
    };
    // Fixed-duration children exit on their own; wait for the trailer.
    if fixed {
        for _ in 0..200 {
            if let Ok(Some(_)) = child.try_wait() {
                return Ok(());
            }
            std::thread::sleep(Duration::from_millis(50));
        }
    }
    if let Ok(Some(_)) = child.try_wait() {
        return Ok(());
    }
    if let Some(mut stdin) = child.stdin.take() {
        let _ = stdin.write_all(b"q");
        let _ = stdin.flush();
    }
    let pid = child.id().to_string();
    for signal in ["-INT", "-KILL"] {
        for _ in 0..20 {
            if let Ok(Some(_)) = child.try_wait() {
                return Ok(());
            }
            std::thread::sleep(Duration::from_millis(50));
        }
        let _ = Command::new("kill").args([signal, &pid]).status();
    }
    match child.wait() {
        Ok(_) => Ok(()),
        Err(error) => Err(RecordError {
            message: format!("wait ffmpeg: {error}"),
        }),
    }
}

#[derive(Clone, Copy)]
enum Signal {
    Stop,
    Cont,
}

fn signal_children(children: [&mut Option<Child>; 2], signal: Signal) -> Result<(), RecordError> {
    for child in children {
        if let Some(child) = child.as_mut() {
            signal_one(child, signal)?;
        }
    }
    Ok(())
}

fn signal_one(child: &mut Child, signal: Signal) -> Result<(), RecordError> {
    let flag = match signal {
        Signal::Stop => "-STOP",
        Signal::Cont => "-CONT",
    };
    let status = Command::new("kill")
        .args([flag, &child.id().to_string()])
        .status()
        .map_err(|error| RecordError {
            message: format!("kill {flag}: {error}"),
        })?;
    if status.success() {
        return Ok(());
    }
    if let Ok(Some(_)) = child.try_wait() {
        return Ok(());
    }
    Err(RecordError {
        message: format!("kill {flag} exited {status}"),
    })
}

fn remember_pids(
    pids: &Mutex<Vec<u32>>,
    screen: Option<&ScreenHandle>,
    cam: Option<&Child>,
    mic: Option<&Child>,
) {
    let mut tracked = Vec::new();
    if let Some(id) = screen.and_then(ScreenHandle::process_id) {
        tracked.push(id);
    }
    if let Some(child) = cam {
        tracked.push(child.id());
    }
    if let Some(child) = mic {
        tracked.push(child.id());
    }
    if let Ok(mut guard) = pids.lock() {
        *guard = tracked;
    }
}

fn watch_take(
    dir: PathBuf,
    halt: Arc<AtomicBool>,
    reason: Arc<Mutex<Option<String>>>,
    pids: Arc<Mutex<Vec<u32>>>,
    stop: Arc<AtomicBool>,
) {
    while !stop.load(Ordering::Relaxed) {
        thread::sleep(Duration::from_secs(1));
        if stop.load(Ordering::Relaxed) {
            break;
        }
        let take_bytes = crate::disk::directory_bytes(&dir);
        let free = crate::disk::free_bytes(&dir).unwrap_or(u64::MAX);
        if let Some(message) = take_limit_reason(take_bytes, free) {
            if let Ok(mut slot) = reason.lock()
                && slot.is_none()
            {
                *slot = Some(message.to_string());
            }
            halt.store(true, Ordering::Relaxed);
            let ids = pids.lock().map(|guard| guard.clone()).unwrap_or_default();
            for pid in ids {
                interrupt_pid(pid);
            }
            break;
        }
    }
}

fn interrupt_pid(pid: u32) {
    let id = pid.to_string();
    let _ = Command::new("kill").args(["-CONT", &id]).status();
    let _ = Command::new("kill").args(["-INT", &id]).status();
}

/// Lavfi take used by the automated gate.
pub fn record_lavfi_fixture(
    out_dir: impl Into<PathBuf>,
    duration_secs: f64,
    layout: LayoutPreset,
) -> Result<TakeResult, RecordError> {
    let session = RecordingSession::start(RecordPlan {
        out_dir: out_dir.into(),
        layout,
        backend: CaptureBackend::Lavfi,
        screen_id: None,
        cam_id: None,
        mic_id: None,
        countdown_secs: 0,
        fixed_duration_secs: Some(duration_secs),
        window_id: None,
        region: None,
    })?;
    // Fixed-duration children exit on their own; still run stop to harvest + probe.
    // Give ffmpeg a moment past the nominal duration for trailer write.
    std::thread::sleep(Duration::from_millis(
        ((duration_secs * 1000.0) as u64).saturating_add(400),
    ));
    session.stop()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cursor::parse_cursor_jsonl;

    fn test_dir(name: &str) -> PathBuf {
        let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../target/m06-record-tests")
            .join(name)
            .join(std::process::id().to_string());
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn lavfi_screen_cam_mic_durations_agree() {
        let dir = test_dir("three-tracks");
        let result = record_lavfi_fixture(&dir, 1.0, LayoutPreset::ScreenCam).unwrap();
        assert!(dir.join(SCREEN_FILE).is_file());
        assert!(dir.join(CAM_FILE).is_file());
        assert!(dir.join(MIC_FILE).is_file());
        assert!(dir.join(CURSOR_FILE).is_file());
        assert!(dir.join(OVERLAY_FILE).is_file());
        let screen = result.screen_duration_ms.unwrap();
        let cam = result.cam_duration_ms.unwrap();
        let mic = result.mic_duration_ms.unwrap();
        assert!((screen - cam).abs() <= 50.0, "{screen} vs {cam}");
        assert!((screen - mic).abs() <= 50.0, "{screen} vs {mic}");
        let samples = parse_cursor_jsonl(&dir.join(CURSOR_FILE)).unwrap();
        assert!(samples.len() > 10);
        assert_monotonic(&samples).unwrap();
    }

    #[test]
    #[cfg(target_os = "macos")]
    #[ignore = "needs Screen Recording permission for this test binary"]
    fn screenkit_writes_mov() {
        let dir = test_dir("screenkit");
        let result = crate::screenkit::KitRecording::start(&dir.join(SCREEN_FILE));
        let recording = result.expect("screen capture start");
        std::thread::sleep(Duration::from_millis(800));
        recording.stop().expect("screen capture stop");
        let path = dir.join(SCREEN_FILE);
        assert!(path.is_file());
        assert!(std::fs::metadata(&path).unwrap().len() > 1000);
    }

    #[test]
    fn portal_capture_is_linux_only() {
        if cfg!(target_os = "linux") {
            return;
        }
        let dir = test_dir("portal-off");
        let started = RecordingSession::start(RecordPlan {
            out_dir: dir,
            layout: LayoutPreset::ScreenOnly,
            backend: CaptureBackend::LinuxFfmpeg,
            screen_id: None,
            cam_id: None,
            mic_id: None,
            countdown_secs: 0,
            fixed_duration_secs: Some(0.2),
            window_id: None,
            region: None,
        });
        let Err(error) = started else {
            panic!("portal capture started off Linux");
        };
        assert!(error.message.contains("Linux"));
    }

    #[test]
    fn take_limit_stops_at_two_gigabytes_or_low_disk() {
        assert!(take_limit_reason(1_999_999_999, 2_000_000_000).is_none());
        assert_eq!(
            take_limit_reason(MAX_TAKE_BYTES, 2_000_000_000),
            Some("Recording stopped at 2 GB so the take cannot fill the disk.")
        );
        assert_eq!(
            take_limit_reason(0, MIN_FREE_BYTES - 1),
            Some("Recording stopped because less than 1 GB is free.")
        );
    }

    #[test]
    fn lavfi_screen_only_skips_cam() {
        let dir = test_dir("screen-only");
        let result = record_lavfi_fixture(&dir, 0.5, LayoutPreset::ScreenOnly).unwrap();
        assert!(result.screen.is_some());
        assert!(result.cam.is_none());
        assert!(result.mic.is_some());
        assert!(!dir.join(CAM_FILE).exists());
    }
}
