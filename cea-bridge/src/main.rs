//! Fail-open JSON stdin/stdout bridge for Hermes tool telemetry.
//!
//! This crate deliberately does not store code-edit signatures or causal graphs.
//! Tool telemetry is synthetic advisory evidence, never causal proof.

use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};

use rusqlite::{params, Connection, OpenFlags};
use serde::{Deserialize, Serialize};

const TELEMETRY_VERSION: &str = "telemetry-v2";
const EVIDENCE_KIND: &str = "synthetic_telemetry";

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("usage: cea-bridge <command> [--db <telemetry-db>]");
        eprintln!("commands: score-relevance, record-telemetry, record-triple, query-provenance, graph-stats, inspect-legacy");
        std::process::exit(1);
    }
    let command = &args[1];
    let result = if command == "inspect-legacy" {
        let legacy = extract_legacy_db_path(&args[2..]).or_else(|| extract_db_path(&args[2..]));
        match legacy {
            Some(path) => cmd_inspect_legacy(&path),
            None => Err("inspect-legacy requires --legacy-db <path>".into()),
        }
    } else {
        let db_path = extract_db_path(&args[2..]).unwrap_or_else(default_db_path);
        match command.as_str() {
            "score-relevance" => cmd_score_relevance(&db_path),
            // Kept only so existing callers do not fail; it writes telemetry, not triples.
            "record-telemetry" | "record-triple" => cmd_record_telemetry(&db_path),
            // Kept only so existing callers receive an explicit quarantine warning.
            "query-provenance" => cmd_query_provenance(&db_path),
            "graph-stats" => cmd_graph_stats(&db_path),
            _ => Err(format!("unknown command: {command}").into()),
        }
    };
    if let Err(error) = result {
        eprintln!("error: {error}");
        std::process::exit(1);
    }
}

fn default_db_path() -> PathBuf {
    PathBuf::from(std::env::var("HERMES_HOME").unwrap_or_else(|_| {
        format!(
            "{}/.hermes",
            std::env::var("HOME").unwrap_or_else(|_| ".".to_string())
        )
    }))
    .join("cea-telemetry-v2.db")
}

fn extract_db_path(args: &[String]) -> Option<PathBuf> {
    extract_named_path(args, "--db")
}

fn extract_legacy_db_path(args: &[String]) -> Option<PathBuf> {
    extract_named_path(args, "--legacy-db")
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

fn blake3_hash(input: &str) -> String {
    blake3::hash(input.as_bytes()).to_hex().to_string()
}

fn normalize_tool_name(name: &str) -> String {
    let normalized: String = name
        .chars()
        .filter_map(|ch| {
            if ch.is_ascii_alphanumeric() {
                Some(ch.to_ascii_lowercase())
            } else if matches!(ch, '_' | '-' | '.') {
                Some('_')
            } else {
                None
            }
        })
        .take(96)
        .collect();
    if normalized.is_empty() {
        "unknown".to_string()
    } else {
        normalized
    }
}

fn normalize_outcome(outcome: &str) -> &'static str {
    match outcome.to_ascii_lowercase().as_str() {
        "success" => "success",
        "error" => "error",
        _ => "unknown",
    }
}

