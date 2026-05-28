//! P31 v11A boundary compiler microkernel scaffold.
//!
//! This is a standalone starter crate. It is intentionally narrow: strict JSON
//! boundary compilation with receipt artifacts. It is not the v11B graph compiler.

mod canonical;
mod digest;
mod json_boundary;
mod strict_json;
mod treatment;
mod types;

pub use crate::json_boundary::compile_json_boundary;
pub use crate::types::*;
