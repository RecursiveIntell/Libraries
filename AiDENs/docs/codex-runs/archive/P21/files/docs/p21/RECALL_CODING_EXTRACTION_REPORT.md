# Recall-Coding Extraction Report

## Source Basis

Inspected read-only sources:

- `/home/sikmindz/Coding/Recall-Coding/recall-session/src/tools/workspace_audit.rs`
- `/home/sikmindz/Coding/Recall-Coding/recall-session/src/tools/workspace_patch.rs`
- `/home/sikmindz/Coding/Recall-Coding/recall-session/src/tools/run_checks.rs`
- `/home/sikmindz/Coding/Recall-Coding/recall-session/src/tools/coding_support.rs`
- `/home/sikmindz/Coding/Recall-Coding/.recall-coding/agents/*.md`
- `/home/sikmindz/Coding/Recall-Coding/.recall-coding/hooks/*.json`

Recall-Coding was treated as a pattern source only. No Recall-Coding crate, session type, tool name, data directory, hook format, DB layout, or application-specific state path was imported into AiDENs.

## Reusable Patterns

### Coding Task Shape

Useful pattern:

- start with a small workspace audit;
- identify manifest files, likely entrypoints, tests, and assets;
- choose a bounded patch target;
- create a pre-change checkpoint where the backing runtime supports it;
- apply a bounded patch;
- run checks;
- return receipts and failure summaries.

AiDENs mapping:

- `aidens new coding-agent` creates a runnable mock-backed project with safe read/search tools and receipts;
- `aidens run --config ...` uses `AiDENsRunner`, not a Recall session;
- side-effect tools remain permit-gated through AiDENs permit/report surfaces;
- verification is represented by explicit command receipts and run reports, not hidden post-patch hooks.

### Tool Routing

Useful pattern:

- expose only tools that are relevant to the current lane;
- keep read-only audit/repo tools separate from write/admin tools;
- do not show shell or patch tools unless policy and permits allow them;
- report tool exposure truth in a machine-readable form.

AiDENs mapping:

- `tools inspect` reports declared, registered, executable, exposed, blocked, permit-required, and provider-schema truth;
- coding-agent templates use read/list/search/stat/propose by default;
- patch apply and run checks remain side-effect/admin paths requiring permits.

### Approval And Permit Handling

Useful pattern:

- approvals are typed decisions, not prompt text;
- write/admin actions require an explicit scoped approval;
- shell-like execution is a separate high-risk class;
- rate limits and hard denylists are policy facts, not model suggestions.

AiDENs mapping:

- `aidens-permit-kit` owns the permit surface;
- `aidens-runner` records permit use in tool invocation receipts;
- generated coding-agent configs do not grant write/admin tools by default.

### Failure Surfacing

Useful pattern:

- checks return structured results with command, exit code, stdout/stderr, duration, and success;
- failure summaries should preserve raw logs by receipt reference;
- failed tools and blocked tools are normal outputs, not exceptions hidden from operators.

AiDENs mapping:

- runner receipts record tool invocations, stop rules, schema failures, degraded parser paths, provider errors, and budget exhaustion;
- `run-test-agent` writes `run-report.json`, `turn-report.json`, `tool-exposure.json`, `agency-policy-reports.json`, `event-log.ndjson`, and `summary.md`.

## Quarantined Recall-Coding Assumptions

These were intentionally not extracted:

- `.recall-coding` project data roots;
- Recall-Coding agent front-matter format as an AiDENs contract;
- Recall-Coding hook names or hook execution model;
- Recall tool IDs such as `recall_workspace_patch`;
- local checkpoint storage as canonical memory or repair truth;
- Recall-Coding artifact directories as receipt stores;
- app-specific session state, UI assumptions, and shell wrappers.

## Extracted AiDENs Artifacts

- `examples/configs/coding-agent.toml`
- `examples/coding-agent/README.md`
- `examples/templates/coding-agent-lane.template.md`
- `crates/aidens-integration-tests/tests/phase_07_recall_extraction.rs`

## Residual Gaps

- AiDENs does not yet implement a first-class workspace audit report equivalent to Recall-Coding's detailed manifest report.
- AiDENs does not yet implement checkpoint create/restore as a canonical safe-write primitive.
- AiDENs does not yet implement a complete patch-and-verify lane. The current safe extraction is docs/templates plus existing read/search/propose/permit-gated write surfaces.
