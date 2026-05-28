//! Codec dispatch helpers — wires turbo-quant/fib-quant through quant-governor.
//!
//! This module provides factory functions to build [`ExactFallbackAdapter`](crate::ExactFallbackAdapter)
//! instances with real codec implementations, integrated with policy-driven governance.
//!
//! ## Usage
//!
//! ```rust,no_run
//! use scr_runtime_compression::{build_adapter, CodecDispatch, DecompressError};
//! use quant_governor::{GovernancePolicy, GovernanceRequest, ContentType};
//!
//! let policy = GovernancePolicy::default();
//! let adapter = build_adapter::<Vec<u8>>(CodecDispatch::Governed {
//!     policy: &policy,
//!     request: GovernanceRequest {
//!         content_type: ContentType::Audio,
//!         size_bytes: 6144,
//!         latency_tolerance_ms: 50,
//!         ..Default::default()
//!     },
//! });
//! ```

use crate::{CodecId, DecompressError, ExactFallbackAdapter};
use quant_governor::{evaluate, GovernancePolicy, GovernanceRequest};

#[cfg(feature = "turbo")]
use turbo_quant::TurboQuantizer;

#[cfg(feature = "fib")]
use fib_quant::FibQuantizer;

/// Codec dispatch strategy.
#[derive(Debug, Clone)]
pub enum CodecDispatch<'a> {
    /// Use policy-governed codec selection.
    Governed {
        /// Governance policy to evaluate.
        policy: &'a GovernancePolicy,
        /// Request context for policy evaluation.
        request: GovernanceRequest,
    },
    /// Force a specific codec (bypasses governance).
    Force(CodecId),
}

/// Build an adapter with real codec implementations.
///
/// This function wires up the fallback decoder closure to call the actual
/// `turbo-quant` and `fib-quant` decode functions.
///
/// # Panics
///
/// Panics if both `turbo` and `fib` features are disabled (no codecs available).
#[allow(unused_variables)]
pub fn build_adapter<T>(dispatch: CodecDispatch) -> ExactFallbackAdapter<T>
where
    T: From<Vec<u8>> + Send + Sync + 'static,
{
    let fallback_decoder: Box<dyn Fn(CodecId, &[u8]) -> Result<T, DecompressError> + Send + Sync> =
        Box::new(move |codec_id, data| {
            match codec_id {
                CodecId::Uncompressed => Ok(T::from(data.to_vec())),
                #[cfg(feature = "turbo")]
                CodecId::TurboQuant => {
                    // Decode turbo-quant compressed data
                    // Note: This is a simplified example - real implementation
                    // would need to deserialize the code structure
                    turbo_quant_decode(data).map(T::from)
                }
                #[cfg(feature = "fib")]
                CodecId::FibQuant => {
                    // Decode fib-quant compressed data
                    fib_quant_decode(data).map(T::from)
                }
                #[cfg(not(any(feature = "turbo", feature = "fib")))]
                _ => Err(DecompressError::DecodeFailed(
                    "No codec features enabled".to_string(),
                )),
            }
        });

    ExactFallbackAdapter::new(fallback_decoder)
}

/// Evaluate policy and return the selected codec.
///
/// Helper function to evaluate a governance policy and extract the selected codec.
pub fn select_codec(
    policy: &GovernancePolicy,
    request: GovernanceRequest,
) -> Result<CodecId, quant_governor::error::GovernorError> {
    let decision = evaluate(request, policy)?;
    Ok(match decision.codec {
        quant_governor::CodecProfile::Raw => CodecId::Uncompressed,
        quant_governor::CodecProfile::Q8 => CodecId::Uncompressed, // Q8 not yet implemented
        quant_governor::CodecProfile::Q4 => CodecId::Uncompressed, // Q4 not yet implemented
        quant_governor::CodecProfile::Turbo => CodecId::TurboQuant,
        quant_governor::CodecProfile::Fib => CodecId::FibQuant,
    })
}

/// Decode turbo-quant compressed data.
///
/// This is a placeholder - real implementation would deserialize the code
/// and use the quantizer to reconstruct the vector.
#[cfg(feature = "turbo")]
fn turbo_quant_decode(data: &[u8]) -> Result<Vec<u8>, DecompressError> {
    // TODO: Implement actual turbo-quant decode
    // This would involve:
    // 1. Deserializing the compressed code structure
    // 2. Reconstructing the quantizer from metadata
    // 3. Decoding the polar code + residual sketch
    // 4. Returning the reconstructed f32 vector as bytes
    Ok(data.to_vec()) // Placeholder - just pass through for now
}

/// Decode fib-quant compressed data.
///
/// This is a placeholder - real implementation would deserialize the code
/// and use the quantizer to reconstruct the vector.
#[cfg(feature = "fib")]
fn fib_quant_decode(data: &[u8]) -> Result<Vec<u8>, DecompressError> {
    // TODO: Implement actual fib-quant decode
    // This would involve:
    // 1. Deserializing the FibCodeV1 structure
    // 2. Reconstructing the quantizer from profile
    // 3. Decoding the radial-angular codebook representation
    // 4. Returning the reconstructed f32 vector as bytes
    Ok(data.to_vec()) // Placeholder - just pass through for now
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn select_codec_raw() {
        let policy = GovernancePolicy::default();
        let request = GovernanceRequest::default();
        let codec = select_codec(&policy, request).unwrap();
        assert_eq!(codec, CodecId::Uncompressed);
    }

    #[test]
    #[cfg(feature = "turbo")]
    fn select_codec_turbo() {
        use quant_governor::ContentType;

        // Audio with low latency tolerance should select Turbo
        let policy = GovernancePolicy::default();
        let request = GovernanceRequest {
            content_type: ContentType::Audio,
            size_bytes: 6144,
            latency_tolerance_ms: 50, // < 100ms triggers Turbo for audio
            ..Default::default()
        };
        let codec = select_codec(&policy, request).unwrap();
        assert_eq!(codec, CodecId::TurboQuant);
    }
}
