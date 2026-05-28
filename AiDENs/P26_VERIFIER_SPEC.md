# P26 Verifier Spec

Create or extend `scripts/p26_verify.sh` and `scripts/p26_verify.py`.

## Required checks

1. `assert_phase_gate_integrity`
2. `assert_current_run_truth`
3. `assert_support_claims`
4. `assert_agent_spec_v1_schema`
5. `assert_agent_spec_v1_fixtures`
6. `assert_plan_act_verify_receipts`
7. `assert_memory_grounded_agent_lane`
8. `assert_coding_agent_v1_lane`
9. `assert_abstention_repair_cases`
10. `assert_run_bundle_v3_replay`
11. `assert_no_shadow_truth`
12. `assert_no_local_substitute_dependencies`
13. `assert_package_validation`
14. `assert_package_self_replay`

## Required command gates

- `cargo fmt --all -- --check`
- `cargo check --workspace`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo doc --workspace --no-deps`

## Evidence output

Write to `target/p26/audit/` and summarize in `P26_STATUS_EVIDENCE_MANIFEST.json`.
