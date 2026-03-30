# forge-pilot — matrix-level target-state spec

**Status:** implementation target-state supplement  
**Relationship to repo:** supplements repo-wide v6/v7; does not weaken them  
**Primary goal:** land a closed-loop orchestration crate without reopening the authority map

---

## 0. Purpose

`forge-pilot` is a new crate that closes the loop between observation, targeting, bounded verification, canonical export/import, and re-observation.

It must sit **on top of** the current stack, not inside its authority planes.

The crate exists to do five things:

1. observe the latest non-authoritative kernel state and public projection state;
2. extract investigation targets with deterministic scored rules;
3. choose a bounded next action;
4. execute either a kernel-oracle check or a real Forge paired experiment;
5. export the result through the canonical V3 lane so the next observation sees the updated state.

---

## 1. Governing law

### 1.1 Consumer-only law

`forge-pilot` is a **consumer/orchestrator crate**.

It MUST NOT:

- define a new evidence bundle schema,
- define a new bridge format,
- persist authoritative truth directly,
- recreate the recursive kernel crates already in the repo,
- or silently join raw receipts into its normal targeting/ranking path.

### 1.2 Authority law

`forge-pilot` may read:

- runtime advisories,
- public projection queries,
- public projection import logs,
- rebuildable kernel payloads,
- Forge execution results,
- and explicit audit-mode evidence handles.

`forge-pilot` may write only:

- rebuildable loop-local history,
- structured reports,
- and canonical exports that re-enter the normal V3 import lane.

### 1.3 Degradation law

If the latest namespace has no usable `kernel_payload_json`, or the compiled graph degrades due to thin export / missing semantics, `forge-pilot` MUST degrade explicitly.

It MUST NOT hallucinate richer joint structure than the export actually provides.

### 1.4 LLM law

LLM use is optional and must be feature-gated.

LLMs MAY refine experiment text or suggest check hints.

LLMs MUST NOT:

- pick targets autonomously,
- override urgency scores,
- or mutate authoritative verification / promotion state.

---

## 2. Crate placement and dependency law

### 2.1 Placement

`forge-pilot` SHOULD be added as a **workspace member** once the default feature set builds without excluded ecosystem crates.

Default features MUST NOT require `LLM-Pipeline`.

### 2.2 Allowed direct dependencies

`forge-pilot` MAY depend directly on:

- `stack-ids`
- `semantic-memory`
- `semantic-memory-forge`
- `forge-memory-bridge`
- `knowledge-runtime`
- `forge-engine` (`living-memory/living-memory`)
- `recursive-kernel-core`
- `constraint-compiler`
- `kernel-execution`
- `kernel-oracles`
- `tokio`
- `serde`, `serde_json`
- `tracing`
- optional `LLM-Pipeline` behind a non-default feature

### 2.3 Forbidden dependencies and shortcuts

Forbidden:

- private DB-table access inside `semantic-memory`,
- direct write-through compatibility export paths,
- new “pilot-local” ID types duplicating `stack-ids`,
- direct bridge-time semantic invention,
- reading Forge raw store as the primary observation source.

---

## 3. Current-code reality the crate must respect

### 3.1 Observation surfaces already exist

The repo already exposes enough for a first real loop:

- runtime advisory summaries,
- public projection query APIs,
- projection import logs with `kernel_payload_json`,
- compiler/execution/oracle crates,
- paired patch execution and canonical V3 export.

### 3.2 The original design needs one correction

The original `forge-pilot` sketch assumed the act phase runs everything through `PairedExperimentRunner`.

That is too narrow for the current repo.

`forge-pilot` v1 MUST support two action families:

1. **Kernel-oracle actions** over the latest imported kernel payload.
2. **Paired patch actions** over a workspace/fixture with a concrete `StructuredPatch`.

Advisory-only “plan recording” MAY exist as an explicit fallback, but MUST NOT masquerade as a loop-closing experiment.

---

## 4. Module map

```text
forge-pilot/
├── Cargo.toml
├── src/
│   ├── lib.rs
│   ├── config.rs
│   ├── error.rs
│   ├── observe.rs
│   ├── targets.rs
│   ├── orient.rs
│   ├── decide.rs
│   ├── act.rs
│   ├── bundle_builder.rs
│   ├── export.rs
│   ├── history.rs
│   ├── loop_runner.rs
│   └── cli.rs
└── tests/
    ├── observation_fixture_tests.rs
    ├── scoring_tests.rs
    ├── oracle_plan_tests.rs
    ├── loop_roundtrip_tests.rs
    └── degradation_tests.rs
```

