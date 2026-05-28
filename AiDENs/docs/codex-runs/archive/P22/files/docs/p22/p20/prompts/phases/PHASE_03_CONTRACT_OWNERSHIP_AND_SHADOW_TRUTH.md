# Phase 03 — Contract Ownership and Shadow-Truth Collapse

## Objective

Prevent `aidens-contracts` and local DTOs from becoming canonical truth.

## Required actions

- Inventory every public type in `crates/aidens-contracts/src/lib.rs`.
- Classify each type.
- Redirect, rename, remove, or quarantine duplicates/ambiguous types.
- Create `docs/p20/CONTRACT_OWNERSHIP_INVENTORY.md` and `.json`.

## Acceptance gate

No unaddressed `duplicate_canonical_concept` or `ambiguous_shadow_semantics` remains.
