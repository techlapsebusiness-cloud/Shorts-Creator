//! SQLite library and crash-safe project folders.

pub const NAME: &str = "store";

pub const LIBRARY_FILE: &str = "library.sqlite";

mod autosave;
mod error;
mod files;
mod library;
mod projects;

pub use autosave::Autosave;
pub use error::StoreError;
pub use files::{load_project, save_project};
pub use library::{HashRow, Library, ProjectRow, TopicRow, UploadRow, UsageInsert};
pub use projects::{NewProject, create_project, default_movies_root, save_if_due};
