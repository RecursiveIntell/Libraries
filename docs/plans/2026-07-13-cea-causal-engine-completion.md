# CEA Causal Engine Completion Implementation Plan

> **For Hermes:** Execute this plan task-by-task with strict RED-GREEN-REFACTOR. Use Codex CLI for implementation, then independently review and verify every task-owned diff.

**Goal:** Turn the existing CEA stack and `forge-engine` into a coherent, receipt-bearing experimental engine that distinguishes association from intervention, executes real paired and bounded ablation trials, learns without corrupting attribution weights, and never skips checks without an explicit evidence gate.

**Architecture:** `cea-core` remains the deterministic domain/statistics layer; `cea-store` owns transactional update semantics; `cea-sqlite` owns durable, versioned storage; `forge-engine` owns execution, pairing, comparability, intervention, and receipts; `forge-pilot` consumes engine outputs without inventing scores; `cea-bridge` is isolated Hermes tool-outcome telemetry and cannot train the code-edit model. Causal claims require intervention-qualified evidence; proximity-only edges remain observational attribution hypotheses.

**Tech stack:** Rust 2021/MSRV 1.75, `petgraph`, `rusqlite`, `serde`, `blake3`, `tokio`, existing `typed-patch`/`check-runner`/`sandbox-workspace`; Python pytest for the Hermes adapter.

---

## Evidence-backed current state — 2026-07-13

**Canonical roots inspected**

- `/home/sikmindz/Coding/Libraries/Primitives/cea-core`
- `/home/sikmindz/Coding/Libraries/Primitives/cea-store`
- `/home/sikmindz/Coding/Libraries/Primitives/cea-sqlite`
- `/home/sikmindz/Coding/Libraries/living-memory/living-memory` (`package.name = "forge-engine"`)
- `/home/sikmindz/Coding/Libraries/cea-bridge`
- `/home/sikmindz/Coding/Libraries/forge-pilot`
- `/home/sikmindz/.hermes/hermes-agent/plugins/context_engine/context_governor/__init__.py`

**Correction to the prior audit**

`forge-engine` is not the 46-line `/Libraries/forge-engine` stub path. The real crate is `/Libraries/living-memory/living-memory`, package `forge-engine` v0.2.0. It builds and has a substantial execution/evidence implementation. The stale skill/report statement must be removed.

**Verified commands**

- `cargo test -p cea-core -p cea-store -p cea-sqlite -p forge-engine --all-targets` — **207 passed, 0 failed**.
- `cargo test --locked` in `cea-bridge` — **7 passed, 0 failed**.
- `cargo test -p forge-pilot --all-targets` — broad suite ran; **one current-tree baseline failure** remains in `verification_control_tests::full_loop_refuted_promoted_state_triggers_rollback_invalidation` with `MissingCanonicalClaimId { record_index: 0 }`. This is outside the CEA target surface and must not be falsely attributed to this pass.
- Task target paths were clean at start: `git diff --quiet -- Primitives/cea-core Primitives/cea-store Primitives/cea-sqlite cea-bridge living-memory/living-memory forge-pilot` returned success.

**Live telemetry receipt**

`~/.hermes/cea.db` currently contains 1,819 nodes, 1,817 edges, and 1,817 run-log rows across 9 eval IDs. All 1,817 persisted confidence values are zero; min observations = 1, max = 2, mean ≈ 1.008. Existing rows are Hermes synthetic tool telemetry, not validated patch/check causal evidence.

**Confirmed defects**

1. `effective_sample_size(observations)` subtracts two prior units even though callers pass observed sample units; observations 1–2 therefore receive exactly zero confidence.
2. `cea-store::update_graph` discards normalized `AttributionTriple.weight` and recomputes a different distance/severity score. Persisted learning disagrees with in-memory learning.
3. Content-only run hashes deduplicate independent repeated trials that happen to produce identical evidence.
4. Prediction double-penalizes sample size, takes the minimum confidence over all outgoing effects, fails to blend unknown signatures with the neutral prior, and uses shared prefixes of BLAKE3 hex as “similarity.”
5. `PairedExperimentRunner` discards the line map, ignores `trial_count`/`RepeatedPaired`, never attributes effects, never predicts, and never updates CEA.
6. Stable baseline failures can be fed to patched-run attribution unless a differential check view is constructed.
7. Ablation is plan-only; no patch ablation is built or executed.
8. `forge-pilot` hard-codes correctness/novelty/stability/weighted totals, `PairComparability { valid: true }`, hypothesis confidence/status, and leaves `attribution_json: None`.
9. The Hermes bridge uses fake 16-hex “blake3,” hard-coded edit fields, ambiguous-result→success classification, no tool-call identity, ignores focus, and writes synthetic tool telemetry through code-edit types.
10. The plugin records after pruning/summarizing tool results, so evidence can already be degraded.

