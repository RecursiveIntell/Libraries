use half::f16;
use serde::{Deserialize, Serialize};

use crate::{
    bitpack::{pack_indices, unpack_indices},
    codebook::FibCodebookV1,
    digest::{bytes_digest, json_digest},
    lloyd::nearest_index,
    metrics,
    profile::{FibQuantProfileV1, NormFormat},
    receipt::FibQuantCompressionReceiptV1,
    rotation::StoredRotation,
    FibQuantError, Result,
};

pub const CODE_SCHEMA: &str = "fib_code_v1";

/// Encoded fixed-rate FibQuant artifact.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FibCodeV1 {
    /// Stable schema marker.
    pub schema_version: String,
    /// Profile digest.
    pub profile_digest: String,
    /// Codebook digest.
    pub codebook_digest: String,
    /// Rotation digest.
    pub rotation_digest: String,
    /// Ambient dimension.
    pub ambient_dim: u32,
    /// Block dimension.
    pub block_dim: u32,
    /// Norm payload format.
    pub norm_format: NormFormat,
    /// Norm bytes.
    pub norm_payload: Vec<u8>,
    /// Bits per fixed-rate index.
    pub wire_index_bits: u8,
    /// Number of indices.
    pub block_count: u32,
    /// Packed fixed-rate indices.
    pub indices: Vec<u8>,
}

/// FibQuant encoder/decoder bound to one profile and codebook.
#[derive(Debug, Clone)]
pub struct FibQuantizer {
    profile: FibQuantProfileV1,
    codebook: FibCodebookV1,
    rotation: StoredRotation,
}

impl FibQuantizer {
    /// Build a quantizer by constructing the profile codebook.
    pub fn new(profile: FibQuantProfileV1) -> Result<Self> {
        let codebook = FibCodebookV1::build(profile)?;
        Self::from_codebook(codebook)
    }

    /// Build a quantizer from a validated codebook.
    pub fn from_codebook(codebook: FibCodebookV1) -> Result<Self> {
        codebook.validate()?;
        let profile = codebook.profile.clone();
        let rotation = StoredRotation::new(profile.ambient_dim as usize, profile.rotation_seed)?;
        Ok(Self {
            profile,
            codebook,
            rotation,
        })
    }

    /// Access the profile.
    pub fn profile(&self) -> &FibQuantProfileV1 {
        &self.profile
    }

    /// Access the codebook.
    pub fn codebook(&self) -> &FibCodebookV1 {
        &self.codebook
    }

    /// Encode a vector into a fixed-rate artifact.
    pub fn encode(&self, x: &[f32]) -> Result<FibCodeV1> {
        let d = self.profile.ambient_dim as usize;
        let k = self.profile.block_dim as usize;
        if x.len() != d {
            return Err(FibQuantError::CorruptPayload(format!(
                "input dimension {}, expected {d}",
                x.len()
            )));
        }
        check_finite(x)?;
        let norm = l2_norm(x);
        if norm == 0.0 {
            return Err(FibQuantError::ZeroNorm);
        }
        let normalized: Vec<f64> = x.iter().map(|value| f64::from(*value) / norm).collect();
        let rotated = self.rotation.apply(&normalized)?;
        let codewords_f64: Vec<f64> = self
            .codebook
            .codewords
            .iter()
            .map(|value| f64::from(*value))
            .collect();
        let block_count = self.profile.block_count() as usize;
        let mut indices = Vec::with_capacity(block_count);
        for block in rotated.chunks_exact(k) {
            indices.push(nearest_index(block, &codewords_f64, k).0 as u32);
        }
        Ok(FibCodeV1 {
            schema_version: CODE_SCHEMA.into(),
            profile_digest: self.profile.digest()?,
            codebook_digest: self.codebook.codebook_digest.clone(),
            rotation_digest: self.rotation.digest()?,
            ambient_dim: self.profile.ambient_dim,
            block_dim: self.profile.block_dim,
            norm_format: self.profile.norm_format.clone(),
            norm_payload: encode_norm(norm, &self.profile.norm_format)?,
            wire_index_bits: self.profile.wire_index_bits,
            block_count: self.profile.block_count(),
            indices: pack_indices(&indices, self.profile.wire_index_bits)?,
        })
    }

