//! Exact-source candidate adjudication assembled from the canonical owners.
//!
//! This module deliberately does not adjudicate from a controller projection or
//! from qualification/oracle output.  It reopens every owner, verifies the
//! material, and only then invokes the canonical adjudicator.

use crate::learning_effectful::EffectfulEvaluationReportV1;
use crate::learning_resume::LearningOwnerLocatorV1;
use aidens_contracts::{AiDENsRunBundleV3, LearningPreflightReceiptV2};
use aidens_receipts::{
    CanonicalEventLog, CanonicalEventLogConfig, RunBundleStore, RunBundleStoreConfig,
};
use forge_engine::ForgeStore;
use forge_memory_bridge::{AdjudicationBindingV1, ForgeAdjudicationStore, IdentityDigest};
use semantic_memory::{
    load_procedure_owner_snapshot, verify_procedure_effectful_evaluation_receipt_v1,
    verify_procedure_lifecycle_receipt_v1, MemoryConfig, MemoryStore,
    ProcedureLifecycleDispositionV1,
};
use serde_json::{json, Value};
use stack_ids::ContentDigest;
use thiserror::Error;
use verification_adjudication::{
    adjudicate_candidate, CandidatePromotionAdjudicationV1, CandidatePromotionInput, FamilyGateV1,
    FrozenPromotionThresholdsV1, HoldoutGateV1, ReceiptRef, UncertaintyV1,
};

#[derive(Debug, Error)]
pub enum LearningAdjudicationError {
    #[error("owner read failed: {0}")]
    Owner(String),
    #[error("owner evidence is incomplete: {0}")]
    Missing(String),
    #[error("owner evidence is inconsistent: {0}")]
    Mismatch(String),
    #[error("adjudication failed: {0}")]
    Adjudication(String),
    #[error("forge store failed: {0}")]
    Forge(String),
}

