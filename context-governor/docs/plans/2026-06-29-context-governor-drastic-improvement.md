# Context-Governor Drastic Improvement Plan

> **For Hermes:** Use subagent-driven-development skill to implement this plan task-by-task.

**Goal:** Transform context-governor from a receipt-backed-but-broken compaction engine into one that beats the built-in ContextCompressor on quality, speed, and capability.

**Architecture:** Three codebases touched:
1. Rust crate (`/home/sikmindz/Coding/Libraries/context-governor/`) — core compaction logic
2. Python adapter (`~/.hermes/hermes-agent/plugins/context_engine/context_governor/__init__.py`) — Hermes ContextEngine wrapper
3. Python tests (`~/.hermes/hermes-agent/tests/plugins/test_context_governor_plugin.py`) — plugin tests

**Tech Stack:** Rust (blake3, serde, chrono, uuid), Python (subprocess, json), Hermes ContextEngine ABC

**Evidence-backed current state (2026-06-29):**
- Crate: 2,065 lines Rust, 20 test files, passes cargo fmt/clippy/test
- Adapter: 266 lines Python, 2 tests passing
- Built-in ContextCompressor: 2,683 lines — significantly more sophisticated
- Config: `context.engine: context_governor` active in config.yaml
- Binary: `/home/sikmindz/.local/bin/context-governor` built from debug target

---

## Phase 1: CRITICAL — Fix the broken value proposition

### Task 1.1: Expose context_expand and context_search tools to the agent

**Objective:** The summary text tells the model to "Use context_expand(receipt_id=..., item_id=...) to recover exact omitted text" but no tool exists. Wire up get_tool_schemas() and handle_tool_call().

**Files:**
- Modify: `~/.hermes/hermes-agent/plugins/context_engine/context_governor/__init__.py`
- Test: `~/.hermes/hermes-agent/tests/plugins/test_context_governor_plugin.py`

**Step 1: Write failing test**

```python
def test_context_governor_exposes_expand_and_search_tools(tmp_path):
    binary = _find_binary()
    engine = ContextGovernorEngine(binary=binary, store_dir=str(tmp_path), timeout_sec=10)
    engine.update_model(model="test", context_length=1000)
    schemas = engine.get_tool_schemas()
    names = [s["function"]["name"] for s in schemas]
    assert "context_expand" in names
    assert "context_search" in names
```

**Step 2: Run test to verify failure**
Run: `cd ~/.hermes/hermes-agent && python -m pytest tests/plugins/test_context_governor_plugin.py::test_context_governor_exposes_expand_and_search_tools -v`
Expected: FAIL — get_tool_schemas returns []

**Step 3: Implement get_tool_schemas() and handle_tool_call()**

Add to ContextGovernorEngine:

```python
def get_tool_schemas(self):
    return [
        {
            "type": "function",
            "function": {
                "name": "context_expand",
                "description": "Recover exact omitted text from a context-governor compaction receipt. Use when the compacted summary references a receipt_id and item_id that you need the full original content for.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "receipt_id": {"type": "string", "description": "The receipt ID from the compaction summary"},
                        "item_id": {"type": "string", "description": "The item ID to expand (e.g. ctxi_0001_abcdef123456)"},
                        "max_chars": {"type": "integer", "description": "Maximum characters to return", "default": 100000},
                    },
                    "required": ["receipt_id", "item_id"],
                },
            },
        },
        {
            "type": "function",
            "function": {
                "name": "context_search",
                "description": "Search across all stored context-governor compaction receipts for omitted content. Returns matching snippets from exact_store, compacted_messages, and receipts.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "query": {"type": "string", "description": "Text to search for"},
                        "top_k": {"type": "integer", "description": "Maximum results", "default": 10},
                        "scope": {"type": "string", "enum": ["all", "exact", "summary", "receipt"], "default": "all"},
                    },
                    "required": ["query"],
                },
            },
        },
        {
            "type": "function",
            "function": {
                "name": "context_status",
                "description": "Show context-governor engine status: compression count, last receipt, last error, stored receipts.",
                "parameters": {"type": "object", "properties": {}},
            },
        },
    ]

def handle_tool_call(self, name, args, **kwargs):
    import json
    try:
        if name == "context_expand":
            result = self._run_json([
                "expand", "--dir", str(self.store_dir),
                "--receipt", args["receipt_id"],
                "--item", args["item_id"],
                "--max-chars", str(args.get("max_chars", 100000)),
            ], {})
            return json.dumps(result)
        elif name == "context_search":
            scope = args.get("scope", "all")
            cmd = ["search", "--dir", str(self.store_dir), "--query", args["query"]]
            cmd.extend(["--top-k", str(args.get("top_k", 10))])
            if scope != "all":
                cmd.extend(["--scope", scope])
            result = self._run_json(cmd, {})
            return json.dumps(result)
        elif name == "context_status":
            return json.dumps(self.get_status())
        else:
            return json.dumps({"error": f"Unknown context-governor tool: {name}"})
    except Exception as exc:
        return json.dumps({"error": str(exc)})
```

