# Phase 06 — Provenance Receipt and Execution Context Hardening

## Goal
Ensure capability outputs are receipt-bearing and execution-aware.

## Required actions

1. Define AiDENs-local `RunBundle` / `AgentRunReport` DTOs only as operator reports, not canonical truth.
2. Strongly reference canonical receipt/evidence concepts where available.
3. Include execution context fields:
   - run id,
   - attempt id/family,
   - provider route,
   - tool route,
   - permits/approvals,
   - budget/deadline,
   - degradation markers,
   - environment fingerprint,
   - replay command,
   - support tier.
4. Add explicit `unsupported_reason` / `blocked_checks` fields for partial/deferred paths.
5. Ensure receipts survive package/replay tests.

## Required tests

- receipt shape test,
- replay command presence test,
- degradation disclosure test,
- no shadow truth assertion.

## Acceptance gate

Every product-facing execution must either emit receipts or fail before claiming completion.
