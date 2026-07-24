//! Deterministic compact wire encoding for [`TurboCode`].
//!
//! The wire bytes are a derived acceleration artifact. They are bound to the
//! quantizer profile on decode and are never authoritative over raw f32 vectors.

use crate::{
    bitpack,
    error::{Result, TurboQuantError},
    polar::PolarCode,
    qjl::QjlSketch,
    rotation::RotationKind,
    turbo::{TurboCode, TurboMode, TurboQuantizer},
};

/// Magic bytes for TurboCode wire format v1.
pub const TURBO_CODE_WIRE_MAGIC: &[u8; 4] = b"TQW1";
/// Exact byte length of a TurboCode wire format v1 header.
pub const TURBO_CODE_WIRE_HEADER_LEN: usize = 46;

/// Magic bytes for self-describing scalar quantization wire format v1.
pub const SCALAR_QUANT_WIRE_MAGIC: &[u8; 4] = b"SQV1";
/// Exact byte length of a scalar quantization wire format v1 header.
pub const SCALAR_QUANT_WIRE_HEADER_LEN: usize = 24;

const VERSION: u16 = 1;
const VARIANT_TURBO_CODE: u8 = 1;
const SCALAR_VERSION: u8 = 1;

/// Encoder/decoder for TurboCode wire format v1.
pub struct TurboCodeWireV1;

impl TurboCodeWireV1 {
    /// Encode a validated TurboCode using the supplied quantizer profile.
    pub fn encode(code: &TurboCode, profile: &TurboQuantizer) -> Result<Vec<u8>> {
        code.validate_for(
            profile.dim(),
            profile.bits(),
            profile.projections(),
            profile.mode(),
        )?;

        let dim = checked_u32(profile.dim(), "dimension")?;
        let polar_bits = code.polar_code.bits;
        let qjl_projections = checked_u32(profile.projections(), "projection count")?;
        let polar_block_count = checked_u32(code.polar_code.radii.len(), "polar block count")?;
        let qjl_sign_count = checked_u32(
            match profile.mode() {
                TurboMode::PolarOnly => 0,
                TurboMode::PolarWithQjl => code.residual_sketch.projections,
            },
            "qjl sign count",
        )?;
        let packed_angle_indices =
            bitpack::pack_indices(&code.polar_code.angle_indices, polar_bits)?;
        let packed_signs = match profile.mode() {
            TurboMode::PolarOnly => Vec::new(),
            TurboMode::PolarWithQjl => bitpack::pack_signs(&code.residual_sketch.signs)?,
        };
        let payload_len = checked_u64(
            code.polar_code.radii.len() * 4 + packed_angle_indices.len() + packed_signs.len(),
            "payload length",
        )?;

        let mut bytes = Vec::with_capacity(TURBO_CODE_WIRE_HEADER_LEN + payload_len as usize);
        bytes.extend_from_slice(TURBO_CODE_WIRE_MAGIC);
        bytes.extend_from_slice(&VERSION.to_le_bytes());
        bytes.extend_from_slice(&rotation_flag(profile.rotation_kind()).to_le_bytes());
        bytes.push(VARIANT_TURBO_CODE);
        bytes.push(0);
        bytes.extend_from_slice(&dim.to_le_bytes());
        bytes.push(polar_bits);
        bytes.extend_from_slice(&[0, 0, 0]);
        bytes.extend_from_slice(&qjl_projections.to_le_bytes());
        bytes.extend_from_slice(&profile.seed().to_le_bytes());
        bytes.extend_from_slice(&polar_block_count.to_le_bytes());
        bytes.extend_from_slice(&qjl_sign_count.to_le_bytes());
        bytes.extend_from_slice(&payload_len.to_le_bytes());

        for radius in &code.polar_code.radii {
            bytes.extend_from_slice(&radius.to_le_bytes());
        }
        bytes.extend_from_slice(&packed_angle_indices);
        bytes.extend_from_slice(&packed_signs);
        Ok(bytes)
    }

