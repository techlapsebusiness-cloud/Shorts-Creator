//! Project model and timeline maths.

pub const NAME: &str = "core";

pub const SCHEMA_VERSION: u32 = 1;

mod edit;
mod migrate;
mod order;
mod project;
mod transcript;

pub use edit::{
    EditError, History, Marker, PROXY_HEIGHT, PROXY_WIDTH, SPEED_MAX, SPEED_MIN, TransitionKind,
    UNDO_LIMIT, clamp_speed, move_span, proxy_file_name, ripple_delete, snap_time, split_clips,
    split_spans, toggle_selection, total_duration_ms, trim_span, waveform_peaks,
};
pub use migrate::{MigrateError, migrate_project};
pub use order::{TimelineSpan, normalize_spans, spans_are_ordered};
pub use project::{
    AiMetadata, Assets, AudioChain, AudioStage, Beat, BeatKind, CamPosition, CamShape, CamSize,
    Clip, ClipBody, CoverTemplate, Ducking, Easing, EndCardStyle, ExportSettings, Keyframe,
    KeyframeProperty, Layout, LayoutPreset, Loudness, Project, SlugError, TakeSync, Track, TrackId,
    VideoCodec, WordStamp, example_project, project_folder_name, slugify,
};
pub use transcript::{
    Chapter, DEFAULT_FILLERS, FillerHit, SILENCE_MIN_SECS, SilenceSpan, TranscriptReport,
    WORD_TOLERANCE_SECS, chapters_from_beats, fillers_in_words, normalize_token, shift_words,
    silences_in_pcm, words_within,
};
