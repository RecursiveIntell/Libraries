//! Canonical boundary compiler adapter.
//!
//! Bridges the standalone `boundary-compiler-core` microkernel into the
//! `aidens-contracts` / `aidens-boundary-kit` world. Every digest and receipt
//! type is converted at this boundary so downstream code sees only canonical
//! `aidens-contracts` types.

use aidens_contracts::{
    display_only_unstable_id, ArtifactId, ArtifactKindV1, BoundaryCompileOutcomeV1,
    BoundaryCompileRequestV1, CanonicalBackpointerV1, DisplayDigestV1, DuplicateKeyFindingV1,
    JsonBoundaryRepairDisplayReportV1,
};
use boundary_compiler_core as bcc;

/// Convert a bcc `DigestHex` (plain SHA-256 hex string) into the canonical
/// `DisplayDigestV1` used by `aidens-contracts` receipts.
fn digest_to_display(opt: Option<String>) -> Option<DisplayDigestV1> {
    opt.map(DisplayDigestV1::from_hex)
}

/// Build a bcc profile from a canonical request.
fn request_to_profile(request: &BoundaryCompileRequestV1) -> bcc::BoundaryCompilerProfileV1 {
    let mut profile = bcc::BoundaryCompilerProfileV1::strict_json_default();
    if request.schema.is_some() {
        profile.schema_id = Some(request.schema_dialect.clone());
    }
    profile.treatment_critical_paths = request.treatment_critical_fields.clone();
    profile
}

/// Map a bcc `BoundaryCompileResultV1` into the canonical
/// `BoundaryCompileOutcomeV1`.
fn result_to_outcome(
    request_id: ArtifactId,
    result: bcc::BoundaryCompileResultV1,
) -> BoundaryCompileOutcomeV1 {
    let accepted = matches!(
        result.decision,
        bcc::BoundaryDecisionV1::Accept | bcc::BoundaryDecisionV1::RepairedAccept
    );
    let degraded = !matches!(result.decision, bcc::BoundaryDecisionV1::Accept);

    let mut reason_codes = Vec::new();
    if degraded {
        reason_codes.push(format!("{:?}", result.decision));
    }
    for err in &result.errors {
        reason_codes.push(format!("{:?}:{}", err.kind, err.message));
    }

    let display_digest = digest_to_display(result.canonical_digest.clone());

    let duplicate_key_findings: Vec<DuplicateKeyFindingV1> = result
        .parse_receipt
        .errors
        .iter()
        .filter(|e| matches!(e.kind, bcc::BoundaryErrorKind::DuplicateKey))
        .map(|e| {
            DuplicateKeyFindingV1::new(
                e.path.clone().unwrap_or_default(),
                e.message.clone(),
                None,
                None,
            )
        })
        .collect();

    let repair_receipt = result
        .repair_receipt
        .map(|rr| JsonBoundaryRepairDisplayReportV1 {
            receipt_id: display_only_unstable_id("json-repair"),
            kind: ArtifactKindV1::BoundaryRepair,
            changed: true,
            repair_kind: rr.repair_operator,
            degraded: true,
            before_raw_digest: Some(rr.before_digest.clone()),
            after_raw_digest: Some(rr.after_digest.clone()),
            before_display_digest: Some(DisplayDigestV1::from_hex(rr.before_digest)),
            after_display_digest: Some(DisplayDigestV1::from_hex(rr.after_digest)),
            treatment_critical_fields: rr.changed_paths.clone(),
            treatment_integrity_warnings: Vec::new(),
            hard_failed: false,
            warnings: Vec::new(),
            reason_codes: vec!["boundary-repair".into()],
            canonical_repair_record_ids: Vec::new(),
            canonical_backpointers: vec![CanonicalBackpointerV1::owner_type(
                "verification-control",
                "BoundaryRepairRecord",
                "canonical-boundary-repair-owner",
            )],
        });

    BoundaryCompileOutcomeV1 {
        outcome_id: display_only_unstable_id("boundary-compile-outcome"),
        request_id,
        accepted,
        degraded,
        value: result.value,
        display_digest,
        duplicate_key_findings,
        schema_validation: None,
        repair_receipt,
        reason_codes,
        compiled_at: chrono::Utc::now(),
    }
}

/// Delegating boundary compile: strict parse path via the canonical bcc.
///
/// If bcc accepts, the result is a canonical `BoundaryCompileOutcomeV1`.
/// If bcc rejects (malformed JSON, duplicate keys, resource ceiling),
/// the caller should fall back to `aidens-boundary-kit`'s repair path
/// (`compile_with_repair`).
pub fn canonical_compile_json_boundary(
    request: BoundaryCompileRequestV1,
) -> BoundaryCompileOutcomeV1 {
    let profile = request_to_profile(&request);
    let raw = request.input.as_bytes();

    let bcc_result = bcc::compile_json_boundary(
        &profile,
        raw,
        request.schema.as_ref(),
        &profile.treatment_critical_paths,
    );

    result_to_outcome(request.request_id, bcc_result)
}
