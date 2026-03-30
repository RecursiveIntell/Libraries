# SPEC.md
# forge-engine v1.1 — Formal Specification

## 1. Scope
A new crate `forge-engine` that depends on `semantic-memory` (read-only).

### 1.1 Runtime
- `MindState` compilation from:
  - user request
  - repo context (Rust/Cargo)
  - evidence from semantic-memory hybrid search
  - answer traces for repeated-question novelty control
- Generation pipeline producing:
  - `StructuredPatch` (internal representation)
  - unified diff (rendered from applied patch)
- Validation pipeline:
  - policy + invariant checks (pre-apply)
  - patch apply to temp workspace
  - host/container checks (fmt/clippy/test)
- Stabilization policy (in order):
  1. Innovative attempt
  2. Stabilize #1
  3. Stabilize #2
  4. Clamp novelty (Δ→0)

### 1.2 Lab
- Candidate representation: `AlgebraSpec`
- Evaluation suite runner (Rust fixtures)
- MAP-Elites archive (quality diversity)
- Promotion to immutable `BasisVersion`

### 1.3 Causal Edit Attribution (CEA) [PROPRIETARY]
CEA is a subsystem that goes beyond pass/fail scoring by attributing observable build/test
outcomes to specific edit operations in a StructuredPatch.

**Core idea:** Each EditOp (Insert/Replace/Delete at a given anchor) is a *cause candidate*.
Each check outcome (fmt-fail line, clippy lint, test failure message) is an *effect*. CEA
instruments runs to collect (cause, effect) pairs and builds a directed causal graph stored
in `forge.db`. Over many runs, the graph gains predictive power — allowing Forge to estimate
patch correctness from topology before executing any check.

**Why this is novel:**
- Existing code-gen eval systems treat runs as black boxes: patch in, score out.
- CEA breaks the black box: it builds a codebase-specific causal model of how edits interact
  with the build/test system.
- The causal graph is trained on *your* codebase and is proprietary IP stored locally.
- Once the graph has sufficient coverage, it enables zero-shot validation: predict score from
  patch topology without running fmt/clippy/test.

See `CEA.md` for the full specification.

## 2. Non-functional requirements
- Must NOT modify `semantic-memory` crate, DB schema, migrations, or tests.
- Must be deterministic where required:
  - MindState rendering is deterministic given same inputs + BasisVersion.
  - CEA graph updates are idempotent (same run data produces same edge weights).
- Must have strong local-only mode (sealed):
  - Container runs with no network.
  - Model routing forbids remote.
- Must degrade gracefully if container runtime unavailable (use host backend).
- CEA instrumentation must not affect check correctness or timing semantics.

## 3. Public API (minimum)
Expose in `lib.rs`:

### Types
- `MindState`
- `StructuredPatch` + `FileEdit` + `EditOp` + `Anchor`
- `AlgebraSpec`
- `BasisVersion`
- `ForgeConfig`
- `ExecutionBackendKind` (Host | Container)
- `EvalTask`, `EvalRunResult`, `ScoreVector`
- `CausalGraph`, `CausalPrediction`, `EditOpSignature`  ← CEA types

### Traits / Interfaces
- `ExecutionBackend`
- `ProjectAdapter`
- `ForgeStore` (forge.db)
- `ModelRouter` (optional; remote-first, local optional)
- `CausalInstrument` — wraps a check run and captures attribution signals

### Runtime functions
- `ForgeRuntime::new(config, store, semantic_memory_handle)`
- `ForgeRuntime::compile_mindstate(...) -> MindState`
- `ForgeRuntime::run_attempts(...) -> RuntimeResult`
- `ForgeRuntime::apply_and_validate_patch(...) -> PatchValidationResult`

### Lab functions
- `ForgeLab::new(config, store, suite)`
- `ForgeLab::run_generation(...) -> GenerationReport`
- `ForgeLab::promote(candidate_id) -> BasisVersion`

### CEA functions
- `CausalAttributionEngine::new(config, store)`
- `CausalAttributionEngine::instrument_run(patch, backend) -> AttributedRunResult`
- `CausalAttributionEngine::update_graph(attribution_result)`
- `CausalAttributionEngine::predict(patch) -> CausalPrediction`
- `CausalAttributionEngine::coverage() -> CoverageSummary`

## 4. Initial domain: Rust/Cargo
Evaluation correctness uses:
- `cargo fmt --check`
- `cargo clippy -- -D warnings`
- `cargo test`

## 5. Patch contract
- Generator must output `StructuredPatch`.
- Forge must validate + apply patch in temp workspace.
- Forge must render unified diff from workspace changes.
- By default: forbid touching tests/fixtures/snapshots/lockfiles unless explicitly enabled.

## 6. Configuration requirements
All behavior driven by `ForgeConfig` with safe defaults. See `CONFIG.md`.

## 7. Persistence
All Forge state in `forge.db` only. See `DB_SCHEMA.md`.
Forge must refuse to open non-Forge DBs. See `INVARIANTS.md`.

## 8. Promotion
Promotion requires passing graduation contract. See `PROMOTION.md`.
Promoted BasisVersions gain a frozen CEA fingerprint: the expected causal signatures
for that version's patch family, used as a regression signal.
