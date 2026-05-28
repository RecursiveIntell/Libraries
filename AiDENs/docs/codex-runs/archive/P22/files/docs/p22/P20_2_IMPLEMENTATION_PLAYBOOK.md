# P20.2 Implementation Playbook

## Principle

Do not optimize for apparent progress. Optimize for verifiable release state.

## Phase sequence

1. **Phase 00 — source truth preflight**: scan actual files and current Cargo topology.
2. **Phase 01 — package integrity closure**: restore missing evals/fixtures/scripts and enforce include target checks.
3. **Phase 02 — test topology split**: make `aidens-testkit` pure and create/move production tests into `aidens-integration-tests`.
4. **Phase 03 — build stabilization**: fix compile/test/clippy errors without weakening invariants.
5. **Phase 04 — canonical test agent**: create deterministic end-to-end agent proof.
6. **Phase 05 — provider/tool/permit/receipt proof**: verify tool-loop boundary honesty.
7. **Phase 06 — agency eval expansion**: harden influence governance and receipt assertions.
8. **Phase 07 — v0.1 operator usability**: examples and commands, not broad feature expansion.
9. **Phase 08 — scanners/conformance**: ensure regressions fail locally and in CI.
10. **Phase 09 — release certification**: final audit bundle + zip replay check.
11. **Phase 10 — guarded stretch**: only after all gates green.

## Repair rules

- Restore required files before deleting tests.
- If a test was wrong, rewrite it with a justification and keep equivalent coverage.
- If ownership is unclear, quarantine and report rather than invent semantics.
- If provider support is not executable, mark unavailable.
- If Cargo fails due sibling canonical crates, report exact dependency failure and do not fake pass.

## Testkit split target

`aidens-testkit` may depend on:

- `aidens-contracts`
- `serde`
- `serde_json`
- `chrono`
- `uuid`
- `thiserror`
- narrow helper crates that do not create production cycles

`aidens-testkit` must not depend on:

- `aidens-runner`
- `aidens-provider-kit`
- `aidens-tool-kit`
- `aidens-cli`
- `aidens-agency-kit`
- `aidens-boundary-kit`
- `aidens-memory-kit`
- `aidens-kernel-kit`
- `aidens-governance-kit`
- canonical sibling runtime crates

Production-dependent tests belong in `aidens-integration-tests`.

## Test agent target

Minimum vertical slice:

```text
agent config
→ provider route selection
→ mock provider emits tool call
→ tool exposure plan
→ permit check
→ boundary parse/repair if needed
→ tool execution
→ tool result
→ agency final-output gate
→ final response
→ receipt/event log assertions
```

## Final rule

If all tests pass but ownership or agency invariants are violated, the run fails.
