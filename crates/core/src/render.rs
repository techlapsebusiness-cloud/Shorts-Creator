//! Export sizes, motion, and the deterministic frame the renderer samples.
//!
//! Skia draws these rects. The pixel tests below are the fixture that drawing
//! has to match.

use crate::Easing;

pub const SIZE_4K: (u32, u32) = (2160, 3840);
pub const SIZE_1440: (u32, u32) = (1440, 2560);
pub const SIZE_1080: (u32, u32) = (1080, 1920);
pub const AUDIO_RATE: u32 = 48_000;
pub const AUDIO_BITRATE_KBPS: u32 = 384;
pub const BLUR_SAMPLES_MIN: u32 = 4;
pub const BLUR_SAMPLES_MAX: u32 = 8;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlacedRect {
    pub x: i32,
    pub y: i32,
    pub side: i32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RenderProgress {
    pub frames_done: u32,
    pub frames_total: u32,
    pub elapsed_secs: f64,
}

pub fn output_size(name: &str) -> Option<(u32, u32)> {
    match name {
        "4K" => Some(SIZE_4K),
        "1440p" => Some(SIZE_1440),
        "1080p" => Some(SIZE_1080),
        _ => None,
    }
}

pub fn video_bitrate_kbps(width: u32) -> u32 {
    match width {
        2160 => 40_000,
        1440 => 16_000,
        _ => 8_000,
    }
}

pub fn select_encoder(codec_hevc: bool, available: &[&str]) -> &'static str {
    let order: &[&str] = if codec_hevc {
        &["hevc_videotoolbox", "hevc_vaapi", "hevc_nvenc", "libx265"]
    } else {
        &["h264_videotoolbox", "h264_vaapi", "h264_nvenc", "libx264"]
    };
    order
        .iter()
        .copied()
        .find(|name| available.contains(name))
        .unwrap_or(if codec_hevc { "libx265" } else { "libx264" })
}

pub fn ease(kind: Easing, t: f64) -> f64 {
    let t = t.clamp(0.0, 1.0);
    match kind {
        Easing::Linear => t,
        Easing::EaseInOut => t * t * (3.0 - 2.0 * t),
        Easing::Spring => spring(t),
    }
}

pub fn blur_samples(requested: u32) -> u32 {
    requested.clamp(BLUR_SAMPLES_MIN, BLUR_SAMPLES_MAX)
}

pub fn sample_times(frame: u32, frames: u32, samples: u32) -> Vec<f64> {
    let frames = frames.max(1);
    let samples = samples.max(1);
    (0..samples)
        .map(|index| (f64::from(frame) + f64::from(index) / f64::from(samples)) / f64::from(frames))
        .collect()
}

pub fn overlay_sample_count(frames: u32, fps: u32, animate_at_60: bool) -> u32 {
    let rate = if animate_at_60 {
        fps.max(60)
    } else {
        fps.max(1)
    };
    frames.saturating_mul(rate) / fps.max(1)
}

pub fn placed_rect(width: u32, height: u32, t: f64, easing: Easing) -> PlacedRect {
    let side = (width / 5).max(1);
    let travel = width.saturating_sub(side);
    let x = (f64::from(travel) * ease(easing, t)).round() as i32;
    let y = (height.saturating_sub(side) / 2) as i32;
    PlacedRect {
        x,
        y,
        side: side as i32,
    }
}

pub fn frame_rects(
    width: u32,
    height: u32,
    frames: u32,
    frame: u32,
    easing: Easing,
    motion_blur: bool,
) -> Vec<PlacedRect> {
    let samples = if motion_blur { blur_samples(6) } else { 1 };
    sample_times(frame, frames, samples)
        .into_iter()
        .map(|t| placed_rect(width, height, t, easing))
        .collect()
}

pub fn paint_rects(width: u32, height: u32, rects: &[PlacedRect]) -> Vec<u8> {
    let pixels = (width as usize).saturating_mul(height as usize);
    let mut cover = vec![0u16; pixels];
    let count = u16::try_from(rects.len().max(1)).unwrap_or(u16::MAX);
    for rect in rects {
        for row in rect.y..rect.y.saturating_add(rect.side) {
            if row < 0 || row >= height as i32 {
                continue;
            }
            for col in rect.x..rect.x.saturating_add(rect.side) {
                if col < 0 || col >= width as i32 {
                    continue;
                }
                let index = row as usize * width as usize + col as usize;
                cover[index] = cover[index].saturating_add(1);
            }
        }
    }
    let mut out = Vec::with_capacity(pixels * 4);
    for hits in cover {
        let value = (u32::from(hits) * 255 / u32::from(count)) as u8;
        out.extend_from_slice(&[value, value, value, 255]);
    }
    out
}

