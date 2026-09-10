//! Opt-in encoding migration. No legacy entry point calls this module.
//! V2 implements RFC 8785 serialization with bounded input. Integer tokens
//! must be exactly representable in binary64 OR already be the canonical
//! ECMAScript spelling of that binary64 value (RFC 8785 Appendix B, note 2).
//! Other integer precision loss is rejected; use strings for exact big integers.
//! Digests carry their scheme; neither bytes nor a digest confer authority.

use serde::de::{MapAccess, Visitor};
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::{value::RawValue, Map, Number, Value};
use thiserror::Error;

const MAX_BYTES: usize = 1 << 20;
const MAX_DEPTH: usize = 64;
const SCHEME: &str = "boundary-compiler/rfc8785-v2";

#[derive(Debug, Error)]
pub enum CanonicalV2Error {
    #[error("V2 JSON is malformed or has duplicate decoded keys")]
    InvalidJson,
    #[error("V2 number is nonfinite or integer conversion would lose precision")]
    NumberNotRepresentable,
    #[error("V2 canonicalization resource limit exceeded")]
    ResourceLimit,
    #[error("V2 digest does not match")]
    DigestMismatch,
}

/// Canonicalize an already parsed value. Duplicate keys or numeric precision
/// lost before this call cannot be recovered; use raw ingress for wire data.
pub fn canonicalize_v2(value: &Value) -> Result<String, CanonicalV2Error> {
    let mut out = String::new();
    emit(value, &mut out, 0)?;
    Ok(out)
}

/// Grammar-checked, duplicate-rejecting ingress. Raw numbers are retained until
/// conversion policy is checked; no arbitrary-precision feature or legacy
/// parser is enabled or reused.
pub fn canonicalize_json_v2(input: &[u8]) -> Result<String, CanonicalV2Error> {
    if input.len() > MAX_BYTES {
        return Err(CanonicalV2Error::ResourceLimit);
    }
    let raw: &RawValue =
        serde_json::from_slice(input).map_err(|_| CanonicalV2Error::InvalidJson)?;
    canonicalize_v2(&decode(raw.get(), 0)?)
}

fn decode(raw: &str, depth: usize) -> Result<Value, CanonicalV2Error> {
    if depth > MAX_DEPTH {
        return Err(CanonicalV2Error::ResourceLimit);
    }
    match raw.as_bytes().first() {
        Some(b'{') => {
            struct ObjectVisitor(usize);
            impl<'de> Visitor<'de> for ObjectVisitor {
                type Value = Value;
                fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    f.write_str("an object with unique decoded keys")
                }
                fn visit_map<A: MapAccess<'de>>(self, mut input: A) -> Result<Value, A::Error> {
                    let mut map = Map::new();
                    while let Some(key) = input.next_key::<String>()? {
                        if map.contains_key(&key) {
                            return Err(serde::de::Error::custom("duplicate decoded key"));
                        }
                        let raw = input.next_value::<Box<RawValue>>()?;
                        let value =
                            decode(raw.get(), self.0 + 1).map_err(serde::de::Error::custom)?;
                        map.insert(key, value);
                    }
                    Ok(Value::Object(map))
                }
            }
            let mut de = serde_json::Deserializer::from_str(raw);
            de.deserialize_map(ObjectVisitor(depth))
                .map_err(|_| CanonicalV2Error::InvalidJson)
        }
        Some(b'[') => {
            let items: Vec<&RawValue> =
                serde_json::from_str(raw).map_err(|_| CanonicalV2Error::InvalidJson)?;
            let values = items
                .into_iter()
                .map(|item| decode(item.get(), depth + 1))
                .collect::<Result<Vec<_>, _>>()?;
            Ok(Value::Array(values))
        }
        Some(b'-' | b'0'..=b'9') => {
            let value: f64 = raw
                .parse()
                .map_err(|_| CanonicalV2Error::NumberNotRepresentable)?;
            if !value.is_finite() {
                return Err(CanonicalV2Error::NumberNotRepresentable);
            }
            if !raw.contains(['.', 'e', 'E'])
                && value != 0.0
                && format!("{value:.0}") != raw
                && ryu_js::Buffer::new().format_finite(value) != raw
            {
                return Err(CanonicalV2Error::NumberNotRepresentable);
            }
            Number::from_f64(value)
                .map(Value::Number)
                .ok_or(CanonicalV2Error::NumberNotRepresentable)
        }
        _ => serde_json::from_str(raw).map_err(|_| CanonicalV2Error::InvalidJson),
    }
}

