//! Paste-ready YouTube Studio kit. Direct upload stays off until an OAuth client exists.

use serde::{Deserialize, Serialize};

pub const TITLE_LIMIT: usize = 100;
pub const UNVERIFIED_NOTICE: &str =
    "An unverified Google API project can only upload as private until Google audits it.";
pub const STUDIO_URL: &str = "https://studio.youtube.com";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StudioField {
    pub label: String,
    pub value: String,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct UploadPolicy {
    pub max_duration_secs: f64,
    pub checked_on: String,
    pub source: String,
    #[serde(default = "default_title_limit")]
    pub title_limit: u32,
    #[serde(default)]
    pub banned_hashtags: Vec<String>,
    #[serde(default)]
    pub studio: Vec<StudioField>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UploadKit {
    pub keyword: String,
    pub title: String,
    pub title_limit: u32,
    pub description: String,
    pub tags: Vec<String>,
    pub pinned_comment: String,
    pub studio: Vec<StudioField>,
    pub checked_on: String,
    pub direct_upload: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AccessToken {
    pub access_token: String,
    pub refresh_token: String,
}

fn default_title_limit() -> u32 {
    TITLE_LIMIT as u32
}

pub fn upload_policy() -> UploadPolicy {
    serde_json::from_str(include_str!("../../../policy/youtube-shorts.json"))
        .expect("upload policy")
}

pub fn build_kit(keyword: &str, hook: &str, long_url: &str, next_topic: &str) -> UploadKit {
    let policy = upload_policy();
    let title = keyword_title(keyword, hook, policy.title_limit as usize);
    let tags = clean_tags(&[keyword, "shorts", "ffmpeg"], &policy.banned_hashtags);
    UploadKit {
        keyword: keyword.trim().to_string(),
        title,
        title_limit: policy.title_limit,
        description: kit_description(hook, long_url, &["#howto"]),
        tags,
        pinned_comment: format!("What should the next short about {next_topic} cover?"),
        studio: policy.studio,
        checked_on: policy.checked_on,
        direct_upload: false,
    }
}

pub fn keyword_title(keyword: &str, hook: &str, limit: usize) -> String {
    let keyword = keyword.trim();
    let hook = hook.trim();
    let combined = if hook.to_lowercase().starts_with(&keyword.to_lowercase()) {
        hook.to_string()
    } else {
        format!("{keyword}: {hook}")
    };
    combined.chars().take(limit.max(1)).collect()
}

pub fn kit_description(hook: &str, long_url: &str, extra: &[&str]) -> String {
    let mut tags = vec!["#Shorts".to_string()];
    for tag in extra {
        let clean = hash_tag(tag);
        if clean.is_empty()
            || banned_tag(&clean)
            || tags.iter().any(|item| item.eq_ignore_ascii_case(&clean))
        {
            continue;
        }
        if tags.len() == 3 {
            break;
        }
        tags.push(clean);
    }
    format!("{}\n{}\n{}", hook.trim(), long_url.trim(), tags.join(" "))
}

pub fn clean_tags(tags: &[&str], banned: &[String]) -> Vec<String> {
    let mut out = Vec::new();
    for tag in tags {
        let clean = tag.trim().trim_start_matches('#').to_string();
        if clean.is_empty() {
            continue;
        }
        if banned.iter().any(|item| item.eq_ignore_ascii_case(&clean)) {
            continue;
        }
        if out
            .iter()
            .any(|item: &String| item.eq_ignore_ascii_case(&clean))
        {
            continue;
        }
        out.push(clean);
    }
    out
}

pub fn plain_text(kit: &UploadKit) -> String {
    let mut text = format!(
        "Title\n{}\n\nDescription\n{}\n\nTags\n{}\n\nPinned comment\n{}\n\n",
        kit.title,
        kit.description,
        kit.tags.join(", "),
        kit.pinned_comment
    );
    for field in &kit.studio {
        text.push_str(&format!("{}: {}\n", field.label, field.value));
    }
    text
}

pub fn direct_upload_allowed(toggle: bool, client_id: Option<&str>) -> bool {
    toggle && client_id.is_some_and(|id| !id.trim().is_empty())
}

pub fn authorize_url(client_id: &str, redirect: &str, state: &str) -> String {
    format!(
        "https://accounts.google.com/o/oauth2/v2/auth?client_id={client_id}&redirect_uri={redirect}&response_type=code&scope=https://www.googleapis.com/auth/youtube.upload&state={state}&access_type=offline"
    )
}

pub fn exchange_code(code: &str) -> Result<AccessToken, String> {
    let code = code.trim();
    if code.is_empty() {
        return Err("the OAuth redirect did not include a code".to_string());
    }
    Ok(AccessToken {
        access_token: format!("mock-{code}"),
        refresh_token: "mock-refresh".to_string(),
    })
}

pub fn insert_body(kit: &UploadKit) -> serde_json::Value {
    serde_json::json!({
        "snippet": {
            "title": kit.title,
            "description": kit.description,
            "tags": kit.tags,
            "categoryId": "28"
        },
        "status": {
            "privacyStatus": "private",
            "selfDeclaredMadeForKids": false
        }
    })
}

pub fn thumbnail_body(video_id: &str) -> serde_json::Value {
    serde_json::json!({ "videoId": video_id, "kind": "thumbnails.set" })
}

pub fn caption_body(video_id: &str, srt_name: &str) -> serde_json::Value {
    serde_json::json!({ "videoId": video_id, "name": srt_name, "kind": "captions.insert" })
}

pub fn reminder_due(hours_since_publish: f64) -> bool {
    (24.0..48.0).contains(&hours_since_publish)
}

fn hash_tag(tag: &str) -> String {
    let bare = tag.trim().trim_start_matches('#');
    if bare.is_empty() || banned_tag(bare) {
        String::new()
    } else {
        format!("#{bare}")
    }
}

fn banned_tag(tag: &str) -> bool {
    let bare = tag.trim().trim_start_matches('#');
    bare.eq_ignore_ascii_case("viral") || bare.eq_ignore_ascii_case("fyp")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_kit_keeps_the_keyword_first_and_drops_banned_tags() {
        let kit = build_kit(
            "rename",
            "This file survives a crash",
            "https://example.com/long",
            "exposure",
        );
        assert!(kit.title.to_lowercase().starts_with("rename"));
        assert!(kit.title.chars().count() <= kit.title_limit as usize);
        assert!(kit.description.contains("#Shorts"));
        assert!(!kit.description.to_lowercase().contains("#viral"));
        assert!(!kit.description.to_lowercase().contains("#fyp"));
        let messy = kit_description("Hook", "https://example.com", &["#Viral", "#fyp", "#howto"]);
        assert!(messy.contains("#howto"));
        assert!(!messy.to_lowercase().contains("viral"));
        assert_eq!(kit.checked_on, "2026-11-22");
        assert!(!kit.direct_upload);
        assert!(!direct_upload_allowed(true, None));
        assert!(direct_upload_allowed(true, Some("client")));
        let text = plain_text(&kit);
        assert!(text.contains("Audience: Not made for kids"));
        let restored: UploadKit =
            serde_json::from_str(&serde_json::to_string(&kit).unwrap()).unwrap();
        assert_eq!(restored, kit);
    }

    #[test]
    fn oauth_and_insert_stay_private_on_the_mock() {
        let url = authorize_url("client", "http://127.0.0.1:8787/callback", "state-1");
        assert!(url.contains("redirect_uri=http://127.0.0.1:8787/callback"));
        assert!(url.contains("state=state-1"));
        let token = exchange_code("abc").unwrap();
        assert_eq!(token.access_token, "mock-abc");
        assert!(exchange_code("  ").is_err());
        let kit = build_kit("rename", "Hook", "https://example.com", "next");
        let body = insert_body(&kit);
        assert_eq!(body["status"]["privacyStatus"], "private");
        assert_eq!(thumbnail_body("vid")["kind"], "thumbnails.set");
        assert_eq!(
            caption_body("vid", "captions.srt")["kind"],
            "captions.insert"
        );
        assert!(UNVERIFIED_NOTICE.contains("private"));
        assert!(reminder_due(30.0));
        assert!(!reminder_due(10.0));
        let long = keyword_title("rename", &"word ".repeat(40), 100);
        assert_eq!(long.chars().count(), 100);
    }
}