fn valid_digest(digest: &str) -> bool {
    digest.len() == 64 && digest.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn open_telemetry(path: &Path) -> Result<Connection, Box<dyn std::error::Error>> {
    if path.file_name().and_then(|name| name.to_str()) == Some("cea.db") {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "refusing to open legacy cea.db as a telemetry database; use cea-telemetry-v2.db",
        )
        .into());
    }

    if path.exists() {
        let inspection = Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY)?;
        let legacy_table_count: i64 = inspection.query_row(
            "SELECT COUNT(*) FROM sqlite_master
             WHERE type = 'table' AND name IN ('cea_nodes', 'cea_edges', 'cea_run_log')",
            [],
            |row| row.get(0),
        )?;
        if legacy_table_count > 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "refusing to add synthetic telemetry tables to a legacy code-attribution database",
            )
            .into());
        }
    }

    let conn = Connection::open(path)?;
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS telemetry_events (
            event_id TEXT PRIMARY KEY NOT NULL,
            session_digest TEXT NOT NULL,
            tool_call_digest TEXT NOT NULL,
            result_digest TEXT NOT NULL,
            tool_name TEXT NOT NULL,
            tool_args_digest TEXT NOT NULL,
            outcome TEXT NOT NULL CHECK(outcome IN ('success','error','unknown')),
            error_class TEXT,
            created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
        );
        CREATE TABLE IF NOT EXISTS telemetry_aggregates (
            tool_name TEXT NOT NULL,
            outcome TEXT NOT NULL CHECK(outcome IN ('success','error','unknown')),
            event_count INTEGER NOT NULL DEFAULT 0,
            PRIMARY KEY (tool_name, outcome)
        );
        CREATE TABLE IF NOT EXISTS telemetry_metadata (
            key TEXT PRIMARY KEY NOT NULL,
            value TEXT NOT NULL
        );
        INSERT OR IGNORE INTO telemetry_metadata(key, value) VALUES ('schema_version', 'telemetry-v2');",
    )?;
    Ok(conn)
}

#[derive(Debug, Deserialize)]
struct TelemetryRequest {
    session_id: String,
    tool_call_id: String,
    result_digest: String,
    tool_name: String,
    #[serde(default)]
    tool_args: Option<String>,
    outcome: String,
    #[serde(default)]
    error_class: Option<String>,
}

impl TelemetryRequest {
    #[cfg(test)]
    fn test_event(tool_call_id: &str, outcome: &str) -> Self {
        Self {
            session_id: "session".to_string(),
            tool_call_id: tool_call_id.to_string(),
            result_digest: blake3_hash("result"),
            tool_name: "terminal".to_string(),
            tool_args: None,
            outcome: outcome.to_string(),
            error_class: None,
        }
    }
}

fn record_telemetry(
    path: &Path,
    request: &TelemetryRequest,
) -> Result<bool, Box<dyn std::error::Error>> {
    if request.session_id.is_empty()
        || request.tool_call_id.is_empty()
        || !valid_digest(&request.result_digest)
    {
        return Err("session_id, tool_call_id, and a 64-hex result_digest are required".into());
    }
    let session_digest = blake3_hash(&request.session_id);
    let tool_call_digest = blake3_hash(&request.tool_call_id);
    let event_id = blake3_hash(&format!(
        "{}\0{}\0{}",
        request.session_id, request.tool_call_id, request.result_digest
    ));
    let args_digest = blake3_hash(request.tool_args.as_deref().unwrap_or(""));
    let tool_name = normalize_tool_name(&request.tool_name);
    let outcome = normalize_outcome(&request.outcome);
    let error_class = request.error_class.as_deref().map(normalize_tool_name);
    let mut conn = open_telemetry(path)?;
    let transaction = conn.transaction()?;
    let inserted = transaction.execute(
        "INSERT OR IGNORE INTO telemetry_events
         (event_id, session_digest, tool_call_digest, result_digest, tool_name, tool_args_digest, outcome, error_class)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
        params![event_id, session_digest, tool_call_digest, request.result_digest.to_ascii_lowercase(), tool_name, args_digest, outcome, error_class],
    )? == 1;
    if inserted {
        transaction.execute(
            "INSERT INTO telemetry_aggregates(tool_name, outcome, event_count) VALUES (?, ?, 1)
             ON CONFLICT(tool_name, outcome) DO UPDATE SET event_count = event_count + 1",
            params![normalize_tool_name(&request.tool_name), outcome],
        )?;
    }
    transaction.commit()?;
    Ok(inserted)
}

