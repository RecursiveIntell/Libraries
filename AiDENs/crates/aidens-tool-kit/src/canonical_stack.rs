pub use llm_tool_runtime::{
    validate_arguments_against_schema, McpSurfaceKind, Tool as CanonicalTool, ToolApprovalKind,
    ToolBackendKind, ToolCall as CanonicalToolCall, ToolCtx as CanonicalToolCtx,
    ToolDescriptor as CanonicalToolDescriptor, ToolError as CanonicalToolError, ToolExposureMode,
    ToolExposurePlan as CanonicalToolExposurePlan,
    ToolExposureRequest as CanonicalToolExposureRequest, ToolIdempotencyClass, ToolOutputMode,
    ToolReceipt as CanonicalToolReceipt, ToolReceiptPersistence,
    ToolRegistry as CanonicalToolRegistry, ToolResult as CanonicalToolResult,
    ToolRuntime as CanonicalToolRuntime, ToolRuntimeConfig as CanonicalToolRuntimeConfig,
};

pub fn validate_canonical_arguments(
    descriptor: &CanonicalToolDescriptor,
    arguments: &serde_json::Value,
) -> Result<(), CanonicalToolError> {
    validate_arguments_against_schema(&descriptor.input_schema, arguments)
}