    /// Decode and validate TurboCode wire bytes against the supplied profile.
    pub fn decode(bytes: &[u8], profile: &TurboQuantizer) -> Result<TurboCode> {
        let mut cursor = WireCursor::new(bytes);
        if cursor.read_exact(TURBO_CODE_WIRE_MAGIC.len())? != TURBO_CODE_WIRE_MAGIC {
            return Err(TurboQuantError::MalformedCode {
                reason: "wrong TurboQuant wire magic".into(),
            });
        }
        let version = cursor.read_u16()?;
        if version != VERSION {
            return Err(TurboQuantError::MalformedCode {
                reason: format!("unsupported TurboQuant wire version {version}"),
            });
        }
        let wire_rotation_flag = cursor.read_u16()?;
        let expected_rotation_flag = rotation_flag(profile.rotation_kind());
        if wire_rotation_flag != expected_rotation_flag {
            return Err(TurboQuantError::MalformedCode {
                reason: format!(
                    "wire rotation flag {wire_rotation_flag} does not match quantizer profile flag {expected_rotation_flag}"
                ),
            });
        }
        let variant = cursor.read_u8()?;
        if variant != VARIANT_TURBO_CODE {
            return Err(TurboQuantError::MalformedCode {
                reason: format!("unsupported TurboQuant wire variant {variant}"),
            });
        }
        let reserved = cursor.read_u8()?;
        if reserved != 0 {
            return Err(TurboQuantError::MalformedCode {
                reason: "nonzero TurboQuant wire reserved byte".into(),
            });
        }

        let dim = cursor.read_u32()? as usize;
        let polar_bits = cursor.read_u8()?;
        let reserved2 = cursor.read_exact(3)?;
        if reserved2 != [0, 0, 0] {
            return Err(TurboQuantError::MalformedCode {
                reason: "nonzero TurboQuant wire reserved bytes".into(),
            });
        }
        let qjl_projections = cursor.read_u32()? as usize;
        let seed = cursor.read_u64()?;
        let polar_block_count = cursor.read_u32()? as usize;
        let qjl_sign_count = cursor.read_u32()? as usize;
        let payload_len = cursor.read_u64()?;
        let payload_start = cursor.offset();

        let expected_polar_bits = match profile.mode() {
            TurboMode::PolarOnly => profile.bits(),
            TurboMode::PolarWithQjl => profile.bits() - 1,
        };
        if dim != profile.dim()
            || polar_bits != expected_polar_bits
            || qjl_projections != profile.projections()
        {
            return Err(TurboQuantError::MalformedCode {
                reason: "wire header does not match quantizer profile".into(),
            });
        }
        if seed != profile.seed() {
            return Err(TurboQuantError::MalformedCode {
                reason: format!(
                    "wire seed {seed} does not match quantizer profile seed {}",
                    profile.seed()
                ),
            });
        }
        if polar_block_count != profile.dim() / 2 {
            return Err(TurboQuantError::MalformedCode {
                reason: format!(
                    "wire polar block count {polar_block_count} does not match dimension {}",
                    profile.dim()
                ),
            });
        }
        let expected_qjl_sign_count = match profile.mode() {
            TurboMode::PolarOnly => 0,
            TurboMode::PolarWithQjl => profile.projections(),
        };
        if qjl_sign_count != expected_qjl_sign_count {
            return Err(TurboQuantError::MalformedCode {
                reason: format!(
                    "wire sign count {qjl_sign_count} does not match expected {expected_qjl_sign_count}"
                ),
            });
        }
        let angle_bytes = bitpack::packed_len(polar_block_count, polar_bits)?;
        let sign_bytes = match profile.mode() {
            TurboMode::PolarOnly => 0,
            TurboMode::PolarWithQjl => profile.projections().div_ceil(8),
        };
        let residual_bytes = sign_bytes;
        let expected_payload_len = checked_u64(
            polar_block_count * 4 + angle_bytes + residual_bytes,
            "expected payload length",
        )?;
        if payload_len != expected_payload_len {
            return Err(TurboQuantError::MalformedCode {
                reason: format!(
                    "TurboQuant wire payload length {payload_len} does not match expected {expected_payload_len}"
                ),
            });
        }
        if payload_len > cursor.remaining_len() as u64 {
            return Err(TurboQuantError::MalformedCode {
                reason: "TurboQuant wire payload length exceeds remaining bytes".into(),
            });
        }

        let mut radii = Vec::with_capacity(polar_block_count);
        for _ in 0..polar_block_count {
            radii.push(cursor.read_f32()?);
        }
        let packed_angle_indices = cursor.read_exact(angle_bytes)?.to_vec();
        let angle_indices =
            bitpack::unpack_indices(&packed_angle_indices, polar_block_count, polar_bits)?;
        let residual_sketch = match profile.mode() {
            TurboMode::PolarOnly => QjlSketch {
                dim: profile.dim(),
                projections: 0,
                signs: Vec::new(),
            },
            TurboMode::PolarWithQjl => {
                let packed_signs = cursor.read_exact(sign_bytes)?.to_vec();
                let signs = bitpack::unpack_signs(&packed_signs, profile.projections())?;
                QjlSketch {
                    dim: profile.dim(),
                    projections: profile.projections(),
                    signs,
                }
            }
        };
        if cursor.offset() - payload_start != payload_len as usize {
            return Err(TurboQuantError::MalformedCode {
                reason: "TurboQuant wire payload length mismatch".into(),
            });
        }
        cursor.finish()?;

        let code = TurboCode {
            polar_code: PolarCode {
                dim: profile.dim(),
                bits: polar_bits,
                radii,
                angle_indices,
            },
            residual_sketch,
        };
        code.validate_for(
            profile.dim(),
            profile.bits(),
            profile.projections(),
            profile.mode(),
        )?;
        Ok(code)
    }
}

/// Scalar affine quantization mode carried by [`ScalarQuantWireV1`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScalarQuantMode {
    /// One signed eight-bit quantized value per vector element.
    Q8,
    /// One signed four-bit quantized value per vector element, packed two per byte.
    Q4,
}

impl ScalarQuantMode {
    fn wire_value(self) -> u8 {
        match self {
            Self::Q8 => 1,
            Self::Q4 => 2,
        }
    }

    fn from_wire(value: u8) -> Result<Self> {
        match value {
            1 => Ok(Self::Q8),
            2 => Ok(Self::Q4),
            _ => Err(TurboQuantError::MalformedCode {
                reason: format!("unknown scalar quantization mode {value}"),
            }),
        }
    }

    fn bit_width(self) -> u8 {
        match self {
            Self::Q8 => 8,
            Self::Q4 => 4,
        }
    }

    fn minimum_value(self) -> i8 {
        match self {
            Self::Q8 => i8::MIN,
            Self::Q4 => -8,
        }
    }

    fn maximum_value(self) -> i8 {
        match self {
            Self::Q8 => i8::MAX,
            Self::Q4 => 7,
        }
    }

    fn payload_len(self, dimension: usize) -> Result<usize> {
        match self {
            Self::Q8 => Ok(dimension),
            Self::Q4 => dimension
                .checked_add(1)
                .map(|rounded| rounded / 2)
                .ok_or_else(|| TurboQuantError::MalformedCode {
                    reason: "scalar quantization dimension overflow".into(),
                }),
        }
    }
}

/// Validated metadata from a scalar quantization wire artifact.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScalarQuantWireHeader {
    /// Quantization mode encoded in the artifact.
    pub mode: ScalarQuantMode,
    /// Explicit number of bits used for each quantized scalar.
    pub bit_width: u8,
    /// Original vector dimension.
    pub dimension: usize,
    /// Finite positive affine quantization scale.
    pub scale: f32,
    /// Signed affine zero point.
    pub zero_point: i8,
    /// Exact number of payload bytes following the header.
    pub payload_len: usize,
}

/// Encoder and decoder for self-describing affine scalar quantization wire format v1.
pub struct ScalarQuantWireV1;

impl ScalarQuantWireV1 {
    /// Deterministically encode a non-empty finite vector using Q8 or packed Q4.
    pub fn encode(vector: &[f32], mode: ScalarQuantMode) -> Result<Vec<u8>> {
        if vector.is_empty() {
            return Err(TurboQuantError::MalformedCode {
                reason: "scalar quantization does not encode empty vectors".into(),
            });
        }
        if vector.iter().any(|value| !value.is_finite()) {
            return Err(TurboQuantError::MalformedCode {
                reason: "scalar quantization input contains a non-finite value".into(),
            });
        }

        let dimension = checked_u32(vector.len(), "scalar quantization dimension")?;
        let payload_len = mode.payload_len(vector.len())?;
        let payload_len_u32 = checked_u32(payload_len, "scalar quantization payload length")?;
        let capacity = SCALAR_QUANT_WIRE_HEADER_LEN
            .checked_add(payload_len)
            .ok_or_else(|| TurboQuantError::MalformedCode {
                reason: "scalar quantization artifact length overflow".into(),
            })?;

        let mut minimum = vector[0];
        let mut maximum = vector[0];
        for &value in &vector[1..] {
            minimum = minimum.min(value);
            maximum = maximum.max(value);
        }
        let (scale, zero_point) = affine_parameters(minimum, maximum, mode)?;

        let mut bytes = Vec::with_capacity(capacity);
        bytes.extend_from_slice(SCALAR_QUANT_WIRE_MAGIC);
        bytes.push(SCALAR_VERSION);
        bytes.push(mode.wire_value());
        bytes.push(mode.bit_width());
        bytes.push(0);
        bytes.extend_from_slice(&dimension.to_le_bytes());
        bytes.extend_from_slice(&scale.to_le_bytes());
        bytes.push(zero_point as u8);
        bytes.extend_from_slice(&[0, 0, 0]);
        bytes.extend_from_slice(&payload_len_u32.to_le_bytes());

        match mode {
            ScalarQuantMode::Q8 => {
                for &value in vector {
                    bytes.push(quantize(value, scale, zero_point, mode) as u8);
                }
            }
            ScalarQuantMode::Q4 => {
                for pair in vector.chunks(2) {
                    let low =
                        (quantize(pair[0], scale, zero_point, mode) - mode.minimum_value()) as u8;
                    let high = if pair.len() == 2 {
                        ((quantize(pair[1], scale, zero_point, mode) - mode.minimum_value()) as u8)
                            << 4
                    } else {
                        0
                    };
                    bytes.push(low | high);
                }
            }
        }
        Ok(bytes)
    }

