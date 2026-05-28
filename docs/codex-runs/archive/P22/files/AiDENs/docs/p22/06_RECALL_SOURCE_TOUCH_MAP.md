# 06 — Recall Source Touch Map

Codex should use this map while extracting from `~/Coding/Recall`.

## Provider route truth

Inspect:

```text
recall-session/src/provider.rs
recall-session/src/provider_bridge.rs
recall-session/tests/native_tool_conformance_tests.rs
recall-session/tests/release_truth_tests.rs
```

Create/modify:

```text
crates/aidens-provider-kit/src/lib.rs
crates/aidens-provider-kit/tests/provider_route_truth.rs
crates/aidens-receipts/src/lib.rs
```

## Tool exposure and registry

Inspect:

```text
recall-session/src/session/tool_dispatch.rs
recall-session/src/tool_catalog.rs
recall-session/tests/tool_catalog_tests.rs
recall-session/tests/tool_routing_tests.rs
_vendor/Libraries/llm-tool-runtime/src/*
```

Create/modify:

```text
crates/aidens-tool-kit/src/lib.rs
crates/aidens-tool-kit/tests/disabled_means_absent.rs
crates/aidens-tool-kit/tests/exposure_policy.rs
```

## Approval / permits

Inspect:

```text
recall-session/src/approval.rs
recall-session/tests/auto_approve_tests.rs
recall-session/tests/phase2_write_tests.rs
```

Create/modify:

```text
crates/aidens-permit-kit/src/lib.rs
crates/aidens-security-kit/src/lib.rs
crates/aidens-permit-kit/tests/permit_required.rs
```

## Config and path safety

Inspect:

```text
recall-session/src/config.rs
recall-session/src/path_safety.rs
recall-session/tests/p0_hotfix_conformance.rs
```

Create/modify:

```text
crates/aidens-config/src/lib.rs
crates/aidens-boundary-kit/src/lib.rs
crates/aidens-config/tests/redaction.rs
```

## Scheduler/queue lessons

Inspect:

```text
recall-session/src/scheduler.rs
recall-session/src/scheduler_store.rs
recall-session/src/scheduler_migration.rs
recall-session/src/jobs.rs
recall-session/tests/future_action_scheduler_tests.rs
recall-session/tests/scheduler_runtime_law_tests.rs
recall-session/tests/trigger_semantics_tests.rs
_vendor/Libraries2/job-queue/src/*
```

Create/modify later:

```text
crates/aidens-schedule-kit/src/lib.rs
crates/aidens-queue-kit/src/lib.rs
crates/aidens-daemon-kit/src/lib.rs
```

## Memory / runtime laws

Inspect:

```text
recall-session/src/session/memory.rs
recall-session/src/memory_policy.rs
_vendor/Libraries/knowledge-runtime/src/*
_vendor/Libraries/semantic-memory/src/*
_vendor/Libraries/semantic-memory-forge/src/*
_vendor/Libraries/forge-memory-bridge/src/*
```

Create/modify later:

```text
crates/aidens-memory-kit/src/lib.rs
```
