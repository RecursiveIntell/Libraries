//! # effect-signature
//!
//! Stable effect payloads and hashing helpers for check outputs.
//!
//! This crate owns the normalized signature types that let execution results be
//! compared, attributed, and persisted across the Forge lane. It does not own
//! check execution or causal graph policy.

//! # effect-signature
//!
//! Shared effect identity types for internal verification flows.
//!
//! This Tier 3 crate keeps one narrow schema for observed check effects so
//! runners, attribution, and storage code do not fork their own effect models.

use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct EffectSignature {
    pub check_kind: String,
    pub outcome: String,
    pub severity: String,
    pub message_class: String,
    pub line_offset_from_edit: Option<i32>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct LocatedEffect {
    pub file: Option<PathBuf>,
    pub line: Option<u32>,
    pub col: Option<u32>,
    pub message: String,
    pub sig: EffectSignature,
}

pub fn effect_signature_hash(signature: &EffectSignature) -> String {
    let payload = serde_json::to_vec(signature).unwrap_or_default();
    blake3::hash(&payload).to_hex().to_string()
}

#[cfg(test)]
mod wave1_tests {
    use super::*;

    fn sample_signature() -> EffectSignature {
        EffectSignature {
            check_kind: "clippy".into(),
            outcome: "fail".into(),
            severity: "warning".into(),
            message_class: "unused_variable".into(),
            line_offset_from_edit: Some(2),
        }
    }

    #[test]
    fn effect_signature_hash_is_deterministic_and_sensitive_to_fields() {
        let baseline = sample_signature();
        let same_hash = effect_signature_hash(&baseline);
        assert_eq!(same_hash, effect_signature_hash(&baseline));

        let mut changed = baseline.clone();
        changed.message_class = "different".into();
        assert_ne!(same_hash, effect_signature_hash(&changed));
    }

    #[test]
    fn located_effect_round_trips_through_json() {
        let located = LocatedEffect {
            file: Some(PathBuf::from("src/lib.rs")),
            line: Some(12),
            col: Some(4),
            message: "unused variable".into(),
            sig: sample_signature(),
        };

        let encoded = serde_json::to_string(&located).unwrap();
        let decoded: LocatedEffect = serde_json::from_str(&encoded).unwrap();
        assert_eq!(decoded, located);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_signature() -> EffectSignature {
        EffectSignature {
            check_kind: "clippy".into(),
            outcome: "warning".into(),
            severity: "medium".into(),
            message_class: "lint".into(),
            line_offset_from_edit: Some(2),
        }
    }

    #[test]
    fn located_effect_round_trips_through_json() {
        let effect = LocatedEffect {
            file: Some("src/lib.rs".into()),
            line: Some(12),
            col: Some(4),
            message: "unused variable".into(),
            sig: sample_signature(),
        };

        let encoded = serde_json::to_string(&effect).unwrap();
        let decoded: LocatedEffect = serde_json::from_str(&encoded).unwrap();

        assert_eq!(decoded, effect);
    }

    #[test]
    fn hash_is_stable_for_identical_signatures() {
        let left = sample_signature();
        let right = sample_signature();

        assert_eq!(effect_signature_hash(&left), effect_signature_hash(&right));
    }

    #[test]
    fn hash_changes_when_signature_changes() {
        let baseline = sample_signature();
        let mut changed = sample_signature();
        changed.message_class = "type_mismatch".into();

        assert_ne!(
            effect_signature_hash(&baseline),
            effect_signature_hash(&changed)
        );
    }
}
