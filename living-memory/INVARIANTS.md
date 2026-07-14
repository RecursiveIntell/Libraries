# INVARIANTS.md
# Invariants (Hard Rules)

These invariants are non-negotiable. Violations are bugs, not configuration issues.
Every invariant has a named enforcement point in code.

---

## I0 — DO NOT MODIFY semantic-memory
- No edits to semantic-memory code, schema, migrations, or tests.
- Enforced by CI diff gate: any PR touching `crates/semantic-memory/` fails.

## I1 — Forge DB isolation
All Forge state lives in `forge.db` only. Forge must refuse to open any DB that is
not a verified Forge DB.

### DB recognition rules (ALL must pass)
1. File must be a valid SQLite database (magic bytes check).
2. `PRAGMA user_version` must be within `[FORGE_MIN_USER_VERSION, FORGE_MAX_USER_VERSION]`.
3. Table `forge_meta` must exist.
4. `forge_meta` row `key = 'schema_hash'` must exist.
5. `forge_meta.value` for `schema_hash` must equal compiled-in constant `FORGE_SCHEMA_HASH`.

If any check fails → error `ForgeError::RefuseToOpenDb { reason }`. Never silently proceed.

**Enforcement point:** `invariants::refuse_to_open_db(path)` called from `store::db::ForgeStore::open()`

## I2 — No semantic-memory DB discovery
Default config must never attempt to open `memory.db` or any other DB path directly.
The semantic-memory handle is injected by the caller. Forge never discovers or opens it.

**Enforcement point:** `ForgeStore::open()` only receives the forge.db path. Any other path is rejected.

## I3 — Forbidden paths (default deny)
Patch validation MUST reject edits to these globs by default:

```
tests/**
**/*_test.rs
**/fixtures/**
**/*.snap
Cargo.lock
.github/**
```

Config can loosen `.github/**` only. Tests, fixtures, snapshots, and Cargo.lock are
hardcoded denies regardless of config when `allow_test_modifications = false`.

**Enforcement point:** `patch::validate::validate_forbidden_paths(patch, config)`
Called before any patch apply. No exceptions.

## I4 — Patch caps (default)
Patch validation MUST reject patches exceeding:
- `max_files_changed`: 8
- `max_total_lines_changed`: 400
- `max_lines_changed_per_file`: 200

These are configurable but must have the above defaults. Exceeding caps → reject before apply.

**Enforcement point:** `patch::validate::validate_patch_caps(patch, config)`

## I5 — No network in sealed mode
When `config.mode == sealed_local`:
- Container backend MUST pass `--network=none` (Docker/Podman) or `--net=none` (nerdctl).
- If the detected runtime does not support a no-network flag → `ContainerBackend` must
  refuse to run in sealed mode (error, not warning).
- `ModelRouter` must refuse remote endpoints.
- `HostBackend` is allowed in sealed mode only if `config.sealed_allow_host_backend = true`
  (default false; this is a footgun and should warn loudly).

**Enforcement point:** `exec::container::ContainerBackend::run_sealed()`

## I6 — Stabilization attempt order
The attempt sequence is FIXED:
1. innovative (Δ = delta_amp_default)
2. stabilize1 (Δ = delta_amp_stabilize1, force_family applied)
3. stabilize2 (Δ = delta_amp_stabilize2, force_minimal_diff applied)
4. clamp (Δ = 0.0)

No skipping. No reordering. No early exit to a different phase on success
(success exits the loop, not the phase sequence).

**Enforcement point:** `runtime::stabilizer::Stabilizer::next_attempt()` — panics in debug
builds if called out of order; returns error in release builds.

## I7 — Deterministic MindState rendering
Given the same inputs (request, evidence, repo context, BasisVersion, config overrides),
`ForgeRuntime::compile_mindstate()` must return byte-identical output.

Requirements:
- No random seeds in rendering path.
- All collections sorted by stable key before serialization.
- Timestamps excluded from rendered MindState (but present in traces).

**Enforcement point:** Snapshot tests in `tests/mindstate_determinism.rs`

## I8 — No silent test edits to pass evaluation
In Lab mode, tasks default to `allow_test_modifications = false`.
A task must explicitly set `constraints.allow_test_modifications = true` in `task.json`
to permit test edits for that task. There is no global Lab override.

**Enforcement point:** `lab::evaluate::EvaluationRunner::run()` reads per-task constraints
and passes them to `patch::validate` — task-level config takes precedence within its scope only.

## I9 — CEA stores no raw source [PROPRIETARY]
CEA nodes (both `EditOpSignature` and `EffectSignature`) must never store raw source code,
file paths that expose the codebase structure beyond hashes, or any content that could
reconstruct the original source.

Allowed in CEA nodes:
- Hashes (blake3) of context lines
- Structural features (op_kind, anchor_kind, line counts, scope_tag, op_index, file_index)
- Lint names (e.g., `clippy::needless_return`) — these are public identifiers
- Test function names — these are compiled into binaries; not secret

Not allowed:
- Raw `context_before` or `context_after` strings
- File paths beyond extension (`.rs`)
- Variable/function/type names

**Enforcement point:** `invariants::validate_cea_no_raw_source(node)` called in
`cea::instrumentation::CausalInstrument::extract_signatures()`

## I10 — CEA graph updates are idempotent and observational-only
An identified execution has a content-bound `run_hash` and an identity-bound
`observation_key`. The same execution identity must not update the graph more than once,
even if a retry presents conflicting content. Distinct trial identities remain independent.
Only `EvidenceKind::Observational` may enter the association edge store; paired,
ablation, counterfactual, and synthetic telemetry evidence remain receipt-only.

**Enforcement point:** `cea_store::update_graph()` checks the idempotency key in
`cea_run_log` before mutation and rejects non-observational evidence before opening a write
transaction.
