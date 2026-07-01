use crate::{HyperQuantError, Result};
use serde::{Deserialize, Serialize};
use std::ops::RangeInclusive;

/// Compact unsigned Rice bitstream with enough metadata for deterministic decode.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RiceBitstream {
    pub k: u8,
    pub value_count: usize,
    pub encoded_bits: usize,
    pub payload: Vec<u8>,
}

impl RiceBitstream {
    /// Number of bytes occupied by the bit payload.
    pub fn encoded_bytes(&self) -> usize {
        self.encoded_bits.div_ceil(8)
    }

    /// Bits per original scalar/code.
    pub fn bits_per_value(&self) -> f32 {
        if self.value_count == 0 {
            0.0
        } else {
            self.encoded_bits as f32 / self.value_count as f32
        }
    }
}

/// Deterministic Rice profile selected for a code slice.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct RiceProfile {
    pub k: u8,
    pub encoded_bits: usize,
    pub encoded_bytes: usize,
    pub bits_per_scalar: f32,
}

/// Zig-zag encode signed i16 into non-negative symbols.
pub fn zigzag_i16(value: i16) -> u32 {
    let value = value as i32;
    ((value << 1) ^ (value >> 31)) as u32
}

/// Decode a zig-zagged value back into i16, saturating only unreachable oversized symbols.
pub fn unzigzag_u32(value: u32) -> i16 {
    let decoded = ((value >> 1) as i32) ^ (-((value & 1) as i32));
    decoded.clamp(i16::MIN as i32, i16::MAX as i32) as i16
}

/// Pick the Rice parameter that minimizes exact encoded bit count.
pub fn estimate_best_rice_profile(
    codes: &[i16],
    k_range: RangeInclusive<u8>,
) -> Result<RiceProfile> {
    let mut best: Option<RiceProfile> = None;
    for k in k_range {
        let stream = rice_encode_i16(codes, k)?;
        let profile = RiceProfile {
            k,
            encoded_bits: stream.encoded_bits,
            encoded_bytes: stream.encoded_bytes(),
            bits_per_scalar: stream.bits_per_value(),
        };
        let should_replace = match best.as_ref() {
            Some(current) => (profile.encoded_bits, profile.k) < (current.encoded_bits, current.k),
            None => true,
        };
        if should_replace {
            best = Some(profile);
        }
    }
    best.ok_or(HyperQuantError::InvalidRiceBitstream {
        reason: "empty k range",
    })
}

/// Encode signed i16 codes using zig-zag + Rice coding.
pub fn rice_encode_i16(codes: &[i16], k: u8) -> Result<RiceBitstream> {
    validate_k(k)?;
    let values: Vec<u32> = codes.iter().copied().map(zigzag_i16).collect();
    rice_encode_u32s(&values, k)
}

/// Decode signed i16 codes from a zig-zag + Rice bitstream.
pub fn rice_decode_i16(stream: &RiceBitstream) -> Result<Vec<i16>> {
    let values = rice_decode_u32s(stream)?;
    Ok(values.into_iter().map(unzigzag_u32).collect())
}

/// Encode non-negative symbols using unary quotient + fixed-width remainder.
pub fn rice_encode_u32s(values: &[u32], k: u8) -> Result<RiceBitstream> {
    validate_k(k)?;
    let mut writer = BitWriter::default();
    let mask = if k == 32 { u32::MAX } else { (1u32 << k) - 1 };
    for &value in values {
        let quotient = value >> k;
        for _ in 0..quotient {
            writer.push_bit(true);
        }
        writer.push_bit(false);
        let remainder = value & mask;
        for bit in (0..k).rev() {
            writer.push_bit(((remainder >> bit) & 1) != 0);
        }
    }
    Ok(RiceBitstream {
        k,
        value_count: values.len(),
        encoded_bits: writer.bits,
        payload: writer.bytes,
    })
}

/// Decode non-negative Rice symbols.
pub fn rice_decode_u32s(stream: &RiceBitstream) -> Result<Vec<u32>> {
    validate_k(stream.k)?;
    if stream.encoded_bits > stream.payload.len() * 8 {
        return Err(HyperQuantError::InvalidRiceBitstream {
            reason: "encoded bit count exceeds payload size",
        });
    }
    let mut reader = BitReader::new(&stream.payload, stream.encoded_bits);
    let mut values = Vec::with_capacity(stream.value_count);
    for _ in 0..stream.value_count {
        let mut quotient = 0u32;
        loop {
            let bit = reader
                .next_bit()?
                .ok_or(HyperQuantError::InvalidRiceBitstream {
                    reason: "missing unary terminator",
                })?;
            if bit {
                quotient =
                    quotient
                        .checked_add(1)
                        .ok_or(HyperQuantError::InvalidRiceBitstream {
                            reason: "quotient overflow",
                        })?;
            } else {
                break;
            }
        }
        let mut remainder = 0u32;
        for _ in 0..stream.k {
            let bit = reader
                .next_bit()?
                .ok_or(HyperQuantError::InvalidRiceBitstream {
                    reason: "missing remainder bits",
                })?;
            remainder = (remainder << 1) | u32::from(bit);
        }
        values.push((quotient << stream.k) | remainder);
    }
    Ok(values)
}

fn validate_k(k: u8) -> Result<()> {
    if k <= 15 {
        Ok(())
    } else {
        Err(HyperQuantError::InvalidRiceParameter { k })
    }
}

#[derive(Default)]
struct BitWriter {
    bytes: Vec<u8>,
    bits: usize,
}

impl BitWriter {
    fn push_bit(&mut self, bit: bool) {
        let byte_index = self.bits / 8;
        let bit_index = 7 - (self.bits % 8);
        if byte_index == self.bytes.len() {
            self.bytes.push(0);
        }
        if bit {
            self.bytes[byte_index] |= 1 << bit_index;
        }
        self.bits += 1;
    }
}

struct BitReader<'a> {
    bytes: &'a [u8],
    bits: usize,
    cursor: usize,
}

impl<'a> BitReader<'a> {
    fn new(bytes: &'a [u8], bits: usize) -> Self {
        Self {
            bytes,
            bits,
            cursor: 0,
        }
    }

    fn next_bit(&mut self) -> Result<Option<bool>> {
        if self.cursor >= self.bits {
            return Ok(None);
        }
        let byte =
            *self
                .bytes
                .get(self.cursor / 8)
                .ok_or(HyperQuantError::InvalidRiceBitstream {
                    reason: "cursor beyond payload",
                })?;
        let bit_index = 7 - (self.cursor % 8);
        self.cursor += 1;
        Ok(Some(((byte >> bit_index) & 1) != 0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zigzag_roundtrips_extremes() {
        for value in [i16::MIN, -2, -1, 0, 1, 2, i16::MAX] {
            assert_eq!(unzigzag_u32(zigzag_i16(value)), value);
        }
    }
}
