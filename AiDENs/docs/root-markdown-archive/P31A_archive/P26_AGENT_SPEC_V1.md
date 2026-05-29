# P26 AgentSpecV1 Specification

## Purpose

`AgentSpecV1` is the declarative unit for creating advanced supported-local AiDENs agents.

It should let an operator define a local agent without rewriting framework wiring.

## Required fields

```json
{
  "schema": "AgentSpecV1",
  "agent_id": "agent:local-coding-demo",
  "display_name": "Local Coding Agent",
  "support_label": "supported-local",
  "profile": "coding|memory|research|custom-local",
  "provider_policy": {
    "provider": "mock|local",
    "cloud_allowed": false,
    "fallback_allowed": false
  },
  "memory_policy": {
    "enabled": true,
    "mode": "fixture|canonical-seam",
    "requires_view_disclosure": true
  },
  "tool_policy": {
    "allowed_tools": ["repo.read", "repo.list", "repo.search", "patch.propose", "patch.apply", "checks.run"],
    "write_tools_require_permit": true
  },
  "permit_policy": {
    "writes": "operator-approved",
    "commands": "operator-approved",
    "network": "forbidden"
  },
  "verification_policy": {
    "required_checks": ["schema", "sandbox", "digest", "support-claim"],
    "fail_closed": true
  },
  "evidence_policy": {
    "emit_run_bundle": true,
    "emit_tool_receipts": true,
    "emit_permit_receipts": true,
    "emit_abstention_receipts": true
  },
  "budget_policy": {
    "max_turns": 8,
    "max_tool_calls": 16,
    "deadline_seconds": 300
  }
}
```

## Validation rules

- `support_label` must be one of the known support labels.
- Cloud provider settings must be rejected unless support profile explicitly allows them.
- Write tools must require permits.
- Memory policy must not designate AiDENs as canonical memory owner.
- Evidence policy must require run bundle and receipts.
- Budget policy must be bounded.

## Ownership

AiDENs owns the AgentSpec display/operator contract. Canonical sibling crates own the semantics the spec routes into.
