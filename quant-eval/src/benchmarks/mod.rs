//! Benchmarks for quant-eval.
//!
//! Implements CompressionBenchmark, SemanticMemoryBenchmark, and AdmissibilityTest.

pub mod admissibility;
pub mod compression;
pub mod semantic;

pub use admissibility::{AdmissibilityTest, CodecProfile};
pub use compression::{CompressionBenchmark, CompressionBenchmarkConfig};
pub use semantic::{SemanticMemoryBenchmark, SemanticMemoryConfig};
