# Master Issue Matrix

## Executive summary

This matrix is built against the **current** repository state, not the earlier empty-spec state.

### Current shape

The repo already has:
- a coherent desktop shell,
- concrete run routing,
- setup and provider configuration,
- durable settings persistence,
- keyring-first secret handling,
- provider test and model discovery commands,
- queued baseline capture.

It still does **not** have a finished repair lane:
- setup readiness is not strict enough,
- repo intake is still too trusting,
- the current failure is not surfaced well,
- phase semantics are still misleading,
- candidate generation is not wired,
- verification is not wired,
- memory, audit, and apply are not wired,
- release hardening is incomplete.

The right next move is **not** another broad polish pass.
The right next move is to make the run lane truthful, deterministic, and inspectable.

## Totals

| Total issues | Done | Open | Open P0 | Open P1 | Open P2 | Total points | Open points |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 47 | 12 | 35 | 15 | 19 | 1 | 206 | 162 |

## Immediate release blockers

Start here, in this order:

1. `FW-013` — strict setup gating after real provider validation
2. `FW-015` — repo preflight before queueing
3. `FW-017` — failed-run diagnostics on the Run page
4. `FW-019` — truthful worker phase/event order
5. `FW-020` — stop marking baseline-only success as full completion
6. `FW-023` through `FW-029` — real candidate lane
7. `FW-030` through `FW-034` — paired verification and hard gates
8. `FW-035` through `FW-041` — memory, audit, apply, and recovery
9. `FW-042` through `FW-045` — security and release discipline

## Landed foundation

| issue_id | priority | title |
| --- | --- | --- |
| FW-001 | P0 | Fix Tauri invoke payload shape for run detail and run events |
| FW-002 | P0 | Introduce concrete run routing with `#/run/:runId` |
| FW-003 | P0 | Make Recent runs and Create run navigate to the exact run detail page |
| FW-004 | P1 | Separate event loading, empty, and error states |
| FW-005 | P1 | Add cancel and retry controls to the run header |
| FW-006 | P0 | Add durable settings schema and migration support |
| FW-007 | P0 | Implement `save_settings` and return a typed redacted `SettingsView` |
| FW-008 | P0 | Add keyring-first secret storage with explicit fallback support |
| FW-009 | P0 | Add provider test and model discovery commands |
| FW-010 | P0 | Build first-run Setup and actionable Settings surfaces |
| FW-011 | P0 | Resolve provider/model selection in the core crate |
| FW-012 | P0 | Gate run creation behind setup state |

## Open issues by epic

### B. Provider and setup hardening

| issue_id | priority | title | depends_on |
| --- | --- | --- | --- |
| FW-013 | P0 | Require successful provider validation before setup is considered complete | FW-009, FW-011, FW-012 |
| FW-014 | P1 | Introduce provider readiness freshness and stale-state semantics | FW-013 |
| FW-047 | P1 | Define provider-scoped model precedence and override validation | FW-013, FW-014 |

### C. Run lane truthfulness and intake safety

| issue_id | priority | title | depends_on |
| --- | --- | --- | --- |
| FW-015 | P0 | Add repo preflight validation before queueing a run | FW-011 |
| FW-016 | P1 | Add a folder picker and repo-root normalization UX | FW-015 |
| FW-017 | P0 | Surface run failure diagnostics directly on the Run page | FW-015, FW-019, FW-022 |
| FW-018 | P1 | Fix status-tone mapping, baseline-card truthfulness, and header metadata clarity | FW-017 |
| FW-019 | P0 | Emit truthful worker phase transitions and progress events | FW-015 |
| FW-020 | P0 | Stop treating baseline-only success as completed repair | FW-019 |
| FW-021 | P1 | Add live progress streaming and an expandable event payload viewer | FW-017, FW-019 |
| FW-022 | P1 | Expand baseline failure taxonomy into structured operator-grade classes | FW-015, FW-019 |

### D. Candidate generation and review

| issue_id | priority | title | depends_on |
| --- | --- | --- | --- |
| FW-023 | P0 | Add candidate persistence schema and typed contracts | FW-020 |
| FW-024 | P0 | Integrate `llm-pipeline` candidate generation into the queued worker | FW-013, FW-023 |
| FW-025 | P1 | Implement parse-retry and validation-retry loops with the correct retry owner | FW-024, FW-026 |
| FW-026 | P0 | Enforce patch policy caps and forbidden-path validation before verification | FW-023 |
| FW-027 | P1 | Persist candidate diagnostics and status transitions | FW-023, FW-026 |
| FW-028 | P0 | Build a real Candidates panel with summaries, diff preview, validation state, and touched symbols | FW-023, FW-027 |
| FW-029 | P1 | Add candidate selection, approval, rejection, and operator action logging | FW-027, FW-028 |

### E. Verification and decisioning

| issue_id | priority | title | depends_on |
| --- | --- | --- | --- |
| FW-030 | P0 | Implement paired verification service and persistence | FW-024, FW-026, FW-027 |
| FW-031 | P1 | Compute recommendation verdicts and structured explanations | FW-030 |
| FW-032 | P0 | Render the Verification panel with baseline, patched, diff, and recommendation state | FW-030, FW-031 |
| FW-033 | P0 | Enforce approval and apply gating on verified recommended candidates only | FW-029, FW-030, FW-031 |
| FW-034 | P1 | Add regression/rejection language and operator guidance | FW-031, FW-032 |

### F. Memory and audit

| issue_id | priority | title | depends_on |
| --- | --- | --- | --- |
| FW-035 | P1 | Implement the canonical export → bridge transform → memory import path | FW-030, FW-031, FW-033 |
| FW-036 | P1 | Implement scoped memory retrieval and the Memory panel | FW-035 |
| FW-037 | P1 | Implement explicit audit/evidence-reference commands and the Audit panel | FW-035 |
| FW-038 | P1 | Surface import warnings and authority-boundary language across memory and audit | FW-035, FW-036, FW-037 |

### G. Apply, recovery, and hardening

| issue_id | priority | title | depends_on |
| --- | --- | --- | --- |
| FW-039 | P0 | Implement the explicit final apply flow to the live working tree | FW-033 |
| FW-040 | P0 | Harden queue restart recovery, stale-job reclaim, and duplicate protection | FW-019, FW-039 |
| FW-041 | P1 | Improve cancellation semantics for in-flight worker execution | FW-019, FW-021, FW-040 |
| FW-042 | P0 | Add security regression tests for secrets, logs, and storage boundaries | FW-008, FW-013, FW-035, FW-039 |
| FW-043 | P1 | Add provider integration fixtures and hermetic model-list tests | FW-013, FW-014, FW-042 |
| FW-044 | P1 | Expand the fixture matrix and acceptance harness for the full product loop | FW-024, FW-030, FW-035, FW-039, FW-040 |
| FW-045 | P1 | Add a manual smoke script, release checklist, and docs-alignment pass | FW-044 |
| FW-046 | P2 | Add a diagnostics surface and honest readiness badges for unwired panels | FW-017, FW-018, FW-038, FW-045 |

## Definition of ‘finished’ for this repo

Forge Workbench is “finished” only when all of the following are true:

1. setup requires a genuinely validated provider/model path,
2. repo intake preflights before queueing,
3. failed runs explain themselves in the UI,
4. the phase model is truthful,
5. candidate generation and patch validation are real,
6. paired verification produces recommendation or rejection,
7. memory import and later retrieval work in scope,
8. audit mode exposes evidence references explicitly,
9. approval and final apply are safe and explicit,
10. restart recovery and secret regressions pass,
11. docs and badges match the product that actually ships.
