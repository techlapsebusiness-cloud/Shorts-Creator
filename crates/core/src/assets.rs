//! Cover templates and the 3 second end card.
//!
//! Text is measured here so a long hook stays inside 1080×1920. Skia paints
//! the PNG from these boxes.

use crate::EndCardStyle;

pub const COVER_W: f64 = 1080.0;
pub const COVER_H: f64 = 1920.0;
pub const END_CARD_SECS: f64 = 3.0;
pub const BURN_SECS: f64 = 0.5;
pub const TAGLINE: &str = "BUILD. LEARN. REPEAT.";
pub const OPEN_MOUTH: f64 = 0.35;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FaceBox {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub mouth_open: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FittedText {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub font_px: f64,
    pub lines: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EndBeat {
    Chevron,
    Next,
    Follow,
    Tagline,
}

pub fn best_face(faces: &[FaceBox]) -> Option<usize> {
    let open: Vec<usize> = faces
        .iter()
        .enumerate()
        .filter(|(_, face)| face.mouth_open >= OPEN_MOUTH)
        .map(|(index, _)| index)
        .collect();
    let pool: Vec<usize> = if open.is_empty() {
        (0..faces.len()).collect()
    } else {
        open
    };
    pool.into_iter()
        .max_by(|left, right| face_area(&faces[*left]).total_cmp(&face_area(&faces[*right])))
}

pub fn key_code_line(lines: &[&str]) -> Option<String> {
    lines
        .iter()
        .copied()
        .filter(|line| code_score(line) > 0)
        .max_by_key(|line| code_score(line))
        .map(str::to_string)
}

pub fn fit_text(text: &str, x: f64, y: f64, max_w: f64, max_h: f64) -> FittedText {
    let words: Vec<&str> = if text.split_whitespace().next().is_none() {
        vec![""]
    } else {
        text.split_whitespace().collect()
    };
    let mut font = 72.0;
    loop {
        let char_w = font * 0.56;
        let line_h = font * 1.2;
        let max_chars = ((max_w / char_w).floor() as usize).max(1);
        let lines = wrap_words(&words, max_chars);
        let used = lines.len() as f64 * line_h;
        if used <= max_h || font <= 16.0 {
            let kept = ((max_h / line_h).floor() as usize).clamp(1, lines.len());
            let lines = lines[..kept].to_vec();
            let width = lines
                .iter()
                .map(|line| line.chars().count() as f64 * char_w)
                .fold(0.0, f64::max)
                .min(max_w);
            let height = (lines.len() as f64 * line_h).min(max_h);
            return FittedText {
                x,
                y,
                width,
                height,
                font_px: font,
                lines,
            };
        }
        font -= 2.0;
    }
}

pub fn hook_fits(text: &str) -> bool {
    let fitted = fit_text(text, 80.0, 160.0, 920.0, 700.0);
    box_inside_cover(&fitted)
}

pub fn end_beat(time: f64) -> EndBeat {
    if time < 0.45 {
        EndBeat::Chevron
    } else if time < 1.05 {
        EndBeat::Next
    } else if time < 1.8 {
        EndBeat::Follow
    } else {
        EndBeat::Tagline
    }
}

pub fn follow_scale(time: f64) -> f64 {
    if !(1.05..1.8).contains(&time) {
        return 1.0;
    }
    let unit = (time - 1.05) / 0.75;
    1.0 + 0.15 * (unit * std::f64::consts::TAU * 2.0).sin().max(0.0)
}

pub fn end_offset(style: EndCardStyle, time: f64) -> (f64, f64) {
    let unit = (time / END_CARD_SECS).clamp(0.0, 1.0);
    match style {
        EndCardStyle::Rise => (0.0, (1.0 - unit) * 120.0),
        EndCardStyle::Glitch => ((time * 17.0).sin() * 10.0, 0.0),
        EndCardStyle::Slide => ((1.0 - unit) * 240.0, 0.0),
        EndCardStyle::Terminal => (0.0, 0.0),
    }
}

pub fn terminal_reveal(time: f64) -> f64 {
    (time / END_CARD_SECS).clamp(0.0, 1.0)
}

pub fn next_title<'a>(queue: &'a [String], current: &str) -> Option<&'a str> {
    queue
        .iter()
        .find(|topic| topic.as_str() != current)
        .map(String::as_str)
}

pub fn burn_frames(fps: u32) -> u32 {
    (f64::from(fps) * BURN_SECS).round() as u32
}

