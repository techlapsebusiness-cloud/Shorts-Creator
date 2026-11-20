use std::fs::File;
use std::io::Write;
use std::path::Path;

use core::{Project, migrate_project};
use serde_json::Value;

use crate::StoreError;

pub fn load_project(dir: &Path) -> Result<Project, StoreError> {
    let text = std::fs::read_to_string(dir.join("project.json"))?;
    let value: Value = serde_json::from_str(&text)?;
    Ok(migrate_project(value)?)
}

pub fn save_project(dir: &Path, project: &Project) -> Result<(), StoreError> {
    let mut bytes = serde_json::to_vec_pretty(project)?;
    bytes.push(b'\n');
    write_atomic(&dir.join("project.json"), &bytes)?;
    Ok(())
}

fn write_atomic(path: &Path, bytes: &[u8]) -> Result<(), StoreError> {
    let Some(name) = path.file_name() else {
        return Err(
            std::io::Error::new(std::io::ErrorKind::InvalidInput, "path has no file name").into(),
        );
    };
    let mut temporary_name = name.to_os_string();
    temporary_name.push(".tmp");
    let temporary = path.with_file_name(temporary_name);
    {
        let mut file = File::create(&temporary)?;
        file.write_all(bytes)?;
        file.sync_all()?;
    }
    std::fs::rename(&temporary, path)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{load_project, save_project};
    use core::example_project;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_dir() -> std::path::PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("shorts-studio-files-{nanos}"));
        fs::create_dir_all(&dir).expect("temp dir");
        dir
    }

    #[test]
    fn a_torn_temp_file_does_not_replace_the_last_save() {
        let dir = temp_dir();
        let project = example_project();
        save_project(&dir, &project).expect("save");

        fs::write(dir.join("project.json.tmp"), b"{\"version\":1,\"name\":").expect("torn tmp");

        let loaded = load_project(&dir).expect("load");
        assert_eq!(loaded, project);
        assert!(dir.join("project.json.tmp").is_file());

        let mut edited = project.clone();
        edited.name = "After the crash".to_string();
        save_project(&dir, &edited).expect("save again");
        assert!(!dir.join("project.json.tmp").is_file());
        assert_eq!(load_project(&dir).expect("reload").name, "After the crash");

        fs::remove_dir_all(&dir).expect("cleanup");
    }

    #[test]
    fn v0_on_disk_opens_as_v1() {
        let dir = temp_dir();
        fs::write(
            dir.join("project.json"),
            include_str!("../../core/fixtures/project-v0.json"),
        )
        .expect("v0");
        let project = load_project(&dir).expect("load");
        assert_eq!(project.version, 1);
        assert_eq!(project.slug, "old-short");
        fs::remove_dir_all(&dir).expect("cleanup");
    }

    #[test]
    fn an_edited_beat_survives_save_and_reload() {
        let dir = temp_dir();
        let mut project = example_project();
        project.ai.raw_draft = "original draft".to_string();
        core::replace_beat_text(&mut project.ai.beats, 0, "Edited in the panel").expect("edit");
        save_project(&dir, &project).expect("save");
        let loaded = load_project(&dir).expect("load");
        assert_eq!(loaded.ai.beats[0].text, "Edited in the panel");
        assert_eq!(loaded.ai.raw_draft, "original draft");
        fs::remove_dir_all(&dir).expect("cleanup");
    }
}
