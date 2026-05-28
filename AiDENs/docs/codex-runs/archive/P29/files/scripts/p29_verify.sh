#!/usr/bin/env bash
set -euo pipefail

cargo fmt --all -- --check
cargo check --workspace --all-targets
cargo test --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
cargo doc --workspace --no-deps

python3 scripts/assert_p29_run_identity.py
python3 scripts/assert_p29_no_archived_current_run.py
python3 scripts/assert_p29_manifest_paths.py
python3 scripts/assert_p29_current_docs_active.py
python3 scripts/assert_p29_final_package_contains_verifier.py
python3 scripts/assert_p29_no_marker_only_hard_gates.py
python3 scripts/assert_p29_v11a_contracts.py
python3 scripts/assert_p29_receipt_chain.py
python3 scripts/assert_p29_boundary_profiles.py
python3 scripts/assert_p29_proof_debt.py
python3 scripts/assert_p29_contracts_megafile_containment.py
python3 scripts/assert_p29_cli_megafile_containment.py
python3 scripts/assert_p29_module_ownership_boundaries.py
python3 scripts/assert_no_canonical_type_duplicates.py
python3 scripts/assert_p29_v11b_seed_surfaces.py
python3 scripts/assert_p29_audit_matrix_closure.py
python3 scripts/assert_p29_no_forbidden_claims.py