pub fn paint_end_card(time: f64, style: EndCardStyle) -> Vec<u8> {
    let mut pixels = vec![0u8; 8 * 8 * 4];
    let column = match end_beat(time) {
        EndBeat::Chevron => 1,
        EndBeat::Next => 3,
        EndBeat::Follow => 5,
        EndBeat::Tagline => 7,
    };
    stamp(&mut pixels, column, 1, 255);
    let style_column = match style {
        EndCardStyle::Rise => 1,
        EndCardStyle::Glitch => 3,
        EndCardStyle::Slide => 5,
        EndCardStyle::Terminal => 7,
    };
    stamp(&mut pixels, style_column, 6, 180);
    pixels
}

fn box_inside_cover(text: &FittedText) -> bool {
    text.x >= 0.0
        && text.y >= 0.0
        && text.x + text.width <= COVER_W + 0.5
        && text.y + text.height <= COVER_H + 0.5
        && text.width > 0.0
        && text.height > 0.0
}

fn face_area(face: &FaceBox) -> f64 {
    face.width.max(0.0) * face.height.max(0.0)
}

fn code_score(line: &str) -> i32 {
    let symbols = line.chars().filter(|ch| "{}();=<>[]".contains(*ch)).count() as i32;
    if symbols == 0 {
        0
    } else {
        symbols * 10 + line.chars().count() as i32
    }
}

fn wrap_words(words: &[&str], max_chars: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current = String::new();
    for word in words {
        for piece in split_word(word, max_chars) {
            let next = if current.is_empty() {
                piece.clone()
            } else {
                format!("{current} {piece}")
            };
            if next.chars().count() <= max_chars {
                current = next;
            } else {
                if !current.is_empty() {
                    lines.push(std::mem::take(&mut current));
                }
                current = piece;
            }
        }
    }
    if !current.is_empty() {
        lines.push(current);
    }
    if lines.is_empty() {
        lines.push(String::new());
    }
    lines
}

fn split_word(word: &str, max_chars: usize) -> Vec<String> {
    let chars: Vec<char> = word.chars().collect();
    if chars.len() <= max_chars {
        return vec![word.to_string()];
    }
    chars
        .chunks(max_chars)
        .map(|chunk| chunk.iter().collect())
        .collect()
}

fn stamp(pixels: &mut [u8], x: usize, y: usize, value: u8) {
    let index = (y * 8 + x) * 4;
    pixels[index] = value;
    pixels[index + 1] = value;
    pixels[index + 2] = value;
    pixels[index + 3] = 255;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_open_face_wins_and_the_code_line_is_the_densest() {
        let faces = [
            FaceBox {
                x: 0.0,
                y: 0.0,
                width: 400.0,
                height: 400.0,
                mouth_open: 0.1,
            },
            FaceBox {
                x: 10.0,
                y: 10.0,
                width: 180.0,
                height: 180.0,
                mouth_open: 0.8,
            },
        ];
        assert_eq!(best_face(&faces), Some(1));
        let quiet = [faces[0]];
        assert_eq!(best_face(&quiet), Some(0));
        let line = key_code_line(&["hello", "let x = 1;", "fn main() { return 1; }"]);
        assert_eq!(line.as_deref(), Some("fn main() { return 1; }"));
    }

    #[test]
    fn short_and_long_hooks_stay_inside_the_cover() {
        assert!(hook_fits("This file survives"));
        let long = "rename failed ".repeat(40);
        assert!(hook_fits(&long));
        let short = fit_text("Hi", 80.0, 160.0, 920.0, 700.0);
        let wide = fit_text(&long, 80.0, 160.0, 920.0, 700.0);
        assert!(wide.lines.len() > short.lines.len());
        assert!(short.font_px >= wide.font_px);
    }

    #[test]
    fn end_card_goldens_at_the_four_times() {
        let times = [0.0, 0.6, 1.3, 2.0];
        let beats = [
            EndBeat::Chevron,
            EndBeat::Next,
            EndBeat::Follow,
            EndBeat::Tagline,
        ];
        let columns = [1usize, 3, 5, 7];
        for ((time, beat), column) in times.iter().zip(beats).zip(columns) {
            assert_eq!(end_beat(*time), beat);
            let frame = paint_end_card(*time, EndCardStyle::Rise);
            assert_eq!(&frame[(8 + column) * 4..][..4], &[255, 255, 255, 255]);
        }
        let rise = paint_end_card(0.6, EndCardStyle::Rise);
        let slide = paint_end_card(0.6, EndCardStyle::Slide);
        assert_ne!(rise, slide);
        assert!(follow_scale(1.14) > 1.05);
        assert!(follow_scale(1.52) > 1.05);
        assert!((follow_scale(1.3) - 1.0).abs() < 0.02);
        let queue = vec!["Rename".to_string(), "Exposure".to_string()];
        assert_eq!(next_title(&queue, "Rename"), Some("Exposure"));
        assert_eq!(burn_frames(30), 15);
        assert_eq!(TAGLINE, "BUILD. LEARN. REPEAT.");
    }
}
