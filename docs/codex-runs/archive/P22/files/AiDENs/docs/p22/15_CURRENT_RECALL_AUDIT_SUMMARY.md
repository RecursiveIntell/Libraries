# 15 — Current Recall Audit Summary

This file summarizes what Codex should assume about current Recall.

## Useful extraction material

- Provider execution mode concepts exist.
- `llm-pipeline` bridge exists.
- `llm-tool-runtime` integration exists.
- Exposure planning concepts exist.
- Approval concepts exist.
- Scheduler/trigger tests exist.
- Receipt/conformance tests exist.
- Config/path safety helpers exist.

## Known design hazards

- `RecallSession` concentrates too many responsibilities.
- Text-first/parser-based tool execution existed as a path and must not become the default happy path.
- Daemon/UI/app surfaces can create split-brain if runtime authority is unclear.
- Scheduler, queue, and host wake semantics must be separated.
- Recall-specific tools must not become generic AiDENs tools.
- App-specific configs must not become global AiDENs law.

## Extraction posture

Prefer:

```text
concept -> new AiDENs crate -> test -> adapter
```

Avoid:

```text
copy module -> rename symbols -> ship
```