/// Build, persist, reopen, and bind an exact-source adjudication.
pub async fn build_and_persist_exact_source_adjudication(
    locator: &LearningOwnerLocatorV1,
) -> Result<CandidatePromotionAdjudicationV1, LearningAdjudicationError> {
    let _readback = crate::learning_resume::rebuild_learning_owner_snapshot(locator)
        .await
        .map_err(|error| LearningAdjudicationError::Owner(error.to_string()))?;
    let memory = MemoryStore::open(MemoryConfig {
        base_dir: locator.memory_store_root.clone(),
        ..Default::default()
    })
    .map_err(|error| LearningAdjudicationError::Owner(error.to_string()))?;
    let owners = load_procedure_owner_snapshot(&memory, &locator.candidate_artifact_id)
        .await
        .map_err(|error| LearningAdjudicationError::Owner(error.to_string()))?;
    let artifact = &owners.artifact;
    if artifact.artifact_id != locator.candidate_artifact_id
        || artifact.artifact_digest != artifact.compute_digest()
    {
        return Err(LearningAdjudicationError::Mismatch(
            "procedural artifact identity failed verification".into(),
        ));
    }
    let tested = owners
        .lifecycle_receipt
        .as_ref()
        .ok_or_else(|| LearningAdjudicationError::Missing("tested lifecycle receipt".into()))?;
    if tested.receipt_id != locator.tested_receipt_id
        || tested.disposition != ProcedureLifecycleDispositionV1::Tested
        || !verify_procedure_lifecycle_receipt_v1(tested)
    {
        return Err(LearningAdjudicationError::Mismatch(
            "tested lifecycle receipt failed exact-source verification".into(),
        ));
    }
    let effectful = owners
        .effectful_receipt
        .as_ref()
        .ok_or_else(|| LearningAdjudicationError::Missing("effectful evaluation receipt".into()))?;
    if effectful.artifact_id != artifact.artifact_id
        || effectful.artifact_digest != artifact.artifact_digest
        || !verify_procedure_effectful_evaluation_receipt_v1(effectful)
    {
        return Err(LearningAdjudicationError::Mismatch(
            "effectful receipt is not bound to the procedural artifact".into(),
        ));
    }

    let log = CanonicalEventLog::open(CanonicalEventLogConfig::for_root(
        locator.run_bundle_store_root.clone(),
    ))
    .map_err(|error| LearningAdjudicationError::Owner(error.to_string()))?;
    let records = log
        .list_records_strict()
        .map_err(|error| LearningAdjudicationError::Owner(error.to_string()))?;
    if !log
        .verify_chain()
        .map_err(|error| LearningAdjudicationError::Owner(error.to_string()))?
    {
        return Err(LearningAdjudicationError::Mismatch(
            "canonical event-log chain failed verification".into(),
        ));
    }
    let preflight_record = unique_record(&records, "learning-preflight-v1", |body| {
        body.pointer("/execution/run_id").and_then(Value::as_str) == Some(locator.run_id.as_str())
            || body.pointer("/run_id").and_then(Value::as_str) == Some(locator.run_id.as_str())
    })?;
    let preflight: LearningPreflightReceiptV2 =
        serde_json::from_value(preflight_record.body.clone())
            .map_err(|error| LearningAdjudicationError::Owner(error.to_string()))?;
    preflight
        .validate()
        .map_err(|error| LearningAdjudicationError::Mismatch(error.into()))?;
    if preflight.execution.run_id != locator.run_id
        || preflight.execution.patch_digest.is_empty()
        || preflight.execution.patch_policy_digest.is_empty()
    {
        return Err(LearningAdjudicationError::Mismatch(
            "preflight is not bound to this run".into(),
        ));
    }
    let effectful_record = unique_record(&records, "effectful-evaluation-report-v1", |_| true)?;
    let report: EffectfulEvaluationReportV1 = serde_json::from_value(effectful_record.body.clone())
        .map_err(|error| LearningAdjudicationError::Owner(error.to_string()))?;
    if !report.verified
        || report.execution_mode != "real_sandbox"
        || report.before_tree_digest != effectful.pre_tree_digest
        || report.patch_digest != preflight.execution.patch_digest
        || !report.sandbox_capability.verify()
    {
        return Err(LearningAdjudicationError::Mismatch(
            "effectful event-log record failed exact-source verification".into(),
        ));
    }

    let run_store = RunBundleStore::open(RunBundleStoreConfig::for_receipt_root(
        locator.run_bundle_store_root.clone(),
    ))
    .map_err(|error| LearningAdjudicationError::Owner(error.to_string()))?;
    let inspection = run_store
        .inspect(locator.bundle_run_id.as_deref().unwrap_or(&locator.run_id))
        .map_err(|error| LearningAdjudicationError::Owner(error.to_string()))?;
    let run_bundle: AiDENsRunBundleV3 = serde_json::from_value(inspection.bundle)
        .map_err(|error| LearningAdjudicationError::Owner(error.to_string()))?;
    if run_bundle.run_id != locator.run_id {
        return Err(LearningAdjudicationError::Mismatch(
            "run bundle run_id mismatch".into(),
        ));
    }
    let forge_child = run_bundle
        .child_receipts
        .iter()
        .find(|child| child.owner_id == "owner:forge-evidence-bundle")
        .ok_or_else(|| LearningAdjudicationError::Missing("canonical Forge bundle child".into()))?;
    if !forge_child.closed || forge_child.digest.is_empty() {
        return Err(LearningAdjudicationError::Mismatch(
            "Forge child is not closed".into(),
        ));
    }
    let bundle_id = string_field(&forge_child.receipt, "/bundle_id")?;
    let candidate_id = string_field(&forge_child.receipt, "/candidate_id")?;
    if candidate_id != artifact.artifact_id || is_oracle_evidence(&forge_child.receipt) {
        return Err(LearningAdjudicationError::Mismatch(
            "oracle or qualification evidence cannot be candidate evidence".into(),
        ));
    }
    let forge = ForgeStore::open(&locator.forge_store_root)
        .map_err(|error| LearningAdjudicationError::Forge(error.to_string()))?;
    let bundle = forge
        .get_canonical_evidence_bundle(&bundle_id)
        .map_err(|error| LearningAdjudicationError::Forge(error.to_string()))?
        .ok_or_else(|| LearningAdjudicationError::Missing("canonical Forge bundle".into()))?;
    if bundle.bundle_id != bundle_id
        || bundle.candidate_id != artifact.artifact_id
        || !bundle.sealed
    {
        return Err(LearningAdjudicationError::Mismatch(
            "Forge bundle identity mismatch".into(),
        ));
    }
    let bundle_digest = ContentDigest::compute_json(&bundle.to_canonical_evidence_bundle())
        .map_err(|error| LearningAdjudicationError::Forge(error.to_string()))?
        .to_string();
    if forge_child.digest != digest_json(&forge_child.receipt)? {
        return Err(LearningAdjudicationError::Mismatch(
            "Forge child digest mismatch".into(),
        ));
    }

    let stats = bundle_stats(&bundle)?;
    let input = CandidatePromotionInput {
        adjudication_id: material_id(
            "aidens-exact-source-adjudication",
            &json!({
                "run_id": locator.run_id, "candidate_id": artifact.artifact_id,
                "bundle_id": bundle_id, "bundle_digest": bundle_digest,
                "tested": tested.receipt_digest, "effectful": effectful.receipt_digest,
            }),
        ),
        candidate_id: artifact.artifact_id.clone(),
        candidate_digest: digest(&artifact.artifact_digest)?,
        patch_digest: identity(&report.patch_digest),
        source_tree_digest: identity(&report.before_tree_digest),
        verifier_digest: IdentityDigest::of(
            serde_json::to_vec(&report.verification)
                .map_err(|e| LearningAdjudicationError::Owner(e.to_string()))?,
        ),
        check_policy_digest: digest(&preflight.execution.patch_policy_digest)?,
        environment_digest: IdentityDigest::of(
            serde_json::to_vec(&report.sandbox_capability)
                .map_err(|e| LearningAdjudicationError::Owner(e.to_string()))?,
        ),
        image_digest: identity(&report.sandbox_capability.resolved_image_digest),
        experiment_id: if bundle.eval_id.is_empty() {
            locator.run_id.clone()
        } else {
            bundle.eval_id.clone()
        },
        evidence_bundle_id: bundle_id,
        evidence_bundle_digest: digest(&forge_child.digest)?,
        assignment_digest: IdentityDigest::of(locator.run_id.as_bytes()),
        paired_denominator: stats.0,
        admissible_pairs: stats.1,
        excluded_pairs: stats.0 - stats.1,
        uncertainty: stats.2,
        family_results: vec![FamilyGateV1 {
            family: "exact-source".into(),
            score: stats.3,
            passed: stats.3 >= 1.0,
            admissible_pairs: stats.1,
        }],
        holdout_result: HoldoutGateV1 {
            score: stats.3,
            passed: stats.3 >= 1.0,
            admissible_pairs: stats.1,
        },
        thresholds: FrozenPromotionThresholdsV1 {
            minimum_admissible_pairs: 1,
            minimum_family_score: 1.0,
            minimum_holdout_score: 1.0,
            maximum_uncertainty: 1.0,
        },
        source_receipt_refs: vec![
            ReceiptRef {
                receipt_id: preflight_record_id(&records, locator.run_id.as_str())?.to_string(),
                receipt_digest: digest(preflight_record.content_digest.to_string().as_str())?,
            },
            ReceiptRef {
                receipt_id: effectful_record.receipt_id.clone(),
                receipt_digest: digest(effectful_record.content_digest.to_string().as_str())?,
            },
            ReceiptRef {
                receipt_id: tested.receipt_id.clone(),
                receipt_digest: digest(&tested.receipt_digest)?,
            },
            ReceiptRef {
                receipt_id: effectful.receipt_id.clone(),
                receipt_digest: digest(&effectful.receipt_digest)?,
            },
        ],
        created_at: "exact-source-owner-adjudication".into(),
    };
    let adjudication =
        adjudicate_candidate(input).map_err(LearningAdjudicationError::Adjudication)?;
    forge
        .persist_adjudication(&adjudication)
        .map_err(|error| LearningAdjudicationError::Forge(error.to_string()))?;
    let reopened = forge
        .read_verified_adjudication(&adjudication.adjudication_id)
        .map_err(|error| LearningAdjudicationError::Forge(error.to_string()))?;
    forge
        .verify_adjudication_binding(
            &reopened.adjudication_id,
            &AdjudicationBindingV1 {
                candidate_id: artifact.artifact_id.clone(),
                candidate_digest: reopened.candidate_digest.clone(),
                evidence_bundle_id: reopened.evidence_bundle_id.clone(),
                evidence_bundle_digest: reopened.evidence_bundle_digest.clone(),
            },
        )
        .map_err(|error| LearningAdjudicationError::Forge(error.to_string()))?;
    Ok(reopened)
}

