use super::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ScaffoldSummary {
    pub(crate) app_dir: PathBuf,
    pub(crate) package_name: String,
    pub(crate) files: Vec<String>,
}

pub(crate) fn scaffold_project(
    profile: AiDENsProfile,
    name: &str,
    destination_root: &Path,
) -> Result<ScaffoldSummary> {
    let package_name = sanitize_package_name(name)?;
    scaffold_project_inner(profile, destination_root.join(&package_name), package_name)
}

pub(crate) fn scaffold_project_at(
    profile: AiDENsProfile,
    app_dir: &Path,
) -> Result<ScaffoldSummary> {
    let name = app_dir
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| anyhow::anyhow!("destination must include an app directory name"))?;
    let package_name = sanitize_package_name(name)?;
    scaffold_project_inner(profile, app_dir.to_path_buf(), package_name)
}

pub(crate) fn scaffold_project_inner(
    profile: AiDENsProfile,
    app_dir: PathBuf,
    package_name: String,
) -> Result<ScaffoldSummary> {
    if app_dir.exists() {
        bail!("target app directory already exists: {}", app_dir.display());
    }

    let plan = profile.expand(&package_name)?;
    plan.validate().map_err(anyhow::Error::msg)?;
    let mut cfg = AiDENsConfigV1::safe_default(&package_name);
    cfg.profile_id = Some(profile.id().into());
    cfg.memory_mode = plan.memory_mode;
    cfg.receipt_level = plan.receipt_level;
    cfg.provider = scaffold_provider_config(profile);
    cfg.receipts.store_root = Some(format!("target/aidens-receipts/{package_name}"));
    cfg.tools.sandbox_root = Some(scaffold_sandbox_root(&app_dir)?.display().to_string());
    cfg.tools.enabled_bundles = scaffold_tool_bundles(profile, &plan.enabled_tool_bundles);

    let files = vec![
        "aidens-scaffold-manifest.json".to_string(),
        "Cargo.toml".to_string(),
        "aidens.toml".to_string(),
        "README.md".to_string(),
        "AGENT.md".to_string(),
        "docs/tools.md".to_string(),
        "docs/permits.md".to_string(),
        "docs/receipts.md".to_string(),
        "src/main.rs".to_string(),
        "tests/smoke.rs".to_string(),
    ];
    let payloads = vec![
        (
            "aidens-scaffold-manifest.json",
            render_scaffold_manifest(profile, &package_name, &files)?,
        ),
        (
            "Cargo.toml",
            render_cargo_toml(&package_name, &workspace_aidens_crate_path()),
        ),
        ("aidens.toml", cfg.to_toml_string()?),
        ("README.md", render_generated_readme(&package_name)),
        ("AGENT.md", render_generated_agent_doc(&package_name)),
        ("docs/tools.md", render_tools_doc()),
        ("docs/permits.md", render_permits_doc()),
        ("docs/receipts.md", render_receipts_doc(&package_name)),
        ("src/main.rs", render_main_rs()),
        ("tests/smoke.rs", render_smoke_test(&package_name)),
    ];
    stage_scaffold_tree(&app_dir, &payloads)?;

    Ok(ScaffoldSummary {
        app_dir,
        package_name,
        files,
    })
}

pub(crate) fn stage_scaffold_tree(app_dir: &Path, payloads: &[(&str, String)]) -> Result<()> {
    let stage_dir = scaffold_stage_dir(app_dir)?;
    if stage_dir.exists() {
        bail!(
            "scaffold staging directory already exists: {}",
            stage_dir.display()
        );
    }
    if let Err(error) = write_staged_scaffold(&stage_dir, payloads)
        .and_then(|()| complete_staged_scaffold(&stage_dir, app_dir))
    {
        let _ = std::fs::remove_dir_all(&stage_dir);
        return Err(error);
    }
    Ok(())
}

pub(crate) fn write_staged_scaffold(stage_dir: &Path, payloads: &[(&str, String)]) -> Result<()> {
    for dir in ["src", "tests", "docs"] {
        std::fs::create_dir_all(stage_dir.join(dir))
            .with_context(|| format!("failed to create {}", stage_dir.join(dir).display()))?;
    }
    for (relative, contents) in payloads {
        write_scaffold_file(stage_dir, relative, contents)?;
    }
    Ok(())
}

pub(crate) fn complete_staged_scaffold(stage_dir: &Path, app_dir: &Path) -> Result<()> {
    if app_dir.exists() {
        bail!("target app directory already exists: {}", app_dir.display());
    }
    std::fs::rename(stage_dir, app_dir).with_context(|| {
        format!(
            "failed to atomically publish scaffold {} -> {}",
            stage_dir.display(),
            app_dir.display()
        )
    })
}

