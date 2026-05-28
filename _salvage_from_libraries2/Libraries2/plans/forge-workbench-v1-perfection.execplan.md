# Archived plan — wrong product surface for this checkout

This file previously described a **Forge Workbench** application pass with paths such as:

- `crates/forge-workbench-core/`
- `apps/forge-workbench/src-tauri/`
- `apps/forge-workbench/ui/`

Those paths do **not** exist in the current libraries workspace checkout.

## Status

**Archived for this checkout. Do not use as the active execution plan.**

The only Tauri app present here is `demo-tauri-libraries/`, which is a small demo and not the product surface described by the original Forge Workbench plan.

## Use instead

For the next Codex pass, use:

- `00_START_HERE.md`
- `plans/libraries-v16-v20-closeout.execplan.md`
- `docs/closeout_v16_v20/README.md`
- `prompts/codex_finish_handoff_prompt_v16_v20.txt`

## Rule

Do not invent missing Forge Workbench app/core/UI directories in this repo. Work against the actual libraries workspace only.
