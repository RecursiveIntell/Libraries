# P22 Phase 06 Report - Assertion Suite and CI Gates

## Scope

Phase 06 made the P22 assertion suite and cargo-enforced verifier the active local/CI gate. It also repaired tests that still assumed archived P20/P21 docs or pre-P22 task JSON remained active.

## Global Invariant Revalidation

- AiDENs directs/wires/packages only: `pass`.
- Canonical stack libraries own truth; no AiDENs substitutes introduced: `pass`.
- No stale Codex-run artifact remains active except current P22 files: `pass`.
- Historical run material archived, not deleted: `pass`.
- Existing archives not rewritten: `pass`.
- `z.py` remains strict, deterministic, stdlib-only, and source-closure aware: `pass`.
- No compatibility shim, shadow truth store, hidden database, or silent semantic widening: `pass`.
- Provider/API-key material redacted in reports/package outputs: `pass`; current findings do not print values.
- Support claims backed by executable proof: `pass for Phase 06`.
- Blocking invariant failures after repair: `none`.

## Post-Phase 05 Guardrail

Status: `PASS`.

- Phase 05 acceptance gate status: `pass`.
- Codex artifacts archived/skipped/left active: post-Phase 05 dry-run planned `0`, moved `0`, skipped `0`, active-after `0`.
- Existing archives left untouched: `pass`.
- `z.py` deterministic and strict: `pass`.
- Stale P20/P21/P22 run instruction contamination risk: `pass`.
- AiDENs local substitute for canonical library truth introduced: `pass`; scaffold/shadow/substitute checks pass.
- Cargo/tests/assertions: `cargo check --workspace --all-targets --all-features`, targeted cargo tests, `bash scripts/p22_verify.sh`, and P22 assertions pass.
- Stop/repair/quarantine required: `none`.

## Work Performed

1. Made `.github/workflows/ci.yml` run `P22_REQUIRE_CARGO=1 bash scripts/p22_verify.sh`.
2. Changed `scripts/verify.sh` to delegate to `scripts/p22_verify.sh` as a compatibility entrypoint.
3. Added `scripts/p22_secret_scan_fixture_test.py` to `scripts/p22_verify.sh`.
4. Updated P22 CI/operator docs:
   - `docs/p22/08_CI_AND_COMMANDS.md`
   - `docs/p22/RE_RUN_AND_VERIFICATION.md`
   - `docs/p22/OPERATOR_QUICKSTART.md`
   - `docs/p22/04_ACCEPTANCE_GATES.md`
5. Updated completion-audit gate truth from pre-P22 commands to `P22_REQUIRE_CARGO=1 bash scripts/p22_verify.sh`.
6. Repaired completion-audit traceability to use active P22 handoff reports when active task JSON has been archived.
7. Repaired integration/provider tests that still required active P21/P20 doc paths.

## Verification

- `python3 scripts/assert_p22_zpy_archive_contract.py z.py` -> pass.
- `python3 scripts/assert_p22_codex_archival_hygiene.py .` -> pass.
- `bash -n scripts/p22_verify.sh scripts/p22_verify_release_archive.sh scripts/verify.sh` -> pass.
- `python3 -m py_compile scripts/assert_p22_codex_archival_hygiene.py scripts/assert_p22_release_package_clean.py scripts/assert_p22_zpy_archive_contract.py scripts/p22_zpy_archival_selftest.py scripts/p22_secret_scan_fixture_test.py z.py` -> pass.
- `cargo test -p aidens-cli package_completion_audit_reports_deferred_horizon_without_healthy_claims -- --nocapture` -> pass.
- `cargo test -p aidens-contracts p19_completion_audit_discloses_deferred_horizon_without_blocking_release_bar -- --nocapture` -> pass.
- `cargo test -p aidens-integration-tests --test phase_07_recall_extraction` -> pass.
- `cargo test -p aidens-provider-kit p20_provider_capability_matrix_matches_executable_truth -- --nocapture` -> pass.
- `P22_REQUIRE_CARGO=1 bash scripts/p22_verify.sh` -> pass.
- `bash scripts/verify.sh` -> pass.
- `python3 scripts/assert_p22_release_package_clean.py --manifest target/p22/audit/p22_verify_codex_context.manifest.json` -> pass.
- Final post-report `python3 z.py --root . --profile aidens --mode codex-context --strict --dry-run --output target/p22/audit/phase06_acceptance_codex_context.zip ...` -> pass; included `1266`, archive planned `0`, moved `0`, active-after `0`.
- Final post-report `bash scripts/p22_verify.sh` -> pass.

