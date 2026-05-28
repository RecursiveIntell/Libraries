# P20 Phase 07 Report - Canonical Adapter Proof

Phase: `07`
Scope: real stack usage
Result: `PASS`

## Operator Injection

Proceed to Phase 07 only.

Focus: real stack usage.

Prove delegation paths for:

- semantic-memory-forge -> forge-memory-bridge -> semantic-memory -> knowledge-runtime;
- constraint/compiler/kernel/oracle/conformance crates;
- verification-* crates;
- repair/retraction surfaces if claimed.

If the canonical owner API is unclear, stop and report ambiguity. Do not invent local substitute semantics.

Stop after Phase 07.

## Files Changed

- `crates/aidens-testkit/Cargo.toml`
- `crates/aidens-integration-tests/tests/phase_07_canonical_adapter_proof.rs`
- `README.md`
- `STATUS.md`
- `docs/MASTER_ISSUE_MATRIX.md`
- `docs/CURRENT_AIDENS_AUDIT.md`
- `docs/p20/reports/PHASE_07_REPORT.md`

## Canonical Delegation Proofs

New focused test:

- `crates/aidens-integration-tests/tests/phase_07_canonical_adapter_proof.rs`

The test does not implement substitute semantics. It uses canonical crates directly through AiDENs adapter facades and `TypeId` equality checks where AiDENs exposes canonical type aliases.

| Required area | Proof |
|---|---|
| Memory chain | `memory_adapter_delegates_forge_bridge_memory_runtime` constructs a `semantic_memory_forge::ExportEnvelopeV3`, validates it, transforms it with `forge_memory_bridge::transform_envelope_v3`, imports through `semantic_memory::MemoryStore`, and queries through `knowledge_runtime::KnowledgeRuntime` via `CanonicalMemoryAdapter`. |
| Type ownership for memory path | The test asserts AiDENs exported memory types are the same types as `forge_memory_bridge::ProjectionImportBatchV3`, `semantic_memory::ProjectionImportResult`, and `semantic_memory::MemoryConfig`. |
| Kernel/compiler/oracle/conformance | `kernel_adapter_delegates_compiler_execution_oracle_conformance` transforms a Forge envelope into a bridge batch, compiles it with `constraint_compiler`, executes through `kernel_execution`, evaluates through `kernel_oracles`, and asserts canonical `kernel_conformance` gate exposure through the adapter. |
| Type ownership for kernel path | The test asserts AiDENs kernel exports are the same types as `constraint_compiler::CompileOutput`, `kernel_execution::ExecutionReport`, and `kernel_oracles::OracleAssessment`. |
| Verification stack | `verification_adapter_delegates_control_policy_calibration_adjudication` creates a canonical verification case, plan, attempt, control receipt, policy decision, calibration snapshot, and adjudication result through `CanonicalGovernanceAdapter`. |
| Type ownership for verification path | The test asserts the adapter path uses the same types as `verification_control::CheckPlan`, `verification_policy::PolicySnapshot`, `verification_calibration::CalibrationSnapshot`, and `verification_adjudication::AdjudicationResult`. |
| Repair/retraction | `repair_adapter_delegates_boundary_repair_and_forge_retraction` mints a `verification_control::BoundaryRepairRecord` through `CanonicalRepairAdapter` and validates a `semantic_memory_forge::RetractionRecordV1` through the adapter. |
| Type ownership for repair path | The test asserts AiDENs repair exports are the same types as `verification_control::BoundaryRepairRecord` and `semantic_memory_forge::RetractionRecordV1`. |

## Ambiguity Review

No canonical owner API ambiguity blocked Phase 07.

The required owner APIs were clear and executable:

- memory/export/bridge/query APIs from `semantic-memory-forge`, `forge-memory-bridge`, `semantic-memory`, and `knowledge-runtime`;
- kernel compile/execute/oracle/conformance APIs from `constraint-compiler`, `kernel-execution`, `kernel-oracles`, `kernel-conformance`, and `recursive-kernel-core`;
- verification/governance APIs from `verification-control`, `verification-policy`, `verification-calibration`, and `verification-adjudication`;
- repair/retraction APIs from `verification-control` and `semantic-memory-forge`.

## Failures Found

- Phase 07 lacked a single focused proof test tying every required canonical adapter path to real owner crates.
- `aidens-testkit` did not directly depend on `verification-calibration`, so the Phase 07 test could not directly prove that the calibration snapshot in the governance adapter is the canonical `verification-calibration` type.
- Active docs still marked canonical adapter proofs as Phase 07 pending.

## Fixes Applied

- Added `verification-calibration` as a testkit dependency for direct canonical type proof.
- Added `phase_07_canonical_adapter_proof.rs` with four executable proof tests covering memory, kernel, verification, and repair/retraction delegation.
- Updated active status docs to mark canonical adapter delegation as `partial/proved`, while keeping AiDENs non-authoritative and phases 08-10 unreached.

## Command Evidence

| Command | Result | Evidence |
|---|---:|---|
| `cargo fmt --all -- --check` | pass | `target/p20-phase07/logs/01_cargo_fmt_check.log` |
| `cargo test -p aidens-testkit --test phase_07_canonical_adapter_proof -- --nocapture` | pass | `target/p20-phase07/logs/02_phase07_canonical_adapter_proof.log` |
| `cargo check --workspace --all-targets --all-features` | pass | `target/p20-phase07/logs/03_cargo_check.log` |
| `cargo test --workspace --all-targets --all-features` | pass | `target/p20-phase07/logs/04_cargo_test.log` |
| `cargo clippy --workspace --all-targets --all-features -- -D warnings` | pass | `target/p20-phase07/logs/05_cargo_clippy.log` |
| `python3 scripts/p20_scan_aidens.py --root . --out target/p20-phase07/scan-through-07 --require-phase-reports-through 7 --fail-on-blocking` | pass | log: `target/p20-phase07/logs/06_p20_scan_through_07.log`; JSON: `target/p20-phase07/scan-through-07/p20_scan.json`; markdown: `target/p20-phase07/scan-through-07/p20_scan.md`; blocking findings: `0`; warnings: `21` |
| `P20_REQUIRED_PHASE_REPORT_THROUGH=7 bash scripts/p20_verify.sh` | pass | log: `target/p20-phase07/logs/07_p20_verify_through_07.log`; scanner output: `target/p20-scan/p20_scan.md`; blocking findings: `0`; agency eval fixture shape: `10 cases`; completed successfully |
| `python3 scripts/p20_scan_aidens.py --root . --out target/p20-phase07/scan-through-07-final --require-phase-reports-through 7 --fail-on-blocking` | pass | log: `target/p20-phase07/logs/08_p20_scan_through_07_final.log`; JSON: `target/p20-phase07/scan-through-07-final/p20_scan.json`; markdown: `target/p20-phase07/scan-through-07-final/p20_scan.md`; blocking findings: `0`; warnings: `21` |

## Unresolved Blockers

None for Phase 07.

P20 is not final-complete. Phases 08-10 have not run, and the final audit bundle has not been generated.

## Phase Gate

Phase 07 gate: `PASS`

Stop here and wait for the Phase 08 operator injection.
