//! A take: three lavfi files, one cursor log, and one clock.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::thread;
use std::time::Instant;

use serde::{Deserialize, Serialize};

pub const COUNTDOWN_MS: u64 = 3_000;
pub const CURSOR_HZ: u64 = 120;
pub const VIDEO_BITRATE_KBPS: u64 = 40_000;
pub const AUDIO_BITRATE_KBPS: u64 = 384;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CursorSample {
    pub t_ms: u64,
    pub x: f64,
    pub y: f64,
    pub click: bool,
    pub keys: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CaptureDevice {
    pub id: String,
    pub label: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceCatalog {
    pub displays: Vec<CaptureDevice>,
    pub cameras: Vec<CaptureDevice>,
    pub microphones: Vec<CaptureDevice>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CursorMode {
    Hidden,
    Embedded,
    Metadata,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DesktopIcons {
    Shown,
    HideBestEffort,
}

#[derive(Debug, Clone)]
pub struct SessionClock {
    origin_ms: u64,
    countdown_ms: u64,
    paused_ms: u64,
    pause_mark: Option<u64>,
}

impl SessionClock {
    pub fn new(origin_ms: u64) -> Self {
        Self {
            origin_ms,
            countdown_ms: COUNTDOWN_MS,
            paused_ms: 0,
            pause_mark: None,
        }
    }

    pub fn pause(&mut self, now_ms: u64) {
        if self.pause_mark.is_none() {
            self.pause_mark = Some(now_ms);
        }
    }

    pub fn resume(&mut self, now_ms: u64) {
        if let Some(start) = self.pause_mark.take() {
            self.paused_ms = self.paused_ms.saturating_add(now_ms.saturating_sub(start));
        }
    }

    pub fn recorded_ms(&self, now_ms: u64) -> u64 {
        let ready = self.origin_ms.saturating_add(self.countdown_ms);
        if now_ms < ready {
            return 0;
        }
        let held = self
            .pause_mark
            .map(|start| now_ms.saturating_sub(start))
            .unwrap_or(0);
        now_ms
            .saturating_sub(ready)
            .saturating_sub(self.paused_ms)
            .saturating_sub(held)
    }
}

pub fn fixture_devices() -> DeviceCatalog {
    DeviceCatalog {
        displays: vec![device("display-0", "Fixture display")],
        cameras: vec![device("cam-0", "Fixture camera")],
        microphones: vec![device("mic-0", "Fixture microphone")],
    }
}

fn device(id: &str, label: &str) -> CaptureDevice {
    CaptureDevice {
        id: id.to_string(),
        label: label.to_string(),
    }
}

pub fn choose_cursor_mode(available: &[CursorMode]) -> CursorMode {
    if available.contains(&CursorMode::Metadata) {
        CursorMode::Metadata
    } else {
        CursorMode::Hidden
    }
}

pub fn desktop_icons_plan() -> DesktopIcons {
    if cfg!(target_os = "macos") {
        DesktopIcons::HideBestEffort
    } else {
        DesktopIcons::Shown
    }
}

pub fn pick_h264_encoder(available: &[&str]) -> &'static str {
    const ORDER: [&str; 4] = ["h264_videotoolbox", "h264_vaapi", "h264_nvenc", "libx264"];
    ORDER
        .into_iter()
        .find(|name| available.contains(name))
        .unwrap_or("libx264")
}

pub fn estimate_bytes(duration_s: f64, video_kbps: u64, audio_kbps: u64, video_tracks: u64) -> u64 {
    if !duration_s.is_finite() || duration_s <= 0.0 {
        return 0;
    }
    let kbps = video_kbps
        .saturating_mul(video_tracks)
        .saturating_add(audio_kbps);
    (kbps as f64 * 1_000.0 / 8.0 * duration_s).round() as u64
}

pub fn mic_level(samples: &[i16]) -> f64 {
    let peak = samples
        .iter()
        .map(|sample| sample.unsigned_abs())
        .max()
        .unwrap_or(0);
    (f64::from(peak) / f64::from(i16::MAX)).min(1.0)
}

pub fn sample_cursor(duration_ms: u64, rate_hz: u64) -> Vec<CursorSample> {
    if duration_ms == 0 || rate_hz == 0 {
        return Vec::new();
    }
    let mut samples = Vec::new();
    let mut index = 0_u64;
    loop {
        let t_ms = index.saturating_mul(1_000) / rate_hz;
        if t_ms >= duration_ms {
            break;
        }
        samples.push(CursorSample {
            t_ms,
            x: f64::from(u32::try_from(index % 200).unwrap_or(0)),
            y: 10.0,
            click: index > 0 && index.is_multiple_of(40),
            keys: if index == 10 {
                vec!["space".to_string()]
            } else {
                Vec::new()
            },
        });
        index += 1;
    }
    samples
}

pub fn write_cursor_jsonl(path: &Path, samples: &[CursorSample]) -> Result<(), String> {
    let mut body = String::new();
    for sample in samples {
        let line = serde_json::to_string(sample).map_err(|error| error.to_string())?;
        body.push_str(&line);
        body.push('\n');
    }
    fs::write(path, body).map_err(|error| error.to_string())
}

pub fn parse_cursor_jsonl(text: &str) -> Result<Vec<CursorSample>, String> {
    let mut samples = Vec::new();
    for (index, line) in text.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let sample: CursorSample =
            serde_json::from_str(line).map_err(|error| format!("line {}: {error}", index + 1))?;
        if !sample.x.is_finite() || !sample.y.is_finite() {
            return Err(format!("line {}: cursor position is not finite", index + 1));
        }
        samples.push(sample);
    }
    Ok(samples)
}

pub fn cursor_is_monotonic(samples: &[CursorSample]) -> bool {
    samples.windows(2).all(|pair| pair[1].t_ms > pair[0].t_ms)
}

#[derive(Debug, Clone, PartialEq)]
pub struct TakeOutput {
    pub screen: PathBuf,
    pub cam: PathBuf,
    pub mic: PathBuf,
    pub cursor: PathBuf,
    pub screen_offset_ms: f64,
    pub cam_offset_ms: f64,
    pub mic_offset_ms: f64,
}

pub fn record_fixture_take(dir: &Path, duration_s: f64) -> Result<TakeOutput, String> {
    if !duration_s.is_finite() || duration_s <= 0.0 {
        return Err("duration must be above zero".into());
    }
    fs::create_dir_all(dir).map_err(|error| error.to_string())?;
    let seconds = seconds_arg(duration_s);
    let screen = dir.join("screen.mov");
    let cam = dir.join("cam.mov");
    let mic = dir.join("mic.wav");
    let cursor = dir.join("cursor.jsonl");
    let duration_ms = (duration_s * 1_000.0).round() as u64;
    write_cursor_jsonl(&cursor, &sample_cursor(duration_ms, CURSOR_HZ))?;

    let screen_args = video_args(&seconds, "64x64", &screen);
    let cam_args = video_args(&seconds, "32x32", &cam);
    let mic_args = audio_args(&seconds, &mic);
    let origin = Instant::now();
    let (screen_offset_ms, cam_offset_ms, mic_offset_ms) = thread::scope(|scope| {
        let screen_job = scope.spawn(|| run_ffmpeg(&screen_args, origin));
        let cam_job = scope.spawn(|| run_ffmpeg(&cam_args, origin));
        let mic_job = scope.spawn(|| run_ffmpeg(&mic_args, origin));
        Ok::<_, String>((
            screen_job
                .join()
                .map_err(|_| "screen thread stopped".to_string())??,
            cam_job
                .join()
                .map_err(|_| "cam thread stopped".to_string())??,
            mic_job
                .join()
                .map_err(|_| "mic thread stopped".to_string())??,
        ))
    })?;
    Ok(TakeOutput {
        screen,
        cam,
        mic,
        cursor,
        screen_offset_ms,
        cam_offset_ms,
        mic_offset_ms,
    })
}

fn seconds_arg(duration_s: f64) -> String {
    if (duration_s - duration_s.round()).abs() < 0.001 {
        format!("{}", duration_s.round() as u64)
    } else {
        format!("{duration_s:.3}")
    }
}

fn video_args(seconds: &str, size: &str, path: &Path) -> Vec<String> {
    vec![
        "-y".into(),
        "-hide_banner".into(),
        "-loglevel".into(),
        "error".into(),
        "-f".into(),
        "lavfi".into(),
        "-i".into(),
        format!("testsrc2=size={size}:rate=30:duration={seconds}"),
        "-an".into(),
        "-pix_fmt".into(),
        "yuv420p".into(),
        "-c:v".into(),
        "libx264".into(),
        path.display().to_string(),
    ]
}

fn audio_args(seconds: &str, path: &Path) -> Vec<String> {
    vec![
        "-y".into(),
        "-hide_banner".into(),
        "-loglevel".into(),
        "error".into(),
        "-f".into(),
        "lavfi".into(),
        "-i".into(),
        format!("sine=frequency=440:sample_rate=48000:duration={seconds}"),
        "-c:a".into(),
        "pcm_s16le".into(),
        path.display().to_string(),
    ]
}

fn run_ffmpeg(args: &[String], origin: Instant) -> Result<f64, String> {
    let offset_ms = origin.elapsed().as_secs_f64() * 1_000.0;
    let output = Command::new("ffmpeg")
        .args(args)
        .output()
        .map_err(|error| error.to_string())?;
    if output.status.success() {
        Ok(offset_ms)
    } else {
        Err(String::from_utf8_lossy(&output.stderr).trim().to_string())
    }
}

pub fn media_duration_seconds(path: &Path) -> Result<f64, String> {
    let output = Command::new("ffprobe")
        .args([
            "-v",
            "error",
            "-show_entries",
            "format=duration",
            "-of",
            "csv=p=0",
        ])
        .arg(path)
        .output()
        .map_err(|error| error.to_string())?;
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).trim().to_string());
    }
    let text = String::from_utf8_lossy(&output.stdout);
    text.trim()
        .parse::<f64>()
        .map_err(|error| format!("duration: {error}"))
}

