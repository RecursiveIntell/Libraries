use std::io::{BufRead, BufReader, Write};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};

use serde_json::{json, Value};

struct Mcp {
    child: Child,
    input: ChildStdin,
    output: BufReader<ChildStdout>,
    id: u64,
}

impl Mcp {
    fn new() -> Self {
        let mut child = Command::new(env!("CARGO_BIN_EXE_agent-graph-mcp"))
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();
        let input = child.stdin.take().unwrap();
        let output = BufReader::new(child.stdout.take().unwrap());
        Self {
            child,
            input,
            output,
            id: 0,
        }
    }

    fn request(&mut self, method: &str, params: Value) -> Value {
        self.id += 1;
        writeln!(
            self.input,
            "{}",
            json!({"jsonrpc":"2.0","id":self.id,"method":method,"params":params})
        )
        .unwrap();
        self.input.flush().unwrap();
        let mut line = String::new();
        self.output.read_line(&mut line).unwrap();
        serde_json::from_str(&line).unwrap()
    }

    fn call(&mut self, name: &str, arguments: Value) -> Value {
        let response = self.request("tools/call", json!({"name":name,"arguments":arguments}));
        response["result"]["structuredContent"].clone()
    }
}

impl Drop for Mcp {
    fn drop(&mut self) {
        let _ = self.child.kill();
    }
}

#[test]
fn legacy_contract_and_exact_tool_names() {
    let mut mcp = Mcp::new();
    let initialized = mcp.request("initialize", json!({}));
    assert_eq!(initialized["result"]["protocolVersion"], "2024-11-05");
    let list = mcp.request("tools/list", json!({}));
    let names: Vec<_> = list["result"]["tools"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v["name"].as_str().unwrap())
        .collect();
    assert_eq!(names, ["graph_create", "graph_execute", "graph_status"]);
    let created = mcp.call("graph_create", json!({"spec":{"name":"legacy","entry":"a","nodes":[{"id":"a","type":"passthrough"}],"edges":[]}}));
    assert_eq!(created["graph_id"], "legacy");
    let run = mcp.call(
        "graph_execute",
        json!({"graph_id":"legacy","input":{"x":1}}),
    );
    assert_eq!(run["success"], true);
    assert_eq!(run["final_state"], json!({"x":1}));
    assert!(run.get("run_id").is_some());
}

#[test]
fn validates_and_runs_parallel_transform_join_with_versions() {
    let mut mcp = Mcp::new();
    let spec = json!({
        "spec_version":"2", "name":"parallel", "entry":"fork", "max_iterations":8,
        "reducers":{"results":"append"},
        "nodes":[
          {"id":"fork","type":"passthrough"},
          {"id":"left","type":"state_transform","config":{"operations":[{"op":"append","path":"results","value":"left"}]}},
          {"id":"right","type":"state_transform","config":{"operations":[{"op":"append","path":"results","value":"right"}]}},
          {"id":"join","type":"join","config":{"inputs":["results"],"output":"joined","mode":"collect_array"}}
        ],
        "edges":[{"from":"fork","to":"left"},{"from":"fork","to":"right"},{"from":"left","to":"join"},{"from":"right","to":"join"},{"from":"join","to":"END"}]
    });
    let validated = mcp.call("graph_create", json!({"action":"validate","spec":spec}));
    assert_eq!(validated["status"], "valid");
    let created = mcp.call("graph_create", json!({"spec":spec}));
    assert!(created["graph_version"]
        .as_str()
        .unwrap()
        .starts_with("sha256:"));
    let run = mcp.call("graph_execute", json!({"graph_id":"parallel","input":{}}));
    assert_eq!(run["success"], true);
    assert_eq!(run["state"]["results"], json!(["left", "right"]));
    assert_eq!(run["receipt"]["replay_capability"], "integrity_verified");
    let events = mcp.call(
        "graph_status",
        json!({"resource":"events","run_id":run["run_id"],"cursor":0}),
    );
    assert!(events["events"].as_array().unwrap().iter().any(|entry| {
        entry["event"]["SuperstepStart"]["nodes"]
            .as_array()
            .is_some_and(|nodes| nodes.len() == 2)
    }));
}

#[test]
fn ordered_router_first_match_and_bounded_loop() {
    let mut mcp = Mcp::new();
    let spec = json!({"spec_version":"2","name":"route","entry":"route","max_iterations":6,"nodes":[
      {"id":"route","type":"router","config":{"rules":[
        {"path":"__input__","op":"contains","value":"deep","targets":["first"]},
        {"path":"__input__","op":"contains","value":"deep research","targets":["second"]}],"default":["END"]}},
      {"id":"first","type":"state_transform","config":{"operations":[{"op":"set","path":"chosen","value":"first"}]}},
      {"id":"second","type":"state_transform","config":{"operations":[{"op":"set","path":"chosen","value":"second"}]}}
    ],"edges":[{"from":"first","to":"END"},{"from":"second","to":"END"}]});
    mcp.call("graph_create", json!({"spec":spec}));
    let run = mcp.call(
        "graph_execute",
        json!({"graph_id":"route","input":"deep research"}),
    );
    assert_eq!(run["state"]["chosen"], "first");

    let looping = json!({"spec_version":"2","name":"loop","entry":"inc","max_iterations":4,"nodes":[
      {"id":"inc","type":"state_transform","config":{"operations":[{"op":"increment","path":"count","value":1}]}},
      {"id":"again","type":"router","config":{"rules":[{"path":"count","op":"lt","value":10,"targets":["inc"]}],"default":["END"]}}
    ],"edges":[{"from":"inc","to":"again"}]});
    mcp.call("graph_create", json!({"spec":looping}));
    let run = mcp.call("graph_execute", json!({"graph_id":"loop","input":{}}));
    assert_eq!(run["success"], false);
    assert!(run["error"].as_str().unwrap().contains("iterations"));
}

