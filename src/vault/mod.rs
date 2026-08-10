use std::path::{Path, PathBuf};

pub fn discover_vault(vault_root: &Path) -> std::io::Result<Vec<PathBuf>> {
    let mut notes = Vec::new();
    for entry in std::fs::read_dir(vault_root)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let path = entry.path();
        if file_type.is_file() && path.extension() == Some(std::ffi::OsStr::new("md")) {
            notes.push(path);
        } else if file_type.is_dir() {
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
        path::{Path, PathBuf},
        sync::atomic::{AtomicU64, Ordering},
    };

    use crate::vault::discover_vault;

    /// Unique temp directory removed on drop — avoids fixed-path races/leftovers.
    struct TempVault {
        path: PathBuf,
    }

    impl TempVault {
        fn new(label: &str) -> Self {
            static COUNTER: AtomicU64 = AtomicU64::new(0);
            let n = COUNTER.fetch_add(1, Ordering::Relaxed);
            let path =
                env::temp_dir().join(format!("manotz_vault_{label}_{}_{n}", std::process::id()));
            fs::create_dir_all(&path).unwrap();
            Self { path }
        }

        fn path(&self) -> &Path {
            &self.path
        }
    }

    impl Drop for TempVault {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }

    #[test]
    fn discover_vault_empty_vault_returns_empty() {
        let dir = TempVault::new("empty");

        let notes = discover_vault(dir.path()).unwrap();

        assert!(notes.is_empty(), "expected no notes, got {notes:?}");
    }

    #[test]
    fn discover_vault_finds_md_ignores_other_files() {
        let dir = TempVault::new("flat");
        fs::write(dir.path().join("note.md"), "# Hi").unwrap();
        fs::write(dir.path().join("skip.txt"), "nope").unwrap();

        let notes = discover_vault(dir.path()).unwrap();

        assert_eq!(notes.len(), 1);
        assert_eq!(
            notes[0].file_name().and_then(|n| n.to_str()),
            Some("note.md")
        );
    }

    #[test]
    fn discover_vault_finds_md_in_subdir() {
        let dir = TempVault::new("nested");
        let nested = dir.path().join("inbox");
        fs::create_dir_all(&nested).unwrap();
        fs::write(nested.join("todo.md"), "- [ ] ship").unwrap();

        let notes = discover_vault(dir.path()).unwrap();

        assert_eq!(notes.len(), 1);
        assert_eq!(
            notes[0].file_name().and_then(|n| n.to_str()),
            Some("todo.md")
        );
    }
}
