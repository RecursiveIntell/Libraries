# P27 Phase Report

## Phase

- Phase ID: 13
- Phase title: Contract/schema conformance and duplicate-key hardening
- Date: 2026-05-05T03:48:05Z

## Scope

- Intended work: harden evidence-bearing JSON inputs so duplicate keys and invalid JSON are rejected before semantic use, and expose generated-schema validation on AgentSpec and RunBundle operator surfaces.
- Issue IDs in scope: `P27-015`.
- Explicit non-goals: no canonical schema ownership transfer, no broad schema regeneration churn, no lenient JSON repair path, no production-cloud/autonomy widening, no changes to sibling canonical-owner boundaries.

## Files inspected

- `prompts/phases/P27_PHASE_13_PROMPT.md`
- `phase_injections/P27_GATE_AFTER_PHASE_13_BEFORE_PHASE_14.md`
- `P27_PHASE_PLAN.md`
- `P27_MASTER_ISSUE_MATRIX.md`
- `P27_ACCEPTANCE_GATES.md`
- `P27_11A_ALIGNMENT.md`
- `STATUS.md`
- `SUPPORT_PROFILE.md`
- `SOURCE_BASIS.md`
- `crates/aidens-boundary-kit/src/lib.rs`
- `crates/aidens-cli/src/lib.rs`
- `scripts/p27_verify.sh`

## Files changed

- `STATUS.md`
- `crates/aidens-cli/src/lib.rs`
- `scripts/assert_p27_strict_structured_inputs.py`
- `scripts/p27_verify.sh`
- `handoffs/p27/PHASE_13_REPORT.md`

## Changes made

- Routed evidence-bearing CLI JSON inputs through `parse_strict_json` so duplicate keys fail before deserialization:
  - permit grants and approval decisions supplied by `--permit-json`
  - command-run receipt inputs
  - AgentSpec validation inputs
  - run-bundle inspection inputs
  - schema files and generated schema manifests used by `schemas check`
- Added generated-schema validation to `agent validate` and `agent inspect` reports.
- Made `agent inspect` fail closed when a run bundle cannot validate against the generated `AiDENsRunBundleV2` or `AiDENsRunBundleV3` schema.
- Strict-parsed `run-bundle-store-record.json` when present during inspection.
- Added Phase 13 regression tests for duplicate-key refusal on AgentSpec, run bundle, permit JSON, and schema-file inputs.
- Added `scripts/assert_p27_strict_structured_inputs.py` as a static verifier guard and wired it into `scripts/p27_verify.sh`.
- Updated `STATUS.md` to close `P27-015` and record Phase 13 evidence.

## Commands run

| Command | Result | Log |
|---|---|---|
| `cargo fmt` | pass | `target/p27/audit/phase13_cargo_fmt.log` |
| `cargo test -p aidens-cli phase13` | fail, fixed context conversion | `target/p27/audit/phase13_cargo_test_cli_phase13.log` |
| `cargo fmt` | pass | `target/p27/audit/phase13_cargo_fmt_after_context_fix.log` |
| `cargo test -p aidens-cli phase13` | fail, fixed assertions to inspect error chain | `target/p27/audit/phase13_cargo_test_cli_phase13_final.log` |
| `cargo fmt` | pass | `target/p27/audit/phase13_cargo_fmt_after_test_assertions.log` |
| `cargo test -p aidens-cli phase13` | pass | `target/p27/audit/phase13_cargo_test_cli_phase13_final2.log` |
| `python3 -m py_compile scripts/assert_p27_strict_structured_inputs.py` | pass | `target/p27/audit/phase13_py_compile_strict_structured_guard.log` |
| `python3 scripts/assert_p27_strict_structured_inputs.py .` | pass | `target/p27/audit/phase13_assert_strict_structured_inputs.log` |
| `cargo test -p aidens-boundary-kit strict_json` | pass | `target/p27/audit/phase13_cargo_test_boundary_strict_json.log` |
| `cargo test -p aidens-cli schemas_` | pass | `target/p27/audit/phase13_cargo_test_cli_schemas.log` |
| `cargo check -p aidens-boundary-kit -p aidens-cli` | pass | `target/p27/audit/phase13_cargo_check_boundary_cli.log` |
| `cargo fmt --check` | pass | `target/p27/audit/phase13_cargo_fmt_check.log` |
| `cargo run --quiet -p aidens-cli -- schemas generate --out target/p27/audit/phase13_generated_schemas` | pass | `target/p27/audit/phase13_cli_schemas_generate_final.log` |
| `cargo run --quiet -p aidens-cli -- schemas check --root target/p27/audit/phase13_generated_schemas` | pass | `target/p27/audit/phase13_cli_schemas_check_final.log` |
| `cargo run --quiet -p aidens-cli -- agent new --template local-coding --out target/p27/audit/phase13_agent_spec_fixture` | pass | `target/p27/audit/phase13_cli_agent_new.log` |
| `cargo run --quiet -p aidens-cli -- agent validate --spec target/p27/audit/phase13_agent_spec_fixture/agent.json` | pass | `target/p27/audit/phase13_cli_agent_validate.json` |
| `cargo run --quiet -p aidens-cli -- agent run --spec target/p27/audit/phase13_agent_spec_fixture/agent.json --task target/p27/audit/phase13_agent_spec_fixture/task.md --sandbox-root target/p27/audit/phase13_agent_spec_fixture/sandbox --out target/p27/audit/phase13_agent_run` | pass | `target/p27/audit/phase13_cli_agent_run.log` |
| `cargo run --quiet -p aidens-cli -- agent inspect --run target/p27/audit/phase13_agent_run` | pass | `target/p27/audit/phase13_cli_agent_inspect.json` |
| Agent inspect schema-validation summary | pass | `target/p27/audit/phase13_cli_agent_inspect_summary.log` |
| `P27_SKIP_CARGO=1 bash scripts/verify_current.sh` | pass | `target/p27/audit/phase13_verify_current_skip_cargo.log` |
| `python3 scripts/assert_support_claims.py .` | pass | `target/p27/audit/phase13_assert_support_claims.log` |
| `python3 scripts/assert_p27_current_run_truth.py .` | pass | `target/p27/audit/phase13_assert_p27_current_run_truth.log` |
| `python3 scripts/assert_p27_agents_md_current.py .` | pass | `target/p27/audit/phase13_assert_p27_agents_md_current.log` |

