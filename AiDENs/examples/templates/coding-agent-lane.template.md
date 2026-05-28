# Coding-Agent Lane Template

Use this as an operator checklist for a bounded coding task.

## Supported AiDENs Surfaces

- Profile: `coding-agent`
- Provider: `mock` by default; cloud providers remain unavailable unless executable and tested
- Default tools: repo read/list/search/stat and patch proposal
- Side effects: patch apply and run checks require explicit permits
- Evidence: run report, turn report, tool exposure, tool invocation receipts, agency reports, canonical receipt log

## Workflow

1. Run `doctor`, `provider-check`, and `tools inspect`.
2. Read only the files needed for the task.
3. Propose or apply a bounded patch only after permit policy allows it.
4. Run targeted checks.
5. Inspect receipts for provider route, tool exposure, permit use, failures, degraded paths, and agency gate output.

## Non-Imported Recall-Coding Assumptions

- no app-local project data directory;
- no app-specific hook runner;
- no app-specific checkpoint store;
- no app-specific agent manifest contract;
- no app-specific session or UI state;
- no Recall-Coding tool IDs.
