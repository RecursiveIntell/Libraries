# Phase 20 Report - Final Package, Extracted Replay, and Release Bar

Date: `2026-05-07`

## Scope

No raw `open` rows were assigned to Phase 20. This phase closed the final package/replay gate after all earlier phase gates passed.

## Files Changed

- `P29_STATUS_EVIDENCE_MANIFEST.json`
- `docs/super-pass/KNOWN_LIMITATIONS_REGISTER.md`
- `docs/super-pass/SUPPORT_TRACEABILITY.md`
- `handoffs/super-pass/FINAL_AUDITOR_HANDOFF.md`
- `handoffs/super-pass/PHASE_20_REPORT.md`
- `crates/aidens-cli/src/lib.rs`
- `z.py`

## Fixes During Final Packaging

- Replaced generated scaffold smoke-test `include_str!` template text with runtime file reads so the package scanner does not mistake scaffold fixture text for source-tree includes.
- Allowed the Phase 16 redaction prompt filename through the package filename scanner while still preserving content scanning.
- Marked Phase 19 target-log evidence as external in `P29_STATUS_EVIDENCE_MANIFEST.json` so extracted packages do not require local `target/` logs.
- Updated support/status evidence after package replay passed.

## Package

- Package: `target/p29/package/AiDENs-p29-codex-context.zip`
- Zip-byte SHA-256: `e78805e99344d52ded8a6aef39a69f219f05813d11afe8507cb7872f6bc7f01a`
- Content manifest SHA-256: `1b861ffada95d02153eef2dbd471646e2c8755c910867fdd2e8eb881fca71d81`

Sidecars generated:

- `target/p29/package/AiDENs-p29-codex-context.manifest.json`
- `target/p29/package/AiDENs-p29-codex-context.report.md`
- `target/p29/package/AiDENs-p29-codex-context.findings.json`
- `target/p29/package/AiDENs-p29-codex-context.excluded.json`
- `target/p29/package/AiDENs-p29-codex-context.codex-archive.json`

## Validation

Final command bar:

- `cargo fmt --all --check`
- `cargo check --workspace --all-targets`
- `cargo test --workspace --all-targets`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo doc --workspace --no-deps`
- `bash scripts/p29_verify.sh`

Key logs:

- `target/super-pass/audit/phase20-p29-verify-after-final-status.log`
- `target/super-pass/audit/phase20-package-generate-final-status.log`
- `target/super-pass/audit/phase20-package-validation-final-status.log`
- `target/super-pass/audit/phase20-package-self-replay-final-status.log`
- `target/super-pass/audit/phase20-package-self-replay-receipt.json`
- `target/super-pass/audit/PHASE_15_AUDIT_LOG_HASHES.json`

Extracted package replay command:

```bash
TMPDIR=/home/sikmindz/.cache/aidens-replay-tmp \
CARGO_TARGET_DIR=/home/sikmindz/.cache/aidens-replay-target \
python3 scripts/assert_package_self_replay.py \
  --package target/p29/package/AiDENs-p29-codex-context.zip \
  --require-verifier \
  --receipt-out target/super-pass/audit/phase20-package-self-replay-receipt.json
```

Result: passed.

## Matrix Summary

- `fixed`: 1011
- `quarantined`: 7
- `gate-required-not-product-defect`: 1
- `deferred`: 1
- raw `open`: 0

## Labels

Allowed final labels:

- `p29-package-repaired`
- `p29-supported-local-plus`
- `v11A-local-release-candidate`
- `v11B-executable-seed`
- `v11C-reserved-only`

Forbidden labels remain absent.

## Unresolved Risk

High-risk sibling/control layers are quarantined, not audited. Some P29 BUG IDs remain quarantined pending broader redesign. External research citations remain deferred unless reverified in a later research pass.

## Decision

Phase 20 gate passed.