fn cmd_record_telemetry(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let request: TelemetryRequest = serde_json::from_value(read_stdin_json()?)?;
    let inserted = record_telemetry(path, &request)?;
    write_stdout_json(&serde_json::json!({
        "status": "ok", "inserted": inserted, "evidence_kind": EVIDENCE_KIND,
        "causal_claim": false, "reason": "synthetic telemetry recorded; not causal evidence"
    }))
}

#[derive(Debug, Deserialize)]
struct ScoreRequest {
    messages: Vec<MessageInfo>,
    #[serde(default)]
    focus: Option<String>,
}

#[derive(Debug, Deserialize)]
struct MessageInfo {
    index: usize,
    #[serde(default)]
    tool_name: Option<String>,
}

#[derive(Debug, Serialize)]
struct ScoreResult {
    index: usize,
    relevance_score: f64,
    reason: String,
    evidence_kind: &'static str,
    causal_claim: bool,
}

impl ScoreResult {
    fn cold_start(index: usize, total: usize) -> Self {
        Self {
            index,
            relevance_score: recency_score(index, total),
            reason: "advisory telemetry cold-start: bounded recency only".to_string(),
            evidence_kind: EVIDENCE_KIND,
            causal_claim: false,
        }
    }
}

fn recency_score(index: usize, total: usize) -> f64 {
    if total <= 1 {
        return 0.9;
    }
    (0.2 + (index as f64 / (total - 1) as f64) * 0.7).clamp(0.2, 0.9)
}

fn cmd_score_relevance(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let request: ScoreRequest = serde_json::from_value(read_stdin_json()?)?;
    let conn = open_telemetry(path)?;
    let output: Result<Vec<_>, rusqlite::Error> = request.messages.iter().map(|message| {
        let base = recency_score(message.index, request.messages.len());
        let tool_name = message.tool_name.as_deref().map(normalize_tool_name);
        let (successes, errors, unknowns): (i64, i64, i64) = match tool_name.as_deref() {
            Some(name) => conn.query_row(
                "SELECT COALESCE(SUM(CASE WHEN outcome='success' THEN event_count END), 0),
                        COALESCE(SUM(CASE WHEN outcome='error' THEN event_count END), 0),
                        COALESCE(SUM(CASE WHEN outcome='unknown' THEN event_count END), 0)
                 FROM telemetry_aggregates WHERE tool_name = ?",
                [name], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )?,
            None => (0, 0, 0),
        };
        let known = successes + errors;
        if known < 3 {
            return Ok(ScoreResult::cold_start(message.index, request.messages.len()));
        }
        let error_rate = errors as f64 / known as f64;
        let focus_overlap = request.focus.as_deref().zip(tool_name.as_deref()).map(|(focus, name)| focus.to_ascii_lowercase().contains(name)).unwrap_or(false);
        let score = (base * 0.8 + error_rate * 0.15 + if focus_overlap { 0.05 } else { 0.0 }).clamp(0.0, 1.0);
        Ok(ScoreResult { index: message.index, relevance_score: score, reason: format!("advisory telemetry: sample-qualified tool/error rates (known={known}, unknown={unknowns})"), evidence_kind: EVIDENCE_KIND, causal_claim: false })
    }).collect();
    write_stdout_json(&serde_json::to_value(output?)?)
}

#[derive(Debug, Deserialize)]
struct ProvenanceRequest {
    query: String,
    #[serde(default = "default_depth")]
    depth: usize,
}
fn default_depth() -> usize {
    5
}

fn cmd_query_provenance(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let request: ProvenanceRequest = serde_json::from_value(read_stdin_json()?)?;
    let conn = open_telemetry(path)?;
    let query = format!("%{}%", normalize_tool_name(&request.query));
    let mut statement = conn.prepare("SELECT tool_name, outcome, event_count FROM telemetry_aggregates WHERE tool_name LIKE ? ORDER BY event_count DESC LIMIT ?")?;
    let history: Vec<serde_json::Value> = statement.query_map(params![query, request.depth.min(100) as i64], |row| Ok(serde_json::json!({"tool_name": row.get::<_, String>(0)?, "outcome": row.get::<_, String>(1)?, "event_count": row.get::<_, i64>(2)?})))?.collect::<Result<_, _>>()?;
    write_stdout_json(&serde_json::json!({
        "warning": "legacy command name: this returns synthetic telemetry history, not provenance or causal proof",
        "history": history, "evidence_kind": EVIDENCE_KIND, "causal_claim": false
    }))
}

