use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const MAX_GRAPHS: usize = 64;
pub const MAX_GRAPH_BYTES: usize = 64 * 1024;
pub const MAX_NODES: usize = 128;
pub const MAX_EDGES: usize = 512;
pub const MAX_ITERATIONS: usize = 64;
pub const MAX_INPUT_BYTES: usize = 64 * 1024;
pub const MAX_OUTPUT_BYTES: usize = 128 * 1024;
pub const MAX_STATE_BYTES: usize = 2 * 1024 * 1024;

fn default_version() -> String {
    "1".into()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphSpec {
    #[serde(default = "default_version")]
    pub spec_version: String,
    pub name: String,
    pub entry: String,
    pub nodes: Vec<NodeSpec>,
    #[serde(default)]
    pub edges: Vec<EdgeSpec>,
    #[serde(default, alias = "recursion_limit")]
    pub max_iterations: Option<usize>,
    #[serde(default)]
    pub max_parallelism: Option<usize>,
    #[serde(default)]
    pub reducers: BTreeMap<String, ReducerKind>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeSpec {
    pub id: String,
    #[serde(rename = "type")]
    pub node_type: NodeType,
    #[serde(default)]
    pub prompt: Option<String>,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default)]
    pub json_mode: bool,
    #[serde(default)]
    pub max_tokens: Option<usize>,
    #[serde(default)]
    pub routes: Option<BTreeMap<String, String>>,
    #[serde(default)]
    pub config: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum NodeType {
    Llm,
    Router,
    Passthrough,
    StateTransform,
    Join,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgeSpec {
    pub from: String,
    pub to: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReducerKind {
    LastWriteWins,
    Append,
    Add,
    Merge,
}

impl GraphSpec {
    pub fn normalize(mut self) -> Self {
        self.spec_version = "2".into();
        if self.max_iterations.is_none() {
            self.max_iterations = Some(64);
        }
        if self.max_parallelism.is_none() {
            self.max_parallelism = Some(8);
        }
        self
    }

    pub fn warnings(&self) -> Vec<String> {
        let mut warnings = Vec::new();
        if self.nodes.iter().any(|n| n.routes.is_some()) {
            warnings.push("legacy route maps are normalized in lexicographic pattern order; use config.rules for explicit first-match order".into());
        }
        warnings
    }
}

pub fn parse_and_validate(raw: &Value) -> Result<GraphSpec, String> {
    ensure_size(raw, MAX_GRAPH_BYTES, "serialized graph spec")?;
    reject_dangerous_keys(raw)?;
    let spec: GraphSpec =
        serde_json::from_value(raw.clone()).map_err(|e| format!("invalid graph spec: {e}"))?;
    validate(&spec)?;
    Ok(spec.normalize())
}

pub fn validate(spec: &GraphSpec) -> Result<(), String> {
    if !valid_id(&spec.name) {
        return Err("graph name must match [A-Za-z0-9_.-]{1,64}".into());
    }
    if spec.nodes.is_empty() || spec.nodes.len() > MAX_NODES {
        return Err(format!("graph nodes must be 1..={MAX_NODES}"));
    }
    if spec.edges.len() > MAX_EDGES {
        return Err(format!("graph edge limit ({MAX_EDGES}) exceeded"));
    }
    let iterations = spec.max_iterations.unwrap_or(MAX_ITERATIONS);
    if iterations == 0 || iterations > MAX_ITERATIONS {
        return Err(format!("max_iterations must be 1..={MAX_ITERATIONS}"));
    }
    if spec.max_parallelism.unwrap_or(8) == 0 || spec.max_parallelism.unwrap_or(8) > 32 {
        return Err("max_parallelism must be 1..=32".into());
    }
    let ids: BTreeSet<_> = spec.nodes.iter().map(|n| n.id.as_str()).collect();
    if ids.len() != spec.nodes.len() {
        return Err("duplicate node ID".into());
    }
    if !ids.contains(spec.entry.as_str()) {
        return Err(format!("entry node '{}' not found", spec.entry));
    }
    for node in &spec.nodes {
        if !valid_id(&node.id) {
            return Err(format!("invalid node ID '{}'", node.id));
        }
        validate_node(node, &ids)?;
    }
    for edge in &spec.edges {
        if !ids.contains(edge.from.as_str()) {
            return Err(format!("edge source '{}' not found", edge.from));
        }
        if edge.to != "END" && !ids.contains(edge.to.as_str()) {
            return Err(format!("edge target '{}' not found", edge.to));
        }
    }
    Ok(())
}

fn validate_node(node: &NodeSpec, ids: &BTreeSet<&str>) -> Result<(), String> {
    if node.node_type == NodeType::Router {
        let targets: Vec<String> = if let Some(routes) = &node.routes {
            routes.values().cloned().collect()
        } else {
            node.config
                .get("rules")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
                .flat_map(|r| {
                    r.get("targets")
                        .and_then(Value::as_array)
                        .into_iter()
                        .flatten()
                })
                .filter_map(|v| v.as_str().map(str::to_owned))
                .chain(
                    node.config
                        .get("default")
                        .and_then(Value::as_array)
                        .into_iter()
                        .flatten()
                        .filter_map(|v| v.as_str().map(str::to_owned)),
                )
                .collect()
        };
        if targets.is_empty() {
            return Err(format!(
                "router node '{}' must define routes/rules and default",
                node.id
            ));
        }
        if node.routes.is_none() {
            if node
                .config
                .get("default")
                .and_then(Value::as_array)
                .is_none()
            {
                return Err(format!(
                    "router node '{}' requires explicit default",
                    node.id
                ));
            }
            for rule in node
                .config
                .get("rules")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
            {
                let op = rule.get("op").and_then(Value::as_str).unwrap_or("");
                if ![
                    "equals", "eq", "exists", "contains", "lt", "lte", "gt", "gte",
                ]
                .contains(&op)
                {
                    return Err(format!(
                        "router node '{}' has unsupported predicate '{op}'",
                        node.id
                    ));
                }
            }
        }
        for target in targets {
            if target != "END" && !ids.contains(target.as_str()) {
                return Err(format!(
                    "router node '{}' target '{}' not found",
                    node.id, target
                ));
            }
        }
    }
    if node.node_type == NodeType::Llm {
        if node
            .prompt
            .as_ref()
            .is_some_and(|prompt| prompt.len() > 16 * 1024)
        {
            return Err("LLM prompt exceeds 16384 bytes".into());
        }
        if node.max_tokens.unwrap_or(1024) > 8192 {
            return Err("LLM max_tokens exceeds 8192".into());
        }
        if node.model.as_ref().is_some_and(|m| !valid_model_alias(m)) {
            return Err("model must be a conservative server alias".into());
        }
        let timeout = node
            .config
            .get("timeout_ms")
            .and_then(Value::as_u64)
            .unwrap_or(120_000);
        if timeout == 0 || timeout > 120_000 {
            return Err("LLM timeout_ms must be 1..=120000".into());
        }
        if let Some(retry) = node.config.get("retry") {
            let attempts = retry
                .get("max_attempts")
                .and_then(Value::as_u64)
                .unwrap_or(3);
            if attempts == 0 || attempts > 5 {
                return Err("retry max_attempts must be 1..=5".into());
            }
        }
    }
    if node.node_type == NodeType::StateTransform {
        let operations = node
            .config
            .get("operations")
            .and_then(Value::as_array)
            .ok_or_else(|| format!("state_transform '{}' requires operations", node.id))?;
        if operations.is_empty() || operations.len() > 64 {
            return Err("transform operations must be 1..=64".into());
        }
        for operation in operations {
            let op = operation.get("op").and_then(Value::as_str).unwrap_or("");
            if ![
                "set",
                "copy",
                "delete",
                "increment",
                "append",
                "merge",
                "merge_object",
                "select",
                "compare",
                "format",
            ]
            .contains(&op)
            {
                return Err(format!("unsupported transform operation '{op}'"));
            }
        }
    }
    if node.node_type == NodeType::Join {
        let mode = node
            .config
            .get("mode")
            .and_then(Value::as_str)
            .unwrap_or("collect_array");
        if ![
            "collect_array",
            "merge_objects",
            "first_non_null",
            "all_success",
            "quorum",
        ]
        .contains(&mode)
        {
            return Err(format!("unsupported join mode '{mode}'"));
        }
        if node
            .config
            .get("inputs")
            .and_then(Value::as_array)
            .is_none()
            || node.config.get("output").and_then(Value::as_str).is_none()
        {
            return Err(format!("join '{}' requires inputs and output", node.id));
        }
    }
    Ok(())
}

pub fn valid_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 64
        && id
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"_.-".contains(&b))
}

fn valid_model_alias(model: &str) -> bool {
    !model.is_empty()
        && model.len() <= 128
        && !model.contains("://")
        && !model.starts_with('/')
        && !model.contains("..")
        && model
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"_.:/-".contains(&b))
}

pub fn ensure_size(value: &Value, limit: usize, label: &str) -> Result<(), String> {
    let len = serde_json::to_vec(value).map_err(|e| e.to_string())?.len();
    if len > limit {
        Err(format!("{label} exceeds {limit} bytes"))
    } else {
        Ok(())
    }
}

fn reject_dangerous_keys(value: &Value) -> Result<(), String> {
    const DENY: &[&str] = &[
        "command",
        "shell",
        "script",
        "filesystem",
        "file",
        "headers",
        "header",
        "secret",
        "env",
        "environment",
        "base_url",
        "provider_url",
        "url",
    ];
    match value {
        Value::Object(map) => {
            for (key, value) in map {
                let normalized = key.to_ascii_lowercase();
                if DENY.contains(&normalized.as_str()) {
                    return Err(format!("policy denied field '{key}'"));
                }
                reject_dangerous_keys(value)?;
            }
        }
        Value::Array(items) => {
            for item in items {
                reject_dangerous_keys(item)?;
            }
        }
        _ => {}
    }
    Ok(())
}
