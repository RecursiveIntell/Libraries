# Phase 07 — Fresh Unzip Certification

- archive generated at: ${ARCHIVE}
- manifest generated at: ${MANIFEST}
- unpacked path: ${EXTRACTED}
- commands run from fresh unzipped material:
  - python3 scripts/validate_schemas.py
  - bash scripts/verify_golden_fixtures.sh
  - python3 scripts/validate_codex_pack.py
  - python3 scripts/assert_codex_active_pack.py
  - bash scripts/run_all_checks.sh
