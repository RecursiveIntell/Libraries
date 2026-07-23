use serde_json::{json, Value};

/// List all available built-in templates.
pub fn list() -> Value {
    json!({
      "available": [
        {
          "id": "council_deliberation",
          "version": "1",
          "description": "Three-analyst parallel council: coordinator splits work, parallel researchers investigate, join synthesizes, judge produces final decision.",
          "params": ["name"],
          "storage_class": "server_builtin",
          "executable": true
        },
        {
          "id": "parallel_council",
          "version": "1",
          "description": "Two-perspective debate: optimist vs skeptic with judge synthesis.",
          "params": ["name"],
          "storage_class": "server_builtin",
          "executable": true
        },
        {
          "id": "plan_critique_refine",
          "version": "1",
          "description": "Sequential plan→critique→refine pipeline.",
          "params": ["name"],
          "storage_class": "server_builtin",
          "executable": true
        },
        {
          "id": "research_pipeline",
          "version": "1",
          "description": "Structured research: planner→searcher→extractor→synthesizer→validator with loop-back on failure.",
          "params": ["name"],
          "storage_class": "server_builtin",
          "executable": true
        },
        {
          "id": "classifier_router",
          "version": "1",
          "description": "LLM classifier routes input to category-specific handlers (bug/feature/question).",
          "params": ["name"],
          "storage_class": "server_builtin",
          "executable": true
        },
        {
          "id": "approval_gated_action",
          "version": "1",
          "description": "Draft→human_review→finalize with approval gate.",
          "params": ["name"],
          "storage_class": "server_builtin",
          "executable": true
        }
      ],
      "unavailable": [
        {
          "id": "map_reduce",
          "reason": "requires dynamic parallel branch count from input data"
        }
      ]
    })
}

