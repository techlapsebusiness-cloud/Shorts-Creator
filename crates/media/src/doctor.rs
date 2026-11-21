//! Startup checks for ffmpeg, encoders, the whisper model, and the Linux capture stack.

use std::path::{Path, PathBuf};
use std::process::Command;

pub const WHISPER_MODEL_FILE: &str = "ggml-large-v3-turbo.bin";

const TRACKED_ENCODERS: &[&str] = &[
    "h264_videotoolbox",
    "hevc_videotoolbox",
    "h264_vaapi",
    "hevc_vaapi",
    "h264_nvenc",
    "hevc_nvenc",
    "libx264",
    "libx265",
];

pub struct Diagnosis {
    pub text: String,
    pub ok: bool,
}

pub fn diagnose() -> Diagnosis {
    let mut lines = Vec::new();
    let mut ok = true;

    match tool_version("ffmpeg") {
        Ok(line) => lines.push(format!("ffmpeg: {line}")),
        Err(err) => {
            ok = false;
            lines.push(err);
        }
    }
    match tool_version("ffprobe") {
        Ok(line) => lines.push(format!("ffprobe: {line}")),
        Err(err) => {
            ok = false;
            lines.push(err);
        }
    }

    match Command::new("ffmpeg")
        .args(["-hide_banner", "-encoders"])
        .output()
    {
        Ok(output) if output.status.success() => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let encoders = tracked_encoders(&stdout);
            if encoders.is_empty() {
                lines.push("encoders: none of the tracked encoders".to_string());
            } else {
                lines.push(format!("encoders: {}", encoders.join(", ")));
            }
        }
        Ok(output) => {
            ok = false;
            lines.push(format!(
                "encoders: ffmpeg -encoders exited {}",
                output.status
            ));
        }
        Err(err) => {
            ok = false;
            lines.push(format!("encoders: {err}"));
        }
    }

    let model_dir = default_model_dir();
    let model_path = model_dir.join(WHISPER_MODEL_FILE);
    if whisper_model_present(&model_dir) {
        lines.push(format!("whisper model: present ({})", model_path.display()));
    } else {
        lines.push(format!(
            "whisper model: absent ({}) — downloaded on first run",
            model_dir.display()
        ));
    }

    if cfg!(target_os = "linux") {
        match portal_binary() {
            Some(path) => lines.push(format!("portal binary: {}", path.display())),
            None => {
                ok = false;
                lines.push("portal binary: missing".to_string());
            }
        }
        if process_argv0_is("xdg-desktop-portal") {
            lines.push("portal process: running".to_string());
        } else {
            ok = false;
            lines.push("portal process: not running".to_string());
        }
        match std::env::var_os("XDG_RUNTIME_DIR") {
            Some(runtime) => {
                let socket = pipewire_socket(Path::new(&runtime));
                if socket.exists() {
                    lines.push(format!("pipewire socket: {}", socket.display()));
                } else {
                    ok = false;
                    lines.push(format!("pipewire socket: missing ({})", socket.display()));
                }
            }
            None => {
                ok = false;
                lines.push("pipewire socket: XDG_RUNTIME_DIR is unset".to_string());
            }
        }
        if process_argv0_is("pipewire") {
            lines.push("pipewire process: running".to_string());
        } else {
            ok = false;
            lines.push("pipewire process: not running".to_string());
        }
        lines.push("macos permissions: skipped".to_string());
    } else if cfg!(target_os = "macos") {
        lines.push(
            "macos permissions: MANUAL — Screen Recording, Camera, and Microphone in System Settings"
                .to_string(),
        );
        lines.push("portal: skipped".to_string());
        lines.push("pipewire: skipped".to_string());
    } else {
        lines.push("portal: skipped".to_string());
        lines.push("pipewire: skipped".to_string());
        lines.push("macos permissions: skipped".to_string());
    }

    if ok {
        lines.push("doctor: ok".to_string());
    } else {
        lines.push("doctor: failed".to_string());
    }

    Diagnosis {
        text: format!("{}\n", lines.join("\n")),
        ok,
    }
}

pub fn tracked_encoders(ffmpeg_encoders_stdout: &str) -> Vec<String> {
    let mut found = Vec::new();
    for line in ffmpeg_encoders_stdout.lines() {
        let mut fields = line.split_whitespace();
        let Some(flags) = fields.next() else {
            continue;
        };
        let Some(name) = fields.next() else {
            continue;
        };
        if is_encoder_flag(flags) && TRACKED_ENCODERS.contains(&name) {
            found.push(name.to_string());
        }
    }
    found
}

pub fn version_line(tool: &str, output: Option<&str>) -> Result<String, String> {
    let Some(output) = output else {
        return Err(format!("{tool}: not found"));
    };
    let line = output.lines().next().unwrap_or("").trim();
    if line.is_empty() {
        return Err(format!("{tool}: empty version output"));
    }
    Ok(line.to_string())
}

