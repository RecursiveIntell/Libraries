# scr-runtime P32-SCR-RUNTIME-SUPERPASS completion bundle

## Purpose

This bundle is a complete Codex super-pass package for finishing `scr-runtime` from the current uploaded snapshot state.

The target is not cosmetic cleanup. The target is a real SCR-P0A completion pass with:

- verified Rust build/test/clippy receipts
- real phase gates instead of inert prompt/gate stubs
- true proposed-action semantics, not only fixture signal resolution
- honest authority/evidence/rollback/owner-boundary basis semantics
- strict schema/Rust parity
- full candidate arbitration trace
- raw input digest + typed input digest discipline
- honest evaluator/build digest
- canonical JSON policy/digest contract
- explicit external-crate adapter boundary
- final hostile-auditor handoff with command receipts

## Run ID

Use:

```text
P32-SCR-RUNTIME-SUPERPASS
```

Do not reuse or silently complete P31. The uploaded tree contains P30/P31 drift and missing P31 final artifacts. This pass should supersede the incomplete state instead of pretending it was already completed.

## How to use

1. Copy this bundle into or next to:

```text
~/Coding/Libraries/scr-runtime
```

2. From the repo root, read:

```text
01_MASTER_PROMPT.md
phase_prompts/
manual_injections/
acceptance/
```

3. Optional but recommended: install the overlay files from `codex_overlay/` into the repo after reviewing them.

4. Start Codex in `/plan` mode first, then paste `01_MASTER_PROMPT.md`.

5. Between phases, paste the matching manual injection from `manual_injections/`.

6. Do not accept completion unless `bash scripts/scr_superpass_run_all.sh final` passes and the final artifact set exists.

## Bundle layout

```text
README_USE_THIS_BUNDLE.md
01_MASTER_PROMPT.md
02_SUPERPASS_SCOPE.md
03_SOURCE_OF_TRUTH_MAP.md
04_IMPLEMENTATION_TARGETS.md
05_ACCEPTANCE_SUMMARY.md
phase_prompts/
manual_injections/
acceptance/
codex_overlay/
scripts/
templates/
subagents/
```

## Critical non-negotiables

- Current repo files beat memory and prose.
- Package-certifier success is not SCR runtime completion.
- No final completion claim without command receipts.
- No opaque-ref token scanning for control truth.
- No external crate integration claim unless compiled/tested against those crates.
- If owner-crate integration cannot be proven in this pass, mark SCR as standalone reference kernel with explicit adapter seams.
