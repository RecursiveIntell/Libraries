# P21 Phase 01 Build Certification

Status: PASS

Run timestamp: 2026-04-30T22:52:23-05:00

## Phase scope

Phase 01 was limited to build certification. No new features, architecture refactors, source rewrites, fixture deletions, eval deletions, scanner deletions, or test weakening were performed.

Initial touch plan:

- No source files or crates unless a build/fmt/test/clippy failure required a minimal repair.
- Required output artifacts only: `target/p21/phase01/` logs and this handoff report.

Most-at-risk invariants rechecked before cargo gates:

- Canonical ownership boundaries for memory/evidence/kernel/repair/verification/federation/mechanism semantics.
- Canonical `stack-ids` path from the sibling `libraries` workspace, not `libraries2`.
- Receipt-bearing degradation/fallback/repair behavior.
- Runtime agency gate behavior for final output and tool-output incorporation.
- Provider/tool capability truth: unsupported cloud/native-tool-loop routes remain unavailable.

## Invariant revalidation

### `bash scripts/assert_stack_paths.sh .`

Log: `target/p21/phase01/invariant_stack_paths.log`

Result: PASS. No forbidden `libraries2` stack-id dependency path was reported.

### `bash scripts/assert_no_local_substitute_dependencies.sh`

Log: `target/p21/phase01/invariant_no_local_substitute_dependencies.log`

```text
PASS: no local substitute dependency red flags detected.
```

### `bash scripts/assert_compat_is_finite.sh .`

Log: `target/p21/phase01/invariant_compat_is_finite.log`

Result: PASS. No forbidden compatibility crate or shim markers were reported.

### `bash scripts/assert_no_shadow_truth.sh .`

Log: `target/p21/phase01/invariant_no_shadow_truth.log`

Result: PASS. No forbidden local canonical shadow-truth type patterns were reported.

### `bash scripts/p21_verify.sh`

Log: `target/p21/phase01/invariant_p21_verify.log`

```text
{
  "include_missing": [],
  "include_refs": 81,
  "manifest_missing": [],
  "ok": true,
  "required_missing": [],
  "root": "/home/sikmindz/Coding/Libraries/AiDENs"
}
{
  "missing_cross_refs": [],
  "ok": true,
  "root": "/home/sikmindz/Coding/Libraries/AiDENs"
}
Agency eval validation OK: 10 cases, 10 surfaces, 22 receipt kinds
P21 verify completed
```

Targeted code reads also confirmed:

- `Cargo.toml` uses `stack-ids = { version = "0.1.0", path = "../stack-ids" }`.
- `crates/aidens-runner/src/lib.rs` evaluates agency policy before final output and after successful tool output, records agency receipt IDs, and persists agency policy reports when durable receipts are configured.
- `crates/aidens-provider-kit/src/lib.rs` keeps `openai-compatible`, `openai`, `openrouter`, and `anthropic` as unavailable boundaries with `native_tool_loop_executable = false`.

No invariant blocker was found before Phase 01 build work.

## Required proof commands

### `cargo fmt --all --check`

Log: `target/p21/phase01/cargo_fmt_check.log`

Result: PASS. Command produced no output.

### `cargo check --workspace --all-targets --all-features`

Log: `target/p21/phase01/cargo_check_workspace_all_targets_all_features.log`

```text
Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.34s
```

### `cargo test --workspace --all-targets --all-features`

Log: `target/p21/phase01/cargo_test_workspace_all_targets_all_features.log`

Result: PASS. The full workspace test suite completed successfully. The log includes unit, integration, runner, provider, agency, tool, package, and release-audit tests, including:

- agency eval and runner agency gate tests;
- provider capability truth tests;
- stack import/source truth tests;
- canonical adapter/delegation tests;
- package honesty and release truth audit tests;
- tool receipt, permit, budget, daemon, queue, and repair tests.

### `cargo clippy --workspace --all-targets --all-features -- -D warnings`

Log: `target/p21/phase01/cargo_clippy_workspace_all_targets_all_features.log`

```text
Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.22s
```

## Files changed

- `handoffs/p21/PHASE_01_BUILD_CERTIFICATION.md`
- `target/p21/phase01/cargo_fmt_check.log`
- `target/p21/phase01/cargo_check_workspace_all_targets_all_features.log`
- `target/p21/phase01/cargo_test_workspace_all_targets_all_features.log`
- `target/p21/phase01/cargo_clippy_workspace_all_targets_all_features.log`
- `target/p21/phase01/invariant_stack_paths.log`
- `target/p21/phase01/invariant_no_local_substitute_dependencies.log`
- `target/p21/phase01/invariant_compat_is_finite.log`
- `target/p21/phase01/invariant_no_shadow_truth.log`
- `target/p21/phase01/invariant_p21_verify.log`

No Rust source files, manifests, fixtures, evals, scanners, or package scripts were changed.

## Repairs

None required. All Phase 01 cargo gates passed on the first run after invariant revalidation.

## Stop condition

Per P21 phase protocol, Codex must stop here and wait for the operator to paste the next global plus Phase 02-specific injection before touching code or proceeding to the `run-test-agent` CLI phase.