## Research basis and design decisions

Primary sources read on 2026-07-13:

- Pietrantuono et al., **“Causal Software Engineering: A Vision and Roadmap”**, arXiv:2605.02454. Design implications: explicit intervention logs, confounders/comparability, uncertainty-qualified estimates, refutation/placebo checks, stability across releases, abstention when identification fails, and benchmark families for intervention effects/counterfactual incidents/causal testing.
- Bonagiri et al., **“CausalFlow: Causal Attribution and Counterfactual Repair for LLM Agent Failures”**, arXiv:2605.25338. Design implications: candidate localization is not proof; causal responsibility requires replacing/removing a candidate and deterministic downstream re-execution. Bounded intervention count is practical; minimal local interventions and executable verifiers are preferred.
- Zeller and Hildebrandt, **Delta Debugging**. Design implication: bounded removal/re-execution is the practical path to minimal failure-inducing edit sets; singleton ablations are the first deterministic tier, group minimization is budgeted follow-up.
- Beta-binomial calibration literature. Design implication: Beta prior mass is not observed sample count. Low-sample evidence should be nonzero but strongly conservative; confidence and sample sufficiency must be separate and monotone.

**Convergence thesis:** CEA must treat patch application and edit ablation as explicit, receipt-bearing interventions over a fixed workload, while proximity and historical co-occurrence remain lower-grade observational hypotheses that can prioritize tests but cannot independently mint causal claims.

---

## Sprint A — Core evidence and prediction truth

### Task A1: Fix sample semantics and conservative confidence

**Files**
- Modify: `Primitives/cea-core/src/calibration.rs`
- Test: `Primitives/cea-core/src/tests.rs`

**RED tests**

- Zero observed samples yield zero sample factor.
- One observed sample yields a positive but low confidence.
- Confidence is monotone with independent samples.
- Contradictions reduce conservative reliability.

**Implementation**

Treat the function argument as observed sample units; do not subtract the Beta prior twice. Preserve a separate minimum-samples sufficiency penalty. Document exact semantics.

**Gate**

`cargo test -p cea-core calibration -- --nocapture`

### Task A2: Add typed observation identity and evidence grade

**Files**
- Modify: `Primitives/cea-core/src/attribution.rs`
- Modify: `Primitives/cea-core/src/types.rs`
- Modify: `Primitives/cea-core/src/lib.rs`
- Test: `Primitives/cea-core/src/tests.rs`

**RED tests**

- Same evidence with the same observation ID hashes identically.
- Same evidence from different independent trial IDs hashes differently.
- Legacy `AttributedRunResult::new` remains deterministic for replay compatibility.
- `SyntheticTelemetry` is not accepted as code-interventional evidence.

**Implementation**

Add `EvidenceKind` (`Observational`, `PairedInterventional`, `Ablation`, `Counterfactual`, `SyntheticTelemetry`) and `ObservationIdentity` with stable observation/run/trial IDs plus optional patch/base/config digests. Add a constructor that includes identity in the run hash. Keep a legacy constructor for backward compatibility.

**Gate**

`cargo test -p cea-core run_hash -- --nocapture`

### Task A3: Preserve attribution weights and unify in-memory/persisted updates

**Files**
- Modify: `Primitives/cea-store/src/lib.rs`
- Modify: `Primitives/cea-sqlite/src/lib.rs`
- Modify: `living-memory/living-memory/src/cea/store.rs`
- Modify: `living-memory/living-memory/src/config.rs`
- Modify: `cea-bridge/src/main.rs`
- Tests: each crate’s existing unit tests

