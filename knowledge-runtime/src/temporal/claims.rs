use crate::ids::EntityId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A temporal claim: an assertion about an entity that holds during a time interval.
///
/// Claims are derived projections — they do NOT own source truth. They are
/// assembled from facts, episodes, and document chunks found via
/// `semantic-memory`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporalClaim {
    /// The entity this claim is about.
    pub entity_id: EntityId,
    /// The assertion text.
    pub claim: String,
    /// When the claim became valid (inclusive).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub valid_from: Option<DateTime<Utc>>,
    /// When the claim ceased to be valid (exclusive).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub valid_until: Option<DateTime<Utc>>,
    /// Confidence in the claim's validity (0.0 to 1.0).
    pub confidence: f32,
    /// Source IDs from `semantic-memory` backing this claim.
    pub source_ids: Vec<String>,
}

impl TemporalClaim {
    /// Check whether this claim is active at a given point in time.
    pub fn active_at(&self, when: &DateTime<Utc>) -> bool {
        let after_start = match self.valid_from.as_ref() {
            Some(from) => when >= from,
            None => true,
        };
        let before_end = match self.valid_until.as_ref() {
            Some(until) => when < until,
            None => true,
        };
        after_start && before_end
    }

    /// Check whether this claim overlaps with another claim's interval.
    pub fn overlaps(&self, other: &TemporalClaim) -> bool {
        // Two intervals [a, b) and [c, d) overlap iff a < d and c < b
        // (with None treated as unbounded)
        let a_before_d = match other.valid_until.as_ref() {
            Some(d) => match self.valid_from.as_ref() {
                Some(a) => a < d,
                None => true,
            },
            None => true,
        };
        let c_before_b = match self.valid_until.as_ref() {
            Some(b) => match other.valid_from.as_ref() {
                Some(c) => c < b,
                None => true,
            },
            None => true,
        };
        a_before_d && c_before_b
    }
}

/// Contradiction status between two temporal claims about the same entity.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TemporalContradictionStatus {
    /// No contradiction detected.
    None,
    /// Claims overlap in time and may contradict each other.
    /// Resolution is deferred to the caller / future Forge layer.
    PossibleContradiction {
        /// Human-readable description of the potential conflict.
        description: String,
    },
}

/// Check two claims for temporal contradiction.
///
/// This is a structural check only — semantic contradiction detection
/// (whether two claims actually conflict in meaning) is deferred to
/// the future Forge causal layer.
pub fn check_contradiction(a: &TemporalClaim, b: &TemporalClaim) -> TemporalContradictionStatus {
    if a.entity_id != b.entity_id {
        return TemporalContradictionStatus::None;
    }
    if !a.overlaps(b) {
        return TemporalContradictionStatus::None;
    }
    TemporalContradictionStatus::PossibleContradiction {
        description: format!(
            "overlapping claims on entity {}: {:?} vs {:?}",
            a.entity_id, a.claim, b.claim
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_claim(
        entity: &str,
        claim: &str,
        from: Option<&str>,
        until: Option<&str>,
    ) -> TemporalClaim {
        TemporalClaim {
            entity_id: EntityId::new(entity),
            claim: claim.to_string(),
            valid_from: from.map(|s| s.parse::<DateTime<Utc>>().unwrap()),
            valid_until: until.map(|s| s.parse::<DateTime<Utc>>().unwrap()),
            confidence: 0.9,
            source_ids: vec![],
        }
    }

    #[test]
    fn active_at_unbounded() {
        let claim = make_claim("e1", "always true", None, None);
        let now = Utc::now();
        assert!(claim.active_at(&now));
    }

    #[test]
    fn active_at_bounded() {
        let claim = make_claim(
            "e1",
            "in range",
            Some("2025-01-01T00:00:00Z"),
            Some("2025-12-31T23:59:59Z"),
        );
        let mid: DateTime<Utc> = "2025-06-15T00:00:00Z".parse().unwrap();
        let before: DateTime<Utc> = "2024-06-15T00:00:00Z".parse().unwrap();
        assert!(claim.active_at(&mid));
        assert!(!claim.active_at(&before));
    }

    #[test]
    fn overlapping_same_entity_is_contradiction() {
        let a = make_claim(
            "e1",
            "X is true",
            Some("2025-01-01T00:00:00Z"),
            Some("2025-06-01T00:00:00Z"),
        );
        let b = make_claim(
            "e1",
            "X is false",
            Some("2025-03-01T00:00:00Z"),
            Some("2025-09-01T00:00:00Z"),
        );
        assert!(matches!(
            check_contradiction(&a, &b),
            TemporalContradictionStatus::PossibleContradiction { .. }
        ));
    }

    #[test]
    fn non_overlapping_same_entity_no_contradiction() {
        let a = make_claim(
            "e1",
            "X is true",
            Some("2025-01-01T00:00:00Z"),
            Some("2025-03-01T00:00:00Z"),
        );
        let b = make_claim(
            "e1",
            "X is false",
            Some("2025-06-01T00:00:00Z"),
            Some("2025-09-01T00:00:00Z"),
        );
        assert_eq!(
            check_contradiction(&a, &b),
            TemporalContradictionStatus::None
        );
    }

    #[test]
    fn different_entity_no_contradiction() {
        let a = make_claim(
            "e1",
            "X",
            Some("2025-01-01T00:00:00Z"),
            Some("2025-12-01T00:00:00Z"),
        );
        let b = make_claim(
            "e2",
            "Y",
            Some("2025-01-01T00:00:00Z"),
            Some("2025-12-01T00:00:00Z"),
        );
        assert_eq!(
            check_contradiction(&a, &b),
            TemporalContradictionStatus::None
        );
    }
}
