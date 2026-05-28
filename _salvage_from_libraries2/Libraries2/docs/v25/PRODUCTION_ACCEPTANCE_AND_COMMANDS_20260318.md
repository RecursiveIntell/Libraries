# Production acceptance and commands — 2026-03-18

## Pack-only checks (should pass as soon as this handoff pack is present)

```bash
bash scripts/check_v25_production_pack_truth.sh
bash scripts/check_no_local_recomposition.sh
bash scripts/run_v25_production_pack_checks.sh
python3 scripts/audit_v25_production_gap.py > /tmp/v25-production-gap.json
```

## Final closure checks (must pass after Codex lands the code)

```bash
bash scripts/check_v25_repo_truth.sh
python3 scripts/check_v25_json_surface.py
bash scripts/check_no_local_recomposition.sh
python3 scripts/check_v25_production_closure.py
cargo run -p contract-schema-gen -- schemas
cargo run -p contract-schema-gen -- --check schemas
cargo test -p effect-runtime
cargo test -p verification-control
cargo test -p verification-policy
cargo test -p verification-adjudication
cargo test -p remote-oracle-admission
cargo test -p federated-settlement
cargo test -p profile-runtime
cargo test -p knowledge-runtime
bash apply/v25/SYNC_LIBRARIES_SOURCE_MIRROR.sh
```

## Release-bar questions

The pass is not done until the answer to each question is “yes”:

1. Does `effect-runtime` use typed IDs and direct composite constitutional citations?
2. Do review, policy, and adjudication artifacts all cite the same composition/effective/obligation lane?
3. Are remote admission and federated settlement artifacts locally reconstructible from constitutional citations?
4. Do all touched external artifacts have schema JSON, example JSON, and tests?
5. Does CI enforce no-local-recomposition and the final production closure gate?
6. Has the mirror been resynced from the active repo root?
