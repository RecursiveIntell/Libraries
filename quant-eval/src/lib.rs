//! quant-eval: Compression and semantic search evaluation benchmark suite.

mod benchmarks;
mod compressed_attention;
mod compressed_scorer_real_corpus;
mod error;
mod fingerprint;
mod hyperquant_eval;
mod hyperquant_real_corpus;
mod rag;
mod receipt;

pub use benchmarks::{
    AdmissibilityTest, CodecProfile, CompressionBenchmark, CompressionBenchmarkConfig,
    SemanticMemoryBenchmark, SemanticMemoryConfig,
};
pub use compressed_attention::{
    run_compressed_attention_eval, CompressedAttentionConfig, CompressedAttentionReceipt,
};
pub use compressed_scorer_real_corpus::{
    run_compressed_scorer_real_corpus_eval, CompressedScorerRealCorpusConfig,
    CompressedScorerRealCorpusProfile, CompressedScorerRealCorpusReceipt,
};
pub use error::QuantEvalError;
pub use fingerprint::MachineFingerprint;
pub use hyperquant_eval::{
    run_hyperquant_eval, HyperQuantEvalConfig, HyperQuantEvalResult, HyperQuantProfileEval,
};
pub use hyperquant_real_corpus::{
    run_hyperquant_real_corpus_eval, HyperQuantRealCorpus, HyperQuantRealCorpusConfig,
    HyperQuantRealCorpusProfile, HyperQuantRealCorpusReceipt, RealCorpusDocument, RealCorpusQuery,
};
pub use rag::{evaluate_rag_fixture, RagEvalResult, RagQueryFixture, RagRetrievedDoc};
pub use receipt::{BenchmarkReceipt, BenchmarkResult, ReceiptDiff};
