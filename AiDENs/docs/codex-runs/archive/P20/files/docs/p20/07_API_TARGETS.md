# 07 — API Targets

## Public easy button

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

## Plan-first flow

```rust
let plan = AiDENsProfile::CodingAgent.expand();
plan.validate()?;
println!("{}", plan.human_summary());
println!("{}", plan.risk_summary());

let app = AiDENsApp::from_plan(plan).build().await?;
```

## Runner flow

```rust
let runner = AiDENsRunner::builder()
    .provider(provider_stack)
    .tools(tool_registry)
    .permits(permit_policy)
    .receipts(receipt_ledger)
    .build()?;

let output = runner.run(AiDENsRunInput::new("inspect this repo")).await?;
assert!(output.receipt.route.actual_route.is_some());
```

## Doctor flow

```bash
aidens doctor
aidens check-config --file aidens.toml
aidens list-capabilities
aidens list-tools
aidens provider-check
```
