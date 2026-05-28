//! Benchmarks for quant-eval.
//!
//! Implements CompressionBenchmark, SemanticMemoryBenchmark, and AdmissibilityTest.

pub mod compression;
pub mod semantic;
pub mod admissibility;

pub use compression::{CompressionBenchmark, CompressionBenchmarkConfig};
pub use semantic::{SemanticMemoryBenchmark, SemanticMemoryConfig};
pub use admissibility::{AdmissibilityTest, CodecProfile};