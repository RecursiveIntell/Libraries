use super::KvEncodedTensorV1;
use crate::{FibQuantError, Result};

/// Stable wire marker for the framed KV artifact projection.
pub const KV_WIRE_MAGIC: [u8; 4] = *b"FQKV";
/// Current framed wire version.
pub const KV_WIRE_VERSION: u16 = 1;
/// Fixed header length: magic, version, flags, payload length, payload digest.
pub const KV_WIRE_HEADER_LEN: usize = 4 + 2 + 2 + 4 + 32;
/// Defensive upper bound checked before deserialization.
pub const KV_WIRE_MAX_PAYLOAD_BYTES: usize = 256 * 1024 * 1024;

/// Encode a receipt-rich KV tensor into a bounded, checksummed binary frame.
pub fn encode_kv_wire(tensor: &KvEncodedTensorV1) -> Result<Vec<u8>> {
    let payload = bincode::serialize(tensor)
        .map_err(|err| FibQuantError::CorruptPayload(format!("kv wire encode: {err}")))?;
    if payload.len() > KV_WIRE_MAX_PAYLOAD_BYTES {
        return Err(FibQuantError::CorruptPayload(format!(
            "kv wire payload exceeds {} bytes",
            KV_WIRE_MAX_PAYLOAD_BYTES
        )));
    }
    let payload_len = u32::try_from(payload.len())
        .map_err(|_| FibQuantError::CorruptPayload("kv wire payload length overflow".into()))?;
    let digest = blake3::hash(&payload);
    let mut frame = Vec::with_capacity(KV_WIRE_HEADER_LEN + payload.len());
    frame.extend_from_slice(&KV_WIRE_MAGIC);
    frame.extend_from_slice(&KV_WIRE_VERSION.to_le_bytes());
    frame.extend_from_slice(&0u16.to_le_bytes());
    frame.extend_from_slice(&payload_len.to_le_bytes());
    frame.extend_from_slice(digest.as_bytes());
    frame.extend_from_slice(&payload);
    Ok(frame)
}

/// Decode one complete KV wire frame, rejecting version, checksum, and trailing-byte errors.
pub fn decode_kv_wire(frame: &[u8]) -> Result<KvEncodedTensorV1> {
    if frame.len() < KV_WIRE_HEADER_LEN {
        return Err(FibQuantError::CorruptPayload(
            "kv wire frame is truncated".into(),
        ));
    }
    if frame[..4] != KV_WIRE_MAGIC {
        return Err(FibQuantError::CorruptPayload(
            "kv wire magic mismatch".into(),
        ));
    }
    let version = u16::from_le_bytes([frame[4], frame[5]]);
    if version != KV_WIRE_VERSION {
        return Err(FibQuantError::CorruptPayload(format!(
            "unsupported kv wire version {version}"
        )));
    }
    let flags = u16::from_le_bytes([frame[6], frame[7]]);
    if flags != 0 {
        return Err(FibQuantError::CorruptPayload(format!(
            "unsupported kv wire flags 0x{flags:04x}"
        )));
    }
    let payload_len = u32::from_le_bytes([frame[8], frame[9], frame[10], frame[11]]) as usize;
    if payload_len > KV_WIRE_MAX_PAYLOAD_BYTES {
        return Err(FibQuantError::CorruptPayload(format!(
            "kv wire payload exceeds {} bytes",
            KV_WIRE_MAX_PAYLOAD_BYTES
        )));
    }
    let expected_len = KV_WIRE_HEADER_LEN
        .checked_add(payload_len)
        .ok_or_else(|| FibQuantError::CorruptPayload("kv wire frame length overflow".into()))?;
    if frame.len() != expected_len {
        return Err(FibQuantError::CorruptPayload(format!(
            "kv wire length mismatch: frame={}, expected={expected_len}",
            frame.len()
        )));
    }
    let expected_digest = &frame[12..44];
    let payload = &frame[KV_WIRE_HEADER_LEN..];
    let actual_digest = blake3::hash(payload);
    if actual_digest.as_bytes() != expected_digest {
        return Err(FibQuantError::CorruptPayload(
            "kv wire payload digest mismatch".into(),
        ));
    }
    let tensor: KvEncodedTensorV1 = bincode::deserialize(payload)
        .map_err(|err| FibQuantError::CorruptPayload(format!("kv wire decode: {err}")))?;
    let canonical = bincode::serialize(&tensor)
        .map_err(|err| FibQuantError::CorruptPayload(format!("kv wire canonicalize: {err}")))?;
    if canonical != payload {
        return Err(FibQuantError::CorruptPayload(
            "kv wire payload is not canonical".into(),
        ));
    }
    Ok(tensor)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kv::{
        encode_kv_tensor, KvAttentionKind, KvAxisPolicyV1, KvCacheLayoutV1, KvCompressionProfileV1,
        KvDType, KvPageGeometryV1, KvRole, KvRopeState, KvTensorShapeV1,
    };
    use crate::{FibQuantProfileV1, FibQuantizer};

    fn fixture() -> KvEncodedTensorV1 {
        let shape = KvTensorShapeV1::new(
            KvRole::Value,
            KvAttentionKind::Mha,
            1,
            1,
            1,
            1,
            1,
            4,
            KvDType::F32,
            KvRopeState::NotApplicable,
        );
        let fib_profile = FibQuantProfileV1::paper_default(4, 2, 4, 7).unwrap();
        let quantizer = FibQuantizer::new(fib_profile.clone()).unwrap();
        let profile = KvCompressionProfileV1::from_parts(
            "wire-test",
            &shape,
            fib_profile,
            quantizer.codebook().codebook_digest.clone(),
            KvAxisPolicyV1::PerToken,
            KvPageGeometryV1::new(1, 4, 1040),
        )
        .unwrap();
        let layout = KvCacheLayoutV1::canonical(&shape).unwrap();
        encode_kv_tensor(shape, layout, profile, &[0.25, -0.5, 0.75, 1.0]).unwrap()
    }

    #[test]
    fn wire_round_trip_and_rejects_trailing_bytes() {
        let frame = encode_kv_wire(&fixture()).unwrap();
        assert_eq!(decode_kv_wire(&frame).unwrap(), fixture());
        let mut trailing = frame;
        trailing.push(0);
        assert!(decode_kv_wire(&trailing).is_err());
    }

    #[test]
    fn wire_rejects_magic_version_flags_digest_and_truncation() {
        let frame = encode_kv_wire(&fixture()).unwrap();
        for end in 0..KV_WIRE_HEADER_LEN {
            assert!(decode_kv_wire(&frame[..end]).is_err());
        }

        let mut magic = frame.clone();
        magic[0] ^= 1;
        assert!(decode_kv_wire(&magic).is_err());

        let mut version = frame.clone();
        version[4] = 2;
        assert!(decode_kv_wire(&version).is_err());

        let mut flags = frame.clone();
        flags[6] = 1;
        assert!(decode_kv_wire(&flags).is_err());

        let mut digest = frame;
        digest[12] ^= 1;
        assert!(decode_kv_wire(&digest).is_err());
    }
}
