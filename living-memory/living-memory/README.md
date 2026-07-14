# forge-engine

Local execution and evidence engine for structured code-patch experiments.

## Responsibilities

`forge-engine` validates and applies `StructuredPatch` values, executes Cargo
checks in fresh workspaces, compares matched baseline/patched arms, persists
operational evidence, and coordinates causal-edit attribution (CEA). It does
not own semantic-memory truth or promote local results into general claims.

## CEA execution model

The public `CausalAttributionEngine` supports:

- advisory prediction over an **observational** edit/effect association graph;
- fresh matched baseline/patched pairs, including repeated pairs;
- a differential check view that excludes baseline-stable failures;
- bounded singleton edit ablations on fresh workspaces;
- deterministic, integrity-bound update and ablation receipts; and
- an explicit prediction gate that defaults to `RunChecks`.

Evidence grades are intentionally separate:

1. `Observational` — proximity/co-occurrence used for localization and advisory prediction;
2. `PairedInterventional` — a local patch-level effect under a captured workload;
3. `Ablation` / `Counterfactual` — edit-removal or replacement evidence;
4. `SyntheticTelemetry` — forbidden from the code-association graph.

A receipt proves what ran and binds its inputs/results. It does not prove
causality outside the captured workload. Individual edit responsibility is not
promoted from proximity alone.

## Persistence and identity

The canonical operational database is `forge.db`. Observational graph updates
are transactional. Identified runs carry both:

- a content-bound `run_hash` for integrity; and
- an identity-only `observation_key` for replay idempotency.

Interventional evidence remains receipt-bearing and does not enter the
observational edge store. Raw source is excluded from CEA node signatures and
receipts.

## Prediction safety

Association-only graph data cannot authorize check skipping. Fuzzy matching is
off by default and remains advisory when explicitly enabled. Unknown signatures
blend toward a neutral prior. The current runtime gate remains fail-closed even
when `enable_zero_shot` is set.

## Local evidence

`examples/cea_replay_eval.rs` exercises deterministic prediction cases and a
tiny two-operation Cargo ablation fixture. The latest local report is
`../docs/benchmarks/CEA_ENGINE_LOCAL_2026-07-13.md`. It records zero risk recall
on the small labeled fixture set, so prediction remains advisory.

## Verification

```bash
cargo test -p cea-core -p cea-store -p cea-sqlite --all-targets
cargo test -p forge-engine --all-targets
cargo run -p forge-engine --example cea_replay_eval --   --output target/cea-eval/receipt.json
```

## Authority boundary

- `semantic-memory-forge` owns raw verification/export wire truth.
- `forge-memory-bridge` owns transformation only.
- `semantic-memory` owns durable projected/queryable truth.
- `forge-engine` owns operational verification work on top of those crates.
- `forge-pilot` consumes receipts and must not invent scores, comparability, or support.
