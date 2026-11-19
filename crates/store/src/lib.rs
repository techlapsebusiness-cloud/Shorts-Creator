//! SQLite library and project folders. Tables arrive in M01.

pub const NAME: &str = "store";

pub const LIBRARY_FILE: &str = "library.sqlite";

#[cfg(test)]
mod tests {
    #[test]
    fn library_file_is_sqlite() {
        assert!(super::LIBRARY_FILE.ends_with(".sqlite"));
    }
}
