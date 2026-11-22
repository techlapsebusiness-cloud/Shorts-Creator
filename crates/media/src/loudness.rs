//! Two-pass loudnorm to −14 LUFS and −1.0 dBTP, then an ebur128 check.

use std::path::Path;
use std::process::Stdio;

use serde_json::Value;

use crate::ffmpeg::{FfmpegError, ffmpeg};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LoudnessReport {
    pub integrated_lufs: f64,
    pub true_peak_db: f64,
}

#[derive(Debug, Clone, Copy)]
struct MeasuredPass {
    integrated: f64,
    true_peak: f64,
    lra: f64,
    threshold: f64,
    offset: f64,
}

pub fn normalize_loudness(input: &Path, output: &Path) -> Result<LoudnessReport, FfmpegError> {
    let measured = measure_pass(input)?;
    let filter = format!(
        "loudnorm=I={}:TP={}:LRA=11:measured_I={}:measured_TP={}:measured_LRA={}:measured_thresh={}:offset={}:linear=true",
        core::TARGET_LUFS,
        core::TARGET_TRUE_PEAK_DB,
        measured.integrated,
        measured.true_peak,
        measured.lra,
        measured.threshold,
        measured.offset,
    );
    let mut command = ffmpeg();
    command
        .arg("-i")
        .arg(input)
        .args(["-af", &filter, "-ar", "48000", "-c:a", "pcm_s16le"])
        .arg(output)
        .stdin(Stdio::null())
        .stdout(Stdio::null());
    let rendered = command.output().map_err(|error| FfmpegError {
        message: format!("loudnorm: {error}"),
    })?;
    if !rendered.status.success() {
        return Err(FfmpegError {
            message: format!(
                "loudnorm exited {}: {}",
                rendered.status,
                String::from_utf8_lossy(&rendered.stderr).trim()
            ),
        });
    }
    measure_ebur128(output)
}

pub fn noise_reducer() -> core::NoiseReducer {
    if ffmpeg_has_filter("arnndn") {
        core::NoiseReducer::Arnndn
    } else {
        core::NoiseReducer::Afftdn
    }
}

fn measure_pass(input: &Path) -> Result<MeasuredPass, FfmpegError> {
    let mut command = ffmpeg();
    command
        .arg("-i")
        .arg(input)
        .args([
            "-af",
            &format!(
                "loudnorm=I={}:TP={}:LRA=11:print_format=json",
                core::TARGET_LUFS,
                core::TARGET_TRUE_PEAK_DB
            ),
            "-f",
            "null",
            "-",
        ])
        .stdin(Stdio::null())
        .stdout(Stdio::null());
    let output = command.output().map_err(|error| FfmpegError {
        message: format!("loudnorm measure: {error}"),
    })?;
    if !output.status.success() {
        return Err(FfmpegError {
            message: format!(
                "loudnorm measure exited {}: {}",
                output.status,
                String::from_utf8_lossy(&output.stderr).trim()
            ),
        });
    }
    let stderr = String::from_utf8_lossy(&output.stderr);
    parse_loudnorm(&stderr).ok_or_else(|| FfmpegError {
        message: "loudnorm did not print a measurement".to_string(),
    })
}

fn measure_ebur128(input: &Path) -> Result<LoudnessReport, FfmpegError> {
    let mut command = ffmpeg();
    command
        .arg("-i")
        .arg(input)
        .args(["-af", "ebur128=peak=true", "-f", "null", "-"])
        .stdin(Stdio::null())
        .stdout(Stdio::null());
    let output = command.output().map_err(|error| FfmpegError {
        message: format!("ebur128: {error}"),
    })?;
    if !output.status.success() {
        return Err(FfmpegError {
            message: format!(
                "ebur128 exited {}: {}",
                output.status,
                String::from_utf8_lossy(&output.stderr).trim()
            ),
        });
    }
    let stderr = String::from_utf8_lossy(&output.stderr);
    parse_ebur128(&stderr).ok_or_else(|| FfmpegError {
        message: format!("ebur128 summary missing: {}", stderr.trim()),
    })
}

