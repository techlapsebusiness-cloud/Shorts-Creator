//! Project model and timeline maths.

pub const NAME: &str = "core";

pub const SCHEMA_VERSION: u32 = 1;

mod migrate;
mod order;
mod project;
mod script;

pub use migrate::{MigrateError, migrate_project};
pub use order::{TimelineSpan, normalize_spans, spans_are_ordered};
pub use project::{
    AiMetadata, Assets, AudioChain, AudioStage, Beat, BeatKind, CamPosition, CamShape, CamSize,
    Clip, ClipBody, CoverTemplate, Ducking, Easing, EndCardStyle, ExportSettings, Keyframe,
    KeyframeProperty, Layout, LayoutPreset, Loudness, Project, SlugError, TakeSync, Track, TrackId,
    VideoCodec, WordStamp, example_project, project_folder_name, slugify,
};
pub use script::{
    ScriptError, ScriptRules, estimate_seconds, replace_beat_text, script_seconds, spoken_text,
    validate_beats, word_count,
};
