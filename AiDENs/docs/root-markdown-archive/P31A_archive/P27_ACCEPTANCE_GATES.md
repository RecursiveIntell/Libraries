# P27 Acceptance Gates

## Gate class A — hard blockers

P27 cannot claim success unless all are closed or explicitly quarantined with operator approval:

1. current verifier wrapper targets exist;
2. CI points to an existing verifier or equivalent check command;
3. active-run docs agree on P27;
4. package self-replay is green or honestly classified;
5. ownership scanner fails closed when canonical sibling baseline is absent;
6. support profile has no unsupported cloud/autonomy/V10 claims;
7. AGENTS.md is current and points to P27 docs.

## Gate class B — capability bar

P27 should close as many as possible:

1. mock-provider Plan→Act→Verify end-to-end path;
2. optional local Ollama smoke path with skip behavior;
3. durable run receipt store;
4. patch apply/check receipts with fail-closed ambiguity handling;
5. coding-agent run path with permit-gated writes and checks;
6. memory grounding evidence with canonical backpointers;
7. strict structured input validation for evidence-bearing JSON.

## Gate class C — structural bar

1. `aidens-contracts` module split or containment plan with at least one landed domain extraction;
2. `aidens-cli` module split or containment plan with at least one landed domain extraction;
3. root Markdown archive/labeling materially reduces drift;
4. scaffold profile crates removed, fenced, or downgraded.

## Gate class D — 11A semantic bar

Every evidence-bearing output touched by P27 must disclose:

- artifact identity or digest where available;
- support tier;
- exact/approx/degraded status;
- replay instructions or reason no replay exists;
- verification/check state;
- canonical owner/backpointer if it references sibling truth;
- known limitations.

## Final release bar

The final report may say `P27 passed` only when:

```text
scripts/verify_current.sh exists and succeeds or fails only on explicitly optional unavailable tools;
active docs agree;
package strict validation emits zero errors;
package replay status is recorded honestly;
support profile matches implementation;
final evidence manifest links all claims to files/commands;
final auditor handoff identifies unresolved issues.
```
