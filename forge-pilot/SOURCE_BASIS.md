# Source basis for the forge-pilot pack

## 1. Repo basis

This pack is grounded in the **current code archive**, not just in older research prose.

### 1.1 Current workspace reality

The workspace already includes the crates `stack-ids`, `llm-tool-runtime`, `recursive-kernel-core`, `constraint-compiler`, `kernel-execution`, `kernel-oracles`, `kernel-conformance`, `semantic-memory-forge`, `forge-memory-bridge`, `semantic-memory`, `knowledge-runtime`, and `living-memory/living-memory`.

### 1.2 Current public surfaces that matter

`forge-pilot` can be built today because the current repo already exposes these useful surfaces:

- `knowledge-runtime::KnowledgeRuntime::{latest_inference_advisory, latest_inference_explanation, latest_risk_gate}`
- `semantic-memory::MemoryStore::{query_claim_versions, query_relation_versions, query_episodes, query_entity_aliases, query_evidence_refs, query_projection_imports}`
- `forge-engine::export_bundle`
- `forge-memory-bridge::transform_envelope_v3`
- `semantic-memory::MemoryStore::import_projection_batch`
- `constraint_compiler::compile_batch`
- `kernel_execution::{schedule_execution, execute_message_passing_baseline, execute_delta_propagation, execute_residual_correction}`
- `kernel_oracles::{evaluate_exact_bounded, evaluate_conservative, evaluate_delta_parity, evaluate_temporal_replay, evaluate_causal_refuter, evaluate_minimal_perturbation}`

### 1.3 Current-code correction to the original forge-pilot design

The original sketch is directionally right, but the **current codebase changes the best implementation shape** in two important ways:

1. `forge-pilot` should **not** re-implement the recursive kernel; it should consume the existing compile / execute / oracle crates.
2. The act phase should support **two families of executable work**:
   - **kernel-oracle work** over the latest imported kernel payload,
   - **paired patch experiments** through `forge-engine::PairedExperimentRunner` when a concrete patch + fixture/workspace are available.

That is a better fit than forcing every target through the patch runner.

## 2. Architecture basis

This pack inherits the repo-wide law from v6 and v7:

- Forge owns raw verification truth.
- `forge-memory-bridge` transforms but does not invent meaning.
- `semantic-memory` owns durable projected truth.
- `knowledge-runtime` owns bounded routing / merge / degradation.
- Recursive kernel artifacts remain **non-authoritative** and must not self-promote.

## 3. Research basis

The pack also inherits the loaded research direction:

- evidence before inference,
- bitemporal truth and replayable provenance,
- message-passing / residual-correction / delta-propagation as the recursive execution law,
- certificates / witnesses over naked scores,
- explicit degradation on thin export,
- causal refutation as part of the normal path, not a decorative appendix.

## 4. Practical conclusion

The missing piece is not another big theory document.

The missing piece is a **closed-loop consumer crate** that:

- reads current advisories and imported kernel payloads,
- extracts actionable investigation targets,
- executes bounded verification oracles and real patch experiments,
- re-enters the canonical V3 lane,
- and reports honestly when the loop cannot close.
