# Codex implementation prompt — forge-pilot

Implement a new crate named `forge-pilot` inside the current repo.

## Objective

Build a **closed-loop orchestrator crate** that consumes the current runtime/kernel/evidence surfaces and closes the loop:

1. observe the latest advisory + projection state,
2. extract deterministic investigation targets,
3. pick the best target under bounded rules,
4. execute either a kernel-oracle plan or a paired patch experiment,
5. export/import the result through the canonical V3 lane,
6. repeat until the loop halts honestly.

## Hard constraints

- Do **not** reopen v6/v7 architecture.
- Do **not** create a new evidence schema.
- Do **not** bypass `export_bundle -> transform_envelope_v3 -> import_projection_batch`.
- Do **not** reimplement `compile_batch`, `schedule_execution`, or oracle logic.
- Do **not** let LLM output choose targets or promote truth.
- Default features must build **without** `LLM-Pipeline`.

## Current-code surfaces you must reuse

- `knowledge-runtime::KnowledgeRuntime::{latest_inference_advisory, latest_inference_explanation, latest_risk_gate}`
- `semantic-memory::MemoryStore::{query_claim_versions, query_relation_versions, query_episodes, query_entity_aliases, query_evidence_refs, query_projection_imports}`
- `constraint_compiler::compile_batch`
- `kernel_execution::{schedule_execution, execute_message_passing_baseline, execute_delta_propagation, execute_residual_correction}`
- `kernel_oracles::{evaluate_exact_bounded, evaluate_conservative, evaluate_delta_parity, evaluate_temporal_replay, evaluate_causal_refuter, evaluate_minimal_perturbation}`
- `forge_engine::{PairedExperimentRunner, ExperimentConfig, StructuredPatch, EvidenceBundle, export_bundle}`
- `forge_memory_bridge::transform_envelope_v3`
- `semantic_memory::MemoryStore::import_projection_batch`

## Implementation order

### Step 1 — crate surface
- add `forge-pilot/Cargo.toml`
- add `forge-pilot/src/{lib,config,error,observe,targets,orient,decide,act,bundle_builder,export,history,loop_runner,cli}.rs`
- add workspace membership in root `Cargo.toml`
- add a `Makefile` target if useful

### Step 2 — observation reconstruction
- build `Observation` using runtime advisories + public projection imports + parsed `kernel_payload_json`
- recompile with `compile_batch()` and reschedule with `schedule_execution()`
- compute oracle/refutation summaries using `kernel-oracles`
- compute `ScopeHealthSummary` using public projection query APIs
- degrade explicitly if kernel payloads are missing or invalid

### Step 3 — targeting and decision
- implement the target taxonomy from the pack spec
- implement stable target keys
- implement deterministic urgency scoring + retry decay
- implement candidate selection + halt threshold
- add feature-gated optional LLM refinement that only edits descriptive text / hints

### Step 4 — act paths
- implement oracle-plan execution
- implement paired patch execution using `PairedExperimentRunner`
- add `bundle_builder.rs` to build `forge_engine::EvidenceBundle` from oracle or experiment outputs
- do not fabricate authoritative promotion state

### Step 5 — canonical roundtrip
- export via `forge_engine::export_bundle`
- transform via `transform_envelope_v3`
- import via `MemoryStore::import_projection_batch`
- update history and loop reports

### Step 6 — loop runner
- implement max-iteration, time-budget, cooldown, halt-threshold, and retry-limit controls
- implement clear halt reasons
- expose JSON-friendly reports

### Step 7 — tests
- observation fixtures
- scoring tests
- oracle-plan tests
- end-to-end loop roundtrip tests
- degradation honesty tests

## Quality bar

You are not done when the code compiles.

You are done when:

- the crate is a thin orchestrator over the current stack,
- both action families are real,
- the canonical roundtrip is real,
- degradation is explicit,
- and the tests prove it.

## If blocked

If you discover a missing helper, prefer these in order:

1. a local helper inside `forge-pilot`,
2. a tiny public helper in `knowledge-runtime` or `forge-engine`,
3. only then a broader upstream change.

If a broader upstream change seems necessary, explain exactly why. Do not silently reopen architecture.
