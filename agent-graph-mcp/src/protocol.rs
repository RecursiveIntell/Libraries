use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Deserialize)]
pub struct RpcRequest {
    #[allow(dead_code)]
    pub jsonrpc: String,
    pub id: Value,
    pub method: String,
    #[serde(default)]
    pub params: Value,
}

#[derive(Debug, Serialize)]
pub struct RpcResponse {
    pub jsonrpc: String,
    pub id: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<RpcError>,
}

#[derive(Debug, Serialize)]
pub struct RpcError {
    pub code: i32,
    pub message: String,
}

pub fn tool_success(id: &Value, value: Value) -> RpcResponse {
    RpcResponse {
        jsonrpc: "2.0".into(),
        id: id.clone(),
        result: Some(serde_json::json!({
            "content":[{"type":"text","text":serde_json::to_string_pretty(&value).unwrap_or_default()}],
            "structuredContent": value
        })),
        error: None,
    }
}

pub fn tool_error(id: &Value, code: &str, message: impl Into<String>) -> RpcResponse {
    let value = serde_json::json!({"code":code,"error":message.into()});
    RpcResponse {
        jsonrpc: "2.0".into(),
        id: id.clone(),
        result: Some(serde_json::json!({
            "content":[{"type":"text","text":serde_json::to_string(&value).unwrap_or_default()}],
            "structuredContent":value,"isError":true
        })),
        error: None,
    }
}
