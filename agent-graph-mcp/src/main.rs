use std::collections::HashMap;
use std::io::{self, BufRead, Write};

use agent_graph::state::AgentState;
use llm_pipeline::payload::Payload;
use llm_pipeline::{ExecCtx, LlmCall, LlmConfig};
use serde::{Deserialize, Serialize};
use serde_json::Value;

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
    executions: Vec<GraphExecutionResult>,
}

impl Server {
    fn new(base_url: String, default_model: String) -> Self {
        Self {
            base_url,
            default_model,
            graphs: HashMap::new(),
            executions: Vec::new(),
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
                    "content": [{ "type": "text", "text": "{\"error\": \"".to_string() + &msg + "\"}" }],
                    "isError": true,
                })),
                error: None,
            },
        }
    }

    fn tool_graph_create(&mut self, args: &Value) -> Result<Value, String> {
        let spec: GraphSpec = serde_json::from_value(args.get("spec").cloned().unwrap_or(Value::Null))
            .map_err(|e| format!("invalid graph spec: {e}"))?;

        if spec.nodes.is_empty() {
            return Err("graph must have at least one node".into());
        }
        if !spec.nodes.iter().any(|n| n.id == spec.entry) {
            return Err(format!("entry node '{}' not found", spec.entry));
        }

        let mut seen = std::collections::HashSet::new();
        for node in &spec.nodes {
            if !seen.insert(&node.id) {
                return Err(format!("duplicate node ID: {}", node.id));
            }
        }

        let graph_id = spec.name.clone();
        self.graphs.insert(graph_id.clone(), spec);
        Ok(serde_json::json!({
            "graph_id": graph_id,
            "status": "created"
        }))
    }

    fn tool_graph_execute(&mut self, args: &Value) -> Result<Value, String> {
        let graph_id = args.get("graph_id").and_then(|v| v.as_str()).ok_or("missing graph_id")?;
        let input = args.get("input").cloned().unwrap_or(Value::Null);
        let spec = self.graphs.get(graph_id).ok_or(format!("graph '{graph_id}' not found"))?.clone();

        let rt = tokio::runtime::Runtime::new().map_err(|e| format!("runtime error: {e}"))?;
        let result = rt.block_on(async { self.execute_graph(&spec, &input).await })?;

        self.executions.push(result.clone());
        Ok(serde_json::to_value(&result).map_err(|e| format!("serialize error: {e}"))?)
    }

    fn tool_graph_status(&self) -> Result<Value, String> {
        Ok(serde_json::json!({
            "graphs": self.graphs.keys().collect::<Vec<_>>(),
            "graph_count": self.graphs.len(),
            "execution_count": self.executions.len(),
            "base_url": self.base_url,
            "default_model": self.default_model,
        }))
    }

    async fn execute_graph(&self, spec: &GraphSpec, input: &Value) -> Result<GraphExecutionResult, String> {
        let state = AgentState::new();
        state.set("__input__", input.clone()).await.ok();

        let mut steps: Vec<StepResult> = Vec::new();
        let mut current_node = spec.entry.clone();
        let mut visited = std::collections::HashSet::new();
        let recursion_limit = spec.recursion_limit.unwrap_or(50);

        // Build node lookup
        let node_map: HashMap<&str, &NodeSpec> = spec.nodes.iter().map(|n| (n.id.as_str(), n)).collect();
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
                    final_state: state.get::<Value>("__input__").await.ok().unwrap_or(Value::Null),
                    steps,
                    error: Some(format!("recursion limit ({recursion_limit}) exceeded")),
                });
            }
            if visited.contains(&current_node) {
                return Ok(GraphExecutionResult {
                    success: false,
                    final_state: state.get::<Value>("__input__").await.ok().unwrap_or(Value::Null),
                    steps,
                    error: Some(format!("cycle detected at '{current_node}'")),
                });
            }
            visited.insert(current_node.clone());

            let node = node_map.get(current_node.as_str())
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
                    let output = llm_call.invoke(&ctx, input_val).await
                        .map_err(|e| format!("LLM call failed at '{current_node}': {e}"))?;

                    let output_val = output.value;
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
                    let routes = node.routes.as_ref().unwrap();

                    let next = routes.iter().find_map(|(pattern, target)| {
                        if input_str.contains(pattern) { Some(target.clone()) } else { None }
                    });

                    steps.push(StepResult {
                        node_id: current_node.clone(),
                        status: "router".into(),
                        output: Some(Value::String(next.clone().unwrap_or_else(|| "END".into()))),
                    });

                    match next {
                        Some(target) => { current_node = target; continue; }
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
                    if next == "END" { break; }
                    current_node = next.to_string();
                }
                _ => break,
            }
        }

        let final_state: Value = state.get("__input__").await.ok().unwrap_or(Value::Null);
        Ok(GraphExecutionResult { success: true, final_state, steps, error: None })
    }
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
            "--base-url" => { if let Some(v) = iter.next() { base_url = v.clone(); } }
            "--model" => { if let Some(v) = iter.next() { default_model = v.clone(); } }
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
        let line = match line { Ok(l) => l, Err(_) => break };
        if line.trim().is_empty() { continue; }

        let req: RpcRequest = match serde_json::from_str(&line) {
            Ok(r) => r,
            Err(e) => { eprintln!("JSON parse error: {e}"); continue; }
        };

        let resp = server.handle_request(&req);
        let json = serde_json::to_string(&resp).unwrap_or_else(|_| {
            r#"{"jsonrpc":"2.0","id":null,"error":{"code":-32603,"message":"serialize error"}}"#.into()
        });

        if writeln!(stdout, "{json}").is_err() { break; }
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
        let node_json = r#"{"id":"decide","type":"router","routes":{"deep":"research","shallow":"summarize"}}"#;
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
        let result = server.tool_graph_execute(&serde_json::json!({"graph_id":"pass","input":"hello"})).unwrap();
        assert_eq!(result["success"], true);
        assert!(result["steps"].as_array().unwrap().len() >= 2);
    }

    #[test]
    fn execute_router_deep_path() {
        let mut server = Server::new("http://localhost:11434".into(), "test".into());
        server.tool_graph_create(&serde_json::json!({"spec":{"name":"routed","entry":"start","nodes":[{"id":"start","type":"passthrough"},{"id":"decide","type":"router","routes":{"deep":"deep_node","shallow":"shallow_node"}},{"id":"deep_node","type":"passthrough"},{"id":"shallow_node","type":"passthrough"}],"edges":[{"from":"start","to":"decide"},{"from":"deep_node","to":"END"},{"from":"shallow_node","to":"END"}]}})).unwrap();
        let result = server.tool_graph_execute(&serde_json::json!({"graph_id":"routed","input":"this is a deep research question"})).unwrap();
        assert_eq!(result["success"], true);
        let router_step = result["steps"].as_array().unwrap().iter().find(|s| s["node_id"] == "decide").unwrap();
        assert_eq!(router_step["output"], "deep_node");
    }

    #[test]
    fn execute_router_shallow_path() {
        let mut server = Server::new("http://localhost:11434".into(), "test".into());
        server.tool_graph_create(&serde_json::json!({"spec":{"name":"routed","entry":"start","nodes":[{"id":"start","type":"passthrough"},{"id":"decide","type":"router","routes":{"deep":"deep_node","shallow":"shallow_node"}},{"id":"deep_node","type":"passthrough"},{"id":"shallow_node","type":"passthrough"}],"edges":[{"from":"start","to":"decide"},{"from":"deep_node","to":"END"},{"from":"shallow_node","to":"END"}]}})).unwrap();
        let result = server.tool_graph_execute(&serde_json::json!({"graph_id":"routed","input":"this is shallow"})).unwrap();
        assert_eq!(result["success"], true);
        let router_step = result["steps"].as_array().unwrap().iter().find(|s| s["node_id"] == "decide").unwrap();
        assert_eq!(router_step["output"], "shallow_node");
    }

    #[test]
    fn mcp_initialize() {
        let mut server = Server::new("http://localhost:11434".into(), "test".into());
        let req = RpcRequest { jsonrpc: "2.0".into(), id: serde_json::json!(1), method: "initialize".into(), params: Value::Null };
        let resp = server.handle_request(&req);
        assert_eq!(resp.result.unwrap()["serverInfo"]["name"], "agent-graph-mcp");
    }

    #[test]
    fn mcp_tools_list() {
        let mut server = Server::new("http://localhost:11434".into(), "test".into());
        let req = RpcRequest { jsonrpc: "2.0".into(), id: serde_json::json!(2), method: "tools/list".into(), params: Value::Null };
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
        let result = server.tool_graph_execute(&serde_json::json!({"graph_id":"cycle","input":"test"})).unwrap();
        assert_eq!(result["success"], false);
        assert!(result["error"].as_str().unwrap().contains("cycle"));
    }
}
