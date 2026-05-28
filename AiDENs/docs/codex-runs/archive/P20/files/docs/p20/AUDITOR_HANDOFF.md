# Auditor Handoff Requirements

A hostile auditor should be able to verify the run without trusting Codex's narrative.

## Auditor command sequence

From repo root:

```bash
bash scripts/phase_verify_contract_ownership.sh final
python3 scripts/make_type_ownership_inventory.py
python3 scripts/assert_no_canonical_type_duplicates.py
bash scripts/assert_no_local_canonical_digest_law.sh
python3 scripts/assert_schema_generation_scope.py
bash scripts/assert_tool_runtime_delegation.sh
bash scripts/assert_no_crate_split.sh
bash scripts/assert_no_compatibility_ledgers.sh
git status --short
git diff --stat
```

If available:

```bash
cargo check --workspace
cargo test --workspace
```

## Auditor questions

1. Does `aidens-contracts` still locally define any exact canonical type names?
2. Are any canonical concepts preserved through aliases, shims, or compat modules?
3. Are digest/schema/identity semantics owned outside canonical crates?
4. Do display wrappers contain canonical backpointers?
5. Are all ambiguous owner decisions quarantined?
6. Do docs match the actual source basis?
7. Did any phase proceed after a failed gate?

## Required final auditor file

Codex must write:

```text
docs/contract-ownership/FINAL_AUDITOR_HANDOFF.md
```

It must include command outputs or file paths to outputs, not just claims.
