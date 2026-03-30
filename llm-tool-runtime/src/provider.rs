use crate::{ToolBackendKind, ToolDescriptor, ToolError, ToolErrorClass};
use serde_json::{json, Value};

/// Renders an OpenAI-compatible tool schema from an internal descriptor.
pub fn render_openai_tool(descriptor: &ToolDescriptor, strict: bool) -> Result<Value, ToolError> {
    match descriptor.backend_kind {
        ToolBackendKind::LocalFunction | ToolBackendKind::OpenAiFunction => Ok(json!({
            "type": "function",
            "name": descriptor.name,
            "description": descriptor.description.clone().unwrap_or_default(),
            "parameters": descriptor.input_schema,
            "strict": strict,
        })),
        ToolBackendKind::OpenAiBuiltIn | ToolBackendKind::RemoteMcp => {
            descriptor.provider_payload.clone().ok_or_else(|| {
                ToolError::new(
                    ToolErrorClass::ProviderContract,
                    format!(
                        "tool {} requires provider_payload for OpenAI rendering",
                        descriptor.name
                    ),
                )
            })
        }
        ToolBackendKind::OllamaFunction => Err(ToolError::new(
            ToolErrorClass::ProviderContract,
            format!("tool {} is Ollama-only", descriptor.name),
        )),
    }
}

/// Renders an Ollama-compatible tool schema from an internal descriptor.
pub fn render_ollama_tool(descriptor: &ToolDescriptor) -> Result<Value, ToolError> {
    match descriptor.backend_kind {
        ToolBackendKind::LocalFunction | ToolBackendKind::OllamaFunction => Ok(json!({
            "type": "function",
            "function": {
                "name": descriptor.name,
                "description": descriptor.description.clone().unwrap_or_default(),
                "parameters": descriptor.input_schema,
            }
        })),
        _ => Err(ToolError::new(
            ToolErrorClass::ProviderContract,
            format!("tool {} cannot be rendered for Ollama", descriptor.name),
        )),
    }
}
