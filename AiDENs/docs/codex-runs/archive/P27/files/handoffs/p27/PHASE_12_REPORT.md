# P27 Phase Report

## Phase

- Phase ID: 12
- Phase title: Memory-grounding durable seam
- Date: 2026-05-05T02:44:55Z

## Scope

- Intended work: strengthen memory-grounding evidence with canonical adapter backpointers, exact/degraded labels, and explicit no-local-truth-store disclosure.
- Issue IDs in scope: `P27-011`.
- Explicit non-goals: no local memory database substitute, no canonical memory ownership change, no production memory truth claim, no provider/autonomy widening, no broad memory runtime rewrite.

## Files inspected

- `prompts/phases/P27_PHASE_12_PROMPT.md`
- `phase_injections/P27_GATE_AFTER_PHASE_12_BEFORE_PHASE_13.md`
- `P27_PHASE_PLAN.md`
- `P27_MASTER_ISSUE_MATRIX.md`
- `P27_ACCEPTANCE_GATES.md`
- `P27_11A_ALIGNMENT.md`
- `STATUS.md`
- `SUPPORT_PROFILE.md`
- `SOURCE_BASIS.md`
- `crates/aidens-memory-kit/src/lib.rs`
- `crates/aidens-memory-kit/Cargo.toml`
- `crates/aidens-runner/src/lib.rs`
- `crates/aidens-cli/src/lib.rs`
- `crates/aidens-contracts/src/lib.rs`
- `scripts/p27_verify.sh`

## Files changed

- `STATUS.md`
- `crates/aidens-memory-kit/Cargo.toml`
- `crates/aidens-memory-kit/src/lib.rs`
- `crates/aidens-runner/src/lib.rs`
- `crates/aidens-cli/src/lib.rs`
- `scripts/assert_p27_memory_no_local_truth.py`
- `scripts/p27_verify.sh`
- `handoffs/p27/PHASE_12_REPORT.md`

## Changes made

- Added `MemoryGroundingEvidenceV1` and `MemoryGroundingBackpointerV1` to `aidens-memory-kit`.
- The receipt records:
  - canonical adapter route: `semantic-memory-forge -> forge-memory-bridge -> semantic-memory -> knowledge-runtime`
  - support tier
  - `semantic_status`
  - query/result/import counts
  - trace context display
  - canonical owner backpointers
  - degradation and widening labels
  - `local_truth_store=false`
  - known limits that AiDENs emits local operator evidence only.
- `PlanActVerifyLoopV1` memory grounding now emits the typed JSON receipt before legacy string breadcrumbs.
- `memory seam-fixture` reports now include `semantic_status`, `local_truth_store=false`, and embedded `grounding_evidence`.
- Added `scripts/assert_p27_memory_no_local_truth.py`.
- Wired the memory no-local-truth guard into `scripts/p27_verify.sh`.
- Updated `STATUS.md` to close `P27-011` and record Phase 12 evidence.

## Commands run

| Command | Result | Log |
|---|---|---|
| `cargo fmt` | pass | `target/p27/audit/phase12_cargo_fmt.log` |
| `cargo fmt --check` | pass | `target/p27/audit/phase12_cargo_fmt_check.log` |
| `python3 -m py_compile scripts/assert_p27_memory_no_local_truth.py` | pass | `target/p27/audit/phase12_py_compile_memory_guard.log` |
| `python3 scripts/assert_p27_memory_no_local_truth.py .` | pass | `target/p27/audit/phase12_assert_memory_no_local_truth.log` |
| `cargo test -p aidens-memory-kit` | pass | `target/p27/audit/phase12_cargo_test_memory_kit.log` |
| `cargo test -p aidens-runner memory_grounding` | pass | `target/p27/audit/phase12_cargo_test_runner_memory_grounding.log` |
| `cargo run --quiet -p aidens-cli -- memory seam-fixture --out target/p27/audit/phase12_memory_seam_fixture` | pass | `target/p27/audit/phase12_cli_memory_seam_fixture.log` |
| Memory seam report validation | pass | `target/p27/audit/phase12_validate_memory_seam_report.log` |
| `cargo check -p aidens-memory-kit -p aidens-runner -p aidens-cli` | pass | `target/p27/audit/phase12_cargo_check_memory_path.log` |
| `P27_SKIP_CARGO=1 bash scripts/verify_current.sh` | pass | `target/p27/audit/phase12_verify_current_skip_cargo.log` |
| `python3 scripts/assert_support_claims.py .` | pass | `target/p27/audit/phase12_assert_support_claims.log` |
| `python3 scripts/assert_p27_current_run_truth.py .` | pass | `target/p27/audit/phase12_assert_p27_current_run_truth.log` |
| `python3 scripts/assert_p27_agents_md_current.py .` | pass | `target/p27/audit/phase12_assert_p27_agents_md_current.log` |

## Evidence emitted

- `target/p27/audit/phase12_memory_seam_fixture/memory-runtime-seam-report.json`
- `target/p27/audit/phase12_memory_seam_fixture/export-envelope-v3.json`
- `target/p27/audit/phase12_memory_seam_fixture/projection-import-batch-v3.json`
- `target/p27/audit/phase12_validate_memory_seam_report.log`
- `target/p27/audit/phase12_assert_memory_no_local_truth.log`
- `target/p27/audit/phase12_cargo_test_memory_kit.log`
- `target/p27/audit/phase12_cargo_test_runner_memory_grounding.log`
- `target/p27/audit/phase12_cargo_check_memory_path.log`
- `target/p27/audit/phase12_verify_current_skip_cargo.log`

## 11A semantic impact

- Exact/approx labels touched: added `semantic_status` to `MemoryGroundingEvidenceV1` and the CLI seam fixture report.
- Degradation labels touched: `MemoryGroundingEvidenceV1` emits `degraded_exact_check` when degradation or widening labels are present.
- Support labels touched: no `SUPPORT_PROFILE.md` support-tier claim was widened. `STATUS.md` records the Phase 12 closure for `P27-011`.
- Proof/check hooks added: memory-kit tests cover exact and degraded receipts, runner tests parse the typed receipt, CLI seam fixture validation checks canonical owners, and the verifier now runs a no-local-truth scanner.

## Support profile impact

- No support-tier claim changed in `SUPPORT_PROFILE.md`.
- Existing memory posture remains adapter-only and partial where appropriate.
- `STATUS.md` now records `P27-011` closed with the limited claim that durable memory grounding evidence names canonical owners and denies local truth storage.

## Canonical-owner impact

- No canonical-owner boundary changed.
- Canonical ownership remains:
  - raw evidence/export packages: `semantic-memory-forge`
  - bridge transforms: `forge-memory-bridge`
  - projected memory store: `semantic-memory`
  - runtime query/view/widening disclosure: `knowledge-runtime`
- AiDENs emits only local operator evidence and adapter routing.

## Issues closed

- `P27-011`: memory grounding now has durable typed evidence with canonical backpointers, degradation labels, and no local truth store assertion.

## New issues / risks

- The runner still stores memory grounding receipts as strings in the existing `PlanActVerifyLoopV1Output`; the first string is now typed JSON for durable parsing.
- Memory seam fixture uses a mock embedder for supported-local evidence; it does not claim production memory retrieval.
- Broader 11A surface uniformity remains Phase 17 scope.

## Decision

Rationale: Memory grounding now emits parseable canonical-seam evidence, the CLI seam fixture carries exact/no-local-truth fields, tests cover exact/degraded receipt semantics, and the verifier fails if a local memory truth surface is introduced.

Decision: continue
