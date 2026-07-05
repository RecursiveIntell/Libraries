//! Tool registry, exposure planning, and safe sandbox-validated dispatch.

pub mod canonical_stack;
mod custom;
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

pub use custom::{CustomExecutorHandle, CustomToolExecutor};
pub use descriptors::*;
pub use dispatcher::*;
pub use exposure::ToolExposurePolicyV1;
pub use registry::*;
