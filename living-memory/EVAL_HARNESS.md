# EVAL_HARNESS.md
# Evaluation Harness (Rust-first)

## Fixture layout
```
fixtures/
  <suite_name>/
    <task_id>/
      repo/          ← Cargo project snapshot (Cargo.toml at root)
      task.json
```

### task.json schema
```json
{
  "task_id": "unique-kebab-id",
  "prompt": "Refactor the error handling in src/store.rs to use thiserror",
  "constraints": {
    "allow_test_modifications": false,
    "max_files_changed": null
  },
  "weights": {
    "correctness": 0.7,
    "novelty": 0.2,
    "stability": 0.1
  },
  "expected": {
    "require_fmt": true,
    "require_clippy": true,
    "require_tests": true
  },
  "cea": {
    "instrument": true,
    "risk_threshold_override": null
  }
}
```

`max_files_changed: null` → use global config default.
`cea.instrument: true` → run CEA instrumentation for this task (default: true if CEA enabled globally).

---

## Evaluation procedure (single run)

```
EvaluationRunner::run(candidate, task, config, store, backends):
  1. Load task.json and fixture repo
  2. Validate fixture: Cargo.toml exists, repo compiles on baseline
  3. Select execution backend (auto or configured)
  4. Prepare workspace: copy fixture/repo to temp dir
  5. Compile MindState(candidate.basis_version, task.prompt, evidence, traces)
  6. Invoke generator → StructuredPatch (in Lab: mocked or model-provided)
  7. Validate patch:
     - forbidden path check (using task constraints merged with global config)
     - caps check
     If validation fails → record EvalRun with score=zero, violations=all; return early
  8. Apply patch → (PatchedWorkspace, LineAttributionMap)
     If apply fails → record EvalRun with score=zero; return early
  9. If task.cea.instrument:
     a. Wrap check run with CausalInstrument
     b. Run checks → (CheckResult, AttributedRunResult)
     c. Call CausalAttributionEngine::update_graph(attributed_result)
  10. Else:
     Run checks → CheckResult
  11. Render diff
  12. Score (see §Scoring)
  13. Persist EvalRun (scores, violations, diff_hash, patch_hash, mindstate_hash, logs_ref, cea_run_hash)
  14. Return EvalRunResult
```

---

## Scoring

### Correctness score (primary)
```
correctness = 0.0

weights = task.weights.correctness_breakdown or defaults:
  fmt:   0.10
  clippy: 0.30
  test:   0.60

if fmt_pass:    correctness += 0.10 * task.weights.correctness
if clippy_pass: correctness += 0.30 * task.weights.correctness
if test_pass:   correctness += 0.60 * task.weights.correctness
```

### Novelty score
```
novelty = 0.0

# Primary: strategy tag novelty
tags = extract_strategy_tags(patch)
recent_tags = query last N answer_traces for same question_sig (N = config.novelty.min_traces_for_orthogonality)

if recent_tags is empty:
  tag_novelty = 1.0    # first answer; maximally novel
else:
  overlap = |intersection(tags, recent_tags_union)| / |union(tags, recent_tags_union)|
  tag_novelty = 1.0 - overlap

# Secondary: topology novelty (optional)
if cea enabled and graph has coverage:
  cea_pred = CausalAttributionEngine::predict(patch)
  # Use causal fingerprint distance from archive cell fingerprints
  topology_novelty = causal_fingerprint_distance(cea_pred, archive_cell_fingerprints)
  novelty = (tag_novelty * 0.7 + topology_novelty * 0.3) * task.weights.novelty
else:
  novelty = tag_novelty * task.weights.novelty
```

### Stability score
Only computed when `suite_config.repeat_runs > 1` (default: 1 run per task).
```
stability = 0.0

if this is a multi-run task:
  strategy_variance = variance of strategy tag jaccard distances across runs
  diff_topology_variance = variance of (files_changed, total_lines) across runs
  stability = (1.0 - mean(strategy_variance, diff_topology_variance)) * task.weights.stability
```

### Anti-fluff rule
Novelty score collapses to 0.0 if ALL of:
- `tags == last_run_tags` (identical strategy tags)
- `patch.edits.len() == last_run.edits.len()` (same number of files)
- All files in patch are same as last run

This prevents trivially rewording a patch to claim novelty credit.

### ScoreVector
```rust
pub struct ScoreVector {
    pub correctness: f64,
    pub novelty:     f64,
    pub stability:   f64,
    pub weighted_total: f64,   // correctness + novelty + stability (weights applied)
    pub cea_confidence: Option<f64>,  // None if CEA not enabled
    pub cea_predicted_correctness: Option<f64>,
}
```

---

## GenerationReport (ForgeLab output)
```rust
pub struct GenerationReport {
    pub batch_id:    Uuid,
    pub task_results: Vec<EvalRunResult>,
    pub archive_updates: Vec<ArchiveCellUpdate>,
    pub cea_graph_delta: Option<CeaGraphDelta>,  // edges added/updated this batch
    pub elapsed_ms:  u64,
}
```
