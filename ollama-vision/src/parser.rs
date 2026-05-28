//! LLM response parser — delegates to llm-output-parser crate.
//!
//! This module re-exports the parsing functions from llm-output-parser
//! for backward compatibility.

pub use llm_output_parser::parse_string_list as parse_tags;
pub use llm_output_parser::strip_think_tags;
pub use llm_output_parser::ParseError;
