//! Decode a plate with ffmpeg, draw the scene in Skia, and encode the file.

use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Instant;

use core::{Easing, PlacedRect, frame_rects, hash_frame, select_encoder};
use skia_safe::{AlphaType, Color, ColorType, Data, ImageInfo, Paint, Rect, images, surfaces};

use crate::ffmpeg::{FfmpegError, ffmpeg, ffprobe};

#[derive(Debug, Clone)]
pub struct RenderRequest {
    pub width: u32,
    pub height: u32,
    pub fps: u32,
    pub frames: u32,
    pub hevc: bool,
    pub easing: Easing,
    pub motion_blur: bool,
    pub video_bitrate_kbps: u32,
    pub encoder: String,
    pub output: PathBuf,
}

#[derive(Debug, Clone)]
pub struct RenderReport {
    pub output: PathBuf,
    pub encoder: String,
    pub frames: u32,
    pub elapsed_secs: f64,
    pub frame_hash: u64,
}

#[derive(Debug, Clone)]
pub struct Probe {
    pub width: u32,
    pub height: u32,
    pub fps: f64,
    pub video_codec: String,
    pub sample_rate: u32,
    pub duration: f64,
}

pub fn choose_encoder(hevc: bool) -> String {
    let listed = ffmpeg_encoders();
    let refs: Vec<&str> = listed.iter().map(String::as_str).collect();
    select_encoder(hevc, &refs).to_string()
}

pub fn render_movie(
    request: &RenderRequest,
    cancel: &AtomicBool,
    frames_done: &Mutex<u32>,
) -> Result<RenderReport, FfmpegError> {
    let started = Instant::now();
    let mut decoder = spawn_decoder(request)?;
    let mut encoder = spawn_encoder(request)?;
    let mut decoded = vec![0u8; frame_bytes(request)];
    let mut first_hash = 0;
    let decoder_stdout = decoder.stdout.take().ok_or_else(|| FfmpegError {
        message: "decoder has no stdout".to_string(),
    })?;
    let mut decoder_stdout = decoder_stdout;
    let encoder_stdin = encoder.stdin.take().ok_or_else(|| FfmpegError {
        message: "encoder has no stdin".to_string(),
    })?;
    let mut encoder_stdin = encoder_stdin;
    for index in 0..request.frames {
        if cancel.load(Ordering::Relaxed) {
            let _ = decoder.kill();
            drop(encoder_stdin);
            let _ = encoder.kill();
            let _ = decoder.wait();
            let _ = encoder.wait();
            return Err(FfmpegError {
                message: format!("render cancelled after {index} frames"),
            });
        }
        read_frame(&mut decoder_stdout, &mut decoded)?;
        let composited = composite_frame(request, index, &decoded)?;
        if index == 0 {
            first_hash = hash_frame(&composited);
        }
        encoder_stdin
            .write_all(&composited)
            .map_err(|error| FfmpegError {
                message: format!("encoder write: {error}"),
            })?;
        if let Ok(mut done) = frames_done.lock() {
            *done = index + 1;
        }
    }
    drop(encoder_stdin);
    drop(decoder_stdout);
    let _ = decoder.kill();
    let _ = decoder.wait();
    finish_child(encoder, "encoder")?;
    Ok(RenderReport {
        output: request.output.clone(),
        encoder: request.encoder.clone(),
        frames: request.frames,
        elapsed_secs: started.elapsed().as_secs_f64(),
        frame_hash: first_hash,
    })
}

pub fn composite_frame(
    request: &RenderRequest,
    frame: u32,
    decoded: &[u8],
) -> Result<Vec<u8>, FfmpegError> {
    let rects = frame_rects(
        request.width,
        request.height,
        request.frames,
        frame,
        request.easing,
        request.motion_blur,
    );
    if rects.len() <= 1 {
        return draw_skia(request.width, request.height, decoded, &rects);
    }
    let mut layers = Vec::with_capacity(rects.len());
    for rect in &rects {
        layers.push(draw_skia(
            request.width,
            request.height,
            decoded,
            std::slice::from_ref(rect),
        )?);
    }
    Ok(core::average_rgba(&layers))
}

