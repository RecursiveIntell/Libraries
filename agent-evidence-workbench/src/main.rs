use agent_evidence_workbench::{
    cli::{Cli, Commands},
    collector::{snapshot_repo, source_snapshot_v2},
    extractor::extract_claims,
    model::*,
    receipt,
    report::generate_markdown,
    run::run_command,
    storage,
};
use chrono::Utc;
use clap::Parser;
use sha2::{Digest, Sha256};
use std::{fs, path::Path};
fn item(
    kind: EvidenceKind,
    source: &str,
    content: &str,
    exit: Option<i32>,
    dur: u128,
) -> EvidenceItem {
    let redacted = agent_evidence_workbench::v2::redact_text(content);
    EvidenceItem {
        id: hex::encode(Sha256::digest(
            format!("{}:{}", source, redacted.text).as_bytes(),
        )),
        kind,
        source: source.into(),
        digest: hex::encode(Sha256::digest(redacted.text.as_bytes())),
        summary: redacted.text.chars().take(200).collect(),
        exit_code: exit,
        duration_ms: dur,
    }
}
async fn resolved(mut r: RunReport) -> anyhow::Result<RunReport> {
    // V1 regex extraction and substring matching are retained only for legacy
    // inspection. They are not release-grade adjudication and cannot emit a
    // verified claim or a Clean verdict. Use the explicit V2 input contract.
    for claim in &mut r.claims {
        claim.status = ClaimStatus::NotChecked;
    }
    r.verdict = RunVerdict::Partial;
    Ok(r)
}
fn load(cwd: &Path, id: &str) -> anyhow::Result<RunReport> {
    Ok(storage::load_run(cwd, id)?)
}
#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let cwd = std::env::current_dir()?;
    match cli.command {
        Commands::Init { path } => println!(
            "initialized {}",
            storage::init(&path.unwrap_or(cwd))?.display()
        ),
        Commands::Run { name, cmd } => {
            let id = name.unwrap_or_else(|| Utc::now().timestamp_millis().to_string());
            let (p, a) = cmd
                .split_first()
                .ok_or_else(|| anyhow::anyhow!("command required"))?;
            let check = run_command(p, a, &cwd).await?;
            let text = format!("{}\n{}", check.stdout, check.stderr);
            let claims = extract_claims(&text);
            let snap = snapshot_repo(&cwd, "").ok();
            let mut ev = vec![item(
                EvidenceKind::CommandResult,
                &check.command,
                &text,
                check.exit_code,
                check.duration_ms,
            )];
            if let Some(s) = &snap {
                ev.push(item(
                    EvidenceKind::GitStatus,
                    "git status",
                    &s.status,
                    None,
                    0,
                ));
                ev.push(item(EvidenceKind::GitDiff, "git diff", &s.diff, None, 0));
            }
            let r = resolved(RunReport {
                run_id: id.clone(),
                verdict: RunVerdict::Partial,
                claims,
                checks: vec![check],
                diff: snap.map(|s| s.diff).unwrap_or_default(),
                evidence_manifest: ev,
            })
            .await?;
            storage::save_run(&cwd, &r)?;
            println!("{}", generate_markdown(&r));
        }
        Commands::ImportTranscript { name, transcript } => {
            let raw_text = fs::read_to_string(&transcript)?;
            let text = agent_evidence_workbench::v2::redact_text(&raw_text).text;
            let claims = extract_claims(&text);
            let ev = item(
                EvidenceKind::Transcript,
                &transcript.display().to_string(),
                &text,
                None,
                0,
            );
            let r = resolved(RunReport {
                run_id: name.clone(),
                verdict: RunVerdict::Partial,
                claims,
                checks: vec![],
                diff: String::new(),
                evidence_manifest: vec![ev],
            })
            .await?;
            storage::save_run(&cwd, &r)?;
            println!("imported {}", name);
        }
        Commands::ImportGraphResult { run_id, result } => {
            let mut r = load(&cwd, &run_id)?;
            let text = fs::read_to_string(&result)?;
            let v: serde_json::Value = serde_json::from_str(&text)?;
            let mut found = Vec::new();
            fn walk(v: &serde_json::Value, out: &mut Vec<String>) {
                match v {
                    serde_json::Value::Object(m) => {
                        for (k, x) in m {
                            if [
                                "estimated_cost",
                                "cost",
                                "input_tokens",
                                "output_tokens",
                                "total_tokens",
                            ]
                            .contains(&k.as_str())
                                && x.is_number()
                            {
                                out.push(format!("{}={}", k, x));
                            }
                            walk(x, out)
                        }
                    }
                    serde_json::Value::Array(a) => {
                        for x in a {
                            walk(x, out)
                        }
                    }
                    _ => {}
                }
            }
            walk(&v, &mut found);
            let summary = if found.is_empty() {
                "Agent Graph result imported; no cost or token fields present".into()
            } else {
                found.join(", ")
            };
            r.evidence_manifest.push(item(
                EvidenceKind::CommandResult,
                &result.display().to_string(),
                &summary,
                None,
                0,
            ));
            storage::save_run(&cwd, &r)?;
            println!("graph result appended; cost_fields={}", found.len());
        }
        Commands::Verify { run_id } => {
            let r = resolved(load(&cwd, &run_id)?).await?;
            storage::save_run(&cwd, &r)?;
            println!("{}", generate_markdown(&r));
        }
        Commands::Report { run_id, .. } => println!("{}", generate_markdown(&load(&cwd, &run_id)?)),
        Commands::Claims { run_id } => {
            let id = run_id.ok_or_else(|| anyhow::anyhow!("run-id required for claims"))?;
            for c in load(&cwd, &id)?.claims {
                println!("{}\t{:?}\t{}", c.id, c.status, c.text)
            }
        }
        Commands::Evidence { run_id } => {
            let id = run_id.ok_or_else(|| anyhow::anyhow!("run-id required for evidence"))?;
            println!(
                "{}",
                serde_json::to_string_pretty(&load(&cwd, &id)?.evidence_manifest)?
            );
        }
        Commands::Adjudicate { .. } => {
            anyhow::bail!(
                "legacy V1 adjudication is disabled: use evaluate-v2 with explicit claim/evidence links"
            )
        }
        Commands::VerifyLibrariesRelease { repo } => println!(
            "{}",
            serde_json::to_string_pretty(
                &agent_evidence_workbench::libraries_release::verify_libraries_release(&repo)?,
            )?
        ),
        Commands::InspectLibrariesRelease { repo } => println!(
            "{}",
            serde_json::to_string_pretty(
                &agent_evidence_workbench::libraries_release::inspect_libraries_release(&repo)?,
            )?
        ),
        Commands::SnapshotV2 => println!(
            "{}",
            serde_json::to_string_pretty(&source_snapshot_v2(&cwd)?)?
        ),
        Commands::CaptureV2 {
            input,
            evidence_id,
            cmd,
        } => {
            let raw = fs::read_to_string(&input)?;
            let mut parsed: agent_evidence_workbench::v2::ReleaseTruthInputV2 =
                serde_json::from_str(&raw)?;
            if parsed
                .commands
                .iter()
                .any(|command| command.id == evidence_id)
            {
                anyhow::bail!("capture evidence id already exists")
            }
            let (program, args) = cmd
                .split_first()
                .ok_or_else(|| anyhow::anyhow!("command required"))?;
            let pre = source_snapshot_v2(&cwd)?;
            let observed_at = Utc::now().to_rfc3339();
            let check = run_command(program, args, &cwd).await?;
            let post = source_snapshot_v2(&cwd)?;
            let mut normalized_pre = pre.clone();
            let mut normalized_post = post.clone();
            normalized_pre.observed_at.clear();
            normalized_post.observed_at.clear();
            if normalized_pre != normalized_post {
                anyhow::bail!(
                    "captured command changed repository source state; V2 capture is fail-closed"
                )
            }
            parsed
                .commands
                .push(agent_evidence_workbench::v2::CommandEvidenceV2 {
                    id: evidence_id,
                    execution_mode: "argv".into(),
                    argv: cmd,
                    cwd: cwd.display().to_string(),
                    outcome: if check.passed {
                        agent_evidence_workbench::v2::CommandOutcomeV2::Passed
                    } else {
                        agent_evidence_workbench::v2::CommandOutcomeV2::Failed
                    },
                    stdout: check.stdout,
                    stderr: check.stderr,
                    observed_at,
                    recorded_at: Utc::now().to_rfc3339(),
                });
            parsed.source_binding =
                Some(agent_evidence_workbench::v2::SourceBindingV2 { pre, post });
            let (sanitized, redaction_count) =
                agent_evidence_workbench::v2::sanitize_input(&parsed);
            let report = agent_evidence_workbench::v2::evaluate(&sanitized)?;
            let run_id = report.run_id.clone();
            let event = agent_evidence_workbench::v2::RunEventV2 {
                schema_version: "aew.run-event.v2".into(),
                event_id: format!("capture-{}", report.canonical_digest),
                kind: "release_truth_command_captured".into(),
                payload: serde_json::json!({
                    "input": sanitized,
                    "report": report.clone(),
                    "redaction_count": redaction_count,
                }),
                observed_at: Utc::now().to_rfc3339(),
                recorded_at: Utc::now().to_rfc3339(),
            };
            let recorded_event = storage::append_v2_event(&cwd, &run_id, &event)?;
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "report": report,
                    "redaction_count": redaction_count,
                    "recorded_event": recorded_event,
                }))?
            );
        }
        Commands::EvaluateV2 { input, record } => {
            let raw = fs::read_to_string(&input)?;
            let parsed: agent_evidence_workbench::v2::ReleaseTruthInputV2 =
                serde_json::from_str(&raw)?;
            let (sanitized, redaction_count) =
                agent_evidence_workbench::v2::sanitize_input(&parsed);
            let binding = sanitized
                .source_binding
                .as_ref()
                .ok_or_else(|| anyhow::anyhow!("V2 source_binding is required"))?;
            let mut expected_pre = binding.pre.clone();
            let mut expected_post = binding.post.clone();
            expected_pre.observed_at.clear();
            expected_post.observed_at.clear();
            let mut observed = source_snapshot_v2(&cwd)?;
            observed.observed_at.clear();
            if expected_pre != observed || expected_post != observed {
                anyhow::bail!("current repository does not match the declared V2 source binding")
            }
            let report = agent_evidence_workbench::v2::evaluate(&sanitized)?;
            let run_id = report.run_id.clone();
            let mut recorded_event = None;
            if record {
                let observed_at = sanitized
                    .commands
                    .first()
                    .map(|command| command.observed_at.clone())
                    .unwrap_or_else(|| Utc::now().to_rfc3339());
                let recorded_at = sanitized
                    .commands
                    .first()
                    .map(|command| command.recorded_at.clone())
                    .unwrap_or_else(|| "not_observed".into());
                let event = agent_evidence_workbench::v2::RunEventV2 {
                    schema_version: "aew.run-event.v2".into(),
                    event_id: format!("evaluation-{}", report.canonical_digest),
                    kind: "release_truth_evaluated".into(),
                    payload: serde_json::json!({
                        "input": sanitized,
                        "report": report.clone(),
                        "redaction_count": redaction_count,
                    }),
                    observed_at,
                    recorded_at,
                };
                recorded_event = Some(storage::append_v2_event(&cwd, &run_id, &event)?);
            }
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "report": report,
                    "redaction_count": redaction_count,
                    "recorded_event": recorded_event,
                }))?
            );
        }
        Commands::Sign { run_id, key_file } => {
            let r = load(&cwd, &run_id)?;
            let k = receipt::parse_key_file(&key_file)?;
            let x = receipt::sign(&run_id, &r, &k, None);
            let p = receipt::receipt_path(&cwd, &run_id);
            fs::create_dir_all(p.parent().unwrap())?;
            fs::write(&p, serde_json::to_vec_pretty(&x)?)?;
            println!(
                "signed receipt {} report_digest={}",
                p.display(),
                x.report_digest
            );
        }
        Commands::VerifyReceipt { run_id, key_file } => {
            let r = load(&cwd, &run_id)?;
            let k = receipt::parse_key_file(&key_file)?;
            let x: receipt::RunReceiptV1 =
                serde_json::from_slice(&fs::read(receipt::receipt_path(&cwd, &run_id))?)?;
            let valid = receipt::verify(&x, &k, &r);
            println!(
                "valid={} receipt_id={} run_id={} report_digest={} receipt_digest={}",
                valid,
                x.receipt_id,
                x.run_id,
                x.report_digest,
                receipt::receipt_digest(&x)
            );
            if !valid {
                anyhow::bail!("receipt verification failed")
            }
        }
        Commands::Promote {
            run_id,
            memory_dir,
            key_file,
        } => {
            let key = receipt::parse_key_file(&key_file)?;
            promote(&cwd, &run_id, &memory_dir, &key).await?
        }
    }
    Ok(())
}
#[cfg(feature = "semantic-memory")]
async fn promote(cwd: &Path, id: &str, dir: &Path, key: &[u8]) -> anyhow::Result<()> {
    let r = load(cwd, id)?;
    let x: receipt::RunReceiptV1 =
        serde_json::from_slice(&fs::read(receipt::receipt_path(cwd, id))?)?;
    if !receipt::verify(&x, key, &r) {
        anyhow::bail!("receipt invalid or does not match report")
    };
    let cfg = semantic_memory::MemoryConfig {
        base_dir: dir.to_path_buf(),
        ..Default::default()
    };
    let store = semantic_memory::MemoryStore::open(cfg)?;
    for c in r
        .claims
        .iter()
        .filter(|c| c.status == ClaimStatus::Verified)
    {
        let meta = serde_json::json!({
            "run_id": id,
            "claim_id": c.id,
            "receipt_digest": receipt::receipt_digest(&x),
            "verified": true
        });
        let fid = store
            .add_fact("agent-evidence-workbench", &c.text, Some("aew"), Some(meta))
            .await?;
        println!("{}", fid)
    }
    Ok(())
}
#[cfg(not(feature = "semantic-memory"))]
async fn promote(_: &Path, _: &str, _: &Path, _: &[u8]) -> anyhow::Result<()> {
    anyhow::bail!("promote requires cargo feature semantic-memory")
}