pub(crate) fn write_scaffold_file(stage_dir: &Path, relative: &str, contents: &str) -> Result<()> {
    if relative.contains("..") || Path::new(relative).is_absolute() {
        bail!("invalid scaffold relative path: {relative}");
    }
    let path = stage_dir.join(relative);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)
        .with_context(|| format!("failed to create-new scaffold file {}", path.display()))?;
    file.write_all(contents.as_bytes())
        .with_context(|| format!("failed to write scaffold file {}", path.display()))
}

pub(crate) fn scaffold_stage_dir(app_dir: &Path) -> Result<PathBuf> {
    let parent = app_dir.parent().unwrap_or_else(|| Path::new("."));
    let name = app_dir
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| anyhow::anyhow!("destination must include an app directory name"))?;
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    Ok(parent.join(format!(
        ".{name}.aidens-scaffold-tmp-{}-{nonce}",
        std::process::id()
    )))
}

pub(crate) fn scaffold_provider_config(profile: AiDENsProfile) -> ProviderConfigV1 {
    if profile == AiDENsProfile::CodingAgent {
        return ProviderConfigV1 {
            kind: "mock".into(),
            model: Some("aidens-safe-mock".into()),
            api_key: None,
            base_url: None,
            mock_response: Some(render_coding_agent_mock_response()),
        };
    }
    ProviderConfigV1 {
        kind: "disabled".into(),
        model: None,
        api_key: None,
        base_url: None,
        mock_response: None,
    }
}

pub(crate) fn scaffold_tool_bundles(
    profile: AiDENsProfile,
    plan_bundles: &[String],
) -> Vec<String> {
    if profile == AiDENsProfile::CodingAgent {
        return [
            "repo-read",
            "repo-list",
            "file-stat",
            "repo-search",
            "patch-propose",
        ]
        .into_iter()
        .map(String::from)
        .collect();
    }
    plan_bundles.to_vec()
}

pub(crate) fn scaffold_sandbox_root(app_dir: &Path) -> Result<PathBuf> {
    if app_dir.is_absolute() {
        Ok(app_dir.to_path_buf())
    } else {
        Ok(std::env::current_dir()?.join(app_dir))
    }
}

pub(crate) fn workspace_aidens_crate_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("workspace root")
        .join("crates")
        .join("aidens")
}

pub(crate) fn render_cargo_toml(package_name: &str, aidens_path: &Path) -> String {
    format!(
        r#"[package]
name = "{package_name}"
version = "0.1.0"
edition = "2021"

[workspace]

[dependencies]
aidens = {{ path = "{}" }}
anyhow = "1"
tokio = {{ version = "1", features = ["rt-multi-thread", "macros"] }}
"#,
        aidens_path.display()
    )
}

pub(crate) fn render_main_rs() -> String {
    r#"use aidens::prelude::*;

#[tokio::main]
pub(crate) async fn main() -> anyhow::Result<()> {
    let config = format!("{}/aidens.toml", env!("CARGO_MANIFEST_DIR"));
    let app = AiDENsApp::from_config(config).build().await?;
    let output = app.run_once("read README").await?;
    println!("{}", output.text);
    Ok(())
}
"#
    .into()
}

pub(crate) fn render_coding_agent_mock_response() -> String {
    format!(
        "{}{}{}",
        serde_json::json!({
            "tool_call": {
                "tool_id": "aidens:repo-read:1",
                "input": {"path": "README.md"},
            }
        }),
        TEST_AGENT_MOCK_RESPONSE_DELIMITER,
        "README evidence summary:\n{{last_tool_content}}"
    )
}

pub(crate) fn render_scaffold_manifest(
    profile: AiDENsProfile,
    package_name: &str,
    files: &[String],
) -> Result<String> {
    Ok(serde_json::to_string_pretty(&serde_json::json!({
        "schema": "AiDENsScaffoldManifestV1",
        "app_id": package_name,
        "profile_id": profile.id(),
        "support_tier": profile.product_surface_status(),
        "support_note": profile.product_surface_note(),
        "provider_route": if profile == AiDENsProfile::CodingAgent {
            "explicit-mock-fixture"
        } else {
            "disabled"
        },
        "receipt_store": format!("target/aidens-receipts/{package_name}"),
        "sandbox_root": ".",
        "files": files,
        "readiness_claim": "scaffold-generated; run receipts and smoke tests before claiming a completed app run",
        "canonical_owners": {
            "receipt_truth": "aidens-receipts / owner crate records",
            "tool_runtime": "llm-tool-runtime through aidens-tool-kit",
            "provider_route": "aidens-provider-kit route receipt"
        },
        "forbidden_claims": [
            "production-cloud-ready",
            "broad-autonomy-ready",
            "canonical-truth-owner",
            "v11B-complete",
            "v11C-complete"
        ],
        "reason_codes": [
            "scaffold-manifest-first",
            "explicit-mock-route",
            "receipt-store-declared",
            "side-effect-tools-not-enabled-by-default"
        ]
    }))?)
}

