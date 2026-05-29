# Phase 02 Report - Root Package Artifact Archival

## Changes

- `z.py`: added `RootPackageArchiveResult` and `archive_root_package_artifacts`.
- `z.py`: added CLI flags:
  - `--archive-root-package-artifacts`
  - `--no-archive-root-package-artifacts`
  - `--verify-root-package-hygiene`
  - `--root-package-archive-root`
  - `--root-package-archive-dry-run`
  - `--include-root-package-archive`
- `z.py`: defaulted root package artifact archival for `next-codex-context`, `codex-run-full`, and `audit-full`.
- `z.py`: archives root package residue into `docs/source-packages/archive/<UTC_STAMP>/files/` and writes `PACKAGE_ARTIFACT_ARCHIVE_MANIFEST.json`.
- `z.py`: root Markdown hygiene now defers package artifacts to the package artifact archival pass.

## Commands

- `python3 z.py --help | rg -n "root-package|pyi|commands|archive-root-package"` - confirmed CLI exposure.
- `python3 z.py --root . --profile generic-rust --mode next-codex-context --verify-root-package-hygiene --no-strict --output poly-kv-generic-rust-next-codex-context-verify.zip` - verify-only mode found 14 current root package artifacts, as expected before archive mode.
- `python3 scripts/test_zpy_hygiene_regression.py` - passed.

## Manual Guardrail

1. Source-of-truth owner: `z.py` owns package archival policy and generated package artifact archive manifests.
2. No duplicate compression, receipt, fallback, or runtime authority introduced.
3. No schema widening, shape coercion, hidden fallback, or compatibility layer introduced.
4. Material operation added: root package artifact movement. Each move or same-hash removal is recorded in `PACKAGE_ARTIFACT_ARCHIVE_MANIFEST.json` and summarized in the package manifest/report.
5. Exact fallback/compressed fixtures not touched.
6. Optional adapters not touched.
7. Regression test covers archive mode and required manifest inclusion.
8. Verify-only reports current root residue as expected pre-archive.
9. Scope exclusions hold.
10. Rollback path: restore archived files from `docs/source-packages/archive/<stamp>/files/` using manifest SHA-256 records, then revert `z.py`.
