# Target Repository Tree

```text
crates/
  scr-reference/
  scr-kernel/
  scr-audit-adapter/
  scr-cli/

schemas/generated/

policies/
  audit_policy_v1.toml
  audit_policy_v1.canonical.json

fixtures/audit/
  cases/
  expected/

scripts/
  run_all_checks.sh
  generate_schemas.sh
  validate_schemas.py
  verify_golden_fixtures.sh
  assert_no_unexplained_golden_changes.sh
  assert_no_feut_contamination.sh
  assert_no_durable_float_scores.sh
  assert_no_naked_decision_booleans.sh
  assert_no_shadow_truth.sh
  assert_no_llm_or_network_calls.sh

docs/
  SOURCE_BASIS.md
  CANONICAL_OWNERS.md
  QUARANTINED_TERMS.md
  ARCHITECTURE.md
  EVALUATOR_REFERENCE.md
  POLICY_MODEL.md
  ACTION_RESOLUTION.md
  DECISION_RECEIPTS.md
  AUDIT_ADAPTER.md
  FAILURE_MODES.md
  INTEGRATION_SEQUENCE.md
  NON_GOALS.md
```