    /// Decode a fixed-rate artifact.
    pub fn decode(&self, code: &FibCodeV1) -> Result<Vec<f32>> {
        self.validate_code_header(code)?;
        let k = self.profile.block_dim as usize;
        let block_count = self.profile.block_count() as usize;
        let unpacked = unpack_indices(&code.indices, block_count, self.profile.wire_index_bits)?;
        let mut rotated = Vec::with_capacity(self.profile.ambient_dim as usize);
        for index in unpacked {
            if index >= self.profile.codebook_size {
                return Err(FibQuantError::IndexOutOfRange {
                    index,
                    codebook_size: self.profile.codebook_size,
                });
            }
            rotated.extend(self.codebook.codeword(index as usize)?);
        }
        let expected_rotated_len = block_count.checked_mul(k).ok_or_else(|| {
            FibQuantError::ResourceLimitExceeded("decoded rotated vector length overflow".into())
        })?;
        if rotated.len() != expected_rotated_len {
            return Err(FibQuantError::CorruptPayload(
                "decoded rotated vector length mismatch".into(),
            ));
        }
        let norm = decode_norm(&code.norm_payload, &code.norm_format)?;
        let reconstructed = self.rotation.apply_inverse(&rotated)?;
        let out: Vec<f32> = reconstructed
            .into_iter()
            .map(|value| (value * norm) as f32)
            .collect();
        check_finite(&out)?;
        Ok(out)
    }

    /// Encode and emit a receipt.
    pub fn encode_with_receipt(
        &self,
        x: &[f32],
    ) -> Result<(FibCodeV1, FibQuantCompressionReceiptV1)> {
        let code = self.encode(x)?;
        let source_vector_digest = source_vector_digest(x)?;
        let mut receipt = FibQuantCompressionReceiptV1::new(
            &self.profile,
            code.profile_digest.clone(),
            code.codebook_digest.clone(),
            code.rotation_digest.clone(),
            source_vector_digest,
            encoded_digest(&code)?,
        );
        let decoded = self.decode(&code)?;
        receipt.mse = Some(metrics::mse(x, &decoded)?);
        receipt.cosine_similarity = Some(metrics::cosine_similarity(x, &decoded)?);
        Ok((code, receipt))
    }

    /// Reconstruction MSE for one vector.
    pub fn reconstruction_mse(&self, x: &[f32]) -> Result<f64> {
        let code = self.encode(x)?;
        let decoded = self.decode(&code)?;
        metrics::mse(x, &decoded)
    }

    /// Reconstruction cosine similarity for one vector.
    pub fn cosine_similarity(&self, x: &[f32]) -> Result<f64> {
        let code = self.encode(x)?;
        let decoded = self.decode(&code)?;
        metrics::cosine_similarity(x, &decoded)
    }

    // ── Batch encode/decode ──

    /// Encode a batch of vectors. Uses gpu-backend for the Hadamard + Lloyd-Max
    /// portions when available, keeping the FibCodeV1 format identical to single encode.
    pub fn encode_batch(&self, vectors: &[&[f32]]) -> Result<Vec<FibCodeV1>> {
        let d = self.profile.ambient_dim as usize;
        let k = self.profile.block_dim as usize;
        let n = vectors.len();
        if n == 0 {
            return Ok(vec![]);
        }

        // Fall back to single encode for small batches
        if n < 4 {
            return vectors.iter().map(|v| self.encode(v)).collect();
        }

        // Flatten input
        let mut flat = Vec::with_capacity(n * d);
        let mut norms_f64 = Vec::with_capacity(n);
        for v in vectors {
            if v.len() != d {
                return Err(FibQuantError::CorruptPayload(format!(
                    "input dimension {}, expected {d}",
                    v.len()
                )));
            }
            check_finite(v)?;
            let norm = l2_norm(v);
            if norm == 0.0 {
                return Err(FibQuantError::ZeroNorm);
            }
            norms_f64.push(norm);
            for &x in *v {
                flat.push((x as f64 / norm) as f32);
            }
        }

        // Apply Hadamard batch rotation (uses gpu-backend when available)
        #[cfg(feature = "gpu")]
        {
            if let Some(_ctx) = gpu_backend::GpuContext::init() {
                if n >= gpu_backend::GpuContext::GPU_MIN_BATCH_SIZE
                    && d >= gpu_backend::GpuContext::GPU_MIN_DIM
                {
                    gpu_backend::hadamard_batch(&mut flat, n, d, self.profile.rotation_seed)
                        .map_err(|e| {
                            FibQuantError::NumericalFailure(format!("gpu hadamard: {}", e))
                        })?;

                    // GPU codebook lookup: the dominant cost in encode_batch
                    // for k=4, N=32. Falls back to CPU if N > 32 or other
                    // gates fail; the indices produced are byte-identical to
                    // the CPU path (verified by gpu-backend parity test).
                    //
                    // The `gpu_codebook_lookup` cfg switches this on. When
                    // off, the rotated data goes back to the CPU for the
                    // codebook loop. The current dispatch path through
                    // gpu_backend pays H2D + D2H per call, which can be
                    // slower than a tight CPU loop for small batches.
                    #[cfg(feature = "gpu_codebook_lookup")]
                    {
                        let block_count = self.profile.block_count() as usize;
                        if let Ok(indices) = gpu_backend::codebook_lookup_batch(
                            &flat,
                            &self.codebook.codewords,
                            n,
                            d,
                            k,
                        ) {
                            if indices.len() == n * block_count {
                                return self.finish_batch_encode_with_indices(
                                    &flat, &norms_f64, &indices, n, d, k,
                                );
                            }
                            // Length mismatch — fall through to CPU for safety.
                        }
                    }

                    // CPU fallback for the codebook lookup (Hadamard already on GPU).
                    return self.finish_batch_encode(&flat, &norms_f64, n, d, k);
                }
            }
        }

        // CPU fallback: use StoredRotation on each vector
        let mut rotated_flat = Vec::with_capacity(n * d);
        for chunk in flat.chunks_exact(d) {
            let f64_chunk: Vec<f64> = chunk.iter().map(|&v| v as f64).collect();
            let rot = self.rotation.apply(&f64_chunk)?;
            rotated_flat.extend(rot.iter().map(|&v| v as f32));
        }

        self.finish_batch_encode(&rotated_flat, &norms_f64, n, d, k)
    }

