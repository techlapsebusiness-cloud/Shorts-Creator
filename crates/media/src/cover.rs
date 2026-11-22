//! 1080×1920 cover PNG painted from the core layout boxes.

use std::path::Path;

use core::{COVER_H, COVER_W, CoverTemplate, FaceBox, fit_text};
use skia_safe::{Color, EncodedImageFormat, Paint, Rect, surfaces};

use crate::ffmpeg::FfmpegError;

pub fn write_cover_png(
    path: &Path,
    template: CoverTemplate,
    hook: &str,
    face: Option<FaceBox>,
) -> Result<(), FfmpegError> {
    let width = COVER_W as i32;
    let height = COVER_H as i32;
    let mut surface = surfaces::raster_n32_premul((width, height)).ok_or_else(|| FfmpegError {
        message: "cover surface".to_string(),
    })?;
    let canvas = surface.canvas();
    let (red, green, blue) = match template {
        CoverTemplate::HookFace => (18, 22, 28),
        CoverTemplate::ErrorHook => (48, 16, 18),
        CoverTemplate::BeforeAfter => (16, 28, 32),
    };
    canvas.clear(Color::from_rgb(red, green, blue));
    let mut paint = Paint::default();
    paint.set_anti_alias(false);
    let fitted = fit_text(hook, 80.0, 160.0, 920.0, 700.0);
    paint.set_color(Color::WHITE);
    for (index, _) in fitted.lines.iter().enumerate() {
        let y = fitted.y + index as f64 * fitted.font_px * 1.2;
        canvas.draw_rect(
            Rect::from_xywh(
                fitted.x as f32,
                y as f32,
                fitted.width as f32,
                (fitted.font_px * 0.7) as f32,
            ),
            &paint,
        );
    }
    if let Some(face) = face {
        paint.set_color(Color::from_rgb(220, 176, 148));
        canvas.draw_rect(
            Rect::from_xywh(
                face.x as f32,
                face.y as f32,
                face.width as f32,
                face.height as f32,
            ),
            &paint,
        );
    }
    let image = surface.image_snapshot();
    let data = image
        .encode(None, EncodedImageFormat::PNG, None)
        .ok_or_else(|| FfmpegError {
            message: "cover png encode".to_string(),
        })?;
    std::fs::write(path, data.as_bytes()).map_err(|error| FfmpegError {
        message: format!("cover write: {error}"),
    })
}

pub fn png_size(bytes: &[u8]) -> Option<(u32, u32)> {
    if bytes.len() < 24 || &bytes[0..8] != b"\x89PNG\r\n\x1a\n" {
        return None;
    }
    if &bytes[12..16] != b"IHDR" {
        return None;
    }
    let width = u32::from_be_bytes(bytes[16..20].try_into().ok()?);
    let height = u32::from_be_bytes(bytes[20..24].try_into().ok()?);
    Some((width, height))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn cover_png_is_1080_by_1920_and_a_bounded_file() {
        let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../target/m12-cover-tests")
            .join(std::process::id().to_string());
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("cover.png");
        let face = FaceBox {
            x: 700.0,
            y: 1200.0,
            width: 280.0,
            height: 280.0,
            mouth_open: 0.8,
        };
        write_cover_png(
            &path,
            CoverTemplate::HookFace,
            "This file survives",
            Some(face),
        )
        .unwrap();
        let bytes = std::fs::read(&path).unwrap();
        assert_eq!(png_size(&bytes), Some((1080, 1920)));
        assert!(
            bytes.len() > 2_000 && bytes.len() < 2_000_000,
            "{}",
            bytes.len()
        );
    }
}