**Step 4: Run test to verify pass**
Run: `cd ~/.hermes/hermes-agent && python -m pytest tests/plugins/test_context_governor_plugin.py::test_context_governor_exposes_expand_and_search_tools -v`
Expected: PASS

**Step 5: Commit**
```bash
cd ~/.hermes/hermes-agent
git add plugins/context_engine/context_governor/__init__.py tests/plugins/test_context_governor_plugin.py
git commit -m "feat: expose context_expand, context_search, context_status tools from context-governor"
```

---

### Task 1.2: Preserve OpenAI message format through roundtrip

**Objective:** _message_to_governor() currently strips tool_calls, tool_call_id, function fields. _message_from_governor() doesn't restore them. This breaks provider transport.

**Files:**
- Modify: `~/.hermes/hermes-agent/plugins/context_engine/context_governor/__init__.py`

**Step 1: Write failing test**

```python
def test_context_governor_preserves_tool_calls_and_tool_call_id(tmp_path):
    binary = _find_binary()
    engine = ContextGovernorEngine(binary=binary, store_dir=str(tmp_path), timeout_sec=10)
    engine.update_model(model="test", context_length=1000)
    messages = [
        {"role": "system", "content": "sys"},
        {"role": "user", "content": "run tests"},
        {"role": "assistant", "content": "", "tool_calls": [
            {"id": "call_abc", "type": "function", "function": {"name": "terminal", "arguments": '{"command": "ls"}'}},
        ]},
        {"role": "tool", "tool_call_id": "call_abc", "content": "file1\nfile2\n" * 200},
        {"role": "user", "content": "what files exist?"},
    ]
    compacted = engine.compress(messages, current_tokens=4000)
    # Latest user must be last
    assert compacted[-1]["role"] == "user"
    assert compacted[-1]["content"] == "what files exist?"
    # No dangling tool messages without paired assistant tool_calls
    for i, msg in enumerate(compacted):
        if msg.get("role") == "tool":
            # Must have tool_call_id
            assert msg.get("tool_call_id"), f"tool message at {i} missing tool_call_id"
    # Assistant messages with tool_calls should preserve them or be converted to text
    has_tool_calls = any(m.get("tool_calls") for m in compacted if m.get("role") == "assistant")
    # Either preserved as-is or converted to text — but not silently dropped
    # Check: no assistant message has empty content AND empty tool_calls (data loss)
    for msg in compacted:
        if msg.get("role") == "assistant":
            has_content = bool(msg.get("content"))
            has_tcs = bool(msg.get("tool_calls"))
            assert has_content or has_tcs, "assistant message lost both content and tool_calls"
```

**Step 2: Run test to verify failure**

**Step 3: Implement — change _message_to_governor to pass through OpenAI fields, and _message_from_governor to restore them**

The key insight: the Rust crate's Message struct has `metadata: BTreeMap<String, Value>`. We can serialize OpenAI-specific fields (tool_calls, tool_call_id, function) into metadata and restore them on the way back.

