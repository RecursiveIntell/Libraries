# 13 — Implementation Roadmap

## Milestone 0 — Source stabilization

Duration target: 0.5–1 day

Deliverables:

```text
source inventory
current Recall extraction map
AiDENs workspace skeleton
CI dependency law script stub
```

## Milestone 1 — Foundation

Duration target: 2–4 days

Crates:

```text
aidens-contracts
aidens-boundary-kit
aidens-config
aidens-receipts
aidens-capability-kit
aidens-testkit
```

Deliverables:

- schemas generated,
- schema meta-validation,
- config generation pinned,
- canonical receipt sink/log,
- capability truth snapshot,
- boundary repair receipt.

Exit demo:

```rust
let config = AiDENsConfig::load("aidens.toml")?;
let truth = RuntimeCapabilityTruthV1::blocked_from_config(&config);
let report = RunReportV1::blocked(...);
```

## Milestone 2 — Minimal agent runner

Duration target: 3–6 days

Crates:

```text
aidens-provider-kit
aidens-tool-kit
aidens-security-kit
aidens-permit-kit
aidens-arbiter-kit
aidens-budget-kit
aidens-runner
```

Deliverables:

- provider factory,
- native tool loop adapter,
- parser fallback adapter,
- one read-only tool,
- one write tool with approval,
- exact route receipt,
- run receipt,
- tool exposure set.

Exit demo:

```rust
let runner = AiDENsRunner::builder()
    .provider(provider)
    .tools(tools)
    .permit_policy(ApprovalRequired)
    .receipts(ledger)
    .build()?;

let out = runner.run("read this file").await?;
assert!(out.receipt.provider_route.is_some());
```

## Milestone 3 — App kit and CLI

Duration target: 2–4 days

Crates:

```text
aidens-app-kit
aidens-cli
aidens-profile-coding
```

Deliverables:

- `AiDENsAppPlanV1`,
- profile expansion,
- generated project,
- doctor checks,
- starter tests.

Exit demo:

```bash
aidens new coding-agent demo
cd demo
aidens doctor
cargo test
cargo run
```

## Milestone 4 — Memory kit

Duration target: 3–5 days

Crates:

```text
aidens-memory-kit
```

Deliverables:

- memory disabled/optional/required modes,
- semantic-memory store open,
- knowledge-runtime adapter,
- memory write receipts,
- temporal query provenance,
- memory scope policy.

Exit demo:

```bash
aidens new memory-agent demo-memory
cd demo-memory
cargo run -- "what did I tell you yesterday?"
```

## Milestone 5 — Queue/schedule/daemon/UI

Duration target: 5–10 days

Crates:

```text
aidens-queue-kit
aidens-schedule-kit
aidens-wake-kit
aidens-daemon-kit
aidens-tauri-kit
```

Deliverables:

- durable leases,
- schedule trigger law,
- daemon namespace,
- IPC event stream,
- Tauri adapter,
- startup recovery,
- cancellation.

Exit demo:

```bash
aidens new desktop-assistant demo-ui
cd demo-ui
cargo tauri dev
```

## Milestone 6 — Kernel/plan/delegation/repair

Duration target: 5–15 days

Crates:

```text
aidens-kernel-kit
aidens-plan-kit
aidens-delegation-kit
aidens-repair-kit
```

Deliverables:

- graph snapshot input,
- compiled graph digest,
- region run receipt,
- plan revision/supersession,
- delegated authority contracts,
- repair proposals and quarantine.

## Conversion of Recall

Once Milestone 3 is complete:

1. Use AiDENs provider/tool/runner in Recall.
2. Keep Recall-specific tools in Recall.
3. Replace runtime status with AiDENs capability truth.
4. Replace generic config/receipt handling.
5. Convert daemon/UI to shell adapters after Milestone 5.

## Release gates

AiDENs should not publish v0.1 until:

```text
minimal CLI app template works
minimal coding app template works
provider truth correct
tool exposure correct
write tool approval correct
receipts emitted
doctor catches common broken configs
dependency law script passes
```
