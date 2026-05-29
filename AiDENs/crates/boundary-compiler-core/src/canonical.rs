use crate::strict_json::StrictJsonValue;

pub fn canonicalize_stable_sorted_json(
    value: &StrictJsonValue,
) -> Result<Vec<u8>, serde_json::Error> {
    let mut out = Vec::new();
    write_canonical(value, &mut out)?;
    Ok(out)
}

fn write_canonical(value: &StrictJsonValue, out: &mut Vec<u8>) -> Result<(), serde_json::Error> {
    match value {
        StrictJsonValue::Null => out.extend_from_slice(b"null"),
        StrictJsonValue::Bool(true) => out.extend_from_slice(b"true"),
        StrictJsonValue::Bool(false) => out.extend_from_slice(b"false"),
        StrictJsonValue::Number(n) => out.extend_from_slice(n.to_string().as_bytes()),
        StrictJsonValue::String(s) => {
            serde_json::to_writer(&mut *out, s)?;
        }
        StrictJsonValue::Array(values) => {
            out.push(b'[');
            for (idx, value) in values.iter().enumerate() {
                if idx > 0 {
                    out.push(b',');
                }
                write_canonical(value, out)?;
            }
            out.push(b']');
        }
        StrictJsonValue::Object(values) => {
            out.push(b'{');
            for (idx, (key, value)) in values.iter().enumerate() {
                if idx > 0 {
                    out.push(b',');
                }
                serde_json::to_writer(&mut *out, key)?;
                out.push(b':');
                write_canonical(value, out)?;
            }
            out.push(b'}');
        }
    }
    Ok(())
}
