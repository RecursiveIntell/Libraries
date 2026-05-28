# P21 Phase 07 Report - Recall/Recall-Coding Extraction

## Scope

Phase 07 used Recall and Recall-Coding as read-only pattern sources. The extraction produced safe AiDENs reports, templates, example configs, and integration tests. No Recall or Recall-Coding crates, DB schemas, session/socket/UI models, hook runners, tool IDs, or local memory/checkpoint stores were imported into AiDENs core.

Touched surfaces:

- `docs/p21/*` for extraction reports.
- `examples/configs/*`, `examples/coding-agent/*`, and `examples/templates/*` for safe operator templates.
- `crates/aidens-integration-tests/tests/phase_07_recall_extraction.rs` for extraction guards.
- `examples/configs/coding-agent.toml` for canonical enum casing needed by existing config validation.

No production Rust crate logic was changed in this phase.

## Invariants Revalidated

Most-at-risk invariants for this phase:

- No shadow memory/evidence/kernel/repair/verification/federation/mechanism truth.
- No Recall app-specific DB/session/socket/UI assumptions in AiDENs core.
- No compatibility shim, parser repair, or semantic widening.
- Generated/operator-facing examples must remain runnable and receipt-bearing.

Pre-change checks:

- `bash scripts/assert_stack_paths.sh .` -> passed.
- `bash scripts/assert_no_local_substitute_dependencies.sh` -> `PASS: no local substitute dependency red flags detected.`
- `bash scripts/assert_compat_is_finite.sh .` -> passed.
- `bash scripts/assert_no_shadow_truth.sh .` -> passed.
- `bash scripts/assert_no_scaffold_promoted.sh .` -> `No scaffold promotion patterns found.`
- `bash scripts/p21_verify.sh` -> passed package integrity, source refs, agency eval validation, and P21 verify.

Post-change checks:

- `bash scripts/assert_stack_paths.sh .` -> passed.
- `bash scripts/assert_no_local_substitute_dependencies.sh` -> `PASS: no local substitute dependency red flags detected.`
- `bash scripts/assert_compat_is_finite.sh .` -> passed.
- `bash scripts/assert_no_shadow_truth.sh .` -> passed.
- `bash scripts/assert_no_scaffold_promoted.sh .` -> `No scaffold promotion patterns found.`
- Recall dependency/app-assumption scan over manifests and crates, excluding the guard test itself -> `PASS: no Recall dependency or app-specific runtime assumption in manifests/crates`.
- `bash scripts/p21_verify.sh` -> passed.

Logs are under `target/p21/phase07/`.

## Source Basis Inspected

Recall:

- `/home/sikmindz/Coding/Recall/recall-daemon/src/core.rs`
- `/home/sikmindz/Coding/Recall/recall-session/src/scheduler.rs`
- `/home/sikmindz/Coding/Recall/recall-session/src/scheduler_store.rs`
- `/home/sikmindz/Coding/Recall/recall-session/src/config.rs`
- `/home/sikmindz/Coding/Recall/recall-session/src/approval.rs`
- `/home/sikmindz/Coding/Recall/recall-daemon/tests/*`

Recall-Coding:

- `/home/sikmindz/Coding/Recall-Coding/recall-session/src/tools/workspace_audit.rs`
- `/home/sikmindz/Coding/Recall-Coding/recall-session/src/tools/workspace_patch.rs`
- `/home/sikmindz/Coding/Recall-Coding/recall-session/src/tools/run_checks.rs`
- `/home/sikmindz/Coding/Recall-Coding/recall-session/src/tools/coding_support.rs`
- `/home/sikmindz/Coding/Recall-Coding/.recall-coding/agents/*.md`
- `/home/sikmindz/Coding/Recall-Coding/.recall-coding/hooks/*.json`

Preflight:

- `bash scripts/preflight_recall.sh /home/sikmindz/Coding/Recall` -> `Preflight ok. Keep Recall read-only.`

## Extracted Patterns

Recall daemon patterns extracted:

- Distinguish loaded, default-created, and invalid config states.
- Redact secrets before operator display.
- Make heartbeat/liveness observable.
- Make safe mode a runtime gate for risky queue admission.
- Use queue/schedule/wake idempotency, leases, cancellation, duplicate suppression, and drain reports.

Recall-Coding patterns extracted:

- Begin coding work with workspace audit and bounded patch target selection.
- Keep read-only audit/search tools distinct from write/admin tools.
- Require explicit scoped permits for patch apply, shell-like execution, checks, and write paths.
- Preserve check outputs as structured receipts.
- Surface blocked/deferred/unavailable tools as operator truth.

Quarantined assumptions:

- Recall session types, DB schemas, scheduler store, socket paths, UI/Tauri bridge, host wake wrappers, and memory/search model.
- Recall-Coding `.recall-coding` data roots, hooks, front matter, tool IDs, checkpoint store, shell wrappers, and app-local artifact directories.

## Files Changed

Added:

- `docs/p21/RECALL_CODING_EXTRACTION_REPORT.md`
- `docs/p21/RECALL_DAEMON_EXTRACTION_REPORT.md`
- `examples/configs/daemon-safe.toml`
- `examples/coding-agent/README.md`
- `examples/templates/coding-agent-lane.template.md`
- `examples/templates/daemon-safe-operator.template.md`
- `crates/aidens-integration-tests/tests/phase_07_recall_extraction.rs`

Modified:

- `examples/configs/coding-agent.toml`

The config edit corrected existing enum values from `Optional`/`Full` to the canonical lowercase forms accepted by AiDENs config validation.

## Commands And Outputs

Setup and invariant logs:

- `mkdir -p target/p21/phase07` -> created proof directory.
- `bash scripts/assert_stack_paths.sh . | tee target/p21/phase07/invariant_stack_paths.before.log` -> passed.
- `bash scripts/assert_no_local_substitute_dependencies.sh | tee target/p21/phase07/invariant_no_local_substitute_dependencies.before.log` -> `PASS: no local substitute dependency red flags detected.`
- `bash scripts/assert_compat_is_finite.sh . | tee target/p21/phase07/invariant_compat_is_finite.before.log` -> passed.
- `bash scripts/assert_no_shadow_truth.sh . | tee target/p21/phase07/invariant_no_shadow_truth.before.log` -> passed.
- `bash scripts/assert_no_scaffold_promoted.sh . | tee target/p21/phase07/invariant_no_scaffold_promoted.before.log` -> `No scaffold promotion patterns found.`
- `bash scripts/p21_verify.sh | tee target/p21/phase07/p21_verify.before.log` -> passed.

Source discovery:

- `find /home/sikmindz/Coding/Recall-Coding -maxdepth 3 -type f ... | tee target/p21/phase07/recall_coding_file_index.head.log` -> Recall-Coding source index captured.
- `find /home/sikmindz/Coding/Recall -maxdepth 3 -type f ... | tee target/p21/phase07/recall_file_index.head.log` -> Recall source index captured.
- `bash scripts/preflight_recall.sh /home/sikmindz/Coding/Recall | tee target/p21/phase07/recall_preflight.log` -> `Preflight ok. Keep Recall read-only.`

Formatting and tests:

- `cargo fmt --all` -> passed.
- `cargo test -p aidens-integration-tests --test phase_07_recall_extraction --all-features | tee target/p21/phase07/recall_extraction_tests.first.log` -> failed on pre-existing invalid casing in `examples/configs/coding-agent.toml`; fixed to canonical lowercase values.
- `cargo test -p aidens-integration-tests --test phase_07_recall_extraction --all-features | tee target/p21/phase07/recall_extraction_tests.log` -> passed, 4 tests.
- `cargo fmt --all --check | tee target/p21/phase07/fmt_check.log` -> passed.
- `cargo check -p aidens-integration-tests --all-targets --all-features | tee target/p21/phase07/check_integration_tests.log` -> passed.
- `cargo clippy -p aidens-integration-tests --all-targets --all-features -- -D warnings | tee target/p21/phase07/clippy_integration_tests.log` -> passed.

Operator-facing proof:

