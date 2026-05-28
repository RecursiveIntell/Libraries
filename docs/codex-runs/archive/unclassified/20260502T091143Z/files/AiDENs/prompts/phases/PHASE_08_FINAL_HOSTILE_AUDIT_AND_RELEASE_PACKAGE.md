# Phase 08 — Final Hostile Audit and Release Package

## Tasks

1. Run all P22 gates.
2. Build normal codex-context archive with `z.py`.
3. Build deliberate audit-full dry-run/package to prove archived history can be included intentionally.
4. Replay package verification.
5. Produce final handoff docs.

## Mandatory commands

```bash
cargo fmt --all --check
cargo check --workspace --all-targets --all-features
cargo test --workspace --all-targets --all-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
python3 scripts/p22_zpy_archival_selftest.py
python3 scripts/assert_p22_zpy_archive_contract.py z.py
python3 scripts/assert_p22_codex_archival_hygiene.py .
P22_REQUIRE_CARGO=1 bash scripts/p22_verify.sh
bash scripts/p22_verify_release_archive.sh target/p22/aidens-p22-release-context.zip
python3 z.py --root . --profile aidens --mode codex-context --strict
python3 z.py --root . --profile aidens --mode audit-full --include-codex-archive --strict --dry-run
```

## Final Handoff

Create `handoffs/p22/FINAL_AUDIT_REPORT.md` with:

- supported/partial/scaffold/deferred/quarantined/failed matrix;
- commands run and results;
- package SHA-256;
- archive normalization summary;
- exact remaining risks;
- changed-file summary;
- final package paths;
- hostile-auditor notes.
