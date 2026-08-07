use std::path::{Path, PathBuf};

pub fn discover_vault(vault_root: &Path) -> std::io::Result<Vec<PathBuf>> {
    let mut notes = Vec::new();
    for entry in std::fs::read_dir(vault_root)? {
        let path = entry?.path();
        if path.is_file() && path.extension() == Some(std::ffi::OsStr::new("md")) {
            notes.push(path);
        } else if path.is_dir() {
            let nested = discover_vault(&path)?;
            notes.extend(nested);
        }
    }
    Ok(notes)
}

#[cfg(test)]
mod tests {
    use std::{
        env,
        fs::{self},
    };

    use crate::vault::discover_vault;

    #[test]
    fn discover_vault_empty_vault_returns_empty() {
        let dir = env::temp_dir().join("manotz_vault_empty_test");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();

        let notes = discover_vault(&dir).unwrap();

        assert!(notes.is_empty(), "expected no notes, got {notes:?}");

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn discover_vault_finds_md_ignores_other_files() {
        let dir = env::temp_dir().join("manotz_vault_flat_test");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("note.md"), "# Hi").unwrap();
        fs::write(dir.join("skip.txt"), "nope").unwrap();
        let notes = discover_vault(&dir).unwrap();

        assert_eq!(notes.len(), 1);
        assert_eq!(
            notes[0].file_name().and_then(|n| n.to_str()),
            Some("note.md")
        );
    }

    #[test]
    fn discover_vault_finds_md_in_subdir() {
        let dir = env::temp_dir().join("manotz_vault_nested_test");
        let _ = fs::remove_dir_all(&dir);
        let nested = dir.join("inbox");
        fs::create_dir_all(&nested).unwrap();
        fs::write(nested.join("todo.md"), "- [ ] ship").unwrap();

        let notes = discover_vault(&dir).unwrap();

        assert_eq!(notes.len(), 1);
        assert_eq!(
            notes[0].file_name().and_then(|n| n.to_str()),
            Some("todo.md")
        );
    }
}