## Evidence emitted

- `target/p27/audit/phase13_assert_strict_structured_inputs.log`
- `target/p27/audit/phase13_cargo_test_cli_phase13_final2.log`
- `target/p27/audit/phase13_cargo_test_boundary_strict_json.log`
- `target/p27/audit/phase13_cargo_test_cli_schemas.log`
- `target/p27/audit/phase13_cargo_check_boundary_cli.log`
- `target/p27/audit/phase13_cli_agent_validate.json`
- `target/p27/audit/phase13_cli_agent_run.log`
- `target/p27/audit/phase13_cli_agent_inspect.json`
- `target/p27/audit/phase13_cli_agent_inspect_summary.log`
- `target/p27/audit/phase13_generated_schemas/generated_schema_manifest_v1.json`
- `target/p27/audit/phase13_cli_schemas_check_final.log`
- `target/p27/audit/phase13_verify_current_skip_cargo.log`

## 11A semantic impact

- Exact/approx labels touched: no broad label vocabulary changed. Evidence-bearing CLI surfaces now refuse ambiguous duplicate-key JSON before using the value as exact input.
- Degradation labels touched: no degradation label text changed.
- Support labels touched: no `SUPPORT_PROFILE.md` support-tier claim was widened. `STATUS.md` records `P27-015` closed with a strict structured-input boundary.
- Proof/check hooks added: AgentSpec and RunBundle reports now include generated-schema validation receipts; the verifier now fails if strict structured-input hooks are removed.

## Support profile impact

- No support-tier claim changed in `SUPPORT_PROFILE.md`.
- `STATUS.md` records the Phase 13 closure for `P27-015`.
- The supported-local claim is narrowed by enforcement: duplicate-key evidence-bearing JSON is rejected instead of repaired or accepted silently.

## Canonical-owner impact

- No canonical-owner boundary changed.
- Canonical schema generation ownership remains with `contract-schema-gen`.
- Verification/control ownership remains delegated to `verification-*`; AiDENs emits local operator validation receipts and schema checks only.

## Issues closed

- `P27-015`: strict structured-output boundary now rejects duplicate-key/invalid JSON on evidence-bearing CLI surfaces and reports schema validation for AgentSpec and RunBundle inspection paths.

## New issues / risks

- JSON Schema validation is still a local operator check, not a canonical verification-control proof.
- The generated-schema compatibility report remains scoped to the current schema generator and touched surfaces; broader schema evolution policy remains outside Phase 13.

## Decision

Rationale: evidence-bearing JSON inputs now fail closed on duplicate keys, AgentSpec and RunBundle operator surfaces carry schema validation evidence, focused tests and CLI smokes pass, and the current verifier includes the new strict structured-input guard.

Decision: continue
