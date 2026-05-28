#!/usr/bin/env bash
set -euo pipefail
MODE="fail"
if [[ "${1:-}" == "--warn" ]]; then MODE="warn"; shift; fi
ROOT="${1:-${AIDENS_REPO_ROOT:-$(pwd)}}"
cd "$ROOT"

patterns=(
  'pub struct ArtifactId'
  'struct ArtifactId'
  'type ArtifactId'
  'pub enum ReceiptKindV1'
  'pub struct ReceiptEnvelopeV1'
  'pub struct ReceiptStoreConfigV1'
  'pub struct ReceiptOutboxRowV1'
  'pub struct ToolInvocationReceiptV1'
  'pub struct RunReceiptV1'
  'pub struct AidensEvidenceDraftV1'
  'pub struct AidensClaimDraftV1'
  'pub struct AidensProjectionDraftV1'
  'pub struct AidensEpisodeDraftBundleV1'
  'pub struct AidensPromotionCheckDraftV1'
  'pub struct AidensRepairDraftV1'
  'pub enum GovernanceDispositionV1'
  'pub type GovernanceDispositionV1'
  'pub enum RiskBearingOutputCategoryV1'
  'pub enum RefutationOutcomeV1'
  'pub enum ContradictionStateV1'
  'pub enum KernelStopStateV1'
  'pub struct AidensKernelRunSummaryV1'
  'pub enum RiskClassV1'
  'pub struct EpisodeBundleV1'
  'pub struct ExecutionContextV1'
  'pub struct EvidenceRecordV1'
  'pub struct ClaimRecordV1'
  'pub struct BitemporalCoordinateV1'
  'pub struct RepairRecordV1'
  'pub struct LocalRepairCandidateV1'
  'pub struct VerificationPlanV1'
  'pub struct ClaimEvidenceBundleV1'
  'pub struct RefutationResultV1'
  'pub struct ContradictionFindingV1'
  'pub struct GovernanceDecisionV1'
  'pub struct PromotionReceiptV1'
  'pub struct KernelRunReceiptV1'
  'pub struct MemoryStore'
  'pub struct AppendOnlyMemoryStore'
)
allow='(^|/)(target|AIDENS_CODEX_REWRITE_PACK_20260428|docs|handoffs|passes|prompts|schemas)/|semantic-memory-forge/|semantic-memory/|stack-ids/|verification-control/|verification-policy/|verification-adjudication/|recursive-kernel-core/|kernel-execution/|kernel-oracles/|constraint-compiler/|llm-tool-runtime/'
found=0
for pat in "${patterns[@]}"; do
  while IFS= read -r line; do
    [[ -z "$line" ]] && continue
    if echo "$line" | grep -Eq "$allow"; then
      continue
    fi
    echo "SHADOW_SEMANTICS: $line" >&2
    found=1
  done < <(grep -RIn --include='*.rs' "$pat" . 2>/dev/null || true)
done

if [[ "$found" -eq 1 && "$MODE" == "fail" ]]; then
  echo "ERROR: shadow canonical semantics found outside allowed paths." >&2
  exit 1
fi
if [[ "$found" -eq 1 ]]; then
  echo "WARN: shadow canonical semantics found; expected before Phase 1, forbidden after." >&2
fi
