//! Take folders on disk, and the project each one opens in the editor.

use std::path::{Path, PathBuf};

use core::{Clip, ClipBody, Project, TakeSync, TrackId, WordStamp};
use serde::Serialize;
use store::{default_movies_root, load_project, save_project};
use tauri::{AppHandle, Manager};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TakeSummary {
    pub id: String,
    pub dir: String,
    pub created_ms: u64,
    pub duration_secs: f64,
    pub screen: bool,
    pub cam: bool,
    pub mic: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenedTake {
    pub project: Project,
    pub dir: String,
}

pub fn scan_takes(root: &Path) -> Vec<TakeSummary> {
    let mut takes = Vec::new();
    let Ok(entries) = std::fs::read_dir(root) else {
        return takes;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let Some(id) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        if !path.join("screen.mov").is_file()
            && !path.join("cam.mov").is_file()
            && !path.join("mic.wav").is_file()
        {
            continue;
        }
        let created_ms = id.parse::<u64>().unwrap_or_else(|_| modified_ms(&path));
        takes.push(TakeSummary {
            id: id.to_string(),
            dir: path.display().to_string(),
            created_ms,
            duration_secs: take_duration(&path),
            screen: path.join("screen.mov").is_file(),
            cam: path.join("cam.mov").is_file(),
            mic: path.join("mic.wav").is_file(),
        });
    }
    takes.sort_by_key(|take| std::cmp::Reverse(take.created_ms));
    takes
}

pub fn load_or_create_take(take_dir: &Path, movies_root: &Path) -> Result<OpenedTake, String> {
    let id = take_dir
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| "take folder has no name".to_string())?;
    let project_dir = movies_root.join(format!("{id}-take"));
    if project_dir.join("project.json").is_file() {
        let project = load_project(&project_dir).map_err(|error| error.to_string())?;
        return Ok(OpenedTake {
            project,
            dir: project_dir.display().to_string(),
        });
    }
    std::fs::create_dir_all(project_dir.join("renders")).map_err(|error| error.to_string())?;
    std::fs::create_dir_all(project_dir.join("cache")).map_err(|error| error.to_string())?;
    let name = format!("Take {id}");
    let slug = core::slugify(&name).unwrap_or_else(|_| format!("{id}-take"));
    let mut project = Project::new(id, 1, name, slug);
    project.sync = read_sync(take_dir);
    let duration = take_duration(take_dir).max(0.1);
    if let Some(track) = track_mut(&mut project, TrackId::V1Screen)
        && take_dir.join("screen.mov").is_file()
    {
        track.clips.push(video_clip(
            "clip-screen",
            take_dir.join("screen.mov"),
            duration,
        ));
    }
    if let Some(track) = track_mut(&mut project, TrackId::V2Cam)
        && take_dir.join("cam.mov").is_file()
    {
        track
            .clips
            .push(video_clip("clip-cam", take_dir.join("cam.mov"), duration));
    }
    if let Some(track) = track_mut(&mut project, TrackId::A1Voice)
        && take_dir.join("mic.wav").is_file()
    {
        track
            .clips
            .push(audio_clip("clip-mic", take_dir.join("mic.wav"), duration));
    }
    if let Some(words) = read_words(take_dir)
        && let Some(track) = track_mut(&mut project, TrackId::Txt)
    {
        let text = words
            .iter()
            .map(|word| word.text.as_str())
            .collect::<Vec<_>>()
            .join(" ");
        track.clips.push(Clip {
            id: "clip-captions".to_string(),
            timeline_start: 0.0,
            duration,
            body: ClipBody::Caption { text, words },
            keyframes: Vec::new(),
        });
    }
    save_project(&project_dir, &project).map_err(|error| error.to_string())?;
    Ok(OpenedTake {
        project,
        dir: project_dir.display().to_string(),
    })
}

pub fn movies_root() -> Result<PathBuf, String> {
    default_movies_root().map_err(|error| error.to_string())
}

fn track_mut(project: &mut Project, id: TrackId) -> Option<&mut core::Track> {
    project.tracks.iter_mut().find(|track| track.id == id)
}

fn video_clip(id: &str, source: PathBuf, duration: f64) -> Clip {
    Clip {
        id: id.to_string(),
        timeline_start: 0.0,
        duration,
        body: ClipBody::Video {
            source: source.display().to_string(),
            source_in: 0.0,
            source_out: duration,
            speed: 1.0,
        },
        keyframes: Vec::new(),
    }
}

