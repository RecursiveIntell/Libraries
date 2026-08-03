use agent_evidence_workbench::{
    adjudicator::adjudicate,
    cli::{Cli, Commands},
    collector::snapshot_repo,
    extractor::extract_claims,
    model::*,
    receipt,
    report::generate_markdown,
    run::run_command,
    storage,
    verifier::verify_claims,
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
    EvidenceItem {
        id: hex::encode(Sha256::digest(format!("{}:{}", source, content).as_bytes())),
        kind,
        source: source.into(),
        digest: hex::encode(Sha256::digest(content.as_bytes())),
        summary: content.chars().take(200).collect(),
        exit_code: exit,
        duration_ms: dur,
    }
}
async fn resolved(mut r: RunReport) -> anyhow::Result<RunReport> {
    let statuses = verify_claims(&r.claims, &r.checks, &r.diff).await;
    for (c, s) in r.claims.iter_mut().zip(statuses) {
        c.status = s;
    }
    r.verdict = if r
        .claims
        .iter()
        .any(|c| c.status == ClaimStatus::Contradicted)
    {
        RunVerdict::Failed
    } else if r.claims.iter().all(|c| c.status == ClaimStatus::Verified) {
        RunVerdict::Clean
    } else {
        RunVerdict::Partial
    };
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
            let text = fs::read_to_string(&transcript)?;
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
        Commands::Adjudicate { run_id } => {
            let r = load(&cwd, &run_id)?;
            for c in &r.claims {
                println!(
                    "{}: {:?}",
                    c.id,
                    adjudicate(c, &r.evidence_manifest).judgment.support_state
                );
            }
        }
        Commands::Sign { run_id, key_hex } => {
            let r = load(&cwd, &run_id)?;
            let k = receipt::parse_key(&key_hex)?;
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
        Commands::VerifyReceipt { run_id, key_hex } => {
            let r = load(&cwd, &run_id)?;
            let k = receipt::parse_key(&key_hex)?;
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
            key_hex,
        } => promote(&cwd, &run_id, &memory_dir, &key_hex).await?,
    }
    Ok(())
}
#[cfg(feature = "semantic-memory")]
async fn promote(cwd: &Path, id: &str, dir: &Path, key: &str) -> anyhow::Result<()> {
    let r = load(cwd, id)?;
    let k = receipt::parse_key(key)?;
    let x: receipt::RunReceiptV1 =
        serde_json::from_slice(&fs::read(receipt::receipt_path(cwd, id))?)?;
    if !receipt::verify(&x, &k, &r) {
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
async fn promote(_: &Path, _: &str, _: &Path, _: &str) -> anyhow::Result<()> {
    anyhow::bail!("promote requires cargo feature semantic-memory")
}
