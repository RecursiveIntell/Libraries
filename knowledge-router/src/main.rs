use std::io::{self, Read, Write};

use knowledge_runtime::query::classify::classify;
use knowledge_runtime::query::route::plan;
use knowledge_runtime::{ClassifyResult, RoutePlan, Scope};
use serde::{Deserialize, Serialize};

// ── CLI entry ────────────────────────────────────────────────────────

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("usage: knowledge-router <command>");
        eprintln!("commands: classify, route");
        std::process::exit(1);
    }

    let command = &args[1];

    let result = match command.as_str() {
        "classify" => cmd_classify(),
        "route" => cmd_route(),
        _ => {
            eprintln!("unknown command: {command}");
            eprintln!("commands: classify, route");
            std::process::exit(1);
        }
    };

    match result {
        Ok(()) => {}
        Err(e) => {
            eprintln!("error: {e}");
            std::process::exit(1);
        }
    }
}

fn read_stdin_json() -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;
    parse_json_input(&input).map_err(Into::into)
}

fn parse_json_input(input: &str) -> serde_json::Result<serde_json::Value> {
    if input.trim().is_empty() {
        Ok(serde_json::Value::Null)
    } else {
        serde_json::from_str(input)
    }
}

fn write_stdout_json(value: &serde_json::Value) -> Result<(), Box<dyn std::error::Error>> {
    let mut stdout = io::stdout();
    serde_json::to_writer_pretty(&mut stdout, value)?;
    stdout.write_all(b"\n")?;
    Ok(())
}

// ── classify ─────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct ClassifyRequest {
    query: String,
}

fn cmd_classify() -> Result<(), Box<dyn std::error::Error>> {
    let req: ClassifyRequest = serde_json::from_value(read_stdin_json()?)?;
    let result: ClassifyResult = classify(&req.query);
    write_stdout_json(&serde_json::to_value(&result)?)
}

// ── route ────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct RouteRequest {
    query: String,
    #[serde(default = "default_namespace")]
    namespace: String,
    #[serde(default)]
    domain: Option<String>,
    #[serde(default)]
    workspace_id: Option<String>,
    #[serde(default)]
    repo_id: Option<String>,
    #[serde(default = "default_limit")]
    default_limit: usize,
}

fn default_namespace() -> String {
    "general".to_string()
}

fn default_limit() -> usize {
    10
}

fn cmd_route() -> Result<(), Box<dyn std::error::Error>> {
    let req: RouteRequest = serde_json::from_value(read_stdin_json()?)?;

    // Build scope from request fields.
    let mut scope = Scope::new(&req.namespace);
    if let Some(d) = &req.domain {
        scope = scope.with_domain(d);
    }
    if let Some(w) = &req.workspace_id {
        scope = scope.with_workspace(w);
    }
    if let Some(r) = &req.repo_id {
        scope = scope.with_repo(r);
    }

    // Classify the query, then plan the route from the classified mode.
    let classify_result = classify(&req.query);
    let route_plan: RoutePlan = plan(&req.query, &classify_result.mode, &scope, req.default_limit);

    // Build a combined response that includes both the classification and the route.
    #[derive(Debug, Serialize)]
    struct RouteResponse {
        classify: ClassifyResult,
        route: RoutePlan,
    }

    let response = RouteResponse {
        classify: classify_result,
        route: route_plan,
    };

    write_stdout_json(&serde_json::to_value(&response)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classify_request_parses() {
        let json = serde_json::json!({"query": "what did I do last week"});
        let req: ClassifyRequest = serde_json::from_value(json).unwrap();
        assert_eq!(req.query, "what did I do last week");
    }

    #[test]
    fn parser_rejects_malformed_json_without_panicking() {
        assert!(parse_json_input("{not json").is_err());
    }

    #[test]
    fn parser_accepts_empty_input_as_null() {
        assert_eq!(parse_json_input("  \n").unwrap(), serde_json::Value::Null);
    }

    #[test]
    fn route_request_parses_with_defaults() {
        let json = serde_json::json!({"query": "find @alice from last week"});
        let req: RouteRequest = serde_json::from_value(json).unwrap();
        assert_eq!(req.query, "find @alice from last week");
        assert_eq!(req.namespace, "general");
        assert_eq!(req.default_limit, 10);
        assert!(req.domain.is_none());
    }

    #[test]
    fn route_request_parses_with_scope() {
        let json = serde_json::json!({
            "query": "find ESP32 work",
            "namespace": "coding",
            "domain": "code",
            "workspace_id": "ws1",
            "repo_id": "embedded-rust",
            "default_limit": 5
        });
        let req: RouteRequest = serde_json::from_value(json).unwrap();
        assert_eq!(req.namespace, "coding");
        assert_eq!(req.domain.as_deref(), Some("code"));
        assert_eq!(req.workspace_id.as_deref(), Some("ws1"));
        assert_eq!(req.repo_id.as_deref(), Some("embedded-rust"));
        assert_eq!(req.default_limit, 5);
    }

    #[test]
    fn classify_temporal_query() {
        let result = classify("what did I do with ESP32 last week");
        // "last week" is a temporal marker → TemporalLookup
        assert_eq!(result.mode.kind(), "temporal");
    }

    #[test]
    fn classify_entity_query() {
        let result = classify("what does @reembed_all do");
        assert_eq!(result.mode.kind(), "entity");
    }

    #[test]
    fn classify_mixed_query() {
        let result = classify("what did @alice do last week");
        assert_eq!(result.mode.kind(), "mixed");
    }

    #[test]
    fn classify_semantic_query() {
        let result = classify("how does the search pipeline work");
        assert_eq!(result.mode.kind(), "semantic");
    }

    #[test]
    fn route_plan_for_temporal() {
        let scope = Scope::new("test");
        let classify_result = classify("what changed last week");
        let plan = plan("what changed last week", &classify_result.mode, &scope, 10);
        assert_eq!(plan.legs.len(), 1);
        assert_eq!(plan.legs[0].strategy.kind(), "temporal");
    }

    #[test]
    fn route_plan_for_mixed() {
        let scope = Scope::new("test");
        let classify_result = classify("what did @alice do last week");
        assert_eq!(classify_result.mode.kind(), "mixed");
        let plan = plan(
            "what did @alice do last week",
            &classify_result.mode,
            &scope,
            10,
        );
        // Mixed with entity + temporal → 2 legs
        assert_eq!(plan.legs.len(), 2);
    }
}