- `cargo run -p aidens-cli -- plan validate --config examples/configs/coding-agent.toml | tee target/p21/phase07/coding_agent_plan_validate.log` -> `valid: p21-coding-agent-example profile=coding-agent provider=mock source=loaded examples/configs/coding-agent.toml`.
- `cargo run -p aidens-cli -- plan validate --config examples/configs/daemon-safe.toml | tee target/p21/phase07/daemon_safe_plan_validate.log` -> `valid: p21-daemon-safe-example profile=autonomous-daemon provider=mock source=loaded examples/configs/daemon-safe.toml`.
- `cargo run -p aidens-cli -- doctor --config examples/configs/daemon-safe.toml | tee target/p21/phase07/daemon_safe_doctor.log` -> passed; output shows mock provider executable, memory disabled, full canonical receipts, daemon queue/schedule/wake available, network disabled, and deferred scaffold surfaces.
- `cargo run -p aidens-cli -- provider-check --config examples/configs/coding-agent.toml | tee target/p21/phase07/coding_agent_provider_check.log` -> passed; mock provider executable with `support_label=fixture-supported-not-cloud`.
- `cargo run -p aidens-cli -- tools inspect --config examples/configs/coding-agent.toml | tee target/p21/phase07/coding_agent_tools_inspect.log` -> passed; read-only tools exposed, write/admin tools blocked or hidden with permit/reason codes.
- `test -s ... && echo 'Phase 07 extraction artifacts present' | tee target/p21/phase07/extraction_artifacts_exist.log` -> `Phase 07 extraction artifacts present`.

Post-change invariant proof:

- `bash scripts/assert_stack_paths.sh . | tee target/p21/phase07/invariant_stack_paths.after.log` -> passed.
- `bash scripts/assert_no_local_substitute_dependencies.sh | tee target/p21/phase07/invariant_no_local_substitute_dependencies.after.log` -> `PASS: no local substitute dependency red flags detected.`
- `bash scripts/assert_compat_is_finite.sh . | tee target/p21/phase07/invariant_compat_is_finite.after.log` -> passed.
- `bash scripts/assert_no_shadow_truth.sh . | tee target/p21/phase07/invariant_no_shadow_truth.after.log` -> passed.
- `bash scripts/assert_no_scaffold_promoted.sh . | tee target/p21/phase07/invariant_no_scaffold_promoted.after.log` -> `No scaffold promotion patterns found.`
- Recall dependency/app-assumption scan -> `PASS: no Recall dependency or app-specific runtime assumption in manifests/crates`.
- `bash scripts/p21_verify.sh | tee target/p21/phase07/p21_verify.log` -> passed.

## Proof Artifacts

Extraction artifacts:

- `docs/p21/RECALL_CODING_EXTRACTION_REPORT.md`
- `docs/p21/RECALL_DAEMON_EXTRACTION_REPORT.md`
- `examples/templates/coding-agent-lane.template.md`
- `examples/templates/daemon-safe-operator.template.md`
- `examples/coding-agent/README.md`
- `examples/configs/daemon-safe.toml`

Proof logs:

- `target/p21/phase07/recall_extraction_tests.log`
- `target/p21/phase07/fmt_check.log`
- `target/p21/phase07/check_integration_tests.log`
- `target/p21/phase07/clippy_integration_tests.log`
- `target/p21/phase07/coding_agent_plan_validate.log`
- `target/p21/phase07/daemon_safe_plan_validate.log`
- `target/p21/phase07/daemon_safe_doctor.log`
- `target/p21/phase07/coding_agent_provider_check.log`
- `target/p21/phase07/coding_agent_tools_inspect.log`
- `target/p21/phase07/no_recall_dependency_scan.log`
- `target/p21/phase07/p21_verify.log`

## Outcome

Phase 07 passed.

Recall and Recall-Coding patterns were extracted into safe AiDENs reports, templates, examples, and tests. Unsupported app-specific assumptions remain quarantined. No new shadow truth, Recall dependency, compatibility shim, parser repair, or production runtime substitute was introduced.

Per P21 phase protocol, stop here and wait for the operator's Phase 08 injection before continuing.
