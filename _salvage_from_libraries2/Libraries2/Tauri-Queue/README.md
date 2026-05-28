# tauri-queue

Tauri integration for job-queue background job processing with frontend event bridging.

## Example

```rust
use tauri_queue::TauriEventEmitter;

// In your Tauri setup:
let emitter = TauriEventEmitter::new(app_handle.clone());
// Events are emitted as queue:job_started, queue:job_completed, etc.
```

## Ecosystem

- **job-queue**: Core queue engine that Tauri-Queue wraps
- **stack-ids**: `TraceCtx` re-exported for frontend correlation

## stack-ids integration

Fully integrated. `TraceCtx` is re-exported from the crate root.
