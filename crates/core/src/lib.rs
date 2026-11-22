//! Project model and timeline maths.

pub const NAME: &str = "core";

pub const SCHEMA_VERSION: u32 = 1;

mod assets;
mod audio;
mod daily;
mod edit;
mod layout;
mod migrate;
mod order;
mod preflight;
mod project;
mod prompter;
mod render;
mod script;
mod transcript;
mod upload;

pub use assets::{
    BURN_SECS, COVER_H, COVER_W, END_CARD_SECS, EndBeat, FaceBox, FittedText, OPEN_MOUTH, TAGLINE,
    best_face, burn_frames, end_beat, end_offset, fit_text, follow_scale, hook_fits, key_code_line,
    next_title, paint_end_card, terminal_reveal,
};
pub use audio::{
    CLICK_DELTA, CROSSFADE_MS, DUCK_ATTACK_MS, DUCK_REDUCTION_DB, DUCK_RELEASE_MS,
    LOUDNESS_TOLERANCE_LU, MixerTrack, NoiseReducer, TARGET_LUFS, TARGET_TRUE_PEAK_DB,
    crossfade_samples, duck_gains, loudness_ok, max_abs_step, preview_filters, splice_crossfade,
    track_linear_gain, voice_active, voice_filters,
};
pub use daily::{
    Actor, AutoEdit, DailyStep, FirstRun, StudioFault, auto_edit, captions_srt, classify_fault,
    daily_steps, explain, filler_count, first_run_ready, next_pause, pauses,
};
pub use edit::{
    EditError, History, Marker, PROXY_HEIGHT, PROXY_WIDTH, SPEED_MAX, SPEED_MIN, TransitionKind,
    UNDO_LIMIT, clamp_speed, move_span, proxy_file_name, ripple_delete, snap_time, split_clips,
    split_spans, toggle_selection, total_duration_ms, trim_span, waveform_peaks,
};
pub use layout::{
    CAPTION_LINE_CHARS, CROP_OMEGA, CaptionPage, CropFollow, MAX_CROP_STEP_PX, Rect, SafeInsets,
    cam_mask, cam_rect, cam_side, caption_page, caption_rect, crop_size, crop_target, frame_margin,
    safe_rect, shorts_insets, step_crop, zoom_at,
};
pub use migrate::{MigrateError, migrate_project};
pub use order::{TimelineSpan, normalize_spans, spans_are_ordered};
pub use preflight::{
    CHANGE_WARN, Check, CheckState, PastShort, PolicyPack, Preflight, ShotCheck, TRIGRAM_WARN,
    blocks, bundled_policy, change_ratio, preflight, trigram_jaccard,
};
pub use project::{
    AiMetadata, Assets, AudioChain, AudioStage, Beat, BeatKind, CamPosition, CamShape, CamSize,
    Clip, ClipBody, CoverTemplate, Ducking, Easing, EndCardStyle, ExportSettings, Keyframe,
    KeyframeProperty, Layout, LayoutPreset, Loudness, Project, SlugError, TakeSync, Track, TrackId,
    VideoCodec, WordStamp, example_project, project_folder_name, slugify,
};
pub use prompter::{
    CaptureFrame, FOLLOW_CONFIDENCE, LineMatch, PxRect, ScrollMode, allow_linux_capture,
    follow_offset_px, frame_contains_magenta, fuzzy_match, nudge_offset, rects_overlap,
    scroll_mode, scroll_offset_px,
};
pub use render::{
    AUDIO_BITRATE_KBPS, AUDIO_RATE, BLUR_SAMPLES_MAX, BLUR_SAMPLES_MIN, PlacedRect, RenderProgress,
    SIZE_4K, SIZE_1080, SIZE_1440, average_rgba, blend_frames, blur_samples, downscale_box, ease,
    eta_secs, fraction, frame_rects, hash_frame, output_size, overlay_sample_count, paint_rects,
    placed_rect, sample_times, select_encoder, smooth_cursor, ssim_rgba, video_bitrate_kbps,
};
pub use script::{
    DEFAULT_WPM, HOOK_LIMIT_SECS, SCRIPT_PROMPT, ScriptDraft, ScriptError, apply_to_project,
    estimate_duration_secs, fixture_script, parse_script, prompt_for, replace_beat, spoken_text,
    validate_script, word_count,
};
pub use transcript::{
    Chapter, DEFAULT_FILLERS, FillerHit, SILENCE_MIN_SECS, SilenceSpan, TranscriptReport,
    WORD_TOLERANCE_SECS, chapters_from_beats, fillers_in_words, normalize_token, shift_words,
    silences_in_pcm, words_within,
};
pub use upload::{
    AccessToken, STUDIO_URL, TITLE_LIMIT, UNVERIFIED_NOTICE, UploadKit, UploadPolicy,
    authorize_url, build_kit, caption_body, clean_tags, direct_upload_allowed, exchange_code,
    insert_body, keyword_title, kit_description, plain_text, reminder_due, thumbnail_body,
    upload_policy,
};