**RED tests**

- Two causes for one effect persist contributions summing to 1.0.
- Changing softmax temperature changes persisted weights.
- SQLite and in-memory graph predictions match after round-trip.
- Same observation ID is idempotent; distinct trial IDs each contribute once.

**Implementation**

Persist `AttributionTriple.weight` exactly. Separate graph historical decay from distance scoring; never use one field for both. Apply any graph decay consistently in memory and SQLite or disable it rather than diverge. Keep updates transactional.

**Gate**

`cargo test -p cea-core -p cea-store -p cea-sqlite --all-targets`

### Task A4: Replace invalid prediction generalization

**Files**
- Modify: `Primitives/cea-core/src/predict.rs`
- Test: `Primitives/cea-core/src/tests.rs`

**RED tests**

- Unknown signatures pull prediction toward 0.5 in proportion to coverage.
- Changing only a cryptographic hash prefix does not create similarity.
- Exact match has coverage 1.0; fuzzy match remains capped and advisory.
- Patch order does not create a high-confidence generalized match by itself.
- Low-confidence unrelated outgoing effects do not collapse all confidence to zero.

**Implementation**

Remove hash-prefix similarity. Use interpretable structural features with an explicit minimum similarity and fuzzy coverage cap; default fuzzy matching off or advisory-only until calibrated. Aggregate evidence without double sample penalties. Zero-shot remains false without independent interventional support.

**Gate**

`cargo test -p cea-core predict -- --nocapture`

---

## Sprint B — Best practical forge causal engine

### Task B1: Make paired execution truthful and repeatable

**Files**
- Modify: `living-memory/living-memory/src/experiment.rs`
- Modify: `living-memory/living-memory/src/config.rs`
- Test: `living-memory/living-memory/tests/experiment_tests.rs`
- Create integration tests as needed under `living-memory/living-memory/tests/`

**RED tests**

- `RepeatedPaired` with three trials yields six arm records and uses fresh workspaces.
- `trial_count = 0` is rejected or normalized explicitly.
- Baseline fingerprint drift invalidates comparability.
- Unknown network/cache state is not hard-coded as known-good.
- Single pair remains `statistically_meaningful = false`; repeated consistent pairs may become meaningful only at the configured minimum.

**Implementation**

Extract `run_pair`; execute independent fresh baseline/patched workspaces per pair; retain line maps; keep per-pair outcomes; aggregate conservatively; record explicit comparability and sample warnings. Do not randomize arm order when applying patch before baseline would contaminate the control; use matched fresh workspaces and deterministic scheduling.

**Gate**

`cargo test -p forge-engine --test experiment_tests`

### Task B2: Build `CausalAttributionEngine`

**Files**
- Create: `living-memory/living-memory/src/cea/engine.rs`
- Modify: `living-memory/living-memory/src/cea/mod.rs`
- Modify: `living-memory/living-memory/src/lib.rs`
- Test: create `living-memory/living-memory/tests/causal_engine_tests.rs`

**API**

- `predict_patch(...) -> PredictionReceipt`
- `observe_pair(...) -> CausalUpdateReceipt`
- `run_and_observe(...) -> CausalExperimentResult`
- `coverage(...) -> CoverageSummary`

**RED tests**

- A baseline-stable failure is not attributed to the patch.
- A new patched failure is attributed using patched-space line mapping.
- A fixed baseline failure creates improvement evidence without claiming general causality.
- Receipt digests bind observation identity, patch/base/config digests, evidence kind, triple digest, update disposition, prediction disposition, and degradation reasons.
- Failed persistence leaves graph and run log unchanged.

**Implementation**

Construct a differential check view before attribution. Persist observational hypotheses separately from paired intervention evidence. A receipt proves execution/integrity, not causality. Default policy always runs checks.

**Gate**

`cargo test -p forge-engine --test causal_engine_tests`

### Task B3: Execute bounded single-edit ablations

**Files**
- Add to: `living-memory/living-memory/src/cea/engine.rs`
- Modify: `living-memory/living-memory/src/experiment.rs`
- Test: `living-memory/living-memory/tests/causal_engine_tests.rs`

**RED tests**

