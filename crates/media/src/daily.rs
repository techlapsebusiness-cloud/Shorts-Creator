//! Fixture daily run: mock script, cover, a short 4K file, SRT, and the kit.

use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::sync::atomic::AtomicBool;
use std::time::Instant;

use core::{CoverTemplate, WordStamp, build_kit, captions_srt, fixture_script, plain_text};

use crate::cover::write_cover_png;
use crate::ffmpeg::FfmpegError;
use crate::render::{RenderRequest, render_movie};

pub struct DailyFiles {
    pub video: PathBuf,
    pub cover: PathBuf,
    pub srt: PathBuf,
    pub kit: PathBuf,
    pub elapsed_secs: f64,
}

pub fn run_fixture(dir: &Path) -> Result<DailyFiles, FfmpegError> {
    let started = Instant::now();
    std::fs::create_dir_all(dir).map_err(|error| FfmpegError {
        message: format!("daily dir: {error}"),
    })?;
    let draft = fixture_script("rename", "english").map_err(|error| FfmpegError {
        message: error.message,
    })?;
    let hook = draft
        .beats
        .first()
        .map(|beat| beat.text.clone())
        .unwrap_or_else(|| "This file survives".to_string());
    let words = [
        WordStamp {
            text: "This".into(),
            start: 0.0,
            end: 0.3,
        },
        WordStamp {
            text: "file".into(),
            start: 0.3,
            end: 0.6,
        },
        WordStamp {
            text: "survives".into(),
            start: 0.6,
            end: 1.0,
        },
    ];
    let srt_path = dir.join("captions.srt");
    std::fs::write(&srt_path, captions_srt(&words)).map_err(|error| FfmpegError {
        message: format!("srt: {error}"),
    })?;
    let cover = dir.join("cover.png");
    write_cover_png(&cover, CoverTemplate::HookFace, &hook, None)?;
    let video = dir.join("short.mp4");
    let request = RenderRequest {
        width: 2160,
        height: 3840,
        fps: 30,
        frames: 4,
        hevc: false,
        easing: core::Easing::Linear,
        motion_blur: false,
        video_bitrate_kbps: 8_000,
        encoder: "libx264".to_string(),
        output: video.clone(),
    };
    render_movie(&request, &AtomicBool::new(false), &Mutex::new(0))?;
    let kit = build_kit("rename", &hook, "https://example.com/long", "exposure");
    let kit_path = dir.join("upload_kit.txt");
    std::fs::write(&kit_path, plain_text(&kit)).map_err(|error| FfmpegError {
        message: format!("kit: {error}"),
    })?;
    Ok(DailyFiles {
        video,
        cover,
        srt: srt_path,
        kit: kit_path,
        elapsed_secs: started.elapsed().as_secs_f64(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::render::probe_movie;

    #[test]
    fn a_fixture_day_writes_4k_cover_srt_and_kit() {
        let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../target/m15-daily-tests")
            .join(std::process::id().to_string());
        let files = run_fixture(&dir).unwrap();
        let probe = probe_movie(&files.video).unwrap();
        assert_eq!((probe.width, probe.height), (2160, 3840));
        assert_eq!(probe.video_codec, "h264");
        let cover = std::fs::read(&files.cover).unwrap();
        assert_eq!(crate::cover::png_size(&cover), Some((1080, 1920)));
        let srt = std::fs::read_to_string(&files.srt).unwrap();
        assert!(srt.contains("survives"));
        let kit = std::fs::read_to_string(&files.kit).unwrap();
        assert!(kit.contains("#Shorts"));
        assert!(!kit.to_lowercase().contains("#viral"));
        eprintln!("daily fixture wall time: {:.3}s", files.elapsed_secs);
    }
}