    /// Parse and validate a complete scalar quantization artifact header.
    pub fn parse_header(bytes: &[u8]) -> Result<ScalarQuantWireHeader> {
        if bytes.len() < SCALAR_QUANT_WIRE_HEADER_LEN {
            return Err(TurboQuantError::MalformedCode {
                reason: format!(
                    "scalar quantization wire header is {} bytes, need {SCALAR_QUANT_WIRE_HEADER_LEN}",
                    bytes.len()
                ),
            });
        }

        let mut cursor = WireCursor::new(bytes);
        if cursor.read_exact(4)? != SCALAR_QUANT_WIRE_MAGIC {
            return Err(TurboQuantError::MalformedCode {
                reason: "wrong scalar quantization wire magic".into(),
            });
        }
        let version = cursor.read_u8()?;
        if version != SCALAR_VERSION {
            return Err(TurboQuantError::MalformedCode {
                reason: format!("unsupported scalar quantization wire version {version}"),
            });
        }
        let mode = ScalarQuantMode::from_wire(cursor.read_u8()?)?;
        let bit_width = cursor.read_u8()?;
        if bit_width != mode.bit_width() {
            return Err(TurboQuantError::MalformedCode {
                reason: format!(
                    "scalar quantization bit width {bit_width} does not match mode {:?}",
                    mode
                ),
            });
        }
        if cursor.read_u8()? != 0 {
            return Err(TurboQuantError::MalformedCode {
                reason: "nonzero scalar quantization reserved byte".into(),
            });
        }
        let dimension =
            usize::try_from(cursor.read_u32()?).map_err(|_| TurboQuantError::MalformedCode {
                reason: "scalar quantization dimension does not fit usize".into(),
            })?;
        if dimension == 0 {
            return Err(TurboQuantError::MalformedCode {
                reason: "scalar quantization dimension must be nonzero".into(),
            });
        }
        let scale = cursor.read_f32()?;
        if !scale.is_finite() || scale <= 0.0 {
            return Err(TurboQuantError::MalformedCode {
                reason: "scalar quantization scale must be finite and positive".into(),
            });
        }
        let zero_point = cursor.read_u8()? as i8;
        if zero_point < mode.minimum_value() || zero_point > mode.maximum_value() {
            return Err(TurboQuantError::MalformedCode {
                reason: format!("scalar quantization zero point {zero_point} is out of range"),
            });
        }
        if cursor.read_exact(3)? != [0, 0, 0] {
            return Err(TurboQuantError::MalformedCode {
                reason: "nonzero scalar quantization reserved bytes".into(),
            });
        }
        let payload_len =
            usize::try_from(cursor.read_u32()?).map_err(|_| TurboQuantError::MalformedCode {
                reason: "scalar quantization payload length does not fit usize".into(),
            })?;
        let expected_payload_len = mode.payload_len(dimension)?;
        if payload_len != expected_payload_len {
            return Err(TurboQuantError::MalformedCode {
                reason: format!(
                    "scalar quantization payload length {payload_len} does not match expected {expected_payload_len}"
                ),
            });
        }
        if cursor.remaining_len() != payload_len {
            return Err(TurboQuantError::MalformedCode {
                reason: format!(
                    "scalar quantization artifact has {} payload bytes, expected {payload_len}",
                    cursor.remaining_len()
                ),
            });
        }

        Ok(ScalarQuantWireHeader {
            mode,
            bit_width,
            dimension,
            scale,
            zero_point,
            payload_len,
        })
    }

