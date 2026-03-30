//! # sandbox-workspace
//!
//! Safe workspace staging and patch filesystem helpers.
//!
//! This Tier 3 crate prepares controlled working directories and enforces
//! workspace-relative file access for patch application and validation. It is a
//! support layer for internal execution crates.

use std::path::{Path, PathBuf};

use forge_policy::{resolve_workspace_path, PolicyError};

#[derive(Debug, thiserror::Error)]
pub enum WorkspaceError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("policy error: {0}")]
    Policy(#[from] PolicyError),
}

pub type WorkspaceResult<T> = Result<T, WorkspaceError>;

#[derive(Debug)]
pub struct Workspace {
    pub host_path: PathBuf,
    pub _temp_dir: Option<tempfile::TempDir>,
}

#[derive(Debug, Clone)]
pub struct PatchedWorkspace {
    pub host_path: PathBuf,
}

pub trait PatchFs {
    fn root(&self) -> &Path;
    fn exists(&self, relative_path: &Path) -> WorkspaceResult<bool>;
    fn read_lines(&self, relative_path: &Path) -> WorkspaceResult<Vec<String>>;
    fn write_lines(&self, relative_path: &Path, lines: &[String]) -> WorkspaceResult<()>;
    fn remove_file(&self, relative_path: &Path) -> WorkspaceResult<()>;
    fn create_parent_dirs(&self, relative_path: &Path) -> WorkspaceResult<()>;
    fn snapshot_lines(&self, relative_path: &Path) -> WorkspaceResult<Option<Vec<String>>>;
}

#[derive(Debug, Clone)]
pub struct LocalPatchFs {
    root: PathBuf,
}

impl LocalPatchFs {
    pub fn new(root: &Path) -> Self {
        Self {
            root: root.to_path_buf(),
        }
    }

    fn resolve(&self, relative_path: &Path) -> WorkspaceResult<PathBuf> {
        Ok(resolve_workspace_path(&self.root, relative_path)?)
    }
}

impl PatchFs for LocalPatchFs {
    fn root(&self) -> &Path {
        &self.root
    }

    fn exists(&self, relative_path: &Path) -> WorkspaceResult<bool> {
        Ok(self.resolve(relative_path)?.exists())
    }

    fn read_lines(&self, relative_path: &Path) -> WorkspaceResult<Vec<String>> {
        let full_path = self.resolve(relative_path)?;
        let text = std::fs::read_to_string(full_path)?;
        Ok(text.lines().map(String::from).collect())
    }

    fn write_lines(&self, relative_path: &Path, lines: &[String]) -> WorkspaceResult<()> {
        let full_path = self.resolve(relative_path)?;
        if let Some(parent) = full_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let rendered = if lines.is_empty() {
            String::new()
        } else {
            let joined = lines.join("\n");
            format!("{joined}\n")
        };
        std::fs::write(full_path, rendered)?;
        Ok(())
    }

    fn remove_file(&self, relative_path: &Path) -> WorkspaceResult<()> {
        let full_path = self.resolve(relative_path)?;
        match std::fs::remove_file(full_path) {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(error) => Err(error.into()),
        }
    }

    fn create_parent_dirs(&self, relative_path: &Path) -> WorkspaceResult<()> {
        let full_path = self.resolve(relative_path)?;
        if let Some(parent) = full_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        Ok(())
    }

    fn snapshot_lines(&self, relative_path: &Path) -> WorkspaceResult<Option<Vec<String>>> {
        let full_path = self.resolve(relative_path)?;
        if !full_path.exists() {
            return Ok(None);
        }

        let text = std::fs::read_to_string(full_path)?;
        Ok(Some(text.lines().map(String::from).collect()))
    }
}

pub fn prepare_workspace(fixture: &Path) -> WorkspaceResult<Workspace> {
    let temp_dir = tempfile::TempDir::new()?;
    copy_dir_recursive(fixture, temp_dir.path())?;
    Ok(Workspace {
        host_path: temp_dir.path().to_path_buf(),
        _temp_dir: Some(temp_dir),
    })
}

