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

/// How a response relates to the causal graph.
///
/// `causal_claim` is true only for [`GraphBasis::Consulted`]: a response that
/// did not derive causal evidence from a graph must not be labelled as causal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum GraphBasis {
    /// No database exists at the requested path (cold start); nothing was read.
    Absent,
    /// A graph was loaded but the request did not query it.
    NotConsulted,
    /// A graph was loaded, but it holds no causal edges at all.
    EmptyGraph,
    /// A graph was loaded and searched, but no requested cause node is in it.
    NodeNotInGraph,
    /// A requested cause node is in the graph, but no causal edge contributed.
    NoCausalEdges,
    /// A graph was loaded and the response was derived from it.
    Consulted,
}

impl GraphBasis {
    fn as_str(self) -> &'static str {
        match self {
            GraphBasis::Absent => "absent",
            GraphBasis::NotConsulted => "not_consulted",
            GraphBasis::EmptyGraph => "empty_graph",
            GraphBasis::NodeNotInGraph => "node_not_in_graph",
            GraphBasis::NoCausalEdges => "no_causal_edges",
            GraphBasis::Consulted => "consulted",
        }
    }

    fn causal_claim(self) -> bool {
        matches!(self, GraphBasis::Consulted)
    }
}

/// Attach the evidence labels every response carries.
fn with_evidence_labels(mut value: serde_json::Value, basis: GraphBasis) -> serde_json::Value {
    if let Some(map) = value.as_object_mut() {
        map.insert("evidence_kind".into(), EVIDENCE_KIND.into());
        map.insert("graph_state".into(), basis.as_str().into());
        map.insert("cold_start".into(), (basis == GraphBasis::Absent).into());
        map.insert("causal_claim".into(), basis.causal_claim().into());
    }
    value
}

/// Neutral prediction returned whenever no causal graph was consulted.
fn neutral_prediction() -> cea_core::CausalPrediction {
    cea_core::CausalPrediction {
        predicted_correctness: 0.5,
        predicted_novelty: 1.0,
        confidence: 0.0,
        coverage_fraction: 0.0,
        risk_flags: Vec::new(),
        zero_shot_eligible: false,
    }
}

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
    write_stdout_json(&predict_response(&request, graph.as_ref(), &db_path))
}

/// Build the `predict` response. Pure: performs no I/O.
fn predict_response(
    request: &PredictRequest,
    graph: Option<&cea_core::CausalGraph>,
    db_path: &Path,
) -> serde_json::Value {
    let (prediction, evidence, basis) = match graph {
        // Cold start: no database exists yet. Neutral prediction, no error.
        None => (
            neutral_prediction(),
            cea_core::PredictionEvidence::default(),
            GraphBasis::Absent,
        ),
        // No signatures requested: neutral result without querying the graph.
        Some(_) if request.signatures.is_empty() => (
            neutral_prediction(),
            cea_core::PredictionEvidence::default(),
            GraphBasis::NotConsulted,
        ),
        Some(graph) => {
            let config = cea_core::PredictionConfig {
                risk_confidence_threshold: request.risk_confidence_threshold.unwrap_or(0.65),
                zero_shot_coverage_threshold: request.zero_shot_coverage_threshold.unwrap_or(0.6),
                ..cea_core::PredictionConfig::default()
            };
            let (prediction, evidence) =
                cea_core::predict_with_evidence(&request.signatures, graph, &config);
            // Causal only when an edge actually contributed; otherwise say what was found.
            let basis = if evidence.contributing_edges > 0 {
                GraphBasis::Consulted
            } else if evidence.matched_signatures > 0 {
                GraphBasis::NoCausalEdges
            } else {
                GraphBasis::NodeNotInGraph
            };
            (prediction, evidence, basis)
        }
    };

    with_evidence_labels(
        serde_json::json!({
            "prediction": prediction,
            "signature_count": request.signatures.len(),
            "matched_signatures": evidence.matched_signatures,
            "exact_matches": evidence.exact_matches,
            "contributing_edges": evidence.contributing_edges,
            "db_path": db_path,
        }),
        basis,
    )
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
    write_stdout_json(&graph_stats_response(graph.as_ref(), &db_path))
}

