use crate::{receipt, scalar, HyperQuantError, HyperQuantReceiptV1, Result};
use serde::{Deserialize, Serialize};

/// Lattice families known to HyperQuant.
///
/// `Z1` and `A2` are implemented. `D4` and `E8` are exposed as explicit roadmap
/// targets and return `UnsupportedLattice` rather than fake placeholder results.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum LatticeKind {
    Z1 = 1,
    A2 = 2,
    D4 = 4,
    E8 = 8,
}

/// Quantization configuration.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct HyperQuantConfig {
    pub kind: LatticeKind,
    pub scale: f32,
}

impl HyperQuantConfig {
    /// Construct a configuration. Scale normalization happens at quantization time.
    pub fn new(kind: LatticeKind, scale: f32) -> Self {
        Self { kind, scale }
    }

    /// Return the deterministic positive scale that will be used.
    pub fn effective_scale(&self) -> f32 {
        scalar::effective_scale(self.scale)
    }

    /// Stable digest over kind + effective scale.
    pub fn config_digest(&self) -> String {
        receipt::config_digest(self)
    }

    /// Quantize with this configuration.
    pub fn quantize(&self, values: &[f32]) -> Result<HyperQuantResult> {
        match self.kind {
            LatticeKind::Z1 => quantize_z1(values, self.scale),
            LatticeKind::A2 => quantize_a2(values, self.scale),
            LatticeKind::D4 => quantize_d4(values, self.scale),
            LatticeKind::E8 => Err(HyperQuantError::UnsupportedLattice(self.kind)),
        }
    }
}

/// Quantized vector and lossy reconstruction.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HyperQuantResult {
    pub kind: LatticeKind,
    pub codes: Vec<i16>,
    pub reconstructed: Vec<f32>,
    pub mse: f32,
    pub effective_scale: f32,
    pub input_len: usize,
    pub input_digest: String,
    pub config_digest: String,
}

impl HyperQuantResult {
    /// Build an auditable receipt for this exact quantization result.
    pub fn receipt(&self) -> HyperQuantReceiptV1 {
        HyperQuantReceiptV1::from_result(self)
    }
}

/// Quantize each coordinate independently on the integer lattice Z1.
pub fn quantize_z1(values: &[f32], scale: f32) -> Result<HyperQuantResult> {
    validate_input(values)?;
    let effective_scale = scalar::effective_scale(scale);
    let mut codes = Vec::with_capacity(values.len());
    let mut reconstructed = Vec::with_capacity(values.len());

    for &value in values {
        let code = scalar::clamp_i16_code(value * effective_scale);
        codes.push(code);
        reconstructed.push(code as f32 / effective_scale);
    }

    let mse = finite_mse(values, &reconstructed)?;
    let config = HyperQuantConfig::new(LatticeKind::Z1, effective_scale);
    Ok(HyperQuantResult {
        kind: LatticeKind::Z1,
        mse,
        codes,
        reconstructed,
        effective_scale,
        input_len: values.len(),
        input_digest: receipt::input_digest(values),
        config_digest: config.config_digest(),
    })
}

/// Quantize pairs on the A2 triangular lattice.
///
/// The A2 basis is `b1=(1,0)`, `b2=(1/2,sqrt(3)/2)`. For each scaled pair,
/// this searches nearby integer basis coordinates and selects the nearest
/// lattice point. Odd trailing dimensions use the same scalar Z1 rule rather
/// than being dropped.
pub fn quantize_a2(values: &[f32], scale: f32) -> Result<HyperQuantResult> {
    validate_input(values)?;
    let effective_scale = scalar::effective_scale(scale);
    let mut codes = Vec::with_capacity(values.len());
    let mut reconstructed = Vec::with_capacity(values.len());

    let mut chunks = values.chunks_exact(2);
    for pair in &mut chunks {
        let scaled_x = pair[0] * effective_scale;
        let scaled_y = pair[1] * effective_scale;
        let (u, v, rx, ry) = nearest_a2_point(scaled_x, scaled_y);
        codes.push(u);
        codes.push(v);
        reconstructed.push(rx / effective_scale);
        reconstructed.push(ry / effective_scale);
    }

    if let [tail] = chunks.remainder() {
        let code = scalar::clamp_i16_code(*tail * effective_scale);
        codes.push(code);
        reconstructed.push(code as f32 / effective_scale);
    }

    let mse = finite_mse(values, &reconstructed)?;
    let config = HyperQuantConfig::new(LatticeKind::A2, effective_scale);
    Ok(HyperQuantResult {
        kind: LatticeKind::A2,
        mse,
        codes,
        reconstructed,
        effective_scale,
        input_len: values.len(),
        input_digest: receipt::input_digest(values),
        config_digest: config.config_digest(),
    })
}

