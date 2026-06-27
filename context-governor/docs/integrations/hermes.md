# Hermes integration sketch

This is an integration sketch, not a claim that Hermes currently loads this crate automatically.

Recommended adapter flow:

1. Map Hermes session messages into `context_governor::Message`.
2. When the session approaches a provider budget threshold, call `compact_context`.
3. Persist the returned `CompactResponse` with `FileContextStore` under the active Hermes profile data directory.
4. Inject `response.compacted_messages` into the next model call.
5. Expose shell/tool commands around:
   - `context-governor search --dir DIR --query TEXT`
   - `context-governor expand --dir DIR --receipt RECEIPT --item ITEM`
   - `context-governor diff < response.json`
6. Use a Hermes-side semantic-memory adapter implementing `MemorySink` if archival is enabled.

Suggested profile-local storage shape:

```text
~/.hermes/context-governor/
  receipts/
    ctxr_<uuid>.json
```

Policy defaults for Hermes:

```json
{
  "target_tokens": 8000,
  "protect_first_n": 3,
  "protect_last_n": 8,
  "summary_max_chars": 8000,
  "allocator": "deterministic_v1",
  "semantic_memory_enabled": false,
  "archive_memory_enabled": false,
  "budget_mode": "soft_warn",
  "token_counter": "approx_chars"
}
```

For autonomous Hermes use, prefer:

- `soft_warn` while validating behavior.
- `hard_cascade` once eval receipts show anchor recovery remains high.
- `fail_closed` only for strict bounded prompts where refusing is better than silent context loss.

Do not enable semantic-memory archival by pretending the core crate writes to semantic-memory. The core crate only emits `MemoryArchiveRecordV1`; the Hermes adapter must perform the actual write and return real fact/document IDs.
