//! `claim-ledger-mcp` library surface: the MCP server plus the trusted-head
//! projection module (FINISH_PLAN Phase 3.3). The binary in `main.rs` is a
//! thin adapter over this library so integration tests exercise the exact
//! production projection path.

pub mod server;
pub mod tools;
pub mod trusted_head;