    /// Decode a complete artifact, rejecting a wire mode different from `expected_mode`.
    pub fn decode(bytes: &[u8], expected_mode: ScalarQuantMode) -> Result<Vec<f32>> {
        let header = Self::parse_header(bytes)?;
        if header.mode != expected_mode {
            return Err(TurboQuantError::MalformedCode {
                reason: format!(
                    "scalar quantization mode {:?} does not match expected {:?}",
                    header.mode, expected_mode
                ),
            });
        }
        let payload = &bytes[SCALAR_QUANT_WIRE_HEADER_LEN..];
        let mut decoded = Vec::with_capacity(header.dimension);
        match header.mode {
            ScalarQuantMode::Q8 => {
                for &byte in payload {
                    decoded.push(dequantize(byte as i8, &header)?);
                }
            }
            ScalarQuantMode::Q4 => {
                for index in 0..header.dimension {
                    let packed = payload[index / 2];
                    let nibble = if index % 2 == 0 {
                        packed & 0x0f
                    } else {
                        packed >> 4
                    };
                    let quantized = (nibble as i8) + header.mode.minimum_value();
                    decoded.push(dequantize(quantized, &header)?);
                }
            }
        }
        Ok(decoded)
    }
}

fn affine_parameters(minimum: f32, maximum: f32, mode: ScalarQuantMode) -> Result<(f32, i8)> {
    let qmin = f64::from(mode.minimum_value());
    let qmax = f64::from(mode.maximum_value());
    let (scale64, zero_point64) = if minimum == maximum {
        if minimum == 0.0 {
            (1.0, 0.0)
        } else {
            let divisor = if minimum > 0.0 { qmax } else { -qmin };
            (f64::from(minimum.abs()) / divisor, 0.0)
        }
    } else {
        let scale = (f64::from(maximum) - f64::from(minimum)) / (qmax - qmin);
        let zero_point = (qmin - f64::from(minimum) / scale)
            .round()
            .clamp(qmin, qmax);
        (scale, zero_point)
    };
    if !scale64.is_finite() || scale64 <= 0.0 || scale64 > f64::from(f32::MAX) {
        return Err(TurboQuantError::MalformedCode {
            reason: "scalar quantization input range cannot produce a finite positive scale".into(),
        });
    }
    let scale = (scale64 as f32).max(f32::MIN_POSITIVE);
    Ok((scale, zero_point64 as i8))
}

fn quantize(value: f32, scale: f32, zero_point: i8, mode: ScalarQuantMode) -> i8 {
    let quantized = (f64::from(value) / f64::from(scale) + f64::from(zero_point)).round();
    quantized.clamp(
        f64::from(mode.minimum_value()),
        f64::from(mode.maximum_value()),
    ) as i8
}

fn dequantize(quantized: i8, header: &ScalarQuantWireHeader) -> Result<f32> {
    let value = (f32::from(quantized) - f32::from(header.zero_point)) * header.scale;
    if !value.is_finite() {
        return Err(TurboQuantError::MalformedCode {
            reason: "scalar quantization payload reconstructs a non-finite value".into(),
        });
    }
    Ok(value)
}

fn checked_u32(value: usize, field: &str) -> Result<u32> {
    u32::try_from(value).map_err(|_| TurboQuantError::MalformedCode {
        reason: format!("{field} {value} does not fit u32 wire field"),
    })
}

fn checked_u64(value: usize, field: &str) -> Result<u64> {
    u64::try_from(value).map_err(|_| TurboQuantError::MalformedCode {
        reason: format!("{field} {value} does not fit u64 wire field"),
    })
}

fn rotation_flag(kind: RotationKind) -> u16 {
    match kind {
        RotationKind::Auto => 0,
        RotationKind::FastHadamard => 1,
        RotationKind::StoredQr => 2,
    }
}

struct WireCursor<'a> {
    bytes: &'a [u8],
    offset: usize,
}

/// Decoded TurboQuant wire header. The wire format carries the full
/// quantizer profile (dim, bits, projections, seed, mode, rotation kind)
/// in the first 46 bytes, so a `TurboCode` can be reconstructed from
/// the wire bytes alone — no external quantizer required.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TurboCodeWireHeader {
    /// Original vector dimension.
    pub dim: usize,
    /// Polar-code bits per angle (b in the paper; b-1 for PolarWithQjl mode).
    pub polar_bits: u8,
    /// QJL projection count for the residual sketch.
    pub qjl_projections: usize,
    /// Seed used to derive the projection state.
    pub seed: u64,
    /// Number of polar code blocks (≈ dim / 2).
    pub polar_block_count: usize,
    /// QJL sign count (0 for PolarOnly mode).
    pub qjl_sign_count: usize,
    /// Length of the payload section following the header.
    pub payload_len: u64,
    /// Rotation kind embedded in the wire.
    pub rotation_kind: RotationKind,
}