pub fn as_patched_workspace(workspace: &Workspace) -> PatchedWorkspace {
    PatchedWorkspace {
        host_path: workspace.host_path.clone(),
    }
}

pub fn copy_dir_recursive_pub(src: &Path, dst: &Path) -> WorkspaceResult<()> {
    copy_dir_recursive(src, dst)
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> WorkspaceResult<()> {
    if !src.exists() {
        return Ok(());
    }

    for entry in walkdir::WalkDir::new(src)
        .follow_links(false)
        .into_iter()
        .filter_map(|entry| entry.ok())
    {
        if entry.file_type().is_symlink() {
            continue;
        }

        let relative = entry
            .path()
            .strip_prefix(src)
            .map_err(|error| std::io::Error::other(error.to_string()))?;
        let target = dst.join(relative);

        if entry.file_type().is_dir() {
            std::fs::create_dir_all(&target)?;
        } else {
            if let Some(parent) = target.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::copy(entry.path(), &target)?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod wave1_tests {
    use super::*;

    #[test]
    fn local_patch_fs_round_trips_and_removes_files() {
        let temp = tempfile::tempdir().unwrap();
        let fs = LocalPatchFs::new(temp.path());
        let relative = Path::new("src/lib.rs");
        let lines = vec!["fn main() {}".to_string(), "println!(\"ok\");".to_string()];

        fs.write_lines(relative, &lines).unwrap();
        assert!(fs.exists(relative).unwrap());
        assert_eq!(fs.read_lines(relative).unwrap(), lines);
        assert_eq!(fs.snapshot_lines(relative).unwrap(), Some(lines.clone()));

        fs.remove_file(relative).unwrap();
        assert!(!fs.exists(relative).unwrap());
        assert_eq!(fs.snapshot_lines(relative).unwrap(), None);
    }

    #[test]
    fn local_patch_fs_rejects_escape_paths() {
        let temp = tempfile::tempdir().unwrap();
        let fs = LocalPatchFs::new(temp.path());

        let err = fs
            .write_lines(Path::new("../escape.txt"), &["nope".to_string()])
            .unwrap_err();
        assert!(matches!(err, WorkspaceError::Policy(_)));
    }

    #[test]
    fn prepare_workspace_copies_fixture_tree() {
        let fixture = tempfile::tempdir().unwrap();
        let nested = fixture.path().join("nested");
        std::fs::create_dir_all(&nested).unwrap();
        std::fs::write(nested.join("file.txt"), "hello").unwrap();

        let workspace = prepare_workspace(fixture.path()).unwrap();
        let copied = workspace.host_path.join("nested/file.txt");
        assert_eq!(std::fs::read_to_string(&copied).unwrap(), "hello");

        let patched = as_patched_workspace(&workspace);
        assert_eq!(patched.host_path, workspace.host_path);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn local_patch_fs_round_trips_file_contents() {
        let root = tempfile::tempdir().unwrap();
        let fs = LocalPatchFs::new(root.path());
        let relative = Path::new("nested/file.txt");
        let lines = vec!["alpha".to_string(), "beta".to_string()];

        fs.write_lines(relative, &lines).unwrap();

        assert!(fs.exists(relative).unwrap());
        assert_eq!(fs.read_lines(relative).unwrap(), lines);
        assert_eq!(fs.snapshot_lines(relative).unwrap(), Some(lines.clone()));

        fs.remove_file(relative).unwrap();

        assert!(!fs.exists(relative).unwrap());
        assert_eq!(fs.snapshot_lines(relative).unwrap(), None);
    }

    #[test]
    fn local_patch_fs_rejects_escape_paths() {
        let root = tempfile::tempdir().unwrap();
        let fs = LocalPatchFs::new(root.path());
        let error = fs
            .write_lines(Path::new("../escape.txt"), &[String::from("nope")])
            .unwrap_err();

        assert!(matches!(error, WorkspaceError::Policy(_)));
        assert!(!root.path().join("../escape.txt").exists());
    }

    #[test]
    fn prepare_workspace_copies_fixture_recursively() {
        let fixture = tempfile::tempdir().unwrap();
        let nested = fixture.path().join("src");
        std::fs::create_dir_all(&nested).unwrap();
        std::fs::write(nested.join("lib.rs"), "pub fn demo() {}\n").unwrap();

        let workspace = prepare_workspace(fixture.path()).unwrap();
        let copied = workspace.host_path.join("src/lib.rs");
        let patched = as_patched_workspace(&workspace);

        assert_eq!(patched.host_path, workspace.host_path);
        assert_eq!(
            std::fs::read_to_string(copied).unwrap(),
            "pub fn demo() {}\n"
        );
    }

    #[cfg(unix)]
    #[test]
    fn local_patch_fs_rejects_reads_and_writes_through_symlink_segments() {
        let root = tempfile::tempdir().unwrap();
        let outside = tempfile::tempdir().unwrap();
        std::os::unix::fs::symlink(outside.path(), root.path().join("linked")).unwrap();
        let fs = LocalPatchFs::new(root.path());

        let read_err = fs.read_lines(Path::new("linked/escape.txt")).unwrap_err();
        let write_err = fs
            .write_lines(Path::new("linked/escape.txt"), &[String::from("nope")])
            .unwrap_err();
        let remove_err = fs.remove_file(Path::new("linked/escape.txt")).unwrap_err();

        assert!(matches!(read_err, WorkspaceError::Policy(_)));
        assert!(matches!(write_err, WorkspaceError::Policy(_)));
        assert!(matches!(remove_err, WorkspaceError::Policy(_)));
    }

    #[cfg(unix)]
    #[test]
    fn prepare_workspace_skips_symlink_entries() {
        let fixture = tempfile::tempdir().unwrap();
        let outside = tempfile::tempdir().unwrap();
        std::fs::write(outside.path().join("secret.txt"), "secret").unwrap();
        std::os::unix::fs::symlink(
            outside.path().join("secret.txt"),
            fixture.path().join("linked.txt"),
        )
        .unwrap();

        let workspace = prepare_workspace(fixture.path()).unwrap();
        assert!(
            !workspace.host_path.join("linked.txt").exists(),
            "symlink entries should not be materialized into the workspace"
        );
    }

    #[test]
    fn create_parent_dirs_rejects_nested_escape_attempts() {
        let root = tempfile::tempdir().unwrap();
        let fs = LocalPatchFs::new(root.path());

        let err = fs
            .create_parent_dirs(Path::new("safe/../../escape.txt"))
            .unwrap_err();
        assert!(matches!(err, WorkspaceError::Policy(_)));
    }

    proptest! {
        #[test]
        fn local_patch_fs_round_trips_safe_relative_paths(
            file_tail in "[A-Za-z0-9_/-]{1,24}",
            line_a in "[A-Za-z0-9_ (){};]{1,24}",
            line_b in "[A-Za-z0-9_ (){};]{1,24}",
        ) {
            prop_assume!(!file_tail.starts_with('/'));
            prop_assume!(!file_tail.contains(".."));
            let root = tempfile::tempdir().unwrap();
            let fs = LocalPatchFs::new(root.path());
            let relative = PathBuf::from(format!("nested/{file_tail}.txt"));
            let lines = vec![line_a, line_b];

            fs.write_lines(&relative, &lines).unwrap();
            prop_assert_eq!(fs.read_lines(&relative).unwrap(), lines.clone());
            prop_assert_eq!(fs.snapshot_lines(&relative).unwrap(), Some(lines));
        }

        #[test]
        fn local_patch_fs_rejects_generated_escape_paths(path_tail in "[A-Za-z0-9_./-]{1,24}") {
            let root = tempfile::tempdir().unwrap();
            let fs = LocalPatchFs::new(root.path());
            let candidate = PathBuf::from(format!("../{path_tail}"));

            prop_assert!(matches!(
                fs.write_lines(&candidate, &[String::from("blocked")]),
                Err(WorkspaceError::Policy(_))
            ));
            prop_assert!(matches!(
                fs.create_parent_dirs(&candidate),
                Err(WorkspaceError::Policy(_))
            ));
        }
    }
}
