# Apply plan

## This pack creates or restores

- the full root control-plane pack,
- the front-door aliases expected by the scripts,
- the audit reconciliation doc,
- the support profile and crate-boundary map,
- the implementation / benchmark / risk / prompt docs.

## Then do

1. `python3 scripts/generate_closeout_receipt.py`
2. `bash scripts/check_repo_surface.sh`
3. `bash scripts/check_doc_truth.sh`
4. `bash scripts/check_manifest_truth.sh`
5. `bash scripts/check_schema_registry_uniqueness.sh`
6. `bash scripts/check_no_prod_panics.sh`
7. `bash scripts/check_mirror_discipline.sh`
8. `bash scripts/check_hotspot_budgets.sh`
9. `python3 scripts/check_public_type_drift.py`
10. `python3 scripts/check_root_archive_manifest.py`
11. `python3 scripts/check_public_api_docs.py`
12. `python3 scripts/check_closeout_receipt.py`

## After the root pack is green

Move to DEMO-001, then BENCH-001, then ARCH-001.
