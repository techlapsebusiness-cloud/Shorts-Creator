//! macOS screen, camera, and mic capture through FFmpeg's avfoundation device.

#[cfg(target_os = "macos")]
use std::fs;
#[cfg(target_os = "macos")]
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
#[cfg(target_os = "macos")]
use std::process::Stdio;
use std::process::{Child, Command};
#[cfg(target_os = "macos")]
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;
use std::time::{Duration, Instant};

use crate::exclusion::RefuseReason;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AvKind {
    Video,
    Audio,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AvDevice {
    pub index: u32,
    pub name: String,
    pub kind: AvKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AvPlan {
    pub screen: u32,
    pub camera: Option<u32>,
    pub microphone: Option<u32>,
}

struct Running {
    child: Child,
    #[cfg(target_os = "macos")]
    log: Arc<Mutex<String>>,
}

struct LiveTake {
    dir: PathBuf,
    jobs: Vec<Running>,
}

static LIVE: Mutex<Option<LiveTake>> = Mutex::new(None);

pub fn parse_avfoundation_devices(text: &str) -> Vec<AvDevice> {
    let mut kind = None;
    let mut devices = Vec::new();
    for line in text.lines() {
        let lower = line.to_ascii_lowercase();
        if lower.contains("video devices") {
            kind = Some(AvKind::Video);
            continue;
        }
        if lower.contains("audio devices") {
            kind = Some(AvKind::Audio);
            continue;
        }
        let Some(kind) = kind else {
            continue;
        };
        let Some((index, name)) = device_name(line) else {
            continue;
        };
        devices.push(AvDevice { index, name, kind });
    }
    devices
}

fn device_name(line: &str) -> Option<(u32, String)> {
    let start = line.rfind("] [")?;
    let rest = &line[start + 2..];
    let close = rest.find(']')?;
    let index = rest[1..close].parse().ok()?;
    let name = rest[close + 1..].trim();
    if name.is_empty() {
        None
    } else {
        Some((index, name.to_string()))
    }
}

pub fn plan_avfoundation(devices: &[AvDevice]) -> Result<AvPlan, String> {
    let screen = devices.iter().find(|device| {
        device.kind == AvKind::Video && device.name.to_ascii_lowercase().contains("capture screen")
    });
    let Some(screen) = screen else {
        return Err(
            "No Capture screen device. Allow Screen Recording for this app, then press Start recording again."
                .into(),
        );
    };
    let camera = devices.iter().find(|device| {
        device.kind == AvKind::Video && !device.name.to_ascii_lowercase().contains("capture screen")
    });
    let microphone = devices.iter().find(|device| device.kind == AvKind::Audio);
    Ok(AvPlan {
        screen: screen.index,
        camera: camera.map(|device| device.index),
        microphone: microphone.map(|device| device.index),
    })
}

pub fn screen_capture_args(index: u32, path: &Path) -> Vec<String> {
    video_args(index, "40M", path)
}

pub fn camera_capture_args(index: u32, path: &Path) -> Vec<String> {
    video_args(index, "8M", path)
}

fn video_args(index: u32, bitrate: &str, path: &Path) -> Vec<String> {
    vec![
        "-y".into(),
        "-hide_banner".into(),
        "-loglevel".into(),
        "error".into(),
        "-f".into(),
        "avfoundation".into(),
        "-capture_cursor".into(),
        "0".into(),
        "-framerate".into(),
        "30".into(),
        "-i".into(),
        format!("{index}:none"),
        "-an".into(),
        "-pix_fmt".into(),
        "yuv420p".into(),
        "-c:v".into(),
        "h264_videotoolbox".into(),
        "-b:v".into(),
        bitrate.into(),
        path.display().to_string(),
    ]
}

pub fn mic_capture_args(index: u32, path: &Path) -> Vec<String> {
    vec![
        "-y".into(),
        "-hide_banner".into(),
        "-loglevel".into(),
        "error".into(),
        "-f".into(),
        "avfoundation".into(),
        "-i".into(),
        format!("none:{index}"),
        "-ac".into(),
        "1".into(),
        "-ar".into(),
        "48000".into(),
        "-c:a".into(),
        "pcm_s16le".into(),
        path.display().to_string(),
    ]
}

pub fn begin_live_take(dir: &Path) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        return start_avfoundation(dir);
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = dir;
        Err(RefuseReason::MonitorCaptureIncludesPrompter.to_string())
    }
}