The cargo-enforced verifier ran:

- `cargo fmt --all --check`
- `cargo check --workspace --all-targets --all-features`
- `cargo test --workspace --all-targets --all-features`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`

## Archive / Quarantine Status

- Codex artifacts archived in Phase 06: `0`.
- Codex artifacts skipped: `0`.
- Active stale artifacts after Phase 06: `0`.
- Existing archive roots rewritten: `0`.
- Files deleted: `0`.
- Files quarantined outside archive: `0`.

## Changed Files

- `.github/workflows/ci.yml`
- `scripts/verify.sh`
- `scripts/p22_verify.sh`
- `docs/p22/08_CI_AND_COMMANDS.md`
- `docs/p22/RE_RUN_AND_VERIFICATION.md`
- `docs/p22/OPERATOR_QUICKSTART.md`
- `docs/p22/04_ACCEPTANCE_GATES.md`
- `crates/aidens-cli/src/lib.rs`
- `crates/aidens-contracts/src/lib.rs`
- `crates/aidens-integration-tests/tests/phase_07_recall_extraction.rs`
- `crates/aidens-provider-kit/src/lib.rs`
- `STATUS.md`
- `MANIFEST.json`
- `docs/codex-runs/CURRENT_RUN.md`
- `handoffs/p22/PHASE_06_REPORT.md`
- `target/p22/audit/p22_verify_*`
- `target/p22/audit/phase06_acceptance_codex_context.*`
- `target/p22/audit/cargo_*`
- `target/p22/audit/COMMAND_LOG_SUMMARY.md`
- `target/p22/audit/CHANGED_FILE_SUMMARY.md`
- `target/p22/audit/UNRESOLVED_RISKS.md`

## Commands Run

- `python3 scripts/assert_p22_zpy_archive_contract.py z.py`
- `python3 scripts/assert_p22_codex_archival_hygiene.py .`
- `python3 scripts/assert_p22_release_package_clean.py --manifest target/p22/audit/post_phase05_guardrail_codex_context.manifest.json`
- `python3 -m py_compile z.py scripts/assert_p22_release_package_clean.py scripts/assert_p22_codex_archival_hygiene.py scripts/assert_p22_zpy_archive_contract.py scripts/p22_zpy_archival_selftest.py scripts/p22_secret_scan_fixture_test.py`
- `bash scripts/assert_no_scaffold_promoted.sh && bash scripts/assert_no_local_substitute_dependencies.sh && bash scripts/assert_no_shadow_truth.sh`
- `cargo check --workspace --all-targets --all-features`
- `cargo test -p aidens-cli provider_route_does_not_claim_native_when_backend_is_unavailable`
- `bash scripts/p22_verify.sh`
- `sed -n` reads for Phase 06 prompt, verifier scripts, CI docs, workflow, and failing test files.
- `cargo test -p aidens-cli package_completion_audit_reports_deferred_horizon_without_healthy_claims -- --nocapture`
- `cargo test -p aidens-contracts p19_completion_audit_discloses_deferred_horizon_without_blocking_release_bar -- --nocapture`
- `cargo test -p aidens-integration-tests --test phase_07_recall_extraction`
- `cargo test -p aidens-provider-kit p20_provider_capability_matrix_matches_executable_truth -- --nocapture`
- `cargo fmt --all`
- `P22_REQUIRE_CARGO=1 bash scripts/p22_verify.sh`
- `bash scripts/verify.sh`
- `python3 z.py --root . --profile aidens --mode codex-context --strict --dry-run --output target/p22/audit/phase06_acceptance_codex_context.zip ...`
- `bash scripts/p22_verify.sh`
- `python3 scripts/assert_p22_release_package_clean.py --manifest target/p22/audit/p22_verify_codex_context.manifest.json`

## Remaining Risks

- Parent Git still reports `AiDENs/` as untracked from `/home/sikmindz/Coding/Libraries`.
- Protective filename exclusions remain for active Phase 05 secret-redaction prompt/test files until those current-run files are archived or renamed.

## Phase Boundary

Phase 06 acceptance gates pass. Phase 07 requires the next manual guardrail before continuing.
