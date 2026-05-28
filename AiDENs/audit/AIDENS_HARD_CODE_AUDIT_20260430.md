# AiDENs hard code audit — 2026-04-30

**Basis:** static inspection of `libraries-source-clean-20260430.zip` after extraction. This audit is intentionally code/package-first. Documentation was only considered where code, manifests, scripts, or release packaging directly reference it.

**Cargo status:** `cargo` and `rustc` were not available in this environment, so this is not a green-build certification. The next Codex run must execute the real cargo gates inside the full workspace with sibling crates present.

## Executive verdict

The architecture is now directionally correct: AiDENs is behaving like an orchestration/profile/runner layer over canonical libraries rather than a local shadow-stack. The remaining blockers are mostly mechanical but release-critical:

1. one Rust compile-time fixture target is missing;
2. the source manifest names five absent files;
3. `aidens-testkit` is still impure and depends on production crates;
4. ownership scanning can produce false confidence when canonical sibling crates are absent;
5. cargo gates still need to be run in the real workspace.

These are small/medium repairs with big effects. They are not reasons to redesign AiDENs.

## Hard blocker 1 — missing Rust include target

Rust `include_str!`/`include_bytes!` references found: **116**. Missing targets: **1**.

- `crates/aidens-testkit/tests/phase_09_reference_hostile_tests.rs` includes `../../../evals/p20_agency_eval_cases.jsonl` -> missing `evals/p20_agency_eval_cases.jsonl`

This is a code/package blocker because `include_str!` fails at compile time for test targets.

## Hard blocker 2 — manifest/package integrity

`MANIFEST.txt` exists, but these listed paths are missing:

- `evals/p20_agency_eval_cases.jsonl`
- `fixtures/runner/expected_event_log.ndjson`
- `supporting/matrices/forbidden_leftovers.csv`
- `supporting/matrices/phase_acceptance_gates.csv`
- `supporting/matrices/source_of_truth_matrix.csv`

At minimum, restore them or regenerate the manifest. Do not ship a manifest that lies.

## Hard blocker 3 — `aidens-testkit` topology

`aidens-testkit` currently has normal dependencies on production crates:

- `aidens-agency-kit`
- `aidens-boundary-kit`
- `aidens-budget-kit`
- `aidens-cli`
- `aidens-contracts`
- `aidens-daemon-kit`
- `aidens-governance-kit`
- `aidens-kernel-kit`
- `aidens-memory-kit`
- `aidens-permit-kit`
- `aidens-provider-kit`
- `aidens-receipts`
- `aidens-repair-kit`
- `aidens-runner`
- `aidens-tool-kit`

These production crates also dev-depend on `aidens-testkit`:

- `aidens-boundary-kit`
- `aidens-config`
- `aidens-permit-kit`
- `aidens-provider-kit`
- `aidens-receipts`
- `aidens-tool-kit`

This violates the reference-testkit concept. The next pass must split:

```text
aidens-testkit              # pure reference interpreters, fixtures, static check helpers
aidens-integration-tests    # may depend on production crates and run end-to-end paths
```

If the name `aidens-testkit` remains, it must become the pure crate. Production-integrating tests move to either root `tests/`, `crates/aidens-integration-tests`, or package-local integration tests that do not create dependency cycles.

## Hard blocker 4 — ownership scanner false confidence

`make_type_ownership_inventory.py` reported:

```text
canonical_types=0
aidens_contracts_types=185
duplicate_findings=0
```

That result is not authoritative if canonical sibling crates are absent or not scanned. The scanner must fail, warn loudly, or record `canonical_inventory_unavailable=true`; it must not imply “no duplicates” from an empty canonical baseline.

## Code size snapshot

- files: **1034**
- Rust files: **54**
- approximate Rust LOC: **32134**
- workspace crates: **32**

| crate | LOC | Rust files |
|---|---:|---:|
| aidens-contracts | 10181 | 1 |
| aidens-testkit | 4656 | 15 |
| aidens-cli | 4306 | 3 |
| aidens-tool-kit | 2483 | 2 |
| aidens-runner | 2130 | 4 |
| aidens-agency-kit | 1585 | 1 |
| aidens-provider-kit | 1414 | 1 |
| aidens-boundary-kit | 1056 | 1 |
| aidens-app-kit | 900 | 3 |
| aidens-queue-kit | 789 | 1 |
| aidens-config | 365 | 1 |
| aidens-permit-kit | 329 | 1 |
| aidens-receipts | 323 | 1 |
| aidens-governance-kit | 298 | 1 |
| aidens-daemon-kit | 245 | 1 |
| aidens-memory-kit | 182 | 1 |
| aidens-capability-kit | 135 | 1 |
| aidens-kernel-kit | 112 | 1 |
| aidens-security-kit | 101 | 1 |
| aidens-profile-coding | 84 | 1 |

## Positive code findings

- `aidens-runner` is real enough to preserve: it wires provider routing, tool exposure, permit checks, budget/deadline handling, boundary repair, tool dispatch, agency policy, control records, and canonical receipt persistence.
- `aidens-provider-kit` is much more honest than before: mock is executable, Ollama is chat-only, unavailable providers are explicitly unavailable, and native tool-loop claims are not globally enabled.
- `aidens-agency-kit` is substantive and runner-integrated; it is heuristic v0.1 policy, not just prompt text.
- `aidens-memory-kit`, `aidens-kernel-kit`, `aidens-governance-kit`, and `aidens-repair-kit` are correctly thin canonical adapters.
- `aidens-boundary-kit` has meaningful strict JSON, duplicate-key, markdown-fence/substring repair, repair receipt, and treatment-integrity behavior.

## Weak code surfaces to keep bounded

- `aidens-contracts` remains the largest local crate; it should stay DTO/report/display-only except for canonical re-exports.
- profile/plan crates remain scaffold-only and should not be promoted.
- `aidens-tool-kit` patching is a controlled simple patch path, not a mature general patch engine.
- agency policy is heuristic and must be labeled/evaluated accordingly.

## Next-pass target

Run **P20.1 CODE/PACKAGE REPAIR**, not a new architecture pass.

The next pass is complete only when:

```text
[ ] evals/p20_agency_eval_cases.jsonl exists and validates
[ ] all include_str/include_bytes targets exist
[ ] MANIFEST.txt has zero missing entries
[ ] aidens-testkit is pure or production integration tests are moved out
[ ] ownership scanner fails when canonical baseline is unavailable
[ ] cargo fmt/check/test/clippy pass in the real workspace
[ ] scripts/p20_1_verify.sh passes
[ ] final audit bundle records exact cargo command outputs
```
