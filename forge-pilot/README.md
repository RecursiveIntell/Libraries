# forge-pilot

Closed-loop orchestrator over Forge execution, canonical export/import, runtime
advisories, and kernel oracles.

## Scope

`forge-pilot` scores targets, builds bounded plans, executes them, and records
canonical roundtrips without taking ownership of truth. It does not compensate
for unresolved kernel uncertainty or promote local output into supported truth
on its own.

## CEA integration

For `PairedPatch` plans, the pilot uses the already-open canonical `ForgeStore`
and the real `CausalAttributionEngine`. It exports:

- matched-pair check outcomes and comparability;
- integrity-bound observational update receipts;
- bounded singleton-ablation receipts and refutation artifacts;
- advisory prediction summaries; and
- explicit degradation warnings.

Patch-level improvements/regressions remain local outcome evidence. Causal
hypothesis support and contradiction counts come only from comparable ablation
receipts. Missing update receipts make verification trials incomplete. Unmeasured
novelty or single-pair stability use compatibility zeros that are explicitly
excluded from weighted scoring and disclosed in bundle warnings.

## Running and verification

```bash
cargo run -p forge-pilot --
cargo test -p forge-pilot --test cea_bundle_tests
cargo test -p forge-pilot --test loop_roundtrip_tests
```

## Ecosystem

The pilot depends on `forge-engine`, `semantic-memory`,
`semantic-memory-forge`, `forge-memory-bridge`, `knowledge-runtime`, kernel
execution/oracle crates, and the verification policy/control stack.
