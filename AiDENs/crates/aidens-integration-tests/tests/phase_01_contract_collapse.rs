use std::any::TypeId;
use std::fs;
use std::path::{Path, PathBuf};

fn aidens_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("aidens workspace root")
        .to_path_buf()
}

#[test]
fn contract_owner_proof() {
    let root = aidens_root();
    let contracts =
        fs::read_to_string(root.join("crates/aidens-contracts/src/lib.rs")).expect("contracts src");
    let ledger = fs::read_to_string(root.join("COMPATIBILITY_LEDGER.md")).expect("compat ledger");

    let forbidden_defs = [
        format!("pub struct {}", "ArtifactId"),
        format!("pub enum {}", "ReceiptKindV1"),
        format!("pub struct {}", "ReceiptEnvelopeV1"),
        format!("pub struct {}", "ReceiptStoreConfigV1"),
        format!("pub struct {}", "ReceiptOutboxRowV1"),
        format!("pub struct {}", "ToolInvocationReceiptV1"),
        format!("pub struct {}", "RunReceiptV1"),
        format!("pub struct {}", "AidensEvidenceDraftV1"),
        format!("pub struct {}", "AidensClaimDraftV1"),
        format!("pub struct {}", "AidensProjectionDraftV1"),
        format!("pub struct {}", "AidensEpisodeDraftBundleV1"),
        format!("pub struct {}", "AidensPromotionCheckDraftV1"),
        format!("pub struct {}", "AidensRepairDraftV1"),
        format!("pub enum {}", "GovernanceDispositionV1"),
        format!("pub type {}", "GovernanceDispositionV1"),
        format!("pub enum {}", "RiskBearingOutputCategoryV1"),
        format!("pub enum {}", "RefutationOutcomeV1"),
        format!("pub enum {}", "ContradictionStateV1"),
        format!("pub enum {}", "KernelStopStateV1"),
        format!("pub struct {}", "AidensKernelRunSummaryV1"),
        format!("pub enum {}", "RiskClassV1"),
        format!("pub struct {}", "ExecutionContextV1"),
        format!("pub struct {}", "EvidenceRecordV1"),
        format!("pub struct {}", "ClaimRecordV1"),
        format!("pub struct {}", "BitemporalCoordinateV1"),
        format!("pub struct {}", "ProjectionRecordV1"),
        format!("pub struct {}", "EpisodeBundleV1"),
        format!("pub struct {}", "ClaimEvidenceBundleV1"),
        format!("pub struct {}", "RefutationResultV1"),
        format!("pub struct {}", "ContradictionFindingV1"),
        format!("pub struct {}", "GovernanceDecisionV1"),
        format!("pub struct {}", "PromotionReceiptV1"),
        format!("pub struct {}", "VerificationPlanV1"),
        format!("pub struct {}", "RepairRecordV1"),
        format!("pub struct {}", "LocalRepairCandidateV1"),
        format!("pub struct {}", "KernelRunReportV1"),
    ];
    for forbidden in forbidden_defs {
        assert!(
            !contracts.contains(&forbidden),
            "aidens-contracts still defines canonical-looking local type: {forbidden}"
        );
    }
    let ledger_has_rows = ledger.lines().any(|line| line.starts_with("| `"));
    let legacy_marker = format!("{}{}", "Legacy", "Aidens");
    let deprecated_attr = format!("#[{}", "deprecated");
    assert!(
        !contracts.contains(&legacy_marker)
            && !contracts.contains(&deprecated_attr)
            && !ledger.contains(&legacy_marker)
            && !ledger_has_rows,
        "compatibility surfaces must not be retained"
    );

    assert_eq!(
        TypeId::of::<aidens_contracts::canonical_stack::ForgeExecutionContextV1>(),
        TypeId::of::<semantic_memory_forge::ExecutionContextV1>()
    );
    assert_eq!(
        TypeId::of::<aidens_contracts::canonical_stack::ForgeEpisodeBundleV1>(),
        TypeId::of::<semantic_memory_forge::EpisodeBundleV1>()
    );
    assert_eq!(
        TypeId::of::<aidens_contracts::canonical_stack::CanonicalVerificationPlan>(),
        TypeId::of::<verification_control::CheckPlan>()
    );
    assert_eq!(
        TypeId::of::<aidens_contracts::canonical_stack::CanonicalBoundaryRepairRecord>(),
        TypeId::of::<verification_control::BoundaryRepairRecord>()
    );
    assert_eq!(
        TypeId::of::<aidens_contracts::canonical_stack::CanonicalKernelRun>(),
        TypeId::of::<recursive_kernel_core::KernelRun>()
    );
    assert_eq!(
        TypeId::of::<aidens_contracts::canonical_stack::CanonicalKernelExecutionReport>(),
        TypeId::of::<kernel_execution::ExecutionReport>()
    );
    assert_eq!(
        TypeId::of::<aidens_contracts::canonical_stack::CanonicalOracleAssessment>(),
        TypeId::of::<kernel_oracles::OracleAssessment>()
    );
}

#[test]
fn canonical_id_roundtrip() {
    assert_eq!(
        TypeId::of::<aidens_contracts::ArtifactId>(),
        TypeId::of::<stack_ids::ArtifactId>()
    );
    assert_eq!(
        TypeId::of::<aidens_contracts::StackArtifactId>(),
        TypeId::of::<stack_ids::ArtifactId>()
    );

    let id = aidens_contracts::ArtifactId::new("artifact:phase-01");
    let stack_id: stack_ids::ArtifactId = id.clone();
    let contracts_id: aidens_contracts::ArtifactId = stack_id.clone();

    assert_eq!(stack_id.as_str(), "artifact:phase-01");
    assert_eq!(contracts_id.as_str(), "artifact:phase-01");

    let json = serde_json::to_string(&contracts_id).expect("serialize canonical id");
    let decoded: stack_ids::ArtifactId =
        serde_json::from_str(&json).expect("deserialize canonical id");
    assert_eq!(decoded, stack_id);
}
