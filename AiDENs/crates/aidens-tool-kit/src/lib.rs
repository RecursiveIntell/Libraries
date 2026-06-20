//! Tool registry, exposure planning, and safe read-only dispatch.

pub mod canonical_stack;
mod descriptors;
mod dispatcher;
mod executors;
mod exposure;
mod patch;
mod registry;
mod sandbox;
#[cfg(test)]
mod tests;

// Delegate tool-runtime types to the canonical llm_tool_runtime crate.
pub use llm_tool_runtime;

pub use descriptors::*;
pub use dispatcher::*;
pub use exposure::ToolExposurePolicyV1;
pub use registry::*;
