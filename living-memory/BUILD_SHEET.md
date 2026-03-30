# BUILD_SHEET.md
# forge-engine v1.1 — Perfect Build Sheet

## Mission
Implement **forge-engine**, a new Rust crate that:
1. Compiles memory + repo signals into a structured **MindState** using a **promoted** algebra version.
2. Generates **StructuredPatch** outputs and renders them into unified diffs.
3. Validates diffs with **host or container** execution backends (auto-detect container runtime).
4. Implements stabilization policy: **Innovative attempt → Stabilize #1 → Stabilize #2 → Clamp novelty**.
5. Provides **ForgeLab** discovery loop (MAP-Elites) over candidate algebras, evaluating on Rust fixtures.
6. **[PROPRIETARY]** Builds a **Causal Edit Attribution (CEA)** graph: instruments build/test pipelines to
   trace which specific edit operations caused which outcomes. Over runs, this produces a codebase-specific
   predictive layer that eventually allows zero-shot patch validation — predicting correctness from topology
   alone before running a single check. See `CEA.md` for the full specification.

## Absolute Constraints (non-negotiable)
- **DO NOT MODIFY semantic-memory behavior, schema, migrations, or tests.**
- Forge must use **its own DB (`forge.db`)** and refuse to open any DB that is not a Forge DB.
- Forge must be Rust-first; target domain: **Cargo projects** (fmt/clippy/test).
- Patch generation uses **StructuredPatch v1** (internal), rendered to unified diffs.
- Container backend must work with **any** runtime available (Docker/Podman/nerdctl). If none,
  gracefully degrade to host backend.
- CEA instrumentation must be **opt-in per run** and **never alter check semantics**.

## Repo Layout (must match)
Create new crate:
- `crates/forge-engine/` (prefer workspace crate; sibling repo acceptable)

Do NOT move or rename existing `semantic-memory` crate.

Required directory tree (within new crate):
```
src/
  lib.rs
  error.rs
  config.rs
  invariants.rs
  store/
    db.rs
    schema.rs
  runtime/
    mindstate.rs
    compiler.rs
    novelty.rs
    stabilizer.rs
    patch/
      types.rs
      validate.rs
      apply.rs
      render_diff.rs
  exec/
    backend.rs
    host.rs
    container.rs
  adapters/
    cargo.rs
  lab/
    evaluate.rs
    archive.rs
    emitters.rs
    promote.rs
    suite.rs
  cea/
    instrumentation.rs
    graph.rs
    predictor.rs
    store.rs
migrations/
tests/
examples/
```

## Dependencies (approved)
Prefer minimal, reliable crates:
- `tokio`, `serde`, `serde_json`, `thiserror`, `anyhow`
- `uuid`, `blake3`
- `sqlx` (sqlite) OR `rusqlite` — pick ONE, be consistent
- `tempfile`, `walkdir`
- `once_cell`
- `petgraph` — for the CEA causal graph
- `regex` — optional but useful for anchor matching and CEA pattern extraction
- **DO NOT** add heavy ML libs. Embeddings are optional in v1. CEA prediction uses graph
  structural features only (no embeddings required).

## Definition of Done (DoD)

### DoD-A — Safety guarantees
- Forge refuses to open `memory.db` (semantic-memory DB) under all default configs.
- Forge writes only to `forge.db`.
- CI gates ensure semantic-memory tests and golden snapshots are unchanged.

### DoD-B — Runtime v0 works
ForgeRuntime can:
- Compile MindState (deterministic render)
- Accept a generator output as StructuredPatch
- Validate/apply patch to a temp workspace
- Render unified diff
- Run checks via HostBackend or ContainerBackend
- Apply stabilization policy and clamp novelty when needed

### DoD-C — Lab v0 works
ForgeLab can:
- Load task fixtures
- Evaluate candidates (generate → patch → apply → checks → score)
- Maintain MAP-Elites archive
- Promote a candidate to BasisVersion with frozen bounds + checksum

### DoD-D — CEA v0 operational [PROPRIETARY]
CausalAttributionEngine can:
- Instrument a check run and capture per-edit-op outcome signals
- Build/update the causal graph in `forge.db`
- Query graph for patch topology → predicted score (zero-shot)
- Report causal confidence and coverage over archive cells

## Milestones (implement in order, no skipping)
1. **Safety scaffolding** — config + forge.db + refuse-to-open + CI gates
2. **StructuredPatch pipeline** — types + validate + apply + diff rendering
3. **Execution backends** — HostBackend + ContainerBackend (runtime autodetect)
4. **Cargo adapter** — fmt/clippy/test runner
5. **Runtime** — MindState compiler + novelty + stabilization loop + traces
6. **Eval harness** — fixture format + scoring + runner
7. **MAP-Elites** — archive bins + emitters (param mutation + crossover)
8. **Promotion** — graduation contract + BasisVersion freezing + golden MindState vectors
9. **CEA instrumentation** — per-op outcome capture + graph construction + predictor
10. **LLM mutator (optional in v1.1)** — structured AlgebraSpec mutation

## Build/Run commands (must work)
From workspace root:
```bash
cargo test -p semantic-memory            # must pass unchanged
cargo test -p forge-engine
cargo clippy -p forge-engine -- -D warnings
cargo fmt --all -- --check
```

Optional CLI:
```bash
cargo run -p forge-engine -- eval --suite <path>
cargo run -p forge-engine -- runtime --repo <path> --request "<text>"
cargo run -p forge-engine -- cea --query <patch-json>   # zero-shot prediction
```

## Read these documents and implement exactly
- SPEC.md
- ARCHITECTURE.md
- INVARIANTS.md
- CONFIG.md
- DB_SCHEMA.md
- PATCH_FORMAT.md
- EXECUTION.md
- ADAPTERS.md
- EVAL_HARNESS.md
- MAP_ELITES.md
- PROMOTION.md
- CEA.md              ← proprietary; implement carefully
- TEST_PLAN.md
- SECURITY.md
- AGENTS.md
- CLAUDE.md
