use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::SCHEMA_VERSION;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SlugError;

pub fn slugify(name: &str) -> Result<String, SlugError> {
    let mut slug = String::new();
    let mut pending_dash = false;
    for character in name.chars() {
        if character.is_ascii_alphanumeric() {
            if pending_dash && !slug.is_empty() {
                slug.push('-');
            }
            pending_dash = false;
            slug.push(character.to_ascii_lowercase());
        } else if !slug.is_empty() {
            pending_dash = true;
        }
    }
    if slug.is_empty() {
        Err(SlugError)
    } else {
        Ok(slug)
    }
}

pub fn project_folder_name(number: u32, slug: &str) -> String {
    format!("{number:03}-{slug}")
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct Project {
    pub version: u32,
    pub id: String,
    pub number: u32,
    pub name: String,
    pub slug: String,
    pub tracks: Vec<Track>,
    pub layout: Layout,
    pub audio_chain: AudioChain,
    pub assets: Assets,
    pub export: ExportSettings,
    pub ai: AiMetadata,
    pub sync: TakeSync,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct Track {
    pub id: TrackId,
    pub clips: Vec<Clip>,
    #[serde(default, skip_serializing_if = "is_false")]
    pub muted: bool,
    #[serde(default, skip_serializing_if = "is_false")]
    pub solo: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
pub enum TrackId {
    #[serde(rename = "V3")]
    V3Fx,
    #[serde(rename = "V2")]
    V2Cam,
    #[serde(rename = "V1")]
    V1Screen,
    #[serde(rename = "TXT")]
    Txt,
    #[serde(rename = "A1")]
    A1Voice,
    #[serde(rename = "A2")]
    A2Music,
    #[serde(rename = "A3")]
    A3Sfx,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct Clip {
    pub id: String,
    pub timeline_start: f64,
    pub duration: f64,
    pub body: ClipBody,
    pub keyframes: Vec<Keyframe>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum ClipBody {
    Video {
        source: String,
        source_in: f64,
        source_out: f64,
        speed: f64,
    },
    Audio {
        source: String,
        source_in: f64,
        source_out: f64,
        speed: f64,
    },
    Caption {
        text: String,
        words: Vec<WordStamp>,
    },
    Overlay {
        label: String,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct WordStamp {
    pub text: String,
    pub start: f64,
    pub end: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct Keyframe {
    pub time: f64,
    pub property: KeyframeProperty,
    pub value: f64,
    pub easing: Easing,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub enum KeyframeProperty {
    Zoom,
    X,
    Y,
    Opacity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub enum Easing {
    Linear,
    EaseInOut,
    Spring,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct Layout {
    pub preset: LayoutPreset,
    pub cam_position: CamPosition,
    pub cam_size: CamSize,
    pub cam_shape: CamShape,
    pub safe_zone: bool,
    pub accent: String,
    #[serde(default = "default_transition", skip_serializing_if = "is_cut")]
    pub transition: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub markers_secs: Vec<f64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub enum LayoutPreset {
    ScreenCam,
    CamOnly,
    ScreenOnly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub enum CamPosition {
    TopLeft,
    TopCenter,
    TopRight,
    MiddleLeft,
    MiddleCenter,
    MiddleRight,
    BottomLeft,
    BottomCenter,
    BottomRight,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
pub enum CamSize {
    #[serde(rename = "S")]
    S,
    #[serde(rename = "M")]
    M,
    #[serde(rename = "L")]
    L,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub enum CamShape {
    Circle,
    Rounded,
    Square,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct AudioChain {
    pub noise: AudioStage,
    pub eq: AudioStage,
    pub compressor: AudioStage,
    pub deesser: AudioStage,
    pub loudness: Loudness,
    pub ducking: Ducking,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct AudioStage {
    pub enabled: bool,
    pub amount: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct Loudness {
    pub enabled: bool,
    pub integrated_lufs: f64,
    pub true_peak_db: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct Ducking {
    pub enabled: bool,
    pub attack_ms: f64,
    pub release_ms: f64,
    pub reduction_db: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct Assets {
    pub cover_template: CoverTemplate,
    pub end_card_style: EndCardStyle,
    pub add_end_card: bool,
    pub burn_cover: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub enum CoverTemplate {
    HookFace,
    ErrorHook,
    BeforeAfter,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub enum EndCardStyle {
    Rise,
    Glitch,
    Slide,
    Terminal,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct ExportSettings {
    pub width: u32,
    pub height: u32,
    pub fps: u32,
    pub codec: VideoCodec,
    pub video_bitrate_kbps: u32,
    pub audio_bitrate_kbps: u32,
    pub easing: Easing,
    pub animate_at_60: bool,
    pub motion_blur: bool,
    pub smooth_cursor: bool,
    pub frame_blend: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
pub enum VideoCodec {
    #[serde(rename = "h264")]
    H264,
    #[serde(rename = "hevc")]
    Hevc,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct AiMetadata {
    pub provider: String,
    pub model: String,
    pub topic: String,
    pub language: String,
    pub raw_draft: String,
    pub beats: Vec<Beat>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct Beat {
    pub kind: BeatKind,
    pub start: f64,
    pub text: String,
    pub cues: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub enum BeatKind {
    Hook,
    Concept,
    Demo,
    Fix,
    Cta,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct TakeSync {
    pub screen_offset_ms: f64,
    pub cam_offset_ms: f64,
    pub mic_offset_ms: f64,
}

impl Project {
    pub fn new(
        id: impl Into<String>,
        number: u32,
        name: impl Into<String>,
        slug: impl Into<String>,
    ) -> Self {
        Self {
            version: SCHEMA_VERSION,
            id: id.into(),
            number,
            name: name.into(),
            slug: slug.into(),
            tracks: default_tracks(),
            layout: Layout::default(),
            audio_chain: AudioChain::default(),
            assets: Assets::default(),
            export: ExportSettings::default(),
            ai: AiMetadata::default(),
            sync: TakeSync::default(),
        }
    }
}

impl Default for Layout {
    fn default() -> Self {
        Self {
            preset: LayoutPreset::ScreenCam,
            cam_position: CamPosition::BottomRight,
            cam_size: CamSize::M,
            cam_shape: CamShape::Circle,
            safe_zone: true,
            accent: "#FF7A3D".to_string(),
            transition: default_transition(),
            markers_secs: Vec::new(),
        }
    }
}

fn default_transition() -> String {
    "cut".to_string()
}

fn is_cut(value: &str) -> bool {
    value == "cut"
}

fn is_false(value: &bool) -> bool {
    !*value
}

impl Default for AudioStage {
    fn default() -> Self {
        Self {
            enabled: false,
            amount: 0.0,
        }
    }
}

impl Default for AudioChain {
    fn default() -> Self {
        Self {
            noise: AudioStage::default(),
            eq: AudioStage::default(),
            compressor: AudioStage::default(),
            deesser: AudioStage::default(),
            loudness: Loudness {
                enabled: false,
                integrated_lufs: -14.0,
                true_peak_db: -1.0,
            },
            ducking: Ducking {
                enabled: false,
                attack_ms: 80.0,
                release_ms: 400.0,
                reduction_db: -12.0,
            },
        }
    }
}

impl Default for Assets {
    fn default() -> Self {
        Self {
            cover_template: CoverTemplate::HookFace,
            end_card_style: EndCardStyle::Rise,
            add_end_card: true,
            burn_cover: false,
        }
    }
}

impl Default for ExportSettings {
    fn default() -> Self {
        Self {
            width: 2160,
            height: 3840,
            fps: 30,
            codec: VideoCodec::H264,
            video_bitrate_kbps: 40_000,
            audio_bitrate_kbps: 384,
            easing: Easing::EaseInOut,
            animate_at_60: true,
            motion_blur: false,
            smooth_cursor: true,
            frame_blend: true,
        }
    }
}

impl Default for AiMetadata {
    fn default() -> Self {
        Self {
            provider: String::new(),
            model: String::new(),
            topic: String::new(),
            language: "hinglish".to_string(),
            raw_draft: String::new(),
            beats: Vec::new(),
        }
    }
}

impl Default for TakeSync {
    fn default() -> Self {
        Self {
            screen_offset_ms: 0.0,
            cam_offset_ms: 0.0,
            mic_offset_ms: 0.0,
        }
    }
}

pub fn default_tracks() -> Vec<Track> {
    [
        TrackId::V3Fx,
        TrackId::V2Cam,
        TrackId::V1Screen,
        TrackId::Txt,
        TrackId::A1Voice,
        TrackId::A2Music,
        TrackId::A3Sfx,
    ]
    .into_iter()
    .map(|id| Track {
        id,
        clips: Vec::new(),
        muted: false,
        solo: false,
    })
    .collect()
}

pub fn example_project() -> Project {
    let mut project = Project::new("proj-example", 1, "Example short", "example-short");
    project.ai.topic = "crash-safe saves".to_string();
    project.ai.raw_draft = "Hook. Show the save. CTA.".to_string();
    project.ai.beats = vec![Beat {
        kind: BeatKind::Hook,
        start: 0.0,
        text: "This file survives a crash.".to_string(),
        cues: vec!["CUE: point at the folder".to_string()],
    }];
    let screen = project
        .tracks
        .iter_mut()
        .find(|track| track.id == TrackId::V1Screen)
        .expect("screen track");
    screen.clips.push(Clip {
        id: "clip-screen".to_string(),
        timeline_start: 0.0,
        duration: 2.5,
        body: ClipBody::Video {
            source: "media/screen.mov".to_string(),
            source_in: 0.0,
            source_out: 2.5,
            speed: 1.0,
        },
        keyframes: vec![Keyframe {
            time: 0.0,
            property: KeyframeProperty::Zoom,
            value: 1.0,
            easing: Easing::EaseInOut,
        }],
    });
    project
}

#[cfg(test)]
mod tests {
    use super::{SlugError, example_project, project_folder_name, slugify};
    use crate::migrate_project;
    use serde_json::Value;

    #[test]
    fn slugify_lowers_and_dashes() {
        assert_eq!(slugify("Hello World!").expect("slug"), "hello-world");
        assert_eq!(slugify("Fix #1").expect("slug"), "fix-1");
        assert_eq!(slugify("   "), Err(SlugError));
    }

    #[test]
    fn folder_name_is_padded() {
        assert_eq!(project_folder_name(1, "hello"), "001-hello");
    }

    #[test]
    fn example_round_trips_through_json() {
        let project = example_project();
        let json = serde_json::to_string(&project).expect("serialize");
        let restored =
            migrate_project(serde_json::from_str(&json).expect("json")).expect("migrate");
        assert_eq!(restored, project);
    }

    #[test]
    fn v1_fixture_matches_example() {
        let value = serde_json::to_value(example_project()).expect("value");
        let path =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures/project-v1.json");
        if std::env::var_os("UPDATE_FIXTURES").is_some() {
            let pretty = serde_json::to_string_pretty(&value).expect("pretty");
            std::fs::write(&path, format!("{pretty}\n")).expect("write fixture");
        }
        let fixture_text = std::fs::read_to_string(&path).expect("read fixture");
        let fixture: Value = serde_json::from_str(&fixture_text).expect("fixture json");
        assert_eq!(value, fixture);
    }
}
