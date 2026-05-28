# P21 Phase 03 Report — Generated Agent Project Proof

Status: PASS

Run timestamp: 2026-04-30T23:10:15-05:00

## Phase scope

Phase 03 focused on making `aidens new coding-agent` generate a safe project that actually runs.

Touched files/crates:

- `crates/aidens-cli/src/lib.rs`
- `handoffs/p21/PHASE_03_REPORT.md`
- `target/p21/phase03/` logs
- generated proof projects:
  - `target/demo-agent/`
  - `target/p21/example-coding-agent/`

No canonical stack crates, runner/provider/tool/agency crates, fixtures, evals, scanners, or source-of-truth documents were changed.

## Invariant revalidation

Before Phase 03 implementation, these checks passed:

- `bash scripts/assert_stack_paths.sh .`
  - Log: `target/p21/phase03/invariant_stack_paths.log`
  - Result: PASS
- `bash scripts/assert_no_local_substitute_dependencies.sh`
  - Log: `target/p21/phase03/invariant_no_local_substitute_dependencies.log`
  - Output: `PASS: no local substitute dependency red flags detected.`
- `bash scripts/assert_compat_is_finite.sh .`
  - Log: `target/p21/phase03/invariant_compat_is_finite.log`
  - Result: PASS
- `bash scripts/assert_no_shadow_truth.sh .`
  - Log: `target/p21/phase03/invariant_no_shadow_truth.log`
  - Result: PASS
- `bash scripts/p21_verify.sh`
  - Log: `target/p21/phase03/invariant_p21_verify.log`
  - Output ended with `P21 verify completed`

After implementation, `bash scripts/p21_verify.sh` passed again.

Log: `target/p21/phase03/p21_verify_after_change.log`

## Baseline finding

Before repair, the generated coding-agent config used a disabled provider and produced only four files:

```text
Cargo.toml
aidens.toml
src/main.rs
tests/smoke.rs
```

The baseline proof failed:

```text
cargo run -p aidens-cli -- run --config target/p21/phase03/baseline-demo-agent/aidens.toml "read README"
Error: provider unavailable: provider-disabled
```

Logs:

- `target/p21/phase03/baseline_new_coding_agent.log`
- `target/p21/phase03/baseline_run_generated_agent.log`

## Implementation summary

Updated the coding-agent scaffold so generated projects are safe and runnable by default:

- provider defaults to local mock provider `aidens-safe-mock`;
- mock response requests `aidens:repo-read:1` for `README.md`, then returns README evidence through the real runner/tool path;
- tool bundles default to read/list/search/stat plus `patch-propose`;
- `patch-apply`, `run-checks`, shell, network, and admin tools are not enabled by default;
- sandbox root is generated as an absolute path to the project directory;
- receipts remain full-level and are emitted under the generated project;
- generated `src/main.rs` loads `aidens.toml` from `CARGO_MANIFEST_DIR`, so the generated binary runs even when invoked by manifest path from another directory;
- generated project now includes operator docs:
  - `README.md`
  - `AGENT.md`
  - `docs/tools.md`
  - `docs/permits.md`
  - `docs/receipts.md`

The existing CLI scaffold test was extended to run the generated config and verify receipt output.

## Required proof

### `cargo run -p aidens-cli -- new coding-agent target/demo-agent`

Log: `target/p21/phase03/new_coding_agent_target_demo.log`

Result: PASS

Generated files:

```text
AGENT.md
Cargo.toml
README.md
aidens.toml
docs/permits.md
docs/receipts.md
docs/tools.md
src/main.rs
tests/smoke.rs
```

File listing log: `target/p21/phase03/generated_project_files.log`

Docs/config/tests presence check:

```text
generated-docs-config-tests-present
```

Log: `target/p21/phase03/generated_project_docs_check.log`

### `cargo run -p aidens-cli -- run --config target/demo-agent/aidens.toml "read README"`

Log: `target/p21/phase03/run_generated_demo_agent.log`