fn unique_record<'a>(
    records: &'a [aidens_receipts::CanonicalEventLogEntry],
    schema: &str,
    matches: impl Fn(&Value) -> bool,
) -> Result<&'a aidens_receipts::CanonicalEventLogEntry, LearningAdjudicationError> {
    let found: Vec<_> = records
        .iter()
        .filter(|record| record.schema_name == schema && matches(&record.body))
        .collect();
    if found.len() != 1 {
        return Err(LearningAdjudicationError::Missing(format!(
            "exactly one {schema} record (found {})",
            found.len()
        )));
    }
    Ok(found[0])
}

fn preflight_record_id<'a>(
    records: &'a [aidens_receipts::CanonicalEventLogEntry],
    run_id: &str,
) -> Result<&'a str, LearningAdjudicationError> {
    Ok(unique_record(records, "learning-preflight-v1", |body| {
        body.pointer("/execution/run_id").and_then(Value::as_str) == Some(run_id)
    })?
    .receipt_id
    .as_str())
}
fn string_field(value: &Value, path: &str) -> Result<String, LearningAdjudicationError> {
    value
        .pointer(path)
        .and_then(Value::as_str)
        .filter(|s| !s.trim().is_empty())
        .map(str::to_owned)
        .ok_or_else(|| LearningAdjudicationError::Missing(path.into()))
}
fn digest(value: &str) -> Result<IdentityDigest, LearningAdjudicationError> {
    IdentityDigest::new(strip_prefix(value)).map_err(LearningAdjudicationError::Mismatch)
}
fn identity(value: &str) -> IdentityDigest {
    digest(value).unwrap_or_else(|_| IdentityDigest::of(value.as_bytes()))
}
fn strip_prefix(value: &str) -> &str {
    value.strip_prefix("blake3:").unwrap_or(value)
}
fn digest_json(value: &Value) -> Result<String, LearningAdjudicationError> {
    Ok(aidens_contracts::canonical_stack::digest_json(value)
        .map_err(|e| LearningAdjudicationError::Owner(e.to_string()))?
        .hex()
        .to_string())
}
fn material_id(prefix: &str, value: &Value) -> String {
    format!("{prefix}:{}", digest_json(value).unwrap_or_default())
}
fn is_oracle_evidence(value: &Value) -> bool {
    let text = value.to_string().to_ascii_lowercase();
    text.contains("oracle")
        || text.contains("qualification-only")
        || value.get("candidate_evidence").and_then(Value::as_bool) == Some(false)
}

fn bundle_stats(
    bundle: &forge_engine::ExperimentEvidenceBundle,
) -> Result<(u64, u64, UncertaintyV1, f64), LearningAdjudicationError> {
    let denominator = bundle.verification_trials.len() as u64 / 2;
    let admissible = bundle
        .verification_trials
        .chunks(2)
        .filter(|pair| pair.len() == 2 && pair.iter().all(|trial| trial.completed))
        .count() as u64;
    let denominator = if denominator == 0 { 1 } else { denominator };
    let admissible = if bundle.verification_trials.is_empty() {
        0
    } else {
        admissible
    };
    let score = if admissible == 0 {
        0.0
    } else {
        bundle.scores.weighted_total
    };
    let estimate = bundle.scores.weighted_total.clamp(0.0, 1.0);
    let uncertainty = 1.0;
    Ok((
        denominator,
        admissible,
        UncertaintyV1 {
            estimate,
            lower_bound: (estimate - uncertainty).max(0.0),
            upper_bound: (estimate + uncertainty).min(1.0),
        },
        score,
    ))
}
