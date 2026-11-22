//! Encode a trimmed take. The picture is the screen file, with the camera on top.

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;

use crate::ffmpeg::{FfmpegError, ffmpeg};

#[derive(Debug, Clone)]
pub struct ExportSegment {
    pub screen: PathBuf,
    pub cam: Option<PathBuf>,
    pub mic: Option<PathBuf>,
    pub start: f64,
    pub end: f64,
    pub speed: f64,
    pub cam_x: i32,
    pub cam_y: i32,
    pub cam_side: u32,
}

#[derive(Debug, Clone)]
pub struct TakeExport {
    pub segments: Vec<ExportSegment>,
    pub width: u32,
    pub height: u32,
    pub fps: u32,
    pub output: PathBuf,
    pub end_card_secs: f64,
}

pub fn export_take(job: &TakeExport, cancel: &AtomicBool) -> Result<(), FfmpegError> {
    if job.segments.is_empty() {
        return Err(FfmpegError {
            message: "the project has no screen clip".to_string(),
        });
    }
    if let Some(parent) = job.output.parent() {
        std::fs::create_dir_all(parent).map_err(|error| FfmpegError {
            message: format!("export dir: {error}"),
        })?;
    }
    if job.segments.len() == 1 && job.end_card_secs <= 0.0 {
        return encode_segment(&job.segments[0], job, &job.output, cancel);
    }
    let scratch = job
        .output
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join(format!("export-parts-{}", std::process::id()));
    std::fs::create_dir_all(&scratch).map_err(|error| FfmpegError {
        message: format!("export scratch: {error}"),
    })?;
    let mut parts = Vec::new();
    for (index, segment) in job.segments.iter().enumerate() {
        let part = scratch.join(format!("part-{index}.mp4"));
        encode_segment(segment, job, &part, cancel)?;
        parts.push(part);
    }
    if job.end_card_secs > 0.0 {
        let card = scratch.join("end-card.mp4");
        encode_card(job, &card)?;
        parts.push(card);
    }
    concat_parts(&parts, &job.output)?;
    let _ = std::fs::remove_dir_all(&scratch);
    Ok(())
}

fn encode_segment(
    segment: &ExportSegment,
    job: &TakeExport,
    output: &Path,
    cancel: &AtomicBool,
) -> Result<(), FfmpegError> {
    let duration = (segment.end - segment.start).max(0.04);
    let speed = segment.speed.clamp(0.25, 4.0);
    let mut command = ffmpeg();
    command.args(["-loglevel", "error"]);
    command
        .arg("-ss")
        .arg(format!("{:.3}", segment.start))
        .arg("-t")
        .arg(format!("{duration:.3}"))
        .arg("-i")
        .arg(&segment.screen);
    let mut next = 1;
    let cam_index = segment.cam.as_ref().map(|cam| {
        command
            .arg("-ss")
            .arg(format!("{:.3}", segment.start))
            .arg("-t")
            .arg(format!("{duration:.3}"))
            .arg("-i")
            .arg(cam);
        let index = next;
        next += 1;
        index
    });
    let mic_index = segment.mic.as_ref().map(|mic| {
        command
            .arg("-ss")
            .arg(format!("{:.3}", segment.start))
            .arg("-t")
            .arg(format!("{duration:.3}"))
            .arg("-i")
            .arg(mic);
        next
    });
    let filter = video_filter(job, segment, cam_index, speed);
    command.args(["-filter_complex", &filter, "-map", "[v]"]);
    if let Some(index) = mic_index {
        let audio_map = format!("{index}:a:0");
        command.args(["-map", &audio_map]);
        let tempo = atempo_chain(speed);
        command.args([
            "-af",
            &format!("loudnorm=I=-14:TP=-1.0:LRA=11,{tempo}"),
            "-c:a",
            "aac",
            "-b:a",
            "192k",
            "-ar",
            "48000",
        ]);
    }
    command.args([
        "-c:v",
        "libx264",
        "-preset",
        "veryfast",
        "-pix_fmt",
        "yuv420p",
        "-r",
        &job.fps.to_string(),
        "-movflags",
        "+faststart",
    ]);
    command.arg(output);
    run_cancellable(command, cancel)
}

fn video_filter(
    job: &TakeExport,
    segment: &ExportSegment,
    cam_index: Option<u32>,
    speed: f64,
) -> String {
    let fitted = format!(
        "[0:v]scale={}:{}:force_original_aspect_ratio=increase,crop={}:{},setsar=1,setpts={:.4}*PTS[bg]",
        job.width,
        job.height,
        job.width,
        job.height,
        1.0 / speed
    );
    let Some(index) = cam_index else {
        return format!("{fitted};[bg]format=yuv420p[v]");
    };
    let side = segment.cam_side.max(2);
    format!(
        "{fitted};[{index}:v]scale={side}:{side}:force_original_aspect_ratio=increase,crop={side}:{side},setsar=1,setpts={:.4}*PTS[fg];[bg][fg]overlay={}:{}:format=auto[v]",
        1.0 / speed,
        segment.cam_x.max(0),
        segment.cam_y.max(0)
    )
}