impl TurboCodeWireV1 {
    /// Parse just the 46-byte wire header. This is the public entry point
    /// for callers that want to extract the quantizer profile from the
    /// wire format without validating against a specific quantizer instance.
    pub fn parse_header(bytes: &[u8]) -> Result<TurboCodeWireHeader> {
        if bytes.len() < TURBO_CODE_WIRE_HEADER_LEN {
            return Err(TurboQuantError::MalformedCode {
                reason: format!(
                    "TurboQuant wire header is {} bytes, need {TURBO_CODE_WIRE_HEADER_LEN}",
                    bytes.len()
                ),
            });
        }
        let mut cursor = WireCursor::new(bytes);
        if cursor.read_exact(4)? != TURBO_CODE_WIRE_MAGIC {
            return Err(TurboQuantError::MalformedCode {
                reason: "wrong TurboQuant wire magic".into(),
            });
        }
        let version = cursor.read_u16()?;
        if version != VERSION {
            return Err(TurboQuantError::MalformedCode {
                reason: format!("unsupported TurboQuant wire version {version}"),
            });
        }
        let wire_rotation_flag = cursor.read_u16()?;
        let rotation_kind = match wire_rotation_flag {
            0 => RotationKind::Auto,
            1 => RotationKind::FastHadamard,
            2 => RotationKind::StoredQr,
            _ => {
                return Err(TurboQuantError::MalformedCode {
                    reason: format!("unknown TurboQuant rotation flag {wire_rotation_flag}"),
                })
            }
        };
        let variant = cursor.read_u8()?;
        if variant != VARIANT_TURBO_CODE {
            return Err(TurboQuantError::MalformedCode {
                reason: format!("unsupported TurboQuant wire variant {variant}"),
            });
        }
        let reserved = cursor.read_u8()?;
        if reserved != 0 {
            return Err(TurboQuantError::MalformedCode {
                reason: "nonzero TurboQuant wire reserved byte".into(),
            });
        }
        let dim = cursor.read_u32()? as usize;
        let polar_bits = cursor.read_u8()?;
        if cursor.read_exact(3)? != [0, 0, 0] {
            return Err(TurboQuantError::MalformedCode {
                reason: "nonzero TurboQuant wire reserved bytes".into(),
            });
        }
        let qjl_projections = cursor.read_u32()? as usize;
        let seed = cursor.read_u64()?;
        let polar_block_count = cursor.read_u32()? as usize;
        let qjl_sign_count = cursor.read_u32()? as usize;
        let payload_len = cursor.read_u64()?;
        Ok(TurboCodeWireHeader {
            dim,
            polar_bits,
            qjl_projections,
            seed,
            polar_block_count,
            qjl_sign_count,
            payload_len,
            rotation_kind,
        })
    }
}

