use serde::de::{self, Deserialize, Deserializer, MapAccess, SeqAccess, Visitor};
use serde_json::{Number, Value};
use std::collections::BTreeMap;
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum StrictJsonValue {
    Null,
    Bool(bool),
    Number(Number),
    String(String),
    Array(Vec<StrictJsonValue>),
    Object(BTreeMap<String, StrictJsonValue>),
}

impl<'de> Deserialize<'de> for StrictJsonValue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(StrictJsonVisitor)
    }
}

struct StrictJsonVisitor;

impl<'de> Visitor<'de> for StrictJsonVisitor {
    type Value = StrictJsonValue;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("strict JSON value with duplicate-key rejection")
    }

    fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E> {
        Ok(StrictJsonValue::Bool(v))
    }

    fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E> {
        Ok(StrictJsonValue::Number(Number::from(v)))
    }

    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E> {
        Ok(StrictJsonValue::Number(Number::from(v)))
    }

    fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Number::from_f64(v)
            .map(StrictJsonValue::Number)
            .ok_or_else(|| E::custom("non-finite number is not valid JSON"))
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(StrictJsonValue::String(v.to_string()))
    }

    fn visit_string<E>(self, v: String) -> Result<Self::Value, E> {
        Ok(StrictJsonValue::String(v))
    }

    fn visit_none<E>(self) -> Result<Self::Value, E> {
        Ok(StrictJsonValue::Null)
    }

    fn visit_unit<E>(self) -> Result<Self::Value, E> {
        Ok(StrictJsonValue::Null)
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut out = Vec::new();
        while let Some(value) = seq.next_element::<StrictJsonValue>()? {
            out.push(value);
        }
        Ok(StrictJsonValue::Array(out))
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut out = BTreeMap::new();
        while let Some(key) = map.next_key::<String>()? {
            if out.contains_key(&key) {
                return Err(de::Error::custom(format!("duplicate key: {key}")));
            }
            let value = map.next_value::<StrictJsonValue>()?;
            out.insert(key, value);
        }
        Ok(StrictJsonValue::Object(out))
    }
}

impl StrictJsonValue {
    pub fn parse(raw: &[u8]) -> Result<Self, serde_json::Error> {
        let mut de = serde_json::Deserializer::from_slice(raw);
        let value = StrictJsonValue::deserialize(&mut de)?;
        de.end()?;
        Ok(value)
    }

    pub fn to_json_value(&self) -> Value {
        match self {
            StrictJsonValue::Null => Value::Null,
            StrictJsonValue::Bool(v) => Value::Bool(*v),
            StrictJsonValue::Number(v) => Value::Number(v.clone()),
            StrictJsonValue::String(v) => Value::String(v.clone()),
            StrictJsonValue::Array(values) => {
                Value::Array(values.iter().map(StrictJsonValue::to_json_value).collect())
            }
            StrictJsonValue::Object(values) => {
                let mut map = serde_json::Map::new();
                for (key, value) in values {
                    map.insert(key.clone(), value.to_json_value());
                }
                Value::Object(map)
            }
        }
    }

    pub fn max_depth(&self) -> usize {
        match self {
            StrictJsonValue::Array(values) => {
                1 + values
                    .iter()
                    .map(StrictJsonValue::max_depth)
                    .max()
                    .unwrap_or(0)
            }
            StrictJsonValue::Object(values) => {
                1 + values
                    .values()
                    .map(StrictJsonValue::max_depth)
                    .max()
                    .unwrap_or(0)
            }
            _ => 1,
        }
    }

    pub fn total_object_keys(&self) -> usize {
        match self {
            StrictJsonValue::Array(values) => {
                values.iter().map(StrictJsonValue::total_object_keys).sum()
            }
            StrictJsonValue::Object(values) => {
                values.len()
                    + values
                        .values()
                        .map(StrictJsonValue::total_object_keys)
                        .sum::<usize>()
            }
            _ => 0,
        }
    }

    pub fn json_type_name(&self) -> &'static str {
        match self {
            StrictJsonValue::Null => "null",
            StrictJsonValue::Bool(_) => "bool",
            StrictJsonValue::Number(_) => "number",
            StrictJsonValue::String(_) => "string",
            StrictJsonValue::Array(_) => "array",
            StrictJsonValue::Object(_) => "object",
        }
    }

    pub fn get_pointer(&self, pointer: &str) -> Option<&StrictJsonValue> {
        if pointer.is_empty() || pointer == "/" {
            return Some(self);
        }
        let mut current = self;
        for segment in pointer.trim_start_matches('/').split('/') {
            let segment = segment.replace("~1", "/").replace("~0", "~");
            match current {
                StrictJsonValue::Object(map) => current = map.get(&segment)?,
                StrictJsonValue::Array(values) => {
                    let index = segment.parse::<usize>().ok()?;
                    current = values.get(index)?;
                }
                _ => return None,
            }
        }
        Some(current)
    }
}