fn atempo_chain(speed: f64) -> String {
    let mut filters = Vec::new();
    let mut left = speed;
    while left > 2.0 {
        filters.push("atempo=2.0".to_string());
        left /= 2.0;
    }
    while left < 0.5 {
        filters.push("atempo=0.5".to_string());
        left /= 0.5;
    }
    filters.push(format!("atempo={left:.4}"));
    filters.join(",")
}

fn encode_card(job: &TakeExport, output: &Path) -> Result<(), FfmpegError> {
    let mut command = ffmpeg();
    command.args([
        "-loglevel",
        "error",
        "-f",
        "lavfi",
        "-i",
        &format!(
            "color=c=0x12141a:s={}x{}:r={}:d={:.3}",
            job.width, job.height, job.fps, job.end_card_secs
        ),
        "-f",
        "lavfi",
        "-i",
        &format!("anullsrc=r=48000:cl=mono:d={:.3}", job.end_card_secs),
        "-c:v",
        "libx264",
        "-preset",
        "veryfast",
        "-pix_fmt",
        "yuv420p",
        "-c:a",
        "aac",
        "-shortest",
        "-movflags",
        "+faststart",
    ]);
    command.arg(output);
    crate::ffmpeg::run_checked(command).map(|_| ())
}

fn concat_parts(parts: &[PathBuf], output: &Path) -> Result<(), FfmpegError> {
    let list = output.with_extension("concat.txt");
    let body = parts
        .iter()
        .map(|path| {
            format!(
                "file '{}'\n",
                path.display().to_string().replace('\'', "'\\''")
            )
        })
        .collect::<String>();
    std::fs::write(&list, body).map_err(|error| FfmpegError {
        message: format!("concat list: {error}"),
    })?;
    let mut command = ffmpeg();
    command.args(["-loglevel", "error", "-f", "concat", "-safe", "0", "-i"]);
    command.arg(&list);
    command.args([
        "-c:v",
        "libx264",
        "-preset",
        "veryfast",
        "-pix_fmt",
        "yuv420p",
        "-c:a",
        "aac",
        "-movflags",
        "+faststart",
    ]);
    command.arg(output);
    crate::ffmpeg::run_checked(command).map(|_| ())
}

fn run_cancellable(mut command: Command, cancel: &AtomicBool) -> Result<(), FfmpegError> {
    command
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped());
    let mut child = command.spawn().map_err(|error| FfmpegError {
        message: format!("export spawn: {error}"),
    })?;
    let stderr = child.stderr.take();
    let reader = thread::spawn(move || {
        let mut text = String::new();
        if let Some(mut pipe) = stderr {
            let _ = std::io::Read::read_to_string(&mut pipe, &mut text);
        }
        text
    });
    let status = loop {
        if cancel.load(Ordering::Relaxed) {
            let _ = child.kill();
            let _ = child.wait();
            let _ = reader.join();
            return Err(FfmpegError {
                message: "render cancelled".to_string(),
            });
        }
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) => thread::sleep(Duration::from_millis(40)),
            Err(error) => {
                return Err(FfmpegError {
                    message: format!("export wait: {error}"),
                });
            }
        }
    };
    let stderr = reader.join().unwrap_or_default();
    if status.success() {
        Ok(())
    } else {
        Err(FfmpegError {
            message: format!("export exited {status}: {}", stderr.trim()),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_trimmed_take_exports_1080_with_audio() {
        let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../target/export-take-tests")
            .join(std::process::id().to_string());
        std::fs::create_dir_all(&dir).unwrap();
        let screen = dir.join("screen.mov");
        let mic = dir.join("mic.wav");
        assert!(
            std::process::Command::new("ffmpeg")
                .args([
                    "-hide_banner",
                    "-y",
                    "-loglevel",
                    "error",
                    "-f",
                    "lavfi",
                    "-i",
                    "color=c=0x224466:s=640x360:d=1",
                    "-c:v",
                    "libx264",
                    "-pix_fmt",
                    "yuv420p",
                ])
                .arg(&screen)
                .status()
                .unwrap()
                .success()
        );
        assert!(
            std::process::Command::new("ffmpeg")
                .args([
                    "-hide_banner",
                    "-y",
                    "-loglevel",
                    "error",
                    "-f",
                    "lavfi",
                    "-i",
                    "sine=frequency=440:duration=1",
                    "-c:a",
                    "pcm_s16le",
                ])
                .arg(&mic)
                .status()
                .unwrap()
                .success()
        );
        let output = dir.join("short.mp4");
        export_take(
            &TakeExport {
                segments: vec![ExportSegment {
                    screen,
                    cam: None,
                    mic: Some(mic),
                    start: 0.0,
                    end: 1.0,
                    speed: 1.0,
                    cam_x: 0,
                    cam_y: 0,
                    cam_side: 200,
                }],
                width: 1080,
                height: 1920,
                fps: 30,
                output: output.clone(),
                end_card_secs: 0.0,
            },
            &AtomicBool::new(false),
        )
        .unwrap();
        let probe = crate::render::probe_movie(&output).unwrap();
        assert_eq!((probe.width, probe.height), (1080, 1920));
        assert_eq!(probe.video_codec, "h264");
        assert!(probe.sample_rate > 0);
        assert!((probe.duration - 1.0).abs() < 1.0 / 30.0 + 0.05);
    }
}
