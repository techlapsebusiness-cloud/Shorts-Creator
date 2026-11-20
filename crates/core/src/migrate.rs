use serde_json::Value;

use crate::project::{Project, slugify};

#[derive(Debug)]
pub enum MigrateError {
    InvalidName,
    Unsupported(u32),
    Json(serde_json::Error),
}

impl std::fmt::Display for MigrateError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidName => write!(formatter, "project name cannot be turned into a slug"),
            Self::Unsupported(version) => {
                write!(formatter, "unsupported project version {version}")
            }
            Self::Json(error) => write!(formatter, "project json: {error}"),
        }
    }
}

impl std::error::Error for MigrateError {}

pub fn migrate_project(value: Value) -> Result<Project, MigrateError> {
    let version = value.get("version").and_then(Value::as_u64).unwrap_or(0);
    match u32::try_from(version).unwrap_or(u32::MAX) {
        0 => upgrade_v0(value),
        1 => serde_json::from_value(value).map_err(MigrateError::Json),
        newer => Err(MigrateError::Unsupported(newer)),
    }
}

fn upgrade_v0(value: Value) -> Result<Project, MigrateError> {
    let name = value
        .get("name")
        .and_then(Value::as_str)
        .filter(|name| !name.trim().is_empty())
        .ok_or(MigrateError::InvalidName)?
        .to_string();
    let slug = match value.get("slug").and_then(Value::as_str) {
        Some(slug) if slugify(slug).ok().as_deref() == Some(slug) => slug.to_string(),
        _ => slugify(&name).map_err(|_| MigrateError::InvalidName)?,
    };
    let number = value
        .get("number")
        .and_then(Value::as_u64)
        .and_then(|number| u32::try_from(number).ok())
        .unwrap_or(1);
    let id = value
        .get("id")
        .and_then(Value::as_str)
        .filter(|id| !id.is_empty())
        .unwrap_or(&slug)
        .to_string();
    let mut project = Project::new(id, number, name, slug);
    if let Some(topic) = value.get("topic").and_then(Value::as_str) {
        project.ai.topic = topic.to_string();
    }
    Ok(project)
}

#[cfg(test)]
mod tests {
    use super::migrate_project;
    use crate::MigrateError;
    use crate::project::example_project;
    use serde_json::json;

    #[test]
    fn v0_fixture_becomes_v1_and_is_stable() {
        let v0: serde_json::Value =
            serde_json::from_str(include_str!("../fixtures/project-v0.json")).expect("v0 fixture");
        let once = migrate_project(v0).expect("migrate v0");
        assert_eq!(once.version, 1);
        assert_eq!(once.name, "Old short");
        assert_eq!(once.slug, "old-short");
        assert_eq!(once.tracks.len(), 7);
        let twice =
            migrate_project(serde_json::to_value(&once).expect("v1 value")).expect("migrate v1");
        assert_eq!(once, twice);
    }

    #[test]
    fn future_versions_are_rejected() {
        let error = migrate_project(json!({"version": 2, "name": "Later"})).expect_err("future");
        match error {
            MigrateError::Unsupported(2) => {}
            other => panic!("unexpected error: {other}"),
        }
    }

    #[test]
    fn example_matches_a_v1_migration() {
        let project = example_project();
        let migrated = migrate_project(serde_json::to_value(&project).expect("value")).expect("v1");
        assert_eq!(migrated, project);
    }
}