#[cfg(test)]
mod tests {
    use super::{
        AUDIO_BITRATE_KBPS, CURSOR_HZ, CursorMode, DesktopIcons, SessionClock, VIDEO_BITRATE_KBPS,
        choose_cursor_mode, cursor_is_monotonic, desktop_icons_plan, estimate_bytes,
        fixture_devices, media_duration_seconds, mic_level, parse_cursor_jsonl, pick_h264_encoder,
        record_fixture_take, sample_cursor,
    };

    fn near(actual: f64, expected: f64) {
        assert!((actual - expected).abs() < 0.001, "{actual} vs {expected}");
    }

    #[test]
    fn the_clock_waits_out_the_countdown_and_a_pause() {
        let mut clock = SessionClock::new(1_000);
        assert_eq!(clock.recorded_ms(1_000), 0);
        assert_eq!(clock.recorded_ms(3_999), 0);
        assert_eq!(clock.recorded_ms(5_000), 1_000);
        clock.pause(5_000);
        assert_eq!(clock.recorded_ms(5_400), 1_000);
        clock.resume(5_400);
        assert_eq!(clock.recorded_ms(5_900), 1_500);
    }

    #[test]
    fn cursor_samples_round_trip_and_stay_monotonic() {
        let samples = sample_cursor(1_000, CURSOR_HZ);
        assert_eq!(samples.len(), 120);
        assert!(cursor_is_monotonic(&samples));
        assert!(samples.iter().any(|sample| sample.click));
        assert!(samples.iter().any(|sample| sample.keys == ["space"]));
        let text = samples
            .iter()
            .map(|sample| serde_json::to_string(sample).unwrap())
            .collect::<Vec<_>>()
            .join("\n");
        let parsed = parse_cursor_jsonl(&text).unwrap();
        assert_eq!(parsed, samples);
        let reversed = "\
{\"tMs\":20,\"x\":1,\"y\":1,\"click\":false,\"keys\":[]}\n\
{\"tMs\":10,\"x\":1,\"y\":1,\"click\":false,\"keys\":[]}\n";
        let backwards = parse_cursor_jsonl(reversed).unwrap();
        assert!(!cursor_is_monotonic(&backwards));
    }

