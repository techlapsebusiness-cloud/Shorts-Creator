//! Capture device discovery via FFmpeg input device listings.

use serde::Serialize;
use std::process::{Command, Stdio};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CaptureDevice {
    pub id: String,
    pub label: String,
    pub kind: DeviceKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum DeviceKind {
    Screen,
    Camera,
    Microphone,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceList {
    pub screens: Vec<CaptureDevice>,
    pub cameras: Vec<CaptureDevice>,
    pub microphones: Vec<CaptureDevice>,
    pub backend: String,
}

pub fn list_devices() -> DeviceList {
    if cfg!(target_os = "macos") {
        list_avfoundation()
    } else if cfg!(target_os = "linux") {
        list_linux()
    } else {
        DeviceList {
            screens: Vec::new(),
            cameras: Vec::new(),
            microphones: Vec::new(),
            backend: "unsupported".to_string(),
        }
    }
}

fn list_avfoundation() -> DeviceList {
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
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .output();

    let stderr = match output {
        Ok(output) => String::from_utf8_lossy(&output.stderr).into_owned(),
        Err(_) => String::new(),
    };

    let mut screens = Vec::new();
    let mut cameras = Vec::new();
    let mut microphones = Vec::new();
    let mut section = Section::None;

    for line in stderr.lines() {
        let lower = line.to_ascii_lowercase();
        if lower.contains("avfoundation video devices") {
            section = Section::Video;
            continue;
        }
        if lower.contains("avfoundation audio devices") {
            section = Section::Audio;
            continue;
        }
        let Some((index, label)) = parse_avfoundation_line(line) else {
            continue;
        };
        let label_lower = label.to_ascii_lowercase();
        match section {
            Section::Video
                if label_lower.contains("capture screen") || label_lower.contains("screen") =>
            {
                screens.push(CaptureDevice {
                    id: index,
                    label,
                    kind: DeviceKind::Screen,
                });
            }
            Section::Video => {
                cameras.push(CaptureDevice {
                    id: index,
                    label,
                    kind: DeviceKind::Camera,
                });
            }
            Section::Audio => {
                microphones.push(CaptureDevice {
                    id: index,
                    label,
                    kind: DeviceKind::Microphone,
                });
            }
            Section::None => {}
        }
    }

    DeviceList {
        screens,
        cameras,
        microphones,
        backend: "avfoundation".to_string(),
    }
}

pub fn pick_screen(list: &DeviceList) -> Option<&CaptureDevice> {
    list.screens
        .iter()
        .find(|device| device.label.to_ascii_lowercase().contains("capture screen"))
        .or(list.screens.first())
}

pub fn pick_camera(list: &DeviceList) -> Option<&CaptureDevice> {
    list.cameras
        .iter()
        .find(|device| !device.label.to_ascii_lowercase().contains("iphone"))
        .or(list.cameras.first())
}

pub fn pick_microphone(list: &DeviceList) -> Option<&CaptureDevice> {
    list.microphones
        .iter()
        .find(|device| !device.label.to_ascii_lowercase().contains("iphone"))
        .or(list.microphones.first())
}

fn list_linux() -> DeviceList {
    let mut cameras = Vec::new();
    if let Ok(entries) = std::fs::read_dir("/dev") {
        for entry in entries.flatten() {
            let name = entry.file_name();
            let name = name.to_string_lossy();
            if name.starts_with("video") {
                cameras.push(CaptureDevice {
                    id: format!("/dev/{name}"),
                    label: format!("V4L2 {name}"),
                    kind: DeviceKind::Camera,
                });
            }
        }
    }

    let microphones = vec![CaptureDevice {
        id: "default".to_string(),
        label: "Default Pulse/PipeWire".to_string(),
        kind: DeviceKind::Microphone,
    }];

    let screens = vec![CaptureDevice {
        id: "portal".to_string(),
        label: "Portal ScreenCast".to_string(),
        kind: DeviceKind::Screen,
    }];

    DeviceList {
        screens,
        cameras,
        microphones,
        backend: "linux".to_string(),
    }
}

#[derive(Clone, Copy)]
enum Section {
    None,
    Video,
    Audio,
}

fn parse_avfoundation_line(line: &str) -> Option<(String, String)> {
    // Lines look like: `[AVFoundation indev @ 0x…] [0] FaceTime HD Camera`
    let bytes = line.as_bytes();
    let mut index_start = None;
    let mut i = 0;
    while i + 2 < bytes.len() {
        if bytes[i] == b'[' && bytes[i + 1].is_ascii_digit() {
            index_start = Some(i + 1);
            break;
        }
        i += 1;
    }
    let start = index_start?;
    let rest = &line[start..];
    let end = rest.find(']')?;
    let index = rest[..end].trim().to_string();
    if !index.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    let label = rest[end + 1..].trim().to_string();
    if label.is_empty() {
        return None;
    }
    Some((index, label))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_avfoundation_device_line() {
        let (id, label) =
            parse_avfoundation_line("[AVFoundation indev @ 0x0] [0] FaceTime HD Camera").unwrap();
        assert_eq!(id, "0");
        assert_eq!(label, "FaceTime HD Camera");
    }

    #[test]
    fn list_devices_returns_a_backend() {
        let list = list_devices();
        assert!(!list.backend.is_empty());
    }
}
