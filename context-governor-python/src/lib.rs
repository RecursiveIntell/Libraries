//! Python bindings for context-governor — context compaction for Hermes.
use context_governor::{
    compact_context, CompactRequest, CompactionPolicy, ContextCompactionReceiptV1, Message,
};
use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct PyMessage {
    role: String,
    content: String,
}

#[derive(Debug, Serialize)]
struct PyCompactResult {
    receipt_id: String,
    original_message_count: usize,
    compacted_message_count: usize,
    original_approx_tokens: usize,
    compacted_approx_tokens: usize,
    token_savings_estimate: isize,
    compacted_transcript_blake3: String,
    compacted_messages: Vec<PyMessage>,
    warnings: Vec<String>,
}

#[pyfunction]
#[pyo3(signature = (messages_json, session_id, target_tokens, protect_first_n=None, protect_last_n=None))]
fn compact(
    messages_json: &str,
    session_id: &str,
    target_tokens: usize,
    protect_first_n: Option<usize>,
    protect_last_n: Option<usize>,
) -> PyResult<String> {
    let py_msgs: Vec<PyMessage> = serde_json::from_str(messages_json)
        .map_err(|e| PyRuntimeError::new_err(format!("invalid messages JSON: {e}")))?;

    let msgs: Vec<Message> = py_msgs
        .iter()
        .map(|m| Message {
            role: m.role.clone(),
            content: m.content.clone(),
            ..Default::default()
        })
        .collect();

    let mut policy = CompactionPolicy::default();
    policy.target_tokens = target_tokens;
    if let Some(value) = protect_first_n {
        policy.protect_first_n = value;
    }
    if let Some(value) = protect_last_n {
        policy.protect_last_n = value;
    }

    let request = CompactRequest {
        session_id: session_id.to_string(),
        messages: msgs,
        policy,
        focus: None,
    };

    let response = compact_context(request).map_err(|e| PyRuntimeError::new_err(e.to_string()))?;

    let receipt: &ContextCompactionReceiptV1 = &response.receipt;

    let result = PyCompactResult {
        receipt_id: receipt.receipt_id.clone(),
        original_message_count: receipt.original_message_count,
        compacted_message_count: receipt.compacted_message_count,
        original_approx_tokens: receipt.original_approx_tokens,
        compacted_approx_tokens: receipt.compacted_approx_tokens,
        token_savings_estimate: receipt.token_savings_estimate,
        compacted_transcript_blake3: receipt.compacted_transcript_blake3.clone(),
        compacted_messages: response
            .compacted_messages
            .iter()
            .map(|m| PyMessage {
                role: m.role.clone(),
                content: m.content.clone(),
            })
            .collect(),
        warnings: receipt.warnings.clone(),
    };

    serde_json::to_string(&result).map_err(|e| PyRuntimeError::new_err(e.to_string()))
}

#[pymodule]
fn _native(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(compact, m)?)?;
    Ok(())
}
