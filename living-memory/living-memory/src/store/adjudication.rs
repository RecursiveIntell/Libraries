use super::ForgeStore;
use chrono::Utc;
use forge_memory_bridge::{
    AdjudicationBindingV1, BridgeError, ForgeAdjudicationPersistenceReceiptV1,
    ForgeAdjudicationStore, FORGE_ADJUDICATION_PERSISTENCE_RECEIPT_V1_SCHEMA,
};
use rusqlite::OptionalExtension;
use verification_adjudication::CandidatePromotionAdjudicationV1;

fn persistence_error(error: impl std::fmt::Display) -> BridgeError {
    BridgeError::AdjudicationPersistence(error.to_string())
}
fn receipt(
    a: &CandidatePromotionAdjudicationV1,
    at: String,
) -> ForgeAdjudicationPersistenceReceiptV1 {
    ForgeAdjudicationPersistenceReceiptV1 {
        schema_version: FORGE_ADJUDICATION_PERSISTENCE_RECEIPT_V1_SCHEMA.into(),
        adjudication_id: a.adjudication_id.clone(),
        adjudication_digest: a.adjudication_digest.clone(),
        owner: "forge-engine/ForgeStore".into(),
        persisted_at: at,
    }
}

impl ForgeAdjudicationStore for ForgeStore {
    fn persist_adjudication(
        &self,
        a: &CandidatePromotionAdjudicationV1,
    ) -> Result<ForgeAdjudicationPersistenceReceiptV1, BridgeError> {
        a.validate().map_err(BridgeError::AdjudicationValidation)?;
        let json = serde_json::to_string(a).map_err(persistence_error)?;
        let now = Utc::now().to_rfc3339();
        self.with_transaction(|tx| {
            let existing: Option<String> = tx.query_row("SELECT adjudication_digest FROM candidate_promotion_adjudications WHERE adjudication_id = ?1", [&a.adjudication_id], |row| row.get(0)).optional()?;
            match existing {
                Some(digest) if digest != a.adjudication_digest.as_str() => Err(crate::error::ForgeError::Other("adjudication conflict".into())),
                Some(_) => Ok(()),
                None => { tx.execute("INSERT INTO candidate_promotion_adjudications (adjudication_id, adjudication_digest, candidate_id, candidate_digest, evidence_bundle_id, evidence_bundle_digest, canonical_json, created_at) VALUES (?1,?2,?3,?4,?5,?6,?7,?8)", rusqlite::params![a.adjudication_id, a.adjudication_digest.as_str(), a.candidate_id, a.candidate_digest.as_str(), a.evidence_bundle_id, a.evidence_bundle_digest.as_str(), json, now])?; Ok(()) }
            }
        }).map_err(|e| match e { crate::error::ForgeError::Other(_) => BridgeError::AdjudicationConflict { adjudication_id: a.adjudication_id.clone() }, other => persistence_error(other) })?;
        Ok(receipt(a, now))
    }

    fn read_verified_adjudication(
        &self,
        id: &str,
    ) -> Result<CandidatePromotionAdjudicationV1, BridgeError> {
        let (expected, json): (String, String) = self.with_conn(|conn| conn.query_row("SELECT adjudication_digest, canonical_json FROM candidate_promotion_adjudications WHERE adjudication_id = ?1", [id], |row| Ok((row.get(0)?, row.get(1)?))).optional()?.ok_or_else(|| crate::error::ForgeError::NotFound(id.into()))).map_err(|e| match e { crate::error::ForgeError::NotFound(_) => BridgeError::AdjudicationNotFound(id.into()), other => persistence_error(other) })?;
        let a: CandidatePromotionAdjudicationV1 =
            serde_json::from_str(&json).map_err(|_| BridgeError::AdjudicationTampered {
                expected: expected.clone(),
                actual: "invalid-json".into(),
            })?;
        let actual = a
            .canonical_digest()
            .map_err(BridgeError::AdjudicationValidation)?
            .as_str()
            .to_owned();
        if actual != expected || a.adjudication_id != id {
            return Err(BridgeError::AdjudicationTampered { expected, actual });
        }
        a.validate().map_err(BridgeError::AdjudicationValidation)?;
        Ok(a)
    }

