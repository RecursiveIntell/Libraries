# P28 Phase 10 Report

## Scope

Hardened package/replay truth, made P28 verifier targets real, labeled archive hash semantics, added content-manifest hashing, tightened text/binary detection, and proved full package self-replay from a fresh extracted package.

## Files changed

- `z.py`
- `scripts/assert_current_run_truth.py`
- `scripts/assert_package_validation.py`
- `scripts/assert_package_self_replay.py`
- `scripts/assert_p28_package_validation_paths.py`
- `scripts/assert_p28_zpy_text_detection.py`
- `scripts/p28_verify.sh`
- `scripts/verify_current.sh`
- `crates/aidens-cli/src/agent.rs`
- `crates/aidens-cli/src/tests.rs`
- `crates/aidens-contracts/src/lib.rs`
- `crates/aidens-contracts/src/tests.rs`
- `crates/aidens-runner/src/lib.rs`
- `handoffs/p28/PHASE_10_REPORT.md`

## Claims made

- Claim: `safe_relative` fails closed and remains regression-tested.
  - status: pass
  - evidence: `target/p28/audit/assert_p28_zpy_safe_relative_phase10.log`
- Claim: non-UTF-8 and NUL-bearing text-like files are rejected instead of lossy-decoded.
  - status: pass
  - evidence: `target/p28/audit/assert_p28_zpy_text_detection_phase10.log`
- Claim: zip hash is labeled as zip-byte hash, not canonical content hash.
  - status: pass
  - evidence: `target/p28/audit/zpy_package_phase10_after_runner_receipt_scope.log`
- Claim: package manifest includes top-level sidecar references and separate `content_manifest_sha256`.
  - status: pass
  - evidence: `target/p28/audit/assert_package_validation_phase10_after_runner_receipt_scope.log`
- Claim: `scripts/verify_current.sh` targets real P28 verifier logic.
  - status: pass
  - evidence: `target/p28/audit/verify_current_p28_skip_cargo_phase10_final.log`
- Claim: package self-replay full pass succeeds from a fresh extracted package.
  - status: pass
  - evidence: `target/p28/audit/assert_package_self_replay_phase10_after_runner_receipt_scope.log`, `target/p28/audit/package_self_replay_phase10_after_runner_receipt_scope_receipt.json`

## Evidence

- `target/p28/audit/python_compile_phase10_closeout.log`
- `target/p28/audit/assert_p28_zpy_text_detection_phase10.log`
- `target/p28/audit/assert_p28_zpy_safe_relative_phase10.log`
- `target/p28/audit/assert_p28_package_validation_paths_phase10.log`
- `target/p28/audit/bash_n_p28_verify_phase10.log`
- `target/p28/audit/verify_current_p28_skip_cargo_phase10_final.log`
- `target/p28/audit/cargo_check_phase10_after_runner_receipt_scope.log`
- `target/p28/audit/zpy_package_phase10_after_runner_receipt_scope.log`
- `target/p28/audit/assert_package_validation_phase10_after_runner_receipt_scope.log`
- `target/p28/audit/assert_package_self_replay_phase10_after_runner_receipt_scope.log`
- `target/p28/audit/package_self_replay_phase10_after_runner_receipt_scope_receipt.json`

## Tests run

```bash
python3 -m py_compile z.py scripts/assert_p28_zpy_text_detection.py scripts/assert_package_validation.py
python3 scripts/assert_p28_zpy_safe_relative.py
python3 scripts/assert_p28_zpy_text_detection.py
python3 scripts/assert_p28_package_validation_paths.py
bash -n scripts/p28_verify.sh scripts/verify_current.sh
P28_SKIP_CARGO=1 bash scripts/verify_current.sh
cargo fmt --all -- --check
cargo check --workspace --all-targets
cargo test -p aidens-cli agent_run_persists_v3_bundle_in_receipt_store_and_inspects_after_restart
cargo test -p aidens-cli package_completion_audit_reports_deferred_horizon_without_healthy_claims
cargo test -p aidens-contracts p19_completion_audit_discloses_deferred_horizon_and_blocks_release_bar
cargo test -p aidens-runner p26_plan_act_verify_loop
python3 z.py --root . --profile aidens --mode next-codex-context --strict --codex-current-run P28 --output target/p28/package/AiDENs-p28-codex-context.zip
python3 scripts/assert_package_validation.py
TMPDIR=/home/sikmindz/p28-replay-tmp CARGO_TARGET_DIR=/home/sikmindz/p28-replay-cargo-target python3 scripts/assert_package_self_replay.py --package target/p28/package/AiDENs-p28-codex-context.zip --verifier scripts/verify_current.sh --require-verifier --receipt-out target/p28/audit/package_self_replay_phase10_after_runner_receipt_scope_receipt.json
```

## Failures / degraded checks

- Initial self-replay failed on stale `scripts/verify_current.sh` delegation to missing `scripts/p27_verify.sh`; fixed with `scripts/p28_verify.sh`.
- Full replay under `/tmp` failed with `No space left on device`; rerun used `/home/sikmindz/p28-replay-tmp`.
- Replay exposed stale CLI/contracts tests that allowed deferred horizon to pass the release bar; fixed to block false-green release claims.
- Replay exposed shared runner receipt-log paths under parallel tests; fixed with process/sequence-scoped default receipt roots.

## Open risks

- External temp and Cargo target directories were required for full package self-replay in this environment due `/tmp` capacity.

## Next phase readiness

Ready: Phase 10 exit gate passed with strict package generation, package validation, and full package self-replay receipt.
