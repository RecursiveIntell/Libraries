# Hermes Integration

Hermes has a `context_governor` context-engine adapter at:

```text
~/.hermes/hermes-agent/plugins/context_engine/context_governor/__init__.py
```

The adapter is intentionally thin. The Rust crate owns deterministic
compaction, receipts, allocation plans, exact fallback, and CLI search/expand.
Hermes owns host-specific message shape, tool-call hygiene, latest-user
ordering, runtime status, and safe fallback behavior.

## Runtime Flow

1. Hermes selects the engine through `~/.hermes/config.yaml`:

   ```yaml
   context:
     engine: context_governor
   ```

2. The adapter maps Hermes message dictionaries into `context_governor::Message`.
3. Near the model threshold, it shells out to:

   ```bash
   context-governor compact
   ```

4. The adapter restores Hermes/OpenAI-specific fields from metadata.
5. The latest user message is reasserted as the final active instruction.
6. Tool-call/result pairs are sanitized so provider APIs do not reject compacted
   transcripts.
7. The full `CompactResponse` is stored before optional LLM summary enhancement,
   so the stored receipt matches the Rust output exactly.

## Store

Default profile-local store:

```text
~/.hermes/context-governor/
  ctxr_<uuid>.json
```

The adapter prunes old receipts according to its local retention setting. The
crate CLI can still operate directly on this directory:

```bash
context-governor search --dir ~/.hermes/context-governor --query NEEDLE
context-governor expand --dir ~/.hermes/context-governor --receipt ctxr_... --item ctxi_...
context-governor status --dir ~/.hermes/context-governor
```

## Tools Exposed To Hermes

The adapter exposes:

- `context_expand(receipt_id, item_id, max_chars?)`
- `context_search(query, top_k?, scope?)`
- `context_status()`

`context_status` includes engine name, binary path, availability, store path,
last receipt, last error, compression count, token-counter status,
semantic-memory archival status, and checkpoint safety metadata.

The crate-level `status --dir` command reports receipt count, store bytes,
stale temporary receipt cleanup, and whether the in-memory index has been built.

## Policy

The Rust crate policy still travels in the normal request shape:

```json
{
  "target_tokens": 8000,
  "protect_first_n": 3,
  "protect_last_n": 20,
  "summary_max_chars": 8000,
  "allocator": "deterministic_v1",
  "semantic_memory_enabled": false,
  "archive_memory_enabled": false,
  "budget_mode": "soft_warn",
  "token_counter": "provider_chat_approx"
}
```

Recommended modes:

- `soft_warn` while validating behavior.
- `hard_cascade` when strict prompt target matters and exact fallback remains
  recoverable.
- `fail_closed` only when refusal is better than any overflow.

## Fallback Behavior

If the binary is missing, times out, or emits invalid JSON, the adapter keeps
the original messages and records `last_error`. It should not silently imply
that a context-governor receipt exists when compaction failed.

If deterministic compaction reaches a fixed point in a long-running session,
the adapter can escalate to a checkpoint summary path. That path runs a
compression-boundary audit and can fall back to extractive summary or freeze
when unsafe.

## Semantic-Memory Boundary

The core crate exposes `MemorySink` and `MemoryArchiveRecordV1`, but it does not
write to semantic memory by itself. The current Hermes adapter reports
`unsupported_no_sink` if semantic/archive memory policy knobs are enabled
without a real sink.

Do not treat `semantic_memory_enabled=true` as proof that facts were written
unless `context_status` shows a wired sink and receipts contain real external
IDs.

## Verification

Core gates:

```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test --all-targets
python3 -m pytest tests_py -q
```

Certification:

```bash
python3 scripts/certify_all.py --quick --skip-hermes
python3 scripts/certify_all.py
```

Next-level comparison/replay receipts:

```bash
python3 scripts/compare_context_engines_live.py --quick
python3 scripts/hermes_task_replay_eval.py --limit 10 --min-messages 12 --target-tokens 1200
```

The comparison markdown intentionally omits raw fixture text and records
unsupported adapters explicitly. In the 2026-07-01 run, Hermes built-in offline,
Squeez, Ogham, headroom, and LLMLingua were recorded as unsupported/not on PATH;
that is an honest receipt, not a failed context-governor result.

The historical replay markdown is aggregate/hash-only. Do not paste raw private
session anchors into public docs.

Hermes plugin tests:

```bash
cd ~/.hermes/hermes-agent
python3 -m pytest tests/plugins/test_context_governor_plugin.py -q
```
