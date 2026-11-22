//! Render queue for a short fixture export. The full movie is `media::render`.

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Instant;

use core::{RenderProgress, eta_secs};
use media::render::{RenderRequest, choose_encoder, render_movie};
use serde::Serialize;
use tauri::{AppHandle, Manager, State};

pub struct RenderState {
    cancel: Arc<AtomicBool>,
    frames_done: Arc<Mutex<u32>>,
    started: Arc<Mutex<Option<Instant>>>,
    status: Arc<Mutex<RenderStatus>>,
}

impl Default for RenderState {
    fn default() -> Self {
        Self {
            cancel: Arc::new(AtomicBool::new(false)),
            frames_done: Arc::new(Mutex::new(0)),
            started: Arc::new(Mutex::new(None)),
            status: Arc::new(Mutex::new(RenderStatus::idle())),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderStatus {
    pub phase: String,
    pub frames_done: u32,
    pub frames_total: u32,
    pub eta_secs: f64,
    pub output: Option<String>,
    pub encoder: Option<String>,
    pub message: Option<String>,
}

impl RenderStatus {
    fn idle() -> Self {
        Self {
            phase: "idle".to_string(),
            frames_done: 0,
            frames_total: 0,
            eta_secs: 0.0,
            output: None,
            encoder: None,
            message: None,
        }
    }
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StartRenderArgs {
    pub width: u32,
    pub height: u32,
    pub fps: u32,
    #[serde(default)]
    pub project_dir: Option<String>,
    #[serde(default)]
    pub cover: Option<bool>,
    #[serde(default)]
    pub captions: Option<bool>,
}

#[tauri::command(async)]
pub fn start_render(
    app: AppHandle,
    state: State<'_, RenderState>,
    args: StartRenderArgs,
) -> Result<(), String> {
    {
        let status = state.status.lock().map_err(|error| error.to_string())?;
        if status.phase == "rendering" {
            return Err("a render is already running".to_string());
        }
    }
    let width = args.width;
    let height = args.height;
    let fps = args.fps;
    let project_dir = args.project_dir.filter(|dir| !dir.is_empty());
    let output = match &project_dir {
        Some(dir) => PathBuf::from(dir).join("renders/short.mp4"),
        None => render_output(&app)?,
    };
    let frames = if project_dir.is_some() {
        ((project_duration(project_dir.as_deref()) * f64::from(fps.max(1))).ceil() as u32).max(1)
    } else if width >= 1440 {
        4
    } else {
        8
    };
    state.cancel.store(false, Ordering::Relaxed);
    if let Ok(mut done) = state.frames_done.lock() {
        *done = 0;
    }
    if let Ok(mut started) = state.started.lock() {
        *started = Some(Instant::now());
    }
    if let Ok(mut status) = state.status.lock() {
        *status = RenderStatus {
            phase: "rendering".to_string(),
            frames_done: 0,
            frames_total: frames,
            eta_secs: 0.0,
            output: Some(output.display().to_string()),
            encoder: None,
            message: None,
        };
    }
    let cancel = Arc::clone(&state.cancel);
    let frames_done = Arc::clone(&state.frames_done);
    let started = Arc::clone(&state.started);
    let status = Arc::clone(&state.status);
    let request = ProjectExport {
        dir: project_dir.clone().unwrap_or_default(),
        width,
        height,
        fps,
        output: output.clone(),
        cover: args.cover.unwrap_or(true),
        captions: args.captions.unwrap_or(true),
    };
    thread::spawn(move || {
        if project_dir.is_some() {
            finish_project_export(&request, &cancel, &status, &started);
            return;
        }
        let encoder = choose_encoder(false);
        let request = RenderRequest {
            width,
            height,
            fps: fps.clamp(1, 60),
            frames,
            hevc: false,
            easing: core::Easing::EaseInOut,
            motion_blur: false,
            video_bitrate_kbps: core::video_bitrate_kbps(width),
            encoder: encoder.clone(),
            output: output.clone(),
        };
        let rendered = render_movie(&request, &cancel, &frames_done);
        let rendered = match rendered {
            Err(error) if encoder != "libx264" && !error.message.contains("cancelled") => {
                let mut fallback = request;
                fallback.encoder = "libx264".to_string();
                render_movie(&fallback, &cancel, &frames_done)
            }
            other => other,
        };
        if let Ok(mut status) = status.lock() {
            match rendered {
                Ok(report) => {
                    status.phase = "done".to_string();
                    status.frames_done = report.frames;
                    status.frames_total = report.frames;
                    status.encoder = Some(report.encoder);
                    status.output = Some(report.output.display().to_string());
                    status.eta_secs = 0.0;
                    status.message = Some(format!("finished in {:.1}s", report.elapsed_secs));
                }
                Err(error) => {
                    status.phase = if error.message.contains("cancelled") {
                        "cancelled".to_string()
                    } else {
                        "error".to_string()
                    };
                    status.message = Some(error.message);
                }
            }
        }
        if let Ok(mut started) = started.lock() {
            *started = None;
        }
    });
    Ok(())
}

#[tauri::command]
pub fn cancel_render(state: State<'_, RenderState>) {
    state.cancel.store(true, Ordering::Relaxed);
}

#[tauri::command]
pub fn render_status(state: State<'_, RenderState>) -> RenderStatus {
    let mut status = state
        .status
        .lock()
        .map(|guard| guard.clone())
        .unwrap_or_else(|_| RenderStatus::idle());
    if status.phase == "rendering" {
        let done = state
            .frames_done
            .lock()
            .map(|guard| *guard)
            .unwrap_or(status.frames_done);
        let elapsed = state
            .started
            .lock()
            .ok()
            .and_then(|guard| *guard)
            .map(|start| start.elapsed().as_secs_f64())
            .unwrap_or(0.0);
        status.frames_done = done;
        status.eta_secs = eta_secs(RenderProgress {
            frames_done: done,
            frames_total: status.frames_total,
            elapsed_secs: elapsed,
        });
    }
    status
}

fn project_duration(dir: Option<&str>) -> f64 {
    let Some(dir) = dir else {
        return 1.0;
    };
    store::load_project(std::path::Path::new(dir))
        .ok()
        .and_then(|project| {
            project
                .tracks
                .iter()
                .find(|track| track.id == core::TrackId::V1Screen)
                .and_then(|track| track.clips.last())
                .map(|clip| clip.timeline_start + clip.duration)
        })
        .unwrap_or(1.0)
}

struct ProjectExport {
    dir: String,
    width: u32,
    height: u32,
    fps: u32,
    output: PathBuf,
    cover: bool,
    captions: bool,
}

fn finish_project_export(
    request: &ProjectExport,
    cancel: &AtomicBool,
    status: &Mutex<RenderStatus>,
    started: &Mutex<Option<Instant>>,
) {
    let result = export_project(request, cancel);
    if let Ok(mut status) = status.lock() {
        match result {
            Ok(()) => {
                status.phase = "done".to_string();
                status.frames_done = status.frames_total;
                status.encoder = Some("libx264".to_string());
                status.output = Some(request.output.display().to_string());
                status.eta_secs = 0.0;
                status.message = Some("exported the take".to_string());
            }
            Err(message) => {
                status.phase = if message.contains("cancelled") {
                    "cancelled".to_string()
                } else {
                    "error".to_string()
                };
                status.message = Some(message);
            }
        }
    }
    if let Ok(mut started) = started.lock() {
        *started = None;
    }
}

fn export_project(request: &ProjectExport, cancel: &AtomicBool) -> Result<(), String> {
    let project = store::load_project(std::path::Path::new(&request.dir))
        .map_err(|error| error.to_string())?;
    let job = take_job(
        &project,
        request.width,
        request.height,
        request.fps,
        &request.output,
    )?;
    media::export_take::export_take(&job, cancel).map_err(|error| error.message)?;
    let renders = std::path::Path::new(&request.dir).join("renders");
    let _ = std::fs::create_dir_all(&renders);
    if request.cover {
        let hook = project
            .ai
            .beats
            .first()
            .map(|beat| beat.text.clone())
            .filter(|text| !text.is_empty())
            .unwrap_or_else(|| project.name.clone());
        let _ = media::cover::write_cover_png(
            &renders.join("cover.png"),
            project.assets.cover_template,
            &hook,
            None,
        );
    }
    if request.captions {
        let words = caption_words(&project);
        let _ = std::fs::write(renders.join("captions.srt"), core::captions_srt(&words));
    }
    let hook = project
        .ai
        .beats
        .first()
        .map(|beat| beat.text.clone())
        .unwrap_or_else(|| project.name.clone());
    let keyword = if project.ai.topic.is_empty() {
        project.slug.clone()
    } else {
        project.ai.topic.clone()
    };
    let kit = core::build_kit(&keyword, &hook, "", "");
    let _ = std::fs::write(renders.join("upload_kit.txt"), core::plain_text(&kit));
    Ok(())
}

fn caption_words(project: &core::Project) -> Vec<core::WordStamp> {
    project
        .tracks
        .iter()
        .find(|track| track.id == core::TrackId::Txt)
        .and_then(|track| track.clips.first())
        .and_then(|clip| match &clip.body {
            core::ClipBody::Caption { words, .. } => Some(words.clone()),
            _ => None,
        })
        .unwrap_or_default()
}

fn take_job(
    project: &core::Project,
    width: u32,
    height: u32,
    fps: u32,
    output: &std::path::Path,
) -> Result<media::export_take::TakeExport, String> {
    let screen = clips_of(project, core::TrackId::V1Screen);
    if screen.is_empty() {
        return Err("this take has no screen clip".to_string());
    }
    let cam = clips_of(project, core::TrackId::V2Cam);
    let mic = clips_of(project, core::TrackId::A1Voice);
    let voice = track_heard(project, core::TrackId::A1Voice);
    let camera = track_heard(project, core::TrackId::V2Cam);
    let rect = core::cam_rect(
        f64::from(width),
        f64::from(height),
        project.layout.cam_position,
        project.layout.cam_size,
        project.layout.safe_zone,
    );
    let mut segments = Vec::new();
    for (source, start, end, speed, timeline, duration) in screen {
        let cam_window = if camera {
            window_at(&cam, timeline, duration)
        } else {
            None
        };
        let mic_window = if voice {
            window_at(&mic, timeline, duration)
        } else {
            None
        };
        segments.push(media::export_take::ExportSegment {
            screen: source,
            cam: cam_window.map(|(path, _, _)| path),
            mic: mic_window.map(|(path, _, _)| path),
            start,
            end,
            speed,
            cam_x: rect.x.round() as i32,
            cam_y: rect.y.round() as i32,
            cam_side: rect.width.round().max(2.0) as u32,
        });
    }
    let end_card_secs = if project.assets.add_end_card {
        1.0
    } else {
        0.0
    };
    Ok(media::export_take::TakeExport {
        segments,
        width,
        height,
        fps: fps.clamp(1, 60),
        output: output.to_path_buf(),
        end_card_secs,
    })
}

type ClipWindow = (std::path::PathBuf, f64, f64, f64, f64, f64);

fn clips_of(project: &core::Project, id: core::TrackId) -> Vec<ClipWindow> {
    let Some(track) = project.tracks.iter().find(|track| track.id == id) else {
        return Vec::new();
    };
    track
        .clips
        .iter()
        .filter_map(|clip| match &clip.body {
            core::ClipBody::Video {
                source,
                source_in,
                source_out,
                speed,
            }
            | core::ClipBody::Audio {
                source,
                source_in,
                source_out,
                speed,
            } => Some((
                std::path::PathBuf::from(source),
                *source_in,
                *source_out,
                *speed,
                clip.timeline_start,
                clip.duration,
            )),
            _ => None,
        })
        .collect()
}

fn window_at(
    clips: &[ClipWindow],
    timeline: f64,
    duration: f64,
) -> Option<(std::path::PathBuf, f64, f64)> {
    let clip = clips.iter().find(|clip| {
        let end = clip.4 + clip.5;
        timeline < end && clip.4 < timeline + duration
    })?;
    let speed = clip.3.max(0.25);
    let into = (timeline - clip.4).max(0.0) * speed;
    let start = clip.1 + into;
    let end = (start + duration * speed).min(clip.2);
    Some((clip.0.clone(), start, end.max(start + 0.04)))
}

fn track_heard(project: &core::Project, id: core::TrackId) -> bool {
    let Some(track) = project.tracks.iter().find(|track| track.id == id) else {
        return false;
    };
    if track.muted {
        return false;
    }
    let solo = project.tracks.iter().any(|item| item.solo);
    !solo || track.solo
}

fn render_output(app: &AppHandle) -> Result<PathBuf, String> {
    let dir = app
        .path()
        .app_data_dir()
        .map_err(|error| error.to_string())?
        .join("renders");
    std::fs::create_dir_all(&dir).map_err(|error| error.to_string())?;
    Ok(dir.join("fixture.mp4"))
}