fn audio_clip(id: &str, source: PathBuf, duration: f64) -> Clip {
    Clip {
        id: id.to_string(),
        timeline_start: 0.0,
        duration,
        body: ClipBody::Audio {
            source: source.display().to_string(),
            source_in: 0.0,
            source_out: duration,
            speed: 1.0,
        },
        keyframes: Vec::new(),
    }
}

fn take_duration(dir: &Path) -> f64 {
    for name in ["screen.mov", "cam.mov", "mic.wav"] {
        let path = dir.join(name);
        if path.is_file()
            && let Ok(seconds) = media::ffmpeg::probe_duration_secs(&path)
            && seconds.is_finite()
            && seconds > 0.0
        {
            return seconds;
        }
    }
    0.0
}

fn read_sync(dir: &Path) -> TakeSync {
    let Ok(text) = std::fs::read_to_string(dir.join("sync.json")) else {
        return TakeSync::default();
    };
    serde_json::from_str(&text).unwrap_or_default()
}

fn read_words(dir: &Path) -> Option<Vec<WordStamp>> {
    let text = std::fs::read_to_string(dir.join("transcript.json")).ok()?;
    let report: core::TranscriptReport = serde_json::from_str(&text).ok()?;
    if report.words.is_empty() {
        None
    } else {
        Some(report.words)
    }
}

fn modified_ms(path: &Path) -> u64 {
    path.metadata()
        .and_then(|meta| meta.modified())
        .ok()
        .and_then(|time| time.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|duration| duration.as_millis() as u64)
        .unwrap_or(0)
}

pub fn write_sync(dir: &Path, sync: &TakeSync) {
    if let Ok(bytes) = serde_json::to_vec(sync) {
        let _ = std::fs::write(dir.join("sync.json"), bytes);
    }
}

fn takes_root(app: &AppHandle) -> Result<PathBuf, String> {
    Ok(app
        .path()
        .app_data_dir()
        .map_err(|error| error.to_string())?
        .join("takes"))
}

#[tauri::command]
pub fn list_takes(app: AppHandle) -> Result<Vec<TakeSummary>, String> {
    Ok(scan_takes(&takes_root(&app)?))
}

#[tauri::command]
pub fn open_take(take_dir: String) -> Result<OpenedTake, String> {
    load_or_create_take(Path::new(&take_dir), &movies_root()?)
}

#[tauri::command]
pub fn save_opened_project(dir: String, project: Project) -> Result<(), String> {
    save_project(Path::new(&dir), &project).map_err(|error| error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ffmpeg_sample(dir: &Path) {
        let screen = dir.join("screen.mov");
        let status = std::process::Command::new("ffmpeg")
            .args([
                "-hide_banner",
                "-y",
                "-loglevel",
                "error",
                "-f",
                "lavfi",
                "-i",
                "color=c=0x3355aa:s=320x180:d=1",
                "-f",
                "lavfi",
                "-i",
                "sine=frequency=440:duration=1",
                "-shortest",
                "-c:v",
                "libx264",
                "-pix_fmt",
                "yuv420p",
                "-c:a",
                "aac",
            ])
            .arg(&screen)
            .status()
            .expect("ffmpeg");
        assert!(status.success());
        let mic = dir.join("mic.wav");
        let status = std::process::Command::new("ffmpeg")
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
            .expect("ffmpeg wav");
        assert!(status.success());
    }

    #[test]
    fn a_take_opens_as_clips_and_keeps_later_edits() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../target/take-open-tests")
            .join(std::process::id().to_string());
        let take = root.join("takes").join("1000");
        std::fs::create_dir_all(&take).unwrap();
        ffmpeg_sample(&take);
        let listed = scan_takes(&root.join("takes"));
        assert_eq!(listed.len(), 1);
        assert!(listed[0].screen && listed[0].mic);
        let opened = load_or_create_take(&take, &root.join("movies")).unwrap();
        let screen = opened
            .project
            .tracks
            .iter()
            .find(|track| track.id == TrackId::V1Screen)
            .unwrap();
        let ClipBody::Video {
            source,
            source_in,
            source_out,
            ..
        } = &screen.clips[0].body
        else {
            panic!("screen clip");
        };
        assert!(source.ends_with("screen.mov"));
        assert_eq!(*source_in, 0.0);
        assert!((*source_out - 1.0).abs() < 0.05);
        let mut edited = opened.project.clone();
        if let ClipBody::Video { source_out, .. } = &mut edited.tracks[2].clips[0].body {
            *source_out = 0.5;
        }
        edited.tracks[2].clips[0].duration = 0.5;
        save_project(Path::new(&opened.dir), &edited).unwrap();
        let again = load_or_create_take(&take, &root.join("movies")).unwrap();
        assert_eq!(again.project.tracks[2].clips[0].duration, 0.5);
    }
}
