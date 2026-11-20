//! Project model and timeline maths.

pub const NAME: &str = "core";

pub const SCHEMA_VERSION: u32 = 1;

mod migrate;
mod order;
mod project;
mod prompter;
mod script;

pub use migrate::{MigrateError, migrate_project};
pub use order::{TimelineSpan, normalize_spans, spans_are_ordered};
pub use project::{
    AiMetadata, Assets, AudioChain, AudioStage, Beat, BeatKind, CamPosition, CamShape, CamSize,
    Clip, ClipBody, CoverTemplate, Ducking, Easing, EndCardStyle, ExportSettings, Keyframe,
    KeyframeProperty, Layout, LayoutPreset, Loudness, Project, SlugError, TakeSync, Track, TrackId,
    VideoCodec, WordStamp, example_project, project_folder_name, slugify,
};
pub use prompter::{
    CHUNK_MAX_MS, CHUNK_MIN_MS, COUNTDOWN_MS, Follow, Heard, MIN_CONFIDENCE, MIN_MATCH, NUDGE_PX,
    PrompterError, ScrollInput, SpokenLine, chunk_ready, countdown_label, follow, scroll_offset_px,
};
pub use script::{
    ScriptError, ScriptRules, estimate_seconds, replace_beat_text, script_seconds, spoken_text,
    validate_beats, word_count,
};
