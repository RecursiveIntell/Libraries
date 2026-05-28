//! quant-eval: Compression and semantic search evaluation benchmark suite.

mod benchmarks;
mod error;
mod fingerprint;
mod receipt;

pub use benchmarks::{
    AdmissibilityTest, CodecProfile, CompressionBenchmark, CompressionBenchmarkConfig,
    SemanticMemoryBenchmark, SemanticMemoryConfig,
};
pub use error::QuantEvalError;
pub use fingerprint::MachineFingerprint;
pub use receipt::{BenchmarkReceipt, BenchmarkResult, ReceiptDiff};