fn parse_loudnorm(stderr: &str) -> Option<MeasuredPass> {
    let start = stderr.rfind('{')?;
    let end = stderr[start..].find('}')? + start;
    let value: Value = serde_json::from_str(&stderr[start..=end]).ok()?;
    Some(MeasuredPass {
        integrated: json_f64(&value, "input_i")?,
        true_peak: json_f64(&value, "input_tp")?,
        lra: json_f64(&value, "input_lra")?,
        threshold: json_f64(&value, "input_thresh")?,
        offset: json_f64(&value, "target_offset")?,
    })
}

fn parse_ebur128(stderr: &str) -> Option<LoudnessReport> {
    let mut integrated = None;
    let mut true_peak = None;
    for line in stderr.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("I:") {
            integrated = first_f64(rest);
        } else if let Some(rest) = trimmed.strip_prefix("Peak:") {
            true_peak = first_f64(rest);
        }
    }
    Some(LoudnessReport {
        integrated_lufs: integrated?,
        true_peak_db: true_peak?,
    })
}

fn json_f64(value: &Value, key: &str) -> Option<f64> {
    let field = value.get(key)?;
    if let Some(number) = field.as_f64() {
        return Some(number);
    }
    field.as_str()?.parse().ok()
}

fn first_f64(text: &str) -> Option<f64> {
    let token = text.split_whitespace().next()?;
    token.parse().ok()
}

fn ffmpeg_has_filter(name: &str) -> bool {
    let Ok(output) = ffmpeg().args(["-filters"]).output() else {
        return false;
    };
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .any(|line| line.split_whitespace().nth(1) == Some(name))
}

pub fn write_sine_wav(
    path: &Path,
    sample_rate: u32,
    seconds: f64,
    amplitude: f32,
) -> std::io::Result<()> {
    let count = (f64::from(sample_rate) * seconds).round() as usize;
    let mut pcm = Vec::with_capacity(count * 2);
    for index in 0..count {
        let t = index as f32 / sample_rate as f32;
        let sample = (t * 440.0 * std::f32::consts::TAU).sin() * amplitude;
        let scaled = (sample.clamp(-1.0, 1.0) * i16::MAX as f32) as i16;
        pcm.extend_from_slice(&scaled.to_le_bytes());
    }
    let mut bytes = Vec::with_capacity(44 + pcm.len());
    bytes.extend_from_slice(b"RIFF");
    bytes.extend_from_slice(&(36 + pcm.len() as u32).to_le_bytes());
    bytes.extend_from_slice(b"WAVEfmt ");
    bytes.extend_from_slice(&16u32.to_le_bytes());
    bytes.extend_from_slice(&1u16.to_le_bytes());
    bytes.extend_from_slice(&1u16.to_le_bytes());
    bytes.extend_from_slice(&sample_rate.to_le_bytes());
    bytes.extend_from_slice(&(sample_rate * 2).to_le_bytes());
    bytes.extend_from_slice(&2u16.to_le_bytes());
    bytes.extend_from_slice(&16u16.to_le_bytes());
    bytes.extend_from_slice(b"data");
    bytes.extend_from_slice(&(pcm.len() as u32).to_le_bytes());
    bytes.extend_from_slice(&pcm);
    std::fs::write(path, bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn loudnorm_fixture_lands_near_minus_fourteen() {
        let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../target/m10-audio-tests")
            .join(std::process::id().to_string());
        std::fs::create_dir_all(&dir).unwrap();
        let source = dir.join("loud.wav");
        let mixed = dir.join("normalized.wav");
        write_sine_wav(&source, 48_000, 3.0, 0.5).unwrap();
        let report = normalize_loudness(&source, &mixed).unwrap();
        assert!(
            core::loudness_ok(report.integrated_lufs, report.true_peak_db),
            "I={} TP={}",
            report.integrated_lufs,
            report.true_peak_db
        );
        let chain = {
            let mut chain = core::AudioChain::default();
            chain.noise.enabled = true;
            chain.noise.amount = 1.0;
            chain
        };
        let graph = core::voice_filters(&chain, noise_reducer());
        assert!(graph.contains("arnndn") || graph.contains("afftdn"));
    }

    #[test]
    fn ebur128_summary_parses() {
        let stderr = "Summary:\n  Integrated loudness:\n    I:         -14.2 LUFS\n  True peak:\n    Peak:      -1.4 dBFS\n";
        let report = parse_ebur128(stderr).unwrap();
        assert!((report.integrated_lufs - -14.2).abs() < 1e-9);
        assert!((report.true_peak_db - -1.4).abs() < 1e-9);
    }
}
