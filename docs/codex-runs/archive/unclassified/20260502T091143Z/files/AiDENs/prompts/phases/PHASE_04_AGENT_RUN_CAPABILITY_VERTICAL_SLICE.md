# Phase 04 — Agent Run Capability Vertical Slice

## Goal
Build actual AiDENs capability: a deterministic local agent/test-agent/coding-agent run path that emits receipts.

## Required capability

Implement or harden a product-facing local flow such as:

```bash
aidens agent run --config examples/configs/coding-agent.toml --fixture fixtures/test-agent/basic-agent.toml --out target/p23/runs/demo
```

Exact command may differ, but the flow must be discoverable through CLI help and operator docs.

## Required output bundle

The run must emit a deterministic run directory containing:

- `run_manifest.json`
- `execution_receipts.ndjson` or canonical receipt equivalent
- `provider_route.json`
- `tool_route.json`
- `permit_decisions.json`
- `budget_report.json`
- `degradation_report.json` when applicable
- `final_output.md` or equivalent
- `support_tier.json`

If sibling crates already own any receipt format, use or reference them. Do not invent canonical truth.

## Required tests

- fixture test proving command/run function completes,
- test proving receipt files exist and contain no secrets,
- test proving unsupported providers degrade explicitly,
- test proving cloud/native paths are not silently promoted.

## Acceptance gate

This phase must ship visible capability, not just docs. If the CLI command is deferred, a library API test-agent run must exist and be documented with exact invocation.
