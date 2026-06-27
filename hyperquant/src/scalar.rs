/// Return a finite, positive quantization scale.
///
/// Non-finite and non-positive scales are normalized to `1.0` so callers get
/// deterministic behavior instead of hidden NaNs or divide-by-zero artifacts.
pub fn effective_scale(scale: f32) -> f32 {
    if scale.is_finite() && scale > 0.0 {
        scale
    } else {
        1.0
    }
}

/// Round a scaled value into the signed 16-bit code range.
pub fn clamp_i16_code(value: f32) -> i16 {
    if !value.is_finite() {
        return 0;
    }
    let rounded = value.round();
    if rounded > i16::MAX as f32 {
        i16::MAX
    } else if rounded < i16::MIN as f32 {
        i16::MIN
    } else {
        rounded as i16
    }
}

/// Mean squared error between equal-prefix slices.
pub fn mse(input: &[f32], reconstructed: &[f32]) -> f32 {
    if input.is_empty() {
        return 0.0;
    }
    let sum_sq: f64 = input
        .iter()
        .zip(reconstructed.iter())
        .map(|(&a, &b)| {
            let diff = (a - b) as f64;
            diff * diff
        })
        .sum();
    let mse = sum_sq / input.len() as f64;
    if mse <= f32::MAX as f64 {
        mse as f32
    } else {
        f32::INFINITY
    }
}

/// Encode bytes as lowercase hexadecimal without adding a dependency.
pub fn hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for &byte in bytes {
        out.push(HEX[(byte >> 4) as usize] as char);
        out.push(HEX[(byte & 0x0f) as usize] as char);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn effective_scale_falls_back_for_bad_values() {
        assert_eq!(effective_scale(0.0), 1.0);
        assert_eq!(effective_scale(-1.0), 1.0);
        assert_eq!(effective_scale(f32::NAN), 1.0);
        assert_eq!(effective_scale(f32::INFINITY), 1.0);
        assert_eq!(effective_scale(2.5), 2.5);
    }

    #[test]
    fn clamp_i16_code_bounds_large_values() {
        assert_eq!(clamp_i16_code(1.0e9), i16::MAX);
        assert_eq!(clamp_i16_code(-1.0e9), i16::MIN);
        assert_eq!(clamp_i16_code(f32::NAN), 0);
    }
}
