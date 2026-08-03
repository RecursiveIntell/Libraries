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
//! })
//! .expect("governance must select an implemented runtime codec");
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

/// Build an exact-fallback adapter for one selected codec.
///
/// The selected dispatch is enforced at decode time: a caller cannot ask a
/// Turbo/Fib-selected adapter to reinterpret bytes as raw uncompressed data.
/// No TurboQuant or FibQuant decoder is currently registered, so strict mode
/// rejects those selections and non-strict callers receive `UnsupportedCodec`.
///
/// # Errors
///
/// Returns a typed error when governed selection fails or selects Q8/Q4, which
/// have no `CodecId` or real decoder. It never substitutes `Uncompressed`.
type FallbackDecoder<T> = Box<dyn Fn(CodecId, &[u8]) -> Result<T, DecompressError> + Send + Sync>;
pub fn build_adapter<T>(dispatch: CodecDispatch) -> Result<ExactFallbackAdapter<T>, DecompressError>
where
    T: From<Vec<u8>> + Send + Sync + 'static,
{
    let selected_codec = match dispatch {
        CodecDispatch::Force(codec) => codec,
        CodecDispatch::Governed { policy, request } => {
            let decision = evaluate(request, policy).map_err(|error| {
                DecompressError::DecodeFailed(format!("governance selection failed: {error}"))
            })?;
            map_profile_to_codec(&decision.codec)?
        }
    };

    let fallback_decoder: FallbackDecoder<T> = Box::new(move |codec_id, data| {
        if codec_id != selected_codec {
            return Err(DecompressError::CodecNotAvailable(format!(
                "dispatch selected `{selected_codec}`, requested `{codec_id}`"
            )));
        }
        match selected_codec {
            CodecId::Uncompressed => Ok(T::from(data.to_vec())),
            CodecId::TurboQuant => {
                Err(DecompressError::UnsupportedCodec("turbo_quant".to_string()))
            }
            CodecId::FibQuant => Err(DecompressError::UnsupportedCodec("fib_quant".to_string())),
        }
    });

    Ok(ExactFallbackAdapter::new(fallback_decoder))
}

/// Register a real FibQuant decoder bound to one profile/codebook instance.
///
/// CMP-003: FibQuant decoding requires the codebook that produced the code
/// (the payload carries digests, not the codebook), so registration is
/// stateful: the caller supplies the [`fib_quant::FibQuantizer`]. The
/// registered decoder validates the payload (serde parse, then the codec's
/// own `validate_code_header`) and converts the decoded f32 vector to
/// little-endian value bytes. It never substitutes raw bytes and never
/// accepts a different `CodecId`.
#[cfg(feature = "fib")]
pub fn build_fib_adapter<T>(codec: fib_quant::FibQuantizer) -> ExactFallbackAdapter<T>
where
    T: From<Vec<u8>> + Send + Sync + 'static,
{
    let decoder: FallbackDecoder<T> = Box::new(move |codec_id, data| {
        if codec_id != CodecId::FibQuant {
            return Err(DecompressError::CodecNotAvailable(format!(
                "fib adapter only decodes `fib_quant`, requested `{codec_id}`"
            )));
        }
        let code: fib_quant::FibCodeV1 = serde_json::from_slice(data).map_err(|error| {
            DecompressError::DecodeFailed(format!("fib payload parse failed: {error}"))
        })?;
        let floats = codec.decode(&code).map_err(|error| {
            DecompressError::DecodeFailed(format!("fib decode failed: {error}"))
        })?;
        let bytes: Vec<u8> = floats
            .into_iter()
            .flat_map(|value| value.to_le_bytes())
            .collect();
        Ok(T::from(bytes))
    });
    ExactFallbackAdapter::new(decoder)
}