```python
def _message_to_governor(self, msg, idx):
    role = msg.get("role") or "assistant"
    if role not in {"system", "user", "assistant", "tool"}:
        role = "assistant"
    out = {
        "id": str(msg.get("id") or msg.get("tool_call_id") or f"m{idx}"),
        "role": role,
        "content": self._content_to_text(msg.get("content")),
    }
    if msg.get("name"):
        out["name"] = str(msg.get("name"))
    # Preserve OpenAI-specific fields in metadata for roundtrip
    metadata = {}
    if msg.get("tool_calls"):
        metadata["tool_calls"] = msg["tool_calls"]
    if msg.get("tool_call_id"):
        metadata["tool_call_id"] = msg["tool_call_id"]
    if metadata:
        out["metadata"] = metadata
    return out

@staticmethod
def _message_from_governor(msg):
    out = {"role": msg.get("role") or "assistant", "content": msg.get("content") or ""}
    if msg.get("name"):
        out["name"] = msg.get("name")
    if msg.get("id"):
        out["id"] = msg.get("id")
    # Restore OpenAI-specific fields from metadata
    metadata = msg.get("metadata") or {}
    if isinstance(metadata, dict):
        if metadata.get("tool_calls"):
            out["tool_calls"] = metadata["tool_calls"]
        if metadata.get("tool_call_id"):
            out["tool_call_id"] = metadata["tool_call_id"]
    return out
```

Also add: for messages in the compacted output that are "tool" role without a preceding assistant tool_call, convert them to assistant-role historical text:

```python
def _sanitize_dangling_tool_messages(self, messages):
    """Convert dangling tool messages to assistant text to avoid provider rejections."""
    result = []
    prev_had_tool_call = False
    for msg in messages:
        if msg.get("role") == "tool" and not prev_had_tool_call:
            # Dangling tool message — convert to assistant text
            content = msg.get("content", "")
            tool_call_id = msg.get("tool_call_id", "")
            text = f"[Tool result {tool_call_id}]: {content}" if tool_call_id else f"[Tool result]: {content}"
            result.append({"role": "assistant", "content": text})
        else:
            result.append(msg)
        # Track whether this message had tool_calls
        prev_had_tool_call = bool(msg.get("tool_calls"))
    return result
```

Call _sanitize_dangling_tool_messages in compress() before returning.

**Step 4: Run test to verify pass**

**Step 5: Commit**

---

### Task 1.3: Add anti-thrashing protection

**Objective:** Built-in compressor tracks _ineffective_compression_count and skips re-compression if last 2 passes saved <10%. Context-governor will re-compress indefinitely.

**Files:**
- Modify: `~/.hermes/hermes-agent/plugins/context_engine/context_governor/__init__.py`

**Step 1: Write failing test**

```python
def test_context_governor_anti_thrashing_skips_ineffective_compression(tmp_path):
    binary = _find_binary()
    engine = ContextGovernorEngine(binary=binary, store_dir=str(tmp_path), timeout_sec=10)
    engine.update_model(model="test", context_length=1000)
    # First compression
    messages = [
        {"role": "system", "content": "sys"},
        {"role": "user", "content": "task"},
        {"role": "assistant", "content": "ok"},
        {"role": "user", "content": "latest task here"},
    ]
    result1 = engine.compress(messages, current_tokens=900)
    assert engine.compression_count == 1
    # Simulate ineffective compression (savings < 10%)
    engine._ineffective_compression_count = 2
    should = engine.should_compress(prompt_tokens=950)
    assert should is False  # Anti-thrashing blocks
```

**Step 2: Run test to verify failure**

**Step 3: Implement**

Add to __init__:
```python
self._ineffective_compression_count = 0
self._last_compression_savings_pct = 100.0
```

Add to should_compress():
```python
def should_compress(self, prompt_tokens=None):
    tokens = int(prompt_tokens if prompt_tokens is not None else self.last_prompt_tokens or 0)
    if not self.threshold_tokens or tokens < self.threshold_tokens:
        return False
    if self._ineffective_compression_count >= 2:
        logger.warning(
            "Compression skipped — last %d compressions saved <10%% each",
            self._ineffective_compression_count,
        )
        return False
    return True
```

Add to compress() after computing result:
```python
# Track compression effectiveness
original_tokens = sum(max(1, len(self._content_to_text(m.get("content"))) // 4) for m in messages if isinstance(m, dict))
compacted_tokens = sum(max(1, len(self._content_to_text(m.get("content"))) // 4) for m in compacted if isinstance(m, dict))
savings_pct = ((original_tokens - compacted_tokens) / max(1, original_tokens)) * 100
self._last_compression_savings_pct = savings_pct
if savings_pct < 10:
    self._ineffective_compression_count += 1
else:
    self._ineffective_compression_count = 0
```

