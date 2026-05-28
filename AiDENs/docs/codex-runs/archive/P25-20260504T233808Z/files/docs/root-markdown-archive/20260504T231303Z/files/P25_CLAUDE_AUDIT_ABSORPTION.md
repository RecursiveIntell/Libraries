# P25 Claude Audit Absorption

## Purpose

This file maps Claude/hard-audit concerns into concrete P25 issue IDs and acceptance gates. If an external Claude audit is present in the workspace, Codex must read it and update this table with direct references before coding.

## Absorption table

| Audit concern | P25 response | Issue IDs |
|---|---|---|
| Codex does not stop for phase injections | Machine-checkable phase-gate protocol and blocking injections after phases 01/03/05/07/09 | P25-003, P25-005 |
| z.py is being overworked | z.py scope contract: only root Markdown archive hygiene | P25-002 |
| Root workspace Markdown noise pollutes context | Safe root Markdown archive feature | P25-001 |
| prior-run stale docs/instructions create drift | Current-run classification and stale-ID verifier | P25-003, P25-004 |
| Fixture-level support may be oversold | Support profile convergence and known limitations | P25-007 |
| Need concrete demonstration of AiDENs value | Flagship supported-local coding-agent demo | P25-006 |
| Giant files create future fragility | Large-file containment plan, no risky P25 refactor | P25-008 |
| V10+ should not contaminate current pass | V10+ design-only track | P25-010 |

## Required action

Codex must not simply say "Claude audit absorbed." It must:
1. list the specific Claude audit files it read;
2. map each relevant finding to issue IDs;
3. state whether each item was fixed, deferred, or rejected with reason;
4. include unresolved Claude findings in the final auditor handoff.
