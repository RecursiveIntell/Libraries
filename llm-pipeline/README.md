# llm-pipeline

Reusable node payloads for LLM workflows: prompt templating, multi-backend calls (Ollama/OpenAI), defensive parsing, streaming, sequential chaining, tool loops, retry policies, and provider routing.

## What it gives you

- **Prompt templating**: `{{variable}}` interpolation with context injection
- **Multi-backend calls**: Ollama and OpenAI-compatible endpoints via feature flags
- **Defensive parsing**: JSON/text extraction from model responses via `llm-output-parser`
- **Streaming**: token-by-token streaming through `PayloadContext` callback
- **Sequential chaining**: compose multiple `LlmCall` stages into pipelines
- **Tool loops**: `tool_loop` module for multi-turn tool-calling workflows
- **Retry policies**: configurable retry with backoff via `retry_policy`
- **Output strategies**: `Text`, `JsonObject`, `JsonArray`, `ThinkingMode`
- **Payload trait**: implements `agent_graph::Payload` for use as graph nodes
- **Trace integration**: `TraceCtx` from `stack-ids` for request correlation

## Quick start

```rust
use llm_pipeline::{ExecCtx, LlmCall, OutputStrategy};

let ctx = ExecCtx::builder("http://localhost:11434").build()?;
let call = LlmCall::new("Summarize: {{text}}")
    .with_model("llama3.2:3b")
    .with_output_strategy(OutputStrategy::JsonObject);

let output = call.invoke(&ctx).await?;
println!("{}", output.value);
```

## Multi-provider routing (OpenAI feature)

```rust
use llm_pipeline::{ExecCtx, LlmCall, OutputStrategy};

let ctx = ExecCtx::builder("https://api.openai.com/v1")
    .with_api_key(std::env::var("OPENAI_API_KEY")?)
    .build()?;
let call = LlmCall::new("Answer: {{question}}")
    .with_model("gpt-4o")
    .with_output_strategy(OutputStrategy::Text);

let output = call.invoke(&ctx).await?;
```

## Streaming pipeline

```rust
use llm_pipeline::{ExecCtx, LlmCall, OutputStrategy};

let ctx = ExecCtx::builder("http://localhost:11434").build()?;
let call = LlmCall::new("Write a story about {{topic}}")
    .with_model("llama3.2:3b")
    .with_streaming(true);

// Tokens stream through PayloadContext::on_token callback
// when used as an agent-graph node
let output = call.invoke(&ctx).await?;
```

## Payload chain

Compose multiple LLM calls into a sequential pipeline:

```rust
use llm_pipeline::{ExecCtx, LlmCall, Pipeline, OutputStrategy};

let ctx = ExecCtx::builder("http://localhost:11434").build()?;
let pipeline = Pipeline::new()
    .stage(LlmCall::new("Extract key points: {{text}}"))
    .stage(LlmCall::new("Summarize in one sentence: {{prev_output}}"));

let output = pipeline.run(&ctx, json!({"text": "..."})).await?;
```

## Examples

```bash
cargo run -p llm-pipeline --example basic_pipeline
cargo run -p llm-pipeline --example streaming_pipeline
cargo run -p llm-pipeline --example thinking_mode
cargo run -p llm-pipeline --example context_injection
cargo run -p llm-pipeline --example payload_chain
cargo run -p llm-pipeline --example mock_example
cargo run -p llm-pipeline --example provider_routing --features openai
```

## Ecosystem

- **stack-ids**: `TraceCtx` for request correlation, `AttemptId`/`TrialId` for retry lineage
- **agent-graph**: `LlmCall` implements `Payload` trait for use as graph nodes
- **llm-output-parser**: Defensive JSON/text extraction from model responses
- **llm-tool-runtime**: Provider-agnostic tool contracts, registry, dispatch
- **agent-graph-mcp**: MCP server exposing graph-orchestrated LLM workflows

## stack-ids integration

Fully integrated. `TraceCtx` is re-exported from the crate root. Per-call trace context flows through the execution pipeline.

## Verification

```bash
cargo test -p llm-pipeline
cargo clippy -p llm-pipeline --all-targets -- -D warnings
```

## License

MIT