### 4.1 Module responsibilities

- `config.rs` — loop config, thresholds, feature flags, plan budgets
- `error.rs` — explicit error taxonomy
- `observe.rs` — reconstruct a rich observation from runtime + memory + kernel payload
- `targets.rs` — target taxonomy and stable keys
- `orient.rs` — rule-based target extraction, scoring, dedupe, exhaustion
- `decide.rs` — deterministic selection and optional LLM refinement
- `act.rs` — execute kernel-oracle or paired-patch plans
- `bundle_builder.rs` — synthesize a local `forge_engine::EvidenceBundle`
- `export.rs` — canonical export/import roundtrip only
- `history.rs` — in-memory retry and exhaustion tracking
- `loop_runner.rs` — the closed loop with budget/cooldown/halt/report
- `cli.rs` — optional binary-facing surfaces and JSON output

---

## 5. State model

### 5.1 Observation

`forge-pilot` MUST reconstruct a rich `Observation` object:

```rust
pub struct Observation {
    pub scope_key: ScopeKey,
    pub advisory: Option<InferenceAdvisory>,
    pub explanation: Option<InferenceExplanation>,
    pub risk_gate: Option<RiskGateDecision>,
    pub import_log: Option<ProjectionImportLogEntry>,
    pub batch: Option<ProjectionImportBatchV3>,
    pub compiled: Option<constraint_compiler::CompileOutput>,
    pub scheduled: Option<kernel_execution::ScheduledExecution>,
    pub oracle: Option<kernel_oracles::OracleAssessment>,
    pub causal_refutation: Option<kernel_oracles::RefutationResult>,
    pub minimal_perturbation: Option<kernel_oracles::RefutationResult>,
    pub claim_versions: Vec<ProjectionClaimVersion>,
    pub relation_versions: Vec<ProjectionRelationVersion>,
    pub episodes: Vec<ProjectionEpisode>,
    pub aliases: Vec<ProjectionEntityAlias>,
    pub evidence_refs: Vec<ProjectionEvidenceRef>,
    pub scope_health: ScopeHealthSummary,
}
```

### 5.2 Observation law

`observe.rs` MUST:

1. call runtime advisory surfaces;
2. query the latest namespace import log;
3. if `kernel_payload_json` exists, parse it into `ProjectionImportBatchV3`;
4. re-run `compile_batch()` and `schedule_execution()` using public crates;
5. compute oracles/refuters from the compiled graph;
6. query public projection rows for scope health;
7. degrade explicitly if kernel payloads are absent or malformed.

### 5.3 Scope health summary

```rust
pub struct ScopeHealthSummary {
    pub total_claim_versions: usize,
    pub total_relation_versions: usize,
    pub total_episodes: usize,
    pub fragile_node_count: usize,
    pub syndrome_count: usize,
    pub degradation_count: usize,
    pub unverified_claim_count: usize,
    pub supersession_candidate_count: usize,
    pub last_import_at: Option<String>,
}
```

---

## 6. Target taxonomy

### 6.1 Target kinds

`forge-pilot` SHOULD use the following v1 taxonomy:

```rust
pub enum TargetKind {
    FragileNode { node_id: String, belief_micros: u64 },
    ActiveSyndrome { signature: String },
    ThinExport { marker: String },
    UnverifiedClaimVersion { claim_version_id: ClaimVersionId },
    RefutationGap { target_node_id: String },
    SupersessionVerification {
        claim_version_id: ClaimVersionId,
        supersedes_claim_version_id: ClaimVersionId,
    },
    ComparabilityDrift { detail: String },
    CalibrationCaveat { marker: String },
    ScopeStale { last_import_at: Option<String> },
}
```

### 6.2 Stable target key law

Every target MUST have a deterministic stable key for retry/exhaustion tracking.

Examples:

- `fragile:<node_id>`
- `syndrome:<signature>`
- `thin_export:<marker>`
- `unverified:<claim_version_id>`
- `supersession:<claim_version_id>:<supersedes_claim_version_id>`

### 6.3 No target hallucination law

`forge-pilot` MUST only emit target kinds that are justified by current public inputs.

