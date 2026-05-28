# P27 Phase 19 Report — Full Validation and Hostile Audit

## Scope

Phase 19 covered full validation and hostile audit. Files inspected included the Phase 19 prompt/injection, `P27_PHASE_PLAN.md`, `P27_ACCEPTANCE_GATES.md`, `P27_COMMANDS.md`, `STATUS.md`, package/replay helpers, completion-audit code, and the failing test surfaces found by validation.

Issues in scope:

- `P27-002`: package self-replay unverified.
- Final release-bar validation across the P27 hard blockers, capability evidence, support truth, and package replay surfaces.

No-go zones observed:

- No new capability was added.
- No support-tier claim was widened.
- No canonical-owner boundary changed.
- No final P27 success claim is made before Phase 20 final package/evidence manifest.

## Changes

- Fixed a stale `aidens-runner` patch-apply test fixture so it uses a contextual diff compatible with Phase 10 fail-closed patch semantics.
- Simplified an equivalent `aidens-cli` failure-taxonomy branch to satisfy clippy.
- Updated completion-audit traceability fallback to use `handoffs/p27` before historical P24/P23/P22 handoff directories.
- Updated Phase 00 source-truth integration tests to run shell guards through `bash`, avoiding executable-bit dependence after zip extraction.
- Updated `STATUS.md` to close `P27-002` with exact package replay evidence.
- Drafted `docs/p27/P27_FINAL_AUDIT_REPORT.md`.
- Drafted `handoffs/p27/FINAL_AUDITOR_HANDOFF.md`.

## Changed Files

- `crates/aidens-runner/src/lib.rs`
- `crates/aidens-cli/src/lib.rs`
- `crates/aidens-cli/src/package.rs`
- `crates/aidens-integration-tests/tests/phase_00_source_truth.rs`
- `STATUS.md`
- `docs/p27/P27_FINAL_AUDIT_REPORT.md`
- `handoffs/p27/FINAL_AUDITOR_HANDOFF.md`
- `handoffs/p27/PHASE_19_REPORT.md`

## Validation

Command logs are under `target/p27/audit/`.

Final green validation set:

- `cargo fmt --all -- --check` passed: `target/p27/audit/cargo_fmt_phase19_after_replay_script_exec_fix.log`
- `cargo check --workspace --all-targets` passed: `target/p27/audit/cargo_check_phase19_after_replay_script_exec_fix.log`
- `cargo test --workspace --all-targets` passed: `target/p27/audit/cargo_test_phase19_after_replay_script_exec_fix.log`
- `cargo clippy --workspace --all-targets -- -D warnings` passed: `target/p27/audit/cargo_clippy_phase19_after_replay_script_exec_fix.log`
- `cargo doc --workspace --no-deps` passed: `target/p27/audit/cargo_doc_phase19_after_replay_script_exec_fix.log`
- `P27_FINAL_STRICT=1 bash scripts/verify_current.sh` passed: `target/p27/audit/verify_current_phase19_after_replay_script_exec_fix_final_strict.log`
- Strict package generation passed with zero findings: `target/p27/audit/zpy_package_phase19_after_replay_script_exec_fix.log`
- Package sidecar validation passed: `target/p27/audit/package_validation_phase19_after_replay_script_exec_fix.log`
- Skip-cargo package replay passed as `degraded_exact_check`: `target/p27/audit/package_self_replay_phase19_after_replay_script_exec_fix_skip_cargo.log`
- Full cargo-backed package replay passed as `exact_check`: `target/p27/audit/package_self_replay_phase19_after_replay_script_exec_fix_full.log`

Final package evidence:

- `target/p27/package/AiDENs-p27-codex-context.zip`
- `target/p27/package/AiDENs-p27-codex-context.report.md`
- `target/p27/package/AiDENs-p27-codex-context.manifest.json`
- `target/p27/package/AiDENs-p27-codex-context.findings.json`
- `target/p27/package/AiDENs-p27-codex-context.excluded.json`
- SHA-256: `9774029315375e1ab5fcd2ec73efaeb19e36c073c540a88c86d9d80f06d3e0a6`

## Hostile Audit Findings Resolved

- Initial full package replay failed in `/tmp` due limited tmpfs space during cargo linking. Retried with replay extraction outside parent workspaces.
- Replay then exposed current package traceability gaps because fallback logic ignored `handoffs/p27`; fixed in `crates/aidens-cli/src/package.rs`.
- Replay then exposed shell-script executable-bit dependence after zip extraction; fixed by invoking scripts through `bash`.
- Initial full workspace test exposed a stale contextless patch test fixture; fixed to align with Phase 10 patch ambiguity rules.
- Clippy exposed an equivalent-branch warning; fixed without behavior change.

Diagnostic failed logs are retained for audit:

- `target/p27/audit/package_self_replay_phase19_full.log`
- `target/p27/audit/package_self_replay_phase19_full_tmpdir.log`
- `target/p27/audit/package_self_replay_phase19_full_home_tmp.log`
- `target/p27/audit/package_self_replay_phase19_full_outside_workspace.log`
- `target/p27/audit/package_self_replay_phase19_after_traceability_fix_full.log`
- `target/p27/audit/cargo_test_phase19.log`
- `target/p27/audit/cargo_clippy_phase19.log`

## Support-Tier Changes

No support-tier claim changed. Phase 19 repaired validation and replay honesty surfaces only.

## Canonical Ownership

No canonical-owner boundary changed. Completion-audit and package replay artifacts remain AiDENs-local operator evidence. Canonical sibling ownership remains delegated as described in `AGENTS.md`.

## Exact / Approx / Degradation Labels

No runtime semantic label contract changed. Phase 19 produced new evidence artifacts:

- `target/p27/audit/package_self_replay_phase19_after_replay_script_exec_fix_full_receipt.json` reports `semantic_status=exact_check`.
- `target/p27/audit/package_self_replay_phase19_after_replay_script_exec_fix_skip_cargo_receipt.json` reports `semantic_status=degraded_exact_check`.
- `docs/p27/P27_FINAL_AUDIT_REPORT.md` labels itself as a local operator audit artifact and distinguishes exact command evidence from degraded skip-cargo replay evidence.

## Quarantine

No issues quarantined.

Decision: continue