    fn finish_batch_encode(
        &self,
        rotated: &[f32],
        norms: &[f64],
        n: usize,
        d: usize,
        k: usize,
    ) -> Result<Vec<FibCodeV1>> {
        let block_count = self.profile.block_count() as usize;
        // Use the SIMD-accelerated f32 argmin from gpu-backend. This
        // replaces the f64-promoted nearest_index loop with an AVX2+FMA
        // f32 implementation. On a trained Lloyd-Max codebook the argmins
        // are byte-identical to the f64 reference (parity verified in
        // gpu-backend).
        let codewords_f32: &[f32] = &self.codebook.codewords;

        let mut codes = Vec::with_capacity(n);
        for vec_idx in 0..n {
            let start = vec_idx * d;
            let chunk = &rotated[start..start + d];
            let mut indices = Vec::with_capacity(block_count);
            for block in chunk.chunks_exact(k) {
                indices.push(gpu_backend::nearest_codeword_f32(block, codewords_f32, k) as u32);
            }

            codes.push(FibCodeV1 {
                schema_version: CODE_SCHEMA.into(),
                profile_digest: self.profile.digest()?,
                codebook_digest: self.codebook.codebook_digest.clone(),
                rotation_digest: self.rotation.digest()?,
                ambient_dim: self.profile.ambient_dim,
                block_dim: self.profile.block_dim,
                norm_format: self.profile.norm_format.clone(),
                norm_payload: encode_norm(norms[vec_idx], &self.profile.norm_format)?,
                wire_index_bits: self.profile.wire_index_bits,
                block_count: self.profile.block_count(),
                indices: pack_indices(&indices, self.profile.wire_index_bits)?,
            });
        }

        Ok(codes)
    }

    /// Build `FibCodeV1` records from a pre-computed index array. Used by
    /// the GPU path after `codebook_lookup_batch` returns the per-block
    /// nearest-codeword indices. Length of `indices` must be `n * (d / k)`.
    #[cfg(all(feature = "gpu", feature = "gpu_codebook_lookup"))]
    fn finish_batch_encode_with_indices(
        &self,
        _rotated: &[f32], // not used; indices are already computed
        norms: &[f64],
        indices: &[u32],
        n: usize,
        _d: usize,
        _k: usize,
    ) -> Result<Vec<FibCodeV1>> {
        let block_count = self.profile.block_count() as usize;
        if indices.len() != n * block_count {
            return Err(FibQuantError::CorruptPayload(format!(
                "indices length {} != n * block_count {}",
                indices.len(),
                n * block_count
            )));
        }

        let mut codes = Vec::with_capacity(n);
        for vec_idx in 0..n {
            let start = vec_idx * block_count;
            let end = start + block_count;
            let vec_indices: Vec<u32> = indices[start..end].to_vec();

            codes.push(FibCodeV1 {
                schema_version: CODE_SCHEMA.into(),
                profile_digest: self.profile.digest()?,
                codebook_digest: self.codebook.codebook_digest.clone(),
                rotation_digest: self.rotation.digest()?,
                ambient_dim: self.profile.ambient_dim,
                block_dim: self.profile.block_dim,
                norm_format: self.profile.norm_format.clone(),
                norm_payload: encode_norm(norms[vec_idx], &self.profile.norm_format)?,
                wire_index_bits: self.profile.wire_index_bits,
                block_count: self.profile.block_count(),
                indices: pack_indices(&vec_indices, self.profile.wire_index_bits)?,
            });
        }
        Ok(codes)
    }

