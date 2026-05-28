# 07 — App Plan and Profile Model

## Why plans exist

Profiles are dangerous if they secretly enable power. AiDENs must make profiles expand into explicit plans before runtime construction.

Bad:

```rust
AiDENsProfile::CodingAgent.build().await?;
```

Good:

```rust
let plan = AiDENsProfile::CodingAgent.expand();
plan.print_risk_summary();
plan.validate()?;
let app = AiDENsApp::from_plan(plan).build().await?;
```

## `AiDENsAppPlanV1`

```text
AiDENsAppPlanV1
  plan_id
  schema_version
  app
  providers
  tools
  security
  permits
  memory
  execution
  queue
  schedule
  daemon
  ui
  conformance
  dangerous_overrides
  created_at
```

### App section

```text
app_id
app_name
namespace
data_dir
cache_dir
config_path
socket_namespace
profile_id
instance_id
```

### Provider section

```text
default_provider
fallback_providers
local_only_policy
provider_health_policy
native_tool_mode_policy
parser_fallback_policy
```

### Tools section

```text
bundles
disabled_bundles
explicit_tools
exposure_policy
risk_class_overrides
descriptor_schema_version
full_registry_debug_allowed
```

### Security section

```text
network_policy
shell_policy
file_read_policy
file_write_policy
sandbox_roots
symlink_policy
secret_redaction_policy
dangerous_auto_approval_allowed
```

### Memory section

```text
mode: disabled | optional | required
stores
scopes
valid_time_policy
recorded_time_policy
capture_policy
personal_memory_allowed
repo_memory_allowed
```

### Execution section

```text
receipt_level
trace_policy
budget_policy
degradation_policy
streaming_policy
config_hot_swap_policy
```

### Queue/schedule section

```text
queue_mode
lease_policy
retry_policy
cancellation_policy
trigger_specs
overlap_policy
misfire_policy
timezone_policy
host_wake_policy
```

## Default profiles

### `ChatOnly`

Safe defaults:

```text
no tools
no memory writes
no daemon
no queue
provider required
receipts basic
```

### `ToolUsingAgent`

```text
read-only starter tools allowed
write tools require approval
network disabled unless provider requires it
full receipts
```

### `CodingAgent`

```text
repo read/search allowed
patch proposal allowed
patch apply requires approval
shell requires approval
web disabled by default
memory optional and repo-scoped
queue disabled by default
full receipts
```

### `MemoryAgent`

```text
memory required
memory writes require policy
personal memory opt-in
bitemporal query provenance enabled
```

### `AutonomousDaemon`

```text
daemon enabled
queue enabled
schedule enabled
write/shell/network still require explicit permit policy
dangerous auto-approval denied by default
max continuation depth set
full receipts
```

### `ResearchWorkbench`

```text
research tools enabled
web configurable
memory optional
kernel optional
simulator loops require explicit budget
```

## Profile expansion rules

1. Expansion is deterministic.
2. Expansion emits an `AppPlanV1`.
3. Dangerous capabilities are denied unless explicitly overridden.
4. Generated project includes the expanded plan summary.
5. User can diff plan revisions.
6. Plan revision supersession is receipt-bearing.

## Plan validation rules

Plan validation must catch:

```text
provider configured but unavailable
memory required but migration failed
daemon requested without namespace
queue requested without receipt ledger
schedule requested without queue policy
auto approval requested without hard denylist
shell allowed without sandbox root
web enabled in local-only profile
native tool mode requested for unsupported provider
parser fallback requested for patch/apply without boundary receipt policy
```

## Output in generated apps

Every generated app should include:

```text
aidens.toml
AppPlan.summary.md
RiskSummary.md
src/main.rs
src/tools.rs
tests/doctor.rs
tests/no_hidden_fallback.rs
tests/capability_truth.rs
tests/approval_policy.rs
```