- In a two-op fixture where only op A causes a failure, removing A flips the outcome and removing B does not.
- Empty ablation equals the baseline arm and is handled without an invalid empty patch.
- An infeasible ablation is `Inconclusive`, emits a receipt, and never reinforces a causal edge.
- Ablation count obeys a hard configured budget.

**Implementation**

Generate a clean patch with one operation removed, run it in a fresh workspace with the same checker plan, compare baseline/full/ablated outcomes, and emit `Supported`, `Contradicted`, or `Inconclusive`. Use singleton ablations now; expose a budgeted group-ablation interface but do not claim minimality until ddmin is actually executed.

**Gate**

`cargo test -p forge-engine --test causal_engine_tests ablation -- --nocapture`

### Task B4: Enforce prediction policy rather than decorative config

**Files**
- Modify: `living-memory/living-memory/src/cea/predictor.rs`
- Modify: `living-memory/living-memory/src/config.rs`
- Add to engine tests

**RED tests**

Every failed precondition returns `RunChecks` plus reasons: disabled opt-in, insufficient independent runs, low coverage, fuzzy-only evidence, scope/config mismatch, missing interventional evidence, risk flags, or unknown effects. No default path skips checks.

**Gate**

`cargo test -p forge-engine prediction_gate -- --nocapture`

---

## Sprint C — Consumer and bridge integrity

### Task C1: Wire forge-pilot to real engine evidence

**Files**
- Modify: `forge-pilot/src/act.rs`
- Modify: `forge-pilot/src/bundle_builder.rs`
- Modify: `forge-pilot/src/loop_runner.rs` if the canonical `ForgeStore` must be passed
- Test: `forge-pilot/tests/loop_roundtrip_tests.rs`
- Add focused bundle tests

**RED tests**

- Patch action persists one CEA observation and exports matching attribution receipt.
- Bundle correctness derives from actual required check outcomes, not hard-coded constants.
- Stability is absent/zero when one pair is run.
- Comparability is copied from engine evidence, never forced true.
- Hypothesis status/confidence reflect support/contradiction receipts and remain provisional.

**Implementation**

Use the already-open canonical Forge store; do not open a shadow database. Preserve CEA failures as explicit degradation/warnings. Populate `cea_confidence`, `cea_predicted_correctness`, `attribution_json`, hypothesis edges, receipts, verification trials, and refutation artifacts from actual engine outputs.

**Gate**

`cargo test -p forge-pilot --test loop_roundtrip_tests`

### Task C2: Quarantine Hermes telemetry and repair bridge identity

**Files**
- Modify: `cea-bridge/Cargo.toml`
- Modify: `cea-bridge/src/main.rs`
- Add bridge tests

**RED tests**

- BLAKE3 hashes are 64 hex characters and deterministic.
- Tool-call telemetry cannot load as Forge code-interventional evidence.
- Ambiguous result is `unknown`, not success.
- `(session_id, tool_call_id, result_digest)` is idempotent while distinct calls can contribute independently.
- Version/reporting identifies legacy `hermes-agent-v1` as quarantined.

**Implementation**

Add real `blake3`. Use a telemetry-specific version/model namespace and explicit evidence kind; never reinterpret the 1,817 legacy rows as code CEA. Keep the old data readable as legacy telemetry. Do not destructively rewrite the live DB.

**Gate**

`cargo test --locked` in `cea-bridge`

### Task C3: Harden the installed Hermes adapter

**Files**
- Modify: `/home/sikmindz/.hermes/hermes-agent/plugins/context_engine/context_governor/__init__.py`
- Modify: `/home/sikmindz/.hermes/hermes-agent/tests/plugins/test_context_governor_plugin.py`

**RED tests**

- Tool telemetry is captured before pruning/summarization.
- Tool-call ID and explicit result digest are sent.
- Ambiguous result remains unknown and is not recorded as success.
- Bridge failure remains fail-open for compaction.
- Relevance protection is bounded and reports that it is telemetry-assisted/advisory, not causal proof.

**Gate**

`PYTHONDONTWRITEBYTECODE=1 python -m pytest tests/plugins/test_context_governor_plugin.py -q -o 'addopts=' -p no:cacheprovider`

### Task C4: Safe live transition

