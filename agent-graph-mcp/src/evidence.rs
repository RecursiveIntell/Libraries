use serde_json::Value;
use sha2::{Digest, Sha256};

pub fn digest(value: &Value) -> String {
    let bytes = serde_json::to_vec(value).unwrap_or_default();
    let hash = Sha256::digest(bytes);
    format!("sha256:{hash:x}")
}

pub fn redact(value: &Value) -> Value {
    match value {
        Value::Object(map) => Value::Object(
            map.iter()
                .map(|(k, v)| {
                    let lower = k.to_ascii_lowercase();
                    let redacted = [
                        "secret",
                        "token",
                        "password",
                        "authorization",
                        "api_key",
                        "credential",
                    ]
                    .iter()
                    .any(|needle| lower.contains(needle));
                    (
                        k.clone(),
                        if redacted {
                            Value::String("[REDACTED]".into())
                        } else {
                            redact(v)
                        },
                    )
                })
                .collect(),
        ),
        Value::Array(items) => Value::Array(items.iter().map(redact).collect()),
        Value::String(text)
            if text.starts_with("sk-")
                || text.starts_with("Bearer ")
                || text.contains("BEGIN PRIVATE KEY") =>
        {
            Value::String("[REDACTED]".into())
        }
        value => value.clone(),
    }
}

pub fn bundle(
    run_id: &str,
    graph_version: &str,
    input: &Value,
    output: &Value,
    receipt: &Value,
) -> Value {
    let payload = serde_json::json!({"schema":"agent-graph-mcp-bundle-v1","run_id":run_id,"graph_version":graph_version,
        "input":redact(input),"output":redact(output),"receipt":redact(receipt),"replay_capability":"integrity_verified",
        "dependency_envelopes_complete":false,"environment":{}});
    let integrity = digest(&payload);
    serde_json::json!({"payload":payload,"integrity":integrity})
}

pub fn verify(bundle: &Value) -> Value {
    let Some(payload) = bundle.get("payload") else {
        return serde_json::json!({"verified":false,"code":"INVALID_BUNDLE"});
    };
    let expected = bundle
        .get("integrity")
        .and_then(Value::as_str)
        .unwrap_or("");
    let actual = digest(payload);
    serde_json::json!({"verified":expected==actual,"level":"integrity_verified","expected":expected,"actual":actual,"models_or_tools_invoked":false})
}
