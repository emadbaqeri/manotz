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

/// Helper: Finds the common directory prefix among a set of paths.
fn common_vault_root(paths: &[PathBuf]) -> Option<PathBuf> {
    if paths.is_empty() {
        return None;
    }
    let mut iter = paths.iter();
    let first = iter.next()?;
    let mut prefix = first.parent()?.to_path_buf();

    for path in iter {
        while !path.starts_with(&prefix) {
            if !prefix.pop() {
                return None;
            }
        }
    }
    Some(prefix)
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
    let root = common_vault_root(all);
    let rel_path = root
        .as_ref()
        .and_then(|r| path.strip_prefix(r).ok())
        .unwrap_or(path);

    let rel_all: Vec<PathBuf> = all
        .iter()
        .map(|p| {
            root.as_ref()
                .and_then(|r| p.strip_prefix(r).ok())
                .unwrap_or(p)
                .to_path_buf()
        })
        .collect();

    let parent = rel_path.parent()?;
    let dir_count = parent.components().count();

    for k in 0..=dir_count {
        let candidate = candidate_at(rel_path, k)?;

        let is_unique = rel_all.iter().all(|other| {
            if other == rel_path {
                true
            } else {
                let other_parent_count = other.parent().map_or(0, |p| p.components().count());
                (0..=other_parent_count)
                    .all(|other_k| candidate_at(other, other_k).as_ref() != Some(&candidate))
            }
        });

        if is_unique {
            return Some(candidate);
        }
    }

    None
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

    #[test]
    fn shortest_unique_path_strips_absolute_vault_root_prefix() {
        let root = PathBuf::from("/Users/emad/vault");
        let work = root.join("work").join("todo.md");
        let personal = root.join("personal").join("todo.md");
        let all = vec![work.clone(), personal.clone()];

        assert_eq!(
            shortest_unique_path(&work, &all),
            Some(PathBuf::from("work").join("todo"))
        );
    }

    #[test]
    fn shortest_unique_path_root_note_vs_nested_note() {
        let root_todo = PathBuf::from("todo.md");
        let work_todo = PathBuf::from("work").join("todo.md");
        let all = vec![root_todo.clone(), work_todo.clone()];

        // Root note 'todo.md' cannot be disambiguated with folder prefix, returns None
        assert_eq!(shortest_unique_path(&root_todo, &all), None);
        // Nested note 'work/todo.md' is disambiguated with 'work/todo'
        assert_eq!(
            shortest_unique_path(&work_todo, &all),
            Some(PathBuf::from("work").join("todo"))
        );
    }
}
