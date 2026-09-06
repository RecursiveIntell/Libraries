use crate::{
    error::{Error, Result},
    model::RepositorySnapshot,
    v2::SourceSnapshotV2,
};
use chrono::Utc;
use sha2::{Digest, Sha256};
#[cfg(unix)]
use std::os::unix::ffi::OsStrExt;
use std::process::Command;
use std::{
    ffi::OsStr,
    path::{Component, Path, PathBuf},
};

fn git(cwd: &Path, args: &[&str]) -> Result<String> {
    let output = Command::new("git").args(args).current_dir(cwd).output()?;
    if !output.status.success() {
        return Err(Error::Command(format!(
            "git {} failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&output.stderr).trim()
        )));
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

fn git_bytes(cwd: &Path, args: &[&str]) -> Result<Vec<u8>> {
    let output = Command::new("git").args(args).current_dir(cwd).output()?;
    if !output.status.success() {
        return Err(Error::Command(format!(
            "git {} failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&output.stderr).trim()
        )));
    }
    Ok(output.stdout)
}

fn repository_relative_path(path: &[u8]) -> Result<PathBuf> {
    #[cfg(unix)]
    let path = PathBuf::from(OsStr::from_bytes(path));
    #[cfg(not(unix))]
    let path = PathBuf::from(
        std::str::from_utf8(path)
            .map_err(|_| Error::Invalid("Git returned a non-UTF-8 path".into()))?,
    );

    if path
        .components()
        .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err(Error::Invalid(
            "Git returned a non-relative untracked path".into(),
        ));
    }
    Ok(path)
}

fn excluded_projection(path: &[u8]) -> bool {
    path == b".aew"
        || path == b".git"
        || path == b"target"
        || path.starts_with(b".aew/")
        || path.starts_with(b".git/")
        || path.starts_with(b"target/")
}

fn credential_shaped_path(path: &Path) -> bool {
    let Some(name) = path.file_name().and_then(OsStr::to_str) else {
        return false;
    };
    let name = name.to_ascii_lowercase();
    name == ".env"
        || name.starts_with(".env.")
        || matches!(name.as_str(), "id_rsa" | "credentials" | "secrets")
        || name.ends_with(".pem")
        || name.ends_with(".key")
}

fn append_length_delimited(content: &mut Vec<u8>, value: &[u8]) {
    content.extend_from_slice(&(value.len() as u64).to_be_bytes());
    content.extend_from_slice(value);
}

fn untracked_paths(cwd: &Path) -> Result<Vec<Vec<u8>>> {
    let mut paths = Vec::new();
    for args in [
        ["ls-files", "-z", "--others", "--exclude-standard"].as_slice(),
        [
            "ls-files",
            "-z",
            "--others",
            "--ignored",
            "--exclude-standard",
        ]
        .as_slice(),
    ] {
        paths.extend(
            git_bytes(cwd, args)?
                .split(|byte| *byte == b'\0')
                .filter(|path| !path.is_empty() && !excluded_projection(path))
                .map(ToOwned::to_owned),
        );
    }
    paths.sort();
    paths.dedup();
    Ok(paths)
}

pub fn snapshot_repo(cwd: &Path, baseline: &str) -> Result<RepositorySnapshot> {
    let final_sha = git(cwd, &["rev-parse", "HEAD"])?.trim().to_string();
    let status = git(cwd, &["status", "--porcelain=v1", "--untracked-files=all"])?;
    let diff = git(cwd, &["diff", "HEAD", "--binary"])?;
    let diff_stat = git(cwd, &["diff", "HEAD", "--stat"])?;
    let digest = hex::encode(Sha256::digest(diff.as_bytes()));
    Ok(RepositorySnapshot {
        path: cwd.display().to_string(),
        baseline_sha: baseline.to_string(),
        final_sha,
        is_clean: status.trim().is_empty(),
        diff_stat,
        diff,
        status,
        diff_digest: digest,
    })
}

/// Captures one source identity snapshot. Callers must capture both pre- and
/// post-command snapshots; this function never infers a baseline from empty data.
pub fn source_snapshot_v2(cwd: &Path) -> Result<SourceSnapshotV2> {
    let head = git(cwd, &["rev-parse", "HEAD"])?.trim().to_string();
    let tree = git(cwd, &["rev-parse", "HEAD^{tree}"])?.trim().to_string();
    let raw_status = git(cwd, &["status", "--porcelain=v1", "--untracked-files=all"])?;
    // AEW's own untracked event store is a projection, not repository source.
    let status = raw_status
        .lines()
        .filter(|line| !line.get(3..).is_some_and(|path| path.starts_with(".aew/")))
        .collect::<Vec<_>>()
        .join("\n");
    let status = if status.is_empty() {
        status
    } else {
        format!("{status}\n")
    };
    let diff = git(cwd, &["diff", "HEAD", "--binary"])?;
    // Include ordinary and ignored untracked files. Git's NUL-delimited output
    // preserves every valid path spelling; only documented projections/build/Git
    // metadata are excluded. The preimage records each path and content length so
    // distinct path/content pairs cannot be concatenation aliases.
    let mut content = Vec::new();
    content.extend_from_slice(b"tracked-diff\0");
    append_length_delimited(&mut content, diff.as_bytes());
    for name in untracked_paths(cwd)? {
        let relative = repository_relative_path(&name)?;
        if credential_shaped_path(&relative) {
            return Err(Error::Invalid(format!(
                "refusing to snapshot credential-shaped untracked path: {}",
                relative.display()
            )));
        }
        let path = cwd.join(&relative);
        let metadata = std::fs::symlink_metadata(&path)?;
        if metadata.file_type().is_symlink() || !metadata.file_type().is_file() {
            continue;
        }
        content.extend_from_slice(b"untracked-file\0");
        append_length_delimited(&mut content, &name);
        append_length_delimited(&mut content, &std::fs::read(path)?);
    }
    Ok(SourceSnapshotV2 {
        repository_path: cwd.display().to_string(),
        head,
        tree,
        is_clean: status.trim().is_empty(),
        status,
        diff_digest: hex::encode(Sha256::digest(diff.as_bytes())),
        workspace_content_digest: hex::encode(Sha256::digest(content)),
        observed_at: Utc::now().to_rfc3339(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn non_repository_fails_closed() {
        let directory = tempdir().expect("tempdir");
        assert!(source_snapshot_v2(directory.path()).is_err());
    }
}
