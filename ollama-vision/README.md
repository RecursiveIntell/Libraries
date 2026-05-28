# ollama-vision

Robust Ollama vision model toolkit for image tagging and captioning with structured output parsing.

## Example

```rust
use ollama_vision::{OllamaVision, VisionRequest};

let client = OllamaVision::new("http://localhost:11434");
let request = VisionRequest::new("llava:latest", "/path/to/image.jpg")
    .with_prompt("Describe this image");
let response = client.generate(&request).await?;
```

## Ecosystem

- **llm-output-parser**: Used internally for structured response parsing

## stack-ids integration

Not yet integrated. Planned: add `TraceCtx` propagation for trace lineage (TRACE-1).
