# Manual Injection 03 — After Phase 4

Before allowing any search effect:

- Shadow mode must be non-authoritative.
- Raw embeddings must still be written and used.
- Shadow encode failure must not break writes unless strict mode is explicitly enabled.
- Encoded vectors must carry profile digest and checksum.
- There must be receipts/evaluation artifacts.
- No approximate score may affect user-visible ranking yet unless explicit config says so.

If any invariant fails, halt and repair. Do not continue by weakening tests.