/// Map a governance profile into a runtime codec without substitution.
fn map_profile_to_codec(
    profile: &quant_governor::CodecProfile,
) -> Result<CodecId, DecompressError> {
    match profile {
        quant_governor::CodecProfile::Raw => Ok(CodecId::Uncompressed),
        quant_governor::CodecProfile::Turbo => Ok(CodecId::TurboQuant),
        quant_governor::CodecProfile::Fib => Ok(CodecId::FibQuant),
        quant_governor::CodecProfile::Q8 => {
            Err(DecompressError::UnsupportedCodec("q8".to_string()))
        }
        quant_governor::CodecProfile::Q4 => {
            Err(DecompressError::UnsupportedCodec("q4".to_string()))
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

    fn forced_adapter(codec: CodecId) -> ExactFallbackAdapter<Vec<u8>> {
        match build_adapter::<Vec<u8>>(CodecDispatch::Force(codec)) {
            Ok(adapter) => adapter,
            Err(error) => panic!("forced codec selection should construct an adapter: {error}"),
        }
    }

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
        use quant_governor::{CodecProfile, ContentType};

        // Audio with low latency tolerance should select Turbo — admitted
        // explicitly (CMP-001: default policy admits only Raw).
        let policy = GovernancePolicy::default().with_admitted_codecs([
            CodecProfile::Raw,
            CodecProfile::Q8,
            CodecProfile::Turbo,
        ]);
        let request = GovernanceRequest {
            content_type: ContentType::Audio,
            size_bytes: 6144,
            latency_tolerance_ms: 50, // < 100ms triggers Turbo for audio
            ..Default::default()
        };
        let codec = select_codec(&policy, request).unwrap();
        assert_eq!(codec, CodecId::TurboQuant);
    }

    /// CMP-001: Verify that TurboQuant is rejected in strict mode when no
    /// real decoder has been registered; encoded bytes never pass through.
    #[test]
    fn turbo_quant_decode_is_rejected_without_decoder() {
        let adapter = forced_adapter(CodecId::TurboQuant);
        let data = b"compressed_payload";
        let result = adapter.decode_exact(CodecId::TurboQuant, data);
        assert!(
            result.is_err(),
            "turbo_quant decode must not succeed without a real decoder"
        );
        let err = result.unwrap_err();
        assert!(
            matches!(err, DecompressError::StrictModeRejected(_)),
            "expected StrictModeRejected, got: {err}"
        );
    }

    /// CMP-001: Verify that FibQuant is rejected in strict mode without a real decoder.
    #[test]
    fn fib_quant_decode_returns_unsupported() {
        let adapter = forced_adapter(CodecId::FibQuant);
        let data = b"compressed_payload";
        let result = adapter.decode_exact(CodecId::FibQuant, data);
        assert!(
            result.is_err(),
            "fib_quant decode must not succeed without a real decoder"
        );
        let err = result.unwrap_err();
        assert!(
            matches!(err, DecompressError::StrictModeRejected(_)),
            "expected StrictModeRejected, got: {err}"
        );
    }

    /// CMP-001: Verify that uncompressed still passes through correctly.
    #[test]
    fn uncompressed_decode_still_works() {
        let adapter = forced_adapter(CodecId::Uncompressed);
        let data = b"raw_data";
        let result = adapter.decode_exact(CodecId::Uncompressed, data).unwrap();
        assert_eq!(result, data);
    }

    /// CMP-001: Verify that compressed bytes cannot pass as exact output.
    #[test]
    fn compressed_bytes_cannot_pass_as_exact() {
        let adapter = forced_adapter(CodecId::TurboQuant);
        let compressed = b"fake_compressed_data";
        let result = adapter.decode_exact(CodecId::TurboQuant, compressed);
        // The result must NOT equal the input — that would be passthrough.
        assert!(result.is_err());
    }

    /// CMP-003: a REAL FibQuant codec round-trips through a registered
    /// decoder — actual decoding, not passthrough and not a typed rejection.
    #[cfg(feature = "fib")]
    #[test]
    fn fib_registered_decoder_roundtrips_real_code() {
        use fib_quant::{FibCodebookV1, FibQuantProfileV1, FibQuantizer};

        let mut profile = FibQuantProfileV1::paper_default(8, 2, 8, 31).unwrap();
        profile.training_samples = 128;
        profile.lloyd_restarts = 2;
        profile.lloyd_iterations = 3;
        let codebook = FibCodebookV1::build(profile.clone()).unwrap();
        let quantizer = FibQuantizer::from_codebook(codebook).unwrap();

        let vector: Vec<f32> = (0..8).map(|i| (i as f32 + 1.0) / 8.0).collect();
        let code = quantizer.encode(&vector).unwrap();
        let payload = serde_json::to_vec(&code).unwrap();

        let adapter =
            build_fib_adapter::<Vec<u8>>(quantizer).with_supported_codecs([CodecId::FibQuant]);
        let decoded = adapter.decode_exact(CodecId::FibQuant, &payload).unwrap();
        assert_eq!(decoded.len(), 8 * 4, "decoded f32 value bytes");

        let floats: Vec<f32> = decoded
            .chunks_exact(4)
            .map(|c| f32::from_le_bytes([c[0], c[1], c[2], c[3]]))
            .collect();
        assert_eq!(floats.len(), 8);
        // Reconstruction is quantized, not bit-exact — verify it is close and
        // finite (never raw passthrough of the payload).
        let mse: f64 = floats
            .iter()
            .zip(&vector)
            .map(|(a, b)| f64::from(*a - *b).powi(2))
            .sum::<f64>()
            / vector.len() as f64;
        assert!(
            mse < 1.0,
            "quantized reconstruction should be bounded, mse={mse}"
        );
        assert_ne!(decoded, payload, "decoded bytes must not be the payload");
    }

    /// CMP-003: garbage payloads fail typed — never a silent default.
    #[cfg(feature = "fib")]
    #[test]
    fn fib_registered_decoder_rejects_garbage_payload_typed() {
        use fib_quant::{FibCodebookV1, FibQuantProfileV1, FibQuantizer};

        let mut profile = FibQuantProfileV1::paper_default(8, 2, 8, 31).unwrap();
        profile.training_samples = 64;
        profile.lloyd_restarts = 1;
        profile.lloyd_iterations = 2;
        let codebook = FibCodebookV1::build(profile).unwrap();
        let quantizer = FibQuantizer::from_codebook(codebook).unwrap();
        let adapter =
            build_fib_adapter::<Vec<u8>>(quantizer).with_supported_codecs([CodecId::FibQuant]);

        let err = adapter
            .decode_exact(CodecId::FibQuant, b"not-json")
            .unwrap_err();
        assert!(
            matches!(err, DecompressError::DecodeFailed(_)),
            "expected typed DecodeFailed, got {err:?}"
        );
    }

    /// CMP-003: a registered fib adapter refuses other codec ids.
    #[cfg(feature = "fib")]
    #[test]
    fn fib_registered_decoder_rejects_other_codec_id() {
        use fib_quant::{FibCodebookV1, FibQuantProfileV1, FibQuantizer};

        let mut profile = FibQuantProfileV1::paper_default(8, 2, 8, 31).unwrap();
        profile.training_samples = 64;
        profile.lloyd_restarts = 1;
        profile.lloyd_iterations = 2;
        let codebook = FibCodebookV1::build(profile).unwrap();
        let quantizer = FibQuantizer::from_codebook(codebook).unwrap();
        let adapter =
            build_fib_adapter::<Vec<u8>>(quantizer).with_supported_codecs([CodecId::FibQuant]);

        let err = adapter
            .decode_exact(CodecId::TurboQuant, b"whatever")
            .unwrap_err();
        // The strict-mode gate rejects a codec the adapter was not registered
        // for — a typed rejection before any decode attempt.
        assert!(
            matches!(
                err,
                DecompressError::StrictModeRejected(_) | DecompressError::CodecNotAvailable(_)
            ),
            "expected typed rejection, got {err:?}"
        );
    }
}