fn cmd_graph_stats(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let conn = open_telemetry(path)?;
    let event_count: i64 = conn.query_row("SELECT COUNT(*) FROM telemetry_events", [], |row| {
        row.get(0)
    })?;
    let aggregate_count: i64 =
        conn.query_row("SELECT COUNT(*) FROM telemetry_aggregates", [], |row| {
            row.get(0)
        })?;
    write_stdout_json(
        &serde_json::json!({"telemetry_event_count": event_count, "telemetry_aggregate_count": aggregate_count, "db_path": path, "version_id": TELEMETRY_VERSION, "evidence_kind": EVIDENCE_KIND, "causal_claim": false}),
    )
}

fn quote_identifier(value: &str) -> String {
    format!("\"{}\"", value.replace('"', "\"\""))
}

fn inspect_legacy(path: &Path) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let conn = Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY)?;
    let integrity: String = conn.query_row("PRAGMA integrity_check", [], |row| row.get(0))?;
    let mut statement =
        conn.prepare("SELECT name FROM sqlite_master WHERE type = 'table' ORDER BY name")?;
    let table_names: Vec<String> = statement
        .query_map([], |row| row.get(0))?
        .collect::<Result<_, _>>()?;
    let mut tables = Vec::new();
    let mut version_ids = Vec::new();
    for name in table_names {
        let count: i64 = conn.query_row(
            &format!("SELECT COUNT(*) FROM {}", quote_identifier(&name)),
            [],
            |row| row.get(0),
        )?;
        let mut columns =
            conn.prepare(&format!("PRAGMA table_info({})", quote_identifier(&name)))?;
        let has_version = columns
            .query_map([], |row| row.get::<_, String>(1))?
            .collect::<Result<Vec<_>, _>>()?
            .iter()
            .any(|column| column == "version_id");
        if has_version {
            let mut versions = conn.prepare(&format!(
                "SELECT DISTINCT version_id FROM {} WHERE version_id IS NOT NULL LIMIT 100",
                quote_identifier(&name)
            ))?;
            version_ids.extend(
                versions
                    .query_map([], |row| row.get::<_, String>(0))?
                    .collect::<Result<Vec<_>, _>>()?,
            );
        }
        tables.push(serde_json::json!({"name": name, "count": count}));
    }
    version_ids.sort();
    version_ids.dedup();
    Ok(
        serde_json::json!({"path": path, "schema": tables, "version_ids": version_ids, "integrity": if integrity == "ok" { "ok" } else { "failed" }, "disposition": "quarantined_read_only", "migrated": false, "written": false}),
    )
}

