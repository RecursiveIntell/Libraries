use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::policy::CodecId;

/// A codec compresses and decompresses KV vectors.
pub trait KVecCodec: Send + Sync {
    /// Return the codec identifier ("fib_k4_n32", "turbo_8bit", etc.).
    fn codec_id(&self) -> CodecId;

    /// Encode a vector of f32 values into a compressed byte payload.
    fn encode(&self, vector: &[f32], seed: u64) -> Result<Vec<u8>>;

    /// Decode a compressed byte payload back into a vector of f32 values.
    fn decode(&self, payload: &[u8], seed: u64) -> Result<Vec<f32>>;

    /// The expected dimension of input/output vectors.
    fn dim(&self) -> usize;

    /// Expected compression ratio (nominal).
    fn compression_ratio(&self) -> f64;
}

/// A serialized compressed block with codec metadata.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompressedBlock {
    /// Codec identifier.
    pub codec: CodecId,
    /// The compressed payload bytes.
    pub encoded_payload: Vec<u8>,
    /// Blake3 digest of the encoded payload.
    pub payload_digest: String,
    /// Original (uncompressed) vector dimension.
    pub original_dim: usize,
    /// Size of the compressed payload in bytes.
    pub compressed_bytes: usize,
}

impl CompressedBlock {
    /// Create a new CompressedBlock from encoded payload.
    pub fn new(codec: CodecId, encoded_payload: Vec<u8>, original_dim: usize) -> Self {
        let compressed_bytes = encoded_payload.len();
        let payload_digest = blake3::hash(&encoded_payload).to_hex().to_string();
        Self {
            codec,
            encoded_payload,
            payload_digest,
            original_dim,
            compressed_bytes,
        }
    }

    /// Compression ratio: original f32 bytes / compressed bytes.
    pub fn compression_ratio(&self) -> f64 {
        let raw_bytes = self.original_dim * 4; // 4 bytes per f32
        if self.compressed_bytes == 0 {
            return f64::INFINITY;
        }
        raw_bytes as f64 / self.compressed_bytes as f64
    }
}

// ── Exact fallback codec (no compression) ──

/// Exact fallback codec: stores raw f32 bytes with no compression.
pub struct ExactFallbackCodec {
    dim: usize,
}

impl ExactFallbackCodec {
    pub fn new(dim: usize) -> Self {
        Self { dim }
    }
}

impl KVecCodec for ExactFallbackCodec {
    fn codec_id(&self) -> CodecId {
        crate::policy::CODEC_EXACT_FALLBACK.into()
    }

    fn encode(&self, vector: &[f32], _seed: u64) -> Result<Vec<u8>> {
        if vector.len() != self.dim {
            return Err(crate::error::PolyKvError::DimensionMismatch {
                expected: self.dim,
                got: vector.len(),
            });
        }
        // Store raw f32 bytes in little-endian
        let bytes: Vec<u8> = vector.iter().flat_map(|v| v.to_le_bytes()).collect();
        Ok(bytes)
    }

    fn decode(&self, payload: &[u8], _seed: u64) -> Result<Vec<f32>> {
        let expected_len = self.dim * 4;
        if payload.len() != expected_len {
            return Err(crate::error::PolyKvError::CorruptPayload(format!(
                "exact fallback payload size {} != expected {}",
                payload.len(),
                expected_len
            )));
        }
        let mut vec = Vec::with_capacity(self.dim);
        for chunk in payload.chunks_exact(4) {
            let arr: [u8; 4] = chunk.try_into().unwrap();
            vec.push(f32::from_le_bytes(arr));
        }
        Ok(vec)
    }

    fn dim(&self) -> usize {
        self.dim
    }

    fn compression_ratio(&self) -> f64 {
        1.0
    }
}

// ── TurboQuant adapter ──

/// Adapter for the turbo-quant crate (8-bit, 32 projections).
#[cfg(feature = "turbo")]
pub struct TurboQuantAdapter {
    dim: usize,
    bits: u8,
    projections: usize,
}

