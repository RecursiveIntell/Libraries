# P29 Verifier Spec

Create and include:

```text
scripts/p29_verify.sh
```

## Required commands

```bash
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
python3 scripts/assert_p29_v11a_contracts.py
python3 scripts/assert_p29_receipt_chain.py
python3 scripts/assert_p29_boundary_profiles.py
python3 scripts/assert_p29_proof_debt.py
python3 scripts/assert_p29_v11b_seed_surfaces.py
python3 scripts/assert_p29_audit_matrix_closure.py
```

## Required behavior

The verifier must fail if:

- current run is not P29;
- P29 artifacts are archived as stale;
- P29 status manifest references missing paths;
- verifier is missing from final package;
- v11A local labels are claimed without evidence;
- v11B completion is claimed;
- v11C completion is claimed;
- unresolved P0/P1 issues are unclassified.
