# Required final state

A successful run leaves the repo in this state:

## z.py

- Includes `.pyi` files.
- Includes `py.typed` marker files.
- Includes current Codex command receipts/logs in context/audit modes.
- Automatically archives root package artifacts before packaging.
- Emits root package archival summary in report and manifest.
- Provides verify-only and archive-only modes for root package hygiene.
- Fails strict packaging if required package hygiene or self-containment gates fail.

## Repo root

Allowed root files only. No previous package sidecars, previous context zips, old codex archive JSONs, `README_BUNDLE.md`, or `BUNDLE_MANIFEST.json` remain active after cleanup.

## Package archive

The generated package manifest must contain:

- `python/poly_kv/_native.pyi`
- `python/poly_kv/py.typed`
- current run command evidence (`commands_run.log` or `commands_run.receipts.jsonl`)
- root package hygiene archive summary
- zero findings under strict mode

## New validation tools

- `scripts/assert_source_package_hygiene.py`
- `scripts/test_zpy_hygiene_regression.py` or equivalent unit test/fixture
- `scripts/build_handoff_package.sh`

## Final evidence

The final Codex report must include:

- changed files
- commands run
- package output path
- manifest path
- root files before/after cleanup
- included/excluded evidence for `_native.pyi`, `py.typed`, and command logs
- z.py regression test results
- rollback instructions
