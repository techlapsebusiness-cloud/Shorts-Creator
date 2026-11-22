//! Cursor telemetry written as JSON Lines during a take.

use serde::{Deserialize, Serialize};
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

pub const CURSOR_HZ: u64 = 120;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CursorSample {
    pub t_ms: f64,
    pub x: f64,
    pub y: f64,
    pub buttons: u8,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub click: Option<bool>,
}

#[derive(Debug)]
pub struct CursorError {
    pub message: String,
}

impl std::fmt::Display for CursorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for CursorError {}

pub fn parse_cursor_jsonl(path: &Path) -> Result<Vec<CursorSample>, CursorError> {
    let file = File::open(path).map_err(|error| CursorError {
        message: format!("open {}: {error}", path.display()),
    })?;
    let mut samples = Vec::new();
    for (index, line) in BufReader::new(file).lines().enumerate() {
        let line = line.map_err(|error| CursorError {
            message: format!("read {}: {error}", path.display()),
        })?;
        if line.trim().is_empty() {
            continue;
        }
        let sample: CursorSample = serde_json::from_str(&line).map_err(|error| CursorError {
            message: format!("cursor.jsonl line {}: {error}", index + 1),
        })?;
        samples.push(sample);
    }
    Ok(samples)
}

pub fn assert_monotonic(samples: &[CursorSample]) -> Result<(), CursorError> {
    let mut previous = f64::NEG_INFINITY;
    for sample in samples {
        if sample.t_ms < previous {
            return Err(CursorError {
                message: format!(
                    "cursor timestamps must be monotonic; saw {} after {previous}",
                    sample.t_ms
                ),
            });
        }
        previous = sample.t_ms;
    }
    Ok(())
}

pub struct CursorWriter {
    path: PathBuf,
    stop: Arc<AtomicBool>,
    handle: Option<JoinHandle<Result<(), CursorError>>>,
}

impl CursorWriter {
    pub fn start_synthetic(path: PathBuf, origin: Instant) -> Result<Self, CursorError> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|error| CursorError {
                message: format!("create {}: {error}", parent.display()),
            })?;
        }
        let mut file = OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(&path)
            .map_err(|error| CursorError {
                message: format!("create {}: {error}", path.display()),
            })?;

        let stop = Arc::new(AtomicBool::new(false));
        let stop_flag = Arc::clone(&stop);
        let handle = thread::spawn(move || {
            let period = Duration::from_nanos(1_000_000_000 / CURSOR_HZ);
            let mut index = 0u64;
            let mut previous_buttons = 0u8;
            while !stop_flag.load(Ordering::SeqCst) {
                let t_ms = origin.elapsed().as_secs_f64() * 1000.0;
                let (x, y, buttons) = live_cursor().unwrap_or_else(|| {
                    let angle = index as f64 * 0.05;
                    (960.0 + angle.cos() * 200.0, 540.0 + angle.sin() * 120.0, 0)
                });
                let click = (buttons != 0 && previous_buttons == 0).then_some(true);
                previous_buttons = buttons;
                let sample = CursorSample {
                    t_ms,
                    x,
                    y,
                    buttons,
                    click,
                };
                let line = serde_json::to_string(&sample).map_err(|error| CursorError {
                    message: format!("serialize cursor sample: {error}"),
                })?;
                writeln!(file, "{line}").map_err(|error| CursorError {
                    message: format!("write cursor sample: {error}"),
                })?;
                index += 1;
                thread::sleep(period);
            }
            file.flush().map_err(|error| CursorError {
                message: format!("flush cursor.jsonl: {error}"),
            })?;
            Ok(())
        });

        Ok(Self {
            path,
            stop,
            handle: Some(handle),
        })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn stop(mut self) -> Result<PathBuf, CursorError> {
        self.stop.store(true, Ordering::SeqCst);
        if let Some(handle) = self.handle.take() {
            handle.join().map_err(|_| CursorError {
                message: "cursor writer thread panicked".to_string(),
            })??;
        }
        Ok(self.path.clone())
    }
}

fn live_cursor() -> Option<(f64, f64, u8)> {
    #[cfg(target_os = "macos")]
    {
        sample_macos()
    }
    #[cfg(not(target_os = "macos"))]
    {
        None
    }
}

#[cfg(target_os = "macos")]
fn sample_macos() -> Option<(f64, f64, u8)> {
    let event = cg_event_create(std::ptr::null_mut());
    if event.is_null() {
        return None;
    }
    let point = cg_event_location(event);
    unsafe { cf_release(event) };
    let button = cg_button_state(1, 0);
    Some((point.x, point.y, u8::from(button != 0)))
}

#[cfg(target_os = "macos")]
#[link(name = "CoreGraphics", kind = "framework")]
unsafe extern "C" {
    #[link_name = "CGEventCreate"]
    safe fn cg_event_create(source: *mut std::ffi::c_void) -> *mut std::ffi::c_void;
    #[link_name = "CGEventGetLocation"]
    safe fn cg_event_location(event: *mut std::ffi::c_void) -> CgPoint;
    #[link_name = "CGEventSourceButtonState"]
    safe fn cg_button_state(state: i32, button: i32) -> u8;
}

#[cfg(target_os = "macos")]
#[link(name = "CoreFoundation", kind = "framework")]
unsafe extern "C" {
    #[link_name = "CFRelease"]
    unsafe fn cf_release(value: *const std::ffi::c_void);
}

#[cfg(target_os = "macos")]
#[repr(C)]
struct CgPoint {
    x: f64,
    y: f64,
}

impl Drop for CursorWriter {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::SeqCst);
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::time::Duration;

    #[test]
    fn synthetic_cursor_is_monotonic_jsonl() {
        let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../target/cursor-test")
            .join(std::process::id().to_string());
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("cursor.jsonl");
        let writer = CursorWriter::start_synthetic(path.clone(), Instant::now()).unwrap();
        thread::sleep(Duration::from_millis(200));
        let path = writer.stop().unwrap();
        let samples = parse_cursor_jsonl(&path).unwrap();
        assert!(samples.len() >= 5);
        assert_monotonic(&samples).unwrap();
        let _ = std::fs::remove_dir_all(&dir);
    }
}
