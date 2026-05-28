use futures_util::StreamExt;
use reqwest::Client;
use serde_json::Value;
use std::time::Duration;

use crate::error::{ComfyError, Result};
use crate::types::*;

fn normalize(endpoint: String) -> String {
    endpoint.trim_end_matches('/').to_string()
}

/// Async client for a ComfyUI server instance.
///
/// Provides REST methods for prompt queuing, history retrieval, image
/// download, and model discovery. Supports both WebSocket-based real-time
/// progress tracking and polling-based fallback.
///
/// # Example
/// ```no_run
/// use comfyui_rs::ComfyClient;
///
/// # async fn example() -> comfyui_rs::Result<()> {
/// let client = ComfyClient::new("http://127.0.0.1:8188");
/// let healthy = client.health().await?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct ComfyClient {
    http: Client,
    endpoint: String,
    client_id: String,
}

impl ComfyClient {
    /// Create a new client pointing at the given ComfyUI endpoint.
    pub fn new(endpoint: impl Into<String>) -> Self {
        Self {
            http: Client::new(),
            endpoint: normalize(endpoint.into()),
            client_id: "comfyui-rs".to_string(),
        }
    }

    /// Use a custom `reqwest::Client` (for connection pooling, timeouts, TLS).
    pub fn with_http_client(mut self, client: Client) -> Self {
        self.http = client;
        self
    }

    /// Set the client ID used for WebSocket filtering and prompt association.
    pub fn with_client_id(mut self, id: impl Into<String>) -> Self {
        self.client_id = id.into();
        self
    }

    /// Returns the configured endpoint URL.
    pub fn endpoint(&self) -> &str {
        &self.endpoint
    }

    /// Returns the configured client ID.
    pub fn client_id(&self) -> &str {
        &self.client_id
    }

    // ── Health ──────────────────────────────────────────────────────

    /// Check whether ComfyUI is reachable. Any HTTP response (even 500)
    /// means the server is running. Only connection/timeout failures return false.
    pub async fn health(&self) -> Result<bool> {
        let url = format!("{}/queue", self.endpoint);
        match self
            .http
            .get(&url)
            .timeout(Duration::from_secs(5))
            .send()
            .await
        {
            Ok(_) => Ok(true),
            Err(e) if e.is_connect() || e.is_timeout() => Ok(false),
            Err(e) => Err(ComfyError::Network {
                context: format!(
                    "Cannot connect to ComfyUI at {} \u{2014} is the service running?",
                    self.endpoint
                ),
                source: e,
            }),
        }
    }

    // ── Prompt ──────────────────────────────────────────────────────

