# ollama-vision

Robust Ollama vision model toolkit for image tagging and captioning with structured output parsing.

## Example

```rust
use ollama_vision::{CaptionOptions, OllamaVisionConfig};
use std::path::Path;

async fn caption(path: &Path) -> Result<String, ollama_vision::CaptionError> {
    let config = OllamaVisionConfig::with_model("llava");
    let client = reqwest::Client::new();
    ollama_vision::caption_image(&client, &config, path, &CaptionOptions::default()).await
}
```

Run Ollama and pull the selected vision model before calling it. The image must be readable at the supplied path. `tag_image` returns parsed tags; the `_base64` variants accept image bytes encoded by the caller. Requests send the image to the configured Ollama endpoint.

## Ecosystem

- **llm-output-parser**: Used internally for structured response parsing

## stack-ids integration

Not yet integrated. Planned: add `TraceCtx` propagation for trace lineage (TRACE-1).
