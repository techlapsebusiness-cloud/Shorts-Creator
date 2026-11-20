use std::path::{Path, PathBuf};
use std::time::Instant;

use core::{Project, project_folder_name, slugify};

use crate::{Autosave, Library, ProjectRow, StoreError, save_project};

pub struct NewProject {
    pub id: String,
    pub name: String,
    pub updated_at: String,
}

pub fn default_movies_root() -> Result<PathBuf, StoreError> {
    let home = std::env::var_os("HOME").ok_or(StoreError::NoHome)?;
    Ok(PathBuf::from(home).join("Movies/ShortsStudio"))
}

pub fn create_project(
    library: &mut Library,
    movies_root: &Path,
    new: NewProject,
) -> Result<(Project, PathBuf), StoreError> {
    let slug = slugify(&new.name).map_err(|_| StoreError::InvalidName)?;
    let number = library.next_number()?;
    let dir = movies_root.join(project_folder_name(number, &slug));
    std::fs::create_dir_all(dir.join("media"))?;
    std::fs::create_dir_all(dir.join("cache"))?;
    std::fs::create_dir_all(dir.join("renders"))?;
    let project = Project::new(new.id, number, new.name, slug);
    save_project(&dir, &project)?;
    library.insert_project(&ProjectRow {
        id: project.id.clone(),
        number: project.number,
        slug: project.slug.clone(),
        name: project.name.clone(),
        dir: dir.display().to_string(),
        updated_at: new.updated_at,
    })?;
    Ok((project, dir))
}

pub fn save_if_due(
    library: &mut Library,
    dir: &Path,
    project: &Project,
    autosave: &mut Autosave,
    now: Instant,
    updated_at: &str,
) -> Result<bool, StoreError> {
    if !autosave.due(now) {
        return Ok(false);
    }
    save_project(dir, project)?;
    library.touch_project(&project.id, updated_at)?;
    autosave.saved();
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::{NewProject, create_project, save_if_due};
    use crate::{Autosave, Library, load_project};
    use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

    fn temp_root() -> std::path::PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("shorts-studio-projects-{nanos}"));
        std::fs::create_dir_all(&dir).expect("root");
        dir
    }

    #[test]
    fn create_open_and_autosave_after_a_missed_write() {
        let root = temp_root();
        let mut library = Library::open(&root.join("library.sqlite")).expect("library");
        let (project, dir) = create_project(
            &mut library,
            &root,
            NewProject {
                id: "p1".to_string(),
                name: "Daily Short".to_string(),
                updated_at: "2026-11-20T00:00:00Z".to_string(),
            },
        )
        .expect("create");
        assert_eq!(dir.file_name().expect("name"), "001-daily-short");
        assert!(dir.join("media").is_dir());
        assert!(dir.join("cache").is_dir());
        assert!(dir.join("renders").is_dir());
        assert_eq!(load_project(&dir).expect("open").id, "p1");

        std::fs::write(dir.join("project.json.tmp"), b"{not json").expect("crash");
        assert_eq!(load_project(&dir).expect("recover").name, "Daily Short");

        let mut edited = project;
        edited.ai.topic = "autosave".to_string();
        let start = Instant::now();
        let mut autosave = Autosave::new();
        autosave.mark_dirty(start);
        assert!(
            !save_if_due(
                &mut library,
                &dir,
                &edited,
                &mut autosave,
                start + Duration::from_secs(4),
                "2026-11-20T00:00:04Z",
            )
            .expect("early")
        );
        assert_eq!(load_project(&dir).expect("unchanged").ai.topic, "");
        assert!(
            save_if_due(
                &mut library,
                &dir,
                &edited,
                &mut autosave,
                start + Duration::from_secs(5),
                "2026-11-20T00:00:05Z",
            )
            .expect("due")
        );
        assert_eq!(load_project(&dir).expect("saved").ai.topic, "autosave");
        assert_eq!(
            library.list_projects().expect("rows")[0].updated_at,
            "2026-11-20T00:00:05Z"
        );
        assert!(
            create_project(
                &mut library,
                &root,
                NewProject {
                    id: "p2".to_string(),
                    name: "!!!".to_string(),
                    updated_at: "2026-11-20T00:00:06Z".to_string(),
                },
            )
            .is_err()
        );

        std::fs::remove_dir_all(&root).expect("cleanup");
    }
}
