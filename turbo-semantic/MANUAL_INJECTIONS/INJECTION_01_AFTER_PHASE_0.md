# Manual Injection 01 — After Phase 0

Revalidate before continuing:

1. Is `turbo-quant` available as the canonical crate?
2. Did you avoid absolute Cargo paths?
3. Did you avoid copying TurboQuant into semantic-memory?
4. Did `cargo metadata` succeed or did you record why not?
5. Did you identify existing semantic-memory quantization/search tests that must not regress?

If any answer is no, stop and emit a precondition failure. Otherwise proceed to TurboQuant hardening.
