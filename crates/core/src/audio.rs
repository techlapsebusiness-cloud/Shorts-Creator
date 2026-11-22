//! Voice chain, sidechain ducking, and click-free word cuts.
//!
//! Loudness numbers match the mix target. The ffmpeg two-pass that hits them
//! lives in the media crate.

use crate::AudioChain;

pub const TARGET_LUFS: f64 = -14.0;
pub const TARGET_TRUE_PEAK_DB: f64 = -1.0;
pub const LOUDNESS_TOLERANCE_LU: f64 = 0.5;
pub const DUCK_ATTACK_MS: f64 = 80.0;
pub const DUCK_RELEASE_MS: f64 = 400.0;
pub const DUCK_REDUCTION_DB: f64 = -12.0;
pub const CROSSFADE_MS: f64 = 20.0;
pub const CLICK_DELTA: f32 = 0.05;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NoiseReducer {
    Arnndn,
    Afftdn,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MixerTrack {
    pub gain_db: f64,
    pub mute: bool,
    pub solo: bool,
}

pub fn loudness_ok(integrated_lufs: f64, true_peak_db: f64) -> bool {
    (integrated_lufs - TARGET_LUFS).abs() <= LOUDNESS_TOLERANCE_LU
        && true_peak_db <= TARGET_TRUE_PEAK_DB + 1e-3
}

pub fn duck_gains(
    voice_active: &[bool],
    sample_rate: u32,
    attack_ms: f64,
    release_ms: f64,
    reduction_db: f64,
) -> Vec<f64> {
    let target = 10f64.powf(reduction_db / 20.0);
    let attack = (f64::from(sample_rate) * attack_ms / 1000.0).max(1.0);
    let release = (f64::from(sample_rate) * release_ms / 1000.0).max(1.0);
    let down = (1.0 - target) / attack;
    let up = (1.0 - target) / release;
    let mut gain = 1.0;
    voice_active
        .iter()
        .map(|active| {
            if *active {
                gain = (gain - down).max(target);
            } else {
                gain = (gain + up).min(1.0);
            }
            gain
        })
        .collect()
}

pub fn voice_active(samples: &[f32], threshold: f32) -> Vec<bool> {
    samples
        .iter()
        .map(|sample| sample.abs() >= threshold)
        .collect()
}

/// Drop `start..end` and crossfade the join so the sample step stays small.
pub fn splice_crossfade(samples: &[f32], start: usize, end: usize, fade: usize) -> Vec<f32> {
    let start = start.min(samples.len());
    let end = end.clamp(start, samples.len());
    let left = &samples[..start];
    let right = &samples[end..];
    let fade = fade.min(left.len()).min(right.len());
    if fade < 2 {
        let mut out = Vec::with_capacity(left.len() + right.len());
        out.extend_from_slice(left);
        out.extend_from_slice(right);
        return out;
    }
    let mut out = Vec::with_capacity(left.len() + right.len() - fade);
    out.extend_from_slice(&left[..left.len() - fade]);
    for index in 0..fade {
        let t = index as f32 / (fade - 1) as f32;
        let mixed = left[left.len() - fade + index] * (1.0 - t) + right[index] * t;
        out.push(mixed);
    }
    out.extend_from_slice(&right[fade..]);
    out
}

pub fn max_abs_step(samples: &[f32]) -> f32 {
    samples
        .windows(2)
        .map(|pair| (pair[1] - pair[0]).abs())
        .fold(0.0, f32::max)
}

pub fn track_linear_gain(tracks: &[MixerTrack], index: usize) -> f64 {
    let Some(track) = tracks.get(index) else {
        return 0.0;
    };
    if track.mute {
        return 0.0;
    }
    if tracks.iter().any(|item| item.solo) && !track.solo {
        return 0.0;
    }
    10f64.powf(track.gain_db / 20.0)
}

pub fn voice_filters(chain: &AudioChain, noise: NoiseReducer) -> String {
    let mut filters = Vec::new();
    if chain.noise.enabled {
        let amount = chain.noise.amount.clamp(0.0, 1.0);
        match noise {
            NoiseReducer::Arnndn => filters.push(format!("arnndn=m={amount:.3}")),
            NoiseReducer::Afftdn => {
                let nr = 6.0 + amount * 18.0;
                filters.push(format!("afftdn=nr={nr:.1}"));
            }
        }
    }
    if chain.eq.enabled {
        let hz = 70.0 + chain.eq.amount.clamp(0.0, 1.0) * 80.0;
        filters.push(format!("highpass=f={hz:.0}"));
    }
    if chain.compressor.enabled {
        let amount = chain.compressor.amount.clamp(0.0, 1.0);
        let threshold = -18.0 - amount * 12.0;
        filters.push(format!(
            "acompressor=threshold={threshold:.1}dB:ratio=3:attack=10:release=80"
        ));
    }
    if chain.deesser.enabled {
        let cut = -2.0 - chain.deesser.amount.clamp(0.0, 1.0) * 8.0;
        filters.push(format!("treble=g={cut:.1}:f=6000"));
    }
    if filters.is_empty() {
        "anull".to_string()
    } else {
        filters.join(",")
    }
}

pub fn preview_filters(chain: &AudioChain, noise: NoiseReducer, wet: bool) -> String {
    if wet {
        voice_filters(chain, noise)
    } else {
        "anull".to_string()
    }
}

pub fn crossfade_samples(sample_rate: u32) -> usize {
    ((f64::from(sample_rate) * CROSSFADE_MS / 1000.0).round() as usize).max(2)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn music_drops_by_the_duck_amount_while_voice_is_present() {
        let rate = 48_000u32;
        let silence = (rate as usize) / 10;
        let voice = (rate as usize) / 5;
        let mut active = vec![false; silence];
        active.extend(std::iter::repeat_n(true, voice));
        active.extend(std::iter::repeat_n(false, rate as usize));
        let gains = duck_gains(
            &active,
            rate,
            DUCK_ATTACK_MS,
            DUCK_RELEASE_MS,
            DUCK_REDUCTION_DB,
        );
        let attack = (f64::from(rate) * DUCK_ATTACK_MS / 1000.0) as usize;
        let ducked = gains[silence + attack - 1];
        let ducked_db = 20.0 * ducked.log10();
        assert!((ducked_db - DUCK_REDUCTION_DB).abs() < 0.5, "{ducked_db}");
        assert!(20.0 * gains[silence / 2].log10() > -1.0);
        let release = (f64::from(rate) * DUCK_RELEASE_MS / 1000.0) as usize;
        let recovered = gains[silence + voice + release];
        assert!(20.0 * recovered.log10() > -1.0);
    }

    #[test]
    fn a_word_cut_crossfade_stays_under_the_click_threshold() {
        let mut samples = vec![0.8f32; 4_000];
        samples.extend(std::iter::repeat_n(-0.8, 2_000));
        let fade = crossfade_samples(48_000);
        let cut = splice_crossfade(&samples, 2_000, 4_000, fade);
        assert!(max_abs_step(&cut) < CLICK_DELTA);
        let hard = splice_crossfade(&samples, 2_000, 4_000, 0);
        assert!(max_abs_step(&hard) > 1.0);
    }

    #[test]
    fn bypass_removes_a_stage_and_dry_preview_is_silent_processing() {
        let mut chain = AudioChain::default();
        chain.noise.enabled = true;
        chain.noise.amount = 0.5;
        chain.eq.enabled = true;
        chain.eq.amount = 0.25;
        let wet = voice_filters(&chain, NoiseReducer::Afftdn);
        assert!(wet.contains("afftdn=nr=15.0"));
        assert!(wet.contains("highpass=f=90"));
        chain.noise.enabled = false;
        assert!(!voice_filters(&chain, NoiseReducer::Afftdn).contains("afftdn"));
        assert_eq!(
            preview_filters(&chain, NoiseReducer::Afftdn, false),
            "anull"
        );
        let solo_music = [
            MixerTrack {
                gain_db: 0.0,
                mute: false,
                solo: false,
            },
            MixerTrack {
                gain_db: -6.0,
                mute: false,
                solo: true,
            },
        ];
        assert_eq!(track_linear_gain(&solo_music, 0), 0.0);
        assert!((track_linear_gain(&solo_music, 1) - 10f64.powf(-6.0 / 20.0)).abs() < 1e-9);
    }
}
