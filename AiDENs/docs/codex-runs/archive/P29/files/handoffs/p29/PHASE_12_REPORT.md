# P29 Phase 12 Report

## Phase

Phase 12 - v11A artifact/execution/operator contract finalization.

## Scope

Added a v11A evidence block to the declared supported-local `run-coding-agent` path. The block is local release-candidate evidence only for that path and does not widen cloud/autonomy claims.

## Files changed

- `crates/aidens-cli/src/lib.rs`
- `crates/aidens-cli/src/tests.rs`
- `handoffs/p29/PHASE_12_REPORT.md`

## Evidence added

- `coding-agent-report.json#/v11a_evidence/artifact_envelope`
- `coding-agent-report.json#/v11a_evidence/input_manifest`
- `coding-agent-report.json#/v11a_evidence/output_manifest`
- `coding-agent-report.json#/v11a_evidence/execution_context`
- `coding-agent-report.json#/v11a_evidence/operator_contract`
- `coding-agent-report.json#/v11a_evidence/operator_invocation_receipt`

## Tests/checks run

| Command | Result | Log |
|---|---|---|
| `cargo test -p aidens-contracts p28_material_done -- --nocapture` | pass | `target/p29/audit/phase12_aidens_contracts_material_done_tests.log` |
| `python3 scripts/assert_p29_v11a_contracts.py` | pass | `target/p29/audit/phase15_assert_p29_v11a_contracts_rerun.log` |
| `cargo check --workspace --all-targets` | pass | `target/p29/audit/phase14_aidens_cargo_check.log` |

## Claims changed

The declared supported-local coding-agent path now has v11A local release-candidate evidence. This is not a final package claim and not a broad v11A/cloud/autonomy claim.

## Risks / limitations

The evidence is emitted by the local CLI report path. Final package generation and extracted replay remain pending.

## Gate status

- [x] pass
- [ ] fail

## Next phase notes

Proceed to Phase 13 proof/degradation/semantic disclosure finalization.
