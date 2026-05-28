# Acceptance Gates and CI

## Universal gate for every pass

```bash
cargo fmt --all --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets --all-features -- -D warnings
bash scripts/assert_no_fake_completion.sh .
bash scripts/assert_no_scaffold_promoted.sh .
bash scripts/check_dependency_boundaries.sh
bash scripts/check_examples.sh
bash scripts/next_smoke.sh
bash scripts/verify.sh
```

## Required `scripts/verify.sh`

The repository must contain a single verification script that runs the universal gate. Codex must update CI to call it.

## Release gate additions after specific passes

| Pass | Additional gate |
|---|---|
| P02 | provider certification fixtures for all provider kinds |
| P03 | turn executor tool-loop fixture |
| P04 | permit/approval denial and grant fixtures |
| P05 | durable receipt restart/inspect/export tests |
| P06 | duplicate-key/schema/canonical-digest/fuzz tests |
| P07 | schema generation + compatibility checks |
| P08 | reference interpreter differential tests |
| P09 | bitemporal as-of query and supersession tests |
| P10 | sandbox/path traversal/patch apply/run-checks tests |
| P11 | duplicate schedule, lease crash, safe-mode tests |
| P12 | canonical verification-control, contradiction-witness, and repair-adapter tests |
| P13 | view disclosure/widening tests |
| P14 | release readiness, operator status, install smoke, and example compile/test fixtures |
| P15 | right-graph, convergence, oracle-slice, and syndrome/canonical-repair-backpointer tests |
| P16 | support-frontier, compaction receipt, and as-of preservation tests |
| P17 | attestation/admission/revocation tests |
| P18 | mechanism/refuter/publication tests |
| P19 | full traceability + release readiness audit |
