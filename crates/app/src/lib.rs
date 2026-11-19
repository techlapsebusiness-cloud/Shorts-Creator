//! Tauri commands. The webview binary lives in `apps/desktop/src-tauri`.
//!
//! Commands are a submodule. `#[tauri::command]` on a public function in the
//! crate root defines its helper macros twice on tauri-macros 2.7.

pub const NAME: &str = "app";

mod commands;

pub use commands::{Health, health};

pub fn linked_crates() -> [&'static str; 4] {
    [core::NAME, media::NAME, ai::NAME, store::NAME]
}

#[cfg(test)]
mod tests {
    use super::linked_crates;

    #[test]
    fn workspace_crates_are_linked() {
        assert_eq!(linked_crates(), ["core", "media", "ai", "store"]);
    }
}
