use crate::{error::Result, model::RunReport};
use std::{
    fs,
    path::{Path, PathBuf},
};
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
    let d = init(cwd)?;
    fs::write(
        d.join(format!("{}.json", report.run_id)),
        serde_json::to_vec_pretty(report)?,
    )?;
    Ok(())
}
pub fn load_run(cwd: &Path, id: &str) -> Result<RunReport> {
    Ok(serde_json::from_slice(&fs::read(
        aew_dir(cwd).join(format!("{}.json", id)),
    )?)?)
}
