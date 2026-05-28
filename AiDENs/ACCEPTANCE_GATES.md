# Acceptance Gates

## Global gates

Run from the AiDENs repo root unless noted.

```bash
cargo metadata --format-version 1
bash scripts/assert_stack_paths.sh
bash scripts/assert_no_shadow_truth.sh
bash scripts/assert_docs_match_cargo.sh
bash scripts/assert_adapter_delegation.sh
bash scripts/assert_compat_is_finite.sh
```

If `cargo` is unavailable, report that build certification cannot be performed and run all shell/static gates that are available.

## Absolute gates

- No dependency may resolve to any `Libraries2` stack-ids path, overlay, or scaffold.
- No banned local canonical type may remain outside canonical crates or generated docs.
- No retained compatibility shim or compatibility ledger row is allowed.
- Golden vertical slice must pass before daemon/scheduler/kernel expansion.
- Tests for each phase must pass before moving to the next phase.
- Docs must match Cargo dependency reality.

## Phase gates

| Phase | Required gates |
|---|---|
| 00 | source paths verified; owner map current; scripts installed; `assert_stack_paths.sh` and `assert_docs_match_cargo.sh` pass |
| 01 | `contract_owner_proof`; `canonical_id_roundtrip`; `assert_no_shadow_truth.sh` passes; compatibility ledger has no rows |
| 02 | `stack_import_smoke`; `adapter_delegation_proof` |
| 03 | `golden_vertical_slice` |
| 04 | `malformed_tool_call_degrades`; `denied_tool_requires_approval`; `budget_exhaustion_receipt`; `provider_route_unavailable` |
| 05 | `bitemporal_asof_query`; `import_atomicity`; `query_widening_disclosure` |
| 06 | `promotion_denies_without_verification`; `approval_required_for_side_effect`; `repair_record_backpointer` |
| 07 | `daemon_namespace_isolated`; `schedule_no_duplicate_storm`; `restart_does_not_reenqueue_completed_jobs` |
| 08 | `kernel_exact_small_slice`; `loopy_nonconvergence_degrades` |
| 09 | `release_truth_audit`; full workspace tests/clippy if available |