    fn verify_adjudication_binding(
        &self,
        id: &str,
        expected: &AdjudicationBindingV1,
    ) -> Result<ForgeAdjudicationPersistenceReceiptV1, BridgeError> {
        let a = self.read_verified_adjudication(id)?;
        if a.candidate_id != expected.candidate_id
            || a.candidate_digest != expected.candidate_digest
            || a.evidence_bundle_id != expected.evidence_bundle_id
            || a.evidence_bundle_digest != expected.evidence_bundle_digest
        {
            return Err(BridgeError::AdjudicationBindingMismatch(id.into()));
        }
        Ok(receipt(&a, a.created_at.clone()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use forge_memory_bridge::{AdjudicationBindingV1, ForgeAdjudicationStore};
    use verification_adjudication::{
        adjudicate_candidate, CandidatePromotionInput, FamilyGateV1, FrozenPromotionThresholdsV1,
        HoldoutGateV1, IdentityDigest, ReceiptRef, UncertaintyV1,
    };

    fn artifact() -> Result<CandidatePromotionAdjudicationV1, BridgeError> {
        let d = |s: &str| IdentityDigest::of(s);
        adjudicate_candidate(CandidatePromotionInput {
            adjudication_id: "adj-1".into(),
            candidate_id: "candidate-1".into(),
            candidate_digest: d("candidate"),
            patch_digest: d("patch"),
            source_tree_digest: d("tree"),
            verifier_digest: d("verifier"),
            check_policy_digest: d("policy"),
            environment_digest: d("env"),
            image_digest: d("image"),
            experiment_id: "experiment-1".into(),
            evidence_bundle_id: "bundle-1".into(),
            evidence_bundle_digest: d("bundle"),
            assignment_digest: d("assignment"),
            paired_denominator: 2,
            admissible_pairs: 2,
            excluded_pairs: 0,
            uncertainty: UncertaintyV1 {
                estimate: 0.9,
                lower_bound: 0.8,
                upper_bound: 1.0,
            },
            family_results: vec![FamilyGateV1 {
                family: "family".into(),
                score: 1.0,
                passed: true,
                admissible_pairs: 2,
            }],
            holdout_result: HoldoutGateV1 {
                score: 1.0,
                passed: true,
                admissible_pairs: 2,
            },
            thresholds: FrozenPromotionThresholdsV1 {
                minimum_admissible_pairs: 1,
                minimum_family_score: 0.5,
                minimum_holdout_score: 0.5,
                maximum_uncertainty: 0.5,
            },
            source_receipt_refs: vec![ReceiptRef {
                receipt_id: "receipt".into(),
                receipt_digest: d("receipt"),
            }],
            created_at: "2026-07-19T00:00:00Z".into(),
        })
        .map_err(BridgeError::AdjudicationValidation)
    }

    #[test]
    fn persistence_is_idempotent_and_binding_is_verified() -> Result<(), BridgeError> {
        let dir = tempfile::tempdir().unwrap();
        let store = ForgeStore::open(&dir.path().join("forge.db")).unwrap();
        let a = artifact()?;
        let first = store.persist_adjudication(&a).unwrap();
        let second = store.persist_adjudication(&a).unwrap();
        assert_eq!(first.adjudication_digest, second.adjudication_digest);
        let read = store.read_verified_adjudication("adj-1").unwrap();
        assert_eq!(read, a);
        store
            .verify_adjudication_binding(
                "adj-1",
                &AdjudicationBindingV1 {
                    candidate_id: a.candidate_id.clone(),
                    candidate_digest: a.candidate_digest.clone(),
                    evidence_bundle_id: a.evidence_bundle_id.clone(),
                    evidence_bundle_digest: a.evidence_bundle_digest.clone(),
                },
            )
            .unwrap();
        Ok(())
    }

    #[test]
    fn conflicting_retry_and_tampered_readback_fail_closed() -> Result<(), BridgeError> {
        let dir = tempfile::tempdir().unwrap();
        let store = ForgeStore::open(&dir.path().join("forge.db")).unwrap();
        let a = artifact()?;
        store.persist_adjudication(&a).unwrap();
        let mut conflict = a.clone();
        conflict.adjudication_id = "adj-1".into();
        conflict.candidate_id = "other".into();
        conflict.adjudication_digest = conflict
            .canonical_digest()
            .map_err(BridgeError::AdjudicationValidation)?;
        assert!(matches!(
            store.persist_adjudication(&conflict),
            Err(BridgeError::AdjudicationConflict { .. })
        ));
        store.with_conn(|conn| { conn.execute("UPDATE candidate_promotion_adjudications SET canonical_json = ?1 WHERE adjudication_id = 'adj-1'", ["{}"])?; Ok(()) }).unwrap();
        assert!(matches!(
            store.read_verified_adjudication("adj-1"),
            Err(BridgeError::AdjudicationTampered { .. })
        ));
        Ok(())
    }
}
