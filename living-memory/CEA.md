# CEA.md
# Causal Edit Attribution — Specification v1.0
# [PROPRIETARY — Do not publish]

## Overview

**Causal Edit Attribution (CEA)** is a local experimental subsystem for learning
edit/effect associations and testing causal hypotheses with matched execution and
bounded interventions.

```text
patch -> fresh matched baseline/patched checks -> differential effects
      -> observational proximity hypotheses + patch-level receipt
      -> bounded edit ablations -> intervention receipts
```

The graph in `forge.db` stores observational associations only. It can produce an
advisory `CausalPrediction` for prioritization, but graph coverage is not causal proof
and never authorizes check skipping. Patch-level and edit-ablation evidence remain
typed, integrity-bound receipts whose claims are limited to the captured workload.

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

### 1.3 Association edge
A directed observational edge: `EditOpSignature -> EffectSignature`.
- `weight`: accumulated normalized attribution contribution;
- `count`: observed positive/negative sample count;
- `alpha` / `beta`: persisted Beta evidence;
- `version_id`: scoped model version; and
- `confidence`: conservative reliability times sample-sufficiency and coverage factors.

An edge localizes a candidate relationship; it is not an intervention result.

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
- `zero_shot_eligible`: currently always false for association-only graph evidence

### 1.6 RiskFlag
- `op_signature`: EditOpSignature — the edit op that is risky
- `predicted_effect`: EffectSignature — the likely bad effect
- `confidence`: f64

---

## 2. Instrumentation Protocol

### 2.1 Instrumented run flow
```
run_and_observe(fixture, patch, config):
  1. prepare independent baseline and patched workspaces
  2. capture baseline provenance for both arms and verify comparability
  3. run the same checker plan on both arms
  4. apply the structured patch only to the patched arm and retain its line map
  5. remove baseline-stable effects from the patched result
  6. normalize proximity contributions across candidate edits
  7. persist only observational triples transactionally
  8. emit a patch-level paired-intervention receipt plus advisory predictions
  9. optionally remove one edit at a time in fresh workspaces and emit ablation receipts
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

### 2.3 Edge updates
Candidate scores combine distance, severity, and line-map confidence, then use a
bounded softmax so all causes for one effect sum to `1.0`. `cea-store` persists
those normalized weights exactly; it does not recompute a second scoring formula.
When a known cause recurs without a previously seen effect, the edge receives a
negative observation. Optional historical decay is version-scoped.

Stable baseline passes/failures and fixed baseline failures do not become proximity
edges. Improvements remain patch-level local outcome evidence.

### 2.4 Integrity and idempotency
Identified runs have two digests:

- `run_hash` binds observation identity plus observed content for integrity; and
- `observation_key` binds stable execution identity for idempotency.

`update_graph` records the idempotency key in `cea_run_log`. Replaying the same
identity with altered content cannot update learning twice, while independent trial
IDs contribute independently. Legacy unidentified runs fall back to their content hash.

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

### 3.4 Prediction gate
Coverage is necessary for a useful advisory prediction but is never sufficient to
replace verification. The engine returns `RunChecks` when any precondition fails,
including disabled opt-in, insufficient independent runs, low/partial coverage,
fuzzy-only evidence, scope/config mismatch, missing intervention evidence, risk
flags, or unknown effects.

The association graph deliberately reports `zero_shot_eligible = false`; therefore
the current runtime does not skip checks even if `cea.enable_zero_shot` is set.

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
  run_hash     TEXT PRIMARY KEY,  -- observation idempotency key; legacy rows may use content hash
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
  advisory_prediction_only: bool,
}
```

Archive cells in the same bin may use different observational association fingerprints as
a diversity heuristic. Fingerprint distance does not establish causal diversity and cannot
change the mandatory verification gate.

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
- Sparse graphs are valid but advisory. Unknown signatures blend toward a neutral prior,
  fuzzy matching is off by default, and the prediction gate remains fail-closed.

---

## 9. Roadmap beyond v1
- v2: embed `EditOpSignature` vectors (small learned embedding) for fuzzy matching of
  structurally similar but not identical edit signatures → improves coverage on new patches.
- v3: cross-codebase transfer — export/import causal graph segments for similar project
  structures (optional; privacy implications must be evaluated).
- v4: active learning — CEA suggests which patch variations to evaluate next to maximize
  graph coverage in low-confidence regions (closes the loop with MAP-Elites emitters).
