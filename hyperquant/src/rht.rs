use crate::{scalar, HyperQuantError, Result};
use serde::{Deserialize, Serialize};

/// Receipt for a deterministic local randomized Hadamard transform tile pass.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RhtReceiptV1 {
    pub input_len: usize,
    pub tile_dim: usize,
    pub seed_digest: String,
    pub claim_boundary: &'static str,
}

/// Apply seeded sign flips followed by normalized Hadamard transforms per tile.
pub fn rht_tile(values: &mut [f32], tile_dim: usize, seed: u64) -> Result<RhtReceiptV1> {
    if tile_dim == 0 || !tile_dim.is_power_of_two() {
        return Err(HyperQuantError::InvalidTileDimension { tile_dim });
    }
    if let Some((index, _)) = values
        .iter()
        .enumerate()
        .find(|(_, value)| !value.is_finite())
    {
        return Err(HyperQuantError::NonFiniteInput { index });
    }

    for (tile_index, tile) in values.chunks_mut(tile_dim).enumerate() {
        if tile.len() == tile_dim {
            seeded_sign_flip(tile, seed ^ tile_index as u64);
            hadamard_in_place_power_of_two(tile)?;
        } else {
            for (offset, value) in tile.iter_mut().enumerate() {
                if sign_for(seed ^ tile_index as u64, offset as u64) {
                    *value = -*value;
                }
            }
        }
    }

    Ok(RhtReceiptV1 {
        input_len: values.len(),
        tile_dim,
        seed_digest: seed_digest(seed),
        claim_boundary: "experimental primitive only; deterministic CPU-local RHT helper",
    })
}

/// Apply deterministic ±1 signs in place.
pub fn seeded_sign_flip(values: &mut [f32], seed: u64) {
    for (idx, value) in values.iter_mut().enumerate() {
        if sign_for(seed, idx as u64) {
            *value = -*value;
        }
    }
}

/// Normalized Walsh-Hadamard transform. Length must be a power of two.
pub fn hadamard_in_place_power_of_two(values: &mut [f32]) -> Result<()> {
    let n = values.len();
    if n == 0 || !n.is_power_of_two() {
        return Err(HyperQuantError::InvalidTileDimension { tile_dim: n });
    }
    let mut step = 1usize;
    while step < n {
        let jump = step * 2;
        for start in (0..n).step_by(jump) {
            for offset in 0..step {
                let a = values[start + offset];
                let b = values[start + offset + step];
                values[start + offset] = a + b;
                values[start + offset + step] = a - b;
            }
        }
        step = jump;
    }
    let norm = (n as f32).sqrt().recip();
    for value in values {
        *value *= norm;
    }
    Ok(())
}

fn sign_for(seed: u64, index: u64) -> bool {
    let mut x = seed ^ index.wrapping_mul(0x9E37_79B9_7F4A_7C15);
    x ^= x >> 30;
    x = x.wrapping_mul(0xBF58_476D_1CE4_E5B9);
    x ^= x >> 27;
    x = x.wrapping_mul(0x94D0_49BB_1331_11EB);
    x ^= x >> 31;
    (x & 1) == 1
}

fn seed_digest(seed: u64) -> String {
    let hash = blake3::hash(&seed.to_le_bytes());
    format!("blake3:{}", scalar::hex(hash.as_bytes()))
}