pub fn probe_movie(path: &Path) -> Result<Probe, FfmpegError> {
    let mut command = ffprobe();
    command.args([
        "-v",
        "error",
        "-print_format",
        "json",
        "-show_streams",
        "-show_format",
    ]);
    command.arg(path);
    let output = crate::ffmpeg::run_checked(command)?;
    let value: serde_json::Value =
        serde_json::from_slice(&output.stdout).map_err(|error| FfmpegError {
            message: format!("ffprobe json: {error}"),
        })?;
    let streams = value
        .get("streams")
        .and_then(|item| item.as_array())
        .ok_or_else(|| FfmpegError {
            message: "ffprobe returned no streams".to_string(),
        })?;
    let video = streams
        .iter()
        .find(|stream| stream.get("codec_type").and_then(|item| item.as_str()) == Some("video"))
        .ok_or_else(|| FfmpegError {
            message: "ffprobe returned no video stream".to_string(),
        })?;
    let audio = streams
        .iter()
        .find(|stream| stream.get("codec_type").and_then(|item| item.as_str()) == Some("audio"));
    let duration = value
        .pointer("/format/duration")
        .and_then(|item| item.as_str())
        .and_then(|text| text.parse::<f64>().ok())
        .ok_or_else(|| FfmpegError {
            message: "ffprobe returned no duration".to_string(),
        })?;
    Ok(Probe {
        width: json_u32(video, "width")?,
        height: json_u32(video, "height")?,
        fps: parse_rate(
            video
                .get("avg_frame_rate")
                .and_then(|item| item.as_str())
                .unwrap_or("0/1"),
        ),
        video_codec: video
            .get("codec_name")
            .and_then(|item| item.as_str())
            .unwrap_or("")
            .to_string(),
        sample_rate: audio
            .and_then(|stream| stream.get("sample_rate"))
            .and_then(|item| item.as_str())
            .and_then(|text| text.parse().ok())
            .unwrap_or(0),
        duration,
    })
}

pub fn moov_before_mdat(path: &Path) -> bool {
    let Ok(bytes) = std::fs::read(path) else {
        return false;
    };
    let mut offset = 0usize;
    let mut moov = None;
    let mut mdat = None;
    while offset + 8 <= bytes.len() {
        let mut size = u32::from_be_bytes(bytes[offset..offset + 4].try_into().unwrap()) as u64;
        let kind = &bytes[offset + 4..offset + 8];
        let header = if size == 1 {
            if offset + 16 > bytes.len() {
                break;
            }
            size = u64::from_be_bytes(bytes[offset + 8..offset + 16].try_into().unwrap());
            16
        } else {
            8
        };
        if size < header as u64 {
            break;
        }
        if kind == b"moov" {
            moov = Some(offset);
        } else if kind == b"mdat" {
            mdat = Some(offset);
        }
        offset += size as usize;
    }
    matches!((moov, mdat), (Some(moov_at), Some(mdat_at)) if moov_at < mdat_at)
}

fn draw_skia(
    width: u32,
    height: u32,
    decoded: &[u8],
    rects: &[PlacedRect],
) -> Result<Vec<u8>, FfmpegError> {
    let info = ImageInfo::new(
        (width as i32, height as i32),
        ColorType::RGBA8888,
        AlphaType::Unpremul,
        None,
    );
    let mut surface =
        surfaces::raster_n32_premul((width as i32, height as i32)).ok_or_else(|| FfmpegError {
            message: "skia surface".to_string(),
        })?;
    let canvas = surface.canvas();
    let row_bytes = (width as usize) * 4;
    if decoded.len() >= row_bytes * height as usize {
        let data = Data::new_copy(&decoded[..row_bytes * height as usize]);
        if let Some(image) = images::raster_from_data(&info, data, row_bytes) {
            canvas.draw_image(&image, (0, 0), None);
        }
    } else {
        canvas.clear(Color::from_rgb(32, 32, 32));
    }
    let mut paint = Paint::default();
    paint.set_anti_alias(false);
    paint.set_color(Color::WHITE);
    for rect in rects {
        canvas.draw_rect(
            Rect::from_xywh(
                rect.x as f32,
                rect.y as f32,
                rect.side as f32,
                rect.side as f32,
            ),
            &paint,
        );
    }
    let mut pixels = vec![0u8; row_bytes * height as usize];
    let read = ImageInfo::new(
        (width as i32, height as i32),
        ColorType::RGBA8888,
        AlphaType::Unpremul,
        None,
    );
    if !surface.read_pixels(&read, &mut pixels, row_bytes, (0, 0)) {
        return Err(FfmpegError {
            message: "skia read_pixels".to_string(),
        });
    }
    Ok(pixels)
}