/// Quantize chunks on the D4 checkerboard lattice.
///
/// D4 is the integer lattice with even coordinate sum in 4D. Each full chunk
/// rounds to the nearest integer vector and flips the coordinate with the
/// largest rounding error when parity repair is needed. Trailing pairs use A2;
/// a single trailing coordinate uses Z1.
pub fn quantize_d4(values: &[f32], scale: f32) -> Result<HyperQuantResult> {
    validate_input(values)?;
    let effective_scale = scalar::effective_scale(scale);
    let mut codes = Vec::with_capacity(values.len());
    let mut reconstructed = Vec::with_capacity(values.len());

    let mut chunks = values.chunks_exact(4);
    for chunk in &mut chunks {
        let scaled = [
            chunk[0] * effective_scale,
            chunk[1] * effective_scale,
            chunk[2] * effective_scale,
            chunk[3] * effective_scale,
        ];
        let (chunk_codes, chunk_reconstructed) = nearest_d4_point(scaled);
        codes.extend_from_slice(&chunk_codes);
        reconstructed.extend(
            chunk_reconstructed
                .iter()
                .map(|value| *value / effective_scale),
        );
    }

    let tail = chunks.remainder();
    if tail.len() >= 2 {
        let (u, v, rx, ry) = nearest_a2_point(tail[0] * effective_scale, tail[1] * effective_scale);
        codes.push(u);
        codes.push(v);
        reconstructed.push(rx / effective_scale);
        reconstructed.push(ry / effective_scale);
    }
    if tail.len() == 1 || tail.len() == 3 {
        let value = tail[tail.len() - 1];
        let code = scalar::clamp_i16_code(value * effective_scale);
        codes.push(code);
        reconstructed.push(code as f32 / effective_scale);
    }

    let mse = finite_mse(values, &reconstructed)?;
    let config = HyperQuantConfig::new(LatticeKind::D4, effective_scale);
    Ok(HyperQuantResult {
        kind: LatticeKind::D4,
        mse,
        codes,
        reconstructed,
        effective_scale,
        input_len: values.len(),
        input_digest: receipt::input_digest(values),
        config_digest: config.config_digest(),
    })
}

fn validate_input(values: &[f32]) -> Result<()> {
    if values.is_empty() {
        return Err(HyperQuantError::EmptyInput);
    }
    if let Some((index, _)) = values
        .iter()
        .enumerate()
        .find(|(_, value)| !value.is_finite())
    {
        return Err(HyperQuantError::NonFiniteInput { index });
    }
    Ok(())
}

fn finite_mse(values: &[f32], reconstructed: &[f32]) -> Result<f32> {
    let mse = scalar::mse(values, reconstructed);
    if mse.is_finite() {
        Ok(mse)
    } else {
        Err(HyperQuantError::NonFiniteArtifact { stage: "mse" })
    }
}

fn nearest_a2_point(x: f32, y: f32) -> (i16, i16, f32, f32) {
    const SQRT_3_OVER_2: f32 = 0.866_025_4;
    let v_float = y / SQRT_3_OVER_2;
    let u_float = x - 0.5 * v_float;
    let u0 = u_float.floor() as i32;
    let v0 = v_float.floor() as i32;

    let mut best = (i16::MAX, i16::MAX, 0.0f32, 0.0f32, f32::INFINITY);
    for du in -2..=2 {
        for dv in -2..=2 {
            let u_i32 = u0.saturating_add(du);
            let v_i32 = v0.saturating_add(dv);
            let u = u_i32.clamp(i16::MIN as i32, i16::MAX as i32) as i16;
            let v = v_i32.clamp(i16::MIN as i32, i16::MAX as i32) as i16;
            let rx = u as f32 + 0.5 * v as f32;
            let ry = SQRT_3_OVER_2 * v as f32;
            let dist = (x - rx).mul_add(x - rx, (y - ry) * (y - ry));
            if dist < best.4 || (dist == best.4 && (u, v) < (best.0, best.1)) {
                best = (u, v, rx, ry, dist);
            }
        }
    }
    (best.0, best.1, best.2, best.3)
}

fn nearest_d4_point(x: [f32; 4]) -> ([i16; 4], [f32; 4]) {
    let mut rounded = [0i32; 4];
    let mut reconstructed = [0.0f32; 4];
    let mut sum = 0i32;
    for idx in 0..4 {
        let code = scalar::clamp_i16_code(x[idx]) as i32;
        rounded[idx] = code;
        reconstructed[idx] = code as f32;
        sum += code;
    }

    if sum % 2 != 0 {
        let mut best_idx = 0usize;
        let mut best_delta = f32::INFINITY;
        let mut best_adjust = 0i32;
        for idx in 0..4 {
            for adjust in [-1i32, 1i32] {
                let candidate = rounded[idx].saturating_add(adjust);
                if !(i16::MIN as i32..=i16::MAX as i32).contains(&candidate) {
                    continue;
                }
                let current_err = (x[idx] - rounded[idx] as f32).powi(2);
                let candidate_err = (x[idx] - candidate as f32).powi(2);
                let delta = candidate_err - current_err;
                if delta < best_delta {
                    best_delta = delta;
                    best_idx = idx;
                    best_adjust = adjust;
                }
            }
        }
        rounded[best_idx] = rounded[best_idx].saturating_add(best_adjust);
        reconstructed[best_idx] = rounded[best_idx] as f32;
    }

    (
        [
            rounded[0] as i16,
            rounded[1] as i16,
            rounded[2] as i16,
            rounded[3] as i16,
        ],
        reconstructed,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nearest_a2_recovers_basis_vector() {
        let (_u, _v, x, y) = nearest_a2_point(0.5, 0.866_025_4);
        assert!((x - 0.5).abs() < 1.0e-6);
        assert!((y - 0.866_025_4).abs() < 1.0e-6);
    }

    #[test]
    fn validate_input_rejects_non_finite_values() {
        assert_eq!(
            validate_input(&[1.0, f32::NAN]),
            Err(HyperQuantError::NonFiniteInput { index: 1 })
        );
        assert_eq!(
            validate_input(&[f32::INFINITY]),
            Err(HyperQuantError::NonFiniteInput { index: 0 })
        );
    }
}