**Files**
- Create a migration/inspection command or script under `cea-bridge` or `cea-sqlite` tests
- Create local receipt under ignored `target/cea-migration/`; never commit raw telemetry

**Procedure**

- Back up `cea.db`, `cea.db-wal`, and `cea.db-shm` if present.
- Record hashes and SQLite integrity/schema inventory.
- Install the new bridge binary only after focused tests pass.
- Do not reinterpret v1 rows. Start v2 telemetry in a quarantined namespace/table/version.
- Run a bridge smoke test and adapter test.

**Binary gate**

A failed transition leaves the source DB byte-identical; integrity checks pass before and after; legacy counts remain visible and quarantined.

---

## Sprint D — Evaluation, documentation, and release gates

### Task D1: Add an offline replay/evaluation harness

**Files**
- Create: `living-memory/living-memory/examples/cea_replay_eval.rs` or a test-owned equivalent
- Create: `living-memory/living-memory/tests/fixtures/cea/` with tiny deterministic Rust fixtures
- Create: `living-memory/docs/benchmarks/CEA_ENGINE_LOCAL_2026-07-13.md`

**Metrics**

Exact/fuzzy coverage, Brier score where labels exist, risk precision/recall, calibration buckets, ablation localization accuracy, false-negative count, runtime/intervention count. Include full-run and naive proximity baselines. Raw fixture receipts stay under ignored `target/`.

**Claim boundary**

Synthetic/local fixture success proves engine mechanics only. It does not prove external superiority, production readiness, or safe zero-shot validation.

**Gate**

`cargo run -p forge-engine --example cea_replay_eval -- --output target/cea-eval/receipt.json`

### Task D2: Truthful docs and stale-skill repair

**Files**
- Modify: `living-memory/living-memory/README.md`
- Modify: `living-memory/CEA.md`
- Modify: CEA crate READMEs as needed
- Modify: `cea-bridge` docs/skill after source verification

Document evidence grades, observation identity, pairing, ablation, prediction gate, live telemetry quarantine, and exact non-claims. Remove the stale “forge-engine is a stub” skill content.

### Task D3: Full verification and independent review

**Focused gates**

1. `cargo test -p cea-core -p cea-store -p cea-sqlite --all-targets`
2. `cargo test -p forge-engine --all-targets`
3. `cargo test --locked` in `cea-bridge`
4. `cargo test -p forge-pilot --test loop_roundtrip_tests`
5. Hermes plugin pytest command above

**Broad gates**

6. `cargo check -p cea-core -p cea-store -p cea-sqlite -p forge-engine -p forge-pilot --all-targets`
7. `cargo clippy -p cea-core -p cea-store -p cea-sqlite -p forge-engine -p forge-pilot --all-targets -- -D warnings`
8. `cargo test -p forge-pilot --all-targets` — compare against the recorded one-failure baseline; any new failure blocks completion.
9. Independent diff review with security, logic, migration, claim-boundary, and change-surface checks.
10. Stage and commit only task-owned paths; do not absorb the pre-existing semantic-memory/example dirty tree.

---

## Public-safe claim boundary

Safe after all gates pass:

- The engine executes matched local paired patch trials and bounded edit ablations.
- It records deterministic, transactional, receipt-bound observational and interventional evidence.
- It preserves normalized attribution weights and enforces explicit prediction gates.
- Hermes tool telemetry is isolated from the code-edit causal model.
- Local deterministic fixtures validate mechanics and ablation localization.

Not safe without an external held-out corpus and reproduced results:

- “CEA proves causality” in general.
- “Zero-shot validation is safe.”
- “CEA outperforms fault localization, delta debugging, or other causal-debugging systems.”
- Production maturity, adoption, customer, compliance, or external superiority claims.

## Hard no list

- No raw source in CEA nodes or public receipts.
- No automatic reinterpretation of legacy Hermes rows as code causal evidence.
- No check skipping by default.
- No synthetic/hard-coded scores, comparability, confidence, or successful outcomes.
- No direct writes into semantic-memory projected truth.
- No destructive live-DB migration without backup, integrity checks, and a receipt.
- No publication of proprietary `CEA.md` or CEA crates; actual publish is out of scope and prohibited by the current source header.
- No push to a remote without explicit user instruction.
