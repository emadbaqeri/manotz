use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct SessionState {
    pub last_open_file: Option<PathBuf>,
    pub cursors: HashMap<PathBuf, usize>,
}

impl SessionState {
    pub fn load(path: &Path) -> Self {
        let Ok(content) = std::fs::read_to_string(path) else {
            return SessionState::default();
        };

        serde_json::from_str(&content).unwrap_or_default()
    }

    pub fn save(&self, path: &Path) -> std::io::Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let json_str = serde_json::to_string_pretty(self)
            .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err))?;

        std::fs::write(path, json_str)?;

        Ok(())
    }

    pub fn update_cursor(&mut self, file: PathBuf, cursor: usize) {
        self.last_open_file = Some(file.clone());
        self.cursors.insert(file, cursor);
    }
}

#[cfg(test)]
mod tests {
    use std::{env, fs, path::PathBuf};

    use crate::session::SessionState;

    #[test]
    fn test_session_state_save() {
        let temp_path = env::temp_dir().join("manotz_test_save.json");
        let mut state = SessionState::default();
        state.update_cursor(PathBuf::from("manotz_test_save.json"), 42);
        state.save(&temp_path).unwrap();
        let content = std::fs::read_to_string(&temp_path).unwrap();
        assert!(content.contains(r#""manotz_test_save.json""#));
        assert!(content.contains("42"));

        let _ = fs::remove_file(&temp_path);
    }

    #[test]
    fn test_session_state_load_success() {
        let temp_path = env::temp_dir().join("manotz_test_load.json");
        let mut original = SessionState::default();
        original.update_cursor(PathBuf::from("load_test.md"), 30);
        original.save(&temp_path).unwrap();

        let loaded = SessionState::load(&temp_path);
        assert_eq!(original, loaded);

        let _ = std::fs::remove_file(&temp_path);
    }

    #[test]
    fn test_session_state_load_not_found_returns_default() {
        let fake_path = PathBuf::from("does_not_exist.json");
        let loaded = SessionState::load(&fake_path);
        assert_eq!(loaded, SessionState::default());
    }
}
