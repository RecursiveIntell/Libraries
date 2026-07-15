use std::collections::HashMap;

use serde_json::Value;

use crate::evidence::{digest, verify};
use crate::protocol::{tool_error, tool_success, RpcError, RpcRequest, RpcResponse};
use crate::run_manager::RunManager;
use crate::spec::{ensure_size, parse_and_validate, GraphSpec, MAX_GRAPHS, MAX_INPUT_BYTES};
use crate::templates;

#[derive(Clone)]
struct RegisteredGraph {
    spec: GraphSpec,
    normalized: Value,
    version: String,
    warnings: Vec<String>,
}

pub struct Server {
    base_url: String,
    default_model: String,
    graphs: HashMap<String, RegisteredGraph>,
    runs: RunManager,
}

impl Server {
    pub fn new(base_url: String, default_model: String) -> Self {
        Self {
            base_url,
            default_model,
            graphs: HashMap::new(),
            runs: RunManager::default(),
        }
    }

    pub fn handle_request(&mut self, req: &RpcRequest) -> RpcResponse {
        match req.method.as_str() {
            "initialize" => RpcResponse {
                jsonrpc: "2.0".into(),
                id: req.id.clone(),
                result: Some(
                    serde_json::json!({"protocolVersion":"2024-11-05","capabilities":{"tools":{}},"serverInfo":{"name":"agent-graph-mcp","version":env!("CARGO_PKG_VERSION")}}),
                ),
                error: None,
            },
            "tools/list" => RpcResponse {
                jsonrpc: "2.0".into(),
                id: req.id.clone(),
                result: Some(tool_list()),
                error: None,
            },
            "tools/call" => self.handle_call(&req.id, &req.params),
            _ => RpcResponse {
                jsonrpc: "2.0".into(),
                id: req.id.clone(),
                result: None,
                error: Some(RpcError {
                    code: -32601,
                    message: format!("method not found: {}", req.method),
                }),
            },
        }
    }

    fn handle_call(&mut self, id: &Value, params: &Value) -> RpcResponse {
        let name = params.get("name").and_then(Value::as_str).unwrap_or("");
        let args = params
            .get("arguments")
            .cloned()
            .unwrap_or_else(|| serde_json::json!({}));
        let result = match name {
            "graph_create" => self.graph_create(&args),
            "graph_execute" => self.graph_execute(&args),
            "graph_status" => self.graph_status(&args),
            _ => Err(("UNKNOWN_TOOL", format!("unknown tool: {name}"))),
        };
        match result {
            Ok(v) => tool_success(id, v),
            Err((c, m)) => tool_error(id, c, m),
        }
    }

    fn graph_create(&mut self, args: &Value) -> Result<Value, (&'static str, String)> {
        let action = args
            .get("action")
            .and_then(Value::as_str)
            .unwrap_or("create");
        if action == "delete" {
            let id = args
                .get("graph_id")
                .and_then(Value::as_str)
                .ok_or(("INVALID_REQUEST", "missing graph_id".into()))?;
            let Some(existing) = self.graphs.get(id) else {
                return Err(("GRAPH_NOT_FOUND", format!("graph '{id}' not found")));
            };
            if let Some(expected) = args.get("if_version").and_then(Value::as_str) {
                if expected != existing.version {
                    return Err(("GRAPH_VERSION_MISMATCH", "if_version does not match".into()));
                }
            }
            self.graphs.remove(id);
            return Ok(
                serde_json::json!({"graph_id":id,"status":"deleted","storage_class":"volatile"}),
            );
        }
        if action != "create" && action != "validate" {
            return Err((
                "UNSUPPORTED",
                format!("unsupported graph_create action '{action}'"),
            ));
        }
        let raw = if let Some(template) = args.get("template") {
            let id = template
                .get("id")
                .and_then(Value::as_str)
                .ok_or(("INVALID_REQUEST", "template.id required".into()))?;
            let name = template
                .get("name")
                .and_then(Value::as_str)
                .or_else(|| args.get("graph_id").and_then(Value::as_str))
                .unwrap_or(id);
            templates::instantiate(id, name).map_err(|e| ("UNSUPPORTED", e))?
        } else {
            args.get("spec")
                .cloned()
                .ok_or(("INVALID_REQUEST", "missing spec".into()))?
        };
        let original_version = raw
            .get("spec_version")
            .and_then(Value::as_str)
            .unwrap_or("1")
            .to_owned();
        let warnings_preview = serde_json::from_value::<GraphSpec>(raw.clone())
            .ok()
            .map(|s| s.warnings())
            .unwrap_or_default();
        let spec = parse_and_validate(&raw).map_err(|e| ("INVALID_SPEC", e))?;
        let normalized = serde_json::to_value(&spec).map_err(|e| ("INTERNAL", e.to_string()))?;
        let version = digest(&normalized);
        let warnings = if original_version == "1" {
            warnings_preview
        } else {
            spec.warnings()
        };
        if action == "validate" {
            return Ok(
                serde_json::json!({"graph_id":spec.name,"graph_version":version,"digest":version,"normalized_spec_version":"2","warnings":warnings,"storage_class":"volatile","status":"valid"}),
            );
        }
        if !self.graphs.contains_key(&spec.name) && self.graphs.len() >= MAX_GRAPHS {
            return Err((
                "LIMIT_EXCEEDED",
                format!("graph limit ({MAX_GRAPHS}) reached"),
            ));
        }
        let id = spec.name.clone();
        self.graphs.insert(
            id.clone(),
            RegisteredGraph {
                spec,
                normalized,
                version: version.clone(),
                warnings: warnings.clone(),
            },
        );
        Ok(
            serde_json::json!({"graph_id":id,"graph_version":version,"digest":version,"normalized_spec_version":"2","warnings":warnings,"storage_class":"volatile","status":"created"}),
        )
    }

