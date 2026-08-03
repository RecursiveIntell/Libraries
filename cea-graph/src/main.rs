//! cea-graph — read-only JSON stdin/stdout bridge for the real CEA causal graph.
//!
//! Reads forge-engine databases (or any cea-sqlite-compatible database) and
//! exposes causal risk prediction, graph statistics, and signature inspection.
//! This binary is STRICTLY read-only: it never writes to the CEA database.
//!
//! This is the "real causal lane" — distinct from cea-bridge (synthetic
//! telemetry, quarantined). Responses carry `evidence_kind = "causal_graph"`
//! and `causal_claim = true`.

use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};

use rusqlite::{Connection, OpenFlags};
use serde::Deserialize;
const EVIDENCE_KIND: &str = "causal_graph";

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("usage: cea-graph <command> [--db <path>] [--version <version_id>]");
        eprintln!("commands: predict, graph-stats, inspect-signature");
        std::process::exit(1);
    }
    let command = &args[1];
    let db_path = extract_named_path(&args[2..], "--db");
    let version_id = extract_named_value(&args[2..], "--version");

    let result = match command.as_str() {
        "predict" => cmd_predict(db_path.as_deref(), version_id.as_deref()),
        "graph-stats" => cmd_graph_stats(db_path.as_deref(), version_id.as_deref()),
        "inspect-signature" => cmd_inspect_signature(db_path.as_deref(), version_id.as_deref()),
        _ => Err(format!("unknown command: {command}").into()),
    };

    if let Err(error) = result {
        eprintln!("error: {error}");
        std::process::exit(1);
    }
}

fn extract_named_value(args: &[String], flag: &str) -> Option<String> {
    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        if arg == flag {
            return iter.next().cloned();
        }
        if let Some(value) = arg.strip_prefix(&format!("{flag}=")) {
            return Some(value.to_string());
        }
    }
    None
}

fn extract_named_path(args: &[String], flag: &str) -> Option<PathBuf> {
    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        if arg == flag {
            return iter.next().map(PathBuf::from);
        }
        if let Some(value) = arg.strip_prefix(&format!("{flag}=")) {
            return Some(PathBuf::from(value));
        }
    }
    None
}

fn read_stdin_json() -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;
    if input.trim().is_empty() {
        Ok(serde_json::Value::Null)
    } else {
        Ok(serde_json::from_str(&input)?)
    }
}

fn write_stdout_json(value: &serde_json::Value) -> Result<(), Box<dyn std::error::Error>> {
    let mut stdout = io::stdout();
    serde_json::to_writer_pretty(&mut stdout, value)?;
    stdout.write_all(b"\n")?;
    Ok(())
}

/// Open a READ-ONLY connection to a CEA-compatible database.
///
/// Uses the raw-connection pattern from forge-engine's `cea/store.rs`: wrap
/// an existing connection in `SqliteCeaStoreConn`. This deliberately bypasses
/// `SqliteCeaStore::open()` schema-version checks because forge-engine
/// databases carry their own `user_version` (e.g. 5) that would be rejected
/// by cea-sqlite's own schema gate. Read-only flag guarantees no mutation.
///
/// Returns `Ok(None)` when the database does not exist — callers use this as
/// the cold-start signal (fail-open, never an error).
fn open_read_only(path: &Path) -> Result<Option<Connection>, Box<dyn std::error::Error>> {
    if !path.exists() {
        return Ok(None);
    }
    let conn = Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY)?;
    Ok(Some(conn))
}

/// Load the causal graph from a database.
///
/// Returns `Ok(None)` when the database does not exist (cold start).
fn load_graph_from_db(
    path: &Path,
    version_id: Option<&str>,
) -> Result<Option<cea_core::CausalGraph>, Box<dyn std::error::Error>> {
    let Some(conn) = open_read_only(path)? else {
        return Ok(None);
    };
    let store = cea_sqlite::SqliteCeaStoreConn::new(&conn);
    let graph = cea_store::load_graph(&store, version_id)?;
    Ok(Some(graph))
}

#[derive(Debug, Deserialize)]
struct PredictRequest {
    /// Edit operation signatures to predict risk for.
    signatures: Vec<cea_core::EditOpSignature>,
    /// Optional risk confidence threshold (default 0.65).
    #[serde(default)]
    risk_confidence_threshold: Option<f64>,
    /// Optional zero-shot coverage threshold (default 0.6).
    #[serde(default)]
    zero_shot_coverage_threshold: Option<f64>,
    /// Optional database path (overrides --db flag).
    #[serde(default)]
    db_path: Option<String>,
    /// Optional version filter.
    #[serde(default)]
    version_id: Option<String>,
}

