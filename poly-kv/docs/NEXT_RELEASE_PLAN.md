# Next Release Plan

## Current alpha API notes

- Existing `KvTensorShape` remains available for alpha examples.
- `KvCacheShapeV2` is available in `quant-codec-core` for batch/query-head/KV-head/attention-kind validation.
- `PoolBuilder::build_from_exact_blocks` is additive and avoids requiring callers to pass duplicate exact fallback input.
- `PoolBuilder::build_from_blocks` plus explicit `exact_fallback` remains available.

## Planned follow-up

- Decide whether `poly-kv` manifests should move from legacy `KvTensorShape` to `KvCacheShapeV2`.
- Add native Python wheel validation once `maturin` is available in the environment.
- Add buffer-protocol CPU fixture validation before claiming broader Python array support.
- Keep TurboQuant and FibQuant adapters unsupported until their APIs are inspected and compiled.

## Non-goals for this alpha

- adaptive controller;
- daemon sidecar;
- serving runtime adapters;
- app truth-store integration;
- real-model benchmark claims.