pub fn average_rgba(frames: &[impl AsRef<[u8]>]) -> Vec<u8> {
    let Some(first) = frames.first() else {
        return Vec::new();
    };
    let first = first.as_ref();
    let mut acc = vec![0u32; first.len()];
    for frame in frames {
        for (slot, byte) in acc.iter_mut().zip(frame.as_ref()) {
            *slot += u32::from(*byte);
        }
    }
    let count = frames.len() as u32;
    acc.into_iter().map(|sum| (sum / count) as u8).collect()
}

pub fn blend_frames(current: &[u8], previous: &[u8], speed: f64, enabled: bool) -> Vec<u8> {
    if !enabled || speed <= 1.0 || previous.len() != current.len() {
        return current.to_vec();
    }
    average_rgba(&[previous, current])
}

pub fn smooth_cursor(samples: &[(f64, f64)], amount: f64) -> Vec<(f64, f64)> {
    let Some((mut x, mut y)) = samples.first().copied() else {
        return Vec::new();
    };
    let gain = amount.clamp(0.05, 1.0);
    let mut out = Vec::with_capacity(samples.len());
    for (sample_x, sample_y) in samples {
        x += (sample_x - x) * gain;
        y += (sample_y - y) * gain;
        out.push((x, y));
    }
    out
}

pub fn downscale_box(pixels: &[u8], width: u32, height: u32, factor: u32) -> Vec<u8> {
    let factor = factor.max(1);
    let out_w = width / factor;
    let out_h = height / factor;
    let mut out = vec![0u8; (out_w * out_h) as usize * 4];
    let area = f64::from(factor * factor);
    for row in 0..out_h {
        for col in 0..out_w {
            let mut sum = [0.0; 4];
            for dy in 0..factor {
                for dx in 0..factor {
                    let source_row = row * factor + dy;
                    let source_col = col * factor + dx;
                    let index = ((source_row * width + source_col) * 4) as usize;
                    for channel in 0..4 {
                        sum[channel] += f64::from(pixels[index + channel]);
                    }
                }
            }
            let dest = ((row * out_w + col) * 4) as usize;
            for channel in 0..4 {
                out[dest + channel] = (sum[channel] / area).round() as u8;
            }
        }
    }
    out
}

pub fn ssim_rgba(left: &[u8], right: &[u8]) -> f64 {
    let pixels = left.len().min(right.len()) / 4;
    if pixels == 0 {
        return 1.0;
    }
    let luma = |bytes: &[u8], index: usize| {
        let offset = index * 4;
        f64::from(bytes[offset]) * 0.2126
            + f64::from(bytes[offset + 1]) * 0.7152
            + f64::from(bytes[offset + 2]) * 0.0722
    };
    let count = pixels as f64;
    let mut mean_left = 0.0;
    let mut mean_right = 0.0;
    for index in 0..pixels {
        mean_left += luma(left, index);
        mean_right += luma(right, index);
    }
    mean_left /= count;
    mean_right /= count;
    let mut var_left = 0.0;
    let mut var_right = 0.0;
    let mut cov = 0.0;
    for index in 0..pixels {
        let delta_left = luma(left, index) - mean_left;
        let delta_right = luma(right, index) - mean_right;
        var_left += delta_left * delta_left;
        var_right += delta_right * delta_right;
        cov += delta_left * delta_right;
    }
    var_left /= count;
    var_right /= count;
    cov /= count;
    let c1 = (0.01 * 255.0_f64).powi(2);
    let c2 = (0.03 * 255.0_f64).powi(2);
    ((2.0 * mean_left * mean_right + c1) * (2.0 * cov + c2))
        / ((mean_left.powi(2) + mean_right.powi(2) + c1) * (var_left + var_right + c2))
}