    /// Decode a batch of codes.
    pub fn decode_batch(&self, codes: &[FibCodeV1]) -> Result<Vec<Vec<f32>>> {
        codes.iter().map(|c| self.decode(c)).collect()
    }

    /// Check if GPU acceleration is available.
    ///
    /// This is a **device-availability** probe: it returns true if a CUDA
    /// device was found at init time. Whether an *individual* encode_batch
    /// call actually dispatches to GPU depends on the call's batch size and
    /// vector dimension crossing the runtime thresholds.
    ///
    /// Use [`Self::is_gpu_accelerated_for`] for an honest per-call check.
    pub fn is_gpu_accelerated(&self) -> bool {
        #[cfg(feature = "gpu")]
        {
            gpu_backend::GpuContext::is_available()
        }
        #[cfg(not(feature = "gpu"))]
        {
            false
        }
    }

    /// Check if a batch of `n` vectors of dimension `d` would actually
    /// dispatch to GPU. Returns true only when:
    ///   - the `gpu` feature is compiled in,
    ///   - a CUDA device is available at runtime,
    ///   - `n >= GPU_MIN_BATCH_SIZE` and `d >= GPU_MIN_DIM`, AND
    ///   - the codebook size `N` is <= 32 (the codebook_lookup kernel
    ///     is one warp wide and falls back to CPU otherwise).
    ///
    /// This is the honest gate for receipts: a 4-doc corpus with dim 64
    /// returns false even with `--features gpu`, because the batch is too
    /// small to overcome GPU launch overhead. A corpus with a codebook
    /// larger than 32 also returns false.
    pub fn is_gpu_accelerated_for(&self, n: usize, d: usize) -> bool {
        #[cfg(feature = "gpu")]
        {
            if !gpu_backend::GpuContext::is_available() {
                return false;
            }
            n >= gpu_backend::GpuContext::GPU_MIN_BATCH_SIZE
                && d >= gpu_backend::GpuContext::GPU_MIN_DIM
                && (self.profile.codebook_size as usize) <= 32
        }
        #[cfg(not(feature = "gpu"))]
        {
            let _ = (n, d);
            false
        }
    }

    /// Per-step GPU dispatch report. `hadamard` is true if a batch of size
    /// `n` at dim `d` would dispatch the Hadamard rotation to GPU.
    /// `codebook_lookup` is true only if both the Hadamard AND the
    /// codebook-lookup step would dispatch (additionally requires codebook
    /// size <= 32). The latter is independent of the `gpu_codebook_lookup`
    /// feature gate — the feature controls whether the dispatch is enabled
    /// in `encode_batch`, not whether the kernel would be a win.
    pub fn gpu_steps_for(&self, n: usize, d: usize) -> GpuStepReport {
        let device_available = {
            #[cfg(feature = "gpu")]
            {
                gpu_backend::GpuContext::is_available()
            }
            #[cfg(not(feature = "gpu"))]
            {
                false
            }
        };
        // Thresholds are the same as gpu_backend::GpuContext's. Hard-code
        // them here to avoid requiring the gpu feature for the probe.
        const MIN_BATCH: usize = 16;
        const MIN_DIM: usize = 64;
        let clears_thresholds = n >= MIN_BATCH && d >= MIN_DIM;
        let codebook_fits = (self.profile.codebook_size as usize) <= 32;
        GpuStepReport {
            hadamard: device_available && clears_thresholds,
            codebook_lookup: device_available && clears_thresholds && codebook_fits,
        }
    }

    // ── End batch methods ──

    fn validate_code_header(&self, code: &FibCodeV1) -> Result<()> {
        if code.schema_version != CODE_SCHEMA {
            return Err(FibQuantError::CorruptPayload(format!(
                "code schema_version {}, expected {CODE_SCHEMA}",
                code.schema_version
            )));
        }
        let expected_profile = self.profile.digest()?;
        if code.profile_digest != expected_profile {
            return Err(FibQuantError::ProfileDigestMismatch {
                expected: expected_profile,
                actual: code.profile_digest.clone(),
            });
        }
        if code.codebook_digest != self.codebook.codebook_digest {
            return Err(FibQuantError::CodebookDigestMismatch {
                expected: self.codebook.codebook_digest.clone(),
                actual: code.codebook_digest.clone(),
            });
        }
        let expected_rotation = self.rotation.digest()?;
        if code.rotation_digest != expected_rotation
            || code.rotation_digest != self.codebook.rotation_digest
        {
            return Err(FibQuantError::RotationDigestMismatch {
                expected: expected_rotation,
                actual: code.rotation_digest.clone(),
            });
        }
        if code.ambient_dim != self.profile.ambient_dim
            || code.block_dim != self.profile.block_dim
            || code.block_count != self.profile.block_count()
            || code.wire_index_bits != self.profile.wire_index_bits
            || code.norm_format != self.profile.norm_format
        {
            return Err(FibQuantError::CorruptPayload(
                "encoded header does not match profile".into(),
            ));
        }
        Ok(())
    }
}

