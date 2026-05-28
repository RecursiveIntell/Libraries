# Codex Prompt — P04 Capability gate, permits, approvals, and side-effect denial law

Read `AGENTS.md`, `SOURCE_BASIS.md`, `BUILD_ORDER_DAG.md`, and `passes/P04_CAPABILITY_GATE_PERMITS_AND_APPROVAL_SPINE.md`.

Implement P04 only. Do not start later passes.

## Goal

Make tool declaration, registration, exposure, executability, invocation, permit, and approval distinct states with receipts.

## Primary crates

- `aidens-tool-kit`
- `aidens-permit-kit`
- `aidens-security-kit`
- `aidens-contracts`
- `aidens-cli`

## Required artifacts

- `CapabilityGateDecisionV1`
- `ToolExposurePlanV2`
- `ApprovalRequestV1`
- `ApprovalDecisionV1`
- `PermitGrantV1`
- `PermitUseReceiptV1`

## Acceptance gates

- list-tools and inspect-tools distinguish declared, registered, executable, exposed, hidden, blocked.
- Side-effect tools cannot be exposed or invoked without permit; denial includes ApprovalRequestV1 or reason code.
- Permits are scoped by risk, tool, sandbox root, time, and optional run/attempt family.

## Forbidden shortcuts

- Do not grant risky tools through profile defaults.
- Do not conflate hidden with blocked; operators need to know the difference.

## Finish by producing a handoff

Include files changed, tests added, commands run, blockers, and next-pass readiness.