#[cfg(feature = "turbo")]
impl TurboQuantAdapter {
    #[allow(clippy::too_many_arguments)]
    pub fn new(dim: usize, bits: u8, projections: usize) -> Result<Self> {
        if dim == 0 {
            return Err(crate::error::PolyKvError::InvalidPolicy(
                "turbo dim must be > 0".into(),
            ));
        }
        if dim % 2 != 0 {
            // Pad to even — turbo-quant requires even dimensions
            return Err(crate::error::PolyKvError::InvalidPolicy(format!(
                "turbo requires even dimension, got {}",
                dim
            )));
        }
        Ok(Self {
            dim,
            bits,
            projections,
        })
    }
}

#[cfg(feature = "turbo")]
impl KVecCodec for TurboQuantAdapter {
    fn codec_id(&self) -> CodecId {
        crate::policy::CODEC_TURBO_8BIT.into()
    }

    fn encode(&self, vector: &[f32], seed: u64) -> Result<Vec<u8>> {
        let quantizer =
            turbo_quant::TurboQuantizer::new(self.dim, self.bits, self.projections, seed).map_err(
                |e| {
                    crate::error::PolyKvError::CompressionFailed(format!(
                        "turbo quantizer init failed: {}",
                        e
                    ))
                },
            )?;

        let code = quantizer.encode(vector).map_err(|e| {
            crate::error::PolyKvError::CompressionFailed(format!("turbo encode failed: {}", e))
        })?;

        // Serialize TurboCode to JSON then to bytes
        serde_json::to_vec(&code).map_err(crate::error::PolyKvError::Serialization)
    }

    fn decode(&self, payload: &[u8], seed: u64) -> Result<Vec<f32>> {
        let code: turbo_quant::TurboCode = serde_json::from_slice(payload).map_err(|e| {
            crate::error::PolyKvError::DecompressionFailed(format!(
                "turbo code deserialize failed: {}",
                e
            ))
        })?;

        // Reconstruct from polar component via independent PolarQuantizer.
        // QJL residual is lossy and not invertible, so we return the polar
        // approximation.
        let polar_quant =
            turbo_quant::PolarQuantizer::new(self.dim, self.bits - 1, seed).map_err(|e| {
                crate::error::PolyKvError::DecompressionFailed(format!(
                    "turbo polar quantizer init failed: {}",
                    e
                ))
            })?;

        let reconstructed = polar_quant.decode(&code.polar_code).map_err(|e| {
            crate::error::PolyKvError::DecompressionFailed(format!("turbo decode failed: {}", e))
        })?;

        Ok(reconstructed)
    }

    fn dim(&self) -> usize {
        self.dim
    }

    fn compression_ratio(&self) -> f64 {
        8.0
    }
}

// ── FibQuant adapter ──

/// Adapter for the fib-quant crate (k=4, N=32, paper core path).
#[cfg(feature = "fib")]
pub struct FibQuantAdapter {
    dim: usize,
    k: u32,
    n: u32,
    training_samples: u32,
    lloyd_restarts: u32,
    lloyd_iterations: u32,
}

#[cfg(feature = "fib")]
impl FibQuantAdapter {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        dim: usize,
        k: u32,
        n: u32,
        training_samples: u32,
        lloyd_restarts: u32,
        lloyd_iterations: u32,
    ) -> Result<Self> {
        if dim == 0 {
            return Err(crate::error::PolyKvError::InvalidPolicy(
                "fib dim must be > 0".into(),
            ));
        }
        if dim % k as usize != 0 {
            return Err(crate::error::PolyKvError::InvalidPolicy(format!(
                "fib ambient dim ({}) must be divisible by k ({})",
                dim, k
            )));
        }
        Ok(Self {
            dim,
            k,
            n,
            training_samples,
            lloyd_restarts,
            lloyd_iterations,
        })
    }

    /// Build a FibQuantizer for the given seed.
    fn build_quantizer(
        &self,
        seed: u64,
    ) -> std::result::Result<fib_quant::FibQuantizer, crate::error::PolyKvError> {
        let mut profile = fib_quant::FibQuantProfileV1::paper_default(
            self.dim,
            self.k as usize,
            self.n as usize,
            seed,
        )
        .map_err(|e| {
            crate::error::PolyKvError::CompressionFailed(format!("fib profile build failed: {}", e))
        })?;

        // Override training parameters
        profile.training_samples = self.training_samples;
        profile.lloyd_restarts = self.lloyd_restarts;
        profile.lloyd_iterations = self.lloyd_iterations;

        fib_quant::FibQuantizer::new(profile).map_err(|e| {
            crate::error::PolyKvError::CompressionFailed(format!(
                "fib quantizer build failed: {}",
                e
            ))
        })
    }
}

