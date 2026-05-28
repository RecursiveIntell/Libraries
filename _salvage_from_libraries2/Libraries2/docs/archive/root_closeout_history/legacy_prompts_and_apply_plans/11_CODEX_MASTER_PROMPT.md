# Codex master prompt

Paste the block below into Codex.

```text
You are finishing the closeout/hardening pass for a Rust library workspace.

This is not a planning-only task.
Your job is to read the closeout bundle, inspect the repo, and then IMPLEMENT the cleanup in code.

## Read these files first, in order
- 00_START_HERE.md
- 01_EXECUTIVE_SUMMARY.md
- 02_SOURCE_BASIS.md
- 03_FINISHING_SCOPE.md
- 04_MASTER_ISSUE_MATRIX.md
- 05_IMPLEMENTATION_SEQUENCE.md
- 06_CRATE_SPLIT_PLAN.md
- 07_HARDENING_AND_GOVERNANCE_PLAN.md
- 08_EXACT_FILE_TOUCH_MAP.md
- 10_ACCEPTANCE_AND_COMMANDS.md

## Mission
Finish tightening, hardening, and closing the main libraries so the repo feels finished rather than “advanced but visibly mid-migration.”

## What is already fixed and must NOT regress
1. `agent-graph` public error cleanup: no `anyhow` in the public error surface, `Other(String)` stays, `kind()` stays.
2. CEA confidence hardening: confidence must remain conservative, coverage-aware, and sample-aware.
3. `forge-pilot` duplicate error alias cleanup: keep a single canonical public `PilotError` surface.

## What is NOT actually fixed yet
1. `forge-pilot/src/main_support/mod.rs` is still a monolith.
2. `knowledge-runtime/src/runtime/core.rs` is still a monolith.
3. production `unwrap` / `expect` / panic edges still remain.
4. deprecated allowances are still too broad in supported core.
5. time/UUID generation is too scattered.
6. release/workspace hygiene is still behind the code quality.
7. `libraries-source/` mirror discipline is weak and drifting.

## Critical truth rules
- Root workspace crates are truth.
- `libraries-source/` is a derivative mirror unless proven otherwise.
- Do not develop against the mirror as the authority source.
- If a local Libraries-style folder exists inside the workspace, inspect it and prefer local path/workspace wiring where relevant.
- In this repo, preserve and repair root workspace truth first, then sync the mirror after the root passes.

## Non-negotiable engineering rules
- Do not be lazy.
- Do not stop at high-level planning.
- Do not count shell-game refactors as completion.
- Do not move a 1600-line file into a subdirectory and call it split.
- Do not reopen the authority split.
- Do not invent a new architecture.
- Do not silently swap local crates for remote crates.
- Prefer correctness, debuggability, and maintainability over cleverness.

## Required work
1. Add root release/workspace hardening files and wire them into CI/Makefile:
   - `rust-toolchain.toml`
   - `clippy.toml`
   - `deny.toml`
   - `nextest.toml`
   - workspace package/lints cleanup
   - metadata / no-prod-panics / mirror / hotspot scripts
2. Perform the `forge-pilot` split for real.
3. Perform the `knowledge-runtime` runtime split for real.
4. Burn down known production panic/unwrap/expect sites.
5. Tighten `stack-ids` construction/validation and split the ID catalog.
6. Concentrate time/UUID generation into explicit seams in the hot runtime crates.
7. Split the biggest hotspot files in `semantic-memory`, `forge-engine`, and `profile-runtime` as far as possible in this pass.
8. Improve crate metadata/docs for the main crates.
9. Make mirror drift checks exact.
10. Run the acceptance lane and leave a clear status note.

## Required execution order
1. Inspect the workspace.
2. Inspect any local library folders inside the workspace.
3. Read the bundle files.
4. Compare the bundle’s file touch map with the current repo.
5. Start coding immediately.
6. Keep the repo coherent as you go.
7. Run formatting/lints/tests/gates.
8. Update docs/status.

## Specific anti-shell-game requirement
The following do NOT count as completion:
- `forge-pilot/src/main_support/mod.rs` staying large while other files are stubs
- `knowledge-runtime/src/runtime/core.rs` staying large while `runtime/mod.rs` is thin

The implementation must physically separate responsibilities.

## Acceptance bar
Use `10_ACCEPTANCE_AND_COMMANDS.md` as the finish line.
Do not declare completion until the repo passes the relevant commands and the structural proof checks make sense.

## Deliverables
By the end of the pass, the repo should contain:
- the code changes
- the new root config files
- improved scripts/CI
- real module splits
- updated docs/status notes
- explicit note of what remains, if anything

Begin now.
```
