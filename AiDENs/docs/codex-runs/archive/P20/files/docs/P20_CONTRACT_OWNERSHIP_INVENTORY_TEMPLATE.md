# Contract Ownership Inventory Template

Use this file to produce `docs/p20/CONTRACT_OWNERSHIP_INVENTORY.md`.

## Summary

| Count | Value |
|---|---:|
| Public types inventoried | TBD |
| Canonical re-exports | TBD |
| AiDENs orchestration DTOs | TBD |
| Display/report projections | TBD |
| Compatibility adapters | TBD |
| Duplicate canonical concepts | 0 required |
| Ambiguous shadow semantics | 0 required |

## Type inventory

| Type | Kind | File | Classification | Canonical owner | AiDENs role | Action | Proof |
|---|---|---|---|---|---|---|---|
| `ExampleV1` | struct | `crates/aidens-contracts/src/lib.rs` | `aidens_orchestration_dto` | n/a | runner report envelope | keep | test name |

## Classification rules

### `canonical_reexport`

The type is imported from and owned by a canonical sibling crate.

### `aidens_orchestration_dto`

The type is local but only controls AiDENs product flow, UI, CLI, runner, or report assembly.

### `display_or_report_projection`

The type is a non-authoritative projection for display/audit/reporting.

### `compatibility_legacy_adapter`

The type exists only to preserve old inputs and must map into canonical/newer surfaces.

### `duplicate_canonical_concept`

Blocking. The type duplicates evidence, memory, kernel, repair, verification, execution, or control truth that belongs to a canonical crate.

### `ambiguous_shadow_semantics`

Blocking. The type’s name/fields make it unclear whether it is authoritative.

## Required closeout

No blocking classifications may remain by the final audit.
