# P26 Rollback and Quarantine Plan

## Quarantine triggers

- Unknown canonical owner.
- AgentSpec cannot validate support policies.
- Package self-replay remains failing without precise cause.
- Any agent action succeeds without required receipt.
- Any write/check action runs without permit.
- Any cloud/provider path becomes implicitly supported.
- Any memory path creates AiDENs-local truth.
- Any phase gate is crossed without operator injection.

## Response

1. Stop.
2. Emit violation report.
3. Quarantine changed files or feature path.
4. Revalidate invariants.
5. Resume only after operator gate.

## Rollback rule

Prefer local rollback/quarantine over global rebuild. Preserve evidence and failure artifacts.
