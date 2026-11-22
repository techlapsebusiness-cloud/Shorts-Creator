//! FFmpeg child processes, capture, and render. M00 ships the doctor checks.
//! M06 ships the three-track recording session and cursor log.

pub const NAME: &str = "media";

pub mod cover;
pub mod cursor;
pub mod daily;
pub mod devices;
pub mod disk;
pub mod doctor;
pub mod export_take;
pub mod ffmpeg;
#[cfg(target_os = "macos")]
pub mod icons;
pub mod loudness;
pub mod portal;
pub mod record;
pub mod render;
#[cfg(target_os = "macos")]
pub mod screenkit;
pub mod transcript;
pub mod whisper;
