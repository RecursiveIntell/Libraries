# comfyui-rs

Async Rust client for ComfyUI -- REST API, WebSocket progress tracking, and workflow building.

## Example

```rust
use comfyui_rs::{ComfyClient, GenerationOutcome};
use std::time::Duration;

async fn submit(workflow: serde_json::Value) -> comfyui_rs::Result<GenerationOutcome> {
    let client = ComfyClient::new("http://127.0.0.1:8188");
    let prompt_id = client.queue_prompt(&workflow).await?;
    client.wait_for_completion(&prompt_id, Duration::from_secs(120)).await
}
```

The server must already be running with the models referenced by a valid ComfyUI API-format workflow. `Txt2ImgRequest` can build a basic workflow; `wait_for_completion_ws` adds progress callbacks. These calls submit work to the configured ComfyUI instance.

## Ecosystem

- **stack-ids**: Traced method variants (`queue_prompt_traced`, `image_traced`, etc.) accept `TraceCtx` for correlation

## stack-ids integration

Integrated via traced method variants. Pass `Option<&TraceCtx>` to any `*_traced()` method for correlation logging.
