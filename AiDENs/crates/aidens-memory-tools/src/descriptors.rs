//! Tool descriptors for memory and claim tools.

use aidens_contracts::ToolDescriptorV1;

// Placeholder — full implementation in progress.
// Each descriptor function returns a ToolDescriptorV1 with namespace="aidens",
// the tool name, version="1", a description for the model, risk_class, and
// input_schema (JSON Schema).

pub fn memory_tool_plan() -> Vec<(ToolDescriptorV1, bool)> {
    Vec::new()
}

pub fn claim_tool_plan() -> Vec<(ToolDescriptorV1, bool)> {
    Vec::new()
}
