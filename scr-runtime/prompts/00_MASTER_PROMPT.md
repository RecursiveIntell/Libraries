# MASTER PROMPT — SCR Runtime P31 Completion / Boundary Hardening

You are Codex operating on the repository at:

```text
/home/sikmindz/Coding/Libraries/scr-runtime
```

This pass is **P31: SCR Runtime Completion / Boundary Hardening**.

## Mission

Take `scr-runtime` from a P0A reference-kernel state to a near-complete, certifiable, owner-boundary-safe SCR runtime handoff.

The pass must fix package truth, stale source surfaces, schema/contract weakness, policy/evaluator correctness, CLI/replay gaps, existing-crate awareness, and certification gaps.

## Non-negotiable source basis

Before implementation, inspect the live repo and the canonical library root:

```bash
pwd
find . -maxdepth 3 -type f | sort
find /home/sikmindz/Coding/Libraries -maxdepth 3 -name Cargo.toml -print | sort
cargo metadata --format-version 1 --no-deps > /tmp/scr-runtime-cargo-metadata.json
```

Then inspect likely owner crates in `~/Coding/Libraries` before creating or hardening local types. Do not assume memory/context is current. Current files outrank all prose.

## Existing known defects to address

Treat these as required work items unless source inspection proves them already fixed:

1. Archive/report/manifest mismatch risk. The uploaded report claimed `.codex`/`.agents` and 170 included files, while direct uploaded ZIP inspection showed 119 files and no `.codex`/`.agents`. Add actual ZIP manifest parity verification and fix the packager/report path.
2. `target_files/` duplicates active source and can overwrite working implementation. Delete, archive, or mark non-authoritative.
3. `manual_injections/` and README still describe manual phase injection even though the desired workflow is automated phase gating. Replace with automated phase gates or archive as legacy.
4. `docs/SOURCE_BASIS.md` is stale; it says no local Cargo workspace exists even though one exists.
5. Root `z.py` is ambiguous. Move/rename as owned packaging tool under `scripts/` or explicitly exclude/classify it.
6. `testtmp/` must not exist in a clean handoff.
7. Any SCR or non-SCR naming in active `.codex`, `.agents`, docs, prompts, reports, or tests must be removed or archived.
8. Wire schemas are weaker than Rust validation: unknown fields, schema version consts, score max, and negative schema tests need hardening.
9. Policy validation accepts unknown hard rules that the evaluator ignores. Unknown hard rules must fail.
10. Policy domain/algorithm/canonicalization compatibility must be enforced.
11. `SignalSet::from_input` must stop tokenizing opaque IDs/refs for control facts.
12. Invalid input must not be silently converted into normal decision semantics.
13. `safe_ref`/equivalent logic must not replace malformed refs with synthetic refs that erase the original failure.
14. `evaluator_algorithm_hash` currently hashes an ID string. Rename or replace with honest digest semantics.
15. `evaluate()` public API currently always returns `EvaluationUnavailable`; remove, rename, or make the explicit-policy path the only public path.
16. CLI generation and verification are conflated. Split commands: generate vs verify.
17. Add `explain-receipt` that explains an existing receipt without re-evaluation.
18. Add negative tests for boundary, policy, schema, signal, and receipt behavior.
19. Add owner-boundary map and automated no-invention checks for `~/Coding/Libraries`.
20. Final package must be certifiable from fresh unzip.

## Execution order

Run phases in order:

1. `prompts/01_PHASE_0_SOURCE_BASIS_AND_OWNER_SCAN.md`
2. `prompts/02_PHASE_1_PACKAGE_TRUTH_AND_SURFACE_CLEANUP.md`
3. `prompts/03_PHASE_2_SCHEMA_AND_BOUNDARY_CONTRACTS.md`
4. `prompts/04_PHASE_3_POLICY_AND_EVALUATOR_CORRECTNESS.md`
5. `prompts/05_PHASE_4_CLI_REPLAY_AND_FIXTURE_DISCIPLINE.md`
6. `prompts/06_PHASE_5_EXISTING_CRATE_ADAPTER_SEAMS.md`
7. `prompts/07_PHASE_6_TESTS_ASSERTIONS_AND_NEGATIVE_FIXTURES.md`
8. `prompts/08_PHASE_7_CERTIFIER_AND_FRESH_UNZIP_HARDENING.md`
9. `prompts/09_PHASE_8_FINAL_REPORT_AND_HOSTILE_HANDOFF.md`

After each phase, execute the matching auto-gate in `auto_gates/`. The auto-gate is mandatory. Do not continue if it fails; either repair or record an explicit blocker.

## Required scripts to add or update

At minimum, create/update:

```text
scripts/verify_archive_manifest_parity.py
scripts/assert_required_archive_paths.py
scripts/assert_no_stale_surfaces.py
scripts/assert_existing_crate_boundaries.py
scripts/validate_strict_schemas.py
scripts/assert_no_opaque_signal_scanning.sh
scripts/run_p31_completion_checks.sh
```

These scripts are included in this bundle under `scripts/` as reference implementations. Use them directly or improve them without weakening their checks.

## Completion standard

This pass is complete only when all are true:

```bash
cargo fmt --all -- --check
cargo check --workspace --all-targets
cargo test --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
python3 scripts/validate_strict_schemas.py
python3 scripts/assert_existing_crate_boundaries.py
python3 scripts/assert_no_stale_surfaces.py
bash scripts/assert_no_opaque_signal_scanning.sh
bash scripts/run_p31_completion_checks.sh
```

If a package zip is produced:

```bash
python3 scripts/verify_archive_manifest_parity.py <zip> <manifest.json>
python3 scripts/assert_required_archive_paths.py <zip>
```

No completion claim without command output.