/// Build the `graph-stats` response. Pure: performs no I/O.
fn graph_stats_response(
    graph: Option<&cea_core::CausalGraph>,
    db_path: &Path,
) -> serde_json::Value {
    let (summary, basis) = match graph {
        Some(graph) => {
            let summary = graph.coverage_summary();
            let basis = if summary.total_edges > 0 {
                GraphBasis::Consulted
            } else {
                GraphBasis::EmptyGraph
            };
            (summary, basis)
        }
        None => (
            cea_core::CoverageSummary {
                total_cause_nodes: 0,
                total_effect_nodes: 0,
                total_edges: 0,
                mean_confidence: 0.0,
            },
            GraphBasis::Absent,
        ),
    };

    with_evidence_labels(
        serde_json::json!({
            "coverage": summary,
            "db_path": db_path,
        }),
        basis,
    )
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

    write_stdout_json(&inspect_response(
        &request.signature,
        graph.as_ref(),
        &db_path,
    ))
}

/// Build the `inspect-signature` response. Pure: performs no I/O.
///
/// `graph_state` says exactly what was found: no database (`absent`), the
/// cause node is not in the graph (`node_not_in_graph`), the node exists but
/// has no causal edges (`no_causal_edges`), or causal edges were returned
/// (`consulted`). Only the last is a causal claim.
fn inspect_response(
    signature: &cea_core::EditOpSignature,
    graph: Option<&cea_core::CausalGraph>,
    db_path: &Path,
) -> serde_json::Value {
    let node_id = cea_core::edit_op_node_id(signature);
    let node_index = graph.and_then(|graph| graph.node_index_map.get(&node_id).copied());
    let edges = match (graph, node_index) {
        (Some(graph), Some(index)) => graph
            .outgoing_edges(index)
            .into_iter()
            .filter_map(
                |(target_index, edge)| match graph.graph.node_weight(target_index)? {
                    cea_core::CausalNode::Effect(effect) => Some(serde_json::json!({
                        "effect": effect,
                        "weight": edge.weight,
                        "count": edge.count,
                        "confidence": edge.confidence,
                        "alpha": edge.stats.alpha,
                        "beta": edge.stats.beta,
                        "observations": edge.stats.observations,
                    })),
                    _ => None,
                },
            )
            .collect::<Vec<_>>(),
        _ => Vec::new(),
    };
    let basis = match (graph, node_index) {
        (None, _) => GraphBasis::Absent,
        (Some(_), None) => GraphBasis::NodeNotInGraph,
        (Some(_), Some(_)) if edges.is_empty() => GraphBasis::NoCausalEdges,
        (Some(_), Some(_)) => GraphBasis::Consulted,
    };

    with_evidence_labels(
        serde_json::json!({
            "node_id": node_id,
            "found": node_index.is_some(),
            "edges": edges,
            "db_path": db_path,
        }),
        basis,
    )
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

    fn predict_request(signatures: serde_json::Value) -> PredictRequest {
        serde_json::from_value(serde_json::json!({ "signatures": signatures })).unwrap()
    }

    fn one_signature() -> serde_json::Value {
        serde_json::json!([{
            "op_kind": "insert", "anchor_kind": "after_line", "lines_added": 1,
            "lines_removed": 0, "context_hash": "abc", "file_extension": "rs",
            "scope_tag": "fn", "op_index": 0, "file_index": 0
        }])
    }

    #[test]
    fn only_a_consulted_graph_is_a_causal_claim() {
        assert!(GraphBasis::Consulted.causal_claim());
        assert!(!GraphBasis::NotConsulted.causal_claim());
        assert!(!GraphBasis::EmptyGraph.causal_claim());
        assert!(!GraphBasis::NodeNotInGraph.causal_claim());
        assert!(!GraphBasis::NoCausalEdges.causal_claim());
        assert!(!GraphBasis::Absent.causal_claim());
    }

    fn signature(context_hash: &str) -> cea_core::EditOpSignature {
        let mut value = one_signature()[0].clone();
        value["context_hash"] = context_hash.into();
        serde_json::from_value(value).unwrap()
    }

    fn effect() -> check_runner::EffectSignature {
        check_runner::EffectSignature {
            check_kind: "test".into(),
            outcome: "fail".into(),
            severity: "error".into(),
            message_class: "assertion".into(),
            line_offset_from_edit: None,
        }
    }

    fn inspect_state(response: &serde_json::Value) -> (&str, bool, bool, usize) {
        (
            response["graph_state"].as_str().unwrap(),
            response["causal_claim"].as_bool().unwrap(),
            response["found"].as_bool().unwrap(),
            response["edges"].as_array().unwrap().len(),
        )
    }

    /// A graph with one cause node that has an edge ("with-edge") and one that has none ("no-edges").
    fn small_graph() -> cea_core::CausalGraph {
        let mut graph = cea_core::CausalGraph::new();
        let with_edge = graph.ensure_cause_node(&signature("with-edge"));
        let effect_index = graph.ensure_effect_node(&effect());
        graph.update_edge(with_edge, effect_index, 1.0);
        graph.ensure_cause_node(&signature("no-edges"));
        graph
    }

    /// A signature that shares nothing with the graph's nodes, so not even a fuzzy match.
    fn unrelated_signature() -> serde_json::Value {
        serde_json::json!({
            "op_kind": "delete", "anchor_kind": "before_line", "lines_added": 0,
            "lines_removed": 40, "context_hash": "unrelated", "file_extension": "py",
            "scope_tag": "mod", "op_index": 7, "file_index": 3
        })
    }

    #[test]
    fn predict_says_exactly_what_it_found() {
        let db = Path::new("/nonexistent/forge.db");
        let graph = small_graph();
        let empty = cea_core::CausalGraph::new();
        let with_edge = serde_json::json!([serde_json::to_value(signature("with-edge")).unwrap()]);
        let no_edges = serde_json::json!([serde_json::to_value(signature("no-edges")).unwrap()]);
        let unrelated = serde_json::json!([unrelated_signature()]);
        let similar = serde_json::json!([serde_json::to_value(signature("similar")).unwrap()]);
        // A graph whose only cause node has no edges: a similar signature matches it, but no
        // causal edge can contribute, so the answer must not be `consulted`.
        let mut edgeless = cea_core::CausalGraph::new();
        edgeless.ensure_cause_node(&signature("no-edges"));
        // (graph_state, causal_claim, matched_signatures, exact_matches, contributing_edges)
        let cases = [
            (
                &empty,
                with_edge.clone(),
                ("node_not_in_graph", false, 0, 0, 0),
            ),
            (&graph, unrelated, ("node_not_in_graph", false, 0, 0, 0)),
            (&graph, no_edges, ("no_causal_edges", false, 1, 1, 0)),
            (&graph, with_edge, ("consulted", true, 1, 1, 1)),
            // Not in the graph, but similar to a node that has an edge: a fuzzy match.
            (&graph, similar.clone(), ("consulted", true, 1, 0, 1)),
            // Similar only to a node with no edges: matched, but no causal evidence.
            (&edgeless, similar, ("no_causal_edges", false, 1, 0, 0)),
        ];
        for (graph, signatures, expected) in cases {
            let response = predict_response(&predict_request(signatures), Some(graph), db);
            let seen = (
                response["graph_state"].as_str().unwrap(),
                response["causal_claim"].as_bool().unwrap(),
                response["matched_signatures"].as_u64().unwrap(),
                response["exact_matches"].as_u64().unwrap(),
                response["contributing_edges"].as_u64().unwrap(),
            );
            assert_eq!(seen, expected);
            assert_eq!(response["cold_start"], false);
        }
    }

    /// `predict_with_config` on a fixed graph, with the values captured from the
    /// base commit before this card. Guards against `predict_with_evidence`
    /// changing the prediction itself.
    #[test]
    fn predict_with_config_output_is_unchanged_from_the_base() {
        let graph = small_graph();
        let config = cea_core::PredictionConfig::default();
        let golden = [
            ("with-edge", 0.0, 0.0, 1.0),
            ("no-edges", 0.5, 0.0, 1.0),
            ("missing", 0.0, 0.19999999999999996, 0.8),
            ("with-edge+missing", 0.0, 0.09999999999999998, 0.9),
        ];
        for (hashes, correctness, novelty, coverage) in golden {
            let want = serde_json::json!({
                "predicted_correctness": correctness, "predicted_novelty": novelty,
                "confidence": 0.0, "coverage_fraction": coverage,
                "risk_flags": [], "zero_shot_eligible": false
            });
            let signatures: Vec<_> = hashes.split('+').map(signature).collect();
            let plain = cea_core::predict_with_config(&signatures, &graph, &config);
            let (with_evidence, _) = cea_core::predict_with_evidence(&signatures, &graph, &config);
            assert_eq!(serde_json::to_value(&plain).unwrap(), want, "{hashes}");
            assert_eq!(
                serde_json::to_value(&with_evidence).unwrap(),
                want,
                "{hashes}"
            );
        }
    }

    #[test]
    fn graph_stats_of_an_empty_graph_is_not_a_causal_claim() {
        let db = Path::new("/nonexistent/forge.db");
        let empty = graph_stats_response(Some(&cea_core::CausalGraph::new()), db);
        assert_eq!(empty["graph_state"], "empty_graph");
        assert_eq!(empty["causal_claim"], false);
        assert_eq!(empty["cold_start"], false);
        let graph = small_graph();
        let populated = graph_stats_response(Some(&graph), db);
        assert_eq!(populated["graph_state"], "consulted");
        assert_eq!(populated["causal_claim"], true);
    }

    #[test]
    fn inspect_signature_says_exactly_what_it_found() {
        let db = Path::new("/nonexistent/forge.db");
        let graph = small_graph();

        let cases = [
            (None, "with-edge", ("absent", false, false, 0)),
            (
                Some(&graph),
                "not-in-graph",
                ("node_not_in_graph", false, false, 0),
            ),
            (
                Some(&graph),
                "no-edges",
                ("no_causal_edges", false, true, 0),
            ),
            (Some(&graph), "with-edge", ("consulted", true, true, 1)),
        ];
        for (graph, hash, expected) in cases {
            let response = inspect_response(&signature(hash), graph, db);
            assert_eq!(inspect_state(&response), expected, "{hash}");
            assert_eq!(response["cold_start"], graph.is_none(), "{hash}");
        }
    }

    #[test]
    fn predict_without_database_is_cold_start_not_a_causal_claim() {
        for signatures in [serde_json::json!([]), one_signature()] {
            let response = predict_response(
                &predict_request(signatures),
                None,
                Path::new("/nonexistent/forge.db"),
            );
            assert_eq!(response["causal_claim"], false);
            assert_eq!(response["cold_start"], true);
            assert_eq!(response["graph_state"], "absent");
            assert_eq!(response["evidence_kind"], EVIDENCE_KIND);
            assert_eq!(response["prediction"]["confidence"], 0.0);
            assert_eq!(response["prediction"]["coverage_fraction"], 0.0);
        }
    }

    #[test]
    fn graph_stats_without_database_is_cold_start_not_a_causal_claim() {
        let response = graph_stats_response(None, Path::new("/nonexistent/forge.db"));
        assert_eq!(response["causal_claim"], false);
        assert_eq!(response["cold_start"], true);
        assert_eq!(response["graph_state"], "absent");
        assert_eq!(response["coverage"]["total_edges"], 0);
    }
}
