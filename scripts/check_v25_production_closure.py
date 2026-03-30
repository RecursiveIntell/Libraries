#!/usr/bin/env python3
from __future__ import annotations

from pathlib import Path
import sys

ROOT = Path(__file__).resolve().parent.parent
errors: list[str] = []

def require_file(rel: str) -> None:
    if not (ROOT / rel).exists():
        errors.append(f'missing required file: {rel}')


def require_markers(rel: str, markers: list[str]) -> None:
    path = ROOT / rel
    if not path.exists():
        errors.append(f'missing required file: {rel}')
        return
    text = path.read_text(encoding='utf-8')
    for marker in markers:
        if marker not in text:
            errors.append(f'{rel} missing marker: {marker}')


def require_absent_markers(rel: str, markers: list[str]) -> None:
    path = ROOT / rel
    if not path.exists():
        errors.append(f'missing required file: {rel}')
        return
    text = path.read_text(encoding='utf-8')
    for marker in markers:
        if marker in text:
            errors.append(f'{rel} still contains forbidden marker: {marker}')

required_new_files = [
    'effect-runtime/src/v25.rs',
    'effect-runtime/tests/v25_citation_flow.rs',
    'verification-control/tests/v25_review_case_roundtrip.rs',
    'verification-control/tests/v25_citation_requirements.rs',
    'verification-policy/tests/v25_policy_citation_flow.rs',
    'verification-adjudication/tests/v25_adjudication_citation_flow.rs',
    'remote-oracle-admission/tests/v25_local_constitution_refs.rs',
    'federated-settlement/tests/v25_local_constitution_refs.rs',
]
for rel in required_new_files:
    require_file(rel)

require_markers('effect-runtime/Cargo.toml', ['stack-ids = { path = "../stack-ids" }'])
require_markers(
    'effect-runtime/src/effect.rs',
    [
        'EffectIntentId',
        'EffectWindowId',
        'EffectPreflightReportId',
        'EffectCommitDecisionId',
        'ApplicabilityContextId',
        'ProfileSetId',
        'CompositionReceiptId',
        'EffectiveConstitutionId',
        'CompiledObligationSetId',
        'ProfileExceptionBundleId',
    ],
)
require_markers(
    'effect-runtime/src/observation.rs',
    ['EffectExecutionReceiptId', 'EffectObservationBundleId', 'EffectiveConstitutionId', 'CompiledObligationSetId'],
)
require_markers(
    'effect-runtime/src/compensation.rs',
    ['CompensationPlanId', 'CompensationExecutionReceiptId', 'EffectiveConstitutionId', 'CompiledObligationSetId'],
)
require_absent_markers(
    'effect-runtime/src/effect.rs',
    [
        'pub effect_window_id: String',
        'pub effect_intent_id: String',
        'pub effect_preflight_report_id: String',
        'pub effect_commit_decision_id: String',
    ],
)
require_markers(
    'verification-control/src/lib.rs',
    [
        'ApplicabilityContextId',
        'ProfileSetId',
        'CompositionReceiptId',
        'EffectiveConstitutionId',
        'CompiledObligationSetId',
        'CompositionConflictSetId',
        'ProfileExceptionBundleId',
        'required_obligation_refs',
        'blocking_obligation_refs',
    ],
)
require_markers(
    'verification-policy/src/lib.rs',
    [
        'CompositionReceiptId',
        'EffectiveConstitutionId',
        'CompiledObligationSetId',
        'ProfileExceptionBundleId',
        'required_obligation_refs',
        'blocking_obligation_refs',
    ],
)
require_markers(
    'verification-adjudication/src/lib.rs',
    [
        'CompositionReceiptId',
        'EffectiveConstitutionId',
        'CompiledObligationSetId',
        'ProfileExceptionBundleId',
        'policy_decision_id',
    ],
)
require_markers(
    'remote-oracle-admission/src/lib.rs',
    ['ApplicabilityContextId', 'ProfileSetId', 'CompositionReceiptId', 'EffectiveConstitutionId', 'CompiledObligationSetId'],
)
require_markers(
    'federated-settlement/src/lib.rs',
    ['ApplicabilityContextId', 'ProfileSetId', 'CompositionReceiptId', 'EffectiveConstitutionId', 'CompiledObligationSetId'],
)
require_markers(
    'contract-schema-gen/src/lib.rs',
    [
        'write_schema::<verification_control::EffectReviewCaseV1>',
        'write_schema::<verification_control::EffectBlockReceiptV1>',
        'write_schema::<verification_control::DelegationReviewCaseV1>',
        'write_schema::<verification_control::ReleaseGateCaseV1>',
        'write_schema::<verification_control::ContinuityReviewCaseV1>',
        'write_schema::<verification_adjudication::EffectAdjudicationReceiptV1>',
        'write_schema::<verification_adjudication::ReleaseRollbackDecisionV1>',
    ],
)

for stem in [
    'control-receipt-v1',
    'effect-review-case-v1',
    'effect-block-receipt-v1',
    'delegation-review-case-v1',
    'release-gate-case-v1',
    'continuity-review-case-v1',
    'policy-decision-v1',
    'promotion-decision-v1',
    'refutation-decision-v1',
    'rollback-plan-v1',
    'effect-adjudication-receipt-v1',
    'release-rollback-decision-v1',
    'remote-slice-request-v1',
    'remote-slice-result-v1',
    'cross-runtime-replay-ticket-v1',
    'settlement-case-v1',
    'settlement-receipt-v1',
    'shared-replay-slice-v1',
    'shared-divergence-report-v1',
    'shared-view-downgrade-v1',
    'local-dissent-record-v1',
]:
    require_file(f'schemas/{stem}.schema.json')
    require_file(f'examples/{stem}.example.json')

require_markers('Makefile', ['v25-local-checks:', 'no-local-recomposition-check:', 'v25-production-pack-check:', 'v25-production-closure:'])
require_markers(
    '.github/workflows/ci.yml',
    ['check_v25_repo_truth.sh', 'check_v25_json_surface.py', 'check_no_local_recomposition.sh', 'check_v25_production_closure.py'],
)

if errors:
    print('v25 production closure check failed:', file=sys.stderr)
    for error in errors:
        print(f'- {error}', file=sys.stderr)
    raise SystemExit(1)

print('v25 production closure checks passed')