pub fn hash_frame(bytes: &[u8]) -> u64 {
    let mut hash = 0xcbf29ce484222325u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

pub fn eta_secs(progress: RenderProgress) -> f64 {
    if progress.frames_done == 0 || progress.frames_done >= progress.frames_total {
        0.0
    } else {
        progress.elapsed_secs / f64::from(progress.frames_done)
            * f64::from(progress.frames_total - progress.frames_done)
    }
}

pub fn fraction(progress: RenderProgress) -> f64 {
    if progress.frames_total == 0 {
        0.0
    } else {
        f64::from(progress.frames_done) / f64::from(progress.frames_total)
    }
}

fn spring(t: f64) -> f64 {
    let omega = 14.0;
    let zeta = 0.3;
    let damped = omega * f64::sqrt(1.0 - zeta * zeta);
    let decay = (-zeta * omega * t).exp();
    let wave = (damped * t).cos() + (zeta * omega / damped) * (damped * t).sin();
    1.0 - decay * wave
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn easing_presets_hold_the_ends_and_the_spring_overshoots() {
        assert!((ease(Easing::Linear, 0.25) - 0.25).abs() < 1e-9);
        let smooth = ease(Easing::EaseInOut, 0.25);
        assert!(smooth < 0.25);
        assert!((ease(Easing::EaseInOut, 0.0)).abs() < 1e-9);
        assert!((ease(Easing::EaseInOut, 1.0) - 1.0).abs() < 1e-9);
        assert!((ease(Easing::Spring, 0.0)).abs() < 1e-6);
        assert!((ease(Easing::Spring, 1.0) - 1.0).abs() < 0.08);
        let overshoot = (1..20).any(|step| ease(Easing::Spring, step as f64 / 20.0) > 1.02);
        assert!(overshoot);
    }

    #[test]
    fn overlays_at_60_have_twice_the_samples_of_30() {
        assert_eq!(overlay_sample_count(30, 30, false), 30);
        assert_eq!(overlay_sample_count(30, 30, true), 60);
        assert_eq!(blur_samples(2), 4);
        assert_eq!(blur_samples(12), 8);
    }

    #[test]
    fn motion_blur_softens_the_square_edge() {
        let sharp = paint_rects(48, 48, &frame_rects(48, 48, 8, 3, Easing::Linear, false));
        let blurred = paint_rects(48, 48, &frame_rects(48, 48, 8, 3, Easing::Linear, true));
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

    #[test]
    fn a_speed_up_blends_the_two_frames_and_a_cursor_jump_shrinks() {
        let first = paint_rects(
            20,
            20,
            &[PlacedRect {
                x: 0,
                y: 0,
                side: 6,
            }],
        );
        let second = paint_rects(
            20,
            20,
            &[PlacedRect {
                x: 10,
                y: 0,
                side: 6,
            }],
        );
        let blended = blend_frames(&second, &first, 2.0, true);
        assert!(
            blended
                .chunks(4)
                .any(|pixel| pixel[0] == 127 || pixel[0] == 128)
        );
        assert_eq!(blend_frames(&second, &first, 1.0, true), second);
        let raw = [(0.0, 0.0), (100.0, 0.0), (100.0, 0.0)];
        let smooth = smooth_cursor(&raw, 0.25);
        let raw_step = 100.0;
        let smooth_step = smooth[1].0 - smooth[0].0;
        assert!(smooth_step < raw_step);
    }

    #[test]
    fn golden_square_matches_and_preview_stays_near_the_export() {
        let rect = PlacedRect {
            x: 2,
            y: 3,
            side: 2,
        };
        let frame = paint_rects(8, 8, &[rect]);
        for y in 0..8 {
            for x in 0..8 {
                let index = (y * 8 + x) * 4;
                let inside = (2..4).contains(&x) && (3..5).contains(&y);
                let expected = if inside { 255 } else { 0 };
                assert_eq!(frame[index], expected);
            }
        }
        let export = paint_rects(64, 64, &frame_rects(64, 64, 4, 1, Easing::Linear, false));
        let preview = paint_rects(32, 32, &frame_rects(32, 32, 4, 1, Easing::Linear, false));
        let down = downscale_box(&export, 64, 64, 2);
        assert!(ssim_rgba(&down, &preview) >= 0.95);
        assert_eq!(hash_frame(&export), hash_frame(&export));
        let again = paint_rects(64, 64, &frame_rects(64, 64, 4, 1, Easing::Linear, false));
        assert_eq!(hash_frame(&export), hash_frame(&again));
    }

    #[test]
    fn encoder_falls_back_and_the_queue_estimates_a_finish() {
        assert_eq!(
            select_encoder(false, &["libx264", "h264_videotoolbox"]),
            "h264_videotoolbox"
        );
        assert_eq!(select_encoder(false, &[]), "libx264");
        assert_eq!(select_encoder(true, &["libx265"]), "libx265");
        assert_eq!(output_size("4K"), Some(SIZE_4K));
        assert_eq!(video_bitrate_kbps(2160), 40_000);
        let progress = RenderProgress {
            frames_done: 10,
            frames_total: 40,
            elapsed_secs: 2.0,
        };
        assert!((eta_secs(progress) - 6.0).abs() < 1e-9);
        assert!((fraction(progress) - 0.25).abs() < 1e-9);
    }
}
