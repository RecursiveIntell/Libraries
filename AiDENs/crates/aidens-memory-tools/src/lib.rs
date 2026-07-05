//! Native semantic-memory, knowledge-runtime, and claim-ledger tools for AiDENs agents.
//!
//! This crate implements `CustomToolExecutor` for each memory operation, allowing
//! the AiDENs runner to call semantic-memory, knowledge-runtime, and claim-ledger
//! as native Rust function calls — no MCP server, no HTTP boundary, no JSON-RPC.
//!
//! ## Tool Categories
//!
//! - **Memory tools (read):** search, get_fact, list_facts, stats, graph_path, etc.
//! - **Memory tools (write):** add_fact, add_edge, ingest_document, update_fact, etc.
//! - **Knowledge-runtime tools:** classify_query, plan_query, query_orchestrated, etc.
//! - **Claim-ledger tools:** create_claim, add_evidence, judge_support, proof_debt, etc.
//!
//! ## Usage
//!
//! ```no_run
//! use aidens_memory_tools::{MemoryToolExecutor, memory_tool_plan, claim_tool_plan, ClaimToolExecutor};
//! use aidens_memory_kit::CanonicalMemoryAdapter;
//! use aidens_tool_kit::registry::ToolRegistryV1;
//! use std::sync::Arc;
//!
//! // Open memory store
//! let adapter = Arc::new(CanonicalMemoryAdapter::open(/* ... */).unwrap());
//!
//! // Build registry with memory + claim tools
//! let memory_exec = Arc::new(MemoryToolExecutor::new(adapter.clone()));
//! let claim_exec = Arc::new(ClaimToolExecutor::new());
//!
//! let mut registry = ToolRegistryV1::default();
//! for (desc, enabled) in memory_tool_plan() {
//!     registry.register_enabled_with_custom_executor(desc, enabled, memory_exec.clone());
//! }
//! for (desc, enabled) in claim_tool_plan() {
//!     registry.register_enabled_with_custom_executor(desc, enabled, claim_exec.clone());
//! }
//! ```

pub mod claim_executor;
pub mod descriptors;
pub mod executor;

pub use claim_executor::ClaimToolExecutor;
pub use descriptors::{claim_tool_plan, memory_tool_plan};
pub use executor::MemoryToolExecutor;