/// Stable digest over the encoded artifact fields.
pub fn encoded_digest(code: &FibCodeV1) -> Result<String> {
    json_digest(CODE_SCHEMA, code)
}

fn source_vector_digest(x: &[f32]) -> Result<String> {
    check_finite(x)?;
    let mut bytes = Vec::with_capacity(32 + std::mem::size_of_val(x));
    bytes.extend_from_slice(b"fib_quant_source_vector_v1");
    bytes.push(0);
    bytes.extend_from_slice(&(x.len() as u64).to_le_bytes());
    for value in x {
        bytes.extend_from_slice(&value.to_le_bytes());
    }
    Ok(bytes_digest(&bytes))
}

fn encode_norm(norm: f64, format: &NormFormat) -> Result<Vec<u8>> {
    if !norm.is_finite() || norm <= 0.0 {
        return Err(FibQuantError::CorruptPayload(
            "norm must be finite and positive".into(),
        ));
    }
    match format {
        NormFormat::Fp16Paper => {
            let narrowed = f16::from_f32(norm as f32);
            if !narrowed.is_finite() || narrowed <= f16::ZERO {
                return Err(FibQuantError::CorruptPayload(
                    "norm cannot be represented as finite positive fp16".into(),
                ));
            }
            Ok(narrowed.to_le_bytes().to_vec())
        }
        NormFormat::F32Reference => {
            let narrowed = norm as f32;
            if !narrowed.is_finite() || narrowed <= 0.0 {
                return Err(FibQuantError::CorruptPayload(
                    "norm cannot be represented as finite positive f32".into(),
                ));
            }
            Ok(narrowed.to_le_bytes().to_vec())
        }
    }
}

fn decode_norm(bytes: &[u8], format: &NormFormat) -> Result<f64> {
    match format {
        NormFormat::Fp16Paper => {
            let bytes: [u8; 2] = bytes
                .try_into()
                .map_err(|_| FibQuantError::CorruptPayload("fp16 norm length".into()))?;
            let value = f16::from_le_bytes(bytes).to_f32() as f64;
            if value.is_finite() && value > 0.0 {
                Ok(value)
            } else {
                Err(FibQuantError::CorruptPayload("invalid fp16 norm".into()))
            }
        }
        NormFormat::F32Reference => {
            let bytes: [u8; 4] = bytes
                .try_into()
                .map_err(|_| FibQuantError::CorruptPayload("f32 norm length".into()))?;
            let value = f32::from_le_bytes(bytes) as f64;
            if value.is_finite() && value > 0.0 {
                Ok(value)
            } else {
                Err(FibQuantError::CorruptPayload("invalid f32 norm".into()))
            }
        }
    }
}

fn l2_norm(x: &[f32]) -> f64 {
    x.iter()
        .map(|value| {
            let value = f64::from(*value);
            value * value
        })
        .sum::<f64>()
        .sqrt()
}

fn check_finite(x: &[f32]) -> Result<()> {
    if let Some((idx, _)) = x.iter().enumerate().find(|(_, value)| !value.is_finite()) {
        return Err(FibQuantError::NonFiniteInput(idx));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn f32_norm_overflow_rejects_before_payload_emit() {
        let err = encode_norm(f64::MAX, &NormFormat::F32Reference).unwrap_err();
        assert!(matches!(err, FibQuantError::CorruptPayload(message) if message.contains("f32")));
    }

    #[test]
    fn f32_norm_underflow_rejects_before_payload_emit() {
        let err = encode_norm(
            f64::from(f32::from_bits(1)) / 2.0,
            &NormFormat::F32Reference,
        )
        .unwrap_err();
        assert!(matches!(err, FibQuantError::CorruptPayload(message) if message.contains("f32")));
    }
}

/// Per-step GPU dispatch report.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GpuStepReport {
    /// Hadamard rotation would dispatch to GPU.
    pub hadamard: bool,
    /// Nearest-codebook index lookup would also dispatch to GPU.
    pub codebook_lookup: bool,
}
