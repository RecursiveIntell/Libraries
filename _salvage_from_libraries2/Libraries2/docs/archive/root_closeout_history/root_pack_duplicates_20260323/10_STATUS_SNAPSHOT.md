
# 10_STATUS_SNAPSHOT

## Current snapshot

### Green in this audit
- `bash scripts/check_doc_truth.sh`
- `python3 scripts/check_current_closeout_lane.py`
- `bash scripts/check_repo_surface.sh`
- `python3 scripts/check_public_api_docs.py`
- `python3 scripts/check_public_type_drift.py` **within its current scoped crate list**
- `bash scripts/check_hotspot_budgets.sh`
- `python3 scripts/check_v25_json_surface.py`
- `bash scripts/check_schema_registry_uniqueness.sh`

### Red in this audit
- `bash scripts/check_pack_truth.sh`
- `python3 scripts/check_root_archive_manifest.py`
- `bash scripts/check_no_prod_panics.sh`
- `bash scripts/check_v25_repo_truth.sh`
- `bash scripts/run_v25_local_checks.sh`
- `bash scripts/run_v25_production_pack_checks.sh`
- `python3 scripts/check_v25_production_closure.py`

### Not re-certified here
- `bash scripts/check_schema_compat.sh`
- the supported cargo lane
- any claim that depends on local Rust toolchain execution

## Snapshot interpretation

The core repo is stronger than the current release story.
The release story is weaker than the core repo.
That is why this pack starts with release truth, not architecture.
