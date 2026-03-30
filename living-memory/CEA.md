# CEA.md
# Causal Edit Attribution — Specification v1.0
# [PROPRIETARY — Do not publish]

## Overview

**Causal Edit Attribution (CEA)** is a subsystem that treats code-generation evaluation as a
*causal inference problem* rather than a black-box scoring problem.

Traditional eval: `patch → run checks → score`
CEA eval:         `patch → instrument run → (cause, effect) pairs → causal graph update → score + attribution`

Over many runs, the causal graph becomes a codebase-specific predictive model. When the graph
has sufficient coverage, Forge can produce `CausalPrediction` — a predicted score with
confidence bounds — from patch topology alone, before running any checks.

This is the proprietary core of forge-engine. The graph lives in `forge.db` and is
specific to the codebase it was trained on. It is not transferable without the training runs.

---

## 1. Core Concepts

### 1.1 EditOpSignature
A structural fingerprint of a single EditOp that is stable across runs and codebases:
- `op_kind`: Insert | Replace | Delete
- `anchor_kind`: AfterLine | BeforeLine | AfterMatch | BeforeMatch
- `lines_added`: u32
- `lines_removed`: u32
- `context_hash`: blake3 hash of trimmed context_before + context_after lines (detects
  structural similarity without storing raw source)
- `file_extension`: `.rs` (always in v1)
- `scope_tag`: inferred scope of the edit — fn | impl | mod | trait | macro | top_level
  (derived from context lines by looking for `fn `, `impl `, `mod `, `trait `, `macro_rules!`)
- `op_index`: position of this op within its FileEdit (first, middle, last)
- `file_index`: position of the FileEdit within the patch (first, middle, last)

### 1.2 EffectSignature
A structural fingerprint of an observable check outcome:
- `check_kind`: fmt | clippy | test
- `outcome`: pass | fail
- `severity`: pass | warning | error | test_fail
- `message_class`: for clippy — the lint name (e.g., `clippy::needless_return`);
  for test — the test function name; for fmt — file path segment
- `line_offset_from_edit`: signed integer — how many lines from the closest edit op
  did this effect appear? (requires mapping check output lines back to source positions)

### 1.3 CausalEdge
A directed edge in the causal graph: `EditOpSignature → EffectSignature`
- `weight`: f64 — accumulated causal score (see §2.3)
- `count`: u64 — number of times this (cause, effect) pair was observed
- `last_seen`: timestamp
- `version_id`: which BasisVersion was active when this edge was observed
- `confidence`: f64 — `count / (count + prior_weight)` — Bayesian-style confidence

### 1.4 CausalGraph
A directed graph stored in `forge.db` (serialized via petgraph + JSON, or as edge rows).
Nodes are `EditOpSignature` and `EffectSignature` variants.
Edges are `CausalEdge`.

### 1.5 CausalPrediction
Output of `predict(patch)`:
- `predicted_correctness`: f64
- `predicted_novelty`: f64 (topology-derived)
- `confidence`: f64 — overall confidence based on graph coverage for this patch's signatures
- `coverage_fraction`: f64 — fraction of edit op signatures in this patch that have graph edges
- `risk_flags`: Vec<RiskFlag> — high-risk (cause, effect) pairs detected with confidence > threshold
- `zero_shot_eligible`: bool — true if coverage_fraction >= configured threshold (default 0.80)

### 1.6 RiskFlag
- `op_signature`: EditOpSignature — the edit op that is risky
- `predicted_effect`: EffectSignature — the likely bad effect
- `confidence`: f64

---

## 2. Instrumentation Protocol

### 2.1 Instrumented run flow
```
instrument_run(patch, backend):
  1. snapshot repo state (hash every source file)
  2. apply patch → record which EditOp touched which line ranges in which files
  3. run checks with output capture (stdout/stderr per check)
  4. parse check outputs → extract EffectSignatures with source positions
  5. for each EffectSignature with a source position:
       find closest EditOp by line distance → form (EditOpSignature, EffectSignature, distance)
  6. filter: only attribute effects within MAX_LINE_DISTANCE (default: 50 lines)
     for effects with no nearby edit, attribute to nearest by file
  7. collect all (EditOpSig, EffectSig, distance) triples → AttributedRunResult
  8. return AttributedRunResult (does NOT update graph — caller decides)
```

### 2.2 Output parsing (Rust/Cargo)
**fmt:**
- stdout: list of files with formatting diffs; extract file + line numbers
- Effect: `{ check_kind: fmt, outcome: fail, message_class: <file_segment> }`

**clippy:**
- stderr: structured JSON with `--message-format=json`
- Extract: `{ lint_name, file, line, col }`
- Effect: `{ check_kind: clippy, severity: warning|error, message_class: <lint_name>, line }`

**test:**
- stdout: parsed test output (`---- test_name FAILED ----`)
- Map test name → source file + line via `cargo test -- --format=json` if available
- Effect: `{ check_kind: test, outcome: fail, message_class: <test_fn_name> }`

### 2.3 Edge weight update formula
When a (cause, effect) pair is observed:
```
new_weight = old_weight + attribution_score(distance, outcome_severity)

attribution_score(distance, severity):
  base = match severity {
    error    => 1.0,
    warning  => 0.5,
    test_fail => 0.8,
    pass      => 0.1,  // positive attribution — edit didn't break this
  }
  decay = 1.0 / (1.0 + distance / 10.0)  // closer edits get more credit
  base * decay
```