Add to on_session_reset():
```python
self._ineffective_compression_count = 0
self._last_compression_savings_pct = 100.0
```

**Step 4: Run test to verify pass**

**Step 5: Commit**

---

### Task 1.4: Make policy config-driven

**Objective:** Policy is hardcoded (semantic_memory_enabled=False, allocator=aggressive_v1, budget_mode=hard_cascade). Make it read from config.yaml.

**Files:**
- Modify: `~/.hermes/hermes-agent/plugins/context_engine/context_governor/__init__.py`

**Step 1: Write failing test**

```python
def test_context_governor_reads_policy_from_config(tmp_path, monkeypatch):
    binary = _find_binary()
    engine = ContextGovernorEngine(binary=binary, store_dir=str(tmp_path), timeout_sec=10)
    # Default policy should be safe (soft_warn, deterministic, no archive)
    assert engine._policy["budget_mode"] == "soft_warn"
    assert engine._policy["allocator"] == "deterministic_v1"
    assert engine._policy["semantic_memory_enabled"] is False
```

**Step 2: Run test to verify failure**

**Step 3: Implement**

Add to __init__:
```python
# Config-driven policy with safe defaults
self._policy = {
    "budget_mode": "soft_warn",
    "allocator": "deterministic_v1",
    "semantic_memory_enabled": False,
    "archive_memory_enabled": False,
    "summary_max_chars": 8000,
    "token_counter": "approx_chars",
}
# Override from config if available
try:
    from hermes_cli.config import load_config
    cfg = load_config()
    ctx_cfg = cfg.get("context", {}).get("governor", {})
    for key in self._policy:
        if key in ctx_cfg:
            self._policy[key] = ctx_cfg[key]
except Exception:
    pass  # Use defaults if config unavailable
```

Update compress() to use self._policy:
```python
"policy": {
    "target_tokens": self._target_tokens(current_tokens),
    "protect_first_n": self.protect_first_n,
    "protect_last_n": self.protect_last_n,
    "summary_max_chars": self._policy["summary_max_chars"],
    "allocator": self._policy["allocator"],
    "semantic_memory_enabled": self._policy["semantic_memory_enabled"],
    "archive_memory_enabled": self._policy["archive_memory_enabled"],
    "budget_mode": self._policy["budget_mode"],
    "token_counter": self._policy["token_counter"],
},
```

**Step 4: Run test to verify pass**

**Step 5: Commit**

---

### Task 1.5: Accept max_tokens in update_model

**Objective:** Built-in compressor accounts for output reservation. Context-governor ignores it.

**Files:**
- Modify: `~/.hermes/hermes-agent/plugins/context_engine/context_governor/__init__.py`

**Implementation:**

```python
def update_model(self, model, context_length, base_url="", api_key="", provider="", api_mode="", max_tokens=None):
    self.context_length = int(context_length or 0)
    self.max_tokens = int(max_tokens) if max_tokens and int(max_tokens) > 0 else None
    effective_window = self.context_length - (self.max_tokens or 0)
    if effective_window <= 0:
        effective_window = self.context_length
    self.threshold_tokens = int(effective_window * self.threshold_percent) if effective_window else 0
```

**Gate:** All Phase 1 tests pass.
```bash
cd ~/.hermes/hermes-agent && python -m pytest tests/plugins/test_context_governor_plugin.py -v
cd ~/Coding/Libraries/context-governor && cargo test --all-targets
```

---

## Phase 2: HIGH — Close the quality gap with built-in compressor

### Task 2.1: Add optional LLM summarization mode

**Objective:** Built-in compressor calls an auxiliary LLM for rich semantic summaries. Context-governor uses extractive-only 220-char previews. Add an optional LLM summary path.

**Files:**
- Modify: `~/.hermes/hermes-agent/plugins/context_engine/context_governor/__init__.py`

**Approach:** The adapter gets an optional `summary_model` and `summary_api_key`. When enabled, after the Rust crate produces the extractive summary + structured anchors, the adapter sends the structured anchors + the omitted item previews to an auxiliary LLM with a prompt like:

