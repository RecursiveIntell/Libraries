# P28 Codex Super Pass Prompt — Paste into Codex

You are operating on the AiDENs repo after P27. Your task is **P28: v11A Constitutional Material-Operation Kernel**.

## Mission

Make AiDENs v11A-conformant on the declared local production path by implementing a material-operation kernel where every material action is a typed, receipt-bearing artifact transition with operator contract, execution context, manifests, receipts, proof/debt/degradation state, and boundary compiler law.

Do **not** attempt broad feature expansion. Do **not** enable hosted providers, broad autonomy, production daemon claims, v11B active runtime claims, or v11C active federation/mechanism/self-hosting claims.

## Read first

Read these before editing:

1. `P28_MASTER_PACKET.md`
2. `P28_ACCEPTANCE_GATES.md`
3. `P28_PHASE_PLAN.md`
4. `P28_BUG_ABSORPTION_MATRIX.csv`
5. `docs/codex-runs/Specs/CANONICAL_STACK_SPEC_V11A_CONSTITUTIONAL_ARTIFACT_RUNTIME_CORE.md`
6. `docs/codex-runs/Specs/V11_PLUS_CONFORMANCE_AND_RELEASE_BAR.md`
7. `P27_STATUS_EVIDENCE_MANIFEST.json`
8. `STATUS.md`, `SUPPORT_PROFILE.md`, `SOURCE_BASIS.md`

## Hard constraints

- Do not create shadow truth ownership in AiDENs.
- Runtime/tool/CLI/runner outputs are not domain truth.
- Sibling canonical truth systems remain canonical.
- Material operations must emit receipts.
- Done without receipts is non-conforming.
- Parser repair cannot silently change treatment.
- Waiver is not proof.
- Degraded is not exact.
- Zip-byte hash is not canonical content hash.
- Random replay-sensitive IDs are forbidden unless explicitly labeled non-deterministic and excluded from replay equality.

## Phase order

Execute `P28_PHASE_PLAN.md` in order. Stop after a phase if its exit gate fails. Write a phase report for every phase.

Minimum phase report fields:

```markdown
# P28 Phase NN Report

## Scope
## Files changed
## Claims made
## Evidence
## Tests run
## Failures / degraded checks
## Open risks
## Next phase readiness
```

## First code priority

Before v11 scaffolding, fix or quarantine the P0 bugs:

- C05 lease identity timestamp fallback
- C07 `z.py safe_relative` symlink/fallback
- C11 profile expansion panic
- C24 dirty directory creation on failed patch
- C25 symlink escape on create/write path
- C32 random replay-sensitive artifact IDs
- C53 package manifest/upload path verification gap
- C54 known limitations empty-register blocking
- C55 waiver satisfying blocked traceability state
- C59 convergence/degraded inconsistent semantics
- C66 history preservation digest logic
- C72 degraded subcheck with exact aggregate status

Each fix must include a regression test.

## v11A implementation target

Implement or admit facades for:

- `ArtifactEnvelopeV1`
- `ArtifactManifestV1`
- `ArtifactLifecycleStateV1`
- `ArtifactTransitionReceiptV1`
- `ExecutionContextEnvelopeV1`
- `OperatorContractV1`
- `OperatorEffectV1`
- `OperatorInvocationReceiptV1`
- `ToolCallReceiptV1`
- `BoundaryCompilerProfileV1`
- `BoundaryCompileReceiptV1`
- `BoundaryRepairReceiptV1`
- `TreatmentIntegrityReceiptV1`
- `ProofProfileV1`
- `ProofDebtLedgerV1`
- `ProofWaiverReceiptV1`
- `SemanticStateV1`
- `ViewDisclosureV1`
- `DegradationRecordV1`

If a sibling crate already owns a canonical family, re-export/reference it; do not duplicate semantics.

## Required declared operator contracts

At minimum register contracts for:

- `aidens.agent.validate`
- `aidens.agent.doctor`
- `aidens.runner.turn`
- `aidens.provider.route`
- `aidens.tool.repo_read`
- `aidens.tool.repo_list`
- `aidens.tool.file_stat`
- `aidens.tool.repo_search`
- `aidens.tool.patch_propose`
- `aidens.tool.patch_apply`
- `aidens.tool.run_checks`
- `aidens.package.generate`
- `aidens.package.validate`
- `aidens.package.self_replay`
- `aidens.report.final_done`

Each contract must declare input/output families, effects, forbidden effects, proof obligations, boundary profile, replay requirements, failure taxonomy, and human approval if applicable.

## Tests to add

Add tests for:

- no done without receipts
- operator contract required for material operations
- undeclared effect blocked
- duplicate JSON key rejected
- parser repair emits treatment-integrity receipt
- proof waiver is not proof
- proof debt restricts promotion/use
- degraded surface blocks readiness unless waived
- run bundle cannot silently overwrite same run id
- event log digest chain detects tampering
- symlink escape blocked for reads/writes
- patch write failure leaves no dirty directories
- command allowlist behavior including strict args
- timeout output marked partial
- package status aggregate downgrades on degraded subcheck

## Final commands

Run and record logs:

```bash
cargo fmt --all -- --check
cargo check --workspace --all-targets
cargo test --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
cargo doc --workspace --no-deps
P28_FINAL_STRICT=1 bash scripts/verify_current.sh
python3 z.py --root . --profile aidens --mode next-codex-context --strict --codex-current-run P28 --output target/p28/package/AiDENs-p28-codex-context.zip
python3 scripts/assert_package_validation.py
python3 scripts/assert_package_self_replay.py --package target/p28/package/AiDENs-p28-codex-context.zip --verifier scripts/verify_current.sh --require-verifier --receipt-out target/p28/audit/package_self_replay_p28_final_receipt.json
```

## Final evidence outputs

- `P28_STATUS_EVIDENCE_MANIFEST.json`
- `docs/p28/P28_FINAL_AUDIT_REPORT.md`
- `handoffs/p28/FINAL_AUDITOR_HANDOFF.md`
- `target/p28/audit/*` logs
- final package sidecars

## Brutal rule

If the artifact, receipt, execution context, proof profile, boundary/compiler record, time coordinates, view disclosure, conformance run, and debt/waiver state cannot be shown, do not claim v11A conformance.