fn spawn_decoder(request: &RenderRequest) -> Result<std::process::Child, FfmpegError> {
    let duration = f64::from(request.frames) / f64::from(request.fps.max(1));
    let color = format!(
        "color=c=0x202020:s={}x{}:r={}:d={duration}",
        request.width, request.height, request.fps
    );
    let mut command = ffmpeg();
    command.args([
        "-loglevel",
        "error",
        "-f",
        "lavfi",
        "-i",
        &color,
        "-f",
        "rawvideo",
        "-pix_fmt",
        "rgba",
        "pipe:1",
    ]);
    command
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| FfmpegError {
            message: format!("decoder spawn: {error}"),
        })
}

fn spawn_encoder(request: &RenderRequest) -> Result<std::process::Child, FfmpegError> {
    let size = format!("{}x{}", request.width, request.height);
    let rate = request.fps.to_string();
    let duration = format!(
        "{}",
        f64::from(request.frames) / f64::from(request.fps.max(1))
    );
    let sine = format!("sine=frequency=440:sample_rate=48000:duration={duration}");
    let bitrate = format!("{}k", request.video_bitrate_kbps);
    let mut command = ffmpeg();
    command.args([
        "-loglevel",
        "error",
        "-f",
        "rawvideo",
        "-pix_fmt",
        "rgba",
        "-s",
        &size,
        "-r",
        &rate,
        "-i",
        "pipe:0",
        "-f",
        "lavfi",
        "-i",
        &sine,
        "-c:v",
        &request.encoder,
        "-b:v",
        &bitrate,
        "-pix_fmt",
        "yuv420p",
        "-c:a",
        "aac",
        "-b:a",
        "384k",
        "-ar",
        "48000",
        "-movflags",
        "+faststart",
        "-shortest",
    ]);
    if request.encoder == "libx264" || request.encoder == "libx265" {
        command.args(["-preset", "veryfast"]);
    }
    command.arg(&request.output);
    command
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| FfmpegError {
            message: format!("encoder spawn: {error}"),
        })
}

fn finish_child(child: std::process::Child, name: &str) -> Result<(), FfmpegError> {
    let output = child.wait_with_output().map_err(|error| FfmpegError {
        message: format!("{name} wait: {error}"),
    })?;
    if output.status.success() {
        Ok(())
    } else {
        Err(FfmpegError {
            message: format!(
                "{name} exited {}: {}",
                output.status,
                String::from_utf8_lossy(&output.stderr).trim()
            ),
        })
    }
}

fn read_frame(reader: &mut impl Read, buffer: &mut [u8]) -> Result<(), FfmpegError> {
    reader.read_exact(buffer).map_err(|error| FfmpegError {
        message: format!("decoder read: {error}"),
    })
}

fn frame_bytes(request: &RenderRequest) -> usize {
    request.width as usize * request.height as usize * 4
}

fn json_u32(value: &serde_json::Value, key: &str) -> Result<u32, FfmpegError> {
    value
        .get(key)
        .and_then(serde_json::Value::as_u64)
        .and_then(|number| u32::try_from(number).ok())
        .ok_or_else(|| FfmpegError {
            message: format!("ffprobe missing {key}"),
        })
}

fn parse_rate(text: &str) -> f64 {
    let mut parts = text.split('/');
    let num = parts
        .next()
        .and_then(|item| item.parse::<f64>().ok())
        .unwrap_or(0.0);
    let den = parts
        .next()
        .and_then(|item| item.parse::<f64>().ok())
        .unwrap_or(1.0);
    if den == 0.0 { 0.0 } else { num / den }
}

