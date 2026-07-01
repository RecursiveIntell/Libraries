#![allow(
    clippy::derivable_impls,
    clippy::manual_range_contains,
    clippy::needless_borrow,
    clippy::needless_range_loop,
    clippy::redundant_closure_call,
    clippy::too_many_arguments,
    clippy::type_complexity,
    clippy::unnecessary_lazy_evaluations,
    clippy::useless_format
)]

//! semantic-memory-mcp — MCP server for semantic-memory.
//!
//! Library target for integration tests. The main binary entry point
//! is in `main.rs`; this module re-exports the public modules so
//! integration tests can access bridge and http_server.

pub mod bridge;
pub mod http_server;
pub mod server;
mod tools;
