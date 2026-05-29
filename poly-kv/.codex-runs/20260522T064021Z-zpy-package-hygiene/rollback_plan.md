# Rollback Plan

1. Revert source/script edits:
   - `z.py`
   - `scripts/assert_source_package_hygiene.py`
   - `scripts/test_zpy_hygiene_regression.py`
   - `scripts/build_handoff_package.sh`
   - `scripts/preflight_next_pass.sh`
   - `scripts/run_next_validation.sh`
   - `scripts/assert_no_boundary_drift.py`
2. Remove generated fresh package sidecars:
   - `poly-kv-generic-rust-next-codex-context-20260522T064721Z.zip`
   - `poly-kv-generic-rust-next-codex-context-20260522T064721Z.manifest.json`
   - `poly-kv-generic-rust-next-codex-context-20260522T064721Z.report.md`
   - `poly-kv-generic-rust-next-codex-context-20260522T064721Z.excluded.json`
   - `poly-kv-generic-rust-next-codex-context-20260522T064721Z.findings.json`
   - `poly-kv-generic-rust-next-codex-context-20260522T064721Z.codex-archive.json`
3. Restore archived root package artifacts from `docs/source-packages/archive/20260522T064721Z/files/` to repo root. Verify each restored file against `docs/source-packages/archive/20260522T064721Z/PACKAGE_ARTIFACT_ARCHIVE_MANIFEST.json`.
4. Remove this run directory if the run itself must be rolled back: `.codex-runs/20260522T064021Z-zpy-package-hygiene/`.

No compression semantics, codec math, receipt schema, or Rust runtime behavior was changed in this pass.