```
You are a context compaction assistant. The following structured anchors and omitted context previews are from a conversation that was compacted. Write a concise summary that preserves:
1. Active task and acceptance gates
2. Key decisions and their rationale
3. Errors encountered and their resolution status
4. File paths and commands used
5. Unresolved questions

Structured anchors:
{structured_summary}

Omitted context previews:
{previews}
```

The LLM output replaces the extractive summary in the compacted messages. The original extractive summary + receipt remain as fallback.

**Key design:** If the LLM call fails, fall back to the extractive summary (which is already correct). This makes the LLM path strictly additive — never worse than current behavior.

**Config:**
```yaml
context:
  engine: context_governor
  governor:
    summary_mode: llm  # or "extractive" (default)
    summary_model: ""
    summary_provider: ""
    summary_api_key: ""
```

### Task 2.2: Port tool output pruning pre-pass

**Objective:** Built-in compressor has _prune_old_tool_results() that deduplicates, summarizes, and truncates. Port equivalent logic.

**Files:**
- Modify: `~/.hermes/hermes-agent/plugins/context_engine/context_governor/__init__.py`

**Approach:** Before sending messages to the Rust binary, run a Python pre-pass that:
1. Deduplicates identical tool results (keep newest, replace older with back-reference)
2. Replaces old tool results >200 chars with 1-line summaries ("[terminal] ran `npm test` -> exit 0, 47 lines")
3. Strips image/screenshot base64 payloads from old tool messages
4. Truncates large tool_call arguments in assistant messages outside protected tail

This reduces the payload size before the Rust crate even sees it, making the subprocess faster and the classification more accurate.

### Task 2.3: Add iterative summary updates

**Objective:** Built-in compressor maintains _previous_summary and iteratively updates it. Context-governor starts fresh each time.

**Files:**
- Modify: `~/.hermes/hermes-agent/plugins/context_engine/context_governor/__init__.py`

**Approach:** Store the last compacted summary. On subsequent compressions, pass it as `focus_topic` or as a metadata field so the Rust crate can extend rather than replace. If the Rust crate doesn't support it natively, the adapter can prepend the previous summary to the new one with a "Prior compaction context:" header.

### Task 2.4: Add deferred preflight support

**Objective:** Built-in compressor has should_defer_preflight_to_real_usage(). Context-governor always uses rough chars/4.

**Files:**
- Modify: `~/.hermes/hermes-agent/plugins/context_engine/context_governor/__init__.py`

**Implementation:**

```python
def should_defer_preflight_to_real_usage(self, rough_tokens):
    if not self.threshold_tokens:
        return False
    if self._ineffective_compression_count >= 2:
        return True  # Don't re-compress from noisy estimates
    if self.last_prompt_tokens > 0 and self.last_prompt_tokens < self.threshold_tokens:
        # Real usage fit under threshold — trust it over rough estimate
        baseline = self._last_rough_tokens_when_real_fit or 0
        if baseline > 0:
            growth = max(0, rough_tokens - baseline)
            tolerated = max(4096, int(self.threshold_tokens * 0.05))
            return growth <= tolerated
    return False
```

**Gate:** All Phase 2 tests pass. Run a real compression smoke with a long synthetic transcript and verify the summary quality is comparable to built-in.

```bash
cd ~/.hermes/hermes-agent && PYTHONPATH=$PWD python - <<'PY'
from plugins.context_engine import load_context_engine
engine = load_context_engine('context_governor')
engine.update_model('test', 100000)
engine.on_session_start('smoke')
messages = [
    {'role': 'system', 'content': 'sys'},
    {'role': 'user', 'content': 'Build parser. Acceptance gate: cargo test must pass.'},
    {'role': 'assistant', 'content': 'Decision: use deterministic JSON parsing.'},
    {'role': 'tool', 'content': ('bulk log\n' * 2000) + 'error[E0425] in /src/lib.rs'},
    {'role': 'user', 'content': 'Latest task: summarize what remains.'},
]
result = engine.compress(messages, current_tokens=20000)
assert result[-1]['role'] == 'user'
assert all(m['role'] != 'tool' for m in result or True)  # tool messages may exist if sanitized
schemas = engine.get_tool_schemas()
print('tools:', [s['function']['name'] for s in schemas])
print('compression_count:', engine.compression_count)
print('receipt:', engine.last_receipt_id)
PY
```

