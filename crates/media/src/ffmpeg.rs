//! Thin wrappers around ffmpeg and ffprobe child processes.

use std::path::Path;
use std::process::{Command, Output, Stdio};

#[derive(Debug)]
pub struct FfmpegError {
    pub message: String,
}

impl std::fmt::Display for FfmpegError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for FfmpegError {}

pub fn ffmpeg() -> Command {
    let mut command = Command::new("ffmpeg");
    command.arg("-hide_banner").arg("-y");
    command
}

pub fn ffprobe() -> Command {
    let mut command = Command::new("ffprobe");
    command.arg("-hide_banner");
    command
}

pub fn run_checked(mut command: Command) -> Result<Output, FfmpegError> {
    let output = command
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .map_err(|error| FfmpegError {
            message: format!("failed to spawn {}: {error}", command_name(&command)),
        })?;
    if output.status.success() {
        Ok(output)
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(FfmpegError {
            message: format!(
                "{} exited {}: {}",
                command_name(&command),
                output.status,
                stderr.trim()
            ),
        })
    }
}

pub fn probe_duration_secs(path: &Path) -> Result<f64, FfmpegError> {
    let mut command = ffprobe();
    command
        .args([
            "-v",
            "error",
            "-show_entries",
            "format=duration",
            "-of",
            "default=nw=1:nk=1",
        ])
        .arg(path);
    let output = run_checked(command)?;
    let text = String::from_utf8_lossy(&output.stdout);
    let line = text
        .lines()
        .find(|line| !line.trim().is_empty())
        .ok_or_else(|| FfmpegError {
            message: format!("ffprobe returned no duration for {}", path.display()),
        })?;
    line.trim().parse::<f64>().map_err(|error| FfmpegError {
        message: format!(
            "ffprobe duration parse failed for {}: {error}",
            path.display()
        ),
    })
}

pub fn preferred_video_encoder() -> &'static str {
    if cfg!(target_os = "macos") {
        "h264_videotoolbox"
    } else {
        "libx264"
    }
}

pub fn encoder_or_fallback(preferred: &str) -> String {
    match Command::new("ffmpeg")
        .args(["-hide_banner", "-encoders"])
        .output()
    {
        Ok(output) if output.status.success() => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            if stdout
                .lines()
                .any(|line| line.split_whitespace().nth(1) == Some(preferred))
            {
                preferred.to_string()
            } else {
                "libx264".to_string()
            }
        }
        _ => "libx264".to_string(),
    }
}

const PROXY_SKIP_ABOVE: u64 = 1_000_000_000;

pub fn encode_proxy(source: &Path, dest: &Path) -> Result<(), FfmpegError> {
    if std::fs::metadata(source)
        .map(|meta| meta.len())
        .unwrap_or(0)
        > PROXY_SKIP_ABOVE
    {
        return Ok(());
    }
    let mut command = ffmpeg();
    command.arg("-i").arg(source).args([
        "-vf",
        "scale=1280:720",
        "-c:v",
        "libx264",
        "-preset",
        "veryfast",
        "-b:v",
        "4M",
        "-pix_fmt",
        "yuv420p",
        "-an",
        "-movflags",
        "+faststart",
    ]);
    command.arg(dest);
    run_checked(command)?;
    Ok(())
}

fn command_name(command: &Command) -> String {
    command.get_program().to_string_lossy().into_owned()
}
