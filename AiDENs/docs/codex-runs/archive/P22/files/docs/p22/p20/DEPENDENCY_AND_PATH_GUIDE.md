# Dependency and Path Guide

## Preferred dependencies

Use canonical libraries from `~/Coding/Libraries` where possible:

```text
llm-tool-runtime
stack-ids
knowledge-runtime
semantic-memory
semantic-memory-forge
forge-memory-bridge
verification-control
verification-policy
job-queue
agent-graph
```

## llm-pipeline source

Recall contains a usable `deps/llm-pipeline` tree. The next run should try to use that for provider execution.

Expected source:

```text
~/Coding/Recall/deps/llm-pipeline
```

If adding it as a path dependency breaks because of workspace dependency expectations, use a local AiDENs provider abstraction with disabled/mock passing first and document the path blocker. Do not fake real execution.

## Keep build narrow

Do not pull every library dependency into AiDENs root in this pass. Add only dependencies needed for:

```text
provider trait + mock/disabled/optional ollama
repo-read tool dispatch
AppPlan/doctor CLI
receipts/capability truth
```
