# Source Roots and Touch Map

## Runtime paths

```text
~/Coding/Libraries/AiDENs       current AiDENs scaffold
~/Coding/Recall                 primary extraction source
~/Coding/Recall-Coding          secondary source for coding-tool ideas
~/Coding/Libraries              canonical libraries
~/Coding/Libraries2             additional libraries
```

## Current AiDENs files to modify first

```text
crates/aidens-runner/src/lib.rs
crates/aidens-provider-kit/src/lib.rs
crates/aidens-tool-kit/src/lib.rs
crates/aidens-cli/src/lib.rs
crates/aidens-app-kit/src/lib.rs
crates/aidens-contracts/src/lib.rs
crates/aidens-config/src/lib.rs
crates/aidens-capability-kit/src/lib.rs
crates/aidens-receipts/src/lib.rs
```

## Recall files to inspect and mine

```text
recall-session/src/provider.rs
recall-session/src/provider_bridge.rs
recall-session/src/session/tool_dispatch.rs
recall-session/src/tool_catalog.rs
recall-session/src/approval.rs
recall-session/src/control.rs
recall-session/src/config.rs
recall-session/src/path_safety.rs
recall-session/src/session/arbiter.rs
recall-session/src/session/arbiter_fast_signals.rs
recall-session/src/session/arbiter_intents.rs
deps/llm-pipeline/src/tool_loop.rs
deps/llm-pipeline/src/lib.rs
```

## Recall-Coding files to mine secondarily

Use only for coding profile/tool examples:

```text
recall-session/src/tools/workspace_audit.rs
recall-session/src/tools/workspace_patch.rs
recall-session/src/tools/run_checks.rs
recall-session/src/tools/coding_support.rs
```

If exact filenames differ, locate with:

```bash
find ~/Coding/Recall-Coding -type f | grep -E 'workspace|coding|run_checks|patch|audit'
```

## Libraries to use where feasible

Primary existing libraries:

```text
llm-tool-runtime
llm-pipeline
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

Do not force all into this pass. Provider/tool/app-plan/doctor path comes first.