pub fn model_dir_from_home(home: &Path, target_os: &str) -> PathBuf {
    if target_os == "macos" {
        home.join("Library/Application Support/shorts-studio/models")
    } else {
        home.join(".local/share/shorts-studio/models")
    }
}

pub fn whisper_model_present(dir: &Path) -> bool {
    dir.join(WHISPER_MODEL_FILE).is_file()
}

pub fn whisper_model_path() -> PathBuf {
    default_model_dir().join(WHISPER_MODEL_FILE)
}

pub fn pipewire_socket(runtime_dir: &Path) -> PathBuf {
    runtime_dir.join("pipewire-0")
}

pub fn portal_binary() -> Option<PathBuf> {
    for candidate in [
        "/usr/lib/xdg-desktop-portal",
        "/usr/libexec/xdg-desktop-portal",
    ] {
        let path = PathBuf::from(candidate);
        if path.is_file() {
            return Some(path);
        }
    }
    on_path("xdg-desktop-portal")
}

fn tool_version(tool: &str) -> Result<String, String> {
    match Command::new(tool).arg("-version").output() {
        Ok(output) if output.status.success() => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            version_line(tool, Some(&stdout))
        }
        Ok(output) => Err(format!("{tool}: exited {}", output.status)),
        Err(err) => Err(format!("{tool}: {err}")),
    }
}

fn default_model_dir() -> PathBuf {
    let home = std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));
    if cfg!(target_os = "macos") {
        return model_dir_from_home(&home, "macos");
    }
    if let Some(xdg) = std::env::var_os("XDG_DATA_HOME") {
        return PathBuf::from(xdg).join("shorts-studio/models");
    }
    model_dir_from_home(&home, "linux")
}

fn on_path(bin: &str) -> Option<PathBuf> {
    let path = std::env::var_os("PATH")?;
    std::env::split_paths(&path).find_map(|dir| {
        let candidate = dir.join(bin);
        candidate.is_file().then_some(candidate)
    })
}

fn process_argv0_is(expected: &str) -> bool {
    let Ok(entries) = std::fs::read_dir("/proc") else {
        return false;
    };
    for entry in entries.flatten() {
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if !name.chars().all(|character| character.is_ascii_digit()) {
            continue;
        }
        let Ok(raw) = std::fs::read(entry.path().join("cmdline")) else {
            continue;
        };
        let Some(argv0) = raw.split(|byte| *byte == 0).next() else {
            continue;
        };
        let Ok(argv0) = std::str::from_utf8(argv0) else {
            continue;
        };
        if argv0.rsplit(['/', '\\']).next() == Some(expected) {
            return true;
        }
    }
    false
}

fn is_encoder_flag(flags: &str) -> bool {
    flags.len() == 6
        && flags
            .chars()
            .all(|character| matches!(character, 'V' | 'A' | 'S' | 'D' | 'F' | '.'))
}

#[cfg(test)]
mod tests {
    use super::{
        WHISPER_MODEL_FILE, diagnose, model_dir_from_home, tracked_encoders, version_line,
        whisper_model_present,
    };
    use std::path::{Path, PathBuf};

    #[test]
    fn parses_tracked_encoders() {
        let sample = "\
 V....D libx264              x
 V....D h264_vaapi           y
 V..... h264_qsv             z
 A....D aac                  a
";
        assert_eq!(
            tracked_encoders(sample),
            vec!["libx264".to_string(), "h264_vaapi".to_string()]
        );
    }

    #[test]
    fn version_line_requires_text() {
        assert!(version_line("ffmpeg", None).is_err());
        assert!(version_line("ffmpeg", Some("")).is_err());
        assert_eq!(
            version_line("ffmpeg", Some("ffmpeg version n9.0.2\n")).expect("version line"),
            "ffmpeg version n9.0.2"
        );
    }

    #[test]
    fn model_dir_follows_the_os() {
        let home = Path::new("/home/akshay");
        assert_eq!(
            model_dir_from_home(home, "macos"),
            PathBuf::from("/home/akshay/Library/Application Support/shorts-studio/models")
        );
        assert_eq!(
            model_dir_from_home(home, "linux"),
            PathBuf::from("/home/akshay/.local/share/shorts-studio/models")
        );
    }

    #[test]
    fn whisper_file_is_detected() {
        let dir = std::env::temp_dir().join(format!("shorts-studio-m00-{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("temp dir");
        assert!(!whisper_model_present(&dir));
        std::fs::write(dir.join(WHISPER_MODEL_FILE), b"model").expect("model file");
        assert!(whisper_model_present(&dir));
        std::fs::remove_dir_all(&dir).expect("cleanup");
    }

    #[test]
    fn this_machine_passes_doctor() {
        let diagnosis = diagnose();
        assert!(diagnosis.ok, "{}", diagnosis.text);
        assert!(diagnosis.text.contains("doctor: ok"));
    }
}