fn ffmpeg_encoders() -> Vec<String> {
    let Ok(output) = Command::new("ffmpeg")
        .args(["-hide_banner", "-encoders"])
        .output()
    else {
        return Vec::new();
    };
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter_map(|line| {
            let mut parts = line.split_whitespace();
            let flags = parts.next()?;
            let name = parts.next()?;
            if flags.contains('V') && flags.contains('E') {
                Some(name.to_string())
            } else {
                None
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::AtomicBool;

    fn request(dir: &Path, name: &str, width: u32, height: u32, frames: u32) -> RenderRequest {
        RenderRequest {
            width,
            height,
            fps: 30,
            frames,
            hevc: false,
            easing: Easing::EaseInOut,
            motion_blur: false,
            video_bitrate_kbps: 2_000,
            encoder: "libx264".to_string(),
            output: dir.join(name),
        }
    }

    fn test_dir(name: &str) -> PathBuf {
        let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../target/m11-render-tests")
            .join(name)
            .join(std::process::id().to_string());
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn fixture_matches_ffprobe_and_a_second_render_hashes_the_same() {
        let dir = test_dir("probe");
        let job = request(&dir, "fixture.mp4", 180, 320, 8);
        let cancel = AtomicBool::new(false);
        let done = Mutex::new(0);
        let report = render_movie(&job, &cancel, &done).unwrap();
        let probe = probe_movie(&report.output).unwrap();
        assert_eq!((probe.width, probe.height), (180, 320));
        assert!((probe.fps - 30.0).abs() < 0.1, "{}", probe.fps);
        assert_eq!(probe.video_codec, "h264");
        assert_eq!(probe.sample_rate, 48_000);
        let frame = 1.0 / 30.0;
        assert!(
            (probe.duration - 8.0 / 30.0).abs() <= frame + 0.02,
            "{}",
            probe.duration
        );
        assert!(moov_before_mdat(&report.output));
        let again = request(&dir, "again.mp4", 180, 320, 8);
        let second = render_movie(&again, &cancel, &done).unwrap();
        assert_eq!(report.frame_hash, second.frame_hash);
        let mut plate = vec![0u8; 180 * 320 * 4];
        for pixel in plate.as_chunks_mut::<4>().0 {
            pixel.copy_from_slice(&[32, 32, 32, 255]);
        }
        let drawn = composite_frame(&job, 0, &plate).unwrap();
        let fixture = core::paint_rects(
            180,
            320,
            &core::frame_rects(180, 320, 8, 0, Easing::EaseInOut, false),
        );
        let mut expected = plate.clone();
        for (index, pixel) in fixture.chunks(4).enumerate() {
            if pixel[0] > 0 {
                let offset = index * 4;
                expected[offset..offset + 4].copy_from_slice(pixel);
            }
        }
        let score = core::ssim_rgba(&drawn, &expected);
        assert!(score >= 0.98, "{score}");
        eprintln!(
            "render wall time: {:.3}s for {} frames at {}x{}",
            report.elapsed_secs, report.frames, job.width, job.height
        );
    }

    #[test]
    fn four_k_and_preview_sizes_probe() {
        let dir = test_dir("sizes");
        let cancel = AtomicBool::new(false);
        let done = Mutex::new(0);
        let started = Instant::now();
        let four_k = request(&dir, "fourk.mp4", 2160, 3840, 4);
        let report = render_movie(&four_k, &cancel, &done).unwrap();
        let probe = probe_movie(&report.output).unwrap();
        assert_eq!((probe.width, probe.height), (2160, 3840));
        let preview = request(&dir, "preview.mp4", 540, 960, 4);
        render_movie(&preview, &cancel, &done).unwrap();
        eprintln!(
            "4k wall time: {:.3}s for 4 frames at 2160x3840",
            started.elapsed().as_secs_f64()
        );
    }

    #[test]
    fn cancel_stops_before_the_file_finishes() {
        let dir = test_dir("cancel");
        let job = request(&dir, "cancel.mp4", 180, 320, 30);
        let cancel = AtomicBool::new(true);
        let done = Mutex::new(0);
        let error = render_movie(&job, &cancel, &done).unwrap_err();
        assert!(error.message.contains("cancelled"));
        assert_eq!(*done.lock().unwrap(), 0);
    }

    #[test]
    fn skia_blur_softens_an_edge_the_sharp_frame_keeps() {
        let mut job = request(&PathBuf::from("."), "unused.mp4", 48, 48, 8);
        let plate = vec![0u8; 48 * 48 * 4];
        job.motion_blur = false;
        let sharp = composite_frame(&job, 3, &plate).unwrap();
        job.motion_blur = true;
        let blurred = composite_frame(&job, 3, &plate).unwrap();
        assert!(
            sharp
                .chunks(4)
                .all(|pixel| pixel[0] == 0 || pixel[0] == 255)
        );
        assert!(
            blurred
                .chunks(4)
                .any(|pixel| pixel[0] > 0 && pixel[0] < 255)
        );
    }
}
