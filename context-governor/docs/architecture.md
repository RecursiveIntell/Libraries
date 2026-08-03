# context-governor architecture

`context-governor` is a deterministic prompt-level context engine for agents.

```text
transcript
  |
  v
classify messages
  |-- authority class: active task / evidence / summary-ok / quarantine
  |-- content kind: json / diff / cargo output / shell log / markdown / rust / prose
  v
allocate budget
  |-- soft_warn: preserve safety, warn on overflow
  |-- hard_cascade: shrink/remove summary before exceeding budget
  |-- fail_closed: refuse if exact-preserve content cannot fit
  v
compact prompt
  |-- exact high-risk messages
  |-- structured anchor summary
  |-- content-aware previews
  v
receipt + fallback store
  |-- BLAKE3 transcript hashes
  |-- allocation plan
  |-- exact fallback refs
  |-- structured loss report
  |-- optional memory sink records
```

Core invariants:

1. Latest user message is active task, not historical context.
2. Summaries are reference-only background.
3. Exact fallback is retained for summarized, omitted, and quarantined records.
4. The receipt says what was kept, summarized, archived, omitted, quarantined, and recoverable.
5. Budget overflow must be explicit: warning, hard cascade, or fail-closed error.

Non-goals:

- no provider KV-cache compression
- no hosted-model context-window extension
- no learned compression claim
- no semantic-memory network calls in the core crate

Adapters own host-specific transport. The core crate owns deterministic governance.