---

## Phase 3: MEDIUM — Structural improvements

### Task 3.1: Add BM25/FTS index to FileContextStore

**Objective:** FileContextStore::search() loads every receipt JSON and searches linearly. Add a simple inverted index.

**Files:**
- Modify: `/home/sikmindz/Coding/Libraries/context-governor/src/lib.rs`
- Test: `/home/sikmindz/Coding/Libraries/context-governor/tests/store.rs`

**Approach:** Add an `IndexedFileContextStore` that maintains a token-to-receipt inverted index in memory. On `save()`, tokenize the content and update the index. On `search()`, use the index for O(1) lookup instead of loading every receipt.

This is a Rust crate change — no Hermes adapter changes needed.

### Task 3.2: Add multimodal content preservation

**Objective:** _content_to_text() converts image parts to "[image]". Preserve them in the protected tail.

**Files:**
- Modify: `~/.hermes/hermes-agent/plugins/context_engine/context_governor/__init__.py`

**Approach:** In _message_to_governor(), if content is a list with image parts, store the full content as a JSON string in metadata["original_content"]. In _message_from_governor(), if metadata has original_content, restore it. The Rust crate sees text; the adapter preserves the original for the protected tail.

### Task 3.3: Add auth/network failure discrimination

**Objective:** Built-in compressor distinguishes auth (401/403), network, and transient failures. Context-governor catches all uniformly.

**Files:**
- Modify: `~/.hermes/hermes-agent/plugins/context_engine/context_governor/__init__.py`

**Implementation:**

```python
def _classify_subprocess_error(self, exc, proc=None):
    """Classify subprocess errors for appropriate fallback behavior."""
    stderr = ""
    if proc and proc.stderr:
        stderr = proc.stderr.lower()
    elif isinstance(exc, subprocess.TimeoutExpired):
        return "timeout"
    msg = str(exc).lower()
    if "401" in msg or "403" in msg or "unauthorized" in stderr or "forbidden" in stderr:
        return "auth"
    if "connection" in msg or "timeout" in msg or "reset" in msg or "broken pipe" in msg:
        return "network"
    return "transient"
```

In compress() exception handler:
```python
except Exception as exc:
    self.last_error = str(exc)
    failure_type = self._classify_subprocess_error(exc)
    if failure_type == "auth":
        logger.error("context-governor auth failure: %s", exc)
    elif failure_type == "network":
        logger.warning("context-governor network failure (will retry next turn): %s", exc)
    else:
        logger.warning("context-governor compaction failed; keeping original: %s", exc)
    return messages
```

### Task 3.4: Wire semantic memory archive to MCP tools

**Objective:** The Rust crate has MemorySink trait and archive_response_to_memory(). The adapter has semantic_memory_enabled=False hardcoded. Wire it to the MCP semantic_memory tools.

**Files:**
- Modify: `~/.hermes/hermes-agent/plugins/context_engine/context_governor/__init__.py`

**Approach:** Implement a Python MemorySink that calls the MCP semantic_memory tools (sm_add_fact, sm_ingest_document) to archive durable/decision items. The adapter calls the Rust crate's archive path, then the Python sink writes to semantic memory.

This requires the adapter to have access to the MCP tools, which may need a callback or reference from run_agent.py. The cleanest path: the adapter emits archive records as JSON, and a hook in run_agent.py sends them to semantic memory.

**Gate:** All Phase 3 tests pass. Run cargo test + pytest + hermes doctor.

```bash
cd ~/Coding/Libraries/context-governor && cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test --all-targets
cd ~/.hermes/hermes-agent && python -m pytest tests/plugins/test_context_governor_plugin.py -v
hermes doctor
```

---

## Phase 4: LOW — Research vectors

### Task 4.1: Token-level importance scoring

**Objective:** Currently priority_score is a static integer based on ItemType. Use embedding similarity to focus_topic to score which messages matter most.

**Approach:** When focus_topic is provided, compute embedding similarity between each message and the focus. Messages with higher similarity get priority_score boost. This requires an embedding model — use the same one as semantic memory (Ollama nomic-embed or similar).

### Task 4.2: Semantic clustering before summarization

