# TEST_PLAN.md
# Test Plan — Required Tests

All tests must pass before a PR is mergeable.
Tests that are flaky by nature (container availability) must use feature flags or skip gracefully.

---

## A. Safety and Isolation tests (required; no skipping)

**A1: `refuse_to_open_non_forge_db`**
- Create a temp SQLite file with no tables.
- Call `ForgeStore::open(path)` → must return `Err(ForgeError::RefuseToOpenDb { .. })`.

**A2: `refuse_to_open_missing_forge_meta`**
- Create a temp SQLite file with a `users` table but no `forge_meta`.
- Must refuse.

**A3: `refuse_to_open_wrong_schema_hash`**
- Create a valid forge DB but manually set `forge_meta.schema_hash = 'wrong'`.
- Must refuse.

**A4: `refuse_to_open_semantic_memory_signature`**
- Create a DB that looks like semantic-memory (has `memory_entries` table, no `forge_meta`).
- Must refuse with `RefuseToOpenDb`.

**A5: `forge_writes_only_to_forge_db`**
- Run a full eval cycle in a temp directory.
- Assert no files were created or modified outside `forge.db` and the temp workspace.
- Specifically: no `memory.db` created anywhere.

---

## B. MindState determinism tests (required)

**B1: `mindstate_render_is_deterministic`**
- Construct fixed `MindState` inputs (mocked evidence, fixed request, fixed BasisVersion).
- Render twice; assert byte-identical output.

**B2: `mindstate_snapshot`**
- Render a MindState with fixed inputs.
- Assert matches stored snapshot file.
- Snapshot lives in `tests/snapshots/mindstate_v1.snap`.

---

## C. StructuredPatch validation tests (required)

**C1: `patch_rejects_forbidden_path_tests`**
- Patch with edit to `tests/my_test.rs` → must fail with `ForbiddenPath` violation.

**C2: `patch_rejects_forbidden_path_snap`**
- Patch with edit to `src/snapshots/foo.snap` → must fail.

**C3: `patch_rejects_cap_files`**
- Patch with 9 FileEdits (> max 8) → must fail with `CapExceeded` violation.

**C4: `patch_rejects_cap_total_lines`**
- Patch with 401 total lines changed → must fail.

**C5: `patch_accepts_well_formed`**
- Patch with 2 files, 50 total lines, no forbidden paths → must pass validation.

**C6: `patch_validation_returns_all_violations`**
- Patch with 3 violations → must return all 3 (not fail-fast).

---

## D. StructuredPatch apply tests (required)

**D1: `apply_insert_after_line`**
- Simple insert after line 3 with matching context.
- Assert resulting file content.

**D2: `apply_replace_range`**
- Replace lines 5-8 with new content.
- Assert resulting content.

**D3: `apply_delete_range`**
- Delete lines 2-4.
- Assert resulting content.

**D4: `apply_match_anchor`**
- Insert after 2nd occurrence of `fn compute`.
- Assert correct insertion point.

**D5: `apply_fails_on_ambiguous_context`**
- Context mismatch → must return `Err`; workspace unchanged.

**D6: `apply_is_atomic`**
- Multi-op patch where op 2 fails → workspace must be restored to pre-apply state.

**D7: `apply_returns_line_attribution_map`**
- After apply, `LineAttributionMap` must correctly map original lines to patched lines.

---

## E. Diff rendering tests (required)

**E1: `diff_render_produces_valid_unified_format`**
- Apply a patch, render diff.
- Assert output starts with `---` / `+++` and has `@@` hunks.

**E2: `diff_render_is_stable`**
- Same input twice → identical diff string.

**E3: `diff_apply_via_git_apply` (skip if git unavailable)**
- Render a diff, then apply it with `git apply --check` to a fresh copy of original.
- Must succeed (exit 0).

**E4: `diff_fallback_renders_correctly`**
- Mock git as unavailable; ensure internal diff renders and is parseable.

---

## F. Execution backend tests (required)

**F1: `host_backend_runs_command_captures_output`**
- Run `echo hello` via HostBackend.
- Assert stdout == "hello\n".

**F2: `host_backend_timeout`**
- Run `sleep 60` with 1-second timeout.
- Must return `Err(ForgeError::CommandTimeout)`.

**F3: `container_backend_autodetect_docker`** (skip if Docker unavailable)
- Mock PATH to have only `docker`.
- Assert `ContainerBackend::detect()` returns `ContainerRuntime::Docker`.

**F4: `container_backend_autodetect_fallback_host`**
- Mock PATH to have no container runtimes.
- Assert `ExecutionBackend::select(auto)` returns `HostBackend`.

**F5: `container_backend_sealed_requires_no_network`**
- Construct sealed config, ContainerBackend.
- Inspect generated command args; assert `--network=none` or `--net=none` present.

---

## G. CargoAdapter tests (required)

**G1: `cargo_adapter_detects_cargo_project`**
- Workspace with `Cargo.toml` → `CargoAdapter::detect()` returns true.
- Workspace without → returns false.

**G2: `cargo_adapter_runs_fmt_on_tiny_fixture`**
- Provide a tiny valid Rust fixture (inline in test).
- Run fmt check → exit 0 (fixture is pre-formatted).

**G3: `cargo_adapter_parses_clippy_json`**
- Mock clippy JSON output with known lint.
- Assert `ParsedCheckOutput.effects` contains the expected `EffectSignature`.

---

## H. Lab pipeline tests (required)

**H1: `evaluate_one_candidate_one_task_mocked_patch`**
- Provide mocked `StructuredPatch` (no generator needed).
- Run full evaluation pipeline on tiny fixture.
- Assert `EvalRunResult` has valid scores and is persisted in `eval_runs`.

**H2: `archive_insert_replaces_lower_score`**
- Insert candidate A with score 0.7 into cell.
- Insert candidate B with score 0.9 into same cell.
- Assert cell now holds B.

**H3: `archive_insert_below_gate_discarded`**
- Insert candidate with correctness 0.80 (< 0.95 gate).
- Assert cell unchanged.

**H4: `archive_preserves_higher_score`**
- Insert candidate A (score 0.9), then B (score 0.8) into same cell.
- Assert cell still holds A.

---

## I. CEA tests (required if CEA enabled)

**I1: `cea_instrumentation_extracts_effect_signatures`**
- Provide mocked clippy JSON with known lint.
- Assert extracted `EffectSignature` matches expected.

**I2: `cea_edit_op_signature_no_raw_source`**
- Construct `EditOpSignature` from a FileEdit with known context.
- Assert `sig_json` does not contain any raw context lines.
- Assert `context_hash` is a 64-char hex string (blake3).

**I3: `cea_graph_update_is_idempotent`**
- Run `update_graph(result)` twice with same `AttributedRunResult`.
- Assert `cea_edges` weights unchanged after second call (run_hash deduplicated).

**I4: `cea_prediction_returns_neutral_for_unknown_sigs`**
- Predict on a patch with no matching graph edges.
- Assert `coverage_fraction == 0.0`, `confidence` is low, `zero_shot_eligible == false`.

**I5: `cea_zero_shot_requires_explicit_enable`**
- Default config: `enable_zero_shot = false`.
- Even with `zero_shot_eligible = true`, runtime must NOT skip checks.
- Enable it: runtime skips checks and uses predicted score.

---

## CI gates (all must be green for merge)
```yaml
- cargo test -p semantic-memory         # must pass; output must not change
- cargo test -p forge-engine
- cargo clippy -p forge-engine -- -D warnings
- cargo fmt --all -- --check
- diff check: no files in crates/semantic-memory/ modified
```
