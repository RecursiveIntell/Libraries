use serde_json::{json, Value};

pub fn list() -> Value {
    json!({
      "available":[
        {"id":"plan_critique_refine","version":"1","storage_class":"server_builtin","executable":true},
        {"id":"parallel_council","version":"1","storage_class":"server_builtin","executable":true}
      ],
      "unavailable":[
        {"id":"approval_gated_action","reason":"correlation-bound durable resume is not implemented"},
        {"id":"implementation_verification","reason":"arbitrary Hermes tool execution is intentionally outside this MCP"},
        {"id":"adaptive_retrieval","reason":"external retrieval interrupt/resume is not implemented"}
      ]
    })
}

pub fn instantiate(id: &str, name: &str) -> Result<Value, String> {
    match id {
        "plan_critique_refine" => Ok(
            json!({"spec_version":"2","name":name,"entry":"plan","max_iterations":12,"nodes":[
          {"id":"plan","type":"llm","prompt":"Create a concise plan for: {input}","config":{"output_key":"draft"}},
          {"id":"critique","type":"llm","prompt":"Critique this plan: {input}","config":{"input_key":"draft","output_key":"critique"}},
          {"id":"refine","type":"llm","prompt":"Refine using this critique: {input}","config":{"input_key":"critique","output_key":"final"}}
        ],"edges":[{"from":"plan","to":"critique"},{"from":"critique","to":"refine"},{"from":"refine","to":"END"}]}),
        ),
        "parallel_council" => Ok(
            json!({"spec_version":"2","name":name,"entry":"fanout","max_iterations":8,"nodes":[
          {"id":"fanout","type":"passthrough"},
          {"id":"optimist","type":"llm","prompt":"Give the strongest case for: {input}","config":{"output_key":"optimist"}},
          {"id":"skeptic","type":"llm","prompt":"Give the strongest critique of: {input}","config":{"output_key":"skeptic"}},
          {"id":"join","type":"join","config":{"inputs":["optimist","skeptic"],"output":"council","mode":"collect_array"}},
          {"id":"judge","type":"llm","prompt":"Judge these ordered views: {input}","config":{"input_key":"council","output_key":"decision"}}
        ],"edges":[{"from":"fanout","to":"optimist"},{"from":"fanout","to":"skeptic"},{"from":"optimist","to":"join"},{"from":"skeptic","to":"join"},{"from":"join","to":"judge"},{"from":"judge","to":"END"}]}),
        ),
        _ => Err(format!("template '{id}' is unavailable")),
    }
}