impl<'a> WireCursor<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, offset: 0 }
    }

    fn offset(&self) -> usize {
        self.offset
    }

    fn remaining_len(&self) -> usize {
        self.bytes.len().saturating_sub(self.offset)
    }

    fn read_exact(&mut self, len: usize) -> Result<&'a [u8]> {
        let end = self
            .offset
            .checked_add(len)
            .ok_or_else(|| TurboQuantError::MalformedCode {
                reason: "wire offset overflow".into(),
            })?;
        if end > self.bytes.len() {
            return Err(TurboQuantError::MalformedCode {
                reason: "truncated TurboQuant wire artifact".into(),
            });
        }
        let out = &self.bytes[self.offset..end];
        self.offset = end;
        Ok(out)
    }

    fn read_u8(&mut self) -> Result<u8> {
        Ok(self.read_exact(1)?[0])
    }

    fn read_u16(&mut self) -> Result<u16> {
        let bytes = self.read_exact(2)?;
        Ok(u16::from_le_bytes([bytes[0], bytes[1]]))
    }

    fn read_u32(&mut self) -> Result<u32> {
        let bytes = self.read_exact(4)?;
        Ok(u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
    }

    fn read_u64(&mut self) -> Result<u64> {
        let bytes = self.read_exact(8)?;
        Ok(u64::from_le_bytes([
            bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
        ]))
    }

    fn read_f32(&mut self) -> Result<f32> {
        let bytes = self.read_exact(4)?;
        Ok(f32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
    }

    fn finish(&self) -> Result<()> {
        if self.offset != self.bytes.len() {
            return Err(TurboQuantError::MalformedCode {
                reason: "trailing bytes in TurboQuant wire artifact".into(),
            });
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scalar_fixture(dim: usize) -> Vec<f32> {
        (0..dim)
            .map(|index| ((index as f32 * 0.37).sin() * 0.49).clamp(-0.5, 0.5))
            .collect()
    }

    fn max_abs_error(left: &[f32], right: &[f32]) -> f32 {
        left.iter()
            .zip(right)
            .map(|(a, b)| (a - b).abs())
            .fold(0.0, f32::max)
    }

    fn make_quantizer(dim: usize, seed: u64) -> TurboQuantizer {
        // Use the simplest possible profile: PolarWithQjl, 8-bit, 32 projections.
        TurboQuantizer::new(dim, 8, 32, seed).expect("quantizer build")
    }

    #[test]
    fn parse_header_round_trips_encoded_code() {
        let q = make_quantizer(128, 42);
        let vector: Vec<f32> = (0..128).map(|i| (i as f32 / 128.0) - 0.5).collect();
        let code = q.encode(&vector).expect("encode");
        let wire = TurboCodeWireV1::encode(&code, &q).expect("wire encode");

        let header = TurboCodeWireV1::parse_header(&wire).expect("parse header");
        assert_eq!(header.dim, 128);
        assert_eq!(header.qjl_projections, 32);
        assert_eq!(header.seed, 42);
        assert!(header.polar_block_count > 0);
        assert!(header.payload_len > 0);
    }

    #[test]
    fn parse_header_rejects_short_buffer() {
        let bytes = vec![0u8; 10];
        let result = TurboCodeWireV1::parse_header(&bytes);
        assert!(result.is_err());
    }

    fn valid_turbo_header_prefix(len: usize) -> Vec<u8> {
        let mut bytes = vec![0u8; len];
        bytes[0..4].copy_from_slice(TURBO_CODE_WIRE_MAGIC);
        bytes[4..6].copy_from_slice(&VERSION.to_le_bytes());
        bytes[6..8].copy_from_slice(&rotation_flag(RotationKind::Auto).to_le_bytes());
        bytes[8] = VARIANT_TURBO_CODE;
        bytes
    }

    #[test]
    fn parse_header_rejects_valid_44_byte_prefix_without_panicking() {
        let result = TurboCodeWireV1::parse_header(&valid_turbo_header_prefix(44));
        assert!(matches!(result, Err(TurboQuantError::MalformedCode { .. })));
    }

    #[test]
    fn parse_header_rejects_valid_45_byte_prefix_without_panicking() {
        let result = TurboCodeWireV1::parse_header(&valid_turbo_header_prefix(45));
        assert!(matches!(result, Err(TurboQuantError::MalformedCode { .. })));
    }

    #[test]
    fn parse_header_rejects_bad_magic() {
        let mut bytes = vec![0u8; 44];
        bytes[0..4].copy_from_slice(b"XXXX");
        let result = TurboCodeWireV1::parse_header(&bytes);
        assert!(result.is_err());
    }

    #[test]
    fn parse_header_rejects_unsupported_version() {
        let mut bytes = vec![0u8; 44];
        bytes[0..4].copy_from_slice(TURBO_CODE_WIRE_MAGIC);
        // version = 99 (unsupported)
        bytes[4..6].copy_from_slice(&99u16.to_le_bytes());
        let result = TurboCodeWireV1::parse_header(&bytes);
        assert!(result.is_err());
    }

    #[test]
    fn scalar_q8_and_q4_encode_deterministically_and_parse_headers() {
        let vector = scalar_fixture(128);
        for (mode, expected_bits) in [(ScalarQuantMode::Q8, 8), (ScalarQuantMode::Q4, 4)] {
            let first = ScalarQuantWireV1::encode(&vector, mode).expect("scalar encode");
            let second = ScalarQuantWireV1::encode(&vector, mode).expect("scalar encode");
            assert_eq!(first, second);

            let header = ScalarQuantWireV1::parse_header(&first).expect("scalar header");
            assert_eq!(header.mode, mode);
            assert_eq!(header.bit_width, expected_bits);
            assert_eq!(header.dimension, vector.len());
            assert!(header.scale.is_finite() && header.scale > 0.0);
            assert!(header.zero_point >= mode.minimum_value());
            assert!(header.zero_point <= mode.maximum_value());
            assert_eq!(header.payload_len, mode.payload_len(vector.len()).unwrap());
        }
    }

    #[test]
    fn scalar_q8_and_q4_round_trip_size_and_fixture_error_are_bounded() {
        let vector = scalar_fixture(128);
        let raw_len = vector.len() * std::mem::size_of::<f32>();
        let q8 = ScalarQuantWireV1::encode(&vector, ScalarQuantMode::Q8).expect("q8 encode");
        let q4 = ScalarQuantWireV1::encode(&vector, ScalarQuantMode::Q4).expect("q4 encode");
        assert!(q8.len() < raw_len);
        assert!(q4.len() < q8.len());

        let decoded_q8 = ScalarQuantWireV1::decode(&q8, ScalarQuantMode::Q8).expect("q8 decode");
        let decoded_q4 = ScalarQuantWireV1::decode(&q4, ScalarQuantMode::Q4).expect("q4 decode");
        assert_eq!(decoded_q8.len(), vector.len());
        assert_eq!(decoded_q4.len(), vector.len());
        assert!(decoded_q8.iter().all(|value| value.is_finite()));
        assert!(decoded_q4.iter().all(|value| value.is_finite()));
        assert!(max_abs_error(&vector, &decoded_q8) <= 0.0021);
        assert!(max_abs_error(&vector, &decoded_q4) <= 0.034);
    }

    #[test]
    fn scalar_q4_round_trips_odd_dimension_without_decoding_padding_nibble() {
        let vector = scalar_fixture(129);
        let encoded = ScalarQuantWireV1::encode(&vector, ScalarQuantMode::Q4).expect("q4 encode");
        let decoded = ScalarQuantWireV1::decode(&encoded, ScalarQuantMode::Q4).expect("q4 decode");
        assert_eq!(decoded.len(), vector.len());
    }

    #[test]
    fn scalar_wire_rejects_empty_and_non_finite_vectors() {
        assert!(matches!(
            ScalarQuantWireV1::encode(&[], ScalarQuantMode::Q8),
            Err(TurboQuantError::MalformedCode { .. })
        ));
        for invalid in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
            assert!(matches!(
                ScalarQuantWireV1::encode(&[0.0, invalid], ScalarQuantMode::Q4),
                Err(TurboQuantError::MalformedCode { .. })
            ));
        }
    }

    #[test]
    fn scalar_wire_rejects_truncation_trailing_bytes_and_mode_mismatch() {
        let encoded =
            ScalarQuantWireV1::encode(&scalar_fixture(17), ScalarQuantMode::Q4).expect("encode");
        assert!(matches!(
            ScalarQuantWireV1::decode(&encoded[..encoded.len() - 1], ScalarQuantMode::Q4),
            Err(TurboQuantError::MalformedCode { .. })
        ));
        let mut trailing = encoded.clone();
        trailing.push(0);
        assert!(matches!(
            ScalarQuantWireV1::decode(&trailing, ScalarQuantMode::Q4),
            Err(TurboQuantError::MalformedCode { .. })
        ));
        assert!(matches!(
            ScalarQuantWireV1::decode(&encoded, ScalarQuantMode::Q8),
            Err(TurboQuantError::MalformedCode { .. })
        ));
    }

    #[test]
    fn scalar_wire_rejects_corrupt_header_fields_and_payload_length() {
        let encoded =
            ScalarQuantWireV1::encode(&scalar_fixture(16), ScalarQuantMode::Q8).expect("encode");
        for (offset, value) in [(0, b'X'), (4, 2), (5, 99), (6, 4), (7, 1), (17, 1)] {
            let mut corrupt = encoded.clone();
            corrupt[offset] = value;
            assert!(matches!(
                ScalarQuantWireV1::decode(&corrupt, ScalarQuantMode::Q8),
                Err(TurboQuantError::MalformedCode { .. })
            ));
        }

        let mut invalid_scale = encoded.clone();
        invalid_scale[12..16].copy_from_slice(&f32::NAN.to_le_bytes());
        assert!(matches!(
            ScalarQuantWireV1::decode(&invalid_scale, ScalarQuantMode::Q8),
            Err(TurboQuantError::MalformedCode { .. })
        ));

        let mut oversized_payload = encoded;
        oversized_payload[20..24].copy_from_slice(&u32::MAX.to_le_bytes());
        assert!(matches!(
            ScalarQuantWireV1::decode(&oversized_payload, ScalarQuantMode::Q8),
            Err(TurboQuantError::MalformedCode { .. })
        ));
    }
}