fn emit(value: &Value, out: &mut String, depth: usize) -> Result<(), CanonicalV2Error> {
    if depth > MAX_DEPTH {
        return Err(CanonicalV2Error::ResourceLimit);
    }
    match value {
        Value::Null => out.push_str("null"),
        Value::Bool(value) => out.push_str(if *value { "true" } else { "false" }),
        Value::String(value) => emit_string(value, out)?,
        Value::Number(value) => {
            let float = value
                .as_f64()
                .filter(|n| n.is_finite())
                .ok_or(CanonicalV2Error::NumberNotRepresentable)?;
            // i128 comparison avoids saturation hiding rounding of i64/u64 maxima.
            if value
                .as_i64()
                .is_some_and(|n| float as i128 != i128::from(n))
                || value
                    .as_u64()
                    .is_some_and(|n| float as i128 != i128::from(n))
            {
                return Err(CanonicalV2Error::NumberNotRepresentable);
            }
            out.push_str(ryu_js::Buffer::new().format_finite(float));
        }
        Value::Array(values) => {
            out.push('[');
            for (index, value) in values.iter().enumerate() {
                if index != 0 {
                    out.push(',');
                }
                emit(value, out, depth + 1)?;
            }
            out.push(']');
        }
        Value::Object(values) => {
            let mut entries: Vec<_> = values.iter().collect();
            entries.sort_by(|(a, _), (b, _)| a.encode_utf16().cmp(b.encode_utf16()));
            out.push('{');
            for (index, (key, value)) in entries.into_iter().enumerate() {
                if index != 0 {
                    out.push(',');
                }
                emit_string(key, out)?;
                out.push(':');
                emit(value, out, depth + 1)?;
            }
            out.push('}');
        }
    }
    if out.len() > MAX_BYTES {
        return Err(CanonicalV2Error::ResourceLimit);
    }
    Ok(())
}

fn emit_string(value: &str, out: &mut String) -> Result<(), CanonicalV2Error> {
    if value.len() > MAX_BYTES {
        return Err(CanonicalV2Error::ResourceLimit);
    }
    // serde_json string serialization preserves Unicode and escapes only the
    // JSON-required ASCII controls, quotation mark, and reverse solidus.
    out.push_str(&serde_json::to_string(value).map_err(|_| CanonicalV2Error::InvalidJson)?);
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
enum SchemeV2 {
    #[serde(rename = "boundary-compiler/rfc8785-v2")]
    Rfc8785,
}

/// New wire type and domain-separated preimage; cannot decode as an old bare
/// digest or infer a scheme from missing metadata. No Default implementation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VersionedDigestV2 {
    scheme: SchemeV2,
    #[serde(deserialize_with = "digest_hex")]
    digest: String,
}

fn digest_hex<'de, D: Deserializer<'de>>(de: D) -> Result<String, D::Error> {
    let text = String::deserialize(de)?;
    if text.len() != 64
        || !text
            .bytes()
            .all(|c| c.is_ascii_digit() || (b'a'..=b'f').contains(&c))
    {
        return Err(serde::de::Error::custom("invalid V2 digest encoding"));
    }
    Ok(text)
}

impl VersionedDigestV2 {
    pub fn compute(value: &Value) -> Result<Self, CanonicalV2Error> {
        let bytes = canonicalize_v2(value)?;
        let mut hasher = blake3::Hasher::new();
        hasher.update(SCHEME.as_bytes());
        hasher.update(b"\0");
        hasher.update(bytes.as_bytes());
        Ok(Self {
            scheme: SchemeV2::Rfc8785,
            digest: hasher.finalize().to_hex().to_string(),
        })
    }

    pub fn compute_json(input: &[u8]) -> Result<Self, CanonicalV2Error> {
        let bytes = canonicalize_json_v2(input)?;
        Self::from_canonical_bytes(bytes.as_bytes())
    }

    pub fn verify(&self, value: &Value) -> Result<(), CanonicalV2Error> {
        if *self != Self::compute(value)? {
            return Err(CanonicalV2Error::DigestMismatch);
        }
        Ok(())
    }

    pub fn verify_json(&self, input: &[u8]) -> Result<(), CanonicalV2Error> {
        if *self != Self::compute_json(input)? {
            return Err(CanonicalV2Error::DigestMismatch);
        }
        Ok(())
    }

    fn from_canonical_bytes(bytes: &[u8]) -> Result<Self, CanonicalV2Error> {
        let mut hasher = blake3::Hasher::new();
        hasher.update(SCHEME.as_bytes());
        hasher.update(b"\0");
        hasher.update(bytes);
        Ok(Self {
            scheme: SchemeV2::Rfc8785,
            digest: hasher.finalize().to_hex().to_string(),
        })
    }
}
