use std::path::Path;

use rusqlite::{Connection, OptionalExtension};

use crate::StoreError;

const SCHEMA: &str = include_str!("../migrations/001_library.sql");

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectRow {
    pub id: String,
    pub number: u32,
    pub slug: String,
    pub name: String,
    pub dir: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TopicRow {
    pub id: String,
    pub title: String,
    pub position: i64,
    pub status: String,
    pub created_at: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct UsageInsert {
    pub provider: String,
    pub model: String,
    pub task: String,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub cost_usd: f64,
    pub created_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UploadRow {
    pub id: String,
    pub project_id: String,
    pub video_id: Option<String>,
    pub published_at: Option<String>,
    pub visibility: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HashRow {
    pub project_id: String,
    pub kind: String,
    pub value: String,
    pub created_at: String,
}

pub struct Library {
    conn: Connection,
}

impl Library {
    pub fn open(path: &Path) -> Result<Self, StoreError> {
        if let Some(parent) = path.parent()
            && !parent.as_os_str().is_empty()
        {
            std::fs::create_dir_all(parent)?;
        }
        let conn = Connection::open(path)?;
        let mut library = Self { conn };
        library.migrate()?;
        Ok(library)
    }

    pub fn open_in_memory() -> Result<Self, StoreError> {
        let conn = Connection::open_in_memory()?;
        let mut library = Self { conn };
        library.migrate()?;
        Ok(library)
    }

    fn migrate(&mut self) -> Result<(), StoreError> {
        self.conn.execute_batch(SCHEMA)?;
        let current: i64 = self.conn.query_row(
            "SELECT COALESCE(MAX(version), 0) FROM schema_migrations",
            [],
            |row| row.get(0),
        )?;
        if current < 1 {
            self.conn
                .execute("INSERT INTO schema_migrations (version) VALUES (1)", [])?;
        }
        Ok(())
    }

    pub fn next_number(&self) -> Result<u32, StoreError> {
        let number: i64 = self.conn.query_row(
            "SELECT COALESCE(MAX(number), 0) + 1 FROM projects",
            [],
            |row| row.get(0),
        )?;
        u32::try_from(number).map_err(|error| {
            StoreError::Io(std::io::Error::new(std::io::ErrorKind::InvalidData, error))
        })
    }

    pub fn insert_project(&mut self, row: &ProjectRow) -> Result<(), StoreError> {
        self.conn.execute(
            "INSERT INTO projects (id, number, slug, name, dir, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            (
                &row.id,
                row.number,
                &row.slug,
                &row.name,
                &row.dir,
                &row.updated_at,
            ),
        )?;
        Ok(())
    }

    pub fn touch_project(&mut self, id: &str, updated_at: &str) -> Result<(), StoreError> {
        self.conn.execute(
            "UPDATE projects SET updated_at = ?1 WHERE id = ?2",
            (updated_at, id),
        )?;
        Ok(())
    }

    pub fn list_projects(&self) -> Result<Vec<ProjectRow>, StoreError> {
        let mut statement = self.conn.prepare(
            "SELECT id, number, slug, name, dir, updated_at FROM projects ORDER BY number",
        )?;
        let rows = statement.query_map([], |row| {
            Ok(ProjectRow {
                id: row.get(0)?,
                number: row.get(1)?,
                slug: row.get(2)?,
                name: row.get(3)?,
                dir: row.get(4)?,
                updated_at: row.get(5)?,
            })
        })?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(StoreError::from)
    }

    pub fn enqueue_topic(
        &mut self,
        id: &str,
        title: &str,
        created_at: &str,
    ) -> Result<TopicRow, StoreError> {
        let position: i64 = self.conn.query_row(
            "SELECT COALESCE(MAX(position), 0) + 1 FROM topics",
            [],
            |row| row.get(0),
        )?;
        let topic = TopicRow {
            id: id.to_string(),
            title: title.to_string(),
            position,
            status: "queued".to_string(),
            created_at: created_at.to_string(),
        };
        self.conn.execute(
            "INSERT INTO topics (id, title, position, status, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            (
                &topic.id,
                &topic.title,
                topic.position,
                &topic.status,
                &topic.created_at,
            ),
        )?;
        Ok(topic)
    }

    pub fn list_topics(&self) -> Result<Vec<TopicRow>, StoreError> {
        let mut statement = self.conn.prepare(
            "SELECT id, title, position, status, created_at FROM topics ORDER BY position",
        )?;
        let rows = statement.query_map([], |row| {
            Ok(TopicRow {
                id: row.get(0)?,
                title: row.get(1)?,
                position: row.get(2)?,
                status: row.get(3)?,
                created_at: row.get(4)?,
            })
        })?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(StoreError::from)
    }

    pub fn record_usage(&mut self, usage: &UsageInsert) -> Result<i64, StoreError> {
        self.conn.execute(
            "INSERT INTO ai_usage (
                provider, model, task, input_tokens, output_tokens, cost_usd, created_at
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            (
                &usage.provider,
                &usage.model,
                &usage.task,
                usage.input_tokens,
                usage.output_tokens,
                usage.cost_usd,
                &usage.created_at,
            ),
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn usage_cost_usd(&self) -> Result<f64, StoreError> {
        let total: f64 = self.conn.query_row(
            "SELECT COALESCE(SUM(cost_usd), 0) FROM ai_usage",
            [],
            |row| row.get(0),
        )?;
        Ok(total)
    }

    pub fn record_upload(&mut self, upload: &UploadRow) -> Result<(), StoreError> {
        self.conn.execute(
            "INSERT INTO uploads (id, project_id, video_id, published_at, visibility)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            (
                &upload.id,
                &upload.project_id,
                &upload.video_id,
                &upload.published_at,
                &upload.visibility,
            ),
        )?;
        Ok(())
    }

    pub fn list_uploads(&self) -> Result<Vec<UploadRow>, StoreError> {
        let mut statement = self.conn.prepare(
            "SELECT id, project_id, video_id, published_at, visibility FROM uploads ORDER BY id",
        )?;
        let rows = statement.query_map([], |row| {
            Ok(UploadRow {
                id: row.get(0)?,
                project_id: row.get(1)?,
                video_id: row.get(2)?,
                published_at: row.get(3)?,
                visibility: row.get(4)?,
            })
        })?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(StoreError::from)
    }

    pub fn record_hash(&mut self, row: &HashRow) -> Result<(), StoreError> {
        self.conn.execute(
            "INSERT INTO originality_hashes (project_id, kind, value, created_at)
             VALUES (?1, ?2, ?3, ?4)",
            (&row.project_id, &row.kind, &row.value, &row.created_at),
        )?;
        Ok(())
    }

    pub fn hashes_for(&self, project_id: &str) -> Result<Vec<HashRow>, StoreError> {
        let mut statement = self.conn.prepare(
            "SELECT project_id, kind, value, created_at
             FROM originality_hashes WHERE project_id = ?1 ORDER BY id",
        )?;
        let rows = statement.query_map([project_id], |row| {
            Ok(HashRow {
                project_id: row.get(0)?,
                kind: row.get(1)?,
                value: row.get(2)?,
                created_at: row.get(3)?,
            })
        })?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(StoreError::from)
    }

    pub fn project_dir(&self, id: &str) -> Result<Option<String>, StoreError> {
        let dir = self
            .conn
            .query_row("SELECT dir FROM projects WHERE id = ?1", [id], |row| {
                row.get(0)
            })
            .optional()?;
        Ok(dir)
    }
}

#[cfg(test)]
mod tests {
    use super::{HashRow, Library, ProjectRow, UploadRow, UsageInsert};

    fn project(id: &str, number: u32) -> ProjectRow {
        ProjectRow {
            id: id.to_string(),
            number,
            slug: format!("slug-{number}"),
            name: format!("Name {number}"),
            dir: format!("/tmp/{number}"),
            updated_at: "2026-11-20T00:00:00Z".to_string(),
        }
    }

    #[test]
    fn library_round_trips_every_table() {
        let library = Library::open_in_memory().expect("open");
        let mut library = library;
        assert_eq!(library.next_number().expect("number"), 1);
        library.insert_project(&project("p1", 1)).expect("insert");
        library
            .touch_project("p1", "2026-11-20T01:00:00Z")
            .expect("touch");
        let projects = library.list_projects().expect("list");
        assert_eq!(projects[0].updated_at, "2026-11-20T01:00:00Z");
        assert_eq!(library.next_number().expect("next"), 2);

        let first = library
            .enqueue_topic("t1", "Rust errors", "2026-11-20T00:00:00Z")
            .expect("topic");
        let second = library
            .enqueue_topic("t2", "FFmpeg filters", "2026-11-20T00:01:00Z")
            .expect("topic");
        assert_eq!(first.position, 1);
        assert_eq!(second.position, 2);
        assert_eq!(library.list_topics().expect("topics").len(), 2);

        library
            .record_usage(&UsageInsert {
                provider: "mock".to_string(),
                model: "fixture".to_string(),
                task: "script".to_string(),
                input_tokens: 10,
                output_tokens: 20,
                cost_usd: 0.25,
                created_at: "2026-11-20T00:00:00Z".to_string(),
            })
            .expect("usage");
        library
            .record_usage(&UsageInsert {
                provider: "mock".to_string(),
                model: "fixture".to_string(),
                task: "script".to_string(),
                input_tokens: 1,
                output_tokens: 1,
                cost_usd: 0.75,
                created_at: "2026-11-20T00:02:00Z".to_string(),
            })
            .expect("usage");
        let cost = library.usage_cost_usd().expect("cost");
        assert!((cost - 1.0).abs() < 0.000_001);

        library
            .record_upload(&UploadRow {
                id: "up1".to_string(),
                project_id: "p1".to_string(),
                video_id: None,
                published_at: None,
                visibility: "private".to_string(),
            })
            .expect("upload");
        assert_eq!(library.list_uploads().expect("uploads")[0].video_id, None);

        library
            .record_hash(&HashRow {
                project_id: "p1".to_string(),
                kind: "title".to_string(),
                value: "abc".to_string(),
                created_at: "2026-11-20T00:00:00Z".to_string(),
            })
            .expect("hash");
        assert_eq!(library.hashes_for("p1").expect("hashes")[0].value, "abc");
        assert!(library.hashes_for("missing").expect("empty").is_empty());
    }
}