#[test]
fn registry_templates_security_and_bundle_verification() {
    let mut mcp = Mcp::new();
    let templates = mcp.call(
        "graph_status",
        json!({"resource":"templates","action":"list"}),
    );
    assert!(templates["available"]
        .as_array()
        .unwrap()
        .iter()
        .any(|v| v["id"] == "parallel_council"));
    let rejected = mcp.call("graph_create", json!({"action":"validate","spec":{"name":"evil","entry":"x","nodes":[{"id":"x","type":"shell","config":{"command":"id"}}],"edges":[]}}));
    assert_eq!(rejected["code"], "INVALID_SPEC");

    let spec =
        json!({"name":"evidence","entry":"x","nodes":[{"id":"x","type":"passthrough"}],"edges":[]});
    mcp.call("graph_create", json!({"spec":spec}));
    let run = mcp.call(
        "graph_execute",
        json!({"graph_id":"evidence","input":{"password":"do-not-export","safe":"ok"}}),
    );
    let bundle = mcp.call(
        "graph_status",
        json!({"resource":"bundle","run_id":run["run_id"]}),
    );
    assert_eq!(bundle["payload"]["input"]["password"], "[REDACTED]");
    let verified = mcp.call(
        "graph_execute",
        json!({"action":"verify_replay","bundle":bundle}),
    );
    assert_eq!(verified["verified"], true);
    let mut tampered = bundle;
    tampered["payload"]["output"] = json!({"changed":true});
    let rejected = mcp.call(
        "graph_execute",
        json!({"action":"verify_replay","bundle":tampered}),
    );
    assert_eq!(rejected["verified"], false);
}

#[test]
fn legacy_model_aliases_and_per_node_core_evidence_are_preserved() {
    let mut mcp = Mcp::new();
    let model_spec = json!({
        "name":"model-alias",
        "entry":"ask",
        "nodes":[{"id":"ask","type":"llm","model":"glm-5.2:cloud","prompt":"{input}"}],
        "edges":[]
    });
    let validated = mcp.call(
        "graph_create",
        json!({"action":"validate","spec":model_spec}),
    );
    assert_eq!(validated["status"], "valid");

    let evidence_spec = json!({
        "name":"node-evidence",
        "entry":"first",
        "nodes":[
            {"id":"first","type":"state_transform","config":{"operations":[{"op":"set","path":"__input__","value":{"stage":"first"}}]}},
            {"id":"second","type":"state_transform","config":{"operations":[{"op":"set","path":"__input__","value":{"stage":"second"}}]}}
        ],
        "edges":[{"from":"first","to":"second"},{"from":"second","to":"END"}]
    });
    mcp.call("graph_create", json!({"spec":evidence_spec}));
    let run = mcp.call(
        "graph_execute",
        json!({"graph_id":"node-evidence","input":{}}),
    );
    assert_eq!(run["success"], true);
    assert_eq!(run["steps"].as_array().unwrap().len(), 2);
    assert_eq!(run["steps"][0]["output"], json!({"stage":"first"}));
    assert_eq!(run["steps"][1]["output"], json!({"stage":"second"}));
    assert_eq!(run["receipt"]["core"]["steps"].as_array().unwrap().len(), 2);
    assert_eq!(run["receipt"]["core"]["steps"][0]["agent_id"], "first");
    assert_eq!(run["receipt"]["core"]["steps"][1]["agent_id"], "second");
}

#[test]
fn accepted_run_is_addressable_and_unsupported_boundaries_are_stable() {
    let mut mcp = Mcp::new();
    mcp.call("graph_create", json!({"spec":{"name":"async","entry":"x","nodes":[{"id":"x","type":"passthrough"}],"edges":[]}}));
    let accepted = mcp.call(
        "graph_execute",
        json!({"action":"start","wait":"accepted","graph_id":"async","input":{}}),
    );
    assert_eq!(accepted["status"], "accepted");
    let run = mcp.call(
        "graph_status",
        json!({"resource":"run","run_id":accepted["run_id"]}),
    );
    assert!(matches!(
        run["status"].as_str(),
        Some("accepted" | "running" | "completed")
    ));
    let unsupported = mcp.call(
        "graph_execute",
        json!({"action":"resume","run_id":accepted["run_id"]}),
    );
    assert_eq!(unsupported["code"], "UNSUPPORTED");
}
