# P23 Capability Build Spec

P23 must move AiDENs toward a product, not only a clean package.

## Minimum capability slice

Deliver one tested local agent lane:

- build/plan an agent from config/fixture,
- run it through mock/local-safe provider and tool flow,
- emit receipt-bearing run directory,
- expose CLI/operator inspection,
- classify support tier honestly.

## Run bundle contract

The exact type names may vary, but the output must include equivalent fields:

```json
{
  "run_id": "...",
  "profile": "coding-agent|test-agent|...",
  "support_tier": "fixture-supported|supported|partial|...",
  "created_at": "...",
  "execution_context": {
    "attempt_family": "...",
    "attempt_id": "...",
    "provider_route": "...",
    "tool_route": "...",
    "budget": { "max_steps": 0, "elapsed_ms": 0 },
    "degradation": [],
    "environment_fingerprint": "..."
  },
  "receipts": ["..."],
  "outputs": ["..."],
  "replay": { "command": "..." },
  "blocked_checks": []
}
```

## Non-negotiable truth boundary

This is an AiDENs operator/run report, not canonical stack truth. If canonical crates expose receipt/evidence types, reference/delegate to them. If not, label the AiDENs DTO as operator evidence and do not promote it.

## Stretch capability after minimum slice

Only after all gates are green, Codex may add one of:

1. `aidens agent doctor` with support-tier decomposition,
2. `aidens run inspect` for receipt directories,
3. `aidens package doctor` to explain package role inclusion/exclusion,
4. local coding-agent template generation with permits and receipts,
5. integration test that round-trips config → plan → run → receipts → inspect.
