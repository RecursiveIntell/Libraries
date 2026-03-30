# Implementation plan

## Phase 0 — restore truth on disk

1. Apply this root pack to the repo root.
2. Regenerate `release/closeout_receipt_v1.json`.
3. Run the static gates that do not require cargo.
4. Confirm that the active front door now exists and matches the archive manifest.

## Phase 1 — ship the narrated demonstrator

1. Create `docs/demos/effect_authority_assurance_release.md`.
2. Add one stitched fixture under `contracts/fixtures/demo/` that references the existing v21, v22, and v23 happy-path artifacts.
3. Add one validating test or script in `verification-control` or an equivalent orchestration-neutral location.
4. Link the demo from `README.md` and `11_BENCHMARK_PLAN.md`.

## Phase 2 — ship the benchmark / forge-bench proof package

1. Freeze the benchmark questions and baselines.
2. Publish fixture sets for temporal correctness, replayability, widening disclosure, and contradiction handling.
3. Emit one score sheet and reproducibility note.

## Phase 3 — finish the archive reduction

1. Move or delete the remaining stale root residue.
2. Tighten `docs/archive/root_closeout_history/manifest.json`.
3. Regenerate the closeout receipt so the residual debt list shrinks.
