# P32 acceptance gates

## Required final commands

```bash
cargo fmt --all -- --check
cargo check --workspace --all-targets
cargo test --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
python3 scripts/validate_strict_schemas.py
python3 scripts/validate_schema_rust_parity.py
python3 scripts/scr_superpass_preflight.py final
python3 scripts/scr_superpass_static_gates.py final
bash scripts/assert_no_opaque_signal_scanning.sh
bash scripts/assert_no_feut_contamination.sh
bash scripts/assert_no_llm_or_network_calls.sh
bash scripts/assert_no_durable_float_scores.sh
bash scripts/assert_no_naked_decision_booleans.sh
bash scripts/assert_no_shadow_truth.sh
bash scripts/assert_no_unexplained_golden_changes.sh
bash scripts/scr_superpass_run_all.sh final
```

## Semantic gates

- Proposed action/effect must materially affect decision.
- Material mutation/release cannot proceed with missing authority.
- Release cannot proceed with insufficient evidence.
- Destructive/apply effects require rollback/containment basis.
- Unknown owner mutation must quarantine or require repair packet.
- Opaque refs cannot be parsed as control signal truth.
- Candidate trace must preserve all candidates.
- Raw input digest and typed canonical digest must be distinct.
- Policy digest canonicalization must be documented and tested.
- External crate integration claims must be compiled/tested or removed.

## Final documentation gates

Required files:

```text
docs/P32_COMPLETION_REPORT.md
docs/P32_COMMAND_RECEIPTS.md
docs/P32_CHANGED_FILES.md
docs/P32_UNRESOLVED_RISKS.md
docs/P32_HOSTILE_AUDITOR_HANDOFF.md
docs/P32_POLICY_CHANGE_RECEIPT.md
docs/P32_ROLLBACK_PLAN.md
docs/SCR_CANONICAL_JSON_V1.md
docs/SCR_ADAPTER_SEAMS.md
docs/SCR_ACTION_SEMANTICS.md
docs/SCHEMA_RUST_PARITY.md
docs/EVALUATOR_BUILD_DIGEST.md
```

No final response may claim completion unless these exist or their absence is explicitly justified as a blocker.
