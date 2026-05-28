# Phase 06 — Runner Vertical Slice Proof

## Objective

Prove one complete runner flow.

## Required path

```text
config -> runner -> provider/mock or ollama -> tool exposure -> permit check -> tool call parse/repair -> tool execution -> final response -> event log -> receipts/control records -> audit report
```

## Acceptance gate

Fixture and test prove the path; receipts are emitted; no bypass mocks that skip real runner logic.
