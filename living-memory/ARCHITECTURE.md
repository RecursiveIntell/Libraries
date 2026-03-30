# ARCHITECTURE.md
# Architecture

## High-level dataflow

```
User request + repo context + evidence + traces
  → MindState compiler (BasisVersion)
  → generator (external LLM or other agent) outputs StructuredPatch
  → patch validator → patch applier → diff renderer
  → [CEA instrumentation wraps next step]
  → execution backend runs checks (fmt/clippy/test)
  → score + trace write
  → CEA: (cause, effect) pair extraction → graph update
  → CEA prediction available for future patches
```

## Module responsibilities

---

### src/runtime/

**mindstate.rs**
- `MindState` structs
- Deterministic rendering function (stable sort + canonical serialization)

**compiler.rs**
- Compile `MindState` from inputs using `BasisVersion`
- Pull evidence from semantic-memory handle (read-only)
- Inject answer traces for novelty context

**novelty.rs**
- Question signature (stable hash of request + repo key)
- Strategy-tag extraction from patch topology + metadata
- Δ policy (bounded novelty controller): amplitude per attempt phase

**stabilizer.rs**
- Implements attempt loop: innovative → stabilize1 → stabilize2 → clamp
- Produces per-attempt overrides to compiler/Δ policy
- Enforces attempt order invariant (see INVARIANTS.md §I6)

**patch/types.rs**
- `StructuredPatch`, `FileEdit`, `EditOp`, `Anchor`, `LineRange`

**patch/validate.rs**
- Policy checks: forbidden paths, caps, empty-patch guard
- Returns `ValidationResult` with all violations (not fail-fast)

**patch/apply.rs**
- Apply `StructuredPatch` to a workspace directory
- Anchor resolution: line anchors with context verification, match anchors
- Atomic: fails entirely on any op failure (no partial apply)
- Returns line-mapping table for CEA position attribution

**patch/render_diff.rs**
- Generate unified diff from (original_dir, patched_dir)
- Prefer `git diff --no-index` if git on PATH
- Fallback: internal line-diff (must produce valid unified format)

---

### src/exec/

**backend.rs**
- `ExecutionBackend` trait
- Backend selection logic (auto-detect based on config + available runtimes)

**host.rs**
- `HostBackend`: runs commands via `std::process::Command`
- Environment sanitization
- Configurable timeouts per command

**container.rs**
- `ContainerBackend`: autodetect docker > podman > nerdctl
- Container lifecycle: create → mount workspace → run commands → collect output → destroy
- Sealed mode: enforces `--network=none` (or equivalent)

---

### src/adapters/

**cargo.rs**
- `CargoAdapter` implements `ProjectAdapter`
- Detection: Cargo.toml at root
- Commands: `cargo fmt --all -- --check`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test --all --all-features`
- Output parsing for CEA: structured check output → `ParsedCheckOutput`

---

### src/store/

**db.rs**
- `ForgeStore` trait + SQLite implementation
- Migrations runner (forge-only)
- DB refuse-to-open enforcement (delegates to invariants.rs)

**schema.rs**
- `FORGE_SCHEMA_HASH` constant
- `FORGE_USER_VERSION` range
- Required table names

---

### src/lab/

**evaluate.rs**
- Run one candidate over suite tasks (with or without CEA instrumentation)
- Calls runtime → checks → score → persist `EvalRun`
- Optionally calls CEA instrumentation and `update_graph`

**archive.rs**
- MAP-Elites archive CRUD
- Cell key computation
- Insert/replace logic (higher score_summary wins)
- CEA archive cell augmentation (see CEA.md §5)

**emitters.rs**
- E1: Param mutation — perturb numeric parameters in `AlgebraSpec`
- E2: Crossover — merge two parent specs
- E3: LLM mutator (optional) — structured `AlgebraSpec` JSON output

**promote.rs**
- Graduation contract enforcement
- `BasisVersion` creation + checksum
- Golden MindState vector generation
- CEA fingerprint snapshot (see CEA.md §6)

**suite.rs**
- Load/parse fixture directories and `task.json` files
- Validate fixture structure

---

### src/cea/

**instrumentation.rs**
- `CausalInstrument` trait
- Wraps a `CheckRun`, captures raw check outputs with timing
- Parses `ParsedCheckOutput` → `EffectSignature` list with positions

**graph.rs**
- In-memory `petgraph::DiGraph` of `EditOpSignature` → `EffectSignature`
- Edge weight computation (attribution_score formula from CEA.md §2.3)
- Serialize to / deserialize from `cea_edges` table

**predictor.rs**
- `CausalPrediction` computation from graph + patch
- Coverage fraction, risk flag extraction, zero-shot eligibility
- Correctness prediction formula (CEA.md §3.3)

**store.rs**
- Persist/load CEA data (cea_nodes, cea_edges, cea_run_log)
- Idempotent update (run_hash deduplication)

---

### src/invariants.rs
- All hard rule enforcement functions
- `refuse_to_open_db(path)` — checks forge_meta, user_version, schema_hash
- `validate_forbidden_paths(patch, config)` — glob matching
- `validate_patch_caps(patch, config)` — line/file count caps
- `validate_cea_no_raw_source(node)` — ensures CEA nodes store only hashes/features

---

### src/config.rs
- `ForgeConfig` with serde (JSON + TOML)
- All default values (see CONFIG.md)

---

## Runtime invariant enforcement points

| Invariant         | Enforced in                      |
|-------------------|----------------------------------|
| DB refuse-to-open | `store/db.rs` + `invariants.rs`  |
| Forbidden paths   | `patch/validate.rs`              |
| Patch caps        | `patch/validate.rs`              |
| Attempt order     | `runtime/stabilizer.rs`          |
| No network sealed | `exec/container.rs`              |
| CEA idempotent    | `cea/store.rs` (run_hash check)  |
| CEA no raw source | `cea/instrumentation.rs`         |

## External boundaries
- `semantic-memory`: read-only via handle passed by caller; Forge never opens memory.db
- Generator (LLM/agent): external interface; Forge receives `StructuredPatch` only
- `petgraph`: in-memory graph; fully serialized to DB on commit
