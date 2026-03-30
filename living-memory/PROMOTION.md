# PROMOTION.md
# Promotion: Emergent Candidate → Frozen BasisVersion

## Goal
Freeze only thoroughly-proven algebras into immutable `BasisVersion` records.
A promoted BasisVersion is the only thing `ForgeRuntime` accepts outside of lab mode.

---

## Graduation contract (all criteria must pass)

| Criterion                           | Default threshold    |
|-------------------------------------|----------------------|
| Suite pass rate (all tasks, hard checks) | >= 0.95         |
| Weighted improvement over baseline  | >= 0.05 (5%)         |
| Stability variance on repeat set    | <= 0.15              |
| Invariant violations on red-team suite | == 0              |
| Forbidden path edits                | == 0 (unless task permits) |
| CEA: causal drift vs. baseline      | < 0.25 (if CEA enabled) |

"Baseline" = the most recently promoted BasisVersion. For `v0001`, baseline is `ForgeConfig::default()`.

Red-team suite: see SECURITY.md. Must exist and run as part of promotion.
CEA criterion: if CEA is enabled and the graph has coverage, the candidate's causal fingerprint
must not drift more than `cea.causal_drift_warning_threshold` from the baseline version's fingerprint.

---

## Promotion procedure

```
ForgeLab::promote(candidate_id):
  candidate = store.get_candidate(candidate_id)
  
  1. Run graduation checks:
     for each criterion in contract:
       evaluate(candidate)
       if fails: return Err(PromotionFailed { criterion, value })
  
  2. Compute checksum:
     content = frozen_spec_json + bounds_json + invariants_json
     checksum = blake3(content)
  
  3. Compute CEA fingerprint snapshot (if CEA enabled):
     runs = store.get_eval_runs_for_candidate(candidate_id)
     attributed_runs = runs.filter(|r| r.cea_run_hash.is_some())
     dominant_edges = compute_dominant_edges(attributed_runs, top_n = 20)
     cea_fingerprint_json = serialize(dominant_edges)
  
  4. Generate golden MindState vectors:
     for each golden_input in config.promotion.golden_inputs:
       ms = ForgeRuntime::compile_mindstate(golden_input, candidate.spec)
       store golden snapshot: (input_hash, ms_hash, rendered_ms)
  
  5. Assign version_id: next in sequence (v0001, v0002, ...)
  
  6. Insert into promotions table:
     { version_id, candidate_id, frozen_spec_json, bounds_json,
       invariants_json, checksum, cea_fingerprint_json, promoted_at }
  
  7. Update candidate status → 'promoted'
  
  8. Return BasisVersion
```

## Current implementation status

The current Rust implementation now enforces deterministic admission checks before writing a
promotion row:

- candidate must have evaluation runs,
- average suite correctness must clear `lab.promotion_min_suite_pass_rate`,
- average weighted score must improve over the latest promoted baseline by at least
  `lab.promotion_min_weighted_improvement` when a baseline exists,
- stability variance across candidate eval runs must remain within
  `lab.archive.stability_variance_threshold`,
- stored invariant violations must remain zero,
- the candidate must have a stored evidence bundle with a deterministic assessment showing
  adequate reproducibility, strong isolation, clean contradiction state, and non-insufficient
  sample support,
- CEA drift is checked against the latest promoted fingerprint when both sides have coverage.

Still deferred:

- golden MindState vectors are not stored or checked yet,
- a distinct red-team promotion suite is not enforced yet.

---

## BasisVersion struct

```rust
pub struct BasisVersion {
    pub version_id:      String,          // "v0001"
    pub candidate_id:    Uuid,
    pub frozen_spec:     AlgebraSpec,
    pub bounds:          ParameterBounds, // locked min/max per parameter
    pub checksum:        String,          // blake3
    pub cea_fingerprint: Option<CausalFingerprint>,
    pub promoted_at:     DateTime<Utc>,
}
```

### Immutability guarantee
Once stored in `promotions`, a BasisVersion row is never updated.
- No UPDATE on `promotions`.
- To supersede: promote a new candidate as v0002, etc.
- Runtime prefers the highest version_id unless configured otherwise.

---

## Golden vectors
Golden vectors are snapshot-tested deterministic renders of MindState:

```rust
pub struct GoldenInput {
    pub request:        String,
    pub evidence_mock:  Vec<EvidenceItem>,  // fixed mock evidence for reproducibility
    pub repo_summary:   String,
}
```

Golden inputs are defined in `config.promotion.golden_inputs` (JSON array).
They must be stable across releases; changing them invalidates the golden set.

At runtime, after loading a BasisVersion:
- Recompute MindState for each golden input.
- Compare rendered hash to stored golden hash.
- If mismatch → `ForgeError::GoldenMindStateMismatch { version_id, input_hash }`.
  This is a regression and must block the version from being used in production.

---

## CEA drift monitoring (post-promotion)
After promotion, ForgeRuntime:
1. Loads the frozen `cea_fingerprint` for the active BasisVersion.
2. After each instrumented run, computes the run's causal fingerprint.
3. Computes fractional edge weight change vs. frozen fingerprint.
4. If change exceeds `cea.causal_drift_warning_threshold`:
   - Emit `ForgeEvent::CausalDriftWarning { version_id, drift_score }`.
   - Log prominently.
   - Does NOT fail the run.
5. If drift consistently exceeds threshold over N runs (configurable):
   - Emit `ForgeEvent::CausalDriftAlert` — signals that the codebase has changed enough
     that the promoted version's causal model may be stale; re-run Lab.

---

## Runtime rule
`ForgeRuntime` outside of lab mode:
- Only accepts `BasisVersion` (loaded from promotions table + checksum verified).
- Refuses `AlgebraSpec` directly.
- Lab mode: `config.lab.allow_raw_spec = true` enables `AlgebraSpec` for evaluation runs only.
