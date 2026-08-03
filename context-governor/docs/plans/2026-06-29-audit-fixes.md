# Context-Governor Remaining Audit Fixes Plan

> **For Hermes:** Execute immediately — all tasks are in the adapter and Rust crate.

**Goal:** Fix all 22 remaining audit issues (issues 5-28 from the system-wide audit).

**Files touched:**
1. `~/.hermes/hermes-agent/plugins/context_engine/context_governor/__init__.py` (adapter)
2. `~/Coding/Libraries/context-governor/src/lib.rs` (Rust crate)
3. `~/.hermes/hermes-agent/tests/plugins/test_context_governor_plugin.py` (tests)
4. `~/.hermes/config.yaml` (config fix)

---

## Task 1: Config fix — set context.engine to compressor (issue 7)

The config says `engine: auto` which Hermes doesn't recognize. Set to `compressor` explicitly since the user reverted from context_governor.

## Task 2: Profile-safe paths (issue 12)

Replace `Path.home() / ".hermes" / "context-governor"` with `get_hermes_home() / "context-governor"`.

## Task 3: Fix _load_prior_session_context (issue 10)

Replace the subprocess call with a simple filesystem check — list .json files in the store dir. No subprocess needed.

## Task 4: Fix compression_count on no-op (issue 8)

Only increment compression_count if the binary actually reduced the message count.

## Task 5: Fix anti-thrashing (issue 9)

Use actual prompt_tokens from update_from_response for the before/after comparison instead of chars/4.

## Task 6: Fix _run_json stdin waste (issue 13)

Don't send stdin payload for search/expand/status commands.

## Task 7: Fix _classify_subprocess_error (issue 15)

Replace HTTP status code checks with subprocess-specific checks: binary not found, timeout, JSON parse error, non-zero exit.

## Task 8: Fix dedup hash (issue 20)

Replace Python hash() with hashlib.md5 for deterministic dedup.

## Task 9: Fix threshold_percent (issue 27)

Make threshold_percent adaptive: 0.50 for small contexts (<128K), 0.85 for large contexts.

## Task 10: Fix LLM summary prompt receipt_id (issue 26)

Inject the receipt_id into the LLM summary prompt so the model can reference it.

## Task 11: Add receipt store cleanup (issue 11)

Add a _cleanup_old_receipts method that keeps only the last N receipts (default 50).

## Task 12: Fix Rust crate tail exclusion (issue 14)

In assemble_compacted_messages, always include tail messages within protect_last_n regardless of kept_set.

## Task 13: Fix extract_file_like_tokens (issue 24)

Add more path patterns: /usr/, /etc/, /var/, /tmp/, and more extensions.

## Task 14: Add stale .tmp cleanup (issue 23)

In FileContextStore::list_receipts, also clean up .json.tmp files.

## Task 15: Run full test suite + Rust gates