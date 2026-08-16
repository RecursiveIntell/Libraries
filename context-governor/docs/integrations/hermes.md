# Hermes Integration

The canonical Hermes source tree has the discoverable Ares adapter at:

```text
~/Coding/Ares/plugins/context_engine/ri-context-governor/__init__.py
~/Coding/Ares/plugins/context_engine/_context_governor/__init__.py
```

The adapter is intentionally thin. The Rust crate owns deterministic
compaction, receipts, allocation plans, exact fallback, and CLI search/expand.
Hermes owns host-specific message shape, tool-call hygiene, latest-user
ordering, runtime status, and safe fallback behavior.

## Runtime Flow

1. Hermes selects the engine through `~/.hermes/config.yaml`:

   ```yaml
   context:
    engine: ri-context-governor
   ```

2. The adapter maps Hermes message dictionaries into `context_governor::Message`.
3. Near the model threshold, it shells out to:

   ```bash
   context-governor compact-v2 --dir ~/.hermes/context-governor \
     --governed-key-fd FD --governed-snapshot-fd FD
   ```

4. The adapter restores Hermes/OpenAI-specific fields from metadata.
5. The latest user message is reasserted as the final active instruction.
6. Tool-call/result pairs are sanitized so provider APIs do not reject compacted
   transcripts.
7. Any optional LLM replacement is sent to authenticated `finalize-v2` as
   `{candidate, compacted_messages}`. Rust authenticates the original candidate,
   rejects provenance changes, rebinds projection hashes, and re-signs.
8. `prepare-v2` writes the finalized receipt under `.pending/`; it is not yet a
   lineage tip and cannot appear in search or expansion.
9. Hermes commits the normalized projection to SessionDB, then calls
   `activate-v2` with the exact expected governor messages. An abort calls
   `discard-v2` instead.
10. After a crash, `pending-v2` returns authenticated expected messages, hashes,
    generation, and creation time so Hermes can narrowly rehydrate fields its
    DB does not round-trip and activate only a committed exact match.

V1 receipts remain readable and locally expandable. They are never selected as
recursive parents automatically; an explicit `--parent-receipt` is required to
bridge one verified V1 receipt into a V2 generation-2 lineage.

## Store

Default profile-local store:

```text
~/.hermes/context-governor/
  ctxr_<uuid>.json
  .pending/ctxr_<uuid>.json
```

Count-based pruning cannot remove a receipt still referenced by a retained V2
descendant. The crate CLI can operate directly on this directory:

```bash
context-governor search --dir ~/.hermes/context-governor --query NEEDLE \
  --governed-key-fd FD --governed-snapshot-fd FD
context-governor expand --dir ~/.hermes/context-governor --receipt ctxr_... --item ctxi_... \
  --governed-key-fd FD --governed-snapshot-fd FD
context-governor status --dir ~/.hermes/context-governor \
  --governed-key-fd FD --governed-snapshot-fd FD
```

For recursive receipts, `--item` may be a transitive `ctxs_...` source ID or a
unique legacy exact item ID. Expansion verifies every ancestor and returns
bytes from the originating receipt, never summary prose.

Governed authority means inherited `--governed-key-fd` and
`--governed-snapshot-fd` arguments, plus a
`--governed-retired-key-fd KEY_ID:FD` for each retained key. Caller-selected key
paths are rejected on every certified V2 command.

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

`summary_max_chars` limits only the prompt-visible projection. It does not
truncate authenticated source evidence or exact fallback; omitted detail stays
recoverable by receipt/source ID. A host should select the cap from its prompt
budget and fall back to the deterministic projection when an LLM exceeds it.

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