    /// Queue a workflow for execution. Returns the `prompt_id`.
    pub async fn queue_prompt(&self, workflow: &Value) -> Result<String> {
        let url = format!("{}/prompt", self.endpoint);
        let body = serde_json::json!({
            "prompt": workflow,
            "client_id": self.client_id,
        });

        let resp = self
            .http
            .post(&url)
            .timeout(Duration::from_secs(30))
            .json(&body)
            .send()
            .await
            .map_err(|e| ComfyError::Network {
                context: format!(
                    "Cannot connect to ComfyUI at {} \u{2014} is the service running?",
                    self.endpoint
                ),
                source: e,
            })?;

        if !resp.status().is_success() {
            let status = resp.status().as_u16();
            let body_text = resp.text().await.unwrap_or_default();
            return Err(ComfyError::Http {
                status,
                body: body_text,
            });
        }

        let json: Value = resp.json().await.map_err(|e| ComfyError::Network {
            context: "Failed to parse ComfyUI /prompt response".into(),
            source: e,
        })?;

        // Check for node errors
        if let Some(errors) = json.get("node_errors") {
            if let Some(obj) = errors.as_object() {
                if !obj.is_empty() {
                    return Err(ComfyError::NodeErrors(
                        serde_json::to_string_pretty(errors).unwrap_or_default(),
                    ));
                }
            }
        }

        json.get("prompt_id")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| ComfyError::InvalidResponse("Response missing prompt_id".into()))
    }

    // ── History ─────────────────────────────────────────────────────

    /// Fetch the history entry for a prompt. Returns `None` if not yet available.
    pub async fn history(&self, prompt_id: &str) -> Result<Option<PromptHistory>> {
        let url = format!("{}/history/{}", self.endpoint, prompt_id);
        let resp = self
            .http
            .get(&url)
            .timeout(Duration::from_secs(10))
            .send()
            .await
            .map_err(|e| ComfyError::Network {
                context: "Failed to fetch ComfyUI history".into(),
                source: e,
            })?;

        if !resp.status().is_success() {
            return Ok(None);
        }

        let json: Value = resp.json().await.map_err(|e| ComfyError::Network {
            context: "Failed to parse ComfyUI history response".into(),
            source: e,
        })?;

        let entry = match json.get(prompt_id) {
            Some(e) => e,
            None => return Ok(None),
        };

        let status_str = entry
            .pointer("/status/status_str")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");

        let completed = entry
            .pointer("/status/completed")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let mut images = Vec::new();
        if let Some(outputs) = entry.get("outputs").and_then(|o| o.as_object()) {
            for (_node_id, node_output) in outputs {
                if let Some(imgs) = node_output.get("images").and_then(|i| i.as_array()) {
                    for img in imgs {
                        if let Some(filename) = img.get("filename").and_then(|f| f.as_str()) {
                            let subfolder =
                                img.get("subfolder").and_then(|s| s.as_str()).unwrap_or("");
                            let img_type =
                                img.get("type").and_then(|t| t.as_str()).unwrap_or("output");
                            images.push(ImageRef {
                                filename: filename.to_string(),
                                subfolder: subfolder.to_string(),
                                img_type: img_type.to_string(),
                            });
                        }
                    }
                }
            }
        }

        Ok(Some(PromptHistory {
            status: status_str.to_string(),
            completed,
            images,
        }))
    }

    // ── Image download ──────────────────────────────────────────────

    /// Download an output image by its reference. Returns raw bytes.
    pub async fn image(&self, img: &ImageRef) -> Result<Vec<u8>> {
        let url = reqwest::Url::parse_with_params(
            &format!("{}/view", self.endpoint),
            &[
                ("filename", img.filename.as_str()),
                ("subfolder", img.subfolder.as_str()),
                ("type", img.img_type.as_str()),
            ],
        )
        .map_err(|e| ComfyError::InvalidResponse(format!("Bad image URL: {}", e)))?;

        let resp = self
            .http
            .get(url)
            .timeout(Duration::from_secs(30))
            .send()
            .await
            .map_err(|e| ComfyError::Network {
                context: format!("Failed to fetch image {} from ComfyUI", img.filename),
                source: e,
            })?;

        if !resp.status().is_success() {
            return Err(ComfyError::Http {
                status: resp.status().as_u16(),
                body: format!("Failed to fetch image {}", img.filename),
            });
        }

        let bytes = resp.bytes().await.map_err(|e| ComfyError::Network {
            context: "Failed to read image bytes".into(),
            source: e,
        })?;
        Ok(bytes.to_vec())
    }

    // ── Queue status ────────────────────────────────────────────────

    /// Get the current ComfyUI queue state (running + pending counts).
    pub async fn queue_status(&self) -> Result<QueueStatus> {
        let url = format!("{}/queue", self.endpoint);
        let resp = self
            .http
            .get(&url)
            .timeout(Duration::from_secs(5))
            .send()
            .await
            .map_err(|e| ComfyError::Network {
                context: "Failed to fetch ComfyUI queue status".into(),
                source: e,
            })?;

        let json: Value = resp.json().await.map_err(|e| ComfyError::Network {
            context: "Failed to parse ComfyUI queue response".into(),
            source: e,
        })?;

        let running = json
            .get("queue_running")
            .and_then(|v| v.as_array())
            .map(|a| a.len() as u32)
            .unwrap_or(0);
        let pending = json
            .get("queue_pending")
            .and_then(|v| v.as_array())
            .map(|a| a.len() as u32)
            .unwrap_or(0);

        Ok(QueueStatus { running, pending })
    }

    // ── Control ─────────────────────────────────────────────────────

    /// Free VRAM. If `unload_models` is true, all models are unloaded.
    pub async fn free_memory(&self, unload_models: bool) -> Result<()> {
        let url = format!("{}/free", self.endpoint);
        let body = if unload_models {
            serde_json::json!({"unload_models": true})
        } else {
            serde_json::json!({"free_memory": true})
        };
        self.http
            .post(&url)
            .timeout(Duration::from_secs(30))
            .json(&body)
            .send()
            .await
            .map_err(|e| ComfyError::Network {
                context: "Failed to send free memory request".into(),
                source: e,
            })?;
        Ok(())
    }

    /// Interrupt the currently running generation.
    pub async fn interrupt(&self) -> Result<()> {
        let url = format!("{}/interrupt", self.endpoint);
        self.http
            .post(&url)
            .timeout(Duration::from_secs(5))
            .send()
            .await
            .map_err(|e| ComfyError::Network {
                context: "Failed to send interrupt".into(),
                source: e,
            })?;
        Ok(())
    }

    // ── Image upload ─────────────────────────────────────────────────

    /// Upload an image to ComfyUI's input directory via `/upload/image`.
    /// Returns the filename as stored by ComfyUI.
    pub async fn upload_image(
        &self,
        image_bytes: &[u8],
        filename: &str,
        overwrite: bool,
    ) -> Result<String> {
        let url = format!("{}/upload/image", self.endpoint);

        let form = reqwest::multipart::Form::new()
            .part(
                "image",
                reqwest::multipart::Part::bytes(image_bytes.to_vec())
                    .file_name(filename.to_string())
                    .mime_str("image/png")
                    .map_err(|e| ComfyError::InvalidResponse(format!("Invalid MIME: {}", e)))?,
            )
            .text("type", "input")
            .text("overwrite", if overwrite { "true" } else { "false" });

        let resp = self
            .http
            .post(&url)
            .timeout(Duration::from_secs(30))
            .multipart(form)
            .send()
            .await
            .map_err(|e| ComfyError::Network {
                context: format!("Failed to upload image to ComfyUI at {}", self.endpoint),
                source: e,
            })?;

        if !resp.status().is_success() {
            let status = resp.status().as_u16();
            let body_text = resp.text().await.unwrap_or_default();
            return Err(ComfyError::Http {
                status,
                body: body_text,
            });
        }

        let json: Value = resp.json().await.map_err(|e| ComfyError::Network {
            context: "Failed to parse ComfyUI /upload/image response".into(),
            source: e,
        })?;

        json["name"]
            .as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| ComfyError::InvalidResponse("Upload response missing name field".into()))
    }

    // ── Model discovery ─────────────────────────────────────────────

    /// List available checkpoint models from ComfyUI.
    pub async fn checkpoints(&self) -> Result<Vec<String>> {
        self.object_info_list(
            "CheckpointLoaderSimple",
            "/CheckpointLoaderSimple/input/required/ckpt_name/0",
        )
        .await
    }

    /// List available sampler algorithms from ComfyUI.
    pub async fn samplers(&self) -> Result<Vec<String>> {
        self.object_info_list("KSampler", "/KSampler/input/required/sampler_name/0")
            .await
    }

    /// List available scheduler algorithms from ComfyUI.
    pub async fn schedulers(&self) -> Result<Vec<String>> {
        self.object_info_list("KSampler", "/KSampler/input/required/scheduler/0")
            .await
    }

    async fn object_info_list(&self, node: &str, pointer: &str) -> Result<Vec<String>> {
        let url = format!("{}/object_info/{}", self.endpoint, node);
        let resp = self
            .http
            .get(&url)
            .timeout(Duration::from_secs(10))
            .send()
            .await
            .map_err(|e| ComfyError::Network {
                context: format!(
                    "Cannot connect to ComfyUI at {} \u{2014} is the service running?",
                    self.endpoint
                ),
                source: e,
            })?;

        if !resp.status().is_success() {
            return Ok(Vec::new());
        }

        let json: Value = resp.json().await.map_err(|e| ComfyError::Network {
            context: format!("Failed to parse {} object_info", node),
            source: e,
        })?;

        Ok(json
            .pointer(pointer)
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default())
    }

    // ── Completion waiting ──────────────────────────────────────────

    /// Poll `/history` until the prompt completes, fails, or times out.
    pub async fn wait_for_completion(
        &self,
        prompt_id: &str,
        timeout: Duration,
    ) -> Result<GenerationOutcome> {
        self.wait_for_completion_poll(prompt_id, Duration::from_secs(2), timeout)
            .await
    }

    /// Wait for completion using ComfyUI's WebSocket for real-time step
    /// progress. Calls `on_progress` for each sampling step. Falls back
    /// to polling automatically if the WebSocket connection fails.
    pub async fn wait_for_completion_ws<F>(
        &self,
        prompt_id: &str,
        timeout: Duration,
        on_progress: F,
    ) -> Result<GenerationOutcome>
    where
        F: FnMut(ProgressUpdate),
    {
        self.wait_ws_inner(prompt_id, timeout, on_progress).await
    }

    async fn wait_for_completion_poll(
        &self,
        prompt_id: &str,
        poll_interval: Duration,
        timeout: Duration,
    ) -> Result<GenerationOutcome> {
        let start = std::time::Instant::now();
        loop {
            if start.elapsed() > timeout {
                return Ok(GenerationOutcome::TimedOut);
            }
            if let Some(history) = self.history(prompt_id).await? {
                if history.completed {
                    return Ok(GenerationOutcome::Completed {
                        images: history.images,
                    });
                } else if history.status == "error" {
                    return Ok(GenerationOutcome::Failed {
                        error: "ComfyUI generation failed".into(),
                    });
                }
            }
            tokio::time::sleep(poll_interval).await;
        }
    }

    async fn wait_ws_inner<F>(
        &self,
        prompt_id: &str,
        timeout: Duration,
        mut on_progress: F,
    ) -> Result<GenerationOutcome>
    where
        F: FnMut(ProgressUpdate),
    {
        let ws_url = format!(
            "{}/ws?clientId={}",
            self.endpoint
                .replace("http://", "ws://")
                .replace("https://", "wss://"),
            self.client_id
        );

        let (mut ws, _) = match tokio_tungstenite::connect_async(&ws_url).await {
            Ok(c) => c,
            Err(e) => {
                tracing::warn!(error = %e, "WebSocket failed, falling back to polling");
                return self
                    .wait_for_completion_poll(prompt_id, Duration::from_secs(2), timeout)
                    .await;
            }
        };

        let start = std::time::Instant::now();
        let mut our_msg_count: usize = 0;
        let mut total_msg_count: usize = 0;
        const MAX_OUR_MESSAGES: usize = 10_000;
        const MAX_TOTAL_MESSAGES: usize = 50_000;

        while let Ok(Some(msg)) = tokio::time::timeout(Duration::from_secs(30), ws.next()).await {
            total_msg_count += 1;
            if total_msg_count > MAX_TOTAL_MESSAGES {
                tracing::warn!(
                    count = MAX_TOTAL_MESSAGES,
                    "WebSocket exceeded max total messages, falling back to polling"
                );
                break;
            }
            if start.elapsed() > timeout {
                return Ok(GenerationOutcome::TimedOut);
            }

            let text = match msg {
                Ok(m) if m.is_text() => m.into_text().unwrap_or_default(),
                Ok(_) => continue,
                Err(_) => break,
            };

            let json: Value = match serde_json::from_str(&text) {
                Ok(j) => j,
                Err(_) => continue,
            };

            let msg_type = json.get("type").and_then(|v| v.as_str()).unwrap_or("");
            let data = json.get("data");
            let pid = data
                .and_then(|d| d.get("prompt_id"))
                .and_then(|v| v.as_str());

            // Skip messages for other prompts
            if pid.is_some() && pid != Some(prompt_id) {
                continue;
            }

            if pid == Some(prompt_id) {
                our_msg_count += 1;
                if our_msg_count > MAX_OUR_MESSAGES {
                    tracing::warn!(
                        prompt_id = prompt_id,
                        count = MAX_OUR_MESSAGES,
                        "Prompt exceeded max messages, falling back to polling"
                    );
                    break;
                }
            }

            match msg_type {
                "progress" => {
                    if let Some(d) = data {
                        let val = d.get("value").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
                        let max = d.get("max").and_then(|v| v.as_u64()).unwrap_or(1) as u32;
                        on_progress(ProgressUpdate {
                            current_step: val,
                            total_steps: max,
                        });
                    }
                }
                "executing"
                    if data
                        .and_then(|d| d.get("node"))
                        .map(|v| v.is_null())
                        .unwrap_or(false) =>
                {
                    // node: null in executing message means generation is done
                    return self.fetch_outcome(prompt_id).await;
                }
                "execution_error" => {
                    let err = data
                        .and_then(|d| d.get("exception_message"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("Unknown error");
                    return Ok(GenerationOutcome::Failed {
                        error: format!("ComfyUI error: {}", err),
                    });
                }
                _ => {}
            }
        }

        // WebSocket closed unexpectedly — fall back to polling
        self.wait_for_completion_poll(prompt_id, Duration::from_secs(2), timeout)
            .await
    }

    // ── Traced variants (stack-ids integration) ──────────────────────

    /// Queue a workflow for execution with optional trace context.
    ///
    /// Behaves identically to [`queue_prompt`](Self::queue_prompt) but logs
    /// the trace ID when a [`stack_ids::TraceCtx`] is provided.
    pub async fn queue_prompt_traced(
        &self,
        workflow: &Value,
        trace_ctx: Option<&stack_ids::TraceCtx>,
    ) -> Result<String> {
        if let Some(ctx) = trace_ctx {
            tracing::debug!(trace_id = %ctx.trace_id, "Queuing prompt to ComfyUI");
        }
        self.queue_prompt(workflow).await
    }

    /// Fetch prompt history with optional trace context.
    ///
    /// Behaves identically to [`history`](Self::history) but logs
    /// the trace ID when a [`stack_ids::TraceCtx`] is provided.
    pub async fn history_traced(
        &self,
        prompt_id: &str,
        trace_ctx: Option<&stack_ids::TraceCtx>,
    ) -> Result<Option<PromptHistory>> {
        if let Some(ctx) = trace_ctx {
            tracing::debug!(trace_id = %ctx.trace_id, prompt_id = %prompt_id, "Fetching history from ComfyUI");
        }
        self.history(prompt_id).await
    }

    /// Download an output image with optional trace context.
    ///
    /// Behaves identically to [`image`](Self::image) but logs
    /// the trace ID when a [`stack_ids::TraceCtx`] is provided.
    pub async fn image_traced(
        &self,
        img: &ImageRef,
        trace_ctx: Option<&stack_ids::TraceCtx>,
    ) -> Result<Vec<u8>> {
        if let Some(ctx) = trace_ctx {
            tracing::debug!(trace_id = %ctx.trace_id, filename = %img.filename, "Downloading image from ComfyUI");
        }
        self.image(img).await
    }

    /// Get queue status with optional trace context.
    ///
    /// Behaves identically to [`queue_status`](Self::queue_status) but logs
    /// the trace ID when a [`stack_ids::TraceCtx`] is provided.
    pub async fn queue_status_traced(
        &self,
        trace_ctx: Option<&stack_ids::TraceCtx>,
    ) -> Result<QueueStatus> {
        if let Some(ctx) = trace_ctx {
            tracing::debug!(trace_id = %ctx.trace_id, "Fetching queue status from ComfyUI");
        }
        self.queue_status().await
    }

    /// Upload an image with optional trace context.
    ///
    /// Behaves identically to [`upload_image`](Self::upload_image) but logs
    /// the trace ID when a [`stack_ids::TraceCtx`] is provided.
    pub async fn upload_image_traced(
        &self,
        image_bytes: &[u8],
        filename: &str,
        overwrite: bool,
        trace_ctx: Option<&stack_ids::TraceCtx>,
    ) -> Result<String> {
        if let Some(ctx) = trace_ctx {
            tracing::debug!(trace_id = %ctx.trace_id, filename = %filename, "Uploading image to ComfyUI");
        }
        self.upload_image(image_bytes, filename, overwrite).await
    }

    /// Interrupt the current generation with optional trace context.
    ///
    /// Behaves identically to [`interrupt`](Self::interrupt) but logs
    /// the trace ID when a [`stack_ids::TraceCtx`] is provided.
    pub async fn interrupt_traced(&self, trace_ctx: Option<&stack_ids::TraceCtx>) -> Result<()> {
        if let Some(ctx) = trace_ctx {
            tracing::debug!(trace_id = %ctx.trace_id, "Interrupting ComfyUI generation");
        }
        self.interrupt().await
    }

    /// Free VRAM with optional trace context.
    ///
    /// Behaves identically to [`free_memory`](Self::free_memory) but logs
    /// the trace ID when a [`stack_ids::TraceCtx`] is provided.
    pub async fn free_memory_traced(
        &self,
        unload_models: bool,
        trace_ctx: Option<&stack_ids::TraceCtx>,
    ) -> Result<()> {
        if let Some(ctx) = trace_ctx {
            tracing::debug!(trace_id = %ctx.trace_id, unload_models = %unload_models, "Freeing ComfyUI memory");
        }
        self.free_memory(unload_models).await
    }

    /// Wait for prompt completion (polling) with optional trace context.
    ///
    /// Behaves identically to [`wait_for_completion`](Self::wait_for_completion)
    /// but logs the trace ID when a [`stack_ids::TraceCtx`] is provided.
    pub async fn wait_for_completion_traced(
        &self,
        prompt_id: &str,
        timeout: Duration,
        trace_ctx: Option<&stack_ids::TraceCtx>,
    ) -> Result<GenerationOutcome> {
        if let Some(ctx) = trace_ctx {
            tracing::debug!(trace_id = %ctx.trace_id, prompt_id = %prompt_id, "Waiting for ComfyUI completion (polling)");
        }
        self.wait_for_completion(prompt_id, timeout).await
    }

    /// Wait for completion via WebSocket with optional trace context.
    ///
    /// Behaves identically to [`wait_for_completion_ws`](Self::wait_for_completion_ws)
    /// but logs the trace ID when a [`stack_ids::TraceCtx`] is provided.
    pub async fn wait_for_completion_ws_traced<F>(
        &self,
        prompt_id: &str,
        timeout: Duration,
        on_progress: F,
        trace_ctx: Option<&stack_ids::TraceCtx>,
    ) -> Result<GenerationOutcome>
    where
        F: FnMut(ProgressUpdate),
    {
        if let Some(ctx) = trace_ctx {
            tracing::debug!(trace_id = %ctx.trace_id, prompt_id = %prompt_id, "Waiting for ComfyUI completion (WebSocket)");
        }
        self.wait_for_completion_ws(prompt_id, timeout, on_progress)
            .await
    }

    async fn fetch_outcome(&self, prompt_id: &str) -> Result<GenerationOutcome> {
        match self.history(prompt_id).await? {
            Some(history) if history.completed => Ok(GenerationOutcome::Completed {
                images: history.images,
            }),
            Some(_) => Ok(GenerationOutcome::Failed {
                error: "Generation failed".into(),
            }),
            None => Ok(GenerationOutcome::Failed {
                error: "No history found after generation".into(),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_endpoint() {
        assert_eq!(
            normalize("http://localhost:8188/".into()),
            "http://localhost:8188"
        );
        assert_eq!(
            normalize("http://localhost:8188".into()),
            "http://localhost:8188"
        );
        assert_eq!(normalize("http://host:8188///".into()), "http://host:8188");
    }

    #[test]
    fn test_client_builder() {
        let client = ComfyClient::new("http://127.0.0.1:8188").with_client_id("my-app");
        assert_eq!(client.endpoint(), "http://127.0.0.1:8188");
        assert_eq!(client.client_id(), "my-app");
    }

    #[test]
    fn test_default_client_id() {
        let client = ComfyClient::new("http://localhost:8188");
        assert_eq!(client.client_id(), "comfyui-rs");
    }

    #[test]
    fn test_parse_history_response() {
        let json: Value = serde_json::from_str(
            r#"{
            "abc123": {
                "status": {"status_str": "success", "completed": true},
                "outputs": {
                    "9": {
                        "images": [
                            {"filename": "ComfyUI_00001_.png", "subfolder": "", "type": "output"}
                        ]
                    }
                }
            }
        }"#,
        )
        .unwrap();

        let entry = json.get("abc123").unwrap();
        let status = entry.pointer("/status/status_str").and_then(|v| v.as_str());
        assert_eq!(status, Some("success"));

        let completed = entry.pointer("/status/completed").and_then(|v| v.as_bool());
        assert_eq!(completed, Some(true));

        let images = entry
            .pointer("/outputs/9/images")
            .and_then(|v| v.as_array());
        assert!(images.is_some());
        assert_eq!(images.unwrap()[0]["filename"], "ComfyUI_00001_.png");
    }

    #[test]
    fn test_parse_queue_response() {
        let json: Value = serde_json::from_str(
            r#"{
            "queue_running": [["item1"]],
            "queue_pending": [["item2"], ["item3"]]
        }"#,
        )
        .unwrap();

        let running = json
            .get("queue_running")
            .and_then(|v| v.as_array())
            .map(|a| a.len())
            .unwrap_or(0);
        let pending = json
            .get("queue_pending")
            .and_then(|v| v.as_array())
            .map(|a| a.len())
            .unwrap_or(0);
        assert_eq!(running, 1);
        assert_eq!(pending, 2);
    }

    #[test]
    fn test_parse_prompt_response() {
        let json: Value = serde_json::from_str(
            r#"{
            "prompt_id": "abc-123-def",
            "number": 1,
            "node_errors": {}
        }"#,
        )
        .unwrap();

        let prompt_id = json.get("prompt_id").and_then(|v| v.as_str());
        assert_eq!(prompt_id, Some("abc-123-def"));

        let errors = json.get("node_errors").and_then(|v| v.as_object());
        assert!(errors.unwrap().is_empty());
    }

    #[test]
    fn test_parse_checkpoint_object_info() {
        let json: Value = serde_json::from_str(
            r#"{
            "CheckpointLoaderSimple": {
                "input": {
                    "required": {
                        "ckpt_name": [
                            ["dreamshaper_8.safetensors", "deliberate_v3.safetensors"]
                        ]
                    }
                }
            }
        }"#,
        )
        .unwrap();

        let checkpoints = json
            .pointer("/CheckpointLoaderSimple/input/required/ckpt_name/0")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();

        assert_eq!(checkpoints.len(), 2);
        assert_eq!(checkpoints[0], "dreamshaper_8.safetensors");
    }

    #[test]
    fn test_parse_sampler_object_info() {
        let json: Value = serde_json::from_str(
            r#"{
            "KSampler": {
                "input": {
                    "required": {
                        "sampler_name": [["euler", "dpmpp_2m", "dpmpp_sde"]],
                        "scheduler": [["normal", "karras", "exponential"]]
                    }
                }
            }
        }"#,
        )
        .unwrap();

        let samplers = json
            .pointer("/KSampler/input/required/sampler_name/0")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();

        assert_eq!(samplers.len(), 3);
        assert!(samplers.contains(&"dpmpp_2m".to_string()));
    }

    #[test]
    fn test_empty_object_info() {
        let json: Value = serde_json::from_str(r#"{}"#).unwrap();

        let checkpoints = json
            .pointer("/CheckpointLoaderSimple/input/required/ckpt_name/0")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();

        assert!(checkpoints.is_empty());
    }

    #[test]
    fn test_image_ref() {
        let img = ImageRef {
            filename: "test.png".to_string(),
            subfolder: "".to_string(),
            img_type: "output".to_string(),
        };
        assert_eq!(img.filename, "test.png");

        let json = serde_json::to_string(&img).unwrap();
        assert!(json.contains("\"filename\":\"test.png\""));
    }

    #[test]
    fn test_queue_status_serialization() {
        let status = QueueStatus {
            running: 1,
            pending: 3,
        };
        let json = serde_json::to_string(&status).unwrap();
        assert!(json.contains("\"running\":1"));
        assert!(json.contains("\"pending\":3"));
    }

    // ── Traced variant tests ────────────────────────────────────────

    /// Compile-time check that all traced methods exist with correct signatures.
    /// This async function is never called but the compiler verifies it.
    #[allow(dead_code)]
    async fn _assert_traced_signatures_compile(client: &ComfyClient) {
        let ctx = stack_ids::TraceCtx::generate();
        let workflow = serde_json::json!({});
        let img = ImageRef {
            filename: "test.png".into(),
            subfolder: "".into(),
            img_type: "output".into(),
        };

        let _ = client.queue_prompt_traced(&workflow, Some(&ctx)).await;
        let _ = client.queue_prompt_traced(&workflow, None).await;
        let _ = client.history_traced("id", Some(&ctx)).await;
        let _ = client.history_traced("id", None).await;
        let _ = client.image_traced(&img, Some(&ctx)).await;
        let _ = client.image_traced(&img, None).await;
        let _ = client.queue_status_traced(Some(&ctx)).await;
        let _ = client.queue_status_traced(None).await;
        let _ = client
            .upload_image_traced(b"png", "f.png", false, Some(&ctx))
            .await;
        let _ = client
            .upload_image_traced(b"png", "f.png", false, None)
            .await;
        let _ = client.interrupt_traced(Some(&ctx)).await;
        let _ = client.interrupt_traced(None).await;
        let _ = client.free_memory_traced(true, Some(&ctx)).await;
        let _ = client.free_memory_traced(false, None).await;
        let _ = client
            .wait_for_completion_traced("id", Duration::from_secs(10), Some(&ctx))
            .await;
        let _ = client
            .wait_for_completion_ws_traced("id", Duration::from_secs(10), |_| {}, Some(&ctx))
            .await;
    }

    #[test]
    fn test_trace_ctx_creation() {
        // Verify stack_ids::TraceCtx integrates correctly.
        let ctx = stack_ids::TraceCtx::generate();
        assert!(!ctx.trace_id.is_empty());

        let ctx2 = stack_ids::TraceCtx::from_legacy_trace_id("test-trace-123");
        assert_eq!(ctx2.trace_id, "test-trace-123");
    }
}
