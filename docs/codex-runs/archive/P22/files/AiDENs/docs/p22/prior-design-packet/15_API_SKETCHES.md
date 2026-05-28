# 15 — API Sketches

These are sketches, not final APIs. They define the intended ergonomic shape.

## App creation

```rust
use aidens::prelude::*;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let app = AiDENsApp::builder()
        .name("my-agent")
        .profile(AiDENsProfile::CodingAgent)
        .config_file("aidens.toml")
        .tools(my_tools())
        .build()
        .await?;

    app.run().await?;
    Ok(())
}
```

## Plan-first creation

```rust
let mut plan = AiDENsProfile::CodingAgent.expand();
plan.security.network_policy = NetworkPolicy::Disabled;
plan.memory.mode = MemoryMode::Optional;
plan.validate()?;

let app = AiDENsApp::from_plan(plan)
    .with_tools(my_tools())
    .build()
    .await?;
```

## Runner use without app kit

```rust
let runner = AiDENsRunner::builder()
    .provider(provider_stack)
    .tools(tool_runtime)
    .arbiter(arbiter)
    .permit_policy(permit_policy)
    .budget(BudgetPolicy::interactive_default())
    .receipts(receipt_ledger)
    .build()?;

let output = runner.run(AiDENsInput::text("inspect this repo")).await?;
println!("{}", output.text);
println!("{}", output.receipt.receipt_id);
```

## Tool bundle

```rust
pub fn my_tools() -> impl ToolBundle {
    ToolBundleBuilder::new("my-app")
        .read_only("repo.read", repo_read_tool())
        .effectful("patch.apply", patch_apply_tool())
        .requires_approval("patch.apply")
        .build()
}
```

## Provider config

```rust
let provider = ProviderStack::builder()
    .default(ProviderConfig::ollama("http://localhost:11434", "qwen2.5-coder:7b"))
    .fallback(ProviderConfig::openai_compatible("openrouter", key))
    .parser_fallback(ParserFallbackPolicy::ExplicitOnly)
    .build()?;
```

## Capability truth

```rust
let truth = app.capabilities().await?;

assert_eq!(truth.provider.default.status, ProviderStatus::Healthy);
assert!(truth.tools.by_id("repo.read").unwrap().executable_this_turn);
assert!(!truth.tools.by_id("patch.apply").unwrap().executable_without_approval);
```

## Receipt inspection

```rust
let out = app.ask("read Cargo.toml").await?;
let receipt = out.receipt;

assert_eq!(receipt.provider_route.mode, ProviderRouteMode::NativeOllama);
assert_eq!(receipt.tool_exposure.allowed_tools, vec!["repo.read"]);
```

## Config generation pinning

```rust
let run = app.runner().prepare("summarize repo").await?;
let pinned = run.execution_context.config_generation_id.clone();

app.apply_config(new_config).await?;

let out = run.execute().await?;
assert_eq!(out.receipt.config_generation_id, pinned);
```

## Daemon mode

```rust
let app = AiDENsApp::builder()
    .profile(AiDENsProfile::AutonomousDaemon)
    .daemon(DaemonMode::Required)
    .build()
    .await?;

app.daemon().start().await?;
```

## Generated CLI

```bash
aidens new coding-agent my-agent
cd my-agent
aidens doctor
aidens list-tools
aidens list-capabilities
aidens provider-check
aidens run "inspect this repo"
```