**Objective:** Currently classifies message-by-message. Cluster related messages and summarize as groups.

**Approach:** Before classification, run a simple clustering pass (e.g., TF-IDF + hierarchical clustering on message content). Messages in the same cluster get summarized together, preserving cross-message context. The structured summary would then have cluster-level sections.

### Task 4.3: Cross-session context transfer

**Objective:** Receipts persist to disk but are never loaded for new sessions. Bootstrap new sessions with relevant prior receipts.

**Approach:** On session start, search the receipt store for receipts matching the session's initial topic. Load relevant compacted content as background context. This creates a "memory" of prior conversations without loading full transcripts.

### Task 4.4: PyO3 bindings instead of subprocess

**Objective:** Replace subprocess+JSON roundtrip with native Python bindings to the Rust crate.

**Approach:** Add a `python` feature to Cargo.toml with pyo3 dependency. Create a Python extension module that wraps compact_context, context_search, context_expand. The adapter imports the module directly instead of shelling out.

This eliminates serialization overhead, enables streaming compaction, and allows holding Rust structs in memory. It's the highest-effort task but also the highest payoff for performance.

**Estimated effort:** Unknown — no historical baseline for PyO3 integration in this codebase.

**Gate:** All Phase 4 changes are experimental. Feature-flagged and tested in isolation before activating.

---

## Verification Checklist (run after each phase)

1. Rust crate gates:
```bash
cd ~/Coding/Libraries/context-governor
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test --all-targets
python3 -m pytest tests_py -q
```

2. Hermes plugin discovery:
```bash
cd ~/.hermes/hermes-agent
PYTHONPATH=$PWD python -c "
from plugins.context_engine import discover_context_engines, load_context_engine
print(discover_context_engines())
engine = load_context_engine('context_governor')
print(type(engine).__name__, engine.name, engine.is_available())
print('tools:', [s['function']['name'] for s in engine.get_tool_schemas()])
"
```

3. Fresh-process smoke:
```bash
cd ~/.hermes/hermes-agent
PYTHONPATH=$PWD python - <<'PY'
from run_agent import AIAgent
agent = AIAgent(api_key='test-key-1234567890', base_url='https://openrouter.ai/api/v1', quiet_mode=True, skip_context_files=True, skip_memory=True)
engine = agent.context_compressor
assert engine.name == 'context_governor'
assert engine.context_length > 0
print(engine.name, type(engine).__name__, engine.context_length, engine.threshold_tokens)
print('tools:', [s['function']['name'] for s in engine.get_tool_schemas()])
PY
```

4. Real compression smoke (longer transcript):
```bash
cd ~/.hermes/hermes-agent
PYTHONPATH=$PWD python - <<'PY'
from plugins.context_engine.context_governor import ContextGovernorEngine
import tempfile
with tempfile.TemporaryDirectory() as tmp:
    engine = ContextGovernorEngine(store_dir=tmp)
    engine.update_model('test', 100000)
    engine.on_session_start('smoke')
    messages = [
        {'role': 'system', 'content': 'sys'},
        {'role': 'user', 'content': 'Build parser. Acceptance gate: cargo test must pass.'},
        {'role': 'assistant', 'content': 'Decision: use deterministic JSON parsing.', 'tool_calls': [{'id': 'tc1', 'type': 'function', 'function': {'name': 'terminal', 'arguments': '{"command": "cargo test"}'}}]},
        {'role': 'tool', 'tool_call_id': 'tc1', 'content': ('bulk log\n' * 2000) + 'error[E0425] in /src/lib.rs'},
        {'role': 'user', 'content': 'Latest task: summarize what remains.'},
    ]
    result = engine.compress(messages, current_tokens=20000)
    assert result[-1]['role'] == 'user'
    assert engine.compression_count == 1
    assert engine.last_receipt_id
    print('OK:', len(messages), '->', len(result), 'messages')
    print('receipt:', engine.last_receipt_id)
    # Test tool exposure
    schemas = engine.get_tool_schemas()
    print('tools:', [s['function']['name'] for s in schemas])
    # Test expand
    import json
    status = json.loads(engine.handle_tool_call('context_status', {}))
    print('status:', status.get('compression_count'), status.get('last_receipt_id'))
PY
```