//! whisper-rs word timestamps. The model file is optional until the first real take.

use core::WordStamp;
use std::path::Path;
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};

pub fn transcribe(model: &Path, samples_16k: &[f32]) -> Result<Vec<WordStamp>, String> {
    let mut context_params = WhisperContextParameters::default();
    context_params.use_gpu(true);
    let context = WhisperContext::new_with_params(model, context_params)
        .map_err(|error| format!("load whisper model: {error}"))?;
    let mut state = context
        .create_state()
        .map_err(|error| format!("whisper state: {error}"))?;

    let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
    params.set_n_threads(4);
    params.set_translate(false);
    params.set_language(None);
    params.set_print_special(false);
    params.set_print_progress(false);
    params.set_print_realtime(false);
    params.set_print_timestamps(false);
    // whisper-rs 0.16 still exposes split_on_word as the word-boundary path.
    // max_len(1) makes each segment one word once token timestamps are on.
    params.set_token_timestamps(true);
    params.set_split_on_word(true);
    params.set_max_len(1);

    state
        .full(params, samples_16k)
        .map_err(|error| format!("whisper full: {error}"))?;

    let mut words = Vec::new();
    let segments = state.full_n_segments();
    for index in 0..segments {
        let Some(segment) = state.get_segment(index) else {
            continue;
        };
        let text = segment
            .to_str()
            .map_err(|error| format!("whisper segment: {error}"))?;
        let text = text.trim();
        if text.is_empty() || text.starts_with('[') || text.starts_with("<|") {
            continue;
        }
        // whisper.cpp reports segment bounds in centiseconds.
        let start = segment.start_timestamp() as f64 / 100.0;
        let end = segment.end_timestamp() as f64 / 100.0;
        words.push(WordStamp {
            text: text.to_string(),
            start,
            end,
        });
    }
    Ok(words)
}
