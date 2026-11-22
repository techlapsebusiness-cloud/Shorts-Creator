//! 9:16 crop, camera chip, and caption layout.
//!
//! The crop follows the cursor with a critically damped spring and never
//! steps more than [`MAX_CROP_STEP_PX`] in one frame. Safe-zone insets keep
//! the camera and captions out of the Shorts button column and title area.

use crate::{CamPosition, CamSize};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SafeInsets {
    pub top: f64,
    pub right: f64,
    pub bottom: f64,
    pub left: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CropFollow {
    pub x: f64,
    pub y: f64,
    pub vx: f64,
    pub vy: f64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CaptionPage {
    pub lines: Vec<Vec<String>>,
    pub active_line: usize,
    pub active_index: usize,
}

pub const MAX_CROP_STEP_PX: f64 = 48.0;
pub const CROP_OMEGA: f64 = 6.0;
pub const CAPTION_LINE_CHARS: usize = 28;

pub fn shorts_insets(frame_w: f64, frame_h: f64) -> SafeInsets {
    SafeInsets {
        top: frame_h * 0.08,
        right: frame_w * 0.16,
        bottom: frame_h * 0.18,
        left: frame_w * 0.04,
    }
}

pub fn safe_rect(frame_w: f64, frame_h: f64) -> Rect {
    let insets = shorts_insets(frame_w, frame_h);
    Rect {
        x: insets.left,
        y: insets.top,
        width: (frame_w - insets.left - insets.right).max(1.0),
        height: (frame_h - insets.top - insets.bottom).max(1.0),
    }
}

pub fn frame_margin(frame_w: f64, frame_h: f64) -> Rect {
    let margin = (frame_w * 0.02).max(8.0);
    Rect {
        x: margin,
        y: margin,
        width: (frame_w - margin * 2.0).max(1.0),
        height: (frame_h - margin * 2.0).max(1.0),
    }
}

pub fn cam_side(size: CamSize, frame_w: f64) -> f64 {
    let unit = frame_w / 1080.0;
    let px = match size {
        CamSize::S => 140.0,
        CamSize::M => 200.0,
        CamSize::L => 280.0,
    };
    px * unit
}

pub fn cam_rect(
    frame_w: f64,
    frame_h: f64,
    position: CamPosition,
    size: CamSize,
    safe: bool,
) -> Rect {
    let bounds = if safe {
        safe_rect(frame_w, frame_h)
    } else {
        frame_margin(frame_w, frame_h)
    };
    let side = cam_side(size, frame_w).min(bounds.width).min(bounds.height);
    let (ax, ay) = slot_anchor(position);
    Rect {
        x: bounds.x + ax * (bounds.width - side),
        y: bounds.y + ay * (bounds.height - side),
        width: side,
        height: side,
    }
}

pub fn caption_rect(frame_w: f64, frame_h: f64, line_count: usize, safe: bool) -> Rect {
    let bounds = if safe {
        safe_rect(frame_w, frame_h)
    } else {
        frame_margin(frame_w, frame_h)
    };
    let lines = line_count.clamp(1, 2) as f64;
    let height = (frame_h * 0.045 * lines).min(bounds.height);
    Rect {
        x: bounds.x,
        y: bounds.y + bounds.height - height,
        width: bounds.width,
        height,
    }
}

pub fn crop_size(source_w: f64, source_h: f64, zoom: f64) -> (f64, f64) {
    let zoom = zoom.max(1.0);
    let mut height = source_h / zoom;
    let mut width = height * 9.0 / 16.0;
    if width > source_w {
        width = source_w / zoom;
        height = width * 16.0 / 9.0;
    }
    (width.max(1.0), height.max(1.0))
}

pub fn crop_target(cursor_x: f64, cursor_y: f64, source_w: f64, source_h: f64, zoom: f64) -> Rect {
    let (width, height) = crop_size(source_w, source_h, zoom);
    clamp_rect(
        Rect {
            x: cursor_x - width / 2.0,
            y: cursor_y - height / 2.0,
            width,
            height,
        },
        source_w,
        source_h,
    )
}

pub fn step_crop(
    state: CropFollow,
    target: Rect,
    source_w: f64,
    source_h: f64,
    dt: f64,
) -> (CropFollow, Rect) {
    let dt = dt.max(1.0 / 240.0);
    let (x, _) = damp(state.x, state.vx, target.x, dt);
    let (y, _) = damp(state.y, state.vy, target.y, dt);
    let dx = x - state.x;
    let dy = y - state.y;
    let distance = (dx * dx + dy * dy).sqrt();
    let (x, y) = if distance > MAX_CROP_STEP_PX {
        let scale = MAX_CROP_STEP_PX / distance;
        (state.x + dx * scale, state.y + dy * scale)
    } else {
        (x, y)
    };
    let rect = clamp_rect(
        Rect {
            x,
            y,
            width: target.width,
            height: target.height,
        },
        source_w,
        source_h,
    );
    (
        CropFollow {
            x: rect.x,
            y: rect.y,
            vx: (rect.x - state.x) / dt,
            vy: (rect.y - state.y) / dt,
        },
        rect,
    )
}

pub fn zoom_at(keys: &[(f64, f64)], time: f64) -> f64 {
    let Some((first_t, first_z)) = keys.first().copied() else {
        return 1.0;
    };
    if time <= first_t {
        return first_z.max(1.0);
    }
    for pair in keys.windows(2) {
        let (t0, z0) = pair[0];
        let (t1, z1) = pair[1];
        if time <= t1 {
            let span = t1 - t0;
            let t = if span == 0.0 { 1.0 } else { (time - t0) / span };
            return (z0 + (z1 - z0) * t).max(1.0);
        }
    }
    keys.last().map(|(_, zoom)| zoom.max(1.0)).unwrap_or(1.0)
}

pub fn caption_page(words: &[impl AsRef<str>], active: usize, max_chars: usize) -> CaptionPage {
    let words: Vec<String> = words.iter().map(|word| word.as_ref().to_string()).collect();
    if words.is_empty() {
        return CaptionPage {
            lines: Vec::new(),
            active_line: 0,
            active_index: 0,
        };
    }
    let active = active.min(words.len() - 1);
    let mut start = active;
    while start > 0 && pack_lines(&words[start - 1..=active], max_chars).len() <= 2 {
        start -= 1;
    }
    let mut end = active;
    while end + 1 < words.len() && pack_lines(&words[start..=end + 1], max_chars).len() <= 2 {
        end += 1;
    }
    let lines = pack_lines(&words[start..=end], max_chars);
    let mut seen = start;
    let mut active_line = 0;
    let mut active_index = 0;
    for (line_index, line) in lines.iter().enumerate() {
        for (word_index, _) in line.iter().enumerate() {
            if seen == active {
                active_line = line_index;
                active_index = word_index;
            }
            seen += 1;
        }
    }
    CaptionPage {
        lines,
        active_line,
        active_index,
    }
}

pub fn cam_mask(shape: crate::CamShape, side: u32) -> Vec<u8> {
    let side = side.max(2);
    let mut pixels = vec![0u8; (side * side) as usize];
    let radius = f64::from(side) / 2.0;
    let round = f64::from(side) / 4.0;
    for y in 0..side {
        for x in 0..side {
            let inside = match shape {
                crate::CamShape::Square => true,
                crate::CamShape::Circle => {
                    let dx = f64::from(x) + 0.5 - radius;
                    let dy = f64::from(y) + 0.5 - radius;
                    dx * dx + dy * dy <= radius * radius
                }
                crate::CamShape::Rounded => rounded_contains(x, y, side, round),
            };
            if inside {
                pixels[(y * side + x) as usize] = 255;
            }
        }
    }
    pixels
}

fn pack_lines(words: &[String], max_chars: usize) -> Vec<Vec<String>> {
    let max_chars = max_chars.max(1);
    let mut lines = Vec::new();
    let mut current = Vec::new();
    let mut used = 0usize;
    for word in words {
        let len = word.chars().count().max(1);
        if current.is_empty() || used + 1 + len <= max_chars {
            if !current.is_empty() {
                used += 1;
            }
            used += len;
            current.push(word.clone());
        } else {
            lines.push(std::mem::take(&mut current));
            current.push(word.clone());
            used = len;
        }
    }
    if !current.is_empty() {
        lines.push(current);
    }
    lines
}

fn slot_anchor(position: CamPosition) -> (f64, f64) {
    match position {
        CamPosition::TopLeft => (0.0, 0.0),
        CamPosition::TopCenter => (0.5, 0.0),
        CamPosition::TopRight => (1.0, 0.0),
        CamPosition::MiddleLeft => (0.0, 0.5),
        CamPosition::MiddleCenter => (0.5, 0.5),
        CamPosition::MiddleRight => (1.0, 0.5),
        CamPosition::BottomLeft => (0.0, 1.0),
        CamPosition::BottomCenter => (0.5, 1.0),
        CamPosition::BottomRight => (1.0, 1.0),
    }
}

fn damp(position: f64, velocity: f64, target: f64, dt: f64) -> (f64, f64) {
    let accel = CROP_OMEGA * CROP_OMEGA * (target - position) - 2.0 * CROP_OMEGA * velocity;
    let velocity = velocity + accel * dt;
    (position + velocity * dt, velocity)
}

fn clamp_rect(rect: Rect, source_w: f64, source_h: f64) -> Rect {
    let width = rect.width.min(source_w).max(1.0);
    let height = rect.height.min(source_h).max(1.0);
    Rect {
        x: rect.x.clamp(0.0, (source_w - width).max(0.0)),
        y: rect.y.clamp(0.0, (source_h - height).max(0.0)),
        width,
        height,
    }
}

fn rounded_contains(x: u32, y: u32, side: u32, radius: f64) -> bool {
    let px = f64::from(x) + 0.5;
    let py = f64::from(y) + 0.5;
    let edge = f64::from(side);
    let cx = if px < radius {
        radius
    } else if px > edge - radius {
        edge - radius
    } else {
        px
    };
    let cy = if py < radius {
        radius
    } else if py > edge - radius {
        edge - radius
    } else {
        py
    };
    let dx = px - cx;
    let dy = py - cy;
    dx * dx + dy * dy <= radius * radius
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{CamPosition, CamShape, CamSize};

    fn contains(outer: Rect, inner: Rect) -> bool {
        inner.x + 0.01 >= outer.x
            && inner.y + 0.01 >= outer.y
            && inner.x + inner.width <= outer.x + outer.width + 0.01
            && inner.y + inner.height <= outer.y + outer.height + 0.01
    }

    #[test]
    fn crop_stays_inside_and_never_jumps() {
        let source_w = 1920.0;
        let source_h = 1080.0;
        let start = crop_target(200.0, 200.0, source_w, source_h, 1.0);
        let mut follow = CropFollow {
            x: start.x,
            y: start.y,
            vx: 0.0,
            vy: 0.0,
        };
        let jump = crop_target(1800.0, 900.0, source_w, source_h, 1.0);
        let (next, rect) = step_crop(follow, jump, source_w, source_h, 1.0 / 30.0);
        let moved = ((next.x - follow.x).powi(2) + (next.y - follow.y).powi(2)).sqrt();
        assert!(moved <= MAX_CROP_STEP_PX + 1e-6, "{moved}");
        assert!(rect.x >= 0.0 && rect.y >= 0.0);
        assert!(rect.x + rect.width <= source_w + 1.0);
        assert!(rect.y + rect.height <= source_h + 1.0);
        follow = next;
        for _ in 0..180 {
            let (state, rect) = step_crop(follow, jump, source_w, source_h, 1.0 / 30.0);
            let moved = ((state.x - follow.x).powi(2) + (state.y - follow.y).powi(2)).sqrt();
            assert!(moved <= MAX_CROP_STEP_PX + 1e-6);
            assert!(rect.x >= -0.01 && rect.y >= -0.01);
            assert!(rect.x + rect.width <= source_w + 1.0);
            assert!(rect.y + rect.height <= source_h + 1.0);
            follow = state;
        }
        assert!((follow.x - jump.x).abs() < 1.0);
        assert!((follow.y - jump.y).abs() < 1.0);
    }

    #[test]
    fn zoom_keyframes_blend_and_safe_zone_holds_the_cam() {
        assert!((zoom_at(&[(0.0, 1.0), (1.0, 2.0)], 0.5) - 1.5).abs() < 1e-9);
        let frame_w = 1080.0;
        let frame_h = 1920.0;
        let safe = safe_rect(frame_w, frame_h);
        let cam = cam_rect(frame_w, frame_h, CamPosition::BottomRight, CamSize::L, true);
        assert!(contains(safe, cam));
        let loose = cam_rect(
            frame_w,
            frame_h,
            CamPosition::BottomRight,
            CamSize::L,
            false,
        );
        assert!(loose.x + loose.width > safe.x + safe.width);
        let caption = caption_rect(frame_w, frame_h, 2, true);
        assert!(contains(safe, caption));
    }

    #[test]
    fn hinglish_caption_breaks_into_two_lines() {
        let words = [
            "Yeh", "file", "crash", "ke", "baad", "bhi", "survive", "karti", "hai", "kyunki",
            "rename", "atomic", "hota", "hai",
        ];
        let page = caption_page(&words, 9, CAPTION_LINE_CHARS);
        assert_eq!(page.lines.len(), 2);
        for line in &page.lines {
            let text = line.join(" ");
            if line.len() > 1 {
                assert!(text.chars().count() <= CAPTION_LINE_CHARS, "{text}");
            }
        }
        let flat: Vec<_> = page.lines.iter().flatten().cloned().collect();
        assert!(flat.iter().any(|word| word == "kyunki"));
        assert_eq!(page.lines[page.active_line][page.active_index], "kyunki");
        let long = caption_page(&["antidisestablishmentarianism"], 0, 8);
        assert_eq!(long.lines.len(), 1);
        assert_eq!(long.lines[0][0], "antidisestablishmentarianism");
    }

    #[test]
    fn cam_shapes_have_distinct_masks() {
        let side = 32u32;
        let circle = cam_mask(CamShape::Circle, side);
        let rounded = cam_mask(CamShape::Rounded, side);
        let square = cam_mask(CamShape::Square, side);
        assert_eq!(circle, cam_mask(CamShape::Circle, side));
        assert_eq!(circle[(side * side - 1) as usize], 0);
        assert_eq!(circle[(16 * side + 16) as usize], 255);
        assert_eq!(square[0], 255);
        assert_eq!(rounded[0], 0);
        assert_eq!(rounded[(16 * side + 16) as usize], 255);
        assert_ne!(circle, square);
        assert_ne!(rounded, square);
        assert_ne!(circle, rounded);
    }
}