fn cmd_predict(
    db_flag: Option<&Path>,
    version_flag: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    let request: PredictRequest = serde_json::from_value(read_stdin_json()?)?;

    let db_path = request
        .db_path
        .as_deref()
        .map(PathBuf::from)
        .or_else(|| db_flag.map(PathBuf::from))
        .ok_or_else(|| {
            "predict requires a database path (--db flag or db_path field)".to_string()
        })?;

    let version_id = request.version_id.as_deref().or(version_flag);
    let graph = load_graph_from_db(&db_path, version_id)?;

    let prediction = match graph {
        None => {
            // Cold start: no database exists yet. Neutral prediction, no error.
            cea_core::CausalPrediction {
                predicted_correctness: 0.5,
                predicted_novelty: 1.0,
                confidence: 0.0,
                coverage_fraction: 0.0,
                risk_flags: Vec::new(),
                zero_shot_eligible: false,
            }
        }
        Some(_graph) if request.signatures.is_empty() => {
            // No signatures requested: neutral result without needing the graph.
            cea_core::CausalPrediction {
                predicted_correctness: 0.5,
                predicted_novelty: 1.0,
                confidence: 0.0,
                coverage_fraction: 0.0,
                risk_flags: Vec::new(),
                zero_shot_eligible: false,
            }
        }
        Some(graph) => {
            let config = cea_core::PredictionConfig {
                risk_confidence_threshold: request.risk_confidence_threshold.unwrap_or(0.65),
                zero_shot_coverage_threshold: request.zero_shot_coverage_threshold.unwrap_or(0.6),
                ..cea_core::PredictionConfig::default()
            };
            cea_core::predict_with_config(&request.signatures, &graph, &config)
        }
    };

    write_stdout_json(&serde_json::json!({
        "prediction": prediction,
        "signature_count": request.signatures.len(),
        "db_path": db_path,
        "evidence_kind": EVIDENCE_KIND,
        "causal_claim": true,
    }))
}

fn cmd_graph_stats(
    db_flag: Option<&Path>,
    version_flag: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    let request: serde_json::Value = read_stdin_json()?;
    let db_path = request
        .get("db_path")
        .and_then(|v| v.as_str())
        .map(PathBuf::from)
        .or_else(|| db_flag.map(PathBuf::from))
        .ok_or_else(|| "graph-stats requires a database path".to_string())?;
    let version_id = request
        .get("version_id")
        .and_then(|v| v.as_str())
        .or(version_flag);

    let graph = load_graph_from_db(&db_path, version_id)?;
    let summary =
        graph
            .map(|graph| graph.coverage_summary())
            .unwrap_or(cea_core::CoverageSummary {
                total_cause_nodes: 0,
                total_effect_nodes: 0,
                total_edges: 0,
                mean_confidence: 0.0,
            });

    write_stdout_json(&serde_json::json!({
        "coverage": summary,
        "db_path": db_path,
        "evidence_kind": EVIDENCE_KIND,
        "causal_claim": true,
    }))
}

#[derive(Debug, Deserialize)]
struct InspectRequest {
    /// The edit-op signature to inspect.
    signature: cea_core::EditOpSignature,
    /// Optional database path (overrides --db flag).
    #[serde(default)]
    db_path: Option<String>,
    /// Optional version filter.
    #[serde(default)]
    version_id: Option<String>,
}

fn cmd_inspect_signature(
    db_flag: Option<&Path>,
    version_flag: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    let request: InspectRequest = serde_json::from_value(read_stdin_json()?)?;

    let db_path = request
        .db_path
        .as_deref()
        .map(PathBuf::from)
        .or_else(|| db_flag.map(PathBuf::from))
        .ok_or_else(|| "inspect-signature requires a database path".to_string())?;

    let version_id = request.version_id.as_deref().or(version_flag);
    let graph = load_graph_from_db(&db_path, version_id)?;

    let node_id = cea_core::edit_op_node_id(&request.signature);

    let (found, edges) = match graph {
        None => (false, Vec::new()),
        Some(graph) => {
            let node_index = graph.node_index_map.get(&node_id).copied();
            let edges = node_index
                .map(|index| {
                    graph
                        .outgoing_edges(index)
                        .into_iter()
                        .filter_map(|(target_index, edge)| {
                            let target = graph.graph.node_weight(target_index)?;
                            match target {
                                cea_core::CausalNode::Effect(signature) => {
                                    Some(serde_json::json!({
                                        "effect": signature,
                                        "weight": edge.weight,
                                        "count": edge.count,
                                        "confidence": edge.confidence,
                                        "alpha": edge.stats.alpha,
                                        "beta": edge.stats.beta,
                                        "observations": edge.stats.observations,
                                    }))
                                }
                                _ => None,
                            }
                        })
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();
            (node_index.is_some(), edges)
        }
    };

    write_stdout_json(&serde_json::json!({
        "node_id": node_id,
        "found": found,
        "edges": edges,
        "db_path": db_path,
        "evidence_kind": EVIDENCE_KIND,
        "causal_claim": true,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_named_path_handles_separate_and_equals_forms() {
        let args = vec!["--db".to_string(), "/tmp/x.db".to_string()];
        assert_eq!(
            extract_named_path(&args, "--db"),
            Some(PathBuf::from("/tmp/x.db"))
        );
        let args = vec!["--db=/tmp/y.db".to_string()];
        assert_eq!(
            extract_named_path(&args, "--db"),
            Some(PathBuf::from("/tmp/y.db"))
        );
        assert_eq!(extract_named_path(&args, "--version"), None);
    }

    #[test]
    fn open_read_only_returns_none_on_missing_db() {
        let result = open_read_only(Path::new("/nonexistent/forge.db")).unwrap();
        assert!(result.is_none());
    }
}
