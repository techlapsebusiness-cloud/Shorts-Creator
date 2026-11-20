use std::fmt::Display;

#[derive(Debug)]
pub enum StoreError {
    Io(std::io::Error),
    Json(serde_json::Error),
    Sqlite(rusqlite::Error),
    Migrate(core::MigrateError),
    InvalidName,
    NoHome,
}

impl Display for StoreError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(error) => write!(formatter, "{error}"),
            Self::Json(error) => write!(formatter, "{error}"),
            Self::Sqlite(error) => write!(formatter, "{error}"),
            Self::Migrate(error) => write!(formatter, "{error}"),
            Self::InvalidName => write!(formatter, "project name cannot be turned into a slug"),
            Self::NoHome => write!(formatter, "HOME is not set"),
        }
    }
}

impl std::error::Error for StoreError {}

impl From<std::io::Error> for StoreError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<serde_json::Error> for StoreError {
    fn from(value: serde_json::Error) -> Self {
        Self::Json(value)
    }
}

impl From<rusqlite::Error> for StoreError {
    fn from(value: rusqlite::Error) -> Self {
        Self::Sqlite(value)
    }
}

impl From<core::MigrateError> for StoreError {
    fn from(value: core::MigrateError) -> Self {
        Self::Migrate(value)
    }
}
