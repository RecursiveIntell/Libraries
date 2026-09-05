use crate::{
    collector::source_snapshot_v2,
    error::{Error, Result},
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{fs, path::Path, process::Command};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LibrariesReleaseVerificationV1 {
    pub schema_version: String,
    pub verifier_path: String,
    pub exit_code: i32,
    pub findings: Vec<String>,
    pub manifest_sha256_before: Option<String>,
    pub receipt_sha256_before: Option<String>,
    pub manifest_sha256_after: Option<String>,
    pub receipt_sha256_after: Option<String>,
}

fn digest_if_file(path: &Path) -> Result<Option<String>> {
    if !path.exists() {
        return Ok(None);
    }
    if !path.is_file() {
        return Err(Error::Invalid(format!("expected file: {}", path.display())));
    }
    Ok(Some(hex::encode(Sha256::digest(fs::read(path)?))))
}

/// Invokes and preserves the canonical verifier's result without reproducing
/// its release semantics. It fails closed on malformed output or mutation.
pub fn inspect_libraries_release(repo: &Path) -> Result<LibrariesReleaseVerificationV1> {
    let script = repo.join("scripts/run_release_gates.py");
    if !script.is_file() {
        return Err(Error::Invalid(format!(
            "canonical Libraries verifier is missing: {}",
            script.display()
        )));
    }
    let manifest = repo.join("STATUS_EVIDENCE_MANIFEST.json");
    let receipt = repo.join("release/closeout_receipt_v1.json");
    let manifest_before = digest_if_file(&manifest)?;
    let receipt_before = digest_if_file(&receipt)?;
    let repository_before = source_snapshot_v2(repo)?.workspace_content_digest;
    let output = Command::new("python3")
        .arg(&script)
        .arg("--repo")
        .arg(repo)
        .output()?;
    let exit_code = output.status.code().unwrap_or(-1);
    let payload: serde_json::Value = serde_json::from_slice(&output.stdout).map_err(|error| {
        Error::Command(format!(
            "canonical verifier did not produce JSON (exit {exit_code}): {error}"
        ))
    })?;
    let schema_version = payload
        .get("schema_version")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| Error::Command("canonical verifier JSON has no schema_version".into()))?;
    if schema_version != "libraries.evidence-verification.v1" {
        return Err(Error::Command(format!(
            "unsupported canonical verifier schema: {schema_version}"
        )));
    }
    let findings = payload
        .get("findings")
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| Error::Command("canonical verifier JSON has no findings array".into()))?
        .iter()
        .map(|value| {
            value
                .as_str()
                .map(str::to_string)
                .ok_or_else(|| Error::Command("canonical verifier finding is not a string".into()))
        })
        .collect::<Result<Vec<_>>>()?;
    let manifest_after = digest_if_file(&manifest)?;
    let receipt_after = digest_if_file(&receipt)?;
    let repository_after = source_snapshot_v2(repo)?.workspace_content_digest;
    if repository_before != repository_after {
        return Err(Error::Command(
            "canonical release verification mutated repository content".into(),
        ));
    }
    if manifest_before != manifest_after || receipt_before != receipt_after {
        return Err(Error::Command(
            "canonical release verification mutated its input evidence".into(),
        ));
    }
    Ok(LibrariesReleaseVerificationV1 {
        schema_version: "aew.libraries-release-verification.v1".into(),
        verifier_path: script.display().to_string(),
        exit_code,
        findings,
        manifest_sha256_before: manifest_before,
        receipt_sha256_before: receipt_before,
        manifest_sha256_after: manifest_after,
        receipt_sha256_after: receipt_after,
    })
}

/// Strict admission wrapper: a canonical finding or nonzero verifier exit is
/// preserved by inspection but is never presented as successful verification.
pub fn verify_libraries_release(repo: &Path) -> Result<LibrariesReleaseVerificationV1> {
    let inspection = inspect_libraries_release(repo)?;
    if inspection.exit_code != 0 || !inspection.findings.is_empty() {
        return Err(Error::Command(format!(
            "canonical verifier is not successful: exit={} findings={}",
            inspection.exit_code,
            inspection.findings.len()
        )));
    }
    Ok(inspection)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn consumes_canonical_json_without_mutating_inputs() {
        let directory = tempdir().expect("tempdir");
        let scripts = directory.path().join("scripts");
        fs::create_dir_all(&scripts).expect("scripts directory");
        fs::write(
            scripts.join("run_release_gates.py"),
            "import json\nprint(json.dumps({'schema_version': 'libraries.evidence-verification.v1', 'findings': []}))\n",
        )
        .expect("verifier script");
        fs::write(
            directory.path().join("STATUS_EVIDENCE_MANIFEST.json"),
            "{}\n",
        )
        .expect("manifest");
        let release = directory.path().join("release");
        fs::create_dir_all(&release).expect("release directory");
        fs::write(release.join("closeout_receipt_v1.json"), "{}\n").expect("receipt");
        for args in [
            vec!["init", "-q"],
            vec!["config", "user.email", "fixture@example.invalid"],
            vec!["config", "user.name", "AEW Fixture"],
            vec!["add", "."],
            vec!["commit", "-qm", "fixture"],
        ] {
            let status = Command::new("git")
                .args(args)
                .current_dir(directory.path())
                .status()
                .expect("git launches");
            assert!(status.success());
        }
        let result = verify_libraries_release(directory.path()).expect("canonical verifier result");
        assert!(result.findings.is_empty());
    }

    #[test]
    fn inspection_preserves_canonical_findings_without_calling_them_success() {
        let directory = tempdir().expect("tempdir");
        let scripts = directory.path().join("scripts");
        fs::create_dir_all(&scripts).expect("scripts directory");
        fs::write(
            scripts.join("run_release_gates.py"),
            "import json\nprint(json.dumps({'schema_version': 'libraries.evidence-verification.v1', 'findings': ['missing source_binding']}))\nraise SystemExit(1)\n",
        )
        .expect("verifier script");
        fs::write(
            directory.path().join("STATUS_EVIDENCE_MANIFEST.json"),
            "{}\n",
        )
        .expect("manifest");
        let release = directory.path().join("release");
        fs::create_dir_all(&release).expect("release directory");
        fs::write(release.join("closeout_receipt_v1.json"), "{}\n").expect("receipt");
        for args in [
            vec!["init", "-q"],
            vec!["config", "user.email", "fixture@example.invalid"],
            vec!["config", "user.name", "AEW Fixture"],
            vec!["add", "."],
            vec!["commit", "-qm", "fixture"],
        ] {
            assert!(Command::new("git")
                .args(args)
                .current_dir(directory.path())
                .status()
                .expect("git launches")
                .success());
        }
        let inspection = inspect_libraries_release(directory.path()).expect("inspection");
        assert_eq!(inspection.exit_code, 1);
        assert_eq!(inspection.findings, vec!["missing source_binding"]);
        assert!(verify_libraries_release(directory.path()).is_err());
    }
}