#[cfg(feature = "fib")]
impl KVecCodec for FibQuantAdapter {
    fn codec_id(&self) -> CodecId {
        crate::policy::CODEC_FIB_K4_N32.into()
    }

    fn encode(&self, vector: &[f32], seed: u64) -> Result<Vec<u8>> {
        let quantizer = self.build_quantizer(seed)?;
        let code = quantizer.encode(vector).map_err(|e| {
            crate::error::PolyKvError::CompressionFailed(format!("fib encode failed: {}", e))
        })?;

        serde_json::to_vec(&code).map_err(crate::error::PolyKvError::Serialization)
    }

    fn decode(&self, payload: &[u8], seed: u64) -> Result<Vec<f32>> {
        let code: fib_quant::FibCodeV1 = serde_json::from_slice(payload).map_err(|e| {
            crate::error::PolyKvError::DecompressionFailed(format!(
                "fib code deserialize failed: {}",
                e
            ))
        })?;

        let quantizer = self.build_quantizer(seed)?;
        let decoded = quantizer.decode(&code).map_err(|e| {
            crate::error::PolyKvError::DecompressionFailed(format!("fib decode failed: {}", e))
        })?;

        Ok(decoded)
    }

    fn dim(&self) -> usize {
        self.dim
    }

    fn compression_ratio(&self) -> f64 {
        50.0
    }
}

/// Create a codec from a policy and vector dimension.
///
/// Returns the appropriate codec based on the codec_id in the policy.
/// If the required compression crate is unavailable, returns an error.
#[allow(clippy::too_many_arguments)]
pub fn create_codec(
    codec_id: &str,
    dim: usize,
    fib_config: Option<&crate::policy::FibConfig>,
    turbo_config: Option<&crate::policy::TurboConfig>,
) -> Result<Box<dyn KVecCodec>> {
    match codec_id {
        crate::policy::CODEC_FIB_K4_N32 => {
            #[cfg(feature = "fib")]
            {
                let fc = fib_config.ok_or_else(|| {
                    crate::error::PolyKvError::InvalidPolicy("fib codec requires fib_config".into())
                })?;
                let adapter = FibQuantAdapter::new(
                    dim,
                    fc.k,
                    fc.n,
                    fc.training_samples,
                    fc.lloyd_restarts,
                    fc.lloyd_iterations,
                )?;
                Ok(Box::new(adapter))
            }
            #[cfg(not(feature = "fib"))]
            {
                Err(crate::error::PolyKvError::CodecUnavailable {
                    codec: crate::policy::CODEC_FIB_K4_N32.into(),
                    feature: "fib".into(),
                })
            }
        }
        crate::policy::CODEC_TURBO_8BIT => {
            #[cfg(feature = "turbo")]
            {
                let tc = turbo_config.ok_or_else(|| {
                    crate::error::PolyKvError::InvalidPolicy(
                        "turbo codec requires turbo_config".into(),
                    )
                })?;
                let adapter = TurboQuantAdapter::new(dim, tc.bits, tc.projections)?;
                Ok(Box::new(adapter))
            }
            #[cfg(not(feature = "turbo"))]
            {
                Err(crate::error::PolyKvError::CodecUnavailable {
                    codec: crate::policy::CODEC_TURBO_8BIT.into(),
                    feature: "turbo".into(),
                })
            }
        }
        crate::policy::CODEC_EXACT_FALLBACK => Ok(Box::new(ExactFallbackCodec::new(dim))),
        other => Err(crate::error::PolyKvError::InvalidPolicy(format!(
            "unknown codec id: {}",
            other
        ))),
    }
}