    fn graph_execute(&mut self, args: &Value) -> Result<Value, (&'static str, String)> {
        let action = args
            .get("action")
            .and_then(Value::as_str)
            .unwrap_or("start");
        if action == "verify_replay" {
            let artifact = if let Some(bundle) = args.get("bundle") {
                bundle.clone()
            } else {
                let id = args
                    .get("run_id")
                    .and_then(Value::as_str)
                    .ok_or(("INVALID_REQUEST", "run_id or bundle required".into()))?;
                self.runs
                    .get(id)
                    .ok_or(("RUN_NOT_FOUND", "run not found".into()))?
                    .bundle
            };
            return Ok(verify(&artifact));
        }
        if action == "cancel" {
            let id = args
                .get("run_id")
                .and_then(Value::as_str)
                .ok_or(("INVALID_REQUEST", "missing run_id".into()))?;
            return self.runs.cancel(id).map_err(|e| ("RUN_NOT_FOUND", e));
        }
        if ["resume", "interrupt"].contains(&action) {
            return Err(("UNSUPPORTED","resume/interrupt require correlation-bound durable continuation and are not implemented".into()));
        }
        if action != "start" {
            return Err((
                "UNSUPPORTED",
                format!("unsupported graph_execute action '{action}'"),
            ));
        }
        let id = args
            .get("graph_id")
            .and_then(Value::as_str)
            .ok_or(("INVALID_REQUEST", "missing graph_id".into()))?;
        let graph = self
            .graphs
            .get(id)
            .cloned()
            .ok_or(("GRAPH_NOT_FOUND", format!("graph '{id}' not found")))?;
        if let Some(pin) = args.get("graph_version").and_then(Value::as_str) {
            if pin != graph.version {
                return Err((
                    "GRAPH_VERSION_MISMATCH",
                    "requested graph version is not current".into(),
                ));
            }
        }
        let input = args.get("input").cloned().unwrap_or(Value::Null);
        ensure_size(&input, MAX_INPUT_BYTES, "execution input")
            .map_err(|e| ("LIMIT_EXCEEDED", e))?;
        let run_id = self
            .runs
            .allocate(id, &graph.version, input)
            .map_err(|error| ("RUN_CAPACITY", error))?;
        if let Err(error) = self.runs.admit_async(&run_id) {
            self.runs.remove(&run_id);
            return Err(("RUN_CAPACITY", error));
        }
        let accepted = args.get("wait").and_then(Value::as_str) == Some("accepted");
        if accepted {
            self.runs.start(
                run_id.clone(),
                graph.spec,
                self.base_url.clone(),
                self.default_model.clone(),
            );
            Ok(
                serde_json::json!({"run_id":run_id,"status":"accepted","storage_class":"volatile","cancellation":"between_node_boundaries"}),
            )
        } else {
            self.runs
                .execute(
                    &run_id,
                    graph.spec,
                    self.base_url.clone(),
                    self.default_model.clone(),
                )
                .map_err(|e| ("EXECUTION_FAILED", e))
        }
    }

