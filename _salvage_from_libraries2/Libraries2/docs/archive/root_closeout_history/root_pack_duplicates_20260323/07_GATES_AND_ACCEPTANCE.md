# Test and conformance plan

## Current hardening gates

Run these commands for the active lane:

- `bash scripts/check_repo_surface.sh`
- `bash scripts/check_doc_truth.sh`
- `bash scripts/check_manifest_truth.sh`
- `bash scripts/check_schema_registry_uniqueness.sh`
- `bash scripts/check_no_prod_panics.sh`
- `bash scripts/check_mirror_discipline.sh`
- `bash scripts/check_hotspot_budgets.sh`
- `python3 scripts/check_public_type_drift.py`
- `python3 scripts/check_root_archive_manifest.py`
- `python3 scripts/check_public_api_docs.py`
- `python3 scripts/generate_closeout_receipt.py`
- `python3 scripts/check_closeout_receipt.py`

When cargo is available, also run:

- `cargo run -p contract-schema-gen -- schemas.generated`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `CARGO_HOME=/tmp/cargo-home-v25-verify CARGO_TARGET_DIR=/tmp/cargo-target-v25-verify bash scripts/check_schema_compat.sh`
- `CARGO_HOME=/tmp/cargo-home-v25-verify CARGO_TARGET_DIR=/tmp/cargo-target-v25-verify cargo test -p verification-policy --test v25_policy_citation_flow`
- `CARGO_HOME=/tmp/cargo-home-v25-verify CARGO_TARGET_DIR=/tmp/cargo-target-v25-verify cargo test -p verification-control --test v25_citation_requirements --test v25_review_case_roundtrip`
- `CARGO_HOME=/tmp/cargo-home-v25-verify CARGO_TARGET_DIR=/tmp/cargo-target-v25-verify cargo test -p contract-schema-gen --test no_prod_panics`
- `CARGO_HOME=/tmp/cargo-home-v25-verify CARGO_TARGET_DIR=/tmp/cargo-target-v25-verify cargo test --manifest-path living-memory/living-memory/Cargo.toml --test export_tests`

## Demonstrator completion gates

DEMO-001 is done only when:

- one README explains the entire v21 -> v22 -> v23 path,
- one stitched demo bundle exists,
- one test or script validates the stitched path,
- the demo does not invent new schema families,
- and the demo remains consumer-only with respect to orchestration.

## Benchmark completion gates

BENCH-001 is done only when:

- the benchmark questions are frozen,
- fixture inputs and expected outputs are published,
- one score sheet is emitted,
- and replayability / temporal correctness are measured directly.
