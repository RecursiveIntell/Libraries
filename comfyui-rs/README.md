# comfyui-rs

Async Rust client for ComfyUI -- REST API, WebSocket progress tracking, and workflow building.

## Example

```rust
use comfyui_rs::ComfyClient;
use serde_json::json;

let client = ComfyClient::new("http://127.0.0.1:8188");
let workflow = json!({ /* ComfyUI workflow JSON */ });
let prompt_id = client.queue_prompt(&workflow).await?;
let output = client.wait_for_completion(&prompt_id).await?;
```

## Ecosystem

- **stack-ids**: Traced method variants (`queue_prompt_traced`, `image_traced`, etc.) accept `TraceCtx` for correlation

## stack-ids integration

Integrated via traced method variants. Pass `Option<&TraceCtx>` to any `*_traced()` method for correlation logging.
