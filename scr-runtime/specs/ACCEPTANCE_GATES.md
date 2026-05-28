# Acceptance Gates

P0A is complete only if:

1. SCR evaluates proposed actions, not vague objects.
2. Kernel is deterministic and LLM-free.
3. Durable scores are integer/fixed-point.
4. Rust types generate schemas.
5. Policies canonicalize before hashing.
6. Hard rules run before scoring.
7. Rule/action conflict resolution is deterministic.
8. Hazard, confidence, uncertainty, authority, containment, and integrity are separate.
9. Evidence confidence cannot erase severity.
10. Low confidence routes high hazard to verification.
11. Integrity failure routes to quarantine.
12. Weak authority/containment restricts autonomy.
13. Every decision emits a replayable receipt.
14. Receipts include input hash, policy hash, algorithm ID, rule results, axes, pressures, chosen action, rejected actions, reason codes, and time basis.
15. Golden fixtures are deterministic and protected.
16. Seeded violations fail.
17. FEUT/EEG/P=NP/Clay language is quarantined.
18. No memory/retrieval/tool/Recall/AiDENs integration occurred.

Required commands:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
python3 scripts/validate_schemas.py
bash scripts/verify_golden_fixtures.sh
bash scripts/assert_no_feut_contamination.sh
bash scripts/assert_no_durable_float_scores.sh
bash scripts/assert_no_naked_decision_booleans.sh
bash scripts/assert_no_shadow_truth.sh
bash scripts/assert_no_llm_or_network_calls.sh
bash scripts/assert_no_unexplained_golden_changes.sh
bash scripts/run_all_checks.sh
```