pub fn end_live_take() -> Result<String, String> {
    let take = live_lock()
        .take()
        .ok_or_else(|| "Not recording".to_string())?;
    for mut job in take.jobs {
        stop_child(&mut job.child);
    }
    Ok(take.dir.display().to_string())
}

#[cfg(target_os = "macos")]
fn start_avfoundation(dir: &Path) -> Result<(), String> {
    {
        let live = live_lock();
        if live.is_some() {
            return Err("Already recording. Press Stop recording first.".into());
        }
    }
    let listed = read_avfoundation_list()?;
    let plan = plan_avfoundation(&parse_avfoundation_devices(&listed))?;
    fs::create_dir_all(dir).map_err(|error| error.to_string())?;
    fs::write(dir.join("cursor.jsonl"), "").map_err(|error| error.to_string())?;
    let mut jobs = Vec::new();
    let screen = spawn_job(&screen_capture_args(plan.screen, &dir.join("screen.mov")))?;
    jobs.push(screen);
    if let Some(index) = plan.camera {
        match spawn_job(&camera_capture_args(index, &dir.join("cam.mov"))) {
            Ok(job) => jobs.push(job),
            Err(error) => {
                stop_jobs(&mut jobs);
                return Err(error);
            }
        }
    }
    if let Some(index) = plan.microphone {
        match spawn_job(&mic_capture_args(index, &dir.join("mic.wav"))) {
            Ok(job) => jobs.push(job),
            Err(error) => {
                stop_jobs(&mut jobs);
                return Err(error);
            }
        }
    }
    thread::sleep(Duration::from_millis(700));
    if let Some(error) = early_exit(&mut jobs) {
        stop_jobs(&mut jobs);
        return Err(error);
    }
    *live_lock() = Some(LiveTake {
        dir: dir.to_path_buf(),
        jobs,
    });
    Ok(())
}

fn live_lock() -> std::sync::MutexGuard<'static, Option<LiveTake>> {
    LIVE.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
}

#[cfg(target_os = "macos")]
fn read_avfoundation_list() -> Result<String, String> {
    let output = Command::new("ffmpeg")
        .args([
            "-hide_banner",
            "-f",
            "avfoundation",
            "-list_devices",
            "true",
            "-i",
            "",
        ])
        .output()
        .map_err(|error| {
            format!("ffmpeg is not available ({error}). Install it with brew install ffmpeg.")
        })?;
    let text = String::from_utf8_lossy(&output.stderr).into_owned();
    if parse_avfoundation_devices(&text).is_empty() {
        return Err(format!(
            "No cameras, screens, or microphones. {}",
            tail(&text)
        ));
    }
    Ok(text)
}

#[cfg(target_os = "macos")]
fn spawn_job(args: &[String]) -> Result<Running, String> {
    let mut child = Command::new("ffmpeg")
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| format!("ffmpeg did not start ({error})"))?;
    let log = Arc::new(Mutex::new(String::new()));
    if let Some(stderr) = child.stderr.take() {
        let log = Arc::clone(&log);
        thread::spawn(move || {
            let mut reader = BufReader::new(stderr);
            let mut line = String::new();
            while reader.read_line(&mut line).unwrap_or(0) > 0 {
                let mut guard = log.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
                guard.push_str(&line);
                if guard.len() > 4_000 {
                    let extra = guard.len() - 4_000;
                    guard.drain(..extra);
                }
                line.clear();
            }
        });
    }
    Ok(Running { child, log })
}

