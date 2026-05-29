#![doc = "Shared KV-cache pool primitives for receipt-bearing compression experiments."]

pub mod adapters;
pub mod codecs;
pub mod error;
pub mod manifest;
pub mod memory;
pub mod metrics;
pub mod pool;
pub mod reader;
pub mod receipts;

pub use codecs::q8_keys::{Q8KeyBlock, Q8KeyCodec};
pub use codecs::raw_exact::{ExactFallback, ExactFallbackRef, ExactKvBlock, RawExactCodec};
pub use codecs::value::{RawExactValueCodec, ValueCodec};
pub use error::PolyKvError;
pub use manifest::{
    BlockManifestEntryV1, CompressionPolicyV1, KvPoolManifestV1, QualityGateResultV1,
};
pub use memory::MemoryAccounting;
pub use pool::{DecodedKvSlice, DecodedLayer, PoolBuilder, SharedKvPool};
pub use reader::{PoolReader, ReaderConfig};
pub use receipts::{
    CompressionEvalReceiptV1, DecodeReceiptV1, FallbackReceiptV1, PoolBuildReceiptV1,
    ReaderInjectionReceiptV1,
};

pub use quant_codec_core::*;