pub(crate) fn render_generated_readme(package_name: &str) -> String {
    format!(
        r#"# {package_name}

This generated AiDENs coding-agent scaffold runs with an explicit mock fixture by default.

It can read this README through the configured repository-read tool and writes receipts under `target/aidens-receipts/{package_name}`.

This is a scaffolded supported-local starting point, not a completed app-run proof. Inspect `aidens-scaffold-manifest.json`, run the generated smoke test, and inspect canonical receipts before making any completion claim.

Default safety posture:

- provider: explicit mock fixture, no cloud API key
- tools: read/list/search/stat plus patch proposal only
- writes/admin commands: not enabled by default
- receipts: full receipt level
- network policy: disabled

Try:

```bash
cargo run -p aidens-cli -- run --config path/to/{package_name}/aidens.toml "read README"
```
"#
    )
}

pub(crate) fn render_generated_agent_doc(package_name: &str) -> String {
    format!(
        r#"# Agent Operator Notes

Agent: `{package_name}`

This project is a runnable AiDENs coding-agent scaffold. It uses AiDENs as the orchestration/profile/policy/product layer and delegates execution evidence to the configured runner and receipt log.

Do not add cloud provider support by editing prompt text. Provider support must be implemented and tested before it can be marked executable.

Do not claim this scaffold is a completed app run until material operation receipts exist and the generated smoke test passes.

The generated default is intentionally safe:

- explicit mock provider fixture only
- repository sandbox scoped to this project directory
- no write/admin tool exposure by default
- receipt log enabled
"#
    )
}

pub(crate) fn render_tools_doc() -> String {
    r#"# Tools

Default exposed tool bundles:

- `repo-read`
- `repo-list`
- `file-stat`
- `repo-search`
- `patch-propose`

The generated config does not enable `patch-apply`, `run-checks`, shell, network, or admin tools. Add side-effect tools only with scoped permits and tests.
"#
    .into()
}

pub(crate) fn render_permits_doc() -> String {
    r#"# Permits

Read-only inspection tools do not need permits.

Any file write, patch apply, shell command, network operation, queue/admin action, or other side-effect tool must be permit-gated with an explicit scope. Do not grant broad write or admin access by default.
"#
    .into()
}

pub(crate) fn render_receipts_doc(package_name: &str) -> String {
    format!(
        r#"# Receipts

This scaffold uses full receipt level.

Default receipt store:

```text
target/aidens-receipts/{package_name}
```

Inspect receipts from the AiDENs workspace:

```bash
cargo run -p aidens-cli -- receipts list --config path/to/{package_name}/aidens.toml
```

The run report records provider route, tool exposure, tool calls, stop rules, agency policy report IDs, degraded paths, and failures.

User-visible completion depends on durable receipt records. A final string alone is not evidence of completion.
"#
    )
}

pub(crate) fn render_smoke_test(app_id: &str) -> String {
    format!(
        r#"use aidens::prelude::*;

#[test]
fn generated_plan_is_safe_and_visible() {{
    let plan = AiDENsProfile::CodingAgent
        .expand("{app_id}")
        .expect("generated profile expands");

    assert!(!plan.dangerous_auto_approval);
    assert!(plan.validate().is_ok());
    assert!(plan.risk_summary().contains("permit_required=true"));
}}

#[test]
fn generated_config_is_receipt_first_and_secret_free() {{
    let manifest = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("aidens-scaffold-manifest.json"),
    )
    .expect("generated scaffold manifest is readable");
    assert!(manifest.contains("\"schema\": \"AiDENsScaffoldManifestV1\""));
    assert!(manifest.contains("\"receipt_store\""));
    assert!(manifest.contains("\"explicit-mock-route\""));

    let config = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("aidens.toml"),
    )
    .expect("generated scaffold config is readable");
    assert!(config.contains("kind = \"mock\""));
    assert!(config.contains("receipt_level = \"full\""));
    assert!(config.contains("store_root = \"target/aidens-receipts/{app_id}\""));
    assert!(!config.contains("api_key"));
    assert!(!config.contains("secret"));
    assert!(!config.contains("token"));
    assert!(!config.contains("patch-apply"));
    assert!(!config.contains("run-checks"));
}}
"#
    )
}
