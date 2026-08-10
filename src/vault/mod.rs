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

pub fn note_stem(path: &Path) -> Option<&str> {
    path.file_stem().and_then(|stem| stem.to_str())
}

/// Helper: Builds a candidate relative path for `path` including `k` parent
/// directory components.
/// k = 0 -> "todo"
/// k = 1 -> "projects/todo"
/// k = 2 -> "work/projects/todo"
pub fn candidate_at(path: &Path, k: usize) -> Option<PathBuf> {
    let stem = note_stem(path)?;

    let parent = path.parent()?;
    let dirs = parent
        .components()
        .map(|c| c.as_os_str())
        .collect::<Vec<&std::ffi::OsStr>>();

    if k > dirs.len() {
        return None;
    }

    let start = dirs.len() - k;
    let mut candidate = PathBuf::new();
    for dir in &dirs[start..] {
        candidate.push(dir);
    }
    candidate.push(stem);
    Some(candidate)
}

pub fn shortest_unique_path(path: &Path, all: &[PathBuf]) -> Option<PathBuf> {
    let parent = path.parent()?;
    let dir_count = parent.components().count();

    for k in 0..=dir_count {
        let candidate = candidate_at(path, k)?;

        let is_unique = all.iter().all(|other| {
            if other == path {
                true
            } else {
                candidate_at(other, k).as_ref() != Some(&candidate)
            }
        });

        if is_unique {
            return Some(candidate);
        }
    }
    candidate_at(path, dir_count)
}

#[cfg(test)]
mod tests {
    use std::{
        env,
        fs::{self},
        path::{Path, PathBuf},
        sync::atomic::{AtomicU64, Ordering},
    };

    use crate::vault::{discover_vault, note_stem, shortest_unique_path};

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

    #[test]
    fn note_stem_from_nested_path() {
        let path = Path::new("inbox").join("todo.md");
        assert_eq!(note_stem(&path), Some("todo"));
    }

    #[test]
    fn shortest_unique_path_single_note_is_stem() {
        let path = PathBuf::from("inbox").join("todo.md");
        let all = vec![path.clone()];

        assert_eq!(
            shortest_unique_path(&path, &all),
            Some(PathBuf::from("todo"))
        );
    }

    #[test]
    fn shortest_unique_path_disambiguates_duplicate_stems() {
        let work = PathBuf::from("work").join("todo.md");
        let personal = PathBuf::from("personal").join("todo.md");
        let all = vec![work.clone(), personal.clone()];

        assert_eq!(
            shortest_unique_path(&work, &all),
            Some(PathBuf::from("work").join("todo"))
        );
        assert_eq!(
            shortest_unique_path(&personal, &all),
            Some(PathBuf::from("personal").join("todo"))
        );
    }
}
