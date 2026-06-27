//! quant-eval: Compression and semantic search evaluation benchmark suite.

mod benchmarks;
mod error;
mod fingerprint;
mod hyperquant_eval;
mod rag;
mod receipt;

pub use benchmarks::{
    AdmissibilityTest, CodecProfile, CompressionBenchmark, CompressionBenchmarkConfig,
    SemanticMemoryBenchmark, SemanticMemoryConfig,
};
pub use error::QuantEvalError;
pub use fingerprint::MachineFingerprint;
pub use hyperquant_eval::{
    run_hyperquant_eval, HyperQuantEvalConfig, HyperQuantEvalResult, HyperQuantProfileEval,
};
pub use rag::{evaluate_rag_fixture, RagEvalResult, RagQueryFixture, RagRetrievedDoc};
pub use receipt::{BenchmarkReceipt, BenchmarkResult, ReceiptDiff};