fn cmd_inspect_legacy(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    write_stdout_json(&inspect_legacy(path)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    #[test]
    fn blake3_digests_are_real_and_64_hex_characters() {
        assert_eq!(
            blake3_hash("abc"),
            "6437b3ac38465133ffb63b75273a8db548c558465d79db03fd359c6cd5bd9d85"
        );
        assert_eq!(blake3_hash("abc").len(), 64);
    }
    #[test]
    fn telemetry_identity_is_idempotent_but_distinct_calls_count_independently() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("telemetry.db");
        let event = TelemetryRequest::test_event("call-1", "success");
        record_telemetry(&path, &event).unwrap();
        record_telemetry(&path, &event).unwrap();
        record_telemetry(&path, &TelemetryRequest::test_event("call-2", "success")).unwrap();
        let conn = Connection::open(path).unwrap();
        assert_eq!(
            conn.query_row("SELECT COUNT(*) FROM telemetry_events", [], |r| r
                .get::<_, i64>(0))
                .unwrap(),
            2
        );
    }
    #[test]
    fn telemetry_persists_only_normalized_fields_and_digests() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("telemetry.db");
        let mut event = TelemetryRequest::test_event("call-sensitive", "unknown");
        event.tool_args = Some(r#"{"token":"raw-secret"}"#.to_string());
        record_telemetry(&path, &event).unwrap();
        let conn = Connection::open(path).unwrap();
        let dump: String = conn
            .query_row(
                "SELECT sql FROM sqlite_master WHERE name = 'telemetry_events'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert!(!dump.contains("tool_args,"));
        assert!(!dump.contains("result_output"));
        let digest: String = conn
            .query_row("SELECT tool_args_digest FROM telemetry_events", [], |r| {
                r.get(0)
            })
            .unwrap();
        assert_eq!(digest.len(), 64);
    }
    #[test]
    fn unknown_outcomes_do_not_count_as_success() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("telemetry.db");
        record_telemetry(&path, &TelemetryRequest::test_event("unknown", "unknown")).unwrap();
        let conn = Connection::open(path).unwrap();
        assert_eq!(
            conn.query_row(
                "SELECT COUNT(*) FROM telemetry_aggregates WHERE outcome = 'success'",
                [],
                |r| r.get::<_, i64>(0)
            )
            .unwrap(),
            0
        );
    }
    #[test]
    fn default_db_quarantines_the_legacy_cea_db_name() {
        assert!(default_db_path().ends_with("cea-telemetry-v2.db"));
        assert!(!default_db_path().ends_with("cea.db"));
    }

    #[test]
    fn telemetry_open_refuses_legacy_name_and_schema_without_mutation() {
        let dir = tempfile::tempdir().unwrap();

        let legacy_name = dir.path().join("cea.db");
        assert!(open_telemetry(&legacy_name).is_err());
        assert!(!legacy_name.exists());

        let disguised_legacy = dir.path().join("renamed-legacy.db");
        let conn = Connection::open(&disguised_legacy).unwrap();
        conn.execute_batch(
            "CREATE TABLE cea_nodes (node_id TEXT PRIMARY KEY);
             CREATE TABLE cea_edges (edge_id TEXT PRIMARY KEY);
             CREATE TABLE cea_run_log (run_hash TEXT PRIMARY KEY);",
        )
        .unwrap();
        drop(conn);
        let before = std::fs::read(&disguised_legacy).unwrap();

        assert!(open_telemetry(&disguised_legacy).is_err());
        assert_eq!(std::fs::read(&disguised_legacy).unwrap(), before);
        let conn = Connection::open_with_flags(&disguised_legacy, OpenFlags::SQLITE_OPEN_READ_ONLY)
            .unwrap();
        let telemetry_tables: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name LIKE 'telemetry_%'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(telemetry_tables, 0);
    }

    #[test]
    fn legacy_inspection_is_read_only_and_reports_integrity() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("legacy.db");
        let conn = Connection::open(&path).unwrap();
        conn.execute("CREATE TABLE causal_runs (version_id TEXT)", [])
            .unwrap();
        conn.execute("INSERT INTO causal_runs VALUES ('legacy-v1')", [])
            .unwrap();
        drop(conn);
        let before = std::fs::metadata(&path).unwrap().len();
        let report = inspect_legacy(&path).unwrap();
        assert_eq!(report["disposition"], "quarantined_read_only");
        assert_eq!(report["integrity"], "ok");
        assert_eq!(std::fs::metadata(&path).unwrap().len(), before);
    }
    #[test]
    fn advisory_responses_are_explicitly_non_causal() {
        let result = ScoreResult::cold_start(0, 1);
        assert_eq!(result.evidence_kind, "synthetic_telemetry");
        assert!(!result.causal_claim);
        assert!(result.reason.contains("advisory"));
    }
}
