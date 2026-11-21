//! Take analysis: silence from the mic file, words from whisper, fillers and chapters.

use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::thread;

use core::{
    Beat, DEFAULT_FILLERS, SILENCE_MIN_SECS, TranscriptReport, chapters_from_beats,
    fillers_in_words, silences_in_pcm,
};

use crate::doctor::{whisper_model_path, whisper_model_present};
use crate::record::MIC_FILE;

pub fn queue(dir: impl Into<PathBuf>) {
    let dir = dir.into();
    thread::spawn(move || {
        let report = analyze_take(&dir, &[]);
        let _ = write_report(&dir.join("transcript.json"), &report);
    });
}

pub fn analyze_take(dir: &Path, beats: &[Beat]) -> TranscriptReport {
    let mic = dir.join(MIC_FILE);
    let (samples, sample_rate) = match read_wav_f32(&mic) {
        Ok(audio) => audio,
        Err(error) => {
            return TranscriptReport {
                status: "error".to_string(),
                message: error,
                words: Vec::new(),
                fillers: Vec::new(),
                silences: Vec::new(),
                chapters: Vec::new(),
            };
        }
    };
    let silences = silences_in_pcm(&samples, sample_rate, SILENCE_MIN_SECS);
    let model = whisper_model_path();
    if !whisper_model_present(model.parent().unwrap_or(Path::new("."))) {
        return TranscriptReport {
            status: "model-absent".to_string(),
            message: format!(
                "whisper model absent ({}) — words wait for the first download",
                model.display()
            ),
            words: Vec::new(),
            fillers: Vec::new(),
            silences,
            chapters: Vec::new(),
        };
    }

    let samples_16k = resample(&samples, sample_rate, 16_000);
    match crate::whisper::transcribe(&model, &samples_16k) {
        Ok(words) => {
            let fillers = fillers_in_words(&words, DEFAULT_FILLERS);
            let chapters = chapters_from_beats(beats, &words);
            TranscriptReport {
                status: "ok".to_string(),
                message: format!("{} words", words.len()),
                words,
                fillers,
                silences,
                chapters,
            }
        }
        Err(error) => TranscriptReport {
            status: "error".to_string(),
            message: error,
            words: Vec::new(),
            fillers: Vec::new(),
            silences,
            chapters: Vec::new(),
        },
    }
}

pub fn write_report(path: &Path, report: &TranscriptReport) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    let json = serde_json::to_string_pretty(report).map_err(|error| error.to_string())?;
    let tmp = path.with_extension("json.tmp");
    let mut file = File::create(&tmp).map_err(|error| error.to_string())?;
    file.write_all(json.as_bytes())
        .map_err(|error| error.to_string())?;
    file.sync_all().map_err(|error| error.to_string())?;
    std::fs::rename(&tmp, path).map_err(|error| error.to_string())
}

fn resample(input: &[f32], from: u32, to: u32) -> Vec<f32> {
    if from == 0 || to == 0 || from == to {
        return input.to_vec();
    }
    let out_len = (input.len() as f64 * f64::from(to) / f64::from(from)).round() as usize;
    let mut output = Vec::with_capacity(out_len);
    for index in 0..out_len {
        let source = index as f64 * f64::from(from) / f64::from(to);
        let left = source.floor() as usize;
        let fraction = (source - left as f64) as f32;
        let a = input.get(left).copied().unwrap_or(0.0);
        let b = input.get(left + 1).copied().unwrap_or(a);
        output.push(a + (b - a) * fraction);
    }
    output
}

