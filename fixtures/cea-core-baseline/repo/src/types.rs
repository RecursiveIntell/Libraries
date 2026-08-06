use std::fmt;

use check_runner::EffectSignature;
use serde::de::{self, Deserializer, Visitor};
use serde::ser::Serializer;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum EditOpKind {
    Insert,
    Delete,
    Replace,
}

impl EditOpKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Insert => "insert",
            Self::Delete => "delete",
            Self::Replace => "replace",
        }
    }
}

impl Serialize for EditOpKind {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for EditOpKind {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        match value.as_str() {
            "insert" | "Insert" => Ok(Self::Insert),
            "delete" | "Delete" => Ok(Self::Delete),
            "replace" | "Replace" => Ok(Self::Replace),
            _ => Err(de::Error::custom(format!("unknown edit op kind: {value}"))),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum AnchorKind {
    AfterLine,
    BeforeLine,
    AfterMatch,
    BeforeMatch,
    Range,
}

impl AnchorKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::AfterLine => "after_line",
            Self::BeforeLine => "before_line",
            Self::AfterMatch => "after_match",
            Self::BeforeMatch => "before_match",
            Self::Range => "range",
        }
    }
}

impl Serialize for AnchorKind {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for AnchorKind {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        match value.as_str() {
            "after_line" | "AfterLine" => Ok(Self::AfterLine),
            "before_line" | "BeforeLine" => Ok(Self::BeforeLine),
            "after_match" | "AfterMatch" => Ok(Self::AfterMatch),
            "before_match" | "BeforeMatch" => Ok(Self::BeforeMatch),
            "range" => Ok(Self::Range),
            _ => Err(de::Error::custom(format!("unknown anchor kind: {value}"))),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ScopeTag {
    Function,
    Implementation,
    Struct,
    Enum,
    Trait,
    Module,
    Macro,
    Unknown,
}

impl ScopeTag {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Function => "fn",
            Self::Implementation => "impl",
            Self::Struct => "struct",
            Self::Enum => "enum",
            Self::Trait => "trait",
            Self::Module => "mod",
            Self::Macro => "macro",
            Self::Unknown => "unknown",
        }
    }
}

impl Serialize for ScopeTag {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for ScopeTag {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        match value.as_str() {
            "fn" => Ok(Self::Function),
            "impl" => Ok(Self::Implementation),
            "struct" => Ok(Self::Struct),
            "enum" => Ok(Self::Enum),
            "trait" => Ok(Self::Trait),
            "mod" => Ok(Self::Module),
            "macro" | "macro_rules" => Ok(Self::Macro),
            "top_level" | "unknown" => Ok(Self::Unknown),
            _ => Err(de::Error::custom(format!("unknown scope tag: {value}"))),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct OpIndex(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct FileIndex(pub u32);

fn serialize_index<S>(value: u32, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_u32(value)
}

struct IndexVisitor;

impl<'de> Visitor<'de> for IndexVisitor {
    type Value = u32;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a u32 index or a legacy position label")
    }

    fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        u32::try_from(value).map_err(|_| E::custom(format!("index out of range: {value}")))
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        match value {
            "only" | "first" => Ok(0),
            "middle" | "last" => Ok(1),
            _ => value
                .parse::<u32>()
                .map_err(|_| E::custom(format!("invalid index label: {value}"))),
        }
    }
}

fn deserialize_index<'de, D>(deserializer: D) -> Result<u32, D::Error>
where
    D: Deserializer<'de>,
{
    deserializer.deserialize_any(IndexVisitor)
}

impl Serialize for OpIndex {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serialize_index(self.0, serializer)
    }
}

impl<'de> Deserialize<'de> for OpIndex {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        if deserializer.is_human_readable() {
            Ok(Self(deserialize_index(deserializer)?))
        } else {
            Ok(Self(u32::deserialize(deserializer)?))
        }
    }
}

impl Serialize for FileIndex {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serialize_index(self.0, serializer)
    }
}

impl<'de> Deserialize<'de> for FileIndex {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        if deserializer.is_human_readable() {
            Ok(Self(deserialize_index(deserializer)?))
        } else {
            Ok(Self(u32::deserialize(deserializer)?))
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EditOpSignature {
    pub op_kind: EditOpKind,
    pub anchor_kind: AnchorKind,
    pub lines_added: u32,
    pub lines_removed: u32,
    pub context_hash: String,
    pub file_extension: String,
    pub scope_tag: ScopeTag,
    pub op_index: OpIndex,
    pub file_index: FileIndex,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CausalPrediction {
    pub predicted_correctness: f64,
    pub predicted_novelty: f64,
    pub confidence: f64,
    pub coverage_fraction: f64,
    pub risk_flags: Vec<RiskFlag>,
    pub zero_shot_eligible: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RiskFlag {
    pub op_signature: EditOpSignature,
    pub predicted_effect: EffectSignature,
    pub confidence: f64,
    pub historical_weight: f64,
}
