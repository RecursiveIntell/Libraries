# Phase 01 — Implement `z.py` Codex Archival Normalization

## Tasks

Implement the `z.py` archival subsystem described in `docs/p22/P22_ZPY_CODEX_ARCHIVAL_SPEC.md`.

Required implementation characteristics:

- stdlib-only;
- deterministic archive paths;
- no overwrite of existing archive files;
- `--dry-run` produces a planned archive report with no mutation;
- `--archive-only` performs normalization and exits without zipping;
- `--verify-codex-archive-hygiene` verifies current cleanliness;
- `--include-codex-archive` deliberately includes archives;
- strict mode fails on remaining active stale run artifacts.

## Tests to add or adapt

- `scripts/p22_zpy_archival_selftest.py`
- `scripts/assert_p22_zpy_archive_contract.py`

## Acceptance Gate

```bash
python3 scripts/p22_zpy_archival_selftest.py
python3 scripts/assert_p22_zpy_archive_contract.py z.py
python3 z.py --root . --profile aidens --mode codex-context --dry-run --strict
```

Dry-run must show planned archival action or verified clean state, and must not write a zip unless requested by normal build flow.