If the export is too thin to justify a richer target, it must fall back to `ThinExport`, `CalibrationCaveat`, or `ScopeStale` rather than inventing missing structure.

---

## 7. Action planning taxonomy

### 7.1 Plan kinds

```rust
pub enum PlanKind {
    OracleExactBounded { oracle_slice_id: OracleSliceId },
    OracleConservative,
    OracleDeltaParity { changed_node_ids: Vec<String>, max_iterations: u32 },
    OracleTemporalReplay { cutoff_recorded_at: String },
    OracleCausalRefuter { target_node_id: String, max_removed_nodes: usize },
    OracleMinimalPerturbation { target_node_id: String, max_removed_nodes: usize },
    PairedPatch {
        fixture_path: String,
        patch: StructuredPatch,
        experiment_config: ExperimentConfig,
    },
    AdvisoryOnlyVerificationPlan { plan: VerificationPlan },
}
```

### 7.2 Plan law

- Oracle plans are the default for targets derivable from the latest imported kernel payload.
- Paired patch plans are allowed only when a concrete patch + fixture/workspace are available.
- `AdvisoryOnlyVerificationPlan` is optional and MUST be visibly advisory-only.

---

## 8. Scoring law

### 8.1 Base urgency

| Target kind | Base urgency |
|---|---:|
| ActiveSyndrome | 0.95 |
| FragileNode | `1.0 - belief_micros / 1_000_000.0` |
| ThinExport | 0.85 |
| UnverifiedClaimVersion | 0.75 |
| RefutationGap | 0.72 |
| SupersessionVerification | 0.68 |
| ComparabilityDrift | 0.60 |
| CalibrationCaveat | 0.50 |
| ScopeStale | 0.25 |

### 8.2 Required modifiers

Apply these deterministic modifiers:

- `+0.05` if the latest risk gate is blocked
- `+0.05` if degradation is active for a target derived from thin export
- `+0.03` if the latest explanation carries residuals but no certificate
- `-retry_decay`, where `retry_decay = 0.5_f64.powi(prior_attempts as i32)`

Clamp to `[0.0, 1.0]`.

### 8.3 Tie-break rule

Sort by:

1. urgency descending,
2. target kind priority order,
3. stable target key ascending.

No randomness in v1.

---

## 9. Decision law

### 9.1 Deterministic candidate selection

`decide.rs` MUST:

- filter exhausted targets,
- sort deterministically,
- pick the highest-scoring candidate above `halt_urgency_threshold`,
- return `None` when nothing clears the threshold.

### 9.2 Optional LLM refinement

If enabled, LLM refinement may only modify:

- free-text verification description,
- suggested check hints,
- rationale text,
- or operator choice within an already-allowed plan family.

It MUST NOT:

- create a new target kind,
- increase urgency,
- lower verification requirements,
- or change the authority class of the result.

---

## 10. Act law

### 10.1 Kernel-oracle execution

For oracle plans, `act.rs` MUST use the existing kernel crates.

Examples:

- `OracleExactBounded` → `evaluate_exact_bounded()`
- `OracleConservative` → `evaluate_conservative()`
- `OracleDeltaParity` → `evaluate_delta_parity()`
- `OracleTemporalReplay` → `evaluate_temporal_replay()`
- `OracleCausalRefuter` → `evaluate_causal_refuter()`
- `OracleMinimalPerturbation` → `evaluate_minimal_perturbation()`

### 10.2 Paired patch execution

For patch plans, `act.rs` MUST:

- choose a backend and adapter,
- construct `PairedExperimentRunner`,
- run baseline + patched checks,
- compute typed diff,
- and hand the result to the local bundle builder.

### 10.3 Advisory-only execution

Advisory-only plans MAY be emitted only when:

- `allow_advisory_only_steps = true`, and
- no executable oracle or patch plan is available.

They MUST NOT count as a loop-closing verification action in the main success metrics.

---

## 11. Bundle-builder law

### 11.1 Why a local builder is required

The current repo exposes `forge_engine::EvidenceBundle`, but there is no thin “one function does it all” helper for turning oracle outputs or paired experiments into a pilot-ready bundle.

`forge-pilot` therefore MUST include a local `bundle_builder.rs`.

### 11.2 Minimum bundle contents

For every executed action, the builder MUST populate at minimum:

- `bundle_id`
- `candidate_id`
- `eval_id`
- `version_id`
- `trace_id`
- `attempt_id` when applicable
- `bundle_scope`
- `verification` or explicit refutation artifacts
- `claim_strength`
- `known_threats`
- `promotion_state = None`
- receipts / verification trials / refutation artifacts as applicable
- `supersedes_claim_version_id` when the target is a supersession verification

### 11.3 Authority class

All pilot-built bundles remain Forge raw truth until they are exported/imported through the canonical path.

---

## 12. Canonical export/import law

The only normal write path is:

```text
EvidenceBundle
  -> forge_engine::export_bundle()
  -> forge_memory_bridge::transform_envelope_v3()
  -> semantic_memory::MemoryStore::import_projection_batch()
```

Forbidden:

- direct memory write-through compatibility paths,
- direct DB writes,
- private importer bypasses,
- or “pilot-local” projection persistence.

---

## 13. History law

### 13.1 v1 storage class

`forge-pilot` v1 history is **non-authoritative and in-memory**.

It tracks:

- stable target key,
- prior attempt bundle IDs,
- prior outcomes,
- last selected timestamp,
- retry count,
- exhaustion state.

### 13.2 Exhaustion rules

A target is exhausted when any of the following hold:

- retry count exceeds `max_retries_per_target`,
- the same outcome repeats with no material residual/certificate change,
- or a higher-confidence superseding target for the same stable key family has landed.

---

## 14. Loop runner law

### 14.1 Core config

```rust
pub struct LoopConfig {
    pub max_iterations: u32,
    pub time_budget_secs: u64,
    pub cooldown_secs: u64,
    pub halt_urgency_threshold: f64,
    pub max_retries_per_target: u32,
    pub allow_advisory_only_steps: bool,
    pub scope: ScopeKey,
    pub workspace_path: String,
    pub include_hyperedges: bool,
    pub oracle_max_iterations: u32,
    pub minimal_perturbation_budget: usize,
}
```

### 14.2 Halt reasons

```rust
pub enum HaltReason {
    MaxIterationsReached,
    TimeBudgetExhausted,
    NothingWorthInvestigating,
    AllTargetsExhausted,
    MissingKernelPayload,
    ThinExportBlockedExecution,
    ExternalHalt,
}
```

### 14.3 Loop report

The final report MUST capture:

- iterations completed,
- actions executed,
- exports/imports completed,
- halt reason,
- targets investigated,
- degraded iterations,
- elapsed seconds,
- and whether any step was advisory-only.

---

## 15. CLI law

`forge-pilot` SHOULD expose at least:

- `forge-pilot observe`
- `forge-pilot run`
- `forge-pilot plan`
- `forge-pilot explain`

### 15.1 Output law

CLI output MUST support:

- human-readable text, and
- `--json` for structured reports.

---

## 16. Conformance gates

`forge-pilot` is not done until all of these are true:

1. It never writes authoritative truth directly.
2. It reconstructs observation deterministically from public surfaces.
3. It degrades explicitly on missing kernel payloads or thin export.
4. Oracle plans execute through existing kernel crates, not pilot-local clones.
5. Patch plans execute through `PairedExperimentRunner`.
6. Export/import roundtrips use only the canonical V3 path.
7. Loop budgets, cooldown, and halt behavior are tested.
8. Optional LLM refinement remains advisory-only.

---

## 17. Forbidden shortcuts

Forbidden:

1. adding a second evidence/export schema inside `forge-pilot`;
2. re-implementing the recursive kernel crates locally;
3. bypassing `transform_envelope_v3()`;
4. using direct memory write-through compat paths as the happy path;
5. inventing hyperedges or joint evidence groups that the export did not provide;
6. reading private runtime internals when public projection import logs already exist;
7. silently running advisory-only plans and counting them as experimental closure;
8. letting LLM output select targets or promote truth;
9. running the loop without stop conditions and retry limits.

---

## 18. Phase-1 exit criteria

Phase 1 is complete only when:

- the crate builds with default features;
- observation reconstruction is real;
- target extraction is deterministic;
- at least one oracle plan and one paired patch plan can execute;
- canonical export/import roundtrip tests pass;
- degradation tests prove missing kernel payloads do not hallucinate structure;
- and the loop can run for a bounded fixture namespace and halt honestly.
