# Script findings

This restored root pack was validated inside the working source snapshot with the static gates that do not require cargo in this container.

## Verified here

- `python3 scripts/generate_closeout_receipt.py` — pass
- `bash scripts/check_repo_surface.sh` — pass
- `bash scripts/check_doc_truth.sh` — pass
- `bash scripts/check_manifest_truth.sh` — pass
- `bash scripts/check_schema_registry_uniqueness.sh` — pass
- `bash scripts/check_no_prod_panics.sh` — pass
- `bash scripts/check_mirror_discipline.sh` — pass
- `bash scripts/check_hotspot_budgets.sh` — pass
- `python3 scripts/check_public_type_drift.py` — pass with zero allowlisted duplicates
- `python3 scripts/check_root_archive_manifest.py` — pass
- `python3 scripts/check_public_api_docs.py` — pass
- `python3 scripts/check_closeout_receipt.py` — pass
- `python3 scripts/check_current_closeout_lane.py` — pass

## Not build-certified here

Cargo is not available in this environment, so cargo-dependent checks were not rerun in-container here. The active hardening evidence manifest remains the authority for those commands.

## Interpretation

The missing front-door problem is fixed by this pack at the document/control-plane level. The remaining finish-line work is the public proof package:

- DEMO-001 — one narrated v21 -> v22 -> v23 path
- BENCH-001 — one benchmark / forge-bench score sheet
- ARCH-001 — final physical root reduction