/// Instantiate a template by ID, producing a valid GraphSpec JSON.
pub fn instantiate(id: &str, name: &str) -> Result<Value, String> {
    match id {
        // ── plan_critique_refine ──────────────────────────────────────
        "plan_critique_refine" => Ok(json!({
            "spec_version": "2",
            "name": name,
            "entry": "plan",
            "output_key": "final",
            "max_iterations": 12,
            "nodes": [
                {"id": "plan", "type": "llm", "prompt": "Create a concise plan for: {input}", "config": {"output_key": "draft"}},
                {"id": "critique", "type": "llm", "prompt": "Critique this plan: {input}", "config": {"input_key": "draft", "output_key": "critique"}},
                {"id": "refine", "type": "llm", "prompt": "Refine using this critique: {input}", "config": {"input_key": "critique", "output_key": "final"}}
            ],
            "edges": [
                {"from": "plan", "to": "critique"},
                {"from": "critique", "to": "refine"},
                {"from": "refine", "to": "END"}
            ]
        })),

        // ── parallel_council (2-person debate) ────────────────────────
        "parallel_council" => Ok(json!({
            "spec_version": "2",
            "name": name,
            "entry": "fanout",
            "output_key": "decision",
            "max_iterations": 8,
            "max_parallelism": 2,
            "nodes": [
                {"id": "fanout", "type": "passthrough"},
                {"id": "optimist", "type": "llm", "prompt": "Give the strongest case for: {input}", "config": {"output_key": "optimist"}},
                {"id": "skeptic", "type": "llm", "prompt": "Give the strongest critique of: {input}", "config": {"output_key": "skeptic"}},
                {"id": "join", "type": "join", "config": {"inputs": ["optimist", "skeptic"], "output": "council", "mode": "collect_array"}},
                {"id": "judge", "type": "llm", "prompt": "Judge these ordered views and produce a decision: {input}", "config": {"input_key": "council", "output_key": "decision"}}
            ],
            "edges": [
                {"from": "fanout", "to": "optimist"},
                {"from": "fanout", "to": "skeptic"},
                {"from": "optimist", "to": "join"},
                {"from": "skeptic", "to": "join"},
                {"from": "join", "to": "judge"},
                {"from": "judge", "to": "END"}
            ]
        })),

        // ── council_deliberation (3-analyst council) ──────────────────
        "council_deliberation" => Ok(json!({
            "spec_version": "2",
            "name": name,
            "entry": "coordinator",
            "output_key": "final_report",
            "max_iterations": 16,
            "max_parallelism": 3,
            "nodes": [
                {"id": "coordinator", "type": "llm", "prompt": "You are a research coordinator. Break this question into 3 distinct research workstreams. Output JSON: {\"workstreams\": [{\"id\":\"ws0\",\"query\":\"...\"}, {\"id\":\"ws1\",\"query\":\"...\"}, {\"id\":\"ws2\",\"query\":\"...\"}]}\n\nQuestion: {input}", "json_mode": true, "config": {"output_key": "workstreams"}},
                {"id": "fanout", "type": "passthrough"},
                {"id": "analyst_0", "type": "llm", "prompt": "Research workstream 0: {input}", "config": {"output_key": "ws0_result"}},
                {"id": "analyst_1", "type": "llm", "prompt": "Research workstream 1: {input}", "config": {"output_key": "ws1_result"}},
                {"id": "analyst_2", "type": "llm", "prompt": "Research workstream 2: {input}", "config": {"output_key": "ws2_result"}},
                {"id": "join", "type": "join", "config": {"inputs": ["ws0_result", "ws1_result", "ws2_result"], "output": "findings", "mode": "collect_array"}},
                {"id": "synthesize", "type": "llm", "prompt": "Synthesize these three research findings into a unified report with recommendations: {input}", "config": {"input_key": "findings", "output_key": "final_report"}}
            ],
            "edges": [
                {"from": "coordinator", "to": "fanout"},
                {"from": "fanout", "to": "analyst_0"},
                {"from": "fanout", "to": "analyst_1"},
                {"from": "fanout", "to": "analyst_2"},
                {"from": "analyst_0", "to": "join"},
                {"from": "analyst_1", "to": "join"},
                {"from": "analyst_2", "to": "join"},
                {"from": "join", "to": "synthesize"},
                {"from": "synthesize", "to": "END"}
            ]
        })),

        // ── research_pipeline (sequential with validation) ────────────
        "research_pipeline" => Ok(json!({
            "spec_version": "2",
            "name": name,
            "entry": "planner",
            "output_key": "final",
            "max_iterations": 20,
            "nodes": [
                {"id": "planner", "type": "llm", "prompt": "Create a research plan for: {input}. Output JSON with 'steps' array.", "json_mode": true, "config": {"output_key": "plan"}},
                {"id": "researcher", "type": "llm", "prompt": "Execute this research step: {input}", "config": {"input_key": "plan", "output_key": "research"}},
                {"id": "extractor", "type": "llm", "prompt": "Extract key claims and evidence from: {input}", "config": {"input_key": "research", "output_key": "claims"}},
                {"id": "synthesizer", "type": "llm", "prompt": "Synthesize these claims into a coherent summary: {input}", "config": {"input_key": "claims", "output_key": "summary"}},
                {"id": "validator", "type": "llm", "prompt": "Validate this summary. Is it accurate and complete? Respond with JSON: {\"valid\": true/false, \"issues\": [...]}", "json_mode": true, "config": {"input_key": "summary", "output_key": "validation"}},
                {"id": "formatter", "type": "llm", "prompt": "Format the final output: {input}", "config": {"input_key": "summary", "output_key": "final"}}
            ],
            "edges": [
                {"from": "planner", "to": "researcher"},
                {"from": "researcher", "to": "extractor"},
                {"from": "extractor", "to": "synthesizer"},
                {"from": "synthesizer", "to": "validator"},
                {"from": "validator", "to": "formatter"},
                {"from": "formatter", "to": "END"}
            ]
        })),

        // ── classifier_router ─────────────────────────────────────────
        "classifier_router" => Ok(json!({
            "spec_version": "2",
            "name": name,
            "entry": "classifier",
            "output_key": "response",
            "max_iterations": 8,
            "nodes": [
                {"id": "classifier", "type": "llm", "prompt": "Classify this input. Respond with exactly one word: 'bug', 'feature', or 'question'.\n\nInput: {input}", "config": {"output_key": "__input__"}},
                {"id": "router", "type": "router", "config": {
                    "rules": [
                        {"path": "__input__", "op": "contains", "value": "bug", "targets": ["bug_handler"]},
                        {"path": "__input__", "op": "contains", "value": "feature", "targets": ["feature_handler"]},
                        {"path": "__input__", "op": "contains", "value": "question", "targets": ["question_handler"]}
                    ],
                    "default": ["general_handler"]
                }},
                {"id": "bug_handler", "type": "llm", "prompt": "Analyze this bug report and suggest a fix: {input}", "config": {"output_key": "response"}},
                {"id": "feature_handler", "type": "llm", "prompt": "Evaluate this feature request: {input}", "config": {"output_key": "response"}},
                {"id": "question_handler", "type": "llm", "prompt": "Answer this question thoroughly: {input}", "config": {"output_key": "response"}},
                {"id": "general_handler", "type": "llm", "prompt": "Handle this general input: {input}", "config": {"output_key": "response"}}
            ],
            "edges": [
                {"from": "classifier", "to": "router"},
                {"from": "bug_handler", "to": "END"},
                {"from": "feature_handler", "to": "END"},
                {"from": "question_handler", "to": "END"},
                {"from": "general_handler", "to": "END"}
            ]
        })),

        // ── approval_gated_action ─────────────────────────────────────
        "approval_gated_action" => Ok(json!({
            "spec_version": "2",
            "name": name,
            "entry": "drafter",
            "output_key": "final",
            "max_iterations": 8,
            "nodes": [
                {"id": "drafter", "type": "llm", "prompt": "Draft a response for: {input}", "config": {"output_key": "draft"}},
                {"id": "human_review", "type": "human_approval", "config": {
                    "prompt_key": "draft",
                    "audience": ["operator"],
                    "allowed_decisions": ["approve", "reject", "request_changes"],
                    "expiry_ms": 300000,
                    "output_key": "review_decision"
                }},
                {"id": "finalizer", "type": "llm", "prompt": "Finalize based on review decision: {input}. Draft was: {draft}", "config": {"output_key": "final"}}
            ],
            "edges": [
                {"from": "drafter", "to": "human_review"},
                {"from": "human_review", "to": "finalizer"},
                {"from": "finalizer", "to": "END"}
            ]
        })),

        _ => Err(format!("template '{id}' is unavailable")),
    }
}

#[cfg(test)]
mod tests {
    use super::{instantiate, list};
    use crate::spec::GraphSpec;

    #[test]
    fn executable_templates_declare_explicit_terminal_outputs() {
        let catalog = list();
        let available = catalog["available"]
            .as_array()
            .expect("available templates");
        for template in available {
            if template["executable"] != true {
                continue;
            }
            let id = template["id"].as_str().expect("template id");
            let spec: GraphSpec =
                serde_json::from_value(instantiate(id, "contract-test").expect("template"))
                    .expect("valid graph spec");
            assert!(
                spec.output_key.is_some(),
                "template '{id}' must declare output_key"
            );
        }
    }
}