    fn graph_status(&self, args: &Value) -> Result<Value, (&'static str, String)> {
        let Some(resource) = args.get("resource").and_then(Value::as_str) else {
            return Ok(
                serde_json::json!({"graphs":self.graphs.keys().collect::<Vec<_>>(),"graph_count":self.graphs.len(),"execution_count":self.runs.list().len(),"retained_execution_count":self.runs.list().len(),"total_execution_count":self.runs.list().len(),"base_url":safe_provider_label(&self.base_url),"default_model":self.default_model}),
            );
        };
        match resource {
            "server" => Ok(
                serde_json::json!({"storage_class":"volatile","capabilities":{"runtime":"agent_graph","async_start":true,"cancellation":"between_node_boundaries","durable_resume":false,"hitl":false,"replay":"integrity_verified"},"limits":{"graphs":MAX_GRAPHS}}),
            ),
            "templates" => Ok(templates::list()),
            "graph" => {
                let id = args
                    .get("graph_id")
                    .and_then(Value::as_str)
                    .ok_or(("INVALID_REQUEST", "missing graph_id".into()))?;
                let g = self
                    .graphs
                    .get(id)
                    .ok_or(("GRAPH_NOT_FOUND", "graph not found".into()))?;
                Ok(
                    serde_json::json!({"graph_id":id,"graph_version":g.version,"normalized_spec":g.normalized,"mermaid":mermaid(&g.spec),"warnings":g.warnings,"storage_class":"volatile"}),
                )
            }
            "run" => {
                if args.get("action").and_then(Value::as_str) == Some("list") {
                    Ok(serde_json::json!({"runs":self.runs.list()}))
                } else {
                    let id = args
                        .get("run_id")
                        .and_then(Value::as_str)
                        .ok_or(("INVALID_REQUEST", "missing run_id".into()))?;
                    self.runs
                        .get(id)
                        .map(|r| r.public())
                        .ok_or(("RUN_NOT_FOUND", "run not found".into()))
                }
            }
            "events" => {
                let id = args
                    .get("run_id")
                    .and_then(Value::as_str)
                    .ok_or(("INVALID_REQUEST", "missing run_id".into()))?;
                self.runs
                    .events(
                        id,
                        args.get("cursor").and_then(Value::as_u64).unwrap_or(0),
                        args.get("limit").and_then(Value::as_u64).unwrap_or(100) as usize,
                    )
                    .map_err(|e| ("RUN_NOT_FOUND", e))
            }
            "receipt" => {
                let id = args
                    .get("run_id")
                    .and_then(Value::as_str)
                    .ok_or(("INVALID_REQUEST", "missing run_id".into()))?;
                Ok(self
                    .runs
                    .get(id)
                    .ok_or(("RUN_NOT_FOUND", "run not found".into()))?
                    .receipt)
            }
            "bundle" => {
                let id = args
                    .get("run_id")
                    .and_then(Value::as_str)
                    .ok_or(("INVALID_REQUEST", "missing run_id".into()))?;
                Ok(self
                    .runs
                    .get(id)
                    .ok_or(("RUN_NOT_FOUND", "run not found".into()))?
                    .bundle)
            }
            _ => Err((
                "INVALID_REQUEST",
                format!("unknown status resource '{resource}'"),
            )),
        }
    }
}

fn mermaid(spec: &GraphSpec) -> String {
    let mut s = String::from("graph TD\n");
    for edge in &spec.edges {
        s.push_str(&format!("  {} --> {}\n", edge.from, edge.to));
    }
    s
}

fn safe_provider_label(url: &str) -> String {
    let without_fragment = url.split(['?', '#']).next().unwrap_or(url);
    if let Some((scheme, rest)) = without_fragment.split_once("://") {
        let authority_and_path = rest.rsplit_once('@').map(|(_, safe)| safe).unwrap_or(rest);
        format!("{scheme}://{authority_and_path}")
    } else {
        "server-configured".into()
    }
}

fn tool_list() -> Value {
    serde_json::json!({"tools":[
     {"name":"graph_create","description":"Create, validate, delete, or instantiate a safe versioned declarative graph.","inputSchema":{"type":"object","properties":{"action":{"type":"string"},"spec":{"type":"object"},"template":{"type":"object"}}}},
     {"name":"graph_execute","description":"Execute/start/cancel a graph or verify an integrity bundle offline.","inputSchema":{"type":"object","properties":{"action":{"type":"string"},"graph_id":{"type":"string"},"input":{}}}},
     {"name":"graph_status","description":"Inspect server, graph, run, events, receipt, bundle, or templates.","inputSchema":{"type":"object","properties":{"resource":{"type":"string"},"run_id":{"type":"string"}}}}
    ]})
}
