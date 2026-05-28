# Ready-to-paste prompt for next major Codex pass

You are working on AiDENs + Rust stack integration.

## Source layout

```text
/workspace/aidens      # from aidens.zip
/workspace/libraries   # from libraries.zip
/workspace/libraries2  # from libraries2.zip, supplemental library pool
```

## Non-negotiable

Use `stack-ids` from `libraries/stack-ids`, not `Libraries2/stack-ids`.

## Objective

Convert AiDENs kit crates into facades/adapters over the real stack crates. Do not continue implementing duplicate local semantics when an actual stack crate owns the domain.

## Read first

1. `README.md`
2. `MASTER_INVENTORY.md`
3. `CANONICAL_SOURCE_OF_TRUTH.md`
4. `ANTI_DUPLICATION_REPORT.md`
5. `AIDENS_STACK_INTEGRATION_GAP.md`
6. `IMPORT_PATH_REWRITE_PLAN.md`
7. `NEXT_CODEX_TASK_MATRIX.md`

## Definition of done

- `aidens-contracts` depends on `libraries/stack-ids` and exposes canonical IDs or lossless wrappers.
- At least one compile/test path proves AiDENs imports real stack crates.
- Memory/evidence routes through semantic-memory-forge, forge-memory-bridge, semantic-memory, and knowledge-runtime where applicable.
- Execution receipts route through llm-tool-runtime / verification-control.
- Kernel kit uses recursive-kernel-core, constraint-compiler, kernel-execution, and kernel-oracles where applicable.
- Governance uses verification-* and authority-delegation surfaces.
- No dependency on `Libraries2/stack-ids`, overlays, or scaffolds remains.

## Do not do

- Do not restart AiDENs from scratch.
- Do not silently rename duplicate local types and call that integration.
- Do not create a new shadow truth/contract/policy crate.
