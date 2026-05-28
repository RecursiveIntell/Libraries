# P21 Recall / Recall-Coding Extraction Plan

## Purpose

Recall and Recall-Coding already contain working application wiring. P21 should extract the reusable laws and product patterns into AiDENs without turning AiDENs into Recall.

## Extraction categories

### Coding-agent workflows

From Recall-Coding, inspect and document:

- how coding tasks are represented;
- how tools are routed;
- how approvals/permits are represented;
- how failures are surfaced;
- how session state initializes;
- what a developer expects from an agent command surface.

Output:

```text
docs/p21/RECALL_CODING_EXTRACTION_REPORT.md
examples/configs/coding-agent.toml
examples/coding-agent/README.md
```

### Continuous assistant / daemon workflows

From Recall, inspect and document:

- daemon lifecycle;
- heartbeat behavior;
- wake/schedule/queue mechanics;
- safe mode;
- job storm prevention;
- session/IPC boundaries.

Output:

```text
docs/p21/RECALL_DAEMON_EXTRACTION_REPORT.md
examples/configs/daemon-safe.toml
```

### Product UX surfaces

Extract commands/templates only where useful:

- `doctor`/status conventions;
- provider health display;
- tool capability display;
- memory status display;
- safe-mode display.

## Forbidden extraction

- Do not import Recall DB/state assumptions into AiDENs core.
- Do not import app-specific daemon socket paths.
- Do not make Recall’s memory representation authoritative.
- Do not create compatibility shims to force old behavior into AiDENs.
- Do not use Libraries2 `stack-ids`.

## Acceptance

Extraction is successful if AiDENs gains usable profiles/templates/tests, not if Recall code is copied wholesale.
