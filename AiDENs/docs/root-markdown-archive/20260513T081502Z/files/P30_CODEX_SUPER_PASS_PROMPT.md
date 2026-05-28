# P30 Codex Super-Pass Prompt — v11B-centered hostile audit absorption

## Objective

Execute the AiDENs P30 super-pass as a **v11B-centered runtime hardening pass**. Absorb the 2026-05-08 hostile audit as aggressively as possible while preserving source-of-truth boundaries and moving the stack from v11A scaffolding toward executable v11B regional/subtractive runtime behavior.

P30 succeeds only if it produces code, tests, scripts, manifests, and reports proving the hardening occurred. Documentation-only fixes do not count for code-path issues.

## Primary inputs

- `matrices/P30_HOSTILE_AUDIT_ABSORPTION_MATRIX.csv`
- `input_evidence/AIDENS_HOSTILE_AUDIT_20260508.md`
- `input_evidence/AIDENS_HOSTILE_AUDIT_ISSUES_20260508.json`
- `input_evidence/AiDENs-aidens-next-codex-context-20260508.report.md`
- `input_evidence/AiDENs-aidens-next-codex-context-20260508.manifest.json`
- v9/v11/end-state spec files in `input_evidence/`

## Strategic target

P30 should materially harden these surfaces, in this order:

1. v11A blockers that would make v11B unsafe: material operation receipts, boundary compiler honesty, proof/degradation honesty, deterministic artifact identity, execution context envelopes.
2. executable tool-call parsing and structured-output boundaries.
3. patch apply / rollback / command execution safety.
4. deterministic replay identity and material artifact ID policy.
5. execution evidence defaults and durable failure receipts.
6. verification semantics and proof/degradation honesty.
7. **v11B graph surface declarations and right-graph gates.**
8. **v11B region contracts, boundary messages/receipts, replay slices, and state snapshots.**
9. **v11B residual/syndrome/convergence reports with stop/damping/oscillation law.**
10. **v11B lawful subtraction: support cores, removal frontiers, invariant-preservation receipts, historical-loss budgets, and subtraction challenge paths.**
11. **v11B causal/interventional bundle law where any attribution or blame-like claim exists.**
12. schema/gate/root-doc package hygiene.
13. panic/dynamic JSON/silent degradation/lint suppression pattern debt.
14. final conformance and audit handoff evidence.

## Hard rule: issue absorption

For every row in `P30_HOSTILE_AUDIT_ABSORPTION_MATRIX.csv`:

- If `gate=must-fix`, fix it or produce a quarantine record explaining why it cannot be fixed in this pass.
- If `gate=must-fix-or-explicit-quarantine`, fix it unless it requires a larger architectural move; then quarantine with owner, blocker, and next-pass target.
- If `gate=fix-if-touched-or-quarantine-with-receipt`, fix broad pattern classes when touching nearby code; otherwise include them in the unresolved-risk ledger.

## v11B implementation rule

Do not implement v11B as inert declarations only. Every v11B surface added in this pass must have at least one of:

- production-path use;
- conformance fixture;
- reference interpreter/checker fixture;
- explicit release debt record with owner, blocker, and next pass.

## Do not do these things

- Do not claim packaging cleanliness proves build or semantic correctness.
- Do not claim v11B compliance merely because v11B structs exist.
- Do not use a storage graph as an inference graph without declaration and conformance coverage.
- Do not recurse without stop law and convergence/degradation report.
- Do not subtract/summarize/delete without support core, removal frontier, invariant-preservation receipt, and historical-loss budget where applicable.
- Do not represent contradictions as lower scores.
- Do not use permissive repair as an execution path for tool calls without strict receipts and blocking semantics.
- Do not drop malformed tool calls, serialization failures, rollback failures, parse repair facts, region boundary failures, convergence failures, or subtraction failures.
- Do not make process-local or random IDs material.
- Do not downgrade proof/degradation states to happy-path success.
- Do not add compatibility shims that reinterpret canonical stack semantics.
- Do not hide failures behind `unwrap_or_default`, `_ =`, broad `allow`, dynamic JSON blobs, or stale docs.

## End-of-run deliverables

Create or update:

- `handoffs/p30/FINAL_AUDITOR_HANDOFF.md`
- `handoffs/p30/KNOWN_LIMITATIONS.md`
- `handoffs/p30/UNRESOLVED_RISK_LEDGER.md`
- `handoffs/p30/ISSUE_ABSORPTION_REPORT.csv/json/md`
- `handoffs/p30/GATE_SUPERSESSION_MANIFEST.md/json`
- `handoffs/p30/V11B_RUNTIME_SPINE_REPORT.md`
- `handoffs/p30/V11B_CONFORMANCE_DEBT_LEDGER.md`
- `handoffs/p30/P30_COMMAND_LOG.md`
- `handoffs/p30/P30_INVARIANT_REVALIDATION.md`
- `handoffs/p30/P30_RELEASE_CLAIMS.md`

Every final claim must include supporting command output or receipt path.
