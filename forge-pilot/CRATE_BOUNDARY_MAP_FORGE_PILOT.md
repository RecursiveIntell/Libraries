# Crate boundary map for forge-pilot

## 1. Boundary summary

`forge-pilot` is a **consumer/orchestrator** over crates that already exist.

It should wire existing crates together, not reopen their responsibilities.

## 2. Allowed crate roles

| Crate | Role inside forge-pilot | Allowed use | Forbidden use |
|---|---|---|---|
| `stack-ids` | shared IDs, `ScopeKey`, `TraceCtx`, canonical identity | reuse only | crate-local copies of IDs or trace wrappers |
| `semantic-memory-forge` | canonical raw truth schema | type reuse only when building/exporting bundles | new schema drift inside pilot |
| `forge-memory-bridge` | deterministic V3 transform | `transform_envelope_v3()` only | semantic invention, live query service |
| `semantic-memory` | public projection reads + canonical import | query surfaces and `import_projection_batch()` | direct table writes or private DB reach-in |
| `knowledge-runtime` | query orchestration + advisory summaries | runtime advisories and optional query helpers | persistent truth, private-context reach-in |
| `forge-engine` | paired experiments, bundle/export helpers | `PairedExperimentRunner`, `ExperimentConfig`, `StructuredPatch`, `EvidenceBundle`, `export_bundle` | direct memory write-through happy path |
| `recursive-kernel-core` | kernel artifact contracts | type reuse only | parallel pilot-local kernel contracts |
| `constraint-compiler` | deterministic compilation | `compile_batch()` | alternate graph compiler in pilot |
| `kernel-execution` | schedule / message passing / delta / residual execution | reuse only | reimplementation |
| `kernel-oracles` | exact/conservative/refutation/oracle helpers | reuse only | pilot-local shadow oracles |
| `kernel-conformance` | fixture patterns and parity thinking | borrow patterns/tests if helpful | treating it as runtime dependency |
| `LLM-Pipeline` | optional refinement only | feature-gated adapter | target selection, authority changes |

## 3. Required pilot-owned code

`forge-pilot` should own only the pieces that are genuinely missing today:

- observation reconstruction logic that stitches runtime + memory + kernel payloads together,
- deterministic target extraction/scoring,
- action planning,
- local bundle-building glue,
- loop runner / history / reports,
- CLI surfaces.

## 4. Acceptable narrow upstream changes

These upstream changes are acceptable **only if proven necessary**:

1. add a tiny public helper in `knowledge-runtime` if observation reconstruction becomes too copy-paste heavy;
2. add a tiny helper in `forge-engine` if bundle building is impossible without duplicating unsafe internals;
3. add a root `Makefile` target for `forge-pilot` tests.

Any upstream change broader than that is a design smell and must be justified.

## 5. Explicitly frozen responsibilities

Do **not** move any of these into `forge-pilot`:

- canonical export schema ownership,
- bridge transformation law,
- projected truth storage,
- runtime-wide query classification/merge logic,
- recursive-kernel compile/execute/oracle primitives,
- queue/retry ownership for external control-plane crates.
