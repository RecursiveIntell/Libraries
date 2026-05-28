# 02 — AiDENs End Product Architecture

## Product posture

AiDENs is the front door to the library ecosystem. It should make a new app possible in hours without hiding unsafe behavior.

The user-facing surface should be small:

```rust
use aidens::prelude::*;

let app = AiDENsApp::builder()
    .name("my-agent")
    .profile(AiDENsProfile::CodingAgent)
    .config_file("aidens.toml")
    .tools(my_tools())
    .build()
    .await?;

app.run().await?;
```

Internally, the app builder should expand the profile into an explicit `AiDENsAppPlanV1`, validate the plan, build the runtime from independent kits, and install doctor/conformance checks.

## Strata

### 1. Law layer

```text
aidens-contracts
aidens-boundary-kit
aidens-config
aidens-receipts
aidens-capability-kit
```

Defines artifact meanings, accepted input language, config truth, execution evidence, and capability truth.

### 2. Capability layer

```text
aidens-provider-kit
aidens-tool-kit
aidens-security-kit
aidens-memory-kit
aidens-kernel-kit
aidens-queue-kit
```

Connects real backends and existing libraries.

### 3. Control layer

```text
aidens-arbiter-kit
aidens-permit-kit
aidens-budget-kit
aidens-governance-kit
aidens-schedule-kit
aidens-delegation-kit
aidens-plan-kit
aidens-repair-kit
```

Decides what can run, when, under what authority, with what checks, and what happens when it fails.

### 4. Composition layer

```text
aidens-runner
aidens-app-kit
```

Coordinates one run and then whole applications.

### 5. Shell layer

```text
aidens-cli
aidens-daemon-kit
aidens-tauri-kit
aidens-web-kit
aidens-testkit
```

Creates projects, hosts runtime, presents UI, exposes diagnostics, and validates semantics.

## Dependency law

```text
aidens-contracts
  ↑
aidens-boundary-kit
  ↑
aidens-config
  ↑
aidens-receipts
  ↑
aidens-capability-kit
  ↑
provider/tool/security/memory/kernel/queue adapters
  ↑
arbiter/permit/budget/governance/schedule/delegation/plan/repair
  ↑
aidens-runner
  ↑
aidens-app-kit
  ↑
aidens / aidens-cli / aidens-daemon-kit / aidens-tauri-kit
```

Forbidden dependency examples:

```text
aidens-contracts -> aidens-runner
aidens-provider-kit -> aidens-memory-kit
aidens-tool-kit -> aidens-tauri-kit
aidens-daemon-kit -> aidens-tauri-kit
aidens-tauri-kit -> provider construction
aidens-app-kit -> new canonical artifact definitions
```

## Authority law

AiDENs should preserve the existing stack's authority distinctions:

| Domain | Authoritative for | Must not become |
|---|---|---|
| Evidence/Forge | raw verification truth | runtime policy engine |
| Bridge | deterministic transform | truth promoter |
| Semantic memory | queryable projection | raw evidence owner |
| Runtime | planning/execution | durable truth store |
| Receipts | execution history | domain truth |
| Control | decision history | memory database |
| UI | presentation | approval/runtime authority |
| Daemon | host runtime ownership | semantic truth source |

## Main public crate

`aidens` should re-export only safe app-level APIs:

```rust
pub mod prelude {
    pub use aidens_app_kit::{AiDENsApp, AiDENsAppBuilder, AiDENsProfile};
    pub use aidens_contracts::{RunReportV1, RuntimeCapabilityTruthV1};
    pub use aidens_tool_kit::{ToolBundle, ToolInstall};
}
```

Advanced users can depend on lower crates directly.

## End product checklist

A correctly built AiDENs app must have:

- one app manifest,
- one expanded app plan,
- one config generation per runtime instance,
- one capability truth surface,
- one receipt ledger,
- explicit provider route labels,
- explicit tool exposure set per run,
- approval/permit decisions for side effects,
- budget/stop rules,
- doctor checks,
- starter conformance tests,
- optional memory/queue/daemon/kernels with explicit receipts.
