# AppPlan and Doctor Spec

## Why AppPlan matters

AiDENs should not expose internal crate complexity to app authors. The central public object is a declarative plan that can be validated, compiled, diagnosed, and then executed.

```text
Profile + TOML + app hooks -> AiDENsAppPlanV1 -> validate -> compile -> runtime
```

## Minimum `AiDENsAppPlanV1`

Required fields or equivalents:

```text
app_id
profile_id
config_generation
provider.kind
provider.model
provider.mock_response optional
provider.base_url optional
tools.enabled_bundles
tools.sandbox_root optional
security.approval_mode
security.network_policy
security.write_policy
memory.mode
receipts.level
runtime.budget/default_timeout
queue.mode
schedule.mode
daemon.mode
```

## Minimum `AiDENsDoctorReportV1`

Required sections:

```text
config
provider
tools
security
receipts
memory
queue
schedule
daemon
runtime
```

Each section must carry:

```text
status: healthy | degraded | disabled | unavailable | deferred | failed
reason_codes: Vec<String>
warnings: Vec<String>
```

## CLI commands

```bash
aidens profile list
aidens profile explain coding-agent
aidens plan validate --config aidens.toml
aidens plan compile --config aidens.toml --out target/aidens-plan.json
aidens doctor --config aidens.toml
aidens run --config aidens.toml "hello"
```

## Profile explanation requirement

`coding-agent` must visibly state that shell/write/network actions require approval and are not auto-approved by default.
