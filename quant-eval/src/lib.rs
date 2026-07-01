//! quant-eval: Compression and semantic search evaluation benchmark suite.

mod benchmarks;
mod compressed_attention;
mod error;
mod fingerprint;
mod hyperquant_eval;
mod hyperquant_retrieval;
mod rag;
mod receipt;

pub use compressed_attention::{
    run_synthetic_compressed_attention_bench, CompressedAttentionBenchConfig,
    CompressedAttentionBenchReceipt,
};

pub use benchmarks::{
    AdmissibilityTest, CodecProfile, CompressionBenchmark, CompressionBenchmarkConfig,
    SemanticMemoryBenchmark, SemanticMemoryConfig,
};
pub use error::QuantEvalError;
pub use fingerprint::MachineFingerprint;
pub use hyperquant_eval::{
    run_governed_hyperquant_eval, run_hyperquant_eval, GovernedHyperQuantBaseline,
    GovernedHyperQuantDecisionTrace, GovernedHyperQuantEvalConfig, GovernedHyperQuantEvalReceipt,
    GovernedHyperQuantPolicyPreset, HyperQuantEvalConfig, HyperQuantEvalResult,
    HyperQuantProfileEval,
};
pub use hyperquant_retrieval::{
    run_hyperquant_retrieval_benchmark, ErrorSummary, HyperQuantRetrievalBenchmarkConfig,
    HyperQuantRetrievalBenchmarkReceipt, HyperQuantRetrievalQuality, HyperQuantRetrievalThresholds,
    LatencySummaryNs, RankDriftSummary,
};
pub use rag::{evaluate_rag_fixture, RagEvalResult, RagQueryFixture, RagRetrievedDoc};
pub use receipt::{BenchmarkReceipt, BenchmarkResult, ReceiptDiff};
