//! FFmpeg child processes, capture, and render. M00 ships the doctor checks.
//! M06 ships the three-track recording session and cursor log.

pub const NAME: &str = "media";

pub mod cursor;
pub mod devices;
pub mod disk;
pub mod doctor;
pub mod ffmpeg;
pub mod record;
#[cfg(target_os = "macos")]
pub mod screenkit;
pub mod transcript;
pub mod whisper;
