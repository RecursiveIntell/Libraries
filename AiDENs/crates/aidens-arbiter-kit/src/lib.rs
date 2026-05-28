//! Thin arbiter facade over canonical contradiction artifacts.

pub mod canonical_stack {
    pub use semantic_memory_forge::{ContradictionWitnessV1, CONTRADICTION_WITNESS_V1_SCHEMA};
    pub use stack_ids::{ClaimId, ContradictionWitnessId};
}

pub use canonical_stack::ContradictionWitnessV1 as CanonicalContradictionWitnessV1;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CanonicalContradictionArbiter;

impl CanonicalContradictionArbiter {
    pub fn validate_witness(
        &self,
        witness: &CanonicalContradictionWitnessV1,
    ) -> Result<(), String> {
        witness.validate()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_canonical_contradiction_witness() {
        let witness = CanonicalContradictionWitnessV1 {
            schema_version: canonical_stack::CONTRADICTION_WITNESS_V1_SCHEMA.into(),
            contradiction_witness_id: canonical_stack::ContradictionWitnessId::new(
                "contradiction-witness:arbiter",
            ),
            claim_id: canonical_stack::ClaimId::new("claim:arbiter"),
            conflicting_token_ids: vec!["support-token:left".into(), "support-token:right".into()],
            summary: Some("canonical witness supplied by semantic-memory-forge".into()),
        };

        assert!(CanonicalContradictionArbiter
            .validate_witness(&witness)
            .is_ok());
    }
}
