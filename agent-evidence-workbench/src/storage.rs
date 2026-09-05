use crate::{
    error::{Error, Result},
    model::RunReport,
    v2::RunEventV2,
};
use regex::Regex;
use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
};

pub fn validate_id(id: &str) -> Result<()> {
    if id.is_empty()
        || id == "."
        || id == ".."
        || id.contains(['/', '\\'])
        || id.bytes().any(|b| b == 0)
    {
        return Err(Error::Invalid("unsafe identifier".into()));
    }
    Ok(())
}

pub fn aew_dir(cwd: &Path) -> PathBuf {
    cwd.join(".aew")
}

pub fn init(cwd: &Path) -> Result<PathBuf> {
    let d = aew_dir(cwd);
    fs::create_dir_all(d.join("evidence"))?;
    fs::write(d.join("manifest.json"), b"{\"version\":1}\n")?;
    Ok(d)
}

pub fn save_run(cwd: &Path, report: &RunReport) -> Result<()> {
    validate_id(&report.run_id)?;
    let d = init(cwd)?;
    fs::write(
        d.join(format!("{}.json", report.run_id)),
        serde_json::to_vec_pretty(report)?,
    )?;
    Ok(())
}

pub fn load_run(cwd: &Path, id: &str) -> Result<RunReport> {
    validate_id(id)?;
    Ok(serde_json::from_slice(&fs::read(
        aew_dir(cwd).join(format!("{}.json", id)),
    )?)?)
}

pub fn v2_run_dir(cwd: &Path, run_id: &str) -> Result<PathBuf> {
    validate_id(run_id)?;
    Ok(aew_dir(cwd).join("v2").join("runs").join(run_id))
}

fn event_path(cwd: &Path, run_id: &str, event_id: &str) -> Result<PathBuf> {
    validate_id(event_id)?;
    Ok(v2_run_dir(cwd, run_id)?
        .join("events")
        .join(format!("{event_id}.json")))
}

/// Atomically appends a uniquely identified V2 event.
/// Exact replay is idempotent; changed bytes for an existing ID fail closed.
pub fn append_v2_event(cwd: &Path, run_id: &str, event: &RunEventV2) -> Result<PathBuf> {
    let path = event_path(cwd, run_id, &event.event_id)?;
    let mut stored = event.clone();
    let mut count = 0usize;
    fn scrub(v: &mut serde_json::Value, count: &mut usize) {
        match v {
            serde_json::Value::String(s) => {
                let x = Regex::new(r##"(?i)bearer\s+[a-z0-9._-]+|api[_-]?key\s*[=:]\s*[^\s\"']+|\bsk-[A-Za-z0-9_-]+|-----BEGIN(?: [A-Z]+)? PRIVATE KEY-----[\s\S]*?-----END(?: [A-Z]+)? PRIVATE KEY-----"##).unwrap().replace_all(s, "[REDACTED]");
                if x.as_ref() != s {
                    *count += 1;
                    *s = x.into_owned();
                }
            }
            serde_json::Value::Array(a) => a.iter_mut().for_each(|v| scrub(v, count)),
            serde_json::Value::Object(o) => o.values_mut().for_each(|v| scrub(v, count)),
            _ => {}
        }
    }
    scrub(&mut stored.payload, &mut count);
    stored.payload = serde_json::json!({
        "content": stored.payload,
        "redaction_count": count,
        "redaction_policy_version": "bounded-credential-patterns-v1",
    });
    let bytes = serde_json::to_vec_pretty(&stored)?;
    if path.is_file() {
        if fs::read(&path)? == bytes {
            return Ok(path);
        }
        return Err(Error::Invalid(format!(
            "conflicting event replay for {}",
            event.event_id
        )));
    }
    let parent = path
        .parent()
        .ok_or_else(|| Error::Invalid("event path has no parent".into()))?;
    fs::create_dir_all(parent)?;
    let temporary = parent.join(format!(".{}.{}.tmp", event.event_id, std::process::id()));
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temporary)?;
    file.write_all(&bytes)?;
    file.sync_all()?;
    drop(file);
    match fs::rename(&temporary, &path) {
        Ok(()) => Ok(path),
        Err(error) if path.is_file() => {
            let _ = fs::remove_file(&temporary);
            if fs::read(&path)? == bytes {
                Ok(path)
            } else {
                Err(Error::Invalid(format!(
                    "conflicting concurrent event replay for {}: {error}",
                    event.event_id
                )))
            }
        }
        Err(error) => {
            let _ = fs::remove_file(&temporary);
            Err(Error::Io(error))
        }
    }
}
