# P28 Commands

## Phase-local commands

```bash
cargo fmt --all -- --check
cargo check --workspace --all-targets
cargo test --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
```

## Targeted test suggestions

Use exact test names once implemented. Suggested module/test names:

```bash
cargo test -p aidens-contracts v11_artifact_lifecycle
cargo test -p aidens-contracts proof_waiver_is_not_proof
cargo test -p aidens-contracts degraded_surface_blocks_release_readiness
cargo test -p aidens-contracts aggregate_status_downgrades_on_degraded_subcheck
cargo test -p aidens-tool-kit symlink_escape_is_blocked
cargo test -p aidens-tool-kit patch_failure_leaves_no_dirty_dirs
cargo test -p aidens-tool-kit timeout_output_is_marked_partial
cargo test -p aidens-boundary-kit duplicate_key_rejected
cargo test -p aidens-boundary-kit repair_emits_treatment_integrity_receipt
cargo test -p aidens-runner done_without_receipts_is_blocked
cargo test -p aidens-receipts event_log_digest_chain_detects_tamper
```

## Final commands

```bash
cargo fmt --all -- --check | tee target/p28/audit/cargo_fmt_p28_final.log
cargo check --workspace --all-targets | tee target/p28/audit/cargo_check_p28_final.log
cargo test --workspace --all-targets | tee target/p28/audit/cargo_test_p28_final.log
cargo clippy --workspace --all-targets -- -D warnings | tee target/p28/audit/cargo_clippy_p28_final.log
cargo doc --workspace --no-deps | tee target/p28/audit/cargo_doc_p28_final.log
P28_FINAL_STRICT=1 bash scripts/verify_current.sh | tee target/p28/audit/verify_current_p28_final.log
python3 z.py --root . --profile aidens --mode next-codex-context --strict --codex-current-run P28 --output target/p28/package/AiDENs-p28-codex-context.zip | tee target/p28/audit/zpy_package_p28_final.log
python3 scripts/assert_package_validation.py | tee target/p28/audit/package_validation_p28_final.log
python3 scripts/assert_package_self_replay.py --package target/p28/package/AiDENs-p28-codex-context.zip --verifier scripts/verify_current.sh --require-verifier --receipt-out target/p28/audit/package_self_replay_p28_final_receipt.json | tee target/p28/audit/package_self_replay_p28_final.log
```

## Honest degraded replay command

Only run if required; it must downgrade aggregate status.

```bash
P28_SKIP_CARGO=1 python3 scripts/assert_package_self_replay.py --package target/p28/package/AiDENs-p28-codex-context.zip --verifier scripts/verify_current.sh --require-verifier --receipt-out target/p28/audit/package_self_replay_p28_skip_cargo_receipt.json | tee target/p28/audit/package_self_replay_p28_skip_cargo.log
```