fn read_wav_f32(path: &Path) -> Result<(Vec<f32>, u32), String> {
    let mut file = File::open(path).map_err(|error| format!("open {}: {error}", path.display()))?;
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes)
        .map_err(|error| format!("read {}: {error}", path.display()))?;
    if bytes.len() < 44 || &bytes[0..4] != b"RIFF" || &bytes[8..12] != b"WAVE" {
        return Err(format!("{} is not a wav file", path.display()));
    }
    let mut cursor = 12;
    let mut sample_rate = 0u32;
    let mut channels = 0u16;
    let mut bits = 0u16;
    let mut data: Option<&[u8]> = None;
    while cursor + 8 <= bytes.len() {
        let id = &bytes[cursor..cursor + 4];
        let size = u32::from_le_bytes(bytes[cursor + 4..cursor + 8].try_into().unwrap()) as usize;
        let start = cursor + 8;
        let end = start + size;
        if end > bytes.len() {
            return Err(format!("truncated wav chunk in {}", path.display()));
        }
        if id == b"fmt " && size >= 16 {
            let audio_format = u16::from_le_bytes(bytes[start..start + 2].try_into().unwrap());
            channels = u16::from_le_bytes(bytes[start + 2..start + 4].try_into().unwrap());
            sample_rate = u32::from_le_bytes(bytes[start + 4..start + 8].try_into().unwrap());
            bits = u16::from_le_bytes(bytes[start + 14..start + 16].try_into().unwrap());
            if audio_format != 1 {
                return Err(format!(
                    "{} is not PCM (format {audio_format})",
                    path.display()
                ));
            }
        } else if id == b"data" {
            data = Some(&bytes[start..end]);
        }
        cursor = end + (size % 2);
    }
    let data = data.ok_or_else(|| format!("{} has no data chunk", path.display()))?;
    if channels != 1 || bits != 16 {
        return Err(format!(
            "{} must be mono 16-bit PCM (got {channels} ch, {bits} bit)",
            path.display()
        ));
    }
    let mut samples = Vec::with_capacity(data.len() / 2);
    let (frames, _) = data.as_chunks::<2>();
    for frame in frames {
        let value = i16::from_le_bytes(*frame);
        samples.push(value as f32 / 32768.0);
    }
    Ok((samples, sample_rate))
}

#[cfg(test)]
fn write_wav_i16(path: &Path, samples: &[f32], sample_rate: u32) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    let mut pcm = Vec::with_capacity(samples.len() * 2);
    for sample in samples {
        let clamped = sample.clamp(-1.0, 1.0);
        let value = (clamped * 32767.0) as i16;
        pcm.extend_from_slice(&value.to_le_bytes());
    }
    let data_size = pcm.len() as u32;
    let mut bytes = Vec::new();
    bytes.extend_from_slice(b"RIFF");
    bytes.extend_from_slice(&(36 + data_size).to_le_bytes());
    bytes.extend_from_slice(b"WAVE");
    bytes.extend_from_slice(b"fmt ");
    bytes.extend_from_slice(&16u32.to_le_bytes());
    bytes.extend_from_slice(&1u16.to_le_bytes());
    bytes.extend_from_slice(&1u16.to_le_bytes());
    bytes.extend_from_slice(&sample_rate.to_le_bytes());
    bytes.extend_from_slice(&(sample_rate * 2).to_le_bytes());
    bytes.extend_from_slice(&2u16.to_le_bytes());
    bytes.extend_from_slice(&16u16.to_le_bytes());
    bytes.extend_from_slice(b"data");
    bytes.extend_from_slice(&data_size.to_le_bytes());
    bytes.extend_from_slice(&pcm);
    std::fs::write(path, bytes).map_err(|error| error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn synthetic_take_reports_silence_without_a_model() {
        let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../target/m07-transcript")
            .join(std::process::id().to_string());
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let sample_rate = 16_000u32;
        let mut samples = vec![0.0f32; sample_rate as usize * 2];
        for sample in &mut samples[..sample_rate as usize / 5] {
            *sample = 0.4;
        }
        write_wav_i16(&dir.join(MIC_FILE), &samples, sample_rate).unwrap();
        let report = analyze_take(&dir, &[]);
        assert!(!report.silences.is_empty());
        assert!(report.silences[0].end - report.silences[0].start >= SILENCE_MIN_SECS);
        if !whisper_model_present(whisper_model_path().parent().unwrap()) {
            assert_eq!(report.status, "model-absent");
        }
        let _ = std::fs::remove_dir_all(&dir);
    }
}
