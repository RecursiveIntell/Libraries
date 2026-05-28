# P25 Flagship Supported-Local Agent Demo Spec

## Goal

Create one local demo that proves AiDENs can build a useful evidence-bearing agent without cloud, fake autonomy, or shadow semantics.

## Demo name

`examples/flagship-local-coding-agent/`

## Required flow

1. Load a fixture repository.
2. Read a task request.
3. Inspect files with supported local repo tools.
4. Produce one of:
   - patch proposal, or
   - explicit abstention with reason.
5. Require permit before write/apply.
6. Emit tool receipts.
7. Emit provider route receipt, even if mock/local.
8. Emit `AiDENsRunBundleV2`.
9. Emit support-tier disclosure.
10. Replay deterministically.

## Required files

Recommended:

```text
examples/flagship-local-coding-agent/README.md
examples/flagship-local-coding-agent/task.md
examples/flagship-local-coding-agent/expected_output.md
fixtures/p25/coding-agent-repo/
fixtures/p25/coding-agent-repo/README.md
fixtures/p25/coding-agent-repo/src/lib.rs
```

## Acceptance

The demo passes if:
- it runs with no network;
- it uses only supported-local surfaces;
- it emits receipts;
- it does not write without permit;
- replay reproduces the run bundle;
- failure/abstention is honest.

## Non-goals

- cloud models,
- real external provider calls,
- autonomous background loops,
- open-ended codebase modification,
- full coding assistant product.