Result: PASS

The output begins with:

```text
README evidence summary:
# demo-agent
```

This proves the generated agent reads its generated README through the mock provider plus `repo-read` runner/tool path.

### Receipts emitted

Receipt list log: `target/p21/phase03/generated_demo_receipts_list.log`

Receipt file:

```text
target/demo-agent/target/aidens-receipts/demo-agent/canonical-receipts.ndjson
```

Receipt summary:

```text
records=10 schemas=agency-policy-report-v1,control-receipt,run-report-v1,tool-exposure-plan-v1
```

Latest run-report summary:

```text
provider=mock route=mock native_tool_loop=false tool_invocations=1 agency_receipts=8
```

Logs:

- `target/p21/phase03/generated_demo_receipt_summary.log`
- `target/p21/phase03/generated_demo_run_report_summary.log`

## Additional proof

The documented acceptance-gate path also passed:

```bash
cargo run -p aidens-cli -- new coding-agent target/p21/example-coding-agent
cargo run -p aidens-cli -- run --config target/p21/example-coding-agent/aidens.toml "read README"
```

Logs:

- `target/p21/phase03/new_coding_agent_target_p21_example.log`
- `target/p21/phase03/run_generated_p21_example_agent.log`

The generated project itself also passed:

```bash
cargo test --manifest-path target/demo-agent/Cargo.toml
cargo run --manifest-path target/demo-agent/Cargo.toml
```

Logs:

- `target/p21/phase03/generated_demo_cargo_test.log`
- `target/p21/phase03/generated_demo_cargo_run.log`

## Validation

- `cargo test -p aidens-cli new_app_scaffold_contains_safe_config_and_tests -- --nocapture`
  - Log: `target/p21/phase03/cargo_test_cli_scaffold.log`
  - Result: PASS
- `cargo fmt --all --check`
  - Log: `target/p21/phase03/cargo_fmt_check.log`
  - Result: PASS
- `cargo check -p aidens-cli --all-targets --all-features`
  - Log: `target/p21/phase03/cargo_check_aidens_cli.log`
  - Result: PASS
- `cargo test -p aidens-cli --all-targets --all-features`
  - Log: `target/p21/phase03/cargo_test_aidens_cli_all_targets.log`
  - Result: PASS
- `cargo clippy -p aidens-cli --all-targets --all-features -- -D warnings`
  - Log: `target/p21/phase03/cargo_clippy_aidens_cli.log`
  - Result: PASS
- `cargo check --workspace --all-targets --all-features`
  - Log: `target/p21/phase03/cargo_check_workspace_all_targets_all_features.log`
  - Result: PASS

## Invariant checks performed

- No shadow truth: scanner passed; scaffold only emits product config/docs and uses existing runner/tool/receipt behavior.
- No local canonical ownership: no memory/evidence/kernel/repair/verification/federation/mechanism semantics were added.
- Canonical IDs: `stack-ids` path scanner passed and no stack dependency wiring changed.
- No silent fallback: README read path uses existing parser-fallback tool-call handling and runner receipts.
- Execution is evidence: generated run emits provider route, tool exposure, tool invocation, agency policy, control, and run-report evidence.
- Agency gate: generated runs produced agency policy receipt IDs through the existing runner.
- Provider/tool truth: generated provider is explicit local mock; no cloud provider or native tool-loop support is claimed.
- Generated project safety: no write/admin bundles are enabled by default.
- No tests, fixtures, evals, or scanners were deleted or weakened.

## Repairs

- Replaced generated coding-agent disabled-provider default with an executable local mock provider.
- Reduced generated coding-agent default tool bundles to safe read/search/stat/proposal tools.
- Added generated operator docs and README.
- Fixed generated binary config loading so it works when run from outside the project directory.

## Stop condition

Per P21 phase protocol, Codex must stop here and wait for the operator to paste the next global plus Phase 04-specific injection before touching code or proceeding to profile and plan-kit usability.