For **pass** effects (the check passed for this edit's file/scope), create positive
attribution edges. These are equally important: they represent what the graph *knows*
is safe.

### 2.4 Idempotency
Each `AttributedRunResult` has a content hash. `update_graph` is a no-op if the hash
already exists in `cea_run_log`. This ensures re-runs don't double-count.

---

## 3. Prediction Algorithm

### 3.1 Signature matching
For a new patch:
1. Compute `EditOpSignature` for every op in the patch.
2. For each signature, query the graph for all outgoing edges.
3. If no edges: this signature is `unknown` — reduces coverage_fraction.

### 3.2 Risk aggregation
For each (EditOpSig, EffectSig, confidence) triple where `EffectSig.outcome == fail`:
- If `confidence >= risk_threshold` (default 0.6) → add to `risk_flags`.

### 3.3 Correctness prediction
```
predicted_correctness =
  sum over known signatures:
    (positive_weight - negative_weight) / (positive_weight + negative_weight + epsilon)
  normalized to [0, 1]

where:
  positive_weight = sum of weights of pass-outcome edges for this sig
  negative_weight = sum of weights of fail-outcome edges for this sig
```

Unknown signatures contribute 0.5 (neutral prior) weighted by `(1 - coverage_fraction)`.

### 3.4 Zero-shot eligibility
`zero_shot_eligible = coverage_fraction >= cea.zero_shot_coverage_threshold`
Default threshold: 0.80.

When eligible, the runtime can skip checks and use `predicted_correctness` as the score.
This must be **explicitly enabled** in config: `cea.enable_zero_shot = false` by default.

---

## 4. Graph Storage (forge.db)

### Table: cea_nodes
```sql
CREATE TABLE cea_nodes (
  node_id   TEXT PRIMARY KEY,
  node_kind TEXT NOT NULL,  -- 'cause' | 'effect'
  sig_json  TEXT NOT NULL,  -- serialized EditOpSignature or EffectSignature
  first_seen TEXT NOT NULL,
  last_seen  TEXT NOT NULL
);
```

### Table: cea_edges
```sql
CREATE TABLE cea_edges (
  edge_id       TEXT PRIMARY KEY,
  cause_node_id TEXT NOT NULL REFERENCES cea_nodes(node_id),
  effect_node_id TEXT NOT NULL REFERENCES cea_nodes(node_id),
  weight        REAL NOT NULL DEFAULT 0.0,
  count         INTEGER NOT NULL DEFAULT 0,
  confidence    REAL NOT NULL DEFAULT 0.0,
  version_id    TEXT NOT NULL,
  last_seen     TEXT NOT NULL,
  UNIQUE(cause_node_id, effect_node_id, version_id)
);
```

### Table: cea_run_log
```sql
CREATE TABLE cea_run_log (
  run_hash     TEXT PRIMARY KEY,  -- blake3 of AttributedRunResult content
  eval_id      TEXT NOT NULL,
  edges_added  INTEGER NOT NULL,
  edges_updated INTEGER NOT NULL,
  processed_at TEXT NOT NULL
);
```

### Indexes
```sql
CREATE INDEX idx_cea_edges_cause   ON cea_edges(cause_node_id);
CREATE INDEX idx_cea_edges_effect  ON cea_edges(effect_node_id);
CREATE INDEX idx_cea_edges_version ON cea_edges(version_id);
```

---

## 5. CEA Archive Cell Augmentation

MAP-Elites archive cells are augmented with CEA data:

```rust
struct ArchiveCellCea {
  cell_key: String,
  dominant_cause_sigs: Vec<EditOpSignature>,  // top 3 cause signatures in this cell
  causal_fingerprint: String,                  // blake3 of sorted dominant_cause_sigs
  mean_correctness_confidence: f64,
  zero_shot_eligible: bool,
}
```

Archive cells in the same bin should ideally have *different* causal fingerprints — this
provides causal diversity on top of strategy diversity. The MAP-Elites emitter can use
causal fingerprint distance as an additional diversity dimension.

---

## 6. BasisVersion CEA Snapshot

When a candidate is promoted to BasisVersion:
1. Compute `CausalFingerprint` — the set of dominant cause-effect edges for this version's
   patch family (all eval runs attributed to this version).
2. Freeze the fingerprint (JSON + checksum) into the `promotions` table.
3. At runtime, after each instrumented run, compare against the frozen fingerprint.
   Drift beyond threshold → emit `CausalDriftWarning`. This is a regression signal that
   the codebase has changed in ways that affect this version's causal model.

---

## 7. Privacy and Security
- All causal data stays in local `forge.db`.
- `cea_nodes.sig_json` stores hashes and structural features, NOT raw source code.
- The `context_hash` in `EditOpSignature` is one-way; raw context cannot be recovered.
- CEA data must never be sent to remote model endpoints.

---

## 8. Implementation notes
- Use `petgraph::DiGraph` in memory during a run; serialize to `cea_edges` table on commit.
- For scope_tag inference: scan `context_before` lines for keywords; take the innermost match.
- For line_offset_from_edit: after patch apply, maintain a line-mapping table
  (original line → patched line) to translate check output positions back to edit positions.
- Start with pass/fail at file level if position mapping is unavailable; improve incrementally.
- CEA is additive — early runs with sparse graphs are valid; predictions simply have low
  confidence and are not zero-shot eligible.

---

## 9. Roadmap beyond v1
- v2: embed `EditOpSignature` vectors (small learned embedding) for fuzzy matching of
  structurally similar but not identical edit signatures → improves coverage on new patches.
- v3: cross-codebase transfer — export/import causal graph segments for similar project
  structures (optional; privacy implications must be evaluated).
- v4: active learning — CEA suggests which patch variations to evaluate next to maximize
  graph coverage in low-confidence regions (closes the loop with MAP-Elites emitters).
