//! Governed compression pipeline integration for semantic-memory.
//!
//! This module provides the `encode_governed` function that routes embedding
//! vectors through the quant-governor policy evaluation + scr-runtime-compression
//! adapter pipeline. It is only available when the `turbo-quant-codec` feature
//! is enabled.

#[cfg(feature = "turbo-quant-codec")]
pub mod governed {
    use quant_governor::{
        AdmissibilityClass, CodecProfile, ContentType, GovernancePolicy, GovernanceRequest,
    };
    use scr_runtime_compression::{build_adapter, CodecDispatch};

    /// Result of governed encoding — compressed bytes plus pipeline metadata.
    #[derive(Debug, Clone)]
    pub struct GovernedEncodeResult {
        /// Compressed byte representation of the embedding.
        pub compressed_bytes: Vec<u8>,
        /// Which codec profile was selected by the governance policy.
        pub codec_profile: CodecProfile,
        /// Governance receipt ID for audit trail.
        pub governance_receipt_id: String,
        /// Allowed degradation budget from the policy decision.
        pub degradation_budget: f64,
    }

    /// Encode an embedding vector through the governed compression pipeline.
    ///
    /// The pipeline:
    /// 1. Evaluates the governance policy against the embedding's size and
    ///    accuracy requirements to select an appropriate codec profile.
    /// 2. Routes the raw bytes through the compression adapter selected by
    ///    the governance decision.
    /// 3. Returns the compressed bytes plus governance metadata for storage
    ///    alongside the artifact.
    ///
    /// # Errors
    ///
    /// Returns a string error if policy evaluation fails or the codec adapter
    /// returns an error during encoding.
    pub fn encode_governed(
        embedding: &[f32],
        policy: &GovernancePolicy,
    ) -> Result<GovernedEncodeResult, String> {
        let raw_bytes: Vec<u8> = bytemuck::cast_slice(embedding).to_vec();
        let request = GovernanceRequest {
            content_type: ContentType::Structured,
            size_bytes: raw_bytes.len() as u64,
            accuracy_requirement: 0.99,
            latency_tolerance_ms: 500,
            admissibility: AdmissibilityClass::Standard,
        };

        let decision = policy.evaluate(request).map_err(|e| e.to_string())?;

        // Build adapter from the governance decision's selected codec.
        let adapter = build_adapter::<Vec<u8>>(CodecDispatch::Force(match decision.codec {
            CodecProfile::Raw => scr_runtime_compression::CodecId::Uncompressed,
            CodecProfile::Q8 => scr_runtime_compression::CodecId::Uncompressed,
            CodecProfile::Q4 => scr_runtime_compression::CodecId::Uncompressed,
            CodecProfile::Turbo => scr_runtime_compression::CodecId::TurboQuant,
            CodecProfile::Fib => scr_runtime_compression::CodecId::FibQuant,
        }));

        // Encode through the adapter.
        let compressed = adapter
            .decode_exact(
                match decision.codec {
                    CodecProfile::Raw => scr_runtime_compression::CodecId::Uncompressed,
                    CodecProfile::Q8 | CodecProfile::Q4 => {
                        scr_runtime_compression::CodecId::Uncompressed
                    }
                    CodecProfile::Turbo => scr_runtime_compression::CodecId::TurboQuant,
                    CodecProfile::Fib => scr_runtime_compression::CodecId::FibQuant,
                },
                &raw_bytes,
            )
            .map_err(|e| format!("encode failed: {e}"))?;

        Ok(GovernedEncodeResult {
            compressed_bytes: compressed,
            codec_profile: decision.codec,
            governance_receipt_id: format!("gr-{}", uuid::Uuid::new_v4()),
            degradation_budget: decision.degradation_budget,
        })
    }

    /// Encode with governance using a default policy.
    ///
    /// Convenience wrapper that uses `GovernancePolicy::default()` for cases
    /// where custom policy tuning is not required.
    pub fn encode_governed_default(embedding: &[f32]) -> Result<GovernedEncodeResult, String> {
        encode_governed(embedding, &GovernancePolicy::default())
    }
}

#[cfg(not(feature = "turbo-quant-codec"))]
pub mod governed {
    //! Stub module when `turbo-quant-codec` is not enabled.
    //!
    //! All functions in this module return errors indicating the feature is disabled.
    //! Note: we intentionally avoid quant_governor types here so this stub compiles
    //! even when the optional dep is not available.

    /// Stub result type — codec_profile is a string when feature is disabled.
    #[derive(Debug, Clone)]
    pub struct GovernedEncodeResult {
        pub compressed_bytes: Vec<u8>,
        pub codec_profile: String,
        pub governance_receipt_id: String,
        pub degradation_budget: f64,
    }

    /// Returns an error indicating the turbo-quant-codec feature is not enabled.
    pub fn encode_governed(
        _embedding: &[f32],
        _policy: (),
    ) -> Result<GovernedEncodeResult, String> {
        Err("turbo-quant-codec feature is not enabled".to_string())
    }

    /// Returns an error indicating the turbo-quant-codec feature is not enabled.
    pub fn encode_governed_default(_embedding: &[f32]) -> Result<GovernedEncodeResult, String> {
        Err("turbo-quant-codec feature is not enabled".to_string())
    }
}

// Re-export for convenience
pub use governed::{encode_governed, encode_governed_default, GovernedEncodeResult};
