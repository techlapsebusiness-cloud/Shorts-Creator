use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Health {
    pub name: String,
    pub version: String,
}

#[tauri::command]
pub fn health() -> Health {
    Health {
        name: "shorts-studio".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::health;

    #[test]
    fn health_names_the_app() {
        let health = health();
        assert_eq!(health.name, "shorts-studio");
        assert!(!health.version.is_empty());
    }
}