    #[test]
    fn encoder_meter_estimate_and_cursor_mode() {
        assert_eq!(
            pick_h264_encoder(&["libx264", "h264_videotoolbox"]),
            "h264_videotoolbox"
        );
        assert_eq!(pick_h264_encoder(&["libx264"]), "libx264");
        assert_eq!(pick_h264_encoder(&[]), "libx264");
        near(mic_level(&[]), 0.0);
        near(mic_level(&[0, i16::MAX, -1_000]), 1.0);
        assert_eq!(
            estimate_bytes(30.0, VIDEO_BITRATE_KBPS, AUDIO_BITRATE_KBPS, 2),
            301_440_000
        );
        assert_eq!(
            estimate_bytes(0.0, VIDEO_BITRATE_KBPS, AUDIO_BITRATE_KBPS, 2),
            0
        );
        assert_eq!(
            choose_cursor_mode(&[CursorMode::Hidden, CursorMode::Metadata]),
            CursorMode::Metadata
        );
        assert_eq!(
            choose_cursor_mode(&[CursorMode::Embedded]),
            CursorMode::Hidden
        );
        let devices = fixture_devices();
        assert_eq!(devices.displays[0].label, "Fixture display");
        assert_eq!(devices.cameras[0].label, "Fixture camera");
        assert_eq!(devices.microphones[0].label, "Fixture microphone");
        if cfg!(target_os = "macos") {
            assert_eq!(desktop_icons_plan(), DesktopIcons::HideBestEffort);
        } else {
            assert_eq!(desktop_icons_plan(), DesktopIcons::Shown);
        }
    }

    #[test]
    fn lavfi_take_writes_three_files_with_matching_durations() {
        let dir = std::env::temp_dir().join(format!(
            "shorts-take-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let take = record_fixture_take(&dir, 1.0).unwrap();
        let screen = media_duration_seconds(&take.screen).unwrap();
        let cam = media_duration_seconds(&take.cam).unwrap();
        let mic = media_duration_seconds(&take.mic).unwrap();
        assert!((screen - cam).abs() < 0.05, "screen {screen} cam {cam}");
        assert!((screen - mic).abs() < 0.05, "screen {screen} mic {mic}");
        assert!(take.screen_offset_ms.is_finite() && take.screen_offset_ms >= 0.0);
        assert!(take.cam_offset_ms.is_finite());
        assert!(take.mic_offset_ms.is_finite());
        let cursor = std::fs::read_to_string(&take.cursor).unwrap();
        let samples = parse_cursor_jsonl(&cursor).unwrap();
        assert_eq!(samples.len(), 120);
        assert!(cursor_is_monotonic(&samples));
        let _ = std::fs::remove_dir_all(&dir);
    }
}
