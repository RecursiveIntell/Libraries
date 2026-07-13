use std::collections::{HashMap, HashSet, VecDeque};
use std::io::{self, BufRead, Write};

use agent_graph::state::AgentState;
use llm_pipeline::payload::Payload;
use llm_pipeline::{ExecCtx, LlmCall, LlmConfig};
use serde::{Deserialize, Serialize};
use serde_json::Value;

// Conservative, process-local bounds for untrusted MCP inputs and retained state.
const MAX_GRAPHS: usize = 64;
const MAX_GRAPH_SERIALIZED_BYTES: usize = 64 * 1024;
const MAX_GRAPH_NODES: usize = 128;
const MAX_GRAPH_EDGES: usize = 512;
const MAX_RECURSION_LIMIT: usize = 64;
const MAX_EXECUTION_INPUT_BYTES: usize = 64 * 1024;
const MAX_EXECUTION_OUTPUT_BYTES: usize = 128 * 1024;
const MAX_RETAINED_EXECUTIONS: usize = 100;

// ── Graph spec types ─────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
struct GraphSpec {
    name: String,
    entry: String,
    nodes: Vec<NodeSpec>,
    edges: Vec<EdgeSpec>,
    #[serde(default)]
    recursion_limit: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct NodeSpec {
    id: String,
    #[serde(rename = "type")]
    node_type: NodeType,
    #[serde(default)]
    prompt: Option<String>,
    #[serde(default)]
    model: Option<String>,
    #[serde(default)]
    json_mode: bool,
    #[serde(default)]
    max_tokens: Option<usize>,
    #[serde(default)]
    routes: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum NodeType {
    #[serde(rename = "llm")]
    Llm,
    #[serde(rename = "router")]
    Router,
    #[serde(rename = "passthrough")]
    Passthrough,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct EdgeSpec {
    from: String,
    to: String,
}

// ── Execution result ─────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
struct GraphExecutionResult {
    success: bool,
    final_state: Value,
    steps: Vec<StepResult>,
    error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct StepResult {
    node_id: String,
    status: String,
    output: Option<Value>,
}

#[derive(Debug, Clone, Serialize)]
struct GraphExecutionSummary {
    success: bool,
    step_count: usize,
    error: Option<String>,
}

// ── MCP protocol types ───────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct RpcRequest {
    jsonrpc: String,
    id: Value,
    method: String,
    #[serde(default)]
    params: Value,
}

#[derive(Debug, Serialize)]
struct RpcResponse {
    jsonrpc: String,
    id: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<RpcError>,
}

#[derive(Debug, Serialize)]
struct RpcError {
    code: i32,
    message: String,
}

// ── Server ───────────────────────────────────────────────────────────

struct Server {
    base_url: String,
    default_model: String,
    graphs: HashMap<String, GraphSpec>,
    executions: VecDeque<GraphExecutionSummary>,
    total_execution_count: usize,
}

impl Server {
    fn new(base_url: String, default_model: String) -> Self {
        Self {
            base_url,
            default_model,
            graphs: HashMap::new(),
            executions: VecDeque::new(),
            total_execution_count: 0,
        }
    }

    fn handle_request(&mut self, req: &RpcRequest) -> RpcResponse {
        match req.method.as_str() {
            "initialize" => RpcResponse {
                jsonrpc: "2.0".into(),
                id: req.id.clone(),
                result: Some(serde_json::json!({
                    "protocolVersion": "2024-11-05",
                    "capabilities": { "tools": {} },
                    "serverInfo": { "name": "agent-graph-mcp", "version": "0.1.0" }
                })),
                error: None,
            },
            "tools/list" => RpcResponse {
                jsonrpc: "2.0".into(),
                id: req.id.clone(),
                result: Some(serde_json::json!({
                    "tools": [
                        {
                            "name": "graph_create",
                            "description": "Create a graph-orchestrated workflow from a JSON spec. Nodes: 'llm' (calls LLM), 'router' (conditional routing), 'passthrough'.",
                            "inputSchema": {
                                "type": "object",
                                "properties": {
                                    "spec": { "type": "object", "description": "Graph spec: {name, entry, nodes:[{id,type,prompt?,model?,json_mode?,routes?}], edges:[{from,to}]}" }
                                },
                                "required": ["spec"]
                            }
                        },
                        {
                            "name": "graph_execute",
                            "description": "Execute a graph with input. Returns final state and per-step results.",
                            "inputSchema": {
                                "type": "object",
                                "properties": {
                                    "graph_id": { "type": "string" },
                                    "input": { "description": "Input value (any JSON)" }
                                },
                                "required": ["graph_id", "input"]
                            }
                        },
                        {
                            "name": "graph_status",
                            "description": "Show server status: registered graphs and executions.",
                            "inputSchema": { "type": "object", "properties": {} }
                        }
                    ]
                })),
                error: None,
            },
            "tools/call" => self.handle_tool_call(&req.id, &req.params),
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

    fn handle_tool_call(&mut self, id: &Value, params: &Value) -> RpcResponse {
        let name = params.get("name").and_then(|v| v.as_str()).unwrap_or("");
        let args = params.get("arguments").cloned().unwrap_or(Value::Null);

        let result = match name {
            "graph_create" => self.tool_graph_create(&args),
            "graph_execute" => self.tool_graph_execute(&args),
            "graph_status" => self.tool_graph_status(),
            _ => Err(format!("unknown tool: {name}")),
        };

        match result {
            Ok(val) => RpcResponse {
                jsonrpc: "2.0".into(),
                id: id.clone(),
                result: Some(serde_json::json!({
                    "content": [{ "type": "text", "text": serde_json::to_string_pretty(&val).unwrap_or_default() }]
                })),
                error: None,
            },
            Err(msg) => RpcResponse {
                jsonrpc: "2.0".into(),
                id: id.clone(),
                result: Some(serde_json::json!({
                    "content": [{ "type": "text", "text": serde_json::to_string(&serde_json::json!({ "error": msg })).unwrap_or_default() }],
                    "isError": true,
                })),
                error: None,
            },
        }
    }

    fn tool_graph_create(&mut self, args: &Value) -> Result<Value, String> {
        let spec: GraphSpec =
            serde_json::from_value(args.get("spec").cloned().unwrap_or(Value::Null))
                .map_err(|e| format!("invalid graph spec: {e}"))?;

        validate_graph_spec(&spec)?;

        let graph_id = spec.name.clone();
        if !self.graphs.contains_key(&graph_id) && self.graphs.len() >= MAX_GRAPHS {
            return Err(format!("graph limit ({MAX_GRAPHS}) reached"));
        }
        self.graphs.insert(graph_id.clone(), spec);
        Ok(serde_json::json!({
            "graph_id": graph_id,
            "status": "created"
        }))
    }

    fn tool_graph_execute(&mut self, args: &Value) -> Result<Value, String> {
        let graph_id = args
            .get("graph_id")
            .and_then(|v| v.as_str())
            .ok_or("missing graph_id")?;
        let input = args.get("input").cloned().unwrap_or(Value::Null);
        ensure_value_within_limit(&input, MAX_EXECUTION_INPUT_BYTES, "execution input")?;
        let spec = self
            .graphs
            .get(graph_id)
            .ok_or(format!("graph '{graph_id}' not found"))?
            .clone();

        let rt = tokio::runtime::Runtime::new().map_err(|e| format!("runtime error: {e}"))?;
        let result = rt.block_on(async { self.execute_graph(&spec, &input).await })?;
        ensure_execution_output_within_limit(&result)?;

        self.record_execution(result.clone());
        Ok(serde_json::to_value(&result).map_err(|e| format!("serialize error: {e}"))?)
    }

    fn record_execution(&mut self, result: GraphExecutionResult) {
        self.total_execution_count += 1;
        if self.executions.len() == MAX_RETAINED_EXECUTIONS {
            self.executions.pop_front();
        }
        self.executions.push_back(GraphExecutionSummary {
            success: result.success,
            step_count: result.steps.len(),
            error: result.error,
        });
    }

    fn tool_graph_status(&self) -> Result<Value, String> {
        Ok(serde_json::json!({
            "graphs": self.graphs.keys().collect::<Vec<_>>(),
            "graph_count": self.graphs.len(),
            "execution_count": self.executions.len(),
            "retained_execution_count": self.executions.len(),
            "total_execution_count": self.total_execution_count,
            "base_url": self.base_url,
            "default_model": self.default_model,
        }))
    }

    async fn execute_graph(
        &self,
        spec: &GraphSpec,
        input: &Value,
    ) -> Result<GraphExecutionResult, String> {
        validate_graph_spec(spec)?;
        ensure_value_within_limit(input, MAX_EXECUTION_INPUT_BYTES, "execution input")?;
        let state = AgentState::new();
        state.set("__input__", input.clone()).await.ok();

        let mut steps: Vec<StepResult> = Vec::new();
        let mut current_node = spec.entry.clone();
        let mut visited = HashSet::new();
        let recursion_limit = spec.recursion_limit.unwrap_or(MAX_RECURSION_LIMIT);

        // Build node lookup
        let node_map: HashMap<&str, &NodeSpec> =
            spec.nodes.iter().map(|n| (n.id.as_str(), n)).collect();
        // Build edge lookup
        let edge_map: HashMap<&str, Vec<&str>> = {
            let mut m: HashMap<&str, Vec<&str>> = HashMap::new();
            for e in &spec.edges {
                m.entry(&e.from).or_default().push(&e.to);
            }
            m
        };

        loop {
            if visited.len() >= recursion_limit {
                return Ok(GraphExecutionResult {
                    success: false,
                    final_state: state
                        .get::<Value>("__input__")
                        .await
                        .ok()
                        .unwrap_or(Value::Null),
                    steps,
                    error: Some(format!("recursion limit ({recursion_limit}) exceeded")),
                });
            }
            if visited.contains(&current_node) {
                return Ok(GraphExecutionResult {
                    success: false,
                    final_state: state
                        .get::<Value>("__input__")
                        .await
                        .ok()
                        .unwrap_or(Value::Null),
                    steps,
                    error: Some(format!("cycle detected at '{current_node}'")),
                });
            }
            visited.insert(current_node.clone());

            let node = node_map
                .get(current_node.as_str())
                .ok_or_else(|| format!("node '{current_node}' not found"))?;

            match node.node_type {
                NodeType::Llm => {
                    let prompt = node.prompt.as_deref().unwrap_or("{input}");
                    let model = node.model.as_deref().unwrap_or(&self.default_model);
                    let input_val: Value = state.get("__input__").await.ok().unwrap_or(Value::Null);
                    let rendered = render_prompt(prompt, &input_val);

                    let mut llm_call = LlmCall::new(&current_node, &rendered).with_model(model);
                    let mut config = LlmConfig::default();
                    if node.json_mode {
                        config = config.with_json_mode(true);
                    }
                    if let Some(mt) = node.max_tokens {
                        config = config.with_max_tokens(mt as u32);
                    }
                    llm_call = llm_call.with_config(config);

                    let ctx = ExecCtx::builder(&self.base_url).build();
                    let output = llm_call
                        .invoke(&ctx, input_val)
                        .await
                        .map_err(|e| format!("LLM call failed at '{current_node}': {e}"))?;

                    let output_val = output.value;
                    ensure_value_within_limit(
                        &output_val,
                        MAX_EXECUTION_OUTPUT_BYTES,
                        "node output",
                    )?;
                    state.set(&current_node, output_val.clone()).await.ok();
                    state.set("__input__", output_val.clone()).await.ok();

                    steps.push(StepResult {
                        node_id: current_node.clone(),
                        status: "completed".into(),
                        output: Some(output_val),
                    });
                }
                NodeType::Router => {
                    let input_val: Value = state.get("__input__").await.ok().unwrap_or(Value::Null);
                    let input_str = serde_json::to_string(&input_val).unwrap_or_default();
                    let routes = node
                        .routes
                        .as_ref()
                        .filter(|routes| !routes.is_empty())
                        .ok_or_else(|| {
                            format!(
                                "router node '{}' must define non-empty routes",
                                current_node
                            )
                        })?;

                    let next = routes.iter().find_map(|(pattern, target)| {
                        if input_str.contains(pattern) {
                            Some(target.clone())
                        } else {
                            None
                        }
                    });

                    steps.push(StepResult {
                        node_id: current_node.clone(),
                        status: "router".into(),
                        output: Some(Value::String(next.clone().unwrap_or_else(|| "END".into()))),
                    });

                    match next {
                        Some(target) if target == "END" => break,
                        Some(target) => {
                            current_node = target;
                            continue;
                        }
                        None => break,
                    }
                }
                NodeType::Passthrough => {
                    steps.push(StepResult {
                        node_id: current_node.clone(),
                        status: "passthrough".into(),
                        output: None,
                    });
                }
            }

            // Find next node via edges
            let next_nodes = edge_map.get(current_node.as_str());
            match next_nodes {
                Some(targets) if !targets.is_empty() => {
                    let next = targets[0];
                    if next == "END" {
                        break;
                    }
                    current_node = next.to_string();
                }
                _ => break,
            }
        }

        let final_state: Value = state.get("__input__").await.ok().unwrap_or(Value::Null);
        let result = GraphExecutionResult {
            success: true,
            final_state,
            steps,
            error: None,
        };
        ensure_execution_output_within_limit(&result)?;
        Ok(result)
    }
}

fn validate_graph_spec(spec: &GraphSpec) -> Result<(), String> {
    let serialized = serde_json::to_vec(spec).map_err(|e| format!("serialize graph spec: {e}"))?;
    if serialized.len() > MAX_GRAPH_SERIALIZED_BYTES {
        return Err(format!(
            "serialized graph spec exceeds {MAX_GRAPH_SERIALIZED_BYTES} bytes"
        ));
    }
    if spec.nodes.is_empty() {
        return Err("graph must have at least one node".into());
    }
    if spec.nodes.len() > MAX_GRAPH_NODES {
        return Err(format!("graph node limit ({MAX_GRAPH_NODES}) exceeded"));
    }
    if spec.edges.len() > MAX_GRAPH_EDGES {
        return Err(format!("graph edge limit ({MAX_GRAPH_EDGES}) exceeded"));
    }
    if spec
        .recursion_limit
        .is_some_and(|limit| limit > MAX_RECURSION_LIMIT)
    {
        return Err(format!("recursion limit exceeds {MAX_RECURSION_LIMIT}"));
    }

    let node_ids: HashSet<&str> = spec.nodes.iter().map(|node| node.id.as_str()).collect();
    if node_ids.len() != spec.nodes.len() {
        return Err("duplicate node ID".into());
    }
    if !node_ids.contains(spec.entry.as_str()) {
        return Err(format!("entry node '{}' not found", spec.entry));
    }

    for edge in &spec.edges {
        if !node_ids.contains(edge.from.as_str()) {
            return Err(format!("edge source '{}' not found", edge.from));
        }
        if edge.to != "END" && !node_ids.contains(edge.to.as_str()) {
            return Err(format!("edge target '{}' not found", edge.to));
        }
    }
    for node in &spec.nodes {
        if matches!(node.node_type, NodeType::Router) {
            let routes = node
                .routes
                .as_ref()
                .filter(|routes| !routes.is_empty())
                .ok_or_else(|| format!("router node '{}' must define non-empty routes", node.id))?;
            for target in routes.values() {
                if target != "END" && !node_ids.contains(target.as_str()) {
                    return Err(format!(
                        "router node '{}' target '{}' not found",
                        node.id, target
                    ));
                }
            }
        }
    }
    Ok(())
}

fn ensure_value_within_limit(value: &Value, limit: usize, label: &str) -> Result<(), String> {
    let bytes = serde_json::to_vec(value).map_err(|e| format!("serialize {label}: {e}"))?;
    if bytes.len() > limit {
        return Err(format!("{label} exceeds {limit} bytes"));
    }
    Ok(())
}

fn ensure_execution_output_within_limit(result: &GraphExecutionResult) -> Result<(), String> {
    let bytes =
        serde_json::to_vec(result).map_err(|e| format!("serialize execution output: {e}"))?;
    if bytes.len() > MAX_EXECUTION_OUTPUT_BYTES {
        return Err(format!(
            "execution output exceeds {MAX_EXECUTION_OUTPUT_BYTES} bytes"
        ));
    }
    Ok(())
}

fn render_prompt(template: &str, input: &Value) -> String {
    template.replace("{input}", &serde_json::to_string(input).unwrap_or_default())
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut base_url = "http://127.0.0.1:11434".to_string();
    let mut default_model = "glm-5.2:cloud".to_string();

    let mut iter = args.iter().skip(1);
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--base-url" => {
                if let Some(v) = iter.next() {
                    base_url = v.clone();
                }
            }
            "--model" => {
                if let Some(v) = iter.next() {
                    default_model = v.clone();
                }
            }
            "--help" => {
                eprintln!("agent-graph-mcp — MCP server for graph-orchestrated LLM workflows");
                eprintln!("Usage: agent-graph-mcp [--base-url URL] [--model MODEL]");
                std::process::exit(0);
            }
            _ => {}
        }
    }

    let mut server = Server::new(base_url, default_model);
    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut stdout = stdout.lock();

    for line in stdin.lock().lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => break,
        };
        if line.trim().is_empty() {
            continue;
        }

        let req: RpcRequest = match serde_json::from_str(&line) {
            Ok(r) => r,
            Err(e) => {
                eprintln!("JSON parse error: {e}");
                continue;
            }
        };

        let resp = server.handle_request(&req);
        let json = serde_json::to_string(&resp).unwrap_or_else(|_| {
            r#"{"jsonrpc":"2.0","id":null,"error":{"code":-32603,"message":"serialize error"}}"#
                .into()
        });

        if writeln!(stdout, "{json}").is_err() {
            break;
        }
        stdout.flush().ok();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_graph_spec() {
        let spec_json = r#"{"name":"test","entry":"start","nodes":[{"id":"start","type":"passthrough"},{"id":"end","type":"passthrough"}],"edges":[{"from":"start","to":"end"},{"from":"end","to":"END"}]}"#;
        let spec: GraphSpec = serde_json::from_str(spec_json).unwrap();
        assert_eq!(spec.name, "test");
        assert_eq!(spec.nodes.len(), 2);
    }

    #[test]
    fn parse_llm_node() {
        let node_json = r#"{"id":"classify","type":"llm","prompt":"Classify: {input}","model":"test","json_mode":true}"#;
        let node: NodeSpec = serde_json::from_str(node_json).unwrap();
        assert_eq!(node.id, "classify");
        assert!(node.json_mode);
    }

    #[test]
    fn parse_router_node() {
        let node_json =
            r#"{"id":"decide","type":"router","routes":{"deep":"research","shallow":"summarize"}}"#;
        let node: NodeSpec = serde_json::from_str(node_json).unwrap();
        assert!(node.routes.is_some());
    }

    #[test]
    fn server_status_empty() {
        let server = Server::new("http://localhost:11434".into(), "test".into());
        let status = server.tool_graph_status().unwrap();
        assert_eq!(status["graph_count"], 0);
    }

    #[test]
    fn create_graph_validates_entry() {
        let mut server = Server::new("http://localhost:11434".into(), "test".into());
        let args = serde_json::json!({"spec":{"name":"bad","entry":"missing","nodes":[{"id":"other","type":"passthrough"}],"edges":[]}});
        assert!(server.tool_graph_create(&args).is_err());
    }

    #[test]
    fn create_graph_rejects_duplicates() {
        let mut server = Server::new("http://localhost:11434".into(), "test".into());
        let args = serde_json::json!({"spec":{"name":"dup","entry":"a","nodes":[{"id":"a","type":"passthrough"},{"id":"a","type":"passthrough"}],"edges":[]}});
        assert!(server.tool_graph_create(&args).is_err());
    }

    #[test]
    fn create_graph_succeeds() {
        let mut server = Server::new("http://localhost:11434".into(), "test".into());
        let args = serde_json::json!({"spec":{"name":"good","entry":"start","nodes":[{"id":"start","type":"passthrough"},{"id":"end","type":"passthrough"}],"edges":[{"from":"start","to":"end"},{"from":"end","to":"END"}]}});
        let result = server.tool_graph_create(&args).unwrap();
        assert_eq!(result["graph_id"], "good");
    }

    #[test]
    fn execute_passthrough_graph() {
        let mut server = Server::new("http://localhost:11434".into(), "test".into());
        server.tool_graph_create(&serde_json::json!({"spec":{"name":"pass","entry":"a","nodes":[{"id":"a","type":"passthrough"},{"id":"b","type":"passthrough"}],"edges":[{"from":"a","to":"b"},{"from":"b","to":"END"}]}})).unwrap();
        let result = server
            .tool_graph_execute(&serde_json::json!({"graph_id":"pass","input":"hello"}))
            .unwrap();
        assert_eq!(result["success"], true);
        assert!(result["steps"].as_array().unwrap().len() >= 2);
    }

    #[test]
    fn execute_router_deep_path() {
        let mut server = Server::new("http://localhost:11434".into(), "test".into());
        server.tool_graph_create(&serde_json::json!({"spec":{"name":"routed","entry":"start","nodes":[{"id":"start","type":"passthrough"},{"id":"decide","type":"router","routes":{"deep":"deep_node","shallow":"shallow_node"}},{"id":"deep_node","type":"passthrough"},{"id":"shallow_node","type":"passthrough"}],"edges":[{"from":"start","to":"decide"},{"from":"deep_node","to":"END"},{"from":"shallow_node","to":"END"}]}})).unwrap();
        let result = server.tool_graph_execute(&serde_json::json!({"graph_id":"routed","input":"this is a deep research question"})).unwrap();
        assert_eq!(result["success"], true);
        let router_step = result["steps"]
            .as_array()
            .unwrap()
            .iter()
            .find(|s| s["node_id"] == "decide")
            .unwrap();
        assert_eq!(router_step["output"], "deep_node");
    }

    #[test]
    fn execute_router_shallow_path() {
        let mut server = Server::new("http://localhost:11434".into(), "test".into());
        server.tool_graph_create(&serde_json::json!({"spec":{"name":"routed","entry":"start","nodes":[{"id":"start","type":"passthrough"},{"id":"decide","type":"router","routes":{"deep":"deep_node","shallow":"shallow_node"}},{"id":"deep_node","type":"passthrough"},{"id":"shallow_node","type":"passthrough"}],"edges":[{"from":"start","to":"decide"},{"from":"deep_node","to":"END"},{"from":"shallow_node","to":"END"}]}})).unwrap();
        let result = server
            .tool_graph_execute(&serde_json::json!({"graph_id":"routed","input":"this is shallow"}))
            .unwrap();
        assert_eq!(result["success"], true);
        let router_step = result["steps"]
            .as_array()
            .unwrap()
            .iter()
            .find(|s| s["node_id"] == "decide")
            .unwrap();
        assert_eq!(router_step["output"], "shallow_node");
    }

    #[test]
    fn execute_router_route_to_end() {
        let mut server = Server::new("http://localhost:11434".into(), "test".into());
        server
            .tool_graph_create(&serde_json::json!({"spec":{"name":"router-end","entry":"router","nodes":[{"id":"router","type":"router","routes":{"stop":"END"}}],"edges":[]}}))
            .unwrap();
        let result = server
            .tool_graph_execute(&serde_json::json!({"graph_id":"router-end","input":"stop now"}))
            .unwrap();
        assert_eq!(result["success"], true);
        assert_eq!(result["steps"][0]["output"], "END");
    }

    #[test]
    fn mcp_initialize() {
        let mut server = Server::new("http://localhost:11434".into(), "test".into());
        let req = RpcRequest {
            jsonrpc: "2.0".into(),
            id: serde_json::json!(1),
            method: "initialize".into(),
            params: Value::Null,
        };
        let resp = server.handle_request(&req);
        assert_eq!(
            resp.result.unwrap()["serverInfo"]["name"],
            "agent-graph-mcp"
        );
    }

    #[test]
    fn mcp_tools_list() {
        let mut server = Server::new("http://localhost:11434".into(), "test".into());
        let req = RpcRequest {
            jsonrpc: "2.0".into(),
            id: serde_json::json!(2),
            method: "tools/list".into(),
            params: Value::Null,
        };
        let resp = server.handle_request(&req);
        let tools = resp.result.unwrap()["tools"].as_array().unwrap().clone();
        let names: Vec<_> = tools.iter().map(|t| t["name"].as_str().unwrap()).collect();
        assert!(names.contains(&"graph_create"));
        assert!(names.contains(&"graph_execute"));
        assert!(names.contains(&"graph_status"));
    }

    #[test]
    fn cycle_detection() {
        let mut server = Server::new("http://localhost:11434".into(), "test".into());
        server.tool_graph_create(&serde_json::json!({"spec":{"name":"cycle","entry":"a","nodes":[{"id":"a","type":"passthrough"},{"id":"b","type":"passthrough"}],"edges":[{"from":"a","to":"b"},{"from":"b","to":"a"}]}})).unwrap();
        let result = server
            .tool_graph_execute(&serde_json::json!({"graph_id":"cycle","input":"test"}))
            .unwrap();
        assert_eq!(result["success"], false);
        assert!(result["error"].as_str().unwrap().contains("cycle"));
    }

    #[test]
    fn create_graph_rejects_router_without_routes() {
        let mut server = Server::new("http://localhost:11434".into(), "test".into());
        let args = serde_json::json!({"spec":{"name":"bad-router","entry":"router","nodes":[{"id":"router","type":"router"}],"edges":[]}});
        assert!(server
            .tool_graph_create(&args)
            .unwrap_err()
            .contains("routes"));
    }

    #[test]
    fn create_graph_rejects_router_with_empty_routes() {
        let mut server = Server::new("http://localhost:11434".into(), "test".into());
        let args = serde_json::json!({"spec":{"name":"empty-router","entry":"router","nodes":[{"id":"router","type":"router","routes":{}}],"edges":[]}});
        assert!(server
            .tool_graph_create(&args)
            .unwrap_err()
            .contains("routes"));
    }

    #[test]
    fn create_graph_validates_edge_endpoints_and_allows_end() {
        let mut server = Server::new("http://localhost:11434".into(), "test".into());
        let missing_from = serde_json::json!({"spec":{"name":"missing-from","entry":"a","nodes":[{"id":"a","type":"passthrough"}],"edges":[{"from":"missing","to":"a"}]}});
        assert!(server.tool_graph_create(&missing_from).is_err());
        let missing_to = serde_json::json!({"spec":{"name":"missing-to","entry":"a","nodes":[{"id":"a","type":"passthrough"}],"edges":[{"from":"a","to":"missing"}]}});
        assert!(server.tool_graph_create(&missing_to).is_err());
        let end = serde_json::json!({"spec":{"name":"end-edge","entry":"a","nodes":[{"id":"a","type":"passthrough"}],"edges":[{"from":"a","to":"END"}]}});
        assert!(server.tool_graph_create(&end).is_ok());
    }

    #[test]
    fn create_graph_validates_router_targets_and_allows_end() {
        let mut server = Server::new("http://localhost:11434".into(), "test".into());
        let missing_target = serde_json::json!({"spec":{"name":"missing-target","entry":"router","nodes":[{"id":"router","type":"router","routes":{"go":"missing"}}],"edges":[]}});
        assert!(server.tool_graph_create(&missing_target).is_err());
        let end_target = serde_json::json!({"spec":{"name":"end-target","entry":"router","nodes":[{"id":"router","type":"router","routes":{"go":"END"}}],"edges":[]}});
        assert!(server.tool_graph_create(&end_target).is_ok());
    }

    #[test]
    fn execute_graph_rejects_in_memory_router_without_routes() {
        let server = Server::new("http://localhost:11434".into(), "test".into());
        let spec = GraphSpec {
            name: "invalid".into(),
            entry: "router".into(),
            nodes: vec![NodeSpec {
                id: "router".into(),
                node_type: NodeType::Router,
                prompt: None,
                model: None,
                json_mode: false,
                max_tokens: None,
                routes: None,
            }],
            edges: vec![],
            recursion_limit: None,
        };
        let rt = tokio::runtime::Runtime::new().unwrap();
        let error = rt
            .block_on(server.execute_graph(&spec, &Value::Null))
            .unwrap_err();
        assert!(error.contains("routes"));
    }

    #[test]
    fn mcp_tool_call_marks_invalid_router_as_error() {
        let mut server = Server::new("http://localhost:11434".into(), "test".into());
        let req = RpcRequest {
            jsonrpc: "2.0".into(),
            id: serde_json::json!(3),
            method: "tools/call".into(),
            params: serde_json::json!({"name":"graph_create","arguments":{"spec":{"name":"bad","entry":"router","nodes":[{"id":"router","type":"router"}],"edges":[]}}}),
        };
        let resp = server.handle_request(&req);
        assert_eq!(resp.result.unwrap()["isError"], true);
    }

    #[test]
    fn graph_create_enforces_all_spec_limits_at_boundaries() {
        let mut server = Server::new("http://localhost:11434".into(), "test".into());
        let node = serde_json::json!({"id":"a","type":"passthrough"});
        let mut at_node_limit = vec![node.clone(); MAX_GRAPH_NODES];
        for (index, node) in at_node_limit.iter_mut().enumerate() {
            node["id"] = Value::String(format!("n{index}"));
        }
        let valid_nodes = serde_json::json!({"spec":{"name":"node-limit","entry":"n0","nodes":at_node_limit,"edges":[]}});
        assert!(server.tool_graph_create(&valid_nodes).is_ok());
        let too_many_nodes = serde_json::json!({"spec":{"name":"too-many-nodes","entry":"n0","nodes":(0..=MAX_GRAPH_NODES).map(|i| serde_json::json!({"id":format!("n{i}"),"type":"passthrough"})).collect::<Vec<_>>(),"edges":[]}});
        assert!(server.tool_graph_create(&too_many_nodes).is_err());

        let edges = (0..MAX_GRAPH_EDGES)
            .map(|_| serde_json::json!({"from":"a","to":"END"}))
            .collect::<Vec<_>>();
        let valid_edges = serde_json::json!({"spec":{"name":"edge-limit","entry":"a","nodes":[node.clone()],"edges":edges}});
        assert!(server.tool_graph_create(&valid_edges).is_ok());
        let too_many_edges = serde_json::json!({"spec":{"name":"too-many-edges","entry":"a","nodes":[node.clone()],"edges":(0..=MAX_GRAPH_EDGES).map(|_| serde_json::json!({"from":"a","to":"END"})).collect::<Vec<_>>()}});
        assert!(server.tool_graph_create(&too_many_edges).is_err());

        let valid_recursion = serde_json::json!({"spec":{"name":"recursion-limit","entry":"a","nodes":[node.clone()],"edges":[],"recursion_limit":MAX_RECURSION_LIMIT}});
        assert!(server.tool_graph_create(&valid_recursion).is_ok());
        let too_much_recursion = serde_json::json!({"spec":{"name":"too-much-recursion","entry":"a","nodes":[node],"edges":[],"recursion_limit":MAX_RECURSION_LIMIT + 1}});
        assert!(server.tool_graph_create(&too_much_recursion).is_err());
    }

    #[test]
    fn graph_create_enforces_serialized_byte_limit() {
        let mut server = Server::new("http://localhost:11434".into(), "test".into());
        let base = GraphSpec {
            name: "x".into(),
            entry: "a".into(),
            nodes: vec![NodeSpec {
                id: "a".into(),
                node_type: NodeType::Passthrough,
                prompt: None,
                model: None,
                json_mode: false,
                max_tokens: None,
                routes: None,
            }],
            edges: vec![],
            recursion_limit: None,
        };
        let at_limit_name =
            "x".repeat(MAX_GRAPH_SERIALIZED_BYTES - serde_json::to_vec(&base).unwrap().len() + 1);
        let at_limit = serde_json::json!({"spec":{"name":at_limit_name,"entry":"a","nodes":[{"id":"a","type":"passthrough"}],"edges":[]}});
        assert!(server.tool_graph_create(&at_limit).is_ok());
        let too_large = serde_json::json!({"spec":{"name":"x".repeat(MAX_GRAPH_SERIALIZED_BYTES),"entry":"a","nodes":[{"id":"a","type":"passthrough"}],"edges":[]}});
        assert!(server
            .tool_graph_create(&too_large)
            .unwrap_err()
            .contains("serialized"));
    }

    #[test]
    fn graph_execute_enforces_input_and_output_limits() {
        let mut server = Server::new("http://localhost:11434".into(), "test".into());
        server.tool_graph_create(&serde_json::json!({"spec":{"name":"bounded","entry":"a","nodes":[{"id":"a","type":"passthrough"}],"edges":[]}})).unwrap();
        let at_limit_input = Value::String("x".repeat(MAX_EXECUTION_INPUT_BYTES - 2));
        assert!(server
            .tool_graph_execute(&serde_json::json!({"graph_id":"bounded","input":at_limit_input}))
            .is_ok());
        let oversized_input = Value::String("x".repeat(MAX_EXECUTION_INPUT_BYTES));
        assert!(server
            .tool_graph_execute(&serde_json::json!({"graph_id":"bounded","input":oversized_input}))
            .is_err());
        let oversized_result = GraphExecutionResult {
            success: true,
            final_state: Value::String("x".repeat(MAX_EXECUTION_OUTPUT_BYTES)),
            steps: vec![],
            error: None,
        };
        let at_limit_output = Value::String("x".repeat(MAX_EXECUTION_OUTPUT_BYTES - 2));
        assert!(ensure_value_within_limit(
            &at_limit_output,
            MAX_EXECUTION_OUTPUT_BYTES,
            "node output"
        )
        .is_ok());
        assert!(ensure_execution_output_within_limit(&oversized_result).is_err());
    }

    #[test]
    fn graph_and_execution_retention_evict_oldest_and_report_totals() {
        let mut server = Server::new("http://localhost:11434".into(), "test".into());
        for index in 0..=MAX_RETAINED_EXECUTIONS {
            server.record_execution(GraphExecutionResult {
                success: true,
                final_state: Value::String(format!("state-{index}")),
                steps: vec![StepResult {
                    node_id: "a".into(),
                    status: "done".into(),
                    output: Some(Value::String("unbounded output is not retained".into())),
                }],
                error: None,
            });
        }
        assert_eq!(server.executions.len(), MAX_RETAINED_EXECUTIONS);
        assert_eq!(server.total_execution_count, MAX_RETAINED_EXECUTIONS + 1);
        assert_eq!(server.executions.front().unwrap().step_count, 1);
        let status = server.tool_graph_status().unwrap();
        assert_eq!(status["retained_execution_count"], MAX_RETAINED_EXECUTIONS);
        assert_eq!(status["total_execution_count"], MAX_RETAINED_EXECUTIONS + 1);

        let mut graph_server = Server::new("http://localhost:11434".into(), "test".into());
        for index in 0..MAX_GRAPHS {
            graph_server.graphs.insert(
                format!("g{index}"),
                GraphSpec {
                    name: format!("g{index}"),
                    entry: "a".into(),
                    nodes: vec![NodeSpec {
                        id: "a".into(),
                        node_type: NodeType::Passthrough,
                        prompt: None,
                        model: None,
                        json_mode: false,
                        max_tokens: None,
                        routes: None,
                    }],
                    edges: vec![],
                    recursion_limit: None,
                },
            );
        }
        let overflow = serde_json::json!({"spec":{"name":"graph-overflow","entry":"a","nodes":[{"id":"a","type":"passthrough"}],"edges":[]}});
        assert!(graph_server.tool_graph_create(&overflow).is_err());
    }
}
