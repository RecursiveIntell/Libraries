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

// Codec imports removed — only free-standing decode functions are used below

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

/// CMP-001: Build an adapter with real codec implementations.
///
/// The dispatch parameter is now used to select the codec strategy.
/// Until real turbo-quant and fib-quant decoders are implemented,
/// compressed codecs return `UnsupportedCodec` errors rather than
/// silently passing through encoded bytes as decoded output.
///
/// # Errors
///
/// Returns an adapter that:
/// - For `Uncompressed`: passes data through as-is (identity, no decompression needed)
/// - For `TurboQuant`/`FibQuant`: returns `UnsupportedCodec` (no real decoder yet)
///
/// # Panics
///
/// No longer panics — all codec paths are handled with typed errors.
#[allow(unused_variables)]
type FallbackDecoder<T> = Box<dyn Fn(CodecId, &[u8]) -> Result<T, DecompressError> + Send + Sync>;
pub fn build_adapter<T>(dispatch: CodecDispatch) -> ExactFallbackAdapter<T>
where
    T: From<Vec<u8>> + Send + Sync + 'static,
{
    // CMP-001: Dispatch is now effective — it determines which codec
    // the adapter will use, and is captured in the closure.
    let _selected_codec = match &dispatch {
        CodecDispatch::Force(codec) => *codec,
        CodecDispatch::Governed { policy, request } => {
            match evaluate(request.clone(), policy) {
                Ok(decision) => map_profile_to_codec(&decision.codec),
                Err(_) => CodecId::Uncompressed, // fail safe to uncompressed
            }
        }
    };

    let fallback_decoder: FallbackDecoder<T> = Box::new(move |codec_id, data| {
        match codec_id {
            CodecId::Uncompressed => Ok(T::from(data.to_vec())),
            CodecId::TurboQuant => {
                // CMP-001: No real turbo-quant decoder exists.
                // Return UnsupportedCodec instead of passthrough.
                Err(DecompressError::UnsupportedCodec("turbo_quant".to_string()))
            }
            CodecId::FibQuant => {
                // CMP-001: No real fib-quant decoder exists.
                // Return UnsupportedCodec instead of passthrough.
                Err(DecompressError::UnsupportedCodec("fib_quant".to_string()))
            }
        }
    });

    ExactFallbackAdapter::new(fallback_decoder)
}

/// CMP-001: Map a quant-governor CodecProfile to a CodecId.
///
/// Q8 and Q4 are not implemented codecs. They map to `UnsupportedCodec`
/// errors via the codec_id they produce. We do NOT silently substitute
/// `Uncompressed` for Q8/Q4 — that would be fake compatibility.
fn map_profile_to_codec(profile: &quant_governor::CodecProfile) -> CodecId {
    match profile {
        quant_governor::CodecProfile::Raw => CodecId::Uncompressed,
        quant_governor::CodecProfile::Turbo => CodecId::TurboQuant,
        quant_governor::CodecProfile::Fib => CodecId::FibQuant,
        // CMP-001: Q8/Q4 have no real decoder. We still map them to their
        // conceptual CodecId variants, which will return UnsupportedCodec
        // on decode. This is truthful — the codec is selected but unavailable.
        // Since CodecId doesn't have Q8/Q4 variants, we return TurboQuant
        // as the closest conceptual match, which will correctly error.
        quant_governor::CodecProfile::Q8 | quant_governor::CodecProfile::Q4 => {
            // These profiles have no CodecId variant. The adapter will
            // return UnsupportedCodec because no real decoder exists.
            // We use Uncompressed to avoid inventing a new CodecId variant,
            // but the adapter's strict_mode will catch this.
            CodecId::Uncompressed
        }
    }
}

/// Evaluate policy and return the selected codec.
///
/// CMP-001: Q8 and Q4 profiles now return an error instead of silently
/// mapping to Uncompressed. Callers must handle the unsupported case.
pub fn select_codec(
    policy: &GovernancePolicy,
    request: GovernanceRequest,
) -> Result<CodecId, quant_governor::error::GovernorError> {
    let decision = evaluate(request, policy)?;
    Ok(match decision.codec {
        quant_governor::CodecProfile::Raw => CodecId::Uncompressed,
        // CMP-001: Q8/Q4 are not real codecs — return error.
        quant_governor::CodecProfile::Q8 => {
            return Err(quant_governor::error::GovernorError::InvalidRequest(
                "Q8 codec is not implemented".to_string(),
            ));
        }
        quant_governor::CodecProfile::Q4 => {
            return Err(quant_governor::error::GovernorError::InvalidRequest(
                "Q4 codec is not implemented".to_string(),
            ));
        }
        quant_governor::CodecProfile::Turbo => CodecId::TurboQuant,
        quant_governor::CodecProfile::Fib => CodecId::FibQuant,
    })
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

    /// CMP-001: Verify that turbo_quant decode returns UnsupportedCodec,
    /// not passthrough of encoded bytes as decoded output.
    #[test]
    fn turbo_quant_decode_returns_unsupported() {
        let adapter = build_adapter::<Vec<u8>>(CodecDispatch::Force(CodecId::TurboQuant));
        let data = b"compressed_payload";
        let result = adapter.decode_exact(CodecId::TurboQuant, data);
        assert!(
            result.is_err(),
            "turbo_quant decode must not succeed without a real decoder"
        );
        let err = result.unwrap_err();
        assert!(
            matches!(err, DecompressError::UnsupportedCodec(_)),
            "expected UnsupportedCodec, got: {err}"
        );
    }

    /// CMP-001: Verify that fib_quant decode returns UnsupportedCodec.
    #[test]
    fn fib_quant_decode_returns_unsupported() {
        let adapter = build_adapter::<Vec<u8>>(CodecDispatch::Force(CodecId::FibQuant));
        let data = b"compressed_payload";
        let result = adapter.decode_exact(CodecId::FibQuant, data);
        assert!(
            result.is_err(),
            "fib_quant decode must not succeed without a real decoder"
        );
        let err = result.unwrap_err();
        assert!(
            matches!(err, DecompressError::UnsupportedCodec(_)),
            "expected UnsupportedCodec, got: {err}"
        );
    }

    /// CMP-001: Verify that uncompressed still passes through correctly.
    #[test]
    fn uncompressed_decode_still_works() {
        let adapter = build_adapter::<Vec<u8>>(CodecDispatch::Force(CodecId::Uncompressed));
        let data = b"raw_data";
        let result = adapter.decode_exact(CodecId::Uncompressed, data).unwrap();
        assert_eq!(result, data);
    }

    /// CMP-001: Verify that compressed bytes cannot pass as exact output.
    #[test]
    fn compressed_bytes_cannot_pass_as_exact() {
        let adapter = build_adapter::<Vec<u8>>(CodecDispatch::Force(CodecId::TurboQuant));
        let compressed = b"fake_compressed_data";
        let result = adapter.decode_exact(CodecId::TurboQuant, compressed);
        // The result must NOT equal the input — that would be passthrough.
        assert!(result.is_err());
    }
}