#[cfg(target_os = "macos")]
fn early_exit(jobs: &mut [Running]) -> Option<String> {
    for job in jobs.iter_mut() {
        match job.child.try_wait() {
            Ok(Some(status)) => {
                let log = job
                    .log
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner())
                    .clone();
                return Some(format!(
                    "macOS blocked the capture ({status}). Allow Screen Recording, Camera, and Microphone, then press Start recording again. {}",
                    tail(&log)
                ));
            }
            Ok(None) => {}
            Err(error) => return Some(error.to_string()),
        }
    }
    None
}

#[cfg(target_os = "macos")]
fn stop_jobs(jobs: &mut Vec<Running>) {
    for job in jobs {
        stop_child(&mut job.child);
    }
}

fn stop_child(child: &mut Child) {
    let id = child.id().to_string();
    let _ = Command::new("kill").args(["-INT", &id]).status();
    let deadline = Instant::now() + Duration::from_secs(2);
    while Instant::now() < deadline {
        if child.try_wait().ok().flatten().is_some() {
            return;
        }
        thread::sleep(Duration::from_millis(40));
    }
    let _ = child.kill();
    let _ = child.wait();
}

#[cfg(target_os = "macos")]
fn tail(text: &str) -> String {
    text.lines()
        .rev()
        .take(3)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::{
        camera_capture_args, mic_capture_args, parse_avfoundation_devices, plan_avfoundation,
        screen_capture_args,
    };

    const LIST: &str = "\
[AVFoundation indev @ 0x1] AVFoundation video devices:
[AVFoundation indev @ 0x1] [0] FaceTime HD Camera
[AVFoundation indev @ 0x1] [1] Capture screen 0
[AVFoundation indev @ 0x1] AVFoundation audio devices:
[AVFoundation indev @ 0x1] [0] MacBook Air Microphone
";

    #[test]
    fn a_device_list_picks_the_screen_camera_and_mic() {
        let devices = parse_avfoundation_devices(LIST);
        assert_eq!(devices.len(), 3);
        let plan = plan_avfoundation(&devices).unwrap();
        assert_eq!(plan.screen, 1);
        assert_eq!(plan.camera, Some(0));
        assert_eq!(plan.microphone, Some(0));
        let screen = screen_capture_args(plan.screen, std::path::Path::new("screen.mov"));
        assert!(screen.windows(2).any(|pair| pair == ["-i", "1:none"]));
        assert!(
            screen
                .windows(2)
                .any(|pair| pair == ["-c:v", "h264_videotoolbox"])
        );
        assert!(
            screen
                .windows(2)
                .any(|pair| pair == ["-capture_cursor", "0"])
        );
        let camera = camera_capture_args(0, std::path::Path::new("cam.mov"));
        assert!(camera.windows(2).any(|pair| pair == ["-i", "0:none"]));
        let mic = mic_capture_args(0, std::path::Path::new("mic.wav"));
        assert!(mic.windows(2).any(|pair| pair == ["-i", "none:0"]));
        assert!(mic.windows(2).any(|pair| pair == ["-ar", "48000"]));
    }

    #[test]
    fn a_list_without_a_screen_explains_the_permission() {
        let devices = parse_avfoundation_devices(
            "[AVFoundation indev @ 0x1] AVFoundation video devices:\n[AVFoundation indev @ 0x1] [0] FaceTime HD Camera\n",
        );
        let error = plan_avfoundation(&devices).unwrap_err();
        assert!(error.contains("Screen Recording"));
    }

    #[test]
    fn this_host_does_not_open_a_monitor_capture_over_the_prompter() {
        if cfg!(target_os = "macos") {
            return;
        }
        let error = super::begin_live_take(std::path::Path::new("/tmp")).unwrap_err();
        assert!(error.contains("another monitor"));
    }
}
