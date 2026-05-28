# P31 Acceptance Gates

## Required local commands

```bash
cargo fmt --all -- --check
cargo check --workspace --all-targets
cargo test --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
python3 scripts/validate_strict_schemas.py
python3 scripts/assert_existing_crate_boundaries.py
python3 scripts/assert_no_stale_surfaces.py
bash scripts/assert_no_opaque_signal_scanning.sh
bash scripts/assert_no_feut_contamination.sh
bash scripts/assert_no_llm_or_network_calls.sh
bash scripts/assert_no_durable_float_scores.sh
bash scripts/assert_no_naked_decision_booleans.sh
bash scripts/assert_no_shadow_truth.sh
bash scripts/assert_no_unexplained_golden_changes.sh
bash scripts/run_p31_completion_checks.sh
```

## Required package commands if ZIP is generated

```bash
python3 scripts/verify_archive_manifest_parity.py <zip> <manifest.json>
python3 scripts/assert_required_archive_paths.py <zip>
rm -rf /tmp/scr-runtime-fresh
mkdir -p /tmp/scr-runtime-fresh
unzip -q <zip> -d /tmp/scr-runtime-fresh
cd /tmp/scr-runtime-fresh
bash scripts/run_p31_completion_checks.sh
```

## Required semantic gates

- No active legacy template overlays duplicated from prior passes.
- No active manual injection workflow.
- No SCR/non-SCR labels in active surfaces.
- No source-basis doc claiming no Rust workspace if Rust workspace exists.
- No opaque-ref scanning for SCR control signals.
- No unknown hard rules accepted by policy validation.
- No wrong policy domain/algorithm accepted.
- No wire-visible unknown fields accepted unless documented exception.
- No schema weaker than Rust score bounds.
- No local duplicate of owner-crate canonical types without adapter/ambiguity record.

## Completion definition

P31 is complete only when the final report contains command receipts proving all required gates passed or explicit blockers explaining why they could not run.
