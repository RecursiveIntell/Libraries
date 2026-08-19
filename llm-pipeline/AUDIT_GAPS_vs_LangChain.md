# llm-pipeline → LangChain Gap Audit

**Date**: 2026-08-19
**Scope**: `/home/sikmindz/Coding/Libraries/llm-pipeline` (v0.2.1, local fork)
**Trigger**: `tencent/hy3:free` (reasoning model) returned empty `content`, breaking the agent-graph daemon. Patch applied (reasoning→content fallback + `reasoning_effort` forwarding). This audit checks for *similar latent edge cases* vs mature frameworks (LangChain).

---

## What llm-pipeline already does well (borrowed from / rivals LangChain)

- **Semantic retry** with LLM-in-the-loop correction prompt (`retry.rs`) — equivalent to LangChain `RetryWithErrorOutputParser`.
- **Cool-down schedules** (Fixed/Linear/Exponential/Adaptive) — LangChain has no native temp-cooldown, this is *better*.
- **Best-of-N parallel retry** — `RetryStrategy::BestOfN`.
- **Lossy + defensive JSON extraction** (`parsing.rs`) — `extract_json_candidate`, `find_bracketed`.
- **Thinking-block extraction** (`<think>` tags, DeepSeek R1 style).
- **Transport retry w/ backoff** (`with_backoff`, `BackoffConfig`) — honors `Retry-After`.
- **Budget + response-size enforcement** (`enforce_token_budget`, `enforce_response_size`).
- **Receipts** for auditability (`types.rs` ProviderCallReceipt).
- **Multi-backend** abstraction (Ollama, OpenAI, Anthropic, Mock, Recording).

---

## GAPS found (these are the "LangChain has it, mine lacks it" list)

### G1. [HIGH] Stream idle timeout is defined but NOT enforced
- `PipelineLimits::stream_idle_timeout` exists (limits.rs:52), `PipelineError::StreamIdle` exists (error.rs:62), but `OpenAiBackend::complete_streaming` (openai.rs:378-425) loops `while let Some(chunk) = stream.next().await` with **no idle timer**. A model that stops sending tokens (but keeps the connection open) hangs until `request_timeout` (120s default) — not 30s.
- **Fix**: track `Instant` of last token; if `now - last > stream_idle_timeout`, return `StreamIdle`. Apply to all streaming backends.

### G2. [HIGH] Streaming calls discard metadata/token_usage/finish_reason
- `complete_streaming` returns `metadata: None, token_usage: None, finish_reason: None` (openai.rs:468-473, anthropic.rs:302-307). Cost accounting + finish_reason (e.g. `content_filter`, `length`) silently break for streaming.
- **Fix**: parse `usage`/`finish_reason` from the final SSE chunk (`[DONE]` carries them in OpenAI), populate the response.

### G3. [HIGH] No `tool_calls` / function-call extraction
- `supports_tools: true` is advertised (openai.rs:489) but neither `complete` nor `complete_streaming` extract `message.tool_calls` or `delta.tool_calls`. Any agentic use expecting tool calls gets empty text.
- **Fix**: extract `tool_calls` array into a `LlmResponse` field (`tool_calls: Option<Vec<ToolCall>>`), stream-aggregate deltas. Mirror LangChain `AIMessage.tool_calls`.

### G4. [MEDIUM] Empty-content-but-also-empty-reasoning returns empty string (silent failure)
- My patch (openai.rs:325) falls back to `reasoning` when `content` is empty. But if BOTH are empty (e.g. model returned `error` object, or `choices: []`, or null `message`), `text` is `""` → daemon rejects as "no usable result" with no root cause.
- **Fix**: after fallback, if `text.trim().is_empty()`, return a typed `PipelineError::EmptyResponse { model, raw }` with the raw JSON. Surfaces root cause immediately.

### G5. [MEDIUM] No `finish_reason` handling for `content_filter` / `function_call`
- `finish_reason` is extracted (openai.rs:247) but never acted on. `content_filter` (moderation block) returns empty text → looks like failure. `tool_calls` finish (G3) is ignored.
- **Fix**: map `finish_reason` into a typed enum + error/warning (LangChain raises `OutputParserException` on content_filter).

### G6. [MEDIUM] No `response_format` validation / JSON-repair loop for non-`json_mode` callers
- `expecting_json()` uses defensive parsing (`parsing.rs`) but the *backend* doesn't repair. LangChain's `OutputFixingParser` re-calls the model with the error to fix malformed JSON. `llm-pipeline` has the retry infra (`RetryConfig`) but no JSON-repair prompt strategy wired to parsing failure.
- **Fix**: add `OutputStrategy::Json` failure → auto-trigger `RetryConfig`-style correction with the serde error (already half-built; connect the two).

### G7. [MEDIUM] Anthropic backend doesn't forward `reasoning_effort` / `thinking`
- `AnthropicBackend::build_body` (anthropic.rs:86) has no `thinking`/`reasoning_effort` handling — it ignores `config.thinking`. Claude thinking models won't activate.
- **Fix**: when `config.thinking`, add `"thinking": {"type": "enabled", "budget_tokens": N}`.

### G8. [LOW] `reasoning_effort` hardcoded to "medium"/"none" — no caller control
- `build_body` (openai.rs:147-151) hardcodes `"medium"`/`"none"`. LangChain exposes `reasoning_effort` as a param (low/medium/high). Callers can't request high-quality reasoning.
- **Fix**: add `LlmConfig::reasoning_effort: Option<ReasoningEffort>` (low/medium/high/auto) and forward it.

### G9. [LOW] No token-budget clamp warning when `max_tokens_limit` clamps
- `build_body` (openai.rs:122-124) silently clamps `max_tokens` to `max_tokens_limit`. Caller never knows their 8192 request became 2048.
- **Fix**: log/emit a `Event::TokenClamp` when clamping occurs.

### G10. [LOW] `complete` doesn't set `ttft_ms`
- `ttft_ms: None` (openai.rs:338) even for non-streaming. Minor, but receipts lose latency data.

### G11. [LOW] No `ProviderMeta::rate_limit_*` header parsing
- `ProviderMeta::from_headers_and_json` (mod.rs:115) ignores `headers` entirely (prefixed `_`). `x-ratelimit-remaining`/`retry-after` not surfaced → agents can't do smart backoff.
- **Fix**: parse `x-ratelimit-*` headers into `ProviderMeta`.

---

## Priority order for patching

1. **G1** (stream idle) — silent hangs are the worst failure mode
2. **G2** (streaming metadata loss) — breaks cost/observability
3. **G3** (tool_calls) — blocks agentic use entirely
4. **G4** (empty-response typed error) — turns silent failures into debuggable ones
5. **G5/G6** (finish_reason + JSON repair) — reliability
6. **G7–G11** — correctness/polish

## Note on upstream
All fixes are local-only (crate is external on crates.io). Consider extracting as a forked `llm-pipeline` with a patch crate or contributing upstream.
