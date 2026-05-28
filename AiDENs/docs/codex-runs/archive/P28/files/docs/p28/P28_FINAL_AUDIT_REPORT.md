# P28 Final Audit Report

## Scope

P28 implemented the v11A constitutional material-operation kernel for the declared supported-local AiDENs path. It added typed artifact transitions, manifests, receipts, execution context, operator contracts, boundary compiler/treatment integrity law, proof/debt/waiver semantics, degradation/view disclosure, package replay hardening, reserved v11B/v11C containment, and adversarial conformance tests.

## Result

Passed for `p28-supported-local-plus` and `v11A-conformant-core:declared-local-agent-path`.

P28 does not claim production cloud readiness, broad autonomy, active v11B regional runtime, active v11C federation/mechanism/self-hosting, or canonical truth ownership.

## Final Evidence

| Check | Result | Evidence |
|---|---|---|
| `cargo fmt --all -- --check` | pass | `target/p28/audit/cargo_fmt_p28_final.log` |
| `cargo check --workspace --all-targets` | pass | `target/p28/audit/cargo_check_p28_final.log` |
| `cargo test --workspace --all-targets` | pass | `target/p28/audit/cargo_test_p28_final.log` |
| `cargo clippy --workspace --all-targets -- -D warnings` | pass after two small style repairs | `target/p28/audit/cargo_clippy_p28_final_after_second_fix.log` |
| `cargo doc --workspace --no-deps` | pass | `target/p28/audit/cargo_doc_p28_final.log` |
| `P28_FINAL_STRICT=1 bash scripts/verify_current.sh` | pass | `target/p28/audit/verify_current_p28_final.log` |
| strict package generation | pass | `target/p28/audit/zpy_package_p28_final.log` |
| package validation | pass | `target/p28/audit/assert_package_validation_p28_final.log` |
| package self-replay | pass with `/home` temp/target dirs after `/tmp` ENOSPC | `target/p28/audit/package_self_replay_p28_final_receipt.json` |

## Package

- Package: `target/p28/package/AiDENs-p28-codex-context.zip`
- Zip-byte SHA-256 and content manifest SHA-256 are recorded in `target/p28/package/AiDENs-p28-codex-context.manifest.json`. The zip-byte hash is not a canonical content hash.
- Sidecars: `target/p28/package/AiDENs-p28-codex-context.{manifest,report,findings,excluded,codex-archive}.json`

## Environment Note

The exact package self-replay command first failed because `/tmp` ran out of space while linking cargo test binaries. That failed receipt is preserved at `target/p28/audit/package_self_replay_p28_final_receipt_tmp_space_failed.json`. The successful replay used `TMPDIR=/home/sikmindz/p28-replay-tmp` and `CARGO_TARGET_DIR=/home/sikmindz/p28-replay-cargo-target`; the successful receipt is installed at the required final path.

## Remaining Limits

See `docs/p28/P28_KNOWN_LIMITATIONS_REGISTER.md`. The main limits are no hosted cloud support, no broad autonomy, no active v11B/v11C runtime, and no AiDENs-local canonical truth ownership.
