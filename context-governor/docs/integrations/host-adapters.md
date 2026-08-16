# Host Adapter Boundary

`context-governor` is a deterministic Rust compaction engine plus CLI. Host adapters own provider/runtime concerns.

## Stable host: Hermes

Current Ares/Hermes integration target:

- adapter paths: `~/Coding/Ares/plugins/context_engine/ri-context-governor/__init__.py` and `_context_governor/__init__.py`
- config selector: `context.engine: ri-context-governor`
- tools exposed by Hermes: `context_expand`, `context_search`, `context_status`
- safety gate: LLM summary replacement must pass `context-governor boundary-audit`; default policy is `fallback_extract`

Verification receipts for the current local integration:

- Hermes plugin/host/context-engine tests: `65 passed` via `uv run --extra dev python -m pytest tests/plugins/test_context_governor_plugin.py tests/agent/test_context_engine_host_contract.py tests/agent/test_context_engine.py -q -o 'addopts='`
- crate certification: `target/certification/20260701-031403/certification.md`

## Planned hosts: Codex, OpenCode, Claude Code wrappers

These are not implemented as first-class adapters in this crate yet. Do not claim they are shipped.

A future host wrapper should:

1. Map native transcript records into `CompactRequest.messages`.
2. Pick a host-appropriate policy and disclose approximate token counting.
3. Open the profile-owned key/snapshot while holding the lifecycle lock and pass
   only inherited governed descriptors to every certified V2 command.
4. Use `compact-v2`, then authenticated `finalize-v2` with the original signed
   candidate and the replacement projection; never construct lineage in the host.
5. Call `prepare-v2`, commit the matching native transcript, then call
   `activate-v2`. Call `discard-v2` after an aborted commit and reconcile
   authenticated `pending-v2` records after a crash.
6. Persist the full `CompactResponseV2` outside the prompt and inject only
   `compacted_messages` into the model call.
7. Expose receipt-backed `search`, `expand`, `diff`, and `status` tools with the
   same governed descriptors; keyless receipt operations are V1 inspection only.
8. Preserve provider-specific message fields through metadata if the host supports tool calls, multimodal payloads, or tool-call IDs. The Rust parent-prefix check is exact; host normalization/rehydration must happen before `compact-v2`.
9. Treat semantic-memory archival as host-owned: wire a real `MemorySink` or leave semantic IDs empty with a warning.
10. Run boundary audit before reinjecting any LLM-generated summary.

## Unsupported competitor adapters

`compare_context_engines_live.py` records unsupported competitors explicitly instead of hiding them.

As of the 2026-07-01 local run:

- Hermes built-in compressor: unsupported in offline identical-input comparison because no stable offline CLI adapter exists; benchmark it only in fresh Hermes process runs.
- Squeez: executable not found on PATH.
- Ogham: executable not found on PATH.
- headroom: executable not found on PATH.
- LLMLingua: executable not found on PATH.

This is an honest benchmark receipt. It does not prove context-governor beats those systems.

## Public claim boundary

Safe:

- deterministic local compaction
- receipts and exact fallback
- local search/expand/diff/status
- Hermes adapter verified locally
- same-transcript synthetic comparison over implemented local baselines
- aggregate/hash-only historical Hermes replay

Not safe without more receipts:

- external-engine superiority
- downstream live LLM quality preservation
- production KV-cache compression
- built-in semantic-memory writes without a real host `MemorySink`
