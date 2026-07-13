//! Example: Multi-provider routing with Ollama and OpenAI-compatible backends.
//!
//! Demonstrates how to:
//! (1) Create a pipeline with an Ollama backend pointing to http://127.0.0.1:11434
//! (2) Execute a simple prompt against Ollama
//! (3) Show chain orchestration with multiple LlmCall steps
//! (4) Route a second context through an OpenAI-compatible backend (Ollama's /v1/ endpoint)
//!
//! Run with: `cargo run -p llm-pipeline --example provider_routing`
//!
//! Requires Ollama running at http://127.0.0.1:11434 with a model like `glm-5.2:cloud`.
//! For the OpenAI-compatible section, compile with `--features openai`.

use llm_pipeline::events::{Event, FnEventHandler};
use llm_pipeline::payload::Payload;
use llm_pipeline::{Chain, ExecCtx, LlmCall, LlmConfig, OllamaBackend};
use serde_json::json;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let ollama_url = "http://127.0.0.1:11434";
    let model = "glm-5.2:cloud";

    println!("=== LLM Pipeline: Multi-Provider Routing Example ===\n");

    // ─────────────────────────────────────────────────────
    // Part 1: Ollama backend — simple single prompt
    // ─────────────────────────────────────────────────────
    println!("--- Part 1: Ollama backend (single prompt) ---\n");

    let ollama_ctx = ExecCtx::builder(ollama_url)
        .var("domain", "systems programming")
        .event_handler(Arc::new(FnEventHandler(|event: Event| match event {
            Event::PayloadStart { name, kind } => {
                eprintln!("  [event] start: {} ({})", name, kind);
            }
            Event::PayloadEnd { name, ok } => {
                eprintln!("  [event] end: {} ok={}", name, ok);
            }
            _ => {}
        })))
        .build();

    println!("  Backend: {}", ollama_ctx.backend.name());
    println!("  Base URL: {}", ollama_ctx.base_url);

    let simple_call = LlmCall::new(
        "classify",
        "In one sentence, explain what {input} means for {domain}.",
    )
    .with_model(model)
    .with_config(LlmConfig::default().with_max_tokens(256))
    .expecting_text();

    let input = json!("Rust's ownership model");
    println!("\n  Prompt input: {}\n", input);

    let output = simple_call.invoke(&ollama_ctx, input).await?;

    println!("\n  Response: {}\n", output.value.as_str().unwrap_or("(non-string output)"));
    if let Some(ref model_name) = output.model {
        println!("  Model: {}", model_name);
    }
    println!("  Wall time: {}ms\n", output.wall_time_ms);

    // ─────────────────────────────────────────────────────
    // Part 2: Chain orchestration with Ollama
    // ─────────────────────────────────────────────────────
    println!("--- Part 2: Chain orchestration (Ollama) ---\n");

    let chain_ctx = ExecCtx::builder(ollama_url)
        .var("domain", "systems programming")
        .var("audience", "engineers")
        .event_handler(Arc::new(FnEventHandler(|event: Event| match event {
            Event::PayloadStart { name, kind } => {
                eprintln!("  [chain event] start: {} ({})", name, kind);
            }
            Event::Token { chunk, .. } => {
                eprint!("{}", chunk);
            }
            Event::PayloadEnd { name, ok } => {
                eprintln!("\n  [chain event] end: {} ok={}", name, ok);
            }
            _ => {}
        })))
        .build();

    let chain = Chain::new("analyze-and-refine")
        .push(Box::new(
            LlmCall::new(
                "draft",
                "You are a {domain} expert writing for {audience}. \
                 Analyze the following and return a JSON object with 'topic' (string) \
                 and 'analysis' (string):\n\n{input}",
            )
            .with_model(model)
            .with_config(LlmConfig::default().with_json_mode(true)),
        ))
        .push(Box::new(
            LlmCall::new(
                "refine",
                "Given this analysis, produce a final JSON with 'topic' (string) \
                 and 'refined_analysis' (string) that is clearer and more concise:\n\n{input}",
            )
            .with_model(model)
            .with_config(LlmConfig::default().with_json_mode(true))
            .with_streaming(true),
        ));

    println!("  Chain steps: {}\n", chain.len());

    let chain_input = json!("Ownership and borrowing in Rust");
    println!("  Chain input: {}\n", chain_input);

    println!("  Executing chain...\n");
    let chain_output = chain.execute(&chain_ctx, chain_input).await?;

    println!("\n\n  === Chain Final Output ===");
    println!(
        "  {}",
        serde_json::to_string_pretty(&chain_output.value).unwrap_or_default()
    );
    if let Some(ref diag) = chain_output.diagnostics {
        println!("  Parse strategy: {:?}", diag.strategy);
        println!("  Parse OK: {}", diag.ok());
    }
    println!("  Wall time: {}ms\n", chain_output.wall_time_ms);

    // ─────────────────────────────────────────────────────
    // Part 3: OpenAI-compatible backend routing
    // ─────────────────────────────────────────────────────
    // Ollama exposes an OpenAI-compatible endpoint at /v1/.
    // This shows how to swap backends while keeping the same payload API.
    #[cfg(feature = "openai")]
    {
        println!("--- Part 3: OpenAI-compatible backend routing ---\n");

        use llm_pipeline::OpenAiBackend;

        let openai_ctx = ExecCtx::builder(ollama_url)
            .backend(Arc::new(OpenAiBackend::new()))
            .var("domain", "systems programming")
            .event_handler(Arc::new(FnEventHandler(|event: Event| match event {
                Event::PayloadStart { name, kind } => {
                    eprintln!("  [openai event] start: {} ({})", name, kind);
                }
                Event::PayloadEnd { name, ok } => {
                    eprintln!("  [openai event] end: {} ok={}", name, ok);
                }
                _ => {}
            })))
            .build();

        println!("  Backend: {}", openai_ctx.backend.name());
        println!("  Base URL: {}", openai_ctx.base_url);

        let openai_call = LlmCall::new(
            "openai-summarize",
            "Summarize in one sentence: {input}",
        )
        .with_model(model)
        .with_config(LlmConfig::default().with_max_tokens(256))
        .expecting_text();

        let openai_input = json!("The Rust ownership system prevents memory safety bugs at compile time without garbage collection");
        println!("\n  Prompt input: {}\n", openai_input);

        let openai_output = openai_call.invoke(&openai_ctx, openai_input).await?;

        println!(
            "\n  Response: {}\n",
            openai_output.value.as_str().unwrap_or("(non-string output)")
        );
        if let Some(ref model_name) = openai_output.model {
            println!("  Model: {}", model_name);
        }
        println!("  Wall time: {}ms\n", openai_output.wall_time_ms);
    }

    #[cfg(not(feature = "openai"))]
    {
        println!("--- Part 3: OpenAI-compatible backend routing ---\n");
        println!("  (Skipped — compile with --features openai to enable OpenAiBackend)\n");
    }

    // ─────────────────────────────────────────────────────
    // Part 4: Explicit backend selection via Arc<dyn Backend>
    // ─────────────────────────────────────────────────────
    println!("--- Part 4: Explicit backend selection ---\n");

    use llm_pipeline::backend::Backend;

    let explicit_backend: Arc<dyn Backend> = Arc::new(OllamaBackend);
    let explicit_ctx = ExecCtx::builder(ollama_url)
        .backend(explicit_backend)
        .build();

    println!("  Backend: {}", explicit_ctx.backend.name());

    let quick_call = LlmCall::new("quick", "Reply with a one-word answer: {input}")
        .with_model(model)
        .with_config(LlmConfig::default().with_max_tokens(64))
        .expecting_text();

    let quick_output = quick_call
        .invoke(&explicit_ctx, json!("What language is Rust?"))
        .await?;

    println!(
        "  Response: {}\n",
        quick_output.value.as_str().unwrap_or("(non-string output)")
    );

    println!("=== Example complete ===");
    Ok(())
